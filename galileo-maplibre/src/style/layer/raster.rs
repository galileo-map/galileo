//! Raster and hillshade layer types.

use serde::Deserialize;
use serde_json::Value;

use super::common::{
    default_layer_maxzoom, default_layer_minzoom, deserialize_maxzoom, deserialize_minzoom,
    CommonLayout,
};

/// Paint properties for a `raster` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct RasterPaint {
    /// Opacity of the raster layer. Supports expressions.
    #[serde(rename = "raster-opacity", skip_serializing_if = "Option::is_none")]
    pub raster_opacity: Option<Value>,

    /// Rotates hues around the colour wheel. Supports expressions.
    #[serde(rename = "raster-hue-rotate", skip_serializing_if = "Option::is_none")]
    pub raster_hue_rotate: Option<Value>,

    /// Increase or reduce the brightness of the image (minimum). Supports expressions.
    #[serde(
        rename = "raster-brightness-min",
        skip_serializing_if = "Option::is_none"
    )]
    pub raster_brightness_min: Option<Value>,

    /// Increase or reduce the brightness of the image (maximum). Supports expressions.
    #[serde(
        rename = "raster-brightness-max",
        skip_serializing_if = "Option::is_none"
    )]
    pub raster_brightness_max: Option<Value>,

    /// Increase or reduce the saturation of the image. Supports expressions.
    #[serde(rename = "raster-saturation", skip_serializing_if = "Option::is_none")]
    pub raster_saturation: Option<Value>,

    /// Increase or reduce the contrast of the image. Supports expressions.
    #[serde(rename = "raster-contrast", skip_serializing_if = "Option::is_none")]
    pub raster_contrast: Option<Value>,

    /// The resampling/interpolation method to use for overscaling.
    #[serde(rename = "raster-resampling", skip_serializing_if = "Option::is_none")]
    pub raster_resampling: Option<Value>,

    /// Fade duration when a new tile is added. Supports expressions.
    #[serde(
        rename = "raster-fade-duration",
        skip_serializing_if = "Option::is_none"
    )]
    pub raster_fade_duration: Option<Value>,

    /// Emission strength of the raster colour.
    #[serde(
        rename = "raster-emissive-strength",
        skip_serializing_if = "Option::is_none"
    )]
    pub raster_emissive_strength: Option<Value>,
}

/// Layout properties for a `raster` layer (visibility only).
pub type RasterLayout = CommonLayout;

/// A `raster` layer renders raster tiles.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RasterLayer {
    /// Unique layer identifier.
    pub id: String,

    /// Source to use for this layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Layer within the source to use.
    #[serde(rename = "source-layer", skip_serializing_if = "Option::is_none")]
    pub source_layer: Option<String>,

    /// Arbitrary properties for tracking the layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// Minimum zoom level at which to show this layer.
    #[serde(
        default = "default_layer_minzoom",
        deserialize_with = "deserialize_minzoom"
    )]
    pub minzoom: f64,

    /// Maximum zoom level at which to show this layer.
    #[serde(
        default = "default_layer_maxzoom",
        deserialize_with = "deserialize_maxzoom"
    )]
    pub maxzoom: f64,

    /// Filter expression to select features from the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,

    /// Layout properties.
    #[serde(default)]
    pub layout: RasterLayout,

    /// Paint properties.
    #[serde(default)]
    pub paint: RasterPaint,
}

/// Paint properties for a `hillshade` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct HillshadePaint {
    /// The direction of the light source. Supports expressions.
    #[serde(
        rename = "hillshade-illumination-direction",
        skip_serializing_if = "Option::is_none"
    )]
    pub hillshade_illumination_direction: Option<Value>,

    /// Whether the illumination is relative to map north or viewport north.
    #[serde(
        rename = "hillshade-illumination-anchor",
        skip_serializing_if = "Option::is_none"
    )]
    pub hillshade_illumination_anchor: Option<Value>,

    /// Intensity of the hillshade. Supports expressions.
    #[serde(
        rename = "hillshade-exaggeration",
        skip_serializing_if = "Option::is_none"
    )]
    pub hillshade_exaggeration: Option<Value>,

    /// The shading colour of areas that face away from the light source.
    #[serde(
        rename = "hillshade-shadow-color",
        skip_serializing_if = "Option::is_none"
    )]
    pub hillshade_shadow_color: Option<Value>,

    /// The shading colour of areas that faces towards the light source.
    #[serde(
        rename = "hillshade-highlight-color",
        skip_serializing_if = "Option::is_none"
    )]
    pub hillshade_highlight_color: Option<Value>,

    /// The shading colour used to accentuate rugged terrain.
    #[serde(
        rename = "hillshade-accent-color",
        skip_serializing_if = "Option::is_none"
    )]
    pub hillshade_accent_color: Option<Value>,

    /// Emission strength of the hillshade colour.
    #[serde(
        rename = "hillshade-emissive-strength",
        skip_serializing_if = "Option::is_none"
    )]
    pub hillshade_emissive_strength: Option<Value>,
}

/// Layout properties for a `hillshade` layer (visibility only).
pub type HillshadeLayout = CommonLayout;

/// A `hillshade` layer renders a client-side hillshade from a raster DEM source.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HillshadeLayer {
    /// Unique layer identifier.
    pub id: String,

    /// Source to use for this layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Layer within the source to use.
    #[serde(rename = "source-layer", skip_serializing_if = "Option::is_none")]
    pub source_layer: Option<String>,

    /// Arbitrary properties for tracking the layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// Minimum zoom level at which to show this layer.
    #[serde(
        default = "default_layer_minzoom",
        deserialize_with = "deserialize_minzoom"
    )]
    pub minzoom: f64,

    /// Maximum zoom level at which to show this layer.
    #[serde(
        default = "default_layer_maxzoom",
        deserialize_with = "deserialize_maxzoom"
    )]
    pub maxzoom: f64,

    /// Layout properties.
    #[serde(default)]
    pub layout: HillshadeLayout,

    /// Paint properties.
    #[serde(default)]
    pub paint: HillshadePaint,
}
