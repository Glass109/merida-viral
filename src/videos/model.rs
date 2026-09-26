use serde::{Deserialize, Serialize};
use topcoat::router::query_params;

#[derive(Debug, Deserialize)]
pub(crate) struct Video {
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

/// The submission form: link, title and location only.
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

#[derive(Serialize)]
pub(crate) struct EmbedPreview {
    pub(crate) title: String,
    pub(crate) creator: String,
    pub(crate) thumbnail: String,
    pub(crate) platform: String,
}

#[derive(Deserialize)]
pub(crate) struct VoteInput {
    pub(crate) id: i64,
}

#[query_params(error = bad_request)]
pub(crate) struct PlaceQuery {
    pub(crate) sort: Option<String>,
}
