//! Maplibre / Mapbox GL style specification — Rust representation.
//!
//! This module provides a type-safe, best-effort deserialisation of the
//! [MapLibre Style Specification v8](https://maplibre.org/maplibre-style-spec/).
//! Unknown or malformed values are logged at `warn` level via the `log` crate
//! and then skipped, so a partially-recognised style is still usable.
//!
//! # Submodules
//! - [`source`] — data source definitions (`vector`, `raster`, `geojson`, …)

pub mod color;
pub mod expression;
pub(super) mod helpers;
pub mod layer;
pub mod source;
pub mod value;

use std::collections::HashMap;

use layer::Layer;
use serde::Deserialize;
use serde_json::Value;
use source::Source;

/// A single sprite definition referencing an external sprite sheet.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SpriteEntry {
    /// Unique identifier for the sprite. When set to `"default"`, the prefix
    /// is omitted when referencing images from this sprite.
    pub id: String,
    /// URL of the sprite sheet (without the `.json`/`.png` extension; both are
    /// loaded automatically).
    pub url: String,
}

/// The sprite value in a MapLibre style.
///
/// Can be either a single URL string (legacy / backwards-compatible form) or
/// an array of [`SpriteEntry`] objects.
#[derive(Debug, Clone, PartialEq)]
pub enum Sprite {
    /// Legacy single-URL form.
    Url(String),
    /// Array of `{id, url}` objects.
    Entries(Vec<SpriteEntry>),
}

impl<'de> Deserialize<'de> for Sprite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match &value {
            Value::String(s) => Ok(Sprite::Url(s.clone())),
            Value::Array(_) => serde_json::from_value::<Vec<SpriteEntry>>(value)
                .map(Sprite::Entries)
                .map_err(serde::de::Error::custom),
            other => Err(serde::de::Error::custom(format!(
                "sprite must be a string or array, got {other}"
            ))),
        }
    }
}

/// Global transition timing defaults.
///
/// Used as the default transition for all paint properties that support
/// transitions unless overridden by a property-specific transition.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct Transition {
    /// Duration in milliseconds over which properties transition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    /// Delay in milliseconds before the transition begins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<f64>,
}

/// Root-level MapLibre GL style document (specification version 8).
///
/// A style's top-level properties define the map's layers, tile sources and
/// other rendering settings.  See the
/// [MapLibre Style Specification](https://maplibre.org/maplibre-style-spec/)
/// for the full reference.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MaplibreStyle {
    /// Must be `8`. Identifies the style specification version.
    pub version: u32,

    /// A human-readable name for the style.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Arbitrary properties useful to track with the stylesheet. These do not
    /// influence rendering. Properties should be prefixed to avoid collisions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// Default map center as `[longitude, latitude]`. Used only if the map has
    /// not been positioned by other means.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_center"
    )]
    pub center: Option<[f64; 2]>,

    /// Default map center altitude in metres above sea level.
    #[serde(
        rename = "centerAltitude",
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_f64"
    )]
    pub center_altitude: Option<f64>,

    /// Default zoom level. Used only if the map has not been positioned by
    /// other means.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_f64"
    )]
    pub zoom: Option<f64>,

    /// Default bearing in degrees. Zero points north. Used only if the map has
    /// not been positioned by other means.
    #[serde(default, deserialize_with = "deserialize_f64_or_default")]
    pub bearing: f64,

    /// Default pitch in degrees. Zero is a straight-down view. Used only if
    /// the map has not been positioned by other means.
    #[serde(default, deserialize_with = "deserialize_f64_or_default")]
    pub pitch: f64,

    /// Default roll in degrees, measured counterclockwise about the camera
    /// boresight. Used only if the map has not been positioned by other means.
    #[serde(default, deserialize_with = "deserialize_f64_or_default")]
    pub roll: f64,

    /// Data sources referenced by the style's layers.
    #[serde(deserialize_with = "deserialize_sources")]
    pub sources: HashMap<String, Source>,

    /// Sprite sheet URL or array of sprite entries.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_sprite"
    )]
    pub sprite: Option<Sprite>,

    /// URL template for loading SDF glyph sets in PBF format.
    /// Must contain `{fontstack}` and `{range}` tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glyphs: Option<String>,

    /// Global transition timing defaults.
    #[serde(default, deserialize_with = "deserialize_transition_or_default")]
    pub transition: Transition,

    /// Ordered list of layers to render.
    #[serde(deserialize_with = "deserialize_layers")]
    pub layers: Vec<Layer>,
}

impl MaplibreStyle {
    /// Parse a MapLibre style from a JSON string.
    ///
    /// Unknown top-level fields are logged at `warn` level. A warning is also
    /// emitted when `version` is not `8`. Individual sources or layers that
    /// cannot be understood are logged and skipped rather than failing the
    /// whole parse.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let de = &mut serde_json::Deserializer::from_str(json);
        let style: Self = serde_ignored::deserialize(de, |path| {
            log::warn!("Unrecognised style field: {path}");
        })?;

