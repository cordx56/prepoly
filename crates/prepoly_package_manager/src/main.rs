mod manifest;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};

use manifest::Manifest;

#[derive(Parser)]
#[command(name = "ppm", about = "Prepoly package manager")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create a new prepoly project.
    New {
        /// The package name.
        name: String,
    },
    /// Type-check the current package.
    Check,
    /// Run the current package.
    Run,
    /// Start the language server with package resolution.
    Lsp,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Cmd::New { name } => cmd_new(&name),
        Cmd::Check => cmd_drive("check"),
        Cmd::Run => cmd_drive("run"),
        Cmd::Lsp => cmd_lsp(),
    }
}

/// `ppm new <name>`: scaffold a project directory with source dir, root file,
/// and `package.toml`.
fn cmd_new(name: &str) -> ExitCode {
    let root = Path::new(".");

    let src_dir = root.join(name);
    if let Err(e) = std::fs::create_dir_all(&src_dir) {
        eprintln!(
            "error: cannot create source directory `{}/`: {e}",
            src_dir.display()
        );
        return ExitCode::FAILURE;
    }

    let root_file = root.join(format!("{name}.pp"));
    if !root_file.exists()
        && let Err(e) = std::fs::write(&root_file, "")
    {
        eprintln!("error: cannot write `{}`: {e}", root_file.display());
        return ExitCode::FAILURE;
    }

    let manifest_path = root.join("package.toml");
    if !manifest_path.exists() {
        let content = format!(
            "[package]\nname = \"{name}\"\nauthor = \"\"\nlicense = \"MIT\"\n\n[dependencies]\n"
        );
        if let Err(e) = std::fs::write(&manifest_path, content) {
            eprintln!("error: cannot write `{}`: {e}", manifest_path.display());
            return ExitCode::FAILURE;
        }
    }

    println!("created project `{name}`");
    ExitCode::SUCCESS
}

/// Read `package.toml`, fetch dependencies, and return the `PREPOLY_PACKAGES`
/// env-var value. Shared by `check`/`run`/`lsp`.
fn resolve_packages() -> Result<String, ExitCode> {
    let manifest = Manifest::load(Path::new("package.toml")).map_err(|e| {
        eprintln!("error: {e}");
        ExitCode::FAILURE
    })?;

    let packages_dir = packages_root().map_err(|e| {
        eprintln!("error: {e}");
        ExitCode::FAILURE
    })?;

    let mut pkg_entries: BTreeMap<String, PathBuf> = BTreeMap::new();
    for (name, dep) in &manifest.dependencies {
        let dir_name = format!("{name}-git-{}", &dep.hash);
        let dest = packages_dir.join(&dir_name);
        if !dest.exists() {
            eprintln!(
                "fetching {name} ({})...",
                &dep.hash[..dep.hash.len().min(8)]
            );
            let status = Command::new("git")
                .args(["clone", &dep.git, &dest.display().to_string()])
                .status();
            match status {
                Ok(s) if s.success() => {}
                Ok(s) => {
                    eprintln!(
                        "error: git clone failed for `{name}` (exit {})",
                        s.code().unwrap_or(-1)
                    );
                    return Err(ExitCode::FAILURE);
                }
                Err(e) => {
                    eprintln!("error: cannot run git: {e}");
                    return Err(ExitCode::FAILURE);
                }
            }
            let status = Command::new("git")
                .args(["-C", &dest.display().to_string(), "checkout", &dep.hash])
                .status();
            match status {
                Ok(s) if s.success() => {}
                Ok(_) => {
                    eprintln!("error: git checkout `{}` failed for `{name}`", dep.hash);
                    return Err(ExitCode::FAILURE);
                }
                Err(e) => {
                    eprintln!("error: cannot run git: {e}");
                    return Err(ExitCode::FAILURE);
                }
            }
        }
        pkg_entries.insert(name.clone(), dest);
    }

    Ok(pkg_entries
        .iter()
        .map(|(name, path)| format!("{name}={}", path.display()))
        .collect::<Vec<_>>()
        .join(":"))
}

/// Read `package.toml` in the current directory, fetch dependencies, set
/// `PREPOLY_PACKAGES`, and invoke `prepoly check` or `prepoly run` on the
/// root file.
fn cmd_drive(mode: &str) -> ExitCode {
    let manifest = match Manifest::load(Path::new("package.toml")) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let env_val = match resolve_packages() {
        Ok(v) => v,
        Err(code) => return code,
    };

    let root_file = format!("{}.pp", manifest.package.name);

    let mut cmd = Command::new("prepoly");
    if mode == "check" {
        cmd.arg("check");
    }
    cmd.arg(&root_file);
    if !env_val.is_empty() {
        cmd.env("PREPOLY_PACKAGES", &env_val);
    }

    match cmd.status() {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(s) => {
            let code: u8 = s.code().unwrap_or(1) as u8;
            ExitCode::from(code)
        }
        Err(e) => {
            eprintln!("error: cannot run prepoly: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Start `prepoly-lsp` with `PREPOLY_PACKAGES` set.
fn cmd_lsp() -> ExitCode {
    let env_val = match resolve_packages() {
        Ok(v) => v,
        Err(code) => return code,
    };

    let mut cmd = Command::new("prepoly-lsp");
    if !env_val.is_empty() {
        cmd.env("PREPOLY_PACKAGES", &env_val);
    }

    match cmd.status() {
        Ok(s) if s.success() => ExitCode::SUCCESS,
        Ok(s) => {
            let code: u8 = s.code().unwrap_or(1) as u8;
            ExitCode::from(code)
        }
        Err(e) => {
            eprintln!("error: cannot run prepoly-lsp: {e}");
            ExitCode::FAILURE
        }
    }
}

/// `$HOME/.prepoly/packages/`, created on demand.
fn packages_root() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let dir = PathBuf::from(home).join(".prepoly").join("packages");
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    Ok(dir)
}
