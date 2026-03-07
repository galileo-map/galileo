//! Style sources — specifications of data that map layers are drawn from.
//!
//! Sources state which data the map should displayfn default_minzoom() -> f64 {f source
//! with the `type` property. Adding a source isn't enough to make data appear
//! on the map because sources don't contain styling details like color or width.
//! Layers refer to a source and give it a visual representation. This makes it
//! possible to style the same source in different ways.

use serde::Deserialize;
use serde_json::Value;

use super::helpers::deserialize_f64_with_fallback;

const DEFAULT_MINZOOM: f64 = 0.0;
const DEFAULT_MAXZOOM: f64 = 22.0;
const DEFAULT_TILE_SIZE: f64 = 512.0;
const DEFAULT_FACTOR: f64 = 1.0;
const DEFAULT_GEOJSON_MAXZOOM: f64 = 18.0;
const DEFAULT_BUFFER: f64 = 128.0;
const DEFAULT_TOLERANCE: f64 = 0.375;
const DEFAULT_CLUSTER_RADIUS: f64 = 50.0;

/// Deserialises an `f64` field that defaults to [`DEFAULT_MINZOOM`].
fn deserialize_minzoom<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_MINZOOM, "minzoom")
}

/// Deserialises an `f64` field that defaults to [`DEFAULT_MAXZOOM`].
fn deserialize_maxzoom<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_MAXZOOM, "maxzoom")
}

/// Deserialises an `f64` field that defaults to [`DEFAULT_GEOJSON_MAXZOOM`].
fn deserialize_geojson_maxzoom<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_GEOJSON_MAXZOOM, "maxzoom")
}

/// Deserialises an `f64` field that defaults to [`DEFAULT_TILE_SIZE`].
fn deserialize_tile_size<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_TILE_SIZE, "tileSize")
}

/// Deserialises an `f64` field that defaults to [`DEFAULT_FACTOR`].
fn deserialize_factor<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_FACTOR, "factor")
}

/// Deserialises an `f64` field that defaults to [`DEFAULT_BUFFER`].
fn deserialize_buffer<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_BUFFER, "buffer")
}

/// Deserialises an `f64` field that defaults to [`DEFAULT_TOLERANCE`].
fn deserialize_tolerance<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_TOLERANCE, "tolerance")
}

/// Deserialises an `f64` field that defaults to [`DEFAULT_CLUSTER_RADIUS`].
fn deserialize_cluster_radius<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_CLUSTER_RADIUS, "clusterRadius")
}

/// Deserialises a [`TileScheme`] field, falling back to `Default` on error.
fn deserialize_tile_scheme_or_default<'de, D>(deserializer: D) -> Result<TileScheme, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(TileScheme::default()),
        v => serde_json::from_value::<TileScheme>(v.clone()).or_else(|err| {
            log::warn!("Invalid tile scheme {v}: {err}; using default");
            Ok(TileScheme::default())
        }),
    }
}

/// Deserialises a [`VectorEncoding`] field, falling back to `Default` on error.
fn deserialize_vector_encoding_or_default<'de, D>(
    deserializer: D,
) -> Result<VectorEncoding, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(VectorEncoding::default()),
        v => serde_json::from_value::<VectorEncoding>(v.clone()).or_else(|err| {
            log::warn!("Invalid vector encoding {v}: {err}; using default");
            Ok(VectorEncoding::default())
        }),
    }
}

/// Deserialises a [`DemEncoding`] field, falling back to `Default` on error.
fn deserialize_dem_encoding_or_default<'de, D>(deserializer: D) -> Result<DemEncoding, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(DemEncoding::default()),
        v => serde_json::from_value::<DemEncoding>(v.clone()).or_else(|err| {
            log::warn!("Invalid DEM encoding {v}: {err}; using default");
            Ok(DemEncoding::default())
        }),
    }
}

/// Tile coordinate scheme.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TileScheme {
    /// Slippy map tilenames scheme (default).
    #[default]
    Xyz,
    /// OSGeo spec scheme.
    Tms,
}

/// Encoding for vector tile sources.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VectorEncoding {
    /// Mapbox Vector Tiles (default).
    #[default]
    Mvt,
    /// MapLibre Vector Tiles.
    Mlt,
}

/// Encoding for raster DEM sources.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DemEncoding {
    /// Terrarium format PNG tiles.
    Terrarium,
    /// Mapbox Terrain RGB tiles (default).
    #[default]
    Mapbox,
    /// Custom decoding using redFactor, blueFactor, greenFactor, baseShift.
    Custom,
}

