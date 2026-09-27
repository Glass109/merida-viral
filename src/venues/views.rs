use fluent_bundle::FluentArgs;
use maud::{DOCTYPE, Markup, html};

use super::{Venue, VenueTag};
use crate::{
    i18n::Text,
    map_pin::{self, MapCoordinates, MapPin},
    ui::layout::{
        document_head, image_upload_field, image_upload_field_with_value, shared_page_scripts,
    },
};

pub(super) fn list(text: &Text, venues: &[Venue]) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("venues-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) {
                    a href="/" { (text.translate("discover-events")) }
                    a href="/videos" { (text.translate("videos")) }
                    a href="/events" { (text.translate("events")) }
                    a href="/artists" { (text.translate("artists")) }
                    a href="/bands" { (text.translate("bands")) }
                    a href="/venues" aria-current="page" { (text.translate("venues")) }
                }
                p class="metro-overline" { (text.translate("music-venues")) }
                h1 class="metro-title" { (text.translate("venues")) }
                @if venues.is_empty() {
                    p class="metro-empty" { (text.translate("venues-empty")) }
                } @else {
                    ul class="metro-list" {
                        @for venue in venues {
                            li data-lng=(venue.x()) data-lat=(venue.y()) data-title=(venue.pin_name()) {
                                a class="metro-venue-link" href=(format!("/venues/{}", venue.id)) {
                                    @if let Some(image) = &venue.image_url { img class="metro-venue-photo" src=(image) alt="" loading="lazy"; }
                                    strong { (&venue.name) }
                                    small { (&venue.address) }
                                    (venue_tags(text, venue))
                                }
                                (map_pin::pin_template(venue))
                            }
                        }
                    }
                }
            }
            nav class="metro-appbar" aria-label=(text.translate("actions")) {
                a href="/venues/new" { span class="metro-action-icon" aria-hidden="true" { "+" } span { (text.translate("add-venue")) } }
            }
        }
    } }
}

fn tag_label(text: &Text, tag: VenueTag) -> String {
    text.translate(match tag {
        VenueTag::PetFriendly => "pet-friendly",
        VenueTag::NoAlcohol => "no-alcohol",
        VenueTag::NoDrugs => "no-drugs",
        VenueTag::LowVolume => "low-volume",
    })
}

fn venue_tags(text: &Text, venue: &Venue) -> Markup {
    html! {
        @if !venue.tags.is_empty() {
            div class="metro-tags" {
                @for tag in [VenueTag::PetFriendly, VenueTag::NoAlcohol, VenueTag::NoDrugs, VenueTag::LowVolume] {
                    @if venue.tags.contains(&tag) { span { (tag_label(text, tag)) } }
                }
            }
        }
    }
}

pub(super) fn detail(text: &Text, venue: &Venue) -> Markup {
    let mut args = FluentArgs::new();
    args.set("title", venue.pin_name());
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate_with_args("detail-title", Some(&args))))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) {
                    a href="/venues" { (text.translate("venues")) }
                }
                h1 class="metro-title" { (&venue.name) }
                @if let Some(image) = &venue.image_url {
                    img class="metro-venue-hero" src=(image) alt="";
                }
                p class="metro-venue-address" { (&venue.address) }
                (venue_tags(text, venue))
                section class="detail-map-wrap" aria-label=(text.translate("right-corner")) {
                    h2 class="metro-subtitle" { (text.translate("right-corner")) }
                    div class="detail-map" id="detail-map" data-lat=(venue.y()) data-lng=(venue.x()) data-title=(venue.pin_name()) {}
                    (map_pin::pin_template(venue))
                }
            }
            (shared_page_scripts())
            nav class="metro-appbar" aria-label=(text.translate("actions")) {
                a href=(format!("/venues/{}/edit", venue.id)) { span class="metro-action-icon" aria-hidden="true" { "✎" } span { (text.translate("edit")) } }
            }
            script src="/detail.js" {}
        }
    } }
}

