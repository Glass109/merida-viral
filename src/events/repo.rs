use std::collections::HashSet;

use chrono::Utc;
use native_db::db_type::Error;

use super::{Event, EventListing, EventPin};
use crate::{
    bands::Band,
    db::{db, next_event_id, run_db},
    venues::Venue,
};

pub(super) async fn list() -> Result<Vec<EventListing>, Error> {
    run_db(|| {
        let txn = db().r_transaction()?;
        let events: Vec<Event> = txn.scan().primary()?.all()?.collect::<Result<_, _>>()?;
        let mut listings = Vec::with_capacity(events.len());
        for event in events {
            let Some(venue) = txn.get().primary::<Venue>(event.venue_id)? else {
                continue;
            };
            let mut bands = Vec::with_capacity(event.band_ids.len());
            for id in &event.band_ids {
                if let Some(band) = txn.get().primary::<Band>(*id)? {
                    bands.push(band);
                }
            }
            listings.push(EventListing {
                pin_style: txn
                    .get()
                    .primary::<EventPin>(event.id)?
                    .map(|pin| pin.style)
                    .unwrap_or_default(),
                event,
                venue,
                bands,
            });
        }
        let now = Utc::now();
        listings.sort_unstable_by(|a, b| {
            let a_past = a.event.is_past(now);
            let b_past = b.event.is_past(now);
            a_past.cmp(&b_past).then_with(|| {
                if a_past {
                    b.event.starts_at.cmp(&a.event.starts_at)
                } else {
                    a.event.starts_at.cmp(&b.event.starts_at)
                }
            })
        });
        Ok(listings)
    })
    .await
}

pub(super) async fn find(id: i64) -> Result<Option<EventListing>, Error> {
    run_db(move || {
        let txn = db().r_transaction()?;
        let Some(event) = txn.get().primary::<Event>(id)? else {
            return Ok(None);
        };
        let Some(venue) = txn.get().primary::<Venue>(event.venue_id)? else {
            return Ok(None);
        };
        let mut bands = Vec::with_capacity(event.band_ids.len());
        for id in &event.band_ids {
            if let Some(band) = txn.get().primary::<Band>(*id)? {
                bands.push(band);
            }
        }
        Ok(Some(EventListing {
            pin_style: txn
                .get()
                .primary::<EventPin>(event.id)?
                .map(|pin| pin.style)
                .unwrap_or_default(),
            event,
            venue,
            bands,
        }))
    })
    .await
}

/// Validate references and insert in the same transaction to avoid stale IDs.
pub(super) async fn create(
    mut event: Event,
    style: crate::map_pin::PinStyle,
) -> Result<Option<i64>, Error> {
    run_db(move || {
        let txn = db().rw_transaction()?;
        if txn.get().primary::<Venue>(event.venue_id)?.is_none() {
            return Ok(None);
        }
        let mut seen = HashSet::new();
        for id in &event.band_ids {
            if !seen.insert(*id) || txn.get().primary::<Band>(*id)?.is_none() {
                return Ok(None);
            }
        }
        event.id = next_event_id()?;
        let id = event.id;
        txn.insert(event)?;
        txn.insert(EventPin {
            event_id: id,
            style,
        })?;
        txn.commit()?;
        Ok(Some(id))
    })
    .await
}

pub(super) async fn update(
    id: i64,
    mut event: Event,
    style: crate::map_pin::PinStyle,
) -> Result<bool, Error> {
    run_db(move || {
        let txn = db().rw_transaction()?;
        let Some(old) = txn.get().primary::<Event>(id)? else {
            return Ok(false);
        };
        if txn.get().primary::<Venue>(event.venue_id)?.is_none() {
            return Ok(false);
        }
        let mut seen = HashSet::new();
        for band_id in &event.band_ids {
            if !seen.insert(*band_id) || txn.get().primary::<Band>(*band_id)?.is_none() {
                return Ok(false);
            }
        }
        event.id = id;
        txn.update(old, event)?;
        let pin = EventPin {
            event_id: id,
            style,
        };
        if let Some(old_pin) = txn.get().primary::<EventPin>(id)? {
            txn.update(old_pin, pin)?;
        } else {
            txn.insert(pin)?;
        }
        txn.commit()?;
        Ok(true)
    })
    .await
}
