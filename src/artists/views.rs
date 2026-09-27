use maud::{DOCTYPE, Markup, html};

use super::Artist;
use crate::{
    bands::Band,
    i18n::Text,
    ui::layout::{document_head, image_upload_field, image_upload_field_with_value},
};

pub(super) fn list(text: &Text, artists: &[Artist]) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("artists-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) {
                    a href="/" { (text.translate("discover-events")) }
                    a href="/videos" { (text.translate("videos")) }
                    a href="/events" { (text.translate("events")) }
                    a href="/artists" aria-current="page" { (text.translate("artists")) }
                    a href="/bands" { (text.translate("bands")) }
                    a href="/venues" { (text.translate("venues")) }
                }
                p class="metro-overline" { (text.translate("local-artists")) }
                h1 class="metro-title" { (text.translate("artists")) }
                @if artists.is_empty() { p class="metro-empty" { (text.translate("artists-empty")) } }
                @else { ul class="metro-list" {
                    @for artist in artists {
                        li { a class="metro-item-link" href=(format!("/artists/{}", artist.id)) {
                            @if let Some(image) = &artist.image_url { img class="metro-thumb" src=(image) alt="" loading="lazy"; }
                            strong { (&artist.name) }
                        } }
                    }
                } }
            }
            nav class="metro-appbar" aria-label=(text.translate("actions")) {
                a href="/artists/new" { span class="metro-action-icon" aria-hidden="true" { "+" } span { (text.translate("add-artist")) } }
            }
        }
    } }
}

pub(super) fn new(text: &Text) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("new-artist-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) { a href="/artists" { (text.translate("artists")) } }
                h1 class="metro-title" { (text.translate("add-artist")) }
                p class="metro-intro" { (text.translate("artist-intro")) }
                form id="artist-form" class="metro-form" action="/artists/new" method="post" data-image-upload="artist" data-uploading=(text.translate("picture-uploading")) data-upload-error=(text.translate("picture-upload-error")) {
                    label for="artist-name" { (text.translate("artist-name")) }
                    input id="artist-name" name="name" type="text" maxlength="90" required;
                    (image_upload_field(text, "artist-image"))
                }
            }
            div class="metro-appbar" { button type="submit" form="artist-form" { span class="metro-action-icon" aria-hidden="true" { "✓" } span { (text.translate("save")) } } }
            script src="/image-upload.js" {}
        }
    } }
}

pub(super) fn detail(
    text: &Text,
    artist: &Artist,
    bands: &[(super::BandAssignment, Band)],
) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(artist.name.clone()))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) { a href="/artists" { (text.translate("artists")) } }
                h1 class="metro-title" { (&artist.name) }
                @if let Some(image) = &artist.image_url { img class="metro-hero-image" src=(image) alt=""; }
                h2 class="metro-subtitle" { (text.translate("bands")) }
                @if bands.is_empty() { p class="metro-empty" { (text.translate("artist-bands-empty")) } }
                @else { ul class="metro-list" { @for (assignment, band) in bands {
                    li { a class="metro-item-link" href=(format!("/bands/{}", band.id)) {
                        @if let Some(image) = &band.image_url { img class="metro-thumb" src=(image) alt="" loading="lazy"; }
                        strong { (&band.name) }
                    }
                    @if !assignment.role.is_empty() { small { (&assignment.role) } }
                    @if assignment.started_at.is_some() || assignment.ended_at.is_some() { small { @if let Some(start) = assignment.started_at { (start) } " — " @if let Some(end) = assignment.ended_at { (end) } } }
                } } } }
            }
            nav class="metro-appbar" aria-label=(text.translate("actions")) {
                a href=(format!("/artists/{}/edit", artist.id)) { span class="metro-action-icon" aria-hidden="true" { "✎" } span { (text.translate("edit")) } }
            }
        }
    } }
}

pub(super) fn edit(text: &Text, artist: &Artist) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("edit-artist-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) { a href=(format!("/artists/{}", artist.id)) { (&artist.name) } }
                h1 class="metro-title" { (text.translate("edit")) " " (&artist.name) }
                form id="artist-form" class="metro-form" action=(format!("/artists/{}/edit", artist.id)) method="post" data-image-upload="artist" data-uploading=(text.translate("picture-uploading")) data-upload-error=(text.translate("picture-upload-error")) {
                    label for="artist-name" { (text.translate("artist-name")) }
                    input id="artist-name" name="name" type="text" maxlength="90" required value=(artist.name);
                    (image_upload_field_with_value(text, "artist-image", artist.image_url.as_deref().unwrap_or("")))
                }
            }
            div class="metro-appbar" { button type="submit" form="artist-form" { span class="metro-action-icon" aria-hidden="true" { "✓" } span { (text.translate("save")) } } }
            script src="/image-upload.js" {}
        }
    } }
}

pub(super) fn new_assignment(text: &Text, band: &Band, artists: &[Artist]) -> Markup {
    html! { (DOCTYPE) html lang=(text.language_tag()) {
        (document_head(text.translate("assign-artist-title")))
        body class="metro-page" {
            main class="metro-main" {
                nav class="metro-context" aria-label=(text.translate("sections")) {
                    a href=(format!("/bands/{}", band.id)) { (&band.name) }
                }
                h1 class="metro-title" { (text.translate("assign-artist")) }
                @if artists.is_empty() {
                    p class="metro-empty" { (text.translate("artists-empty")) " " a href="/artists/new" { (text.translate("add-artist")) } }
                } @else {
                    form id="assignment-form" class="metro-form" method="post" action=(format!("/bands/{}/artists/new", band.id)) {
                        label for="assignment-artist" { (text.translate("artist-name")) }
                        select id="assignment-artist" name="artist_id" required {
                            @for artist in artists { option value=(artist.id) { (&artist.name) } }
                        }
                        label for="assignment-role" { (text.translate("artist-role")) }
                        input id="assignment-role" type="text" name="role" maxlength="90";
                        div class="metro-coordinates" {
                            label for="assignment-start" { (text.translate("assignment-start")) input id="assignment-start" type="date" name="started_at"; }
                            label for="assignment-end" { (text.translate("assignment-end")) input id="assignment-end" type="date" name="ended_at"; }
                        }
                    }
                }
            }
            @if !artists.is_empty() { div class="metro-appbar" { button type="submit" form="assignment-form" { span class="metro-action-icon" aria-hidden="true" { "✓" } span { (text.translate("save")) } } } }
        }
    } }
}
