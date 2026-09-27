use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};
use topcoat::router::query_params;

use crate::map_pin::{MapCoordinates, MapPin};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub(crate) struct Video {
    #[primary_key]
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) creator: String,
    pub(crate) platform: String,
    pub(crate) url: String,
    pub(crate) lat: f64,
    pub(crate) lng: f64,
    pub(crate) neighborhood: String,
    pub(crate) thumbnail: String,
    pub(crate) votes: i64,
}

impl MapCoordinates for Video {
    fn x(&self) -> f64 {
        self.lng
    }
    fn y(&self) -> f64 {
        self.lat
    }
}

impl MapPin for Video {
    fn pin_name(&self) -> &str {
        &self.title
    }
}

#[derive(Deserialize)]
pub(crate) struct VideoForm {
    pub(crate) url: String,
    pub(crate) title: String,
    pub(crate) lat: f64,
    pub(crate) lng: f64,
}

/// A new row with everything derived (author, platform, neighbourhood,
/// thumbnail) already resolved by the route.
pub(crate) struct NewVideo {
    pub(crate) title: String,
    pub(crate) creator: String,
    pub(crate) platform: String,
    pub(crate) url: String,
    pub(crate) lat: f64,
    pub(crate) lng: f64,
    pub(crate) neighborhood: String,
    pub(crate) thumbnail: String,
}

#[derive(Deserialize)]
pub(crate) struct EmbedRequest {
    pub(crate) url: String,
}

#[derive(Deserialize)]
pub(crate) struct VoteInput {
    pub(crate) id: i64,
}

#[query_params(error = bad_request)]
pub(crate) struct PlaceQuery {
    pub(crate) sort: Option<String>,
}
