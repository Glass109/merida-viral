use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use std::sync::OnceLock;
use topcoat::{
    context::Cx,
    router::{
        header::{HeaderValue, VARY},
        request,
        response::response_headers,
    },
};
use unic_langid::LanguageIdentifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Language {
    En,
    Es,
}

pub(crate) struct Text {
    language: Language,
    bundle: FluentBundle<&'static FluentResource>,
}

pub(crate) fn resource(language: Language) -> &'static FluentResource {
    static EN: OnceLock<FluentResource> = OnceLock::new();
    static ES: OnceLock<FluentResource> = OnceLock::new();
    let (cell, source) = match language {
        Language::En => (&EN, include_str!("locales/en.ftl")),
        Language::Es => (&ES, include_str!("locales/es.ftl")),
    };
    cell.get_or_init(|| FluentResource::try_new(source.to_owned()).expect("valid Fluent resource"))
}

impl Text {
    pub(crate) fn new(cx: &Cx) -> Self {
        response_headers(cx).append(VARY, HeaderValue::from_static("Accept-Language"));
        let preferred = request::headers(cx)
            .get("accept-language")
            .and_then(|header| header.to_str().ok())
            .unwrap_or("");
        let language = preferred_language(preferred);
        let id: LanguageIdentifier = match language {
            Language::En => "en",
            Language::Es => "es",
        }
        .parse()
        .expect("valid locale");
        let mut bundle = FluentBundle::new(vec![id]);
        bundle
            .add_resource(resource(language))
            .expect("unique Fluent messages");
        Self { language, bundle }
    }

    pub(crate) fn lang(&self) -> &'static str {
        if self.language == Language::Es {
            "es"
        } else {
            "en"
        }
    }

    pub(crate) fn t(&self, key: &str) -> String {
        self.with_args(key, None)
    }

    pub(crate) fn with_args(&self, key: &str, args: Option<&FluentArgs<'_>>) -> String {
        let message = self
            .bundle
            .get_message(key)
            .unwrap_or_else(|| panic!("missing translation: {key}"));
        let mut errors = Vec::new();
        let value = self
            .bundle
            .format_pattern(
                message.value().expect("translation value"),
                args,
                &mut errors,
            )
            .into_owned();
        assert!(errors.is_empty(), "translation errors in {key}: {errors:?}");
        value
    }
}

pub(crate) fn preferred_language(header: &str) -> Language {
    header
        .split(',')
        .filter_map(|entry| {
            let mut parts = entry.trim().split(';');
            let tag = parts.next()?.trim();
            let quality = parts
                .find_map(|part| {
                    part.trim()
                        .strip_prefix("q=")
                        .and_then(|q| q.parse::<f32>().ok())
                })
                .unwrap_or(1.0);
            if !(0.0..=1.0).contains(&quality) || quality == 0.0 {
                return None;
            }
            let lang = tag.split('-').next()?;
            if lang.eq_ignore_ascii_case("es") {
                Some((Language::Es, quality))
            } else if lang.eq_ignore_ascii_case("en") {
                Some((Language::En, quality))
            } else {
                None
            }
        })
        .fold((Language::En, 0.0_f32), |best, candidate| {
            if candidate.1 > best.1 {
                candidate
            } else {
                best
            }
        })
        .0
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_language_preferences_and_quality() {
        assert_eq!(preferred_language("es-MX,es;q=0.9,en;q=0.8"), Language::Es);
        assert_eq!(preferred_language("en,es;q=0.8"), Language::En);
        assert_eq!(preferred_language("es;q=0,en;q=0.6"), Language::En);
        assert_eq!(preferred_language("en,es"), Language::En);
        assert_eq!(preferred_language("fr-FR,es;q=0.7"), Language::Es);
    }

    #[test]
    fn resources_parse_and_contain_matching_messages() {
        let _ = resource(Language::En);
        let _ = resource(Language::Es);
        let keys = |source: &'static str| {
            source
                .lines()
                .filter_map(|line| line.split_once(" = ").map(|(key, _)| key))
                .collect::<std::collections::BTreeSet<_>>()
        };
        assert_eq!(
            keys(include_str!("locales/en.ftl")),
            keys(include_str!("locales/es.ftl"))
        );
    }
}