/// A vector tile source. Provides tiled vector data.
///
/// Either `url` (a TileJSON URL) or `tiles` must be provided.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VectorSource {
    /// A URL to a TileJSON resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// An array of one or more tile source URLs, as in the TileJSON spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<String>>,

    /// Bounding box `[sw.lng, sw.lat, ne.lng, ne.lat]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[f64; 4]>,

    /// Tile coordinate scheme. Defaults to `xyz`.
    #[serde(default, deserialize_with = "deserialize_tile_scheme_or_default")]
    pub scheme: TileScheme,

    /// Minimum zoom level for which tiles are available.
    #[serde(default = "default_minzoom", deserialize_with = "deserialize_minzoom")]
    pub minzoom: f64,

    /// Maximum zoom level for which tiles are available.
    #[serde(default = "default_maxzoom", deserialize_with = "deserialize_maxzoom")]
    pub maxzoom: f64,

    /// Attribution text to display when the map is shown to a user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,

    /// A property to use as a feature id (for feature state).
    #[serde(rename = "promoteId", skip_serializing_if = "Option::is_none")]
    pub promote_id: Option<Value>,

    /// Whether the source's tiles are cached locally.
    #[serde(default)]
    pub volatile: bool,

    /// The encoding used by this source.
    #[serde(default, deserialize_with = "deserialize_vector_encoding_or_default")]
    pub encoding: VectorEncoding,
}

/// A raster tile source. Provides tiled raster images.
///
/// Either `url` (a TileJSON URL) or `tiles` must be provided.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RasterSource {
    /// A URL to a TileJSON resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// An array of one or more tile source URLs, as in the TileJSON spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<String>>,

    /// Bounding box `[sw.lng, sw.lat, ne.lng, ne.lat]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[f64; 4]>,

    /// Minimum zoom level for which tiles are available.
    #[serde(default = "default_minzoom", deserialize_with = "deserialize_minzoom")]
    pub minzoom: f64,

    /// Maximum zoom level for which tiles are available.
    #[serde(default = "default_maxzoom", deserialize_with = "deserialize_maxzoom")]
    pub maxzoom: f64,

    /// The minimum visual size to display tiles for this layer, in pixels.
    #[serde(
        rename = "tileSize",
        default = "default_tile_size",
        deserialize_with = "deserialize_tile_size"
    )]
    pub tile_size: f64,

    /// Tile coordinate scheme. Defaults to `xyz`.
    #[serde(default, deserialize_with = "deserialize_tile_scheme_or_default")]
    pub scheme: TileScheme,

    /// Attribution text to display when the map is shown to a user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,

    /// Whether the source's tiles are cached locally.
    #[serde(default)]
    pub volatile: bool,
}

/// A raster DEM (digital elevation model) source.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RasterDemSource {
    /// A URL to a TileJSON resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// An array of one or more tile source URLs, as in the TileJSON spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tiles: Option<Vec<String>>,

    /// Bounding box `[sw.lng, sw.lat, ne.lng, ne.lat]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[f64; 4]>,

    /// Minimum zoom level for which tiles are available.
    #[serde(default = "default_minzoom", deserialize_with = "deserialize_minzoom")]
    pub minzoom: f64,

    /// Maximum zoom level for which tiles are available.
    #[serde(default = "default_maxzoom", deserialize_with = "deserialize_maxzoom")]
    pub maxzoom: f64,

    /// The minimum visual size to display tiles for this layer, in pixels.
    #[serde(
        rename = "tileSize",
        default = "default_tile_size",
        deserialize_with = "deserialize_tile_size"
    )]
    pub tile_size: f64,

    /// Attribution text to display when the map is shown to a user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,

    /// The encoding used by this DEM source.
    #[serde(default, deserialize_with = "deserialize_dem_encoding_or_default")]
    pub encoding: DemEncoding,

    /// Multiplied by the red channel value when decoding (custom encoding only).
    #[serde(
        rename = "redFactor",
        default = "default_factor",
        deserialize_with = "deserialize_factor"
    )]
    pub red_factor: f64,

    /// Multiplied by the blue channel value when decoding (custom encoding only).
    #[serde(
        rename = "blueFactor",
        default = "default_factor",
        deserialize_with = "deserialize_factor"
    )]
    pub blue_factor: f64,

    /// Multiplied by the green channel value when decoding (custom encoding only).
    #[serde(
        rename = "greenFactor",
        default = "default_factor",
        deserialize_with = "deserialize_factor"
    )]
    pub green_factor: f64,

    /// Added to the encoding mix when decoding (custom encoding only).
    #[serde(rename = "baseShift", default)]
    pub base_shift: f64,

    /// Whether the source's tiles are cached locally.
    #[serde(default)]
    pub volatile: bool,
}

