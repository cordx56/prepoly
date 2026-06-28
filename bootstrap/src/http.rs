use anyhow::Context as _;
use indicatif::*;
use std::sync::{LazyLock, Mutex};

static PROGRESS: LazyLock<Mutex<Option<MultiProgress>>> = LazyLock::new(|| Mutex::new(None));
fn register_progress(message: &str) -> anyhow::Result<ProgressBar> {
    let mut m = PROGRESS.lock().unwrap();
    let m = m.get_or_insert(MultiProgress::new());
    let pb = m.add(ProgressBar::new(100));
    pb.set_style(progress_bar_style()?);
    pb.set_message(message.to_string());
    Ok(pb)
}
fn clear_progress() {
    *PROGRESS.lock().unwrap() = None;
}

fn progress_bar_style() -> anyhow::Result<ProgressStyle> {
    use indicatif::*;
    Ok(
        ProgressStyle::with_template("{spinner:.green} {msg:<10} [{bar:30.cyan/blue}]  {pos:>3}%")
            .context("failed to setup progress bar")?
            .progress_chars("#>-"),
    )
}

pub async fn download(name: impl AsRef<str>, url: impl AsRef<str>) -> anyhow::Result<Vec<u8>> {
    let progress = register_progress(name.as_ref())?;

    let mut resp = reqwest::get(url.as_ref())
        .await
        .context("failed to download LLVM")?;
    // `reqwest::get` does not fail on a 4xx/5xx; check explicitly so a missing
    // asset reports the status rather than feeding an error page to the unpacker.
    if !resp.status().is_success() {
        anyhow::bail!("download failed: HTTP {} for {}", resp.status(), url.as_ref());
    }
    let content_length = resp.content_length().unwrap_or(2_000_000_000) as usize;
    let mut data = Vec::with_capacity(content_length);
    while let Some(chunk) = resp.chunk().await.context("failed to download")? {
        data.extend_from_slice(chunk.as_ref());
        let pos = (100.0 * (data.len() as f32 / content_length as f32)).floor() as u64;
        progress.set_position(pos);
    }

    progress.finish_and_clear();
    clear_progress();
    Ok(data)
}
