use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::America::Merida;
use topcoat::{
    Result,
    context::Cx,
    router::{
        content::Html,
        error::{RouterErrorExt, SeeOther, bad_request, see_other},
        path_param,
        request::Bytes,
        route,
    },
};

use super::{Event, repo, views};
use crate::{bands, i18n::Text, images, venues};

path_param!(event_id: i64);

#[route(GET "/")]
async fn home(cx: &Cx) -> Result<Html<String>> {
    let mut events = repo::list().await?;
    let now = Utc::now();
    events.retain(|item| !item.event.is_past(now));
    Ok(Html(views::home(&Text::new(cx), &events).into_string()))
}

#[route(GET "/events")]
async fn list_events(cx: &Cx) -> Result<Html<String>> {
    let events = repo::list().await?;
    Ok(Html(views::list(&Text::new(cx), &events).into_string()))
}

#[route(GET "/events/new")]
async fn new_event(cx: &Cx) -> Result<Html<String>> {
    let venues = venues::repo::list().await?;
    let bands = bands::repo::list().await?;
    Ok(Html(
        views::new(&Text::new(cx), &venues, &bands).into_string(),
    ))
}

#[route(POST "/events/new")]
async fn create_event(cx: &Cx, body: Bytes) -> Result<SeeOther> {
    let event = parse_event_form(&body)
        .ok_or_else(|| bad_request(Text::new(cx).translate("invalid-event")))?;
    let styles: Vec<_> = form_urlencoded::parse(&body)
        .filter(|(key, _)| key == "pin_style")
        .map(|(_, value)| value.into_owned())
        .collect();
    let style = match styles.as_slice() {
        [] => crate::map_pin::PinStyle::default(),
        [value] => crate::map_pin::PinStyle::parse(value)
            .ok_or_else(|| bad_request("Invalid pin style"))?,
        _ => return Err(bad_request("Duplicate pin style").into()),
    };
    let id = repo::create(event, style)
        .await?
        .ok_or_else(|| bad_request(Text::new(cx).translate("invalid-event")))?;
    Ok(see_other(format!("/events/{id}")))
}

#[route(GET "/events/{event_id}")]
async fn event_detail(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<EventId>(cx).ok_or_not_found()?;
    let event = repo::find(id).await?.ok_or_not_found()?;
    Ok(Html(views::detail(&Text::new(cx), &event).into_string()))
}

#[route(GET "/events/{event_id}/edit")]
async fn edit_event(cx: &Cx) -> Result<Html<String>> {
    let id = *path_param::<EventId>(cx).ok_or_not_found()?;
    let item = repo::find(id).await?.ok_or_not_found()?;
    let venues = venues::repo::list().await?;
    let bands = bands::repo::list().await?;
    Ok(Html(
        views::edit(&Text::new(cx), &venues, &bands, &item).into_string(),
    ))
}

#[route(POST "/events/{event_id}/edit")]
async fn update_event(cx: &Cx, body: Bytes) -> Result<SeeOther> {
    let id = *path_param::<EventId>(cx).ok_or_not_found()?;
    let event = parse_event_form(&body)
        .ok_or_else(|| bad_request(Text::new(cx).translate("invalid-event")))?;
    let styles: Vec<_> = form_urlencoded::parse(&body)
        .filter(|(key, _)| key == "pin_style")
        .map(|(_, value)| value.into_owned())
        .collect();
    let style = match styles.as_slice() {
        [] => repo::find(id).await?.ok_or_not_found()?.pin_style,
        [value] => crate::map_pin::PinStyle::parse(value)
            .ok_or_else(|| bad_request("Invalid pin style"))?,
        _ => return Err(bad_request("Duplicate pin style").into()),
    };
    if !repo::update(id, event, style).await? {
        return Err(topcoat::router::error::not_found().into());
    }
    Ok(see_other(format!("/events/{id}")))
}

/// HTML datetime-local values have no offset; interpret them in Mérida,
/// rejecting historically ambiguous or nonexistent civil times.
fn merida_time(value: &str) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M").ok()?;
    Some(
        Merida
            .from_local_datetime(&naive)
            .single()?
            .with_timezone(&Utc),
    )
}

fn parse_event_form(body: &[u8]) -> Option<Event> {
    let mut fields = HashMap::new();
    let mut band_ids = Vec::new();
    for (key, value) in form_urlencoded::parse(body) {
        if key == "band_id" {
            band_ids.push(value.parse::<i64>().ok()?);
        } else if matches!(
            key.as_ref(),
            "title" | "description" | "venue_id" | "starts_at" | "ends_at" | "image_url"
        ) {
            if fields
                .insert(key.into_owned(), value.into_owned())
                .is_some()
            {
                return None;
            }
        }
    }
    let title = fields.remove("title")?.trim().to_owned();
    let description = fields
        .remove("description")
        .unwrap_or_default()
        .trim()
        .to_owned();
    let venue_id = fields.remove("venue_id")?.parse::<i64>().ok()?;
    let starts_at = merida_time(&fields.remove("starts_at")?)?;
    let ends_at = match fields.remove("ends_at").as_deref() {
        None | Some("") => None,
        Some(value) => Some(merida_time(value)?),
    };
    let image_url = images::normalized_image_url(fields.remove("image_url"));
    if title.is_empty()
        || title.chars().count() > 90
        || description.chars().count() > 2000
        || venue_id <= 0
        || band_ids.is_empty()
        || band_ids.len() > 20
        || band_ids.iter().any(|id| *id <= 0)
        || band_ids.iter().collect::<HashSet<_>>().len() != band_ids.len()
        || ends_at.is_some_and(|end| end <= starts_at)
        || !images::valid_image_url_for("event", &image_url)
    {
        return None;
    }
    Some(Event {
        id: 0,
        title,
        description,
        venue_id,
        band_ids,
        starts_at,
        ends_at,
        image_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    #[test]
    fn merida_local_time_is_stored_in_utc() {
        let event = parse_event_form(b"title=Festival&venue_id=1&band_id=2&band_id=3&starts_at=2026-10-01T20%3A30&ends_at=2026-10-01T23%3A00&image_url=").unwrap();
        assert_eq!(event.starts_at.year(), 2026);
        assert_eq!(event.starts_at.hour(), 2); // 20:30 in Mérida = 02:30 UTC next day
        assert_eq!(event.starts_at.day(), 2);
        assert_eq!(event.band_ids, vec![2, 3]);
        assert!(event.ends_at.unwrap() > event.starts_at);
        assert!(event.image_url.is_none());
    }

    #[test]
    fn invalid_lineups_and_dates_are_rejected() {
        assert!(
            parse_event_form(
                b"title=Test&venue_id=1&band_id=2&band_id=2&starts_at=2026-10-01T20%3A30"
            )
            .is_none()
        );
        assert!(parse_event_form(b"title=Test&venue_id=1&starts_at=2026-10-01T20%3A30").is_none());
        assert!(parse_event_form(b"title=Test&venue_id=1&band_id=2&starts_at=2026-10-01T20%3A30&ends_at=2026-10-01T19%3A00").is_none());
        assert!(
            parse_event_form(b"title=Test&venue_id=1&band_id=2&starts_at=not-a-date").is_none()
        );
    }
}
