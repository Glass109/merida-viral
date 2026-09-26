use super::{model::Video, repo};
use crate::{
    i18n::Text,
    ui::{
        components::{brand, settings_panel},
        layout::{head, scripts, topbar},
    },
};
use fluent_bundle::FluentArgs;
use maud::{DOCTYPE, Markup, html};
use topcoat::router::content::Html;
fn rendered(markup: Markup) -> Html<String> {
    Html(markup.into_string())
}

/// Join the non-empty parts of a place's meta line with a separator.
fn place_meta(video: &Video) -> String {
    [
        video.neighborhood.as_str(),
        video.platform.as_str(),
        video.creator.as_str(),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" · ")
}

pub(crate) fn places_fragment(text: &Text, newest: bool) -> Html<String> {
    rendered(html! {
        @let videos = repo::list_videos(newest);
        @if videos.is_empty() { p class="loading" { (text.t("no-places")) } }
        @for (i, v) in videos.iter().enumerate() {
            article class="place-row" data-lat=(v.lat) data-lng=(v.lng) data-title=(&v.title) {
                @let mut args = FluentArgs::new();
                @let _ = args.set("title", v.title.as_str());
                button class="place-number" type="button" aria-label=(text.with_args("show-on-map", Some(&args))) { (format!("{:02}", i + 1)) }
                div class="place-copy" {
                    a class="place-title" href=(format!("/video/{}", v.id)) { (&v.title) }
                    div class="place-meta" { (place_meta(v)) }
                }
                button class="place-vote" hx-post="/vote" hx-vals=(format!("{{\"id\":{}}}", v.id)) hx-target="#video-grid" hx-swap="innerHTML" aria-label=(text.with_args("vote-label", Some(&args))) { "↑ " (v.votes) }
            }
        }
    })
}
pub(crate) fn detail_vote(text: &Text, id: i64, votes: i64) -> Markup {
    html! { button class="button-dark detail-vote" id="detail-vote" hx-post="/video-vote" hx-vals=(format!("{{\"id\":{id}}}")) hx-target="#detail-vote" hx-swap="outerHTML" { (text.t("upvote")) " · " (votes) span { "↑" } } }
}

