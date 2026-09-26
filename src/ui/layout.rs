use crate::i18n::Text;
use maud::{Markup, html};

pub(crate) fn head(title_text: String) -> Markup {
    html! { head {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover";
        meta name="theme-color" content="#101010";
        title { (title_text) }
        link rel="icon" type="image/svg+xml" href="/favicon.svg";
        link rel="stylesheet" href="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.css";
        link rel="stylesheet" href="/app.css";
    } }
}

pub(crate) fn topbar(text: &Text) -> Markup {
    html! { header class="topbar" {
        (super::components::brand(text))
        a class="back-link" href="/" { (text.t("back-map")) }
    } }
}

pub(crate) fn scripts() -> Markup {
    html! {
        script src="https://unpkg.com/htmx.org@2.0.4" {}
        script src="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.js" {}
        script src="/theme.js" {}
        script src="/map-common.js" {}
    }
}
