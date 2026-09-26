use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use regex::Regex;
use serde::Serialize;
use url::Url;

use super::http::client;

/// How long a fetched preview is reused. Keeps the live preview and the
/// submission from hitting TikTok/YouTube twice for the same link.
const CACHE_TTL: Duration = Duration::from_secs(600);
const CACHE_MAX: usize = 256;

/// Title, author, thumbnail and platform read from a video link.
#[derive(Clone, Debug, Default, Serialize)]
pub(crate) struct EmbedData {
    pub(crate) title: String,
    pub(crate) creator: String,
    pub(crate) thumbnail: String,
    pub(crate) platform: String,
}

/// Map a link to the platform label the UI already uses.
pub(crate) fn platform_for(url: &Url) -> &'static str {
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if host_matches(&host, "tiktok.com") {
        "TikTok"
    } else if host_matches(&host, "instagram.com") {
        if url.path().starts_with("/reel") {
            "Reels"
        } else {
            "Instagram"
        }
    } else if host_matches(&host, "youtube.com") || host_matches(&host, "youtu.be") {
        "YouTube"
    } else {
        "Other"
    }
}

/// Read title/author/thumbnail/platform for a video link.
///
/// Only known video hosts are fetched (to avoid turning the server into an
/// open proxy). Provider oEmbed is tried first because it is stable JSON;
/// Instagram has no open oEmbed, so Open Graph tags are scraped as a fallback.
pub(crate) async fn lookup(raw_url: &str) -> EmbedData {
    let Ok(url) = Url::parse(raw_url) else {
        return EmbedData::default();
    };
    let mut data = EmbedData {
        platform: platform_for(&url).to_owned(),
        ..EmbedData::default()
    };
    if !supported(&url) {
        return data;
    }
    if let Some(cached) = cache_get(raw_url) {
        return cached;
    }

    if let Some(json) = fetch_oembed(&url).await {
        apply_oembed(&mut data, &json);
    }
    if data.title.is_empty() || data.thumbnail.is_empty() || data.creator.is_empty() {
        if let Some(html) = fetch_html(raw_url).await {
            apply_open_graph(&mut data, &html);
        }
    }

    cache_put(raw_url, &data);
    data
}

fn host_matches(host: &str, base: &str) -> bool {
    host == base || host.ends_with(&format!(".{base}"))
}

fn supported(url: &Url) -> bool {
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    ["tiktok.com", "instagram.com", "youtube.com", "youtu.be"]
        .iter()
        .any(|base| host_matches(&host, base))
}

async fn fetch_oembed(url: &Url) -> Option<serde_json::Value> {
    let host = url.host_str()?.to_ascii_lowercase();
    let endpoint = if host_matches(&host, "tiktok.com") {
        "https://www.tiktok.com/oembed"
    } else if host_matches(&host, "youtube.com") || host_matches(&host, "youtu.be") {
        "https://www.youtube.com/oembed"
    } else {
        return None;
    };

    let mut request = client().get(endpoint).query(&[("url", url.as_str())]);
    if endpoint.contains("youtube") {
        request = request.query(&[("format", "json")]);
    }
    let response = request.send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let text = response.text().await.ok()?;
    serde_json::from_str(&text).ok()
}

async fn fetch_html(url: &str) -> Option<String> {
    let response = client()
        .get(url)
        .header("Accept", "text/html,application/xhtml+xml")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = response.text().await.ok()?;
    (body.len() <= 4_000_000).then_some(body)
}

fn apply_oembed(data: &mut EmbedData, json: &serde_json::Value) {
    assign_once(&mut data.title, json.get("title").and_then(|v| v.as_str()));
    assign_once(
        &mut data.creator,
        json.get("author_name").and_then(|v| v.as_str()),
    );
    assign_once(
        &mut data.thumbnail,
        json.get("thumbnail_url").and_then(|v| v.as_str()),
    );
}

fn apply_open_graph(data: &mut EmbedData, html: &str) {
    let meta = meta_tags(html);
    if data.title.is_empty() {
        data.title = meta
            .get("og:title")
            .or_else(|| meta.get("twitter:title"))
            .cloned()
            .or_else(|| document_title(html))
            .unwrap_or_default();
    }
    if data.thumbnail.is_empty() {
        data.thumbnail = meta
            .get("og:image")
            .or_else(|| meta.get("twitter:image"))
            .cloned()
            .unwrap_or_default();
    }
    if data.creator.is_empty() {
        data.creator = meta
            .get("author")
            .or_else(|| meta.get("og:article:author"))
            .cloned()
            .unwrap_or_default();
    }
}

fn assign_once(target: &mut String, value: Option<&str>) {
    if target.is_empty() {
        if let Some(value) = value {
            *target = value.trim().to_owned();
        }
    }
}

