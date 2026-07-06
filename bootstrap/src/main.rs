pub mod http;
mod llvm;

use std::env;
use std::os::unix::process::CommandExt;
use std::process;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let llvm_path = llvm::setup_llvm_path().await;

    let path = env::var("PATH").unwrap();
    let split = env::split_paths(&path);
    let path: Vec<_> = [llvm_path.clone().join("bin")]
        .into_iter()
        .chain(split)
        .collect();

    let args: Vec<_> = env::args().skip(1).collect();
    let mut cmd = process::Command::new(&args[0]);
    cmd.args(&args[1..])
        .env(llvm::LLVM_SYS_ENV, &llvm_path)
        .env("PATH", env::join_paths(path).unwrap());
    Err(cmd.exec())
}
