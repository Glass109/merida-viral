use super::model::Video;
use crate::{
    i18n::Text,
    map_pin::{self, MapCoordinates, MapPin},
    ui::layout::{document_head, shared_page_scripts},
};
use fluent_bundle::FluentArgs;
use maud::{DOCTYPE, Markup, html};
use topcoat::router::content::Html;
fn html_response(markup: Markup) -> Html<String> {
    Html(markup.into_string())
}

/// Join the non-empty parts of a place's meta line with a separator.
fn place_metadata_line(video: &Video) -> String {
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

pub(crate) fn places_list_fragment(text: &Text, videos: &[Video]) -> Html<String> {
    html_response(html! {
        @if videos.is_empty() { p class="loading" { (text.translate("no-places")) } }
        @for (i, v) in videos.iter().enumerate() {
            article class="place-row" data-lat=(v.y()) data-lng=(v.x()) data-title=(v.pin_name()) {
                @let mut args = FluentArgs::new();
                @let _ = args.set("title", v.title.as_str());
                button class="place-number" type="button" aria-label=(text.translate_with_args("show-on-map", Some(&args))) { (format!("{:02}", i + 1)) }
                div class="place-copy" {
                    a class="place-title" href=(format!("/video/{}", v.id)) { (&v.title) }
                    div class="place-meta" { (place_metadata_line(v)) }
                }
                button class="place-vote" hx-post="/vote" hx-vals=(format!("{{\"id\":{}}}", v.id)) hx-target="#video-grid" hx-swap="innerHTML" aria-label=(text.translate_with_args("vote-label", Some(&args))) { "↑ " (v.votes) }
                (map_pin::pin_template(v))
            }
        }
    })
}
pub(crate) fn video_detail_vote_button(text: &Text, id: i64, votes: i64) -> Markup {
    html! { button class="button-dark detail-vote" id="detail-vote" hx-post="/video-vote" hx-vals=(format!("{{\"id\":{id}}}")) hx-target="#detail-vote" hx-swap="outerHTML" { (text.translate("upvote")) " · " (votes) span { "↑" } } }
}

pub(crate) fn home_page(text: &Text) -> Html<String> {
    html_response(html! {
        (DOCTYPE)
        html lang=(text.language_tag()) {
            (document_head(text.translate("app-title")))
            body class="map-app" {
                main class="map-workspace" data-view="places" {
                    section class="map-tools" id="places-view" aria-label=(text.translate("places")) {
                        div class="map-heading" {
                            span class="section-kicker" { (text.translate("explore-city")) }
                            h1 { (text.translate("heading-first")) " " (text.translate("heading-second")) br; em { (text.translate("heading-third")) } }
                            nav class="metro-collection-links" aria-label=(text.translate("sections")) {
                                a href="/events" { (text.translate("events")) }
                                a href="/artists" { (text.translate("artists")) }
                                a href="/bands" { (text.translate("bands")) }
                                a href="/venues" { (text.translate("venues")) }
                            }
                            p { strong id="video-count" { "…" } " " (text.translate("trending-nearby")) }
                        }
                        section class="place-list" id="video-grid" data-error=(text.translate("places-error")) hx-get="/places" hx-trigger="load" hx-swap="innerHTML" aria-live="polite" { p class="loading" { (text.translate("finding")) } }
                    }
                    section class="map-view" id="map-view" aria-label=(text.translate("map-label")) {
                        div class="map-view-heading" { span class="section-kicker" { (text.translate("explore-city")) } h1 { (text.translate("map")) } }
                        div id="map" class="map" aria-label=(text.translate("map-label")) {}
                        div class="map-controls" {
                            button id="locate" type="button" aria-label=(text.translate("my-location-label")) title=(text.translate("my-location-label")) { "◎" }
                            button id="zoom-in" type="button" aria-label=(text.translate("zoom-in")) { "+" }
                            button id="zoom-out" type="button" aria-label=(text.translate("zoom-out")) { "−" }
                        }
                    }
                    nav class="app-bar" aria-label=(text.translate("actions")) {
                        button class="bar-action" id="view-toggle" type="button" aria-controls="map-view" aria-expanded="false" data-map=(text.translate("map")) data-places=(text.translate("places")) { span class="bar-icon" aria-hidden="true" { "▤" } span class="bar-label" { (text.translate("map")) } }
                        button class="bar-action" id="sort-toggle" type="button" data-sort="trending" data-trending=(text.translate("trending")) data-newest=(text.translate("newest")) data-sort-trending=(text.translate("sort-by-trending")) data-sort-newest=(text.translate("sort-by-newest")) aria-label=(text.translate("sort-by-trending")) aria-pressed="true" {
                            span class="bar-icon" aria-hidden="true" {
                                svg class="sort-icon-trending" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {
                                    path d="M13.7489 5.5L21.3027 5.50052L21.403 5.51444L21.5018 5.54205L21.5621 5.5676C21.6413 5.60246 21.7155 5.65315 21.7808 5.71836L21.8215 5.7624L21.865 5.81878L21.9192 5.9089L21.9579 5.99922L21.977 6.0633L21.9906 6.1273L22 6.2215L22.0004 13.7539C22.0004 14.1681 21.6647 14.5039 21.2504 14.5039C20.8708 14.5039 20.557 14.2217 20.5073 13.8557L20.5004 13.7539L20.5 8.059L12.7812 15.7793C12.5149 16.0455 12.0982 16.0698 11.8046 15.8519L11.7205 15.7793L8.75001 12.8089L3.28033 18.2786C2.98744 18.5715 2.51256 18.5715 2.21967 18.2786C1.9534 18.0123 1.9292 17.5957 2.14705 17.3021L2.21967 17.2179L8.21967 11.2179C8.48593 10.9517 8.90259 10.9275 9.1962 11.1453L9.28032 11.2179L12.2508 14.1883L19.438 7H13.7489C13.3692 7 13.0554 6.71785 13.0058 6.35177L12.9989 6.25C12.9989 5.8703 13.2811 5.55651 13.6472 5.50685L13.7489 5.5Z" fill="currentColor";
                                }
                                svg class="sort-icon-newest" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" {
                                    path d="M3.5 12C3.5 7.30558 7.30558 3.5 12 3.5C16.6944 3.5 20.5 7.30558 20.5 12C20.5 16.6944 16.6944 20.5 12 20.5C7.30558 20.5 3.5 16.6944 3.5 12ZM12 2C6.47715 2 2 6.47715 2 12C2 17.5228 6.47715 22 12 22C17.5228 22 22 17.5228 22 12C22 6.47715 17.5228 2 12 2ZM11.9931 6.64827C11.9435 6.28233 11.6295 6 11.25 6C10.836 6 10.5 6.336 10.5 6.75V12.75L10.5069 12.8517C10.5565 13.2177 10.8706 13.5 11.25 13.5H15.25L15.3517 13.4931C15.7177 13.4435 16 13.1297 16 12.75C16 12.336 15.664 12 15.25 12H12V6.75L11.9931 6.64827Z" fill="currentColor";
                                }
                            }
                            span class="bar-label" { (text.translate("trending")) }
                        }
                        a class="bar-action" href="/submit" { span class="bar-icon" aria-hidden="true" { "+" } span class="bar-label" { (text.translate("post")) } }
                    }
                }
                (shared_page_scripts())
                script src="/app.js" {}
            }
        }
    })
}
pub(crate) fn video_submission_page(text: &Text) -> Html<String> {
    html_response(html! {
        (DOCTYPE)
        html lang=(text.language_tag()) {
            (document_head(text.translate("submit-title")))
            body class="submit-page" {
                main class="submit-main" {
                    p class="eyebrow" { (text.translate("add-neighborhood")) }
                    h1 { (text.translate("good-things")) br; (text.translate("happen")) " " em { (text.translate("submit-accent")) } }
                    p class="submit-lede" { (text.translate("submit-lede")) }
                    form id="submit-form" class="submit-form" hx-post="/videos" hx-target="#form-message" hx-swap="innerHTML" {
                        label { (text.translate("video-link")) input id="video-url" name="url" type="url" placeholder=(text.translate("video-link-placeholder")) required; }
                        div id="embed-preview" class="embed-preview" aria-live="polite" data-reading=(text.translate("reading-video")) data-unavailable=(text.translate("preview-unavailable")) {}
                        label { (text.translate("give-title")) input id="video-title" name="title" maxlength="90" placeholder=(text.translate("title-placeholder")) required; }
                        div class="location-entry" {
                            div { span class="location-glyph" aria-hidden="true" { "◎" } div { b { (text.translate("pin-it")) } small id="location-status" { (text.translate("location-instructions")) } } }
                            button type="button" id="get-location" data-finding=(text.translate("finding-location")) data-set=(text.translate("location-set")) data-retry=(text.translate("try-again")) data-unavailable=(text.translate("location-unavailable")) data-pinned=(text.translate("pinned-at")) data-adjust=(text.translate("pin-adjust")) { (text.translate("use-location")) }
                        }
                        div class="submit-map-wrap" {
                            div id="submit-map" class="submit-map" {}
                            button type="button" id="submit-map-toggle" class="map-expand" aria-controls="submit-map" aria-expanded="false" data-expand=(text.translate("expand-map")) data-collapse=(text.translate("collapse-map")) {
                                span class="map-expand-icon" aria-hidden="true" { "⤢" }
                                span class="map-expand-label" { (text.translate("expand-map")) }
                            }
                        }
                        input type="hidden" name="lat" value="20.975";
                        input type="hidden" name="lng" value="-89.62";
                        button class="button-dark submit-button" type="submit" { (text.translate("add-to-map")) " " span { "↗" } }
                        p class="form-footnote" { (text.translate("public-video-note")) }
                        div id="form-message" role="status" {}
                    }
                }
                (shared_page_scripts())
                script src="/submit.js" {}
            }
        }
    })
}
pub(crate) fn video_detail_page(text: &Text, video: &Video) -> Html<String> {
    let platform = video.platform.to_uppercase();
    let mut args = FluentArgs::new();
    args.set("title", video.title.as_str());
    html_response(html! {
        (DOCTYPE)
        html lang=(text.language_tag()) {
            (document_head(text.translate_with_args("detail-title", Some(&args))))
            body {
                main class="detail-main" {
                    p class="eyebrow" { (text.translate("moment-city")) }
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
                                    @if !video.creator.is_empty() { (text.translate("shared-by")) " " b { (&video.creator) } }
                                    @if !video.creator.is_empty() && !video.neighborhood.is_empty() { " " }
                                    @if !video.neighborhood.is_empty() { (text.translate("in-neighborhood")) " " b { (&video.neighborhood) } }
                                }
                            }
                            p class="detail-platform" { (platform) " " (text.translate("video-merida")) }
                            div class="detail-actions" { a class="button-dark" href=(&video.url) target="_blank" rel="noopener" { (text.translate("watch-original")) " " span { "↗" } } (video_detail_vote_button(text, video.id, video.votes)) }
                        }
                    }
                    div class="detail-map-wrap" { p class="eyebrow" { (text.translate("right-corner")) } div class="detail-map" id="detail-map" data-lat=(video.y()) data-lng=(video.x()) data-title=(video.pin_name()) {} (map_pin::pin_template(video)) }
                }
                (shared_page_scripts())
                script src="/detail.js" {}
            }
        }
    })
}
