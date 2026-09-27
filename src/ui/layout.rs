use crate::i18n::Text;
use maud::{Markup, html};

pub(crate) fn document_head(title_text: String) -> Markup {
    html! { head {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover";
        meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)";
        meta name="theme-color" content="#101010" media="(prefers-color-scheme: dark)";
        title { (title_text) }
        link rel="icon" type="image/svg+xml" href="/favicon.ico";
        link rel="preconnect" href="https://fonts.googleapis.com";
        link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous";
        link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Archivo+Black&family=Bebas+Neue&family=Permanent+Marker&family=Space+Grotesk:wght@400;500;600;700&display=swap";
        link rel="stylesheet" href="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.css";
        link rel="stylesheet" href="/app.css";
        // pagereveal must be registered before the first render, including BFCache restores.
        script src="/page-transitions.js" {}
    } }
}

pub(crate) fn shared_page_scripts() -> Markup {
    html! {
        script src="https://unpkg.com/htmx.org@2.0.4" {}
        script src="https://unpkg.com/maplibre-gl@4.7.1/dist/maplibre-gl.js" {}
        script src="/map-common.js" {}
    }
}

pub(crate) fn image_upload_field(text: &Text, id: &str) -> Markup {
    image_upload_field_with_value(text, id, "")
}

pub(crate) fn image_upload_field_with_value(text: &Text, id: &str, image_url: &str) -> Markup {
    html! {
        label for=(id) { (text.translate("picture")) }
        input id=(id) type="file" accept="image/jpeg,image/png,image/webp";
        input type="hidden" name="image_url" value=(image_url);
        p class="metro-hint" { (text.translate("picture-limit")) }
        p class="metro-upload-status" role="status" aria-live="polite" {}
    }
}
