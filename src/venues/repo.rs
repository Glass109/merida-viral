use native_db::db_type::Error;

use super::Venue;
use crate::db::{db, next_venue_id, run_db};

pub(crate) async fn list() -> Result<Vec<Venue>, Error> {
    run_db(|| {
        let txn = db().r_transaction()?;
        let mut venues: Vec<Venue> = txn.scan().primary()?.all()?.collect::<Result<_, _>>()?;
        venues.sort_unstable_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(venues)
    })
    .await
}

pub(super) async fn find(id: i64) -> Result<Option<Venue>, Error> {
    run_db(move || db().r_transaction()?.get().primary::<Venue>(id)).await
}

pub(super) async fn create(mut venue: Venue) -> Result<i64, Error> {
    run_db(move || {
        venue.id = next_venue_id()?;
        let txn = db().rw_transaction()?;
        let id = venue.id;
        txn.insert(venue)?;
        txn.commit()?;
        Ok(id)
    })
    .await
}

pub(super) async fn update(venue: Venue) -> Result<bool, Error> {
    run_db(move || {
        let txn = db().rw_transaction()?;
        let Some(old) = txn.get().primary::<Venue>(venue.id)? else {
            return Ok(false);
        };
        txn.update(old, venue)?;
        txn.commit()?;
        Ok(true)
    })
    .await
}
