//! Fill and fill-extrusion layer types.

use serde::Deserialize;
use serde_json::Value;

use super::common::{
    default_layer_maxzoom, default_layer_minzoom, deserialize_maxzoom, deserialize_minzoom,
    CommonLayout,
};

/// Paint properties for a `fill` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct FillPaint {
    /// Whether or not the fill should be antialiased. Supports expressions.
    #[serde(rename = "fill-antialias", skip_serializing_if = "Option::is_none")]
    pub fill_antialias: Option<Value>,

    /// Fill colour. Supports expressions.
    #[serde(rename = "fill-color", skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<Value>,

    /// Outline colour. Supports expressions.
    #[serde(rename = "fill-outline-color", skip_serializing_if = "Option::is_none")]
    pub fill_outline_color: Option<Value>,

    /// Opacity of the entire fill layer. Supports expressions.
    #[serde(rename = "fill-opacity", skip_serializing_if = "Option::is_none")]
    pub fill_opacity: Option<Value>,

    /// Name of image in sprite to use for drawing the fill pattern.
    #[serde(rename = "fill-pattern", skip_serializing_if = "Option::is_none")]
    pub fill_pattern: Option<Value>,

    /// Translation of the fill pixels. Supports expressions.
    #[serde(rename = "fill-translate", skip_serializing_if = "Option::is_none")]
    pub fill_translate: Option<Value>,

    /// Control whether the translation is relative to the map or viewport.
    #[serde(
        rename = "fill-translate-anchor",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_translate_anchor: Option<Value>,

    /// Emission strength of the fill colour.
    #[serde(
        rename = "fill-emissive-strength",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_emissive_strength: Option<Value>,
}

/// Layout properties for a `fill` layer (visibility only).
pub type FillLayout = CommonLayout;

/// A `fill` layer draws filled polygons.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FillLayer {
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
    pub layout: FillLayout,

    /// Paint properties.
    #[serde(default)]
    pub paint: FillPaint,
}

/// Paint properties for a `fill-extrusion` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct FillExtrusionPaint {
    /// Base height of the extrusion. Supports expressions.
    #[serde(
        rename = "fill-extrusion-base",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_base: Option<Value>,

    /// Fill colour of the extrusion. Supports expressions.
    #[serde(
        rename = "fill-extrusion-color",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_color: Option<Value>,

    /// Height of the extrusion. Supports expressions.
    #[serde(
        rename = "fill-extrusion-height",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_height: Option<Value>,

    /// Opacity of the extrusion. Supports expressions.
    #[serde(
        rename = "fill-extrusion-opacity",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_opacity: Option<Value>,

    /// Name of image in sprite for drawing the extrusion pattern.
    #[serde(
        rename = "fill-extrusion-pattern",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_pattern: Option<Value>,

    /// Translation offset of the extrusion. Supports expressions.
    #[serde(
        rename = "fill-extrusion-translate",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_translate: Option<Value>,

    /// Control whether the translation is relative to map or viewport.
    #[serde(
        rename = "fill-extrusion-translate-anchor",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_translate_anchor: Option<Value>,

    /// Whether to apply a floor-level shadow.
    #[serde(
        rename = "fill-extrusion-flood-light-color",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_flood_light_color: Option<Value>,

    /// Intensity of the flood-light effect.
    #[serde(
        rename = "fill-extrusion-flood-light-intensity",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_flood_light_intensity: Option<Value>,

    /// Emission strength of the fill-extrusion colour.
    #[serde(
        rename = "fill-extrusion-emissive-strength",
        skip_serializing_if = "Option::is_none"
    )]
    pub fill_extrusion_emissive_strength: Option<Value>,
}

/// Layout properties for a `fill-extrusion` layer (visibility only).
pub type FillExtrusionLayout = CommonLayout;

/// A `fill-extrusion` layer draws extruded (3D) polygons.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FillExtrusionLayer {
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
    pub layout: FillExtrusionLayout,

    /// Paint properties.
    #[serde(default)]
    pub paint: FillExtrusionPaint,
}
