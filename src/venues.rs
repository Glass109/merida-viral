use std::collections::HashSet;

use native_db::{ToKey, native_db};
use native_model::{Model, native_model};
use serde::{Deserialize, Serialize};

use crate::map_pin::{MapCoordinates, MapPin};

pub(crate) mod repo;
mod routes;
mod views;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[native_model(id = 4, version = 2, from = legacy::Venue)]
#[native_db]
pub struct Venue {
    #[primary_key]
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) address: String,
    /// (longitude, latitude) in degrees.
    pub(crate) location: (f64, f64),
    pub(crate) tags: HashSet<VenueTag>,
    pub(crate) image_url: Option<String>,
}

pub(crate) mod legacy {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
    #[native_model(id = 4, version = 1)]
    #[native_db]
    pub(crate) struct Venue {
        #[primary_key]
        pub(crate) id: i64,
        pub(crate) name: String,
        pub(crate) address: String,
        pub(crate) location: (f64, f64),
        pub(crate) tags: HashSet<VenueTag>,
    }
}

impl From<legacy::Venue> for Venue {
    fn from(old: legacy::Venue) -> Self {
        Self {
            id: old.id,
            name: old.name,
            address: old.address,
            location: old.location,
            tags: old.tags,
            image_url: None,
        }
    }
}

impl From<Venue> for legacy::Venue {
    fn from(current: Venue) -> Self {
        Self {
            id: current.id,
            name: current.name,
            address: current.address,
            location: current.location,
            tags: current.tags,
        }
    }
}

impl MapCoordinates for Venue {
    fn x(&self) -> f64 {
        self.location.0
    }
    fn y(&self) -> f64 {
        self.location.1
    }
}

impl MapPin for Venue {
    fn pin_image(&self) -> Option<&str> {
        self.image_url.as_deref()
    }
    fn pin_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum VenueTag {
    PetFriendly,
    NoAlcohol,
    NoDrugs,
    LowVolume,
}

#[derive(Deserialize)]
struct NewVenueForm {
    name: String,
    address: String,
    lng: f64,
    lat: f64,
    pet_friendly: Option<String>,
    no_alcohol: Option<String>,
    no_drugs: Option<String>,
    low_volume: Option<String>,
    image_url: Option<String>,
}

impl NewVenueForm {
    fn tags(&self) -> HashSet<VenueTag> {
        let mut tags = HashSet::new();
        if self.pet_friendly.is_some() {
            tags.insert(VenueTag::PetFriendly);
        }
        if self.no_alcohol.is_some() {
            tags.insert(VenueTag::NoAlcohol);
        }
        if self.no_drugs.is_some() {
            tags.insert(VenueTag::NoDrugs);
        }
        if self.low_volume.is_some() {
            tags.insert(VenueTag::LowVolume);
        }
        tags
    }
}