/// A GeoJSON data source.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GeoJsonSource {
    /// A URL to a GeoJSON file, or inline GeoJSON.
    pub data: Value,

    /// Maximum zoom level at which to create vector tiles.
    #[serde(
        default = "default_geojson_maxzoom",
        deserialize_with = "deserialize_geojson_maxzoom"
    )]
    pub maxzoom: f64,

    /// Attribution text to display when the map is shown to a user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,

    /// Tile buffer size on each side (0–512).
    #[serde(default = "default_buffer", deserialize_with = "deserialize_buffer")]
    pub buffer: f64,

    /// Douglas-Peucker simplification tolerance.
    #[serde(
        default = "default_tolerance",
        deserialize_with = "deserialize_tolerance"
    )]
    pub tolerance: f64,

    /// Whether to cluster point features by radius.
    #[serde(default)]
    pub cluster: bool,

    /// Radius of each cluster if clustering is enabled.
    #[serde(
        rename = "clusterRadius",
        default = "default_cluster_radius",
        deserialize_with = "deserialize_cluster_radius"
    )]
    pub cluster_radius: f64,

    /// Max zoom on which to cluster points.
    #[serde(rename = "clusterMaxZoom", skip_serializing_if = "Option::is_none")]
    pub cluster_max_zoom: Option<f64>,

    /// Minimum number of points required to form a cluster.
    #[serde(rename = "clusterMinPoints", skip_serializing_if = "Option::is_none")]
    pub cluster_min_points: Option<f64>,

    /// Custom aggregation properties on generated clusters.
    #[serde(rename = "clusterProperties", skip_serializing_if = "Option::is_none")]
    pub cluster_properties: Option<Value>,

    /// Whether to calculate line distance metrics (required for `line-gradient`).
    #[serde(rename = "lineMetrics", default)]
    pub line_metrics: bool,

    /// Whether to auto-assign feature ids based on array index.
    #[serde(rename = "generateId", default)]
    pub generate_id: bool,

    /// A property to use as a feature id (for feature state).
    #[serde(rename = "promoteId", skip_serializing_if = "Option::is_none")]
    pub promote_id: Option<Value>,
}

/// A video data source.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VideoSource {
    /// URLs to video content in order of preferred format.
    pub urls: Vec<String>,

    /// Corners of the video specified as `[[lng, lat]; 4]`.
    pub coordinates: [[f64; 2]; 4],
}

/// An image data source.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ImageSource {
    /// URL pointing to the image.
    pub url: String,

    /// Corners of the image specified as `[[lng, lat]; 4]`.
    pub coordinates: [[f64; 2]; 4],
}

/// A map data source — one of the supported source types.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    /// A vector tile source.
    #[serde(rename = "vector")]
    Vector(VectorSource),
    /// A raster tile source.
    #[serde(rename = "raster")]
    Raster(RasterSource),
    /// A raster DEM source.
    #[serde(rename = "raster-dem")]
    RasterDem(RasterDemSource),
    /// A GeoJSON data source.
    #[serde(rename = "geojson")]
    GeoJson(GeoJsonSource),
    /// A video data source.
    #[serde(rename = "video")]
    Video(VideoSource),
    /// An image data source.
    #[serde(rename = "image")]
    Image(ImageSource),
}

