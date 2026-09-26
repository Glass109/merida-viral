use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use super::http::client;

/// Reverse geocode a coordinate into a neighbourhood name.
///
/// Uses OpenStreetMap's Nominatim, which is free but asks for a clear user
/// agent and light traffic (both handled by the shared client). Results are
/// cached and any failure degrades to an empty name rather than blocking the
/// submission.
pub(crate) async fn neighborhood(lat: f64, lng: f64) -> String {
    let key = format!("{lat:.4},{lng:.4}");
    if let Some(cached) = cache_get(&key) {
        return cached;
    }
    let value = fetch(lat, lng).await.unwrap_or_default();
    cache_put(key, value.clone());
    value
}

async fn fetch(lat: f64, lng: f64) -> Option<String> {
    let lat = lat.to_string();
    let lng = lng.to_string();
    let response = client()
        .get("https://nominatim.openstreetmap.org/reverse")
        .header("Accept", "application/json")
        .query(&[
            ("format", "jsonv2"),
            ("lat", lat.as_str()),
            ("lon", lng.as_str()),
            ("zoom", "16"),
            ("addressdetails", "1"),
        ])
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = response.json::<serde_json::Value>().await.ok()?;
    let address = body.get("address")?;
    for key in [
        "neighbourhood",
        "suburb",
        "quarter",
        "city_district",
        "village",
        "hamlet",
        "town",
        "city",
    ] {
        if let Some(name) = address.get(key).and_then(|value| value.as_str()) {
            let name = name.trim();
            if !name.is_empty() {
                return Some(name.to_owned());
            }
        }
    }
    None
}

fn cache() -> &'static Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_get(key: &str) -> Option<String> {
    cache().lock().ok()?.get(key).cloned()
}

fn cache_put(key: String, value: String) {
    if let Ok(mut cache) = cache().lock() {
        if cache.len() >= 512 {
            cache.clear();
        }
        cache.insert(key, value);
    }
}
