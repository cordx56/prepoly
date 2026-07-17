pub mod http;
mod llvm;

use anyhow::{Context as _, bail};
use std::os::unix::process::CommandExt;
use std::process;
use std::{env, ffi::OsString};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = env::args_os().skip(1);
    let Some(program) = args.next() else {
        bail!("usage: ./x <command> [arguments...]");
    };
    let llvm_path = llvm::setup_llvm_path().await?;

    let inherited_path = env::var_os("PATH").unwrap_or_default();
    let split = env::split_paths(&inherited_path);
    let path: Vec<_> = [llvm_path.clone().join("bin")]
        .into_iter()
        .chain(split)
        .collect();
    let path = env::join_paths(path).context("cannot construct PATH for the requested command")?;

    let mut cmd = process::Command::new(&program);
    cmd.args(args)
        .env(llvm::LLVM_SYS_ENV, &llvm_path)
        .env("PATH", path);
    let error = cmd.exec();
    Err(error).with_context(|| format!("failed to execute `{}`", display_arg(program)))
}

fn display_arg(arg: OsString) -> String {
    arg.to_string_lossy().into_owned()
}
