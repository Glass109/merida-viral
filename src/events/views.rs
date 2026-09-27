use chrono::{DateTime, Utc};
use chrono_tz::America::Merida;
use maud::{DOCTYPE, Markup, html};

use super::EventListing;
use crate::map_pin::{self, MapCoordinates, PinStyle};
use crate::{
    bands::Band,
    i18n::Text,
    ui::layout::{document_head, image_upload_field, shared_page_scripts},
    venues::Venue,
};

fn local_time(utc: DateTime<Utc>) -> String {
    utc.with_timezone(&Merida)
        .format("%d/%m/%Y · %H:%M")
        .to_string()
}

pub(super) fn list(text: &Text, events: &[EventListing]) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("events-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) {
                    a href="/" { (text.translate("discover-events")) }
                    a href="/videos" { (text.translate("videos")) }
                    a href="/events" aria-current="page" { (text.translate("events")) }
                    a href="/artists" { (text.translate("artists")) }
                    a href="/bands" { (text.translate("bands")) }
                    a href="/venues" { (text.translate("venues")) }
                }
                p class="metro-overline" { (text.translate("gig-guide")) }
                h1 class="metro-title" { (text.translate("events")) }
                @if events.is_empty() { p class="metro-empty" { (text.translate("events-empty")) } }
                @else { ul class="metro-list" {
                    @for item in events {
                        li {
                            a class="metro-item-link" href=(format!("/events/{}", item.event.id)) {
                                @if let Some(image) = &item.event.image_url { img class="metro-thumb" src=(image) alt="" loading="lazy"; }
                                strong { (&item.event.title) }
                            }
                            small { time datetime=(item.event.starts_at.to_rfc3339()) { (local_time(item.event.starts_at)) } " · " (&item.venue.name) }
                            @if !item.bands.is_empty() { small { (item.bands.iter().map(|band| band.name.as_str()).collect::<Vec<_>>().join(" · ")) } }
                            @if item.event.is_past(Utc::now()) { small class="metro-past" { (text.translate("past-event")) } }
                        }
                    }
                } }
            }
            nav class="metro-appbar" aria-label=(text.translate("actions")) {
                a href="/events/new" { span class="metro-action-icon" aria-hidden="true" { "+" } span { (text.translate("add-event")) } }
            }
        }
    } }
}

pub(super) fn new(text: &Text, venues: &[Venue], bands: &[Band]) -> Markup {
    event_form(text, venues, bands, None)
}

pub(super) fn edit(text: &Text, venues: &[Venue], bands: &[Band], item: &EventListing) -> Markup {
    event_form(text, venues, bands, Some(item))
}

fn event_form(
    text: &Text,
    venues: &[Venue],
    bands: &[Band],
    item: Option<&EventListing>,
) -> Markup {
    let event = item.map(|listing| &listing.event);
    let form_action = event.map_or_else(
        || "/events/new".to_owned(),
        |event| format!("/events/{}/edit", event.id),
    );
    let page_title = if event.is_some() {
        text.translate("edit-event-title")
    } else {
        text.translate("new-event-title")
    };
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(page_title))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) { @if let Some(event) = event { a href=(format!("/events/{}", event.id)) { (&event.title) } } @else { a href="/events" { (text.translate("events")) } } }
                h1 class="metro-title" { (text.translate(if event.is_some() { "edit" } else { "add-event" })) @if let Some(event) = event { " " (&event.title) } }
                @if venues.is_empty() { p class="metro-empty" { (text.translate("event-needs-venue")) " " a href="/venues/new" { (text.translate("add-venue")) } } }
                @if bands.is_empty() { p class="metro-empty" { (text.translate("event-needs-band")) " " a href="/bands/new" { (text.translate("add-band")) } } }
                @if !venues.is_empty() && !bands.is_empty() {
                    form id="event-form" class="metro-form" method="post" action=(form_action) data-image-upload="event" data-uploading=(text.translate("picture-uploading")) data-upload-error=(text.translate("picture-upload-error")) data-band-required=(text.translate("event-band-required")) {
                        label for="event-title" { (text.translate("event-name")) }
                        input id="event-title" name="title" type="text" maxlength="90" required value=[event.map(|event| event.title.as_str())];
                        label for="event-description" { (text.translate("event-description")) }
                        textarea id="event-description" name="description" rows="4" maxlength="2000" { @if let Some(event) = event { (&event.description) } }
                        label for="event-venue" { (text.translate("event-venue")) }
                        select id="event-venue" name="venue_id" required {
                            @for venue in venues { option value=(venue.id) selected[event.is_some_and(|event| event.venue_id == venue.id)] { (&venue.name) " · " (&venue.address) } }
                        }
                        fieldset class="metro-tags-field" {
                            legend { (text.translate("event-lineup")) }
                            p class="metro-hint" { (text.translate("event-lineup-hint")) }
                            @for band in bands {
                                label { input type="checkbox" name="band_id" value=(band.id) checked[event.is_some_and(|event| event.band_ids.contains(&band.id))]; (&band.name) }
                            }
                        }
                        label for="event-start" { (text.translate("event-start")) }
                        input id="event-start" name="starts_at" type="datetime-local" required value=[event.map(|event| event.starts_at.with_timezone(&Merida).format("%Y-%m-%dT%H:%M").to_string())];
                        label for="event-end" { (text.translate("event-end")) }
                        input id="event-end" name="ends_at" type="datetime-local" value=[event.and_then(|event| event.ends_at.map(|end| end.with_timezone(&Merida).format("%Y-%m-%dT%H:%M").to_string()))];
                        p class="metro-hint" { (text.translate("event-timezone")) }
                        @if let Some(event) = event { (crate::ui::layout::image_upload_field_with_value(text, "event-image", event.image_url.as_deref().unwrap_or(""))) }
                        @else { (image_upload_field(text, "event-image")) }
                        fieldset class="pin-picker" {
                            legend { (text.translate("pin-style")) }
                            @for style in PinStyle::ALL {
                                label {
                                    input type="radio" name="pin_style" value=(style.key()) checked[item.map_or(style == PinStyle::default(), |listing| style == listing.pin_style)];
                                    (map_pin::pin_art(style, None))
                                    span { (text.translate(&format!("pin-{}", style.key()))) }
                                }
                            }
                        }
                    }
                }
            }
            @if !venues.is_empty() && !bands.is_empty() {
                div class="metro-appbar" { button type="submit" form="event-form" { span class="metro-action-icon" aria-hidden="true" { "✓" } span { (text.translate("save")) } } }
                script src="/event-new.js" {}
                script src="/pin-picker.js" {}
                script src="/image-upload.js" {}
            }
        }
    } }
}

