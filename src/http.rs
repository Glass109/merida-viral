use reqwest::Client;
use std::sync::LazyLock;
use std::time::Duration;

static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent("merida-underground/0.1 (link preview; contact: glas109@gmail.com)")
        .timeout(Duration::from_secs(6))
        .build()
        .expect("build HTTP client")
});

pub(crate) fn shared_http_client() -> &'static Client {
    &CLIENT
}
