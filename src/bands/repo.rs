use native_db::db_type::Error;

use crate::db::{db, next_band_id, run_db};

use super::Band;

pub(crate) async fn list() -> Result<Vec<Band>, Error> {
    run_db(|| {
        let txn = db().r_transaction()?;
        let mut bands: Vec<Band> = txn.scan().primary()?.all()?.collect::<Result<_, _>>()?;
        bands.sort_unstable_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(bands)
    })
    .await
}

pub(crate) async fn find(id: i64) -> Result<Option<Band>, Error> {
    run_db(move || db().r_transaction()?.get().primary::<Band>(id)).await
}

pub(super) async fn create(name: String, image_url: Option<String>) -> Result<i64, Error> {
    run_db(move || {
        let id = next_band_id()?;
        let txn = db().rw_transaction()?;
        txn.insert(Band {
            id,
            name,
            members: Vec::new(),
            image_url,
        })?;
        txn.commit()?;
        Ok(id)
    })
    .await
}

pub(super) async fn update(
    id: i64,
    name: String,
    image_url: Option<String>,
) -> Result<bool, Error> {
    run_db(move || {
        let txn = db().rw_transaction()?;
        let Some(old) = txn.get().primary::<Band>(id)? else {
            return Ok(false);
        };
        let updated = Band {
            name,
            image_url,
            ..old.clone()
        };
        txn.update(old, updated)?;
        txn.commit()?;
        Ok(true)
    })
    .await
}