fn default_minzoom() -> f64 {
    DEFAULT_MINZOOM
}
fn default_maxzoom() -> f64 {
    DEFAULT_MAXZOOM
}
fn default_tile_size() -> f64 {
    DEFAULT_TILE_SIZE
}
fn default_factor() -> f64 {
    DEFAULT_FACTOR
}
fn default_geojson_maxzoom() -> f64 {
    DEFAULT_GEOJSON_MAXZOOM
}
fn default_buffer() -> f64 {
    DEFAULT_BUFFER
}
fn default_tolerance() -> f64 {
    DEFAULT_TOLERANCE
}
fn default_cluster_radius() -> f64 {
    DEFAULT_CLUSTER_RADIUS
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn parse_vector_source_with_url() {
        let json = r#"{
            "type": "vector",
            "url": "https://example.com/tiles.json",
            "attribution": "© Example"
        }"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Vector(v) = source else {
            panic!("expected Vector source");
        };
        assert_eq!(v.url.as_deref(), Some("https://example.com/tiles.json"));
        assert_eq!(v.attribution.as_deref(), Some("© Example"));
        assert_eq!(v.minzoom, DEFAULT_MINZOOM);
        assert_eq!(v.maxzoom, DEFAULT_MAXZOOM);
        assert_eq!(v.scheme, TileScheme::Xyz);
        assert_eq!(v.encoding, VectorEncoding::Mvt);
    }

    #[test]
    fn parse_vector_source_with_tiles() {
        let json = r#"{
            "type": "vector",
            "tiles": ["https://a.example.com/{z}/{x}/{y}.pbf"],
            "maxzoom": 14,
            "scheme": "tms"
        }"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Vector(v) = source else {
            panic!("expected Vector source");
        };
        assert_eq!(
            v.tiles,
            Some(vec!["https://a.example.com/{z}/{x}/{y}.pbf".to_string()])
        );
        assert_eq!(v.maxzoom, 14.0);
        assert_eq!(v.scheme, TileScheme::Tms);
    }

    #[test]
    fn parse_raster_source() {
        let json = r#"{
            "type": "raster",
            "tiles": ["https://example.com/{z}/{x}/{y}.png"],
            "tileSize": 256
        }"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Raster(r) = source else {
            panic!("expected Raster source");
        };
        assert_eq!(r.tile_size, 256.0);
        assert_eq!(r.scheme, TileScheme::Xyz);
    }

    #[test]
    fn parse_raster_dem_source() {
        let json = r#"{
            "type": "raster-dem",
            "url": "https://example.com/dem.json",
            "encoding": "terrarium"
        }"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::RasterDem(d) = source else {
            panic!("expected RasterDem source");
        };
        assert_eq!(d.encoding, DemEncoding::Terrarium);
        assert_eq!(d.red_factor, DEFAULT_FACTOR);
    }

    #[test]
    fn parse_geojson_source() {
        let json = r#"{
            "type": "geojson",
            "data": "https://example.com/data.geojson",
            "cluster": true,
            "clusterRadius": 80
        }"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::GeoJson(g) = source else {
            panic!("expected GeoJson source");
        };
        assert!(g.cluster);
        assert_eq!(g.cluster_radius, 80.0);
        assert_eq!(g.tolerance, DEFAULT_TOLERANCE);
    }

    #[test]
    fn parse_video_source() {
        let json = r#"{
            "type": "video",
            "urls": ["https://example.com/video.mp4"],
            "coordinates": [
                [-122.51596391201019, 37.56238816766053],
                [-122.51467645168304, 37.56410183312965],
                [-122.51309394836426, 37.563391708549425],
                [-122.51423120498657, 37.56161849366671]
            ]
        }"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Video(v) = source else {
            panic!("expected Video source");
        };
        assert_eq!(v.urls, vec!["https://example.com/video.mp4"]);
    }

    #[test]
    fn parse_image_source() {
        let json = r#"{
            "type": "image",
            "url": "https://example.com/image.png",
            "coordinates": [
                [-80.425, 46.437],
                [-71.516, 46.437],
                [-71.516, 37.936],
                [-80.425, 37.936]
            ]
        }"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Image(i) = source else {
            panic!("expected Image source");
        };
        assert_eq!(i.url, "https://example.com/image.png");
    }

    #[test]
    fn unknown_source_type_returns_error() {
        let json = r#"{"type": "unknown"}"#;
        assert!(serde_json::from_str::<Source>(json).is_err());
    }

    #[test]
    fn deserialize_sources_map_skips_invalid() {
        let json = r#"{
            "good": {"type": "vector", "url": "https://example.com/tiles.json"},
            "bad":  {"type": "not-a-real-type"}
        }"#;
        let mut map: HashMap<String, serde_json::Value> = serde_json::from_str(json).unwrap();
        // Test the helper via a manual iteration (mirrors what deserialize_sources does)
        let mut out = HashMap::new();
        for (key, val) in map.drain() {
            if let Ok(s) = serde_json::from_value::<Source>(val) {
                out.insert(key, s);
            }
        }
        assert!(out.contains_key("good"));
        assert!(!out.contains_key("bad"));
    }

    #[test]
    fn parse_maptiler_sources() {
        // Matches the sources block from the maptiler_fmt.json fixture.
        let json = r#"{
            "maptiler_attribution": {
                "attribution": "<a href=\"https://www.maptiler.com/copyright/\">© MapTiler</a>",
                "type": "vector"
            },
            "maptiler_planet": {
                "url": "https://api.maptiler.com/tiles/v3/tiles.json?key=xxx",
                "type": "vector"
            }
        }"#;
        let mut map: HashMap<String, serde_json::Value> = serde_json::from_str(json).unwrap();
        let mut out = HashMap::new();
        for (key, val) in map.drain() {
            if let Ok(s) = serde_json::from_value::<Source>(val) {
                out.insert(key, s);
            }
        }
        assert_eq!(out.len(), 2);
        let Source::Vector(planet) = &out["maptiler_planet"] else {
            panic!("expected vector");
        };
        assert_eq!(
            planet.url.as_deref(),
            Some("https://api.maptiler.com/tiles/v3/tiles.json?key=xxx")
        );
    }

    #[test]
    fn invalid_minzoom_falls_back_to_default() {
        let json = r#"{"type": "vector", "url": "https://example.com/t.json", "minzoom": "bad"}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Vector(v) = source else {
            panic!("expected Vector")
        };
        assert_eq!(v.minzoom, DEFAULT_MINZOOM);
    }

    #[test]
    fn invalid_maxzoom_falls_back_to_default() {
        let json = r#"{"type": "raster", "tiles": ["https://example.com/{z}/{x}/{y}.png"], "maxzoom": "bad"}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Raster(r) = source else {
            panic!("expected Raster")
        };
        assert_eq!(r.maxzoom, DEFAULT_MAXZOOM);
    }

    #[test]
    fn invalid_tile_size_falls_back_to_default() {
        let json = r#"{"type": "raster", "tiles": ["https://example.com/{z}/{x}/{y}.png"], "tileSize": [512]}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Raster(r) = source else {
            panic!("expected Raster")
        };
        assert_eq!(r.tile_size, DEFAULT_TILE_SIZE);
    }

    #[test]
    fn invalid_scheme_falls_back_to_default() {
        let json =
            r#"{"type": "vector", "url": "https://example.com/t.json", "scheme": "not-a-scheme"}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Vector(v) = source else {
            panic!("expected Vector")
        };
        assert_eq!(v.scheme, TileScheme::Xyz);
    }

    #[test]
    fn invalid_vector_encoding_falls_back_to_default() {
        let json = r#"{"type": "vector", "url": "https://example.com/t.json", "encoding": 42}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::Vector(v) = source else {
            panic!("expected Vector")
        };
        assert_eq!(v.encoding, VectorEncoding::Mvt);
    }

    #[test]
    fn invalid_dem_encoding_falls_back_to_default() {
        let json =
            r#"{"type": "raster-dem", "url": "https://example.com/dem.json", "encoding": "bogus"}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::RasterDem(d) = source else {
            panic!("expected RasterDem")
        };
        assert_eq!(d.encoding, DemEncoding::Mapbox);
    }

    #[test]
    fn invalid_red_factor_falls_back_to_default() {
        let json =
            r#"{"type": "raster-dem", "url": "https://example.com/dem.json", "redFactor": "x"}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::RasterDem(d) = source else {
            panic!("expected RasterDem")
        };
        assert_eq!(d.red_factor, DEFAULT_FACTOR);
    }

    #[test]
    fn invalid_buffer_falls_back_to_default() {
        let json = r#"{"type": "geojson", "data": "https://example.com/d.json", "buffer": "wide"}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::GeoJson(g) = source else {
            panic!("expected GeoJson")
        };
        assert_eq!(g.buffer, DEFAULT_BUFFER);
    }

    #[test]
    fn invalid_tolerance_falls_back_to_default() {
        let json =
            r#"{"type": "geojson", "data": "https://example.com/d.json", "tolerance": null}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::GeoJson(g) = source else {
            panic!("expected GeoJson")
        };
        assert_eq!(g.tolerance, DEFAULT_TOLERANCE);
    }

    #[test]
    fn invalid_cluster_radius_falls_back_to_default() {
        let json =
            r#"{"type": "geojson", "data": "https://example.com/d.json", "clusterRadius": "big"}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::GeoJson(g) = source else {
            panic!("expected GeoJson")
        };
        assert_eq!(g.cluster_radius, DEFAULT_CLUSTER_RADIUS);
    }

    #[test]
    fn invalid_geojson_maxzoom_falls_back_to_default() {
        let json =
            r#"{"type": "geojson", "data": "https://example.com/d.json", "maxzoom": "high"}"#;
        let source: Source = serde_json::from_str(json).unwrap();
        let Source::GeoJson(g) = source else {
            panic!("expected GeoJson")
        };
        assert_eq!(g.maxzoom, DEFAULT_GEOJSON_MAXZOOM);
    }
}
