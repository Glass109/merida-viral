use maud::{Markup, html};
use serde::{Deserialize, Serialize};

/// Geographic coordinates in degrees: x is longitude, y is latitude.
pub(crate) trait MapCoordinates {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
}

/// A closed catalogue: callers choose a shape, never supply HTML.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PinStyle {
    #[default]
    GuitarPick,
    Skull,
    Flame,
    Star,
    Lightning,
    Coffin,
    Heart,
    Burst,
}

impl PinStyle {
    pub(crate) const ALL: [Self; 8] = [
        Self::GuitarPick,
        Self::Skull,
        Self::Flame,
        Self::Star,
        Self::Lightning,
        Self::Coffin,
        Self::Heart,
        Self::Burst,
    ];
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::GuitarPick => "guitar-pick",
            Self::Skull => "skull",
            Self::Flame => "flame",
            Self::Star => "star",
            Self::Lightning => "lightning",
            Self::Coffin => "coffin",
            Self::Heart => "heart",
            Self::Burst => "burst",
        }
    }
    pub(crate) fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|style| style.key() == value)
    }
}

pub(crate) trait MapPin: MapCoordinates {
    fn pin_name(&self) -> &str;
    fn pin_style(&self) -> PinStyle {
        PinStyle::default()
    }
    fn pin_image(&self) -> Option<&str> {
        None
    }
}

pub(crate) fn pin_art(style: PinStyle, image: Option<&str>) -> Markup {
    let source = image.unwrap_or("/noise.jpg");
    html! {
        span class=(format!("photo-pin pin-{}", style.key())) aria-hidden="true" {
            span class="photo-pin-outline" {}
            span class="photo-pin-rim" {
                img src=(source) alt="";
            }
            span class="photo-pin-fill" {
                img class=[image.is_none().then_some("photo-pin-placeholder")] src=(source) alt="";
                @if image.is_none() { span class="photo-pin-fallback" { "♪" } }
            }
        }
    }
}

pub(crate) fn pin_template(pin: &impl MapPin) -> Markup {
    html! { template class="map-pin-content" { (pin_art(pin.pin_style(), pin.pin_image())) } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_catalogue_shapes_are_accepted() {
        for style in PinStyle::ALL {
            assert_eq!(PinStyle::parse(style.key()), Some(style));
        }
        assert_eq!(PinStyle::parse("<script>alert(1)</script>"), None);
    }
    #[test]
    fn image_attributes_are_escaped() {
        let rendered = pin_art(PinStyle::Skull, Some("\" onerror=\"bad")).into_string();
        assert!(rendered.contains("&quot;"));
        assert!(!rendered.contains("src=\"\" onerror"));
    }
}
