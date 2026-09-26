use crate::i18n::Text;
use maud::{Markup, html};

pub(crate) fn brand(text: &Text) -> Markup {
    html! { a class="wordmark" href="/" aria-label=(text.t("brand-map-label")) {
        span class="brand-name" { "MÉRIDA" span { "VIRAL" } }
    } }
}

pub(crate) fn settings_panel(text: &Text) -> Markup {
    html! { section class="settings-panel" id="settings-panel" aria-label=(text.t("appearance")) hidden {
        h2 { (text.t("appearance")) }
        p { (text.t("theme")) }
        div class="settings-options" {
            label { input type="radio" name="theme" value="system" checked; (text.t("system")) }
            label { input type="radio" name="theme" value="dark"; (text.t("dark")) }
            label { input type="radio" name="theme" value="light"; (text.t("light")) }
        }
        p { (text.t("accent")) }
        div class="settings-options accents" {
            label { input type="radio" name="accent" value="blue" checked; span class="swatch blue" {} (text.t("blue")) }
            label { input type="radio" name="accent" value="plum"; span class="swatch plum" {} (text.t("plum")) }
            label { input type="radio" name="accent" value="green"; span class="swatch green" {} (text.t("green")) }
        }
    } }
}