/// Collect `<meta>` tags as `key -> content`, tolerating either attribute order.
fn meta_tags(html: &str) -> HashMap<String, String> {
    static META: OnceLock<Regex> = OnceLock::new();
    static ATTR: OnceLock<Regex> = OnceLock::new();
    let tag_re = META.get_or_init(|| Regex::new(r"(?is)<meta\b[^>]*>").expect("meta regex"));
    let attr_re = ATTR.get_or_init(|| {
        Regex::new(r#"(?is)([a-zA-Z_:][-a-zA-Z0-9_:.]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+))"#)
            .expect("attribute regex")
    });

    let mut out = HashMap::new();
    for tag in tag_re.find_iter(html).map(|m| m.as_str()) {
        let mut attrs = HashMap::new();
        for cap in attr_re.captures_iter(tag) {
            let key = cap[1].to_ascii_lowercase();
            let value = cap
                .get(2)
                .or_else(|| cap.get(3))
                .or_else(|| cap.get(4))
                .map(|m| m.as_str().to_owned())
                .unwrap_or_default();
            attrs.insert(key, value);
        }
        let key = attrs
            .get("property")
            .or_else(|| attrs.get("name"))
            .map(|k| k.to_ascii_lowercase())
            .unwrap_or_default();
        if key.is_empty() {
            continue;
        }
        if let Some(content) = attrs.get("content") {
            out.entry(key).or_insert_with(|| decode_entities(content));
        }
    }
    out
}

fn document_title(html: &str) -> Option<String> {
    static TITLE: OnceLock<Regex> = OnceLock::new();
    let re =
        TITLE.get_or_init(|| Regex::new(r"(?is)<title[^>]*>(.*?)</title>").expect("title regex"));
    re.captures(html)
        .and_then(|cap| cap.get(1))
        .map(|m| decode_entities(m.as_str().trim()))
        .filter(|title| !title.is_empty())
}

fn decode_entities(input: &str) -> String {
    if !input.contains('&') {
        return input.to_owned();
    }
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(index) = rest.find('&') {
        out.push_str(&rest[..index]);
        rest = &rest[index..];
        if let Some(end) = rest.find(';').filter(|end| *end <= 10) {
            if let Some(decoded) = decode_entity(&rest[1..end]) {
                out.push(decoded);
                rest = &rest[end + 1..];
                continue;
            }
        }
        out.push('&');
        rest = &rest[1..];
    }
    out.push_str(rest);
    out
}

fn decode_entity(entity: &str) -> Option<char> {
    match entity {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        "nbsp" => Some(' '),
        _ => {
            let code = entity.strip_prefix('#')?;
            let value = if let Some(hex) = code.strip_prefix(['x', 'X']) {
                u32::from_str_radix(hex, 16).ok()?
            } else {
                code.parse::<u32>().ok()?
            };
            char::from_u32(value)
        }
    }
}

type Cache = Mutex<HashMap<String, (Instant, EmbedData)>>;

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_get(url: &str) -> Option<EmbedData> {
    let mut cache = cache().lock().ok()?;
    match cache.get(url) {
        Some((stored, data)) if stored.elapsed() < CACHE_TTL => Some(data.clone()),
        Some(_) => {
            cache.remove(url);
            None
        }
        None => None,
    }
}

fn cache_put(url: &str, data: &EmbedData) {
    if let Ok(mut cache) = cache().lock() {
        if cache.len() >= CACHE_MAX {
            cache.retain(|_, (stored, _)| stored.elapsed() < CACHE_TTL);
        }
        cache.insert(url.to_owned(), (Instant::now(), data.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_platforms_from_hosts() {
        let platform = |url: &str| platform_for(&Url::parse(url).expect("valid url"));
        assert_eq!(platform("https://www.tiktok.com/@a/video/1"), "TikTok");
        assert_eq!(platform("https://www.instagram.com/reel/abc/"), "Reels");
        assert_eq!(platform("https://www.instagram.com/p/abc/"), "Instagram");
        assert_eq!(platform("https://youtu.be/abc"), "YouTube");
        assert_eq!(platform("https://www.youtube.com/watch?v=abc"), "YouTube");
        assert_eq!(platform("https://example.com/video"), "Other");
    }

    #[test]
    fn parses_open_graph_in_either_attribute_order() {
        let html = r#"
            <html><head>
            <meta content="A pink house" property="og:title">
            <meta property='og:image' content='https://cdn.example.com/a.jpg' />
            <meta name="author" content="&#64;vale &amp; friends">
            </head></html>
        "#;
        let meta = meta_tags(html);
        assert_eq!(
            meta.get("og:title").map(String::as_str),
            Some("A pink house")
        );
        assert_eq!(
            meta.get("og:image").map(String::as_str),
            Some("https://cdn.example.com/a.jpg")
        );
        assert_eq!(
            meta.get("author").map(String::as_str),
            Some("@vale & friends")
        );
    }

    #[test]
    fn falls_back_to_document_title() {
        let html = "<title>A slow morning &mdash; Mérida</title>";
        assert_eq!(
            document_title(html).as_deref(),
            Some("A slow morning &mdash; Mérida")
        );
    }

    #[test]
    fn skips_unknown_hosts() {
        assert!(!supported(
            &Url::parse("https://example.com/v/1").expect("url")
        ));
        assert!(supported(
            &Url::parse("https://www.tiktok.com/@a/video/1").expect("url")
        ));
    }
}
