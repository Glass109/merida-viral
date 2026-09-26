use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

/// Shared outbound HTTP client for link previews and reverse geocoding.
///
/// The user agent identifies this app (required by Nominatim's usage policy)
/// and requests are time-boxed so a slow third party can't stall a submission.
pub(crate) fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent("merida-viral/0.1 (link preview; contact: admin@merida-viral.local)")
            .timeout(Duration::from_secs(6))
            .build()
            .expect("build HTTP client")
    })
}
