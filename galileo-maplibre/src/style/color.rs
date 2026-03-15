//! CSS color parsing and serde helpers for [`galileo::Color`].

use std::ops::Deref;

use galileo::Color;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct MlColor(pub(crate) Color);

impl<'de> Deserialize<'de> for MlColor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_css_color(&s)
            .map(MlColor)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid CSS color: '{s}'")))
    }
}

impl From<MlColor> for Color {
    fn from(value: MlColor) -> Self {
        value.0
    }
}

impl Deref for MlColor {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Parses a CSS color string into a [`Color`].
///
/// Delegates to the `csscolorparser` crate, so all CSS color formats are supported.
/// Returns `None` for invalid input.
pub fn parse_css_color(s: &str) -> Option<Color> {
    let c = csscolorparser::parse(s).ok()?;
    Some(Color::rgba(
        (c.r * 255.0).round() as u8,
        (c.g * 255.0).round() as u8,
        (c.b * 255.0).round() as u8,
        (c.a * 255.0).round() as u8,
    ))
}

/// Deserializes an `Option<Color>` from a JSON string using CSS color parsing.
///
/// Intended for use with `#[serde(deserialize_with = "crate::style::color::deserialize_opt")]`.
/// Logs a warning and returns `None` for invalid color strings.
pub fn deserialize_opt<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(ref s) => {
            if let Some(color) = parse_css_color(s) {
                Ok(Some(color))
            } else {
                log::warn!("Failed to parse CSS color '{s}'; ignoring");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_short() {
        let c = parse_css_color("#f00").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (255, 0, 0));
    }

    #[test]
    fn hex_long() {
        let c = parse_css_color("#ff8000").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (255, 128, 0));
    }

    #[test]
    fn rgb() {
        let c = parse_css_color("rgb(10, 20, 30)").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (10, 20, 30));
    }

    #[test]
    fn rgba() {
        let c = parse_css_color("rgba(10, 20, 30, 0.5)").unwrap();
        assert_eq!((c.r(), c.g(), c.b(), c.a()), (10, 20, 30, 128));
    }

    #[test]
    fn hsl_red() {
        let c = parse_css_color("hsl(0, 100%, 50%)").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (255, 0, 0));
    }

    #[test]
    fn hsla() {
        let c = parse_css_color("hsla(0, 100%, 50%, 0.5)").unwrap();
        assert_eq!((c.r(), c.g(), c.b(), c.a()), (255, 0, 0, 128));
    }

    #[test]
    fn named_color() {
        let c = parse_css_color("red").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (255, 0, 0));
    }

    #[test]
    fn invalid_color_returns_none() {
        assert!(parse_css_color("not-a-color").is_none());
    }
}
