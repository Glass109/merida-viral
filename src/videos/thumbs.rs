use std::path::PathBuf;

use super::http::client;

/// Cap on a downloaded thumbnail; anything larger is ignored.
const MAX_BYTES: usize = 6 * 1024 * 1024;
const THUMB_DIR: &str = "public/thumbs";

/// Download a remote thumbnail and store it under `public/thumbs` so the
/// preview keeps working after signed CDN links expire.
///
/// Returns the local public path (e.g. `/thumbs/12.jpg`), or `None` when the
/// thumbnail is missing or the download fails.
pub(crate) async fn cache(video_id: i64, remote: &str) -> Option<String> {
    if remote.is_empty() {
        return None;
    }
    let url = url::Url::parse(remote).ok()?;
    if url.scheme() != "https" {
        return None;
    }
    let response = client().get(url).send().await.ok()?;
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|len| len > MAX_BYTES as u64)
    {
        return None;
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let bytes = response.bytes().await.ok()?;
    if bytes.is_empty() || bytes.len() > MAX_BYTES {
        return None;
    }

    let extension = match content_type.split(';').next().unwrap_or_default().trim() {
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "jpg",
    };
    let directory = PathBuf::from(THUMB_DIR);
    std::fs::create_dir_all(&directory).ok()?;
    let filename = format!("{video_id}.{extension}");
    std::fs::write(directory.join(&filename), &bytes).ok()?;
    Some(format!("/thumbs/{filename}"))
}
