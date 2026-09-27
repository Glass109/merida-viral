use std::{collections::HashMap, io, path::Path, sync::OnceLock};

use aws_sdk_s3::{
    Client,
    config::{Credentials, Region},
    error::ProvideErrorMetadata,
};
use serde::Serialize;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::{Json, multipart::Multipart},
        error::{RouterErrorExt, bad_request, not_found},
        header::{CACHE_CONTROL, CONTENT_TYPE, HeaderName, HeaderValue},
        path_param, route,
    },
};
use uuid::Uuid;

const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;

struct R2Storage {
    client: Client,
    bucket: String,
}

path_param!(kind: String);
path_param!(file: String);

#[derive(Serialize)]
struct UploadedImage {
    url: String,
}

fn allowed_kind(kind: &str) -> bool {
    matches!(kind, "artist" | "band" | "venue" | "event")
}

fn image_type(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(("jpg", "image/jpeg"))
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(("png", "image/png"))
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        Some(("webp", "image/webp"))
    } else {
        None
    }
}

fn image_file_type(file: &str) -> Option<&'static str> {
    let (id, extension) = file.rsplit_once('.')?;
    if Uuid::parse_str(id).ok()?.to_string() != id {
        return None;
    }
    match extension {
        "jpg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

pub(crate) fn valid_image_url_for(kind: &str, url: &Option<String>) -> bool {
    match url.as_deref() {
        None | Some("") => true,
        Some(url) => {
            let Some(file) = url.strip_prefix(&format!("/images/{kind}/")) else {
                return false;
            };
            allowed_kind(kind) && image_file_type(file).is_some()
        }
    }
}

pub(crate) fn normalized_image_url(url: Option<String>) -> Option<String> {
    url.filter(|value| !value.is_empty())
}

// The existing config.toml is a key=value file, not TOML. Environment values
// take precedence in production; never emit credentials in logs or responses.
fn config_value(key: &str, file: &HashMap<String, String>) -> Option<String> {
    std::env::var(key)
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| file.get(key).cloned())
}

fn storage() -> Result<&'static R2Storage> {
    static STORAGE: OnceLock<R2Storage> = OnceLock::new();
    if let Some(storage) = STORAGE.get() {
        return Ok(storage);
    }
    let source = std::fs::read_to_string(Path::new("config.toml")).unwrap_or_default();
    let file: HashMap<String, String> = source
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            Some((
                key.trim().to_owned(),
                value.trim().trim_matches(['\'', '"']).to_owned(),
            ))
        })
        .collect();
    let get = |key: &str| {
        config_value(key, &file)
            .ok_or_else(|| io::Error::other("R2 credentials are not configured"))
    };
    let endpoint = get("R2_ENDPOINT").or_else(|_| get("S3_ENDPOINT"))?;
    let access_key = get("R2_ACCESS_KEY_ID").or_else(|_| get("S3_ACCESS_KEY_ID"))?;
    let secret_key = get("R2_SECRET_ACCESS_KEY").or_else(|_| get("s3_SECRET_ACCESS_KEY"))?;
    let bucket = get("R2_BUCKET").or_else(|_| get("S3_BUCKET_NAME"))?;
    if bucket.is_empty()
        || !bucket
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(io::Error::other("invalid R2 bucket name").into());
    }
    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version_latest()
        .region(Region::new("auto"))
        .endpoint_url(endpoint)
        .credentials_provider(Credentials::new(access_key, secret_key, None, None, "R2"))
        .force_path_style(true)
        .build();
    Ok(STORAGE.get_or_init(|| R2Storage {
        client: Client::from_conf(config),
        bucket,
    }))
}

#[route(POST "/images/{kind}")]
async fn upload_image(cx: &Cx, mut multipart: Multipart) -> Result<Json<UploadedImage>> {
    let kind = path_param::<Kind>(cx).ok_or_not_found()?;
    if !allowed_kind(kind) {
        return Err(not_found().into());
    }
    let Some(field) = multipart.next_field().await? else {
        return Err(bad_request("missing picture").into());
    };
    if field.name() != Some("picture") {
        return Err(bad_request("missing picture").into());
    }
    let declared_type = field.content_type().map(str::to_owned);
    let data = field.bytes().await?;
    if data.is_empty() || data.len() > MAX_IMAGE_BYTES {
        return Err(bad_request("picture must be 2 MB or smaller").into());
    }
    let Some((extension, content_type)) = image_type(&data) else {
        return Err(bad_request("unsupported picture format").into());
    };
    if declared_type.as_deref() != Some(content_type) {
        return Err(bad_request("picture content type does not match file").into());
    }
    if multipart.next_field().await?.is_some() {
        return Err(bad_request("only one picture is allowed").into());
    }
    let file = format!("{}.{}", Uuid::new_v4(), extension);
    let key = format!("{kind}/{file}");
    let storage = storage()?;
    storage
        .client
        .put_object()
        .bucket(&storage.bucket)
        .key(&key)
        .content_type(content_type)
        .body(aws_sdk_s3::primitives::ByteStream::from(data.to_vec()))
        .send()
        .await
        .map_err(|error| {
            eprintln!(
                "R2 upload failed: {}",
                error
                    .as_service_error()
                    .and_then(|service| service.code())
                    .unwrap_or("transport error")
            );
            io::Error::other("picture upload failed")
        })?;
    Ok(Json(UploadedImage {
        url: format!("/images/{key}"),
    }))
}

#[route(GET "/images/{kind}/{file}")]
async fn get_image(cx: &Cx) -> Result<([(HeaderName, HeaderValue); 3], Vec<u8>)> {
    let kind = path_param::<Kind>(cx).ok_or_not_found()?;
    let file = path_param::<File>(cx).ok_or_not_found()?;
    if !allowed_kind(kind) {
        return Err(not_found().into());
    }
    let content_type = image_file_type(file).ok_or_else(not_found)?;
    let key = format!("{kind}/{file}");
    let storage = storage()?;
    let object = storage
        .client
        .get_object()
        .bucket(&storage.bucket)
        .key(&key)
        .send()
        .await
        .map_err(|_| not_found())?;
    if object.content_length().unwrap_or_default() > MAX_IMAGE_BYTES as i64 {
        return Err(not_found().into());
    }
    let bytes = object
        .body
        .collect()
        .await
        .map_err(|_| io::Error::other("picture download failed"))?
        .into_bytes();
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(not_found().into());
    }
    Ok((
        [
            (CONTENT_TYPE, HeaderValue::from_static(content_type)),
            (
                CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            ),
            (
                HeaderName::from_static("x-content-type-options"),
                HeaderValue::from_static("nosniff"),
            ),
        ],
        bytes.to_vec(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_recognized_images_and_keys_are_allowed() {
        assert_eq!(
            image_type(b"\x89PNG\r\n\x1a\nmore"),
            Some(("png", "image/png"))
        );
        assert_eq!(image_type(b"<svg onload=alert(1)>"), None);
        assert!(!valid_image_url_for(
            "artist",
            &Some("https://elsewhere.test/image.png".into())
        ));
        assert!(!valid_image_url_for(
            "band",
            &Some("/images/artist/00000000-0000-4000-8000-000000000000.png".into())
        ));
        assert!(valid_image_url_for(
            "band",
            &Some("/images/band/00000000-0000-4000-8000-000000000000.png".into())
        ));
    }
}