pub(crate) fn home_page(text: &Text) -> Html<String> {
    rendered(html! {
        (DOCTYPE)
        html lang=(text.lang()) {
            (head(text.t("app-title")))
            body class="map-app" {
                main class="map-workspace" data-view="places" {
                    header class="map-header" {
                        (brand(text))
                        div class="city-label" { i {} span { (text.t("city")) } }
                    }
                    section class="map-tools" id="places-view" aria-label=(text.t("places")) {
                        div class="map-heading" {
                            span class="section-kicker" { (text.t("explore-city")) }
                            h1 { (text.t("heading-first")) " " (text.t("heading-second")) br; em { (text.t("heading-third")) } }
                            p { strong id="video-count" { "…" } " " (text.t("trending-nearby")) }
                        }
                        section class="place-list" id="video-grid" data-error=(text.t("places-error")) hx-get="/places" hx-trigger="load" hx-swap="innerHTML" aria-live="polite" { p class="loading" { (text.t("finding")) } }
                    }
                    section class="map-view" id="map-view" aria-label=(text.t("map-label")) {
                        div class="map-view-heading" { span class="section-kicker" { (text.t("explore-city")) } h1 { (text.t("map")) } }
                        div id="map" class="map" aria-label=(text.t("map-label")) {}
                        div class="map-controls" {
                            button id="locate" type="button" aria-label=(text.t("my-location-label")) title=(text.t("my-location-label")) { "◎" }
                            button id="zoom-in" type="button" aria-label=(text.t("zoom-in")) { "+" }
                            button id="zoom-out" type="button" aria-label=(text.t("zoom-out")) { "−" }
                        }
                    }
                    nav class="app-bar" aria-label=(text.t("actions")) {
                        button class="bar-action" id="view-toggle" type="button" aria-controls="map-view" aria-expanded="false" data-map=(text.t("map")) data-places=(text.t("places")) { span class="bar-icon" aria-hidden="true" { "▤" } span class="bar-label" { (text.t("map")) } }
                        button class="bar-action" id="sort-toggle" type="button" data-trending=(text.t("trending")) data-newest=(text.t("newest")) data-sort-trending=(text.t("sort-by-trending")) data-sort-newest=(text.t("sort-by-newest")) aria-label=(text.t("sort-by-trending")) { span class="bar-icon" aria-hidden="true" { "≡" } span class="bar-label" { (text.t("trending")) } }
                        a class="bar-action" href="/submit" { span class="bar-icon" aria-hidden="true" { "+" } span class="bar-label" { (text.t("post")) } }
                        button class="bar-action" type="button" id="settings-toggle" aria-controls="settings-panel" aria-expanded="false" aria-label=(text.t("appearance")) { span class="bar-icon" aria-hidden="true" { "···" } span class="bar-label" { (text.t("more")) } }
                    }
                    (settings_panel(text))
                }
                (scripts())
                script src="/app.js" {}
            }
        }
    })
}
pub(crate) fn submit_page(text: &Text) -> Html<String> {
    rendered(html! {
        (DOCTYPE)
        html lang=(text.lang()) {
            (head(text.t("submit-title")))
            body class="submit-page" {
                (topbar(text))
                main class="submit-main" {
                    p class="eyebrow" { (text.t("add-neighborhood")) }
                    h1 { (text.t("good-things")) br; (text.t("happen")) " " em { (text.t("submit-accent")) } }
                    p class="submit-lede" { (text.t("submit-lede")) }
                    form id="submit-form" class="submit-form" hx-post="/videos" hx-target="#form-message" hx-swap="innerHTML" {
                        label { (text.t("video-link")) input id="video-url" name="url" type="url" placeholder=(text.t("video-link-placeholder")) required; }
                        div id="embed-preview" class="embed-preview" aria-live="polite" data-reading=(text.t("reading-video")) data-unavailable=(text.t("preview-unavailable")) {}
                        label { (text.t("give-title")) input id="video-title" name="title" maxlength="90" placeholder=(text.t("title-placeholder")) required; }
                        div class="location-entry" {
                            div { span class="location-glyph" aria-hidden="true" { "◎" } div { b { (text.t("pin-it")) } small id="location-status" { (text.t("location-instructions")) } } }
                            button type="button" id="get-location" data-finding=(text.t("finding-location")) data-set=(text.t("location-set")) data-retry=(text.t("try-again")) data-unavailable=(text.t("location-unavailable")) data-pinned=(text.t("pinned-at")) data-adjust=(text.t("pin-adjust")) { (text.t("use-location")) }
                        }
                        div class="submit-map-wrap" {
                            div id="submit-map" class="submit-map" {}
                            button type="button" id="submit-map-toggle" class="map-expand" aria-controls="submit-map" aria-expanded="false" data-expand=(text.t("expand-map")) data-collapse=(text.t("collapse-map")) {
                                span class="map-expand-icon" aria-hidden="true" { "⤢" }
                                span class="map-expand-label" { (text.t("expand-map")) }
                            }
                        }
                        input type="hidden" name="lat" value="20.975";
                        input type="hidden" name="lng" value="-89.62";
                        button class="button-dark submit-button" type="submit" { (text.t("add-to-map")) " " span { "↗" } }
                        p class="form-footnote" { (text.t("public-video-note")) }
                        div id="form-message" role="status" {}
                    }
                }
                (scripts())
                script src="/submit.js" {}
            }
        }
    })
}
pub(crate) fn video_page(text: &Text, video: &Video) -> Html<String> {
    let platform = video.platform.to_uppercase();
    let mut args = FluentArgs::new();
    args.set("title", video.title.as_str());
    rendered(html! {
        (DOCTYPE)
        html lang=(text.lang()) {
            (head(text.with_args("detail-title", Some(&args))))
            body {
                (topbar(text))
                main class="detail-main" {
                    p class="eyebrow" { (text.t("moment-city")) }
                    article class="detail-card" {
                        @if !video.thumbnail.is_empty() {
                            a class="detail-thumb" href=(&video.url) target="_blank" rel="noopener" {
                                img src=(&video.thumbnail) alt="" loading="lazy" referrerpolicy="no-referrer";
                            }
                        }
                        div class="detail-copy" {
                            h1 { (&video.title) }
                            @if !video.creator.is_empty() || !video.neighborhood.is_empty() {
                                p {
                                    @if !video.creator.is_empty() { (text.t("shared-by")) " " b { (&video.creator) } }
                                    @if !video.creator.is_empty() && !video.neighborhood.is_empty() { " " }
                                    @if !video.neighborhood.is_empty() { (text.t("in-neighborhood")) " " b { (&video.neighborhood) } }
                                }
                            }
                            p class="detail-platform" { (platform) " " (text.t("video-merida")) }
                            div class="detail-actions" { a class="button-dark" href=(&video.url) target="_blank" rel="noopener" { (text.t("watch-original")) " " span { "↗" } } (detail_vote(text, video.id, video.votes)) }
                        }
                    }
                    div class="detail-map-wrap" { p class="eyebrow" { (text.t("right-corner")) } div class="detail-map" id="detail-map" data-lat=(video.lat) data-lng=(video.lng) data-title=(&video.title) {} }
                }
                (scripts())
                script src="/detail.js" {}
            }
        }
    })
}