        if style.version != 8 {
            log::warn!(
                "Unsupported style version {} (expected 8); parsing will continue best-effort",
                style.version
            );
        }

        Ok(style)
    }
}

/// Deserialises the `sources` map, logging and skipping any entry that fails
/// to parse rather than hard-failing the whole document.
fn deserialize_sources<'de, D>(deserializer: D) -> Result<HashMap<String, Source>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: HashMap<String, Value> = HashMap::deserialize(deserializer)?;
    let mut out = HashMap::with_capacity(raw.len());
    for (key, val) in raw {
        let json_str = val.to_string();
        let de = &mut serde_json::Deserializer::from_str(&json_str);
        match serde_ignored::deserialize(de, |path| {
            log::warn!("Unrecognised field in source {key:?}: {path}");
        }) {
            Ok(src) => {
                out.insert(key, src);
            }
            Err(err) => {
                log::warn!("Failed to parse source {key:?}: {err}");
            }
        }
    }
    Ok(out)
}

/// Deserialises the `layers` array, logging and skipping any entry that cannot
/// be parsed (unknown type or malformed data) rather than failing the whole document.
fn deserialize_layers<'de, D>(deserializer: D) -> Result<Vec<Layer>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: Vec<Value> = Vec::deserialize(deserializer)?;
    let mut out = Vec::with_capacity(raw.len());
    for val in raw {
        let id = val
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let json_str = val.to_string();
        let de = &mut serde_json::Deserializer::from_str(&json_str);
        match serde_ignored::deserialize(de, |path| {
            log::warn!("Unrecognised field in layer {id:?}: {path}");
        }) {
            Ok(layer) => out.push(layer),
            Err(err) => log::warn!("Failed to parse layer {id:?}: {err}; skipping"),
        }
    }
    Ok(out)
}

/// Deserialises a required `f64` field, falling back to `Default` and logging
/// a warning when the value is present but not a valid number.
fn deserialize_f64_or_default<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(f64::default()),
        v => serde_json::from_value::<f64>(v.clone()).or_else(|err| {
            log::warn!("Invalid f64 value {v}: {err}; using default");
            Ok(f64::default())
        }),
    }
}

/// Deserialises an optional `f64` field, falling back to `None` and logging a
/// warning when the value is present but cannot be parsed as a number.
fn deserialize_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(None),
        v => match serde_json::from_value::<f64>(v.clone()) {
            Ok(n) => Ok(Some(n)),
            Err(err) => {
                log::warn!("Invalid f64 value {v}: {err}; ignoring field");
                Ok(None)
            }
        },
    }
}

/// Deserialises an optional `[f64; 2]` center field, falling back to `None`
/// and logging a warning on parse failure.
fn deserialize_opt_center<'de, D>(deserializer: D) -> Result<Option<[f64; 2]>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(None),
        v => match serde_json::from_value::<[f64; 2]>(v.clone()) {
            Ok(c) => Ok(Some(c)),
            Err(err) => {
                log::warn!("Invalid center value {v}: {err}; ignoring field");
                Ok(None)
            }
        },
    }
}

/// Deserialises an optional [`Sprite`] field, falling back to `None` and
/// logging a warning when the value cannot be parsed.
fn deserialize_opt_sprite<'de, D>(deserializer: D) -> Result<Option<Sprite>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(None),
        v => match serde_json::from_value::<Sprite>(v.clone()) {
            Ok(s) => Ok(Some(s)),
            Err(err) => {
                log::warn!("Invalid sprite value {v}: {err}; ignoring field");
                Ok(None)
            }
        },
    }
}

