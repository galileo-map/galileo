//! TileJSON 3.0 specification parser.
//!
//! Implements parsing of [TileJSON](https://github.com/mapbox/tilejson-spec/blob/master/3.0.0/README.md)
//! manifest files, which describe metadata about map tile sets including tile endpoints,
//! zoom range, bounds, and vector layer descriptions.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer};
use serde_json::Value;

const DEFAULT_MINZOOM: u8 = 0;
const DEFAULT_MAXZOOM: u8 = 30;

fn deserialize_minzoom<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
    deserialize_u8_with_fallback(deserializer, DEFAULT_MINZOOM, "minzoom")
}

fn deserialize_maxzoom<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
    deserialize_u8_with_fallback(deserializer, DEFAULT_MAXZOOM, "maxzoom")
}

fn deserialize_u8_with_fallback<'de, D: Deserializer<'de>>(
    deserializer: D,
    fallback: u8,
    field_name: &str,
) -> Result<u8, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(fallback),
        v => serde_json::from_value::<u8>(v.clone()).or_else(|err| {
            log::warn!("Invalid {field_name} value {v}: {err}; using default ({fallback})");
            Ok(fallback)
        }),
    }
}

/// The tile coordinate scheme.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Scheme {
    /// XYZ — y increases downward (default, used by most web maps).
    #[default]
    Xyz,
    /// TMS — y increases upward.
    Tms,
}

/// Description of a single vector layer available in the tile set.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VectorLayer {
    /// Layer identifier, matching the layer name in the MVT data.
    pub id: String,

    /// Map of field names to their human-readable descriptions.
    pub fields: HashMap<String, String>,

    /// Human-readable description of this layer's contents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Lowest zoom level at which this layer appears.
    #[serde(default = "default_minzoom", deserialize_with = "deserialize_minzoom")]
    pub minzoom: u8,

    /// Highest zoom level at which this layer appears.
    #[serde(default = "default_maxzoom", deserialize_with = "deserialize_maxzoom")]
    pub maxzoom: u8,
}

fn default_minzoom() -> u8 {
    DEFAULT_MINZOOM
}
fn default_maxzoom() -> u8 {
    DEFAULT_MAXZOOM
}

/// A TileJSON 3.0 manifest describing a set of map tiles.
///
/// See the [TileJSON 3.0.0 specification](https://github.com/mapbox/tilejson-spec/blob/master/3.0.0/README.md)
/// for the full field reference.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TileJson {
    /// TileJSON spec version implemented by this document (e.g. `"3.0.0"`).
    pub tilejson: String,

    /// Tile endpoint URL templates. `{z}`, `{x}`, `{y}` are replaced with tile coordinates.
    pub tiles: Vec<String>,

    /// Vector layer descriptions. Required for vector tile sets.
    #[serde(default)]
    pub vector_layers: Vec<VectorLayer>,

    /// Human-readable name of the tile set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Human-readable description of the tile set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Attribution HTML to display when the map is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,

    /// Tile set version string (semver).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Minimum zoom level available.
    #[serde(default = "default_minzoom", deserialize_with = "deserialize_minzoom")]
    pub minzoom: u8,

    /// Maximum zoom level available.
    #[serde(default = "default_maxzoom", deserialize_with = "deserialize_maxzoom")]
    pub maxzoom: u8,

    /// Bounding box of available tiles as `[west, south, east, north]` in WGS 84.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_bounds"
    )]
    pub bounds: Option<[f64; 4]>,

    /// Default view as `[longitude, latitude, zoom]`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_opt_center"
    )]
    pub center: Option<[f64; 3]>,

    /// Tile coordinate scheme.
    #[serde(default)]
    pub scheme: Scheme,

    /// Zoom level from which to generate overzoomed tiles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fillzoom: Option<u8>,

    /// Legend text or HTML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legend: Option<String>,

    /// Mustache template for UTFGrid interactivity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// UTFGrid interactivity endpoint URL templates.
    #[serde(default)]
    pub grids: Vec<String>,

    /// GeoJSON data file URL templates.
    #[serde(default)]
    pub data: Vec<String>,
}

impl TileJson {
    /// Parse a TileJSON manifest from a JSON string.
    ///
    /// Unknown fields are logged at `warn` level and skipped. Returns an error
    /// only if the JSON is structurally invalid or required fields are missing.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let de = &mut serde_json::Deserializer::from_str(json);
        let tilejson: Self = serde_ignored::deserialize(de, |path| {
            log::warn!("Unrecognised TileJSON field: {path}");
        })?;
        Ok(tilejson)
    }
}

fn deserialize_opt_bounds<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<[f64; 4]>, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(None),
        v => match serde_json::from_value::<[f64; 4]>(v.clone()) {
            Ok(b) => Ok(Some(b)),
            Err(err) => {
                log::warn!("Invalid bounds value {v}: {err}; ignoring");
                Ok(None)
            }
        },
    }
}

