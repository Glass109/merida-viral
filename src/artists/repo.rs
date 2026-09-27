use native_db::db_type::Error;

use super::{Artist, ArtistAssignment, BandAssignment};
use crate::{
    bands::Band,
    db::{db, next_artist_id, next_assignment_id, run_db},
};

pub(super) async fn list() -> Result<Vec<Artist>, Error> {
    run_db(|| {
        let txn = db().r_transaction()?;
        let mut artists: Vec<Artist> = txn.scan().primary()?.all()?.collect::<Result<_, _>>()?;
        artists.sort_unstable_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(artists)
    })
    .await
}

pub(super) async fn create(name: String, image_url: Option<String>) -> Result<i64, Error> {
    run_db(move || {
        let id = next_artist_id()?;
        let txn = db().rw_transaction()?;
        txn.insert(Artist {
            id,
            name,
            image_url,
        })?;
        txn.commit()?;
        Ok(id)
    })
    .await
}

pub(super) async fn find(id: i64) -> Result<Option<Artist>, Error> {
    run_db(move || db().r_transaction()?.get().primary::<Artist>(id)).await
}

pub(super) async fn update(
    id: i64,
    name: String,
    image_url: Option<String>,
) -> Result<bool, Error> {
    run_db(move || {
        let txn = db().rw_transaction()?;
        let Some(old) = txn.get().primary::<Artist>(id)? else {
            return Ok(false);
        };
        txn.update(
            old.clone(),
            Artist {
                id,
                name,
                image_url,
            },
        )?;
        txn.commit()?;
        Ok(true)
    })
    .await
}

pub(crate) async fn for_band(band_id: i64) -> Result<Vec<ArtistAssignment>, Error> {
    run_db(move || {
        let txn = db().r_transaction()?;
        let assignments: Vec<BandAssignment> =
            txn.scan().primary()?.all()?.collect::<Result<_, _>>()?;
        let mut result = Vec::new();
        for assignment in assignments.into_iter().filter(|a| a.band_id == band_id) {
            if let Some(artist) = txn.get().primary::<Artist>(assignment.artist_id)? {
                result.push(ArtistAssignment { artist, assignment });
            }
        }
        result.sort_unstable_by(|a, b| {
            a.artist
                .name
                .to_lowercase()
                .cmp(&b.artist.name.to_lowercase())
        });
        Ok(result)
    })
    .await
}

pub(crate) async fn bands_for_artist(artist_id: i64) -> Result<Vec<(BandAssignment, Band)>, Error> {
    run_db(move || {
        let txn = db().r_transaction()?;
        let assignments: Vec<BandAssignment> =
            txn.scan().primary()?.all()?.collect::<Result<_, _>>()?;
        let mut result = Vec::new();
        for assignment in assignments
            .into_iter()
            .filter(|item| item.artist_id == artist_id)
        {
            if let Some(band) = txn.get().primary::<Band>(assignment.band_id)? {
                result.push((assignment, band));
            }
        }
        result.sort_unstable_by(|a, b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()));
        Ok(result)
    })
    .await
}

pub(super) async fn assign(mut assignment: BandAssignment) -> Result<bool, Error> {
    run_db(move || {
        let txn = db().rw_transaction()?;
        if txn.get().primary::<Band>(assignment.band_id)?.is_none()
            || txn.get().primary::<Artist>(assignment.artist_id)?.is_none()
        {
            return Ok(false);
        }
        let existing: Vec<BandAssignment> =
            txn.scan().primary()?.all()?.collect::<Result<_, _>>()?;
        if existing
            .iter()
            .any(|a| a.band_id == assignment.band_id && a.artist_id == assignment.artist_id)
        {
            return Ok(false);
        }
        assignment.id = next_assignment_id()?;
        txn.insert(assignment)?;
        txn.commit()?;
        Ok(true)
    })
    .await
}
