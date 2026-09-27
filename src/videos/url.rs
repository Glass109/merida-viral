use clearurls::UrlCleaner;
use std::sync::LazyLock;
use url::Url;

/// Tracking parameters that are noise on every site.
const TRACKING_PARAMS: &[&str] = &[
    "fbclid", "gclid", "dclid", "gbraid", "wbraid", "msclkid", "twclid", "yclid", "srsltid",
    "mc_cid", "mc_eid", "igshid", "igsh", "ref_src", "ref_url",
];

/// Parameters that only count as noise on the listed hosts.
const HOST_TRACKING_PARAMS: &[(&str, &[&str])] = &[(
    "tiktok.com",
    &[
        "is_from_webapp",
        "sender_device",
        "checksum",
        "share_link_id",
        "share_app_id",
        "share_iid",
        "tt_from",
    ],
)];

static CLEANER: LazyLock<UrlCleaner> =
    LazyLock::new(|| UrlCleaner::from_embedded_rules().expect("load URL cleaning rules"));

/// Return a shareable version of `raw` with tracking parameters removed.
///
/// Crowd-sourced [ClearURLs] rules do most of the work; a few well-known
/// parameters that those rules miss (notably TikTok's `is_from_webapp` and
/// `sender_device`) are removed locally. URLs that cannot be parsed are
/// returned trimmed but otherwise untouched.
///
/// [ClearURLs]: https://clearurls.xyz/
pub(crate) fn clean_video_url(raw: &str) -> String {
    let raw = raw.trim();
    let Ok(mut url) = Url::parse(raw) else {
        return raw.to_owned();
    };

    if let Ok(cleaned) = CLEANER.clear_single_url(&url) {
        url = cleaned.into_owned();
    }

    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    let kept = url
        .query_pairs()
        .filter(|(name, _)| !is_tracking_parameter(&host, name))
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();

    if kept.is_empty() {
        url.set_query(None);
    } else {
        url.query_pairs_mut().clear().extend_pairs(kept);
    }

    if url.fragment() == Some("") {
        url.set_fragment(None);
    }

    url.to_string()
}

fn is_tracking_parameter(host: &str, name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    if name.starts_with("utm_") || name.starts_with("pk_") {
        return true;
    }
    if TRACKING_PARAMS.contains(&name.as_str()) {
        return true;
    }
    HOST_TRACKING_PARAMS
        .iter()
        .any(|(base, params)| host_matches_domain(host, base) && params.contains(&name.as_str()))
}

fn host_matches_domain(host: &str, base: &str) -> bool {
    host == base || host.ends_with(&format!(".{base}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tiktok_share_parameters() {
        let raw = "https://www.tiktok.com/@n.mas/video/7686489296114748679?is_from_webapp=1&sender_device=pc";
        assert_eq!(
            clean_video_url(raw),
            "https://www.tiktok.com/@n.mas/video/7686489296114748679"
        );
    }

    #[test]
    fn strips_generic_tracking_parameters() {
        assert_eq!(
            clean_video_url("https://example.com/post?utm_source=news&utm_medium=email&fbclid=abc"),
            "https://example.com/post"
        );
    }

    #[test]
    fn keeps_meaningful_parameters() {
        assert_eq!(
            clean_video_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );
        assert_eq!(
            clean_video_url("https://example.com/search?q=merida"),
            "https://example.com/search?q=merida"
        );
    }

    #[test]
    fn applies_crowd_sourced_rules() {
        // `si` is not in the local list; ClearURLs is what removes it.
        assert_eq!(
            clean_video_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ&si=AbCdEfGhIjK"),
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );
    }

    #[test]
    fn host_rules_do_not_leak_to_other_hosts() {
        assert_eq!(
            clean_video_url("https://example.com/watch?tt_from=copy"),
            "https://example.com/watch?tt_from=copy"
        );
    }

    #[test]
    fn leaves_unparseable_input_trimmed() {
        assert_eq!(clean_video_url("  not a url  "), "not a url");
    }
}