fn deserialize_opt_center<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<[f64; 3]>, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(None),
        v => match serde_json::from_value::<[f64; 3]>(v.clone()) {
            Ok(c) => Ok(Some(c)),
            Err(err) => {
                log::warn!("Invalid center value {v}: {err}; ignoring");
                Ok(None)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"{
        "tilejson": "3.0.0",
        "tiles": ["https://example.com/{z}/{x}/{y}.pbf"]
    }"#;

    #[test]
    fn parse_minimal() {
        let tj = TileJson::from_json(MINIMAL).unwrap();
        assert_eq!(tj.tilejson, "3.0.0");
        assert_eq!(tj.tiles, ["https://example.com/{z}/{x}/{y}.pbf"]);
        assert_eq!(tj.minzoom, 0);
        assert_eq!(tj.maxzoom, 30);
        assert!(tj.vector_layers.is_empty());
        assert!(tj.name.is_none());
        assert!(tj.bounds.is_none());
        assert!(tj.center.is_none());
        assert_eq!(tj.scheme, Scheme::Xyz);
    }

    #[test]
    fn parse_optional_scalars() {
        let json = r#"{
            "tilejson": "3.0.0",
            "tiles": ["https://example.com/{z}/{x}/{y}.pbf"],
            "name": "Test Tiles",
            "description": "A test tileset",
            "attribution": "<a href='https://example.com'>Example</a>",
            "version": "1.2.3",
            "minzoom": 2,
            "maxzoom": 14,
            "scheme": "tms",
            "fillzoom": 10,
            "legend": "red = hot"
        }"#;
        let tj = TileJson::from_json(json).unwrap();
        assert_eq!(tj.name.as_deref(), Some("Test Tiles"));
        assert_eq!(tj.description.as_deref(), Some("A test tileset"));
        assert_eq!(tj.minzoom, 2);
        assert_eq!(tj.maxzoom, 14);
        assert_eq!(tj.scheme, Scheme::Tms);
        assert_eq!(tj.fillzoom, Some(10));
        assert_eq!(tj.version.as_deref(), Some("1.2.3"));
    }

    #[test]
    fn parse_bounds_and_center() {
        let json = r#"{
            "tilejson": "3.0.0",
            "tiles": ["https://example.com/{z}/{x}/{y}.pbf"],
            "bounds": [-180, -85.05, 180, 85.05],
            "center": [-76.27, 39.15, 8]
        }"#;
        let tj = TileJson::from_json(json).unwrap();
        assert_eq!(tj.bounds, Some([-180.0, -85.05, 180.0, 85.05]));
        assert_eq!(tj.center, Some([-76.27, 39.15, 8.0]));
    }

    #[test]
    fn parse_vector_layers() {
        let json = r#"{
            "tilejson": "3.0.0",
            "tiles": ["https://example.com/{z}/{x}/{y}.pbf"],
            "vector_layers": [
                {
                    "id": "roads",
                    "description": "Roads",
                    "minzoom": 5,
                    "maxzoom": 14,
                    "fields": {"name": "String", "class": "One of: primary, secondary"}
                },
                {
                    "id": "water",
                    "fields": {}
                }
            ]
        }"#;
        let tj = TileJson::from_json(json).unwrap();
        assert_eq!(tj.vector_layers.len(), 2);
        let roads = &tj.vector_layers[0];
        assert_eq!(roads.id, "roads");
        assert_eq!(roads.description.as_deref(), Some("Roads"));
        assert_eq!(roads.minzoom, 5);
        assert_eq!(roads.maxzoom, 14);
        assert_eq!(roads.fields["name"], "String");
        let water = &tj.vector_layers[1];
        assert_eq!(water.id, "water");
        assert!(water.fields.is_empty());
    }

    #[test]
    fn invalid_bounds_falls_back_to_none() {
        let json = r#"{
            "tilejson": "3.0.0",
            "tiles": ["https://example.com/{z}/{x}/{y}.pbf"],
            "bounds": "not-an-array"
        }"#;
        let tj = TileJson::from_json(json).unwrap();
        assert!(tj.bounds.is_none());
    }

    #[test]
    fn invalid_center_falls_back_to_none() {
        let json = r#"{
            "tilejson": "3.0.0",
            "tiles": ["https://example.com/{z}/{x}/{y}.pbf"],
            "center": "invalid"
        }"#;
        let tj = TileJson::from_json(json).unwrap();
        assert!(tj.center.is_none());
    }

    #[test]
    fn unknown_fields_are_tolerated() {
        let json = r#"{
            "tilejson": "3.0.0",
            "tiles": ["https://example.com/{z}/{x}/{y}.pbf"],
            "custom_extension": "some_value",
            "format": "pbf"
        }"#;
        TileJson::from_json(json).unwrap();
    }

    #[test]
    fn parse_maptiler_fixture() {
        let json = include_str!("../data/tiles.json");
        let tj = TileJson::from_json(json).unwrap();
        assert_eq!(tj.minzoom, 0);
        assert_eq!(tj.maxzoom, 15);
        assert!(!tj.vector_layers.is_empty());
        assert_eq!(tj.tiles.len(), 1);
        assert!(tj.tiles[0].contains("{VT_API_KEY}"));
    }
}
