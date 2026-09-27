use chrono::{DateTime, Utc};
use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

pub(crate) mod repo;
mod routes;
mod views;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub(crate) struct BandMember {
    #[primary_key]
    pub(crate) id: i64,
    pub(crate) name: String,
    born_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[native_model(id = 3, version = 2, from = legacy::Band)]
#[native_db]
pub(crate) struct Band {
    #[primary_key]
    pub(crate) id: i64,
    pub(crate) name: String,
    // Retained for existing records; new memberships live in BandAssignment.
    members: Vec<BandMember>,
    pub(crate) image_url: Option<String>,
}

pub(crate) mod legacy {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[native_model(id = 3, version = 1)]
    #[native_db]
    pub(crate) struct Band {
        #[primary_key]
        pub(crate) id: i64,
        pub(crate) name: String,
        pub(crate) members: Vec<BandMember>,
    }
}

impl From<legacy::Band> for Band {
    fn from(old: legacy::Band) -> Self {
        Self {
            id: old.id,
            name: old.name,
            members: old.members,
            image_url: None,
        }
    }
}

impl From<Band> for legacy::Band {
    fn from(current: Band) -> Self {
        Self {
            id: current.id,
            name: current.name,
            members: current.members,
        }
    }
}

#[derive(Deserialize)]
struct NewBandForm {
    name: String,
    image_url: Option<String>,
}
