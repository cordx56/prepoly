use anyhow::Context as _;
use indicatif::*;
use std::sync::{LazyLock, Mutex};

static PROGRESS: LazyLock<Mutex<Option<MultiProgress>>> = LazyLock::new(|| Mutex::new(None));
fn register_progress(message: &str, content_length: Option<u64>) -> anyhow::Result<ProgressBar> {
    let mut m = PROGRESS.lock().unwrap();
    let m = m.get_or_insert(MultiProgress::new());
    let pb = match content_length {
        Some(len) => ProgressBar::new(len),
        None => ProgressBar::new_spinner(),
    };
    let pb = m.add(pb);
    pb.set_style(progress_bar_style(content_length.is_some())?);
    pb.set_message(message.to_string());
    Ok(pb)
}
fn clear_progress() {
    *PROGRESS.lock().unwrap() = None;
}

fn progress_bar_style(has_length: bool) -> anyhow::Result<ProgressStyle> {
    use indicatif::*;
    let template = if has_length {
        "{spinner:.green} {msg:<10} [{bar:30.cyan/blue}] {bytes}/{total_bytes}"
    } else {
        "{spinner:.green} {msg:<10} {bytes}"
    };
    Ok(ProgressStyle::with_template(template)
        .context("failed to setup progress bar")?
        .progress_chars("#>-"))
}

pub async fn download(name: impl AsRef<str>, url: impl AsRef<str>) -> anyhow::Result<Vec<u8>> {
    let mut resp = reqwest::get(url.as_ref())
        .await
        .context("failed to download LLVM")?;
    // `reqwest::get` does not fail on a 4xx/5xx; check explicitly so a missing
    // asset reports the status rather than feeding an error page to the unpacker.
    if !resp.status().is_success() {
        anyhow::bail!(
            "download failed: HTTP {} for {}",
            resp.status(),
            url.as_ref()
        );
    }
    let content_length = resp.content_length();
    let progress = register_progress(name.as_ref(), content_length)?;
    // A missing or implausibly large header must not reserve that much memory
    // before the first byte arrives. The buffer grows normally beyond this
    // initial allocation as the archive is received.
    const MAX_INITIAL_CAPACITY: u64 = 16 * 1024 * 1024;
    let capacity = content_length
        .unwrap_or(0)
        .min(MAX_INITIAL_CAPACITY)
        .try_into()
        .unwrap_or(0);
    let mut data = Vec::with_capacity(capacity);
    while let Some(chunk) = resp.chunk().await.context("failed to download")? {
        data.extend_from_slice(chunk.as_ref());
        progress.set_position(data.len() as u64);
    }

    progress.finish_and_clear();
    clear_progress();
    Ok(data)
}
