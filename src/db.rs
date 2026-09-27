use std::{
    io,
    path::Path,
    sync::{
        LazyLock, OnceLock,
        atomic::{AtomicI64, Ordering},
    },
};

use native_db::{Builder, Database, Models, db_type::Error};

use crate::{
    artists::{Artist, BandAssignment},
    bands::{self, Band, BandMember},
    events::Event,
    venues::{self, Venue},
    videos::model::Video,
};

const DEFAULT_DATABASE_PATH: &str = "merida-underground.native_db";

static MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models.define::<Video>().expect("define video model");
    models
        .define::<BandMember>()
        .expect("define band member model");
    models.define::<Band>().expect("define band model");
    models
        .define::<bands::legacy::Band>()
        .expect("define original band model");
    models.define::<Venue>().expect("define venue model");
    models
        .define::<venues::legacy::Venue>()
        .expect("define original venue model");
    models.define::<Artist>().expect("define artist model");
    models
        .define::<BandAssignment>()
        .expect("define band assignment model");
    models.define::<Event>().expect("define event model");
    models
        .define::<crate::events::EventPin>()
        .expect("define event pin model");
    models
});
static DB: OnceLock<Database<'static>> = OnceLock::new();
static NEXT_VIDEO_ID: AtomicI64 = AtomicI64::new(1);
static NEXT_BAND_ID: AtomicI64 = AtomicI64::new(1);
static NEXT_VENUE_ID: AtomicI64 = AtomicI64::new(1);
static NEXT_ARTIST_ID: AtomicI64 = AtomicI64::new(1);
static NEXT_ASSIGNMENT_ID: AtomicI64 = AtomicI64::new(1);
static NEXT_EVENT_ID: AtomicI64 = AtomicI64::new(1);

pub(crate) async fn initialize_database() -> Result<(), Error> {
    let path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| DEFAULT_DATABASE_PATH.to_owned());
    let path = Path::new(&path);
    let database = if path.exists() {
        Builder::new().open(&MODELS, path)?
    } else {
        Builder::new().create(&MODELS, path)?
    };
    install(database)
}

fn install(database: Database<'static>) -> Result<(), Error> {
    let migration = database.rw_transaction()?;
    migration.migrate::<Band>()?;
    migration.migrate::<Venue>()?;
    migration.commit()?;
    let txn = database.r_transaction()?;
    let video_id = txn
        .scan()
        .primary::<Video>()?
        .all()?
        .last()
        .transpose()?
        .map_or(0, |v| v.id);
    let band_id = txn
        .scan()
        .primary::<Band>()?
        .all()?
        .last()
        .transpose()?
        .map_or(0, |b| b.id);
    let venue_id = txn
        .scan()
        .primary::<Venue>()?
        .all()?
        .last()
        .transpose()?
        .map_or(0, |v| v.id);
    let artist_id = txn
        .scan()
        .primary::<Artist>()?
        .all()?
        .last()
        .transpose()?
        .map_or(0, |a| a.id);
    let assignment_id = txn
        .scan()
        .primary::<BandAssignment>()?
        .all()?
        .last()
        .transpose()?
        .map_or(0, |a| a.id);
    let event_id = txn
        .scan()
        .primary::<Event>()?
        .all()?
        .last()
        .transpose()?
        .map_or(0, |event| event.id);
    NEXT_VIDEO_ID.store(
        video_id
            .checked_add(1)
            .ok_or_else(|| invalid_data("video IDs exhausted"))?,
        Ordering::Relaxed,
    );
    NEXT_BAND_ID.store(
        band_id
            .checked_add(1)
            .ok_or_else(|| invalid_data("band IDs exhausted"))?,
        Ordering::Relaxed,
    );
    NEXT_VENUE_ID.store(
        venue_id
            .checked_add(1)
            .ok_or_else(|| invalid_data("venue IDs exhausted"))?,
        Ordering::Relaxed,
    );
    NEXT_ARTIST_ID.store(
        artist_id
            .checked_add(1)
            .ok_or_else(|| invalid_data("artist IDs exhausted"))?,
        Ordering::Relaxed,
    );
    NEXT_ASSIGNMENT_ID.store(
        assignment_id
            .checked_add(1)
            .ok_or_else(|| invalid_data("assignment IDs exhausted"))?,
        Ordering::Relaxed,
    );
    NEXT_EVENT_ID.store(
        event_id
            .checked_add(1)
            .ok_or_else(|| invalid_data("event IDs exhausted"))?,
        Ordering::Relaxed,
    );
    drop(txn);
    DB.set(database)
        .map_err(|_| invalid_data("database initialized more than once"))
}

pub(crate) fn invalid_data(message: impl Into<String>) -> Error {
    Error::Io(io::Error::new(io::ErrorKind::InvalidData, message.into()))
}

pub(crate) async fn run_db<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, Error> + Send + 'static,
) -> Result<T, Error> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| invalid_data(format!("database task failed: {error}")))?
}

pub(crate) fn db() -> &'static Database<'static> {
    DB.get().expect("database must be initialized first")
}

fn next_id(counter: &AtomicI64, kind: &str) -> Result<i64, Error> {
    counter
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map_err(|_| invalid_data(format!("{kind} IDs exhausted")))
}

pub(crate) fn next_video_id() -> Result<i64, Error> {
    next_id(&NEXT_VIDEO_ID, "video")
}
pub(crate) fn next_band_id() -> Result<i64, Error> {
    next_id(&NEXT_BAND_ID, "band")
}
pub(crate) fn next_venue_id() -> Result<i64, Error> {
    next_id(&NEXT_VENUE_ID, "venue")
}
pub(crate) fn next_artist_id() -> Result<i64, Error> {
    next_id(&NEXT_ARTIST_ID, "artist")
}
pub(crate) fn next_assignment_id() -> Result<i64, Error> {
    next_id(&NEXT_ASSIGNMENT_ID, "assignment")
}
pub(crate) fn next_event_id() -> Result<i64, Error> {
    next_id(&NEXT_EVENT_ID, "event")
}

#[cfg(test)]
pub(crate) fn install_test_database() {
    install(Builder::new().create_in_memory(&MODELS).unwrap()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn existing_band_and_venue_rows_gain_empty_image_urls() {
        let database = Builder::new().create_in_memory(&MODELS).unwrap();
        let txn = database.rw_transaction().unwrap();
        txn.insert(bands::legacy::Band {
            id: 7,
            name: "Band".into(),
            members: Vec::new(),
        })
        .unwrap();
        txn.insert(venues::legacy::Venue {
            id: 9,
            name: "Venue".into(),
            address: "Mérida".into(),
            location: (-89.62, 20.975),
            tags: HashSet::new(),
        })
        .unwrap();
        txn.commit().unwrap();

        let txn = database.rw_transaction().unwrap();
        txn.migrate::<Band>().unwrap();
        txn.migrate::<Venue>().unwrap();
        txn.commit().unwrap();

        let txn = database.r_transaction().unwrap();
        let band = txn.get().primary::<Band>(7_i64).unwrap().unwrap();
        let venue = txn.get().primary::<Venue>(9_i64).unwrap().unwrap();
        assert_eq!(band.name, "Band");
        assert_eq!(band.image_url, None);
        assert_eq!(venue.location, (-89.62, 20.975));
        assert_eq!(venue.image_url, None);
    }
}
