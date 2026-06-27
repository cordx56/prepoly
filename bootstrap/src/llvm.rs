use anyhow::Context as _;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tar::Archive;
use xz::read::XzDecoder;

pub const LLVM_VERSION: &str = "22.1.0";
pub const LLVM_SYS_ENV: &str = "LLVM_SYS_221_PREFIX";

pub async fn download_llvm(dest: impl AsRef<Path>) -> anyhow::Result<()> {
    let data = crate::http::download(format!("LLVM-{LLVM_VERSION}"), llvm_url()?).await?;
    let xz_dec = XzDecoder::new(data.as_slice());
    let mut untar = Archive::new(xz_dec);
    for entry in untar.entries().context("failed to unpack LLVM")? {
        let mut entry = entry.context("failed to unpack LLVM")?;
        let path = entry.path().context("failed to unpack LLVM")?;
        let path = dest.as_ref().join(PathBuf::from_iter(path.iter().skip(1)));
        fs::create_dir_all(path.parent().unwrap()).context("failed to unpack LLVM")?;
        entry.unpack(&path).context("failed to unpack LLVM")?;
    }
    Ok(())
}

fn llvm_url() -> anyhow::Result<String> {
    Ok(format!(
        "https://github.com/llvm/llvm-project/releases/download/llvmorg-22.1.0/LLVM-{}-{}-{}.tar.xz",
        LLVM_VERSION,
        get_os_repr()?,
        get_target_repr()?,
    ))
}

fn get_os_repr() -> anyhow::Result<&'static str> {
    if cfg!(target_os = "linux") {
        Ok("Linux")
    } else if cfg!(target_os = "macos") {
        Ok("macOS")
    } else {
        anyhow::bail!("unsupported OS")
    }
}

fn get_target_repr() -> anyhow::Result<&'static str> {
    if cfg!(target_arch = "aarch64") {
        Ok("ARM64")
    } else if cfg!(target_arch = "x86_64") {
        Ok("X64")
    } else {
        anyhow::bail!("unsupported arch")
    }
}
