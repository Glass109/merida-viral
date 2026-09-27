use chrono::{DateTime, Utc};
use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

use crate::{bands::Band, venues::Venue};

mod repo;
mod routes;
mod views;

/// Venue and band references are IDs, rather than snapshots of their names.
/// The order of `band_ids` is the order chosen for the lineup.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[native_model(id = 7, version = 1)]
#[native_db]
pub(crate) struct Event {
    #[primary_key]
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) venue_id: i64,
    pub(crate) band_ids: Vec<i64>,
    pub(crate) starts_at: DateTime<Utc>,
    pub(crate) ends_at: Option<DateTime<Utc>>,
    pub(crate) image_url: Option<String>,
}

impl Event {
    pub(crate) fn is_past(&self, now: DateTime<Utc>) -> bool {
        self.ends_at.as_ref().unwrap_or(&self.starts_at) < &now
    }
}

pub(crate) struct EventListing {
    pub(crate) event: Event,
    pub(crate) venue: Venue,
    pub(crate) bands: Vec<Band>,
    pub(crate) pin_style: crate::map_pin::PinStyle,
}

// Separate presentation record preserves existing Event v1 data unchanged.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[native_model(id = 8, version = 1)]
#[native_db]
pub(crate) struct EventPin {
    #[primary_key]
    pub(crate) event_id: i64,
    pub(crate) style: crate::map_pin::PinStyle,
}

impl crate::map_pin::MapCoordinates for EventListing {
    fn x(&self) -> f64 {
        self.venue.location.0
    }
    fn y(&self) -> f64 {
        self.venue.location.1
    }
}
impl crate::map_pin::MapPin for EventListing {
    fn pin_name(&self) -> &str {
        &self.event.title
    }
    fn pin_style(&self) -> crate::map_pin::PinStyle {
        self.pin_style
    }
    fn pin_image(&self) -> Option<&str> {
        self.event
            .image_url
            .as_deref()
            .or(self.venue.image_url.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn ongoing_events_are_not_past_until_they_end() {
        let now = Utc::now();
        let mut event = Event {
            id: 1,
            title: "Show".into(),
            description: String::new(),
            venue_id: 1,
            band_ids: vec![1],
            starts_at: now - Duration::hours(1),
            ends_at: Some(now + Duration::hours(1)),
            image_url: None,
        };
        assert!(!event.is_past(now));
        event.ends_at = Some(now - Duration::minutes(1));
        assert!(event.is_past(now));
        event.ends_at = None;
        assert!(event.is_past(now));
    }
}
