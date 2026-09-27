use maud::{DOCTYPE, Markup, html};

use super::Band;
use crate::{
    artists::ArtistAssignment,
    i18n::Text,
    ui::layout::{document_head, image_upload_field, image_upload_field_with_value},
};

pub(super) fn list(text: &Text, bands: &[Band]) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("bands-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) {
                    a href="/" { (text.translate("discover-events")) }
                    a href="/videos" { (text.translate("videos")) }
                    a href="/events" { (text.translate("events")) }
                    a href="/artists" { (text.translate("artists")) }
                    a href="/bands" aria-current="page" { (text.translate("bands")) }
                    a href="/venues" { (text.translate("venues")) }
                }
                p class="metro-overline" { (text.translate("local-lineup")) }
                h1 class="metro-title" { (text.translate("bands")) }
                @if bands.is_empty() {
                    p class="metro-empty" { (text.translate("bands-empty")) }
                } @else {
                    ul class="metro-list" {
                        @for band in bands {
                            li {
                                a class="metro-item-link" href=(format!("/bands/{}", band.id)) {
                                    @if let Some(image) = &band.image_url { img class="metro-thumb" src=(image) alt="" loading="lazy"; }
                                    strong { (&band.name) }
                                }
                            }
                        }
                    }
                }
            }
            nav class="metro-appbar" aria-label=(text.translate("actions")) {
                a href="/bands/new" { span class="metro-action-icon" aria-hidden="true" { "+" } span { (text.translate("add-band")) } }
            }
        }
    } }
}

pub(super) fn new(text: &Text) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("new-band-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) {
                    a href="/bands" { (text.translate("bands")) }
                }
                h1 class="metro-title" { (text.translate("add-band")) }
                p class="metro-intro" { (text.translate("band-intro")) }
                form id="band-form" action="/bands/new" method="post" class="metro-form" data-image-upload="band" data-uploading=(text.translate("picture-uploading")) data-upload-error=(text.translate("picture-upload-error")) {
                    label for="band-name" { (text.translate("band-name")) }
                    input id="band-name" name="name" type="text" maxlength="90" required autocomplete="off";
                    (image_upload_field(text, "band-image"))
                }
            }
            div class="metro-appbar" { button type="submit" form="band-form" { span class="metro-action-icon" aria-hidden="true" { "✓" } span { (text.translate("save")) } } }
            script src="/image-upload.js" {}
        }
    } }
}

pub(super) fn detail(text: &Text, band: &Band, assignments: &[ArtistAssignment]) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("bands-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) { a href="/bands" { (text.translate("bands")) } }
                h1 class="metro-title" { (&band.name) }
                @if let Some(image) = &band.image_url { img class="metro-hero-image" src=(image) alt=""; }
                h2 class="metro-subtitle" { (text.translate("artists")) }
                @if assignments.is_empty() { p class="metro-empty" { (text.translate("band-artists-empty")) } }
                @else { ul class="metro-list" {
                    @for item in assignments {
                        li {
                            div class="metro-item-link" {
                                @if let Some(image) = &item.artist.image_url { img class="metro-thumb" src=(image) alt="" loading="lazy"; }
                                strong { (&item.artist.name) }
                            }
                            @if !item.assignment.role.is_empty() { small { (&item.assignment.role) } }
                            @if item.assignment.started_at.is_some() || item.assignment.ended_at.is_some() {
                                small { @if let Some(start) = item.assignment.started_at { (start) } " — " @if let Some(end) = item.assignment.ended_at { (end) } }
                            }
                        }
                    }
                } }
            }
            nav class="metro-appbar" aria-label=(text.translate("actions")) {
                a href=(format!("/bands/{}/edit", band.id)) { span class="metro-action-icon" aria-hidden="true" { "✎" } span { (text.translate("edit")) } }
                a href=(format!("/bands/{}/artists/new", band.id)) { span class="metro-action-icon" aria-hidden="true" { "+" } span { (text.translate("assign-artist")) } }
            }
        }
    } }
}

pub(super) fn edit(text: &Text, band: &Band) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("edit-band-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) { a href=(format!("/bands/{}", band.id)) { (&band.name) } }
                h1 class="metro-title" { (text.translate("edit")) " " (&band.name) }
                form id="band-form" action=(format!("/bands/{}/edit", band.id)) method="post" class="metro-form" data-image-upload="band" data-uploading=(text.translate("picture-uploading")) data-upload-error=(text.translate("picture-upload-error")) {
                    label for="band-name" { (text.translate("band-name")) }
                    input id="band-name" name="name" type="text" maxlength="90" required value=(band.name);
                    (image_upload_field_with_value(text, "band-image", band.image_url.as_deref().unwrap_or("")))
                }
            }
            div class="metro-appbar" { button type="submit" form="band-form" { span class="metro-action-icon" aria-hidden="true" { "✓" } span { (text.translate("save")) } } }
            script src="/image-upload.js" {}
        }
    } }
}
