//! Bakes the git commit of the working tree into the crate. A build outside a
//! git checkout (a source tarball, a vendored build) records `unknown` instead
//! of failing.

use std::path::Path;
use std::process::Command;

fn main() {
    // Read with `option_env!` in the library, so a channel change must recompile.
    println!("cargo::rerun-if-env-changed=BUILD_CHANNEL");

    let commit_hash = git(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    println!("cargo::rustc-env=COMMIT_HASH={commit_hash}");

    // Nothing in this package changes when HEAD moves, so without watching the
    // git metadata cargo would keep handing out the commit of the build that
    // first ran this script.
    if let Some(git_dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        let git_dir = Path::new(&git_dir);
        watch(&git_dir.join("HEAD"));
        watch(&git_dir.join("packed-refs"));
        if let Some(head_ref) = git(&["symbolic-ref", "--quiet", "HEAD"]) {
            watch(&git_dir.join(head_ref));
        }
    }
}

/// Run `git` with `args`, `None` when git is missing or the command fails --
/// not a repository, or a repository with no commit yet.
fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let out = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!out.is_empty()).then_some(out)
}

/// Re-run the script when `path` changes. A missing path is skipped: cargo
/// re-runs the script on every build when told to watch a file that is not
/// there, which would rebuild the whole compiler each time (a detached HEAD has
/// no ref file, and a fresh clone has no `packed-refs`).
fn watch(path: &Path) {
    if path.exists() {
        println!("cargo::rerun-if-changed={}", path.display());
    }
}
