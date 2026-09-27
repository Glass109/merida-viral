use cached::macros::cached;

use crate::http::shared_http_client;

/// Latitude and longitude in degrees, in that order. MapLibre uses the
/// reverse order ([longitude, latitude]) when positioning a marker.
pub(crate) fn valid_coordinates(lat: f64, lng: f64) -> bool {
    (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng)
}

/// Reverse geocode a coordinate into a neighbourhood name.
///
/// Uses OpenStreetMap's Nominatim, which is free but asks for a clear user
/// agent and light traffic (both handled by the shared client). Results are
/// cached and any failure degrades to an empty name rather than blocking the
/// submission.
pub(crate) async fn neighborhood_for_coordinates(lat: f64, lng: f64) -> String {
    if !valid_coordinates(lat, lng) {
        return String::new();
    }
    fetch_neighborhood_from_nominatim(lat, lng)
        .await
        .unwrap_or_default()
}

// Reuse nearby coordinates for an hour; don't cache failed or empty lookups.
#[cached(
    max_size = 512,
    ttl_secs = 3600,
    key = "String",
    convert = { format!("{lat:.4},{lng:.4}") },
    sync_writes = "by_key"
)]
async fn fetch_neighborhood_from_nominatim(lat: f64, lng: f64) -> Option<String> {
    let lat = lat.to_string();
    let lng = lng.to_string();
    let response = shared_http_client()
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

#[cfg(test)]
mod tests {
    use super::valid_coordinates;

    #[test]
    fn coordinates_must_be_finite_and_within_latitude_longitude_bounds() {
        assert!(valid_coordinates(20.975, -89.62));
        assert!(valid_coordinates(-90.0, 180.0));
        assert!(!valid_coordinates(90.01, 0.0));
        assert!(!valid_coordinates(0.0, -180.01));
        assert!(!valid_coordinates(f64::NAN, 0.0));
        assert!(!valid_coordinates(0.0, f64::INFINITY));
    }
}