pub(super) fn edit(text: &Text, venue: &Venue) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("edit-venue-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) { a href=(format!("/venues/{}", venue.id)) { (&venue.name) } }
                h1 class="metro-title" { (text.translate("edit")) " " (&venue.name) }
                form id="venue-form" class="metro-form" action=(format!("/venues/{}/edit", venue.id)) method="post" data-image-upload="venue" data-uploading=(text.translate("picture-uploading")) data-upload-error=(text.translate("picture-upload-error")) {
                    label for="venue-name" { (text.translate("venue-name")) }
                    input id="venue-name" name="name" type="text" maxlength="90" required value=(venue.name);
                    label for="venue-address" { (text.translate("venue-address")) }
                    input id="venue-address" name="address" type="text" maxlength="200" required value=(venue.address);
                    (image_upload_field_with_value(text, "venue-image", venue.image_url.as_deref().unwrap_or("")))
                    fieldset class="metro-tags-field" {
                        legend { (text.translate("venue-tags")) }
                        label { input type="checkbox" name="pet_friendly" value="on" checked[venue.tags.contains(&VenueTag::PetFriendly)]; (text.translate("pet-friendly")) }
                        label { input type="checkbox" name="no_alcohol" value="on" checked[venue.tags.contains(&VenueTag::NoAlcohol)]; (text.translate("no-alcohol")) }
                        label { input type="checkbox" name="no_drugs" value="on" checked[venue.tags.contains(&VenueTag::NoDrugs)]; (text.translate("no-drugs")) }
                        label { input type="checkbox" name="low_volume" value="on" checked[venue.tags.contains(&VenueTag::LowVolume)]; (text.translate("low-volume")) }
                    }
                    p class="metro-field-title" { (text.translate("venue-location")) }
                    p class="metro-hint" { (text.translate("venue-map-hint")) }
                    div id="venue-map" class="metro-venue-map" aria-label=(text.translate("venue-location")) {}
                    div class="metro-coordinates" {
                        label for="venue-lat" { (text.translate("latitude")) input id="venue-lat" name="lat" type="number" step="any" min="-90" max="90" required value=(venue.location.1); }
                        label for="venue-lng" { (text.translate("longitude")) input id="venue-lng" name="lng" type="number" step="any" min="-180" max="180" required value=(venue.location.0); }
                    }
                }
            }
            div class="metro-appbar" { button type="submit" form="venue-form" { span class="metro-action-icon" aria-hidden="true" { "✓" } span { (text.translate("save")) } } }
            (shared_page_scripts())
            script src="/venue-new.js" {}
            script src="/image-upload.js" {}
        }
    } }
}

pub(super) fn new(text: &Text) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("new-venue-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) {
                    a href="/venues" { (text.translate("venues")) }
                }
                h1 class="metro-title" { (text.translate("add-venue")) }
                p class="metro-intro" { (text.translate("venue-intro")) }
                form id="venue-form" class="metro-form" action="/venues/new" method="post" data-image-upload="venue" data-uploading=(text.translate("picture-uploading")) data-upload-error=(text.translate("picture-upload-error")) {
                    label for="venue-name" { (text.translate("venue-name")) }
                    input id="venue-name" name="name" type="text" maxlength="90" required;
                    label for="venue-address" { (text.translate("venue-address")) }
                    input id="venue-address" name="address" type="text" maxlength="200" required;
                    (image_upload_field(text, "venue-image"))
                    fieldset class="metro-tags-field" {
                        legend { (text.translate("venue-tags")) }
                        label { input type="checkbox" name="pet_friendly" value="on"; (text.translate("pet-friendly")) }
                        label { input type="checkbox" name="no_alcohol" value="on"; (text.translate("no-alcohol")) }
                        label { input type="checkbox" name="no_drugs" value="on"; (text.translate("no-drugs")) }
                        label { input type="checkbox" name="low_volume" value="on"; (text.translate("low-volume")) }
                    }
                    p class="metro-field-title" { (text.translate("venue-location")) }
                    p class="metro-hint" { (text.translate("venue-map-hint")) }
                    div id="venue-map" class="metro-venue-map" aria-label=(text.translate("venue-location")) {}
                    div class="metro-coordinates" {
                        label for="venue-lat" { (text.translate("latitude")) input id="venue-lat" name="lat" type="number" step="any" min="-90" max="90" required; }
                        label for="venue-lng" { (text.translate("longitude")) input id="venue-lng" name="lng" type="number" step="any" min="-180" max="180" required; }
                    }
                }
            }
            div class="metro-appbar" { button type="submit" form="venue-form" { span class="metro-action-icon" aria-hidden="true" { "✓" } span { (text.translate("save")) } } }
            (shared_page_scripts())
            script src="/venue-new.js" {}
            script src="/image-upload.js" {}
        }
    } }
}