/// Deserialises a [`Transition`] field, falling back to `Default` and logging
/// a warning when the value is present but malformed.
fn deserialize_transition_or_default<'de, D>(deserializer: D) -> Result<Transition, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(Transition::default()),
        v => match serde_json::from_value::<Transition>(v.clone()) {
            Ok(t) => Ok(t),
            Err(err) => {
                log::warn!("Invalid transition value {v}: {err}; using default");
                Ok(Transition::default())
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal valid style — only required fields.
    const MINIMAL: &str = r#"{
        "version": 8,
        "sources": {},
        "layers": []
    }"#;

    #[test]
    fn parse_minimal_style() {
        let style = MaplibreStyle::from_json(MINIMAL).unwrap();
        assert_eq!(style.version, 8);
        assert!(style.name.is_none());
        assert!(style.sources.is_empty());
        assert!(style.layers.is_empty());
        assert_eq!(style.bearing, 0.0);
        assert_eq!(style.pitch, 0.0);
    }

    #[test]
    fn parse_optional_scalars() {
        let json = r#"{
            "version": 8,
            "name": "Test Style",
            "center": [-73.97, 40.77],
            "zoom": 12.5,
            "bearing": 29.0,
            "pitch": 50.0,
            "glyphs": "https://example.com/fonts/{fontstack}/{range}.pbf",
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert_eq!(style.name.as_deref(), Some("Test Style"));
        assert_eq!(style.center, Some([-73.97, 40.77]));
        assert_eq!(style.zoom, Some(12.5));
        assert_eq!(style.bearing, 29.0);
        assert_eq!(style.pitch, 50.0);
        assert_eq!(
            style.glyphs.as_deref(),
            Some("https://example.com/fonts/{fontstack}/{range}.pbf")
        );
    }

    #[test]
    fn parse_sprite_url() {
        let json = r#"{
            "version": 8,
            "sprite": "https://example.com/sprite",
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert_eq!(
            style.sprite,
            Some(Sprite::Url("https://example.com/sprite".into()))
        );
    }

    #[test]
    fn parse_sprite_entries() {
        let json = r#"{
            "version": 8,
            "sprite": [
                {"id": "default", "url": "https://example.com/sprite"},
                {"id": "extra",   "url": "https://example.com/extra-sprite"}
            ],
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        let Sprite::Entries(entries) = style.sprite.unwrap() else {
            panic!("expected Entries");
        };
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "default");
    }

    #[test]
    fn parse_transition() {
        let json = r#"{
            "version": 8,
            "transition": {"duration": 300, "delay": 0},
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert_eq!(style.transition.duration, Some(300.0));
        assert_eq!(style.transition.delay, Some(0.0));
    }

    #[test]
    fn parse_sources_in_style() {
        let json = r#"{
            "version": 8,
            "sources": {
                "my_tiles": {"type": "vector", "url": "https://example.com/tiles.json"}
            },
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert!(style.sources.contains_key("my_tiles"));
    }

    #[test]
    fn bad_source_is_skipped() {
        let json = r#"{
            "version": 8,
            "sources": {
                "good": {"type": "vector", "url": "https://example.com/tiles.json"},
                "bad":  {"type": "nonexistent"}
            },
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        // The bad source is skipped; the good one is retained.
        assert!(style.sources.contains_key("good"));
        assert!(!style.sources.contains_key("bad"));
    }

    #[test]
    fn parse_maptiler_style_root() {
        // Root-level fields from the maptiler_fmt.json fixture.
        let json = r#"{
            "version": 8,
            "id": "streets-v2",
            "name": "Streets",
            "sources": {
                "maptiler_attribution": {
                    "attribution": "© MapTiler",
                    "type": "vector"
                },
                "maptiler_planet": {
                    "url": "https://api.maptiler.com/tiles/v3/tiles.json?key=xxx",
                    "type": "vector"
                }
            },
            "layers": [],
            "glyphs": "https://api.maptiler.com/fonts/{fontstack}/{range}.pbf?key=xxx",
            "sprite": "https://api.maptiler.com/maps/streets-v2/sprite",
            "bearing": 0,
            "pitch": 0,
            "center": [0, 0],
            "zoom": 1
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert_eq!(style.version, 8);
        assert_eq!(style.name.as_deref(), Some("Streets"));
        assert_eq!(style.sources.len(), 2);
        assert_eq!(style.zoom, Some(1.0));
        assert_eq!(style.center, Some([0.0, 0.0]));
        assert!(matches!(style.sprite, Some(Sprite::Url(_))));
    }

    #[test]
    fn unknown_top_level_field_is_tolerated() {
        // "id" and "foo" are not in the spec; they should be ignored (logged).
        let json = r#"{
            "version": 8,
            "id": "my-style",
            "foo": 42,
            "sources": {},
            "layers": []
        }"#;
        // Must not return an error.
        MaplibreStyle::from_json(json).unwrap();
    }

    #[test]
    fn invalid_version_is_tolerated() {
        // Version 7 is unrecognised but we still parse best-effort.
        let json = r#"{
            "version": 7,
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert_eq!(style.version, 7);
    }

    #[test]
    fn invalid_bearing_falls_back_to_default() {
        let json = r#"{
            "version": 8,
            "bearing": "not-a-number",
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert_eq!(style.bearing, 0.0);
    }

    #[test]
    fn invalid_pitch_falls_back_to_default() {
        let json = r#"{
            "version": 8,
            "pitch": true,
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert_eq!(style.pitch, 0.0);
    }

    #[test]
    fn invalid_zoom_falls_back_to_none() {
        let json = r#"{
            "version": 8,
            "zoom": "far",
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert!(style.zoom.is_none());
    }

    #[test]
    fn invalid_center_falls_back_to_none() {
        let json = r#"{
            "version": 8,
            "center": "not-an-array",
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert!(style.center.is_none());
    }

    #[test]
    fn invalid_sprite_falls_back_to_none() {
        let json = r#"{
            "version": 8,
            "sprite": 42,
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert!(style.sprite.is_none());
    }

    #[test]
    fn invalid_transition_falls_back_to_default() {
        let json = r#"{
            "version": 8,
            "transition": "instant",
            "sources": {},
            "layers": []
        }"#;
        let style = MaplibreStyle::from_json(json).unwrap();
        assert_eq!(style.transition, Transition::default());
    }
}
