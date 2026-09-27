use chrono::NaiveDate;
use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

pub(crate) mod repo;
mod routes;
mod views;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[native_model(id = 5, version = 1)]
#[native_db]
pub(crate) struct Artist {
    #[primary_key]
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) image_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[native_model(id = 6, version = 1)]
#[native_db]
pub(crate) struct BandAssignment {
    #[primary_key]
    pub(crate) id: i64,
    pub(crate) band_id: i64,
    pub(crate) artist_id: i64,
    pub(crate) role: String,
    pub(crate) started_at: Option<NaiveDate>,
    pub(crate) ended_at: Option<NaiveDate>,
}

#[derive(Deserialize)]
struct NewArtistForm {
    name: String,
    image_url: Option<String>,
}

#[derive(Deserialize)]
struct NewAssignmentForm {
    artist_id: i64,
    role: String,
    started_at: Option<String>,
    ended_at: Option<String>,
}

pub(crate) struct ArtistAssignment {
    pub(crate) artist: Artist,
    pub(crate) assignment: BandAssignment,
}