pub(super) fn detail(text: &Text, item: &EventListing) -> Markup {
    let event = &item.event;
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(event.title.clone()))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) { a href="/events" { (text.translate("events")) } }
                p class="metro-overline" { time datetime=(event.starts_at.to_rfc3339()) { (local_time(event.starts_at)) } }
                h1 class="metro-title" { (&event.title) }
                @if let Some(image) = &event.image_url { img class="metro-hero-image" src=(image) alt=""; }
                @if let Some(end) = event.ends_at { p class="metro-hint" { (text.translate("event-until")) " " time datetime=(end.to_rfc3339()) { (local_time(end)) } } }
                @if !event.description.is_empty() { p class="metro-description" { (&event.description) } }
                h2 class="metro-subtitle" { (text.translate("event-lineup")) }
                ul class="metro-list" {
                    @for band in &item.bands { li { a class="metro-item-link" href=(format!("/bands/{}", band.id)) {
                        @if let Some(image) = &band.image_url { img class="metro-thumb" src=(image) alt="" loading="lazy"; }
                        strong { (&band.name) }
                    } } }
                }
                h2 class="metro-subtitle" { (text.translate("event-venue")) }
                p { strong { (&item.venue.name) } br; (&item.venue.address) }
                div class="detail-map-wrap" { div class="detail-map" id="detail-map" data-lat=(item.venue.location.1) data-lng=(item.venue.location.0) data-title=(&event.title) {} (map_pin::pin_template(item)) }
            }
            (shared_page_scripts())
            script src="/detail.js" {}
            nav class="metro-appbar" aria-label=(text.translate("actions")) {
                a href=(format!("/events/{}/edit", event.id)) { span class="metro-action-icon" aria-hidden="true" { "✎" } span { (text.translate("edit")) } }
            }
        }
    } }
}

pub(super) fn home(text: &Text, events: &[EventListing]) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("events-title")))
        body class="metro-page event-home" {
            main class="event-discovery" {
                header {
                    nav class="metro-context" aria-label=(text.translate("sections")) {
                        a href="/" aria-current="page" { (text.translate("discover-events")) }
                        a href="/events" { (text.translate("events")) }
                        a href="/bands" { (text.translate("bands")) }
                        a href="/artists" { (text.translate("artists")) }
                        a href="/venues" { (text.translate("venues")) }
                        a href="/videos" { (text.translate("videos")) }
                    }
                    h1 class="metro-title" { (text.translate("discover-events")) }
                    p class="metro-intro" { (text.translate("discover-intro")) }
                }
                div class="event-discovery-grid" {
                    section class="event-map-section" aria-label=(text.translate("event-map")) {
                        div id="event-map" aria-label=(text.translate("event-map")) {}
                        p class="metro-hint" { (text.translate("event-map-hint")) }
                    }
                    section aria-labelledby="upcoming-title" {
                        h2 id="upcoming-title" class="metro-subtitle" { (text.translate("upcoming-events")) }
                        @if events.is_empty() { p class="metro-empty" { (text.translate("upcoming-empty")) } }
                        ul class="metro-list event-discovery-list" {
                            @for item in events {
                                li id=(format!("event-{}", item.event.id)) data-lat=(item.y()) data-lng=(item.x()) data-title=(&item.event.title) {
                                    a class="event-show-link" href=(format!("/events/{}", item.event.id)) {
                                        @if let Some(image) = &item.event.image_url { img class="event-poster" src=(image) alt="" loading="lazy"; }
                                        time datetime=(item.event.starts_at.to_rfc3339()) { (local_time(item.event.starts_at)) }
                                        strong { (&item.event.title) }
                                        small { (&item.venue.name) " · " (&item.venue.address) }
                                        small { (item.bands.iter().map(|band| band.name.as_str()).collect::<Vec<_>>().join(" · ")) }
                                    }
                                    button class="event-locate" type="button" { (text.translate("event-locate")) }
                                    (map_pin::pin_template(item))
                                }
                            }
                        }
                        a href="/events" { (text.translate("all-events")) }
                    }
                }
            }
            nav class="metro-appbar" aria-label=(text.translate("actions")) {
                a href="/events/new" { span class="metro-action-icon" aria-hidden="true" { "+" } span { (text.translate("add-event")) } }
            }
            (shared_page_scripts())
            script src="/event-map.js" {}
        }
    } }
}
