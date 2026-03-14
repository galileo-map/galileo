//! Circle and heatmap layer types.

use serde::Deserialize;
use serde_json::Value;

use super::common::{CommonLayout, Visibility, deserialize_visibility_or_default};
use crate::style::deserialize_opt_f64;

/// Paint properties for a `circle` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct CirclePaint {
    /// Circle fill colour. Supports expressions.
    #[serde(rename = "circle-color", skip_serializing_if = "Option::is_none")]
    pub circle_color: Option<Value>,

    /// Circle radius in pixels. Supports expressions.
    #[serde(rename = "circle-radius", skip_serializing_if = "Option::is_none")]
    pub circle_radius: Option<Value>,

    /// Circle opacity. Supports expressions.
    #[serde(rename = "circle-opacity", skip_serializing_if = "Option::is_none")]
    pub circle_opacity: Option<Value>,

    /// Width of the circle's stroke. Supports expressions.
    #[serde(
        rename = "circle-stroke-width",
        skip_serializing_if = "Option::is_none"
    )]
    pub circle_stroke_width: Option<Value>,

    /// Stroke colour of the circle. Supports expressions.
    #[serde(
        rename = "circle-stroke-color",
        skip_serializing_if = "Option::is_none"
    )]
    pub circle_stroke_color: Option<Value>,

    /// Stroke opacity. Supports expressions.
    #[serde(
        rename = "circle-stroke-opacity",
        skip_serializing_if = "Option::is_none"
    )]
    pub circle_stroke_opacity: Option<Value>,

    /// Circle blur. Supports expressions.
    #[serde(rename = "circle-blur", skip_serializing_if = "Option::is_none")]
    pub circle_blur: Option<Value>,

    /// Translation offset in pixels. Supports expressions.
    #[serde(rename = "circle-translate", skip_serializing_if = "Option::is_none")]
    pub circle_translate: Option<Value>,

    /// Control whether the translation is relative to map or viewport.
    #[serde(
        rename = "circle-translate-anchor",
        skip_serializing_if = "Option::is_none"
    )]
    pub circle_translate_anchor: Option<Value>,

    /// Controls the scaling of the circle (map vs. viewport).
    #[serde(rename = "circle-pitch-scale", skip_serializing_if = "Option::is_none")]
    pub circle_pitch_scale: Option<Value>,

    /// Orientation of circle when map is pitched.
    #[serde(
        rename = "circle-pitch-alignment",
        skip_serializing_if = "Option::is_none"
    )]
    pub circle_pitch_alignment: Option<Value>,

    /// Emission strength of the circle colour.
    #[serde(
        rename = "circle-emissive-strength",
        skip_serializing_if = "Option::is_none"
    )]
    pub circle_emissive_strength: Option<Value>,
}

/// Layout properties for a `circle` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct CircleLayout {
    /// Whether this layer is displayed.
    #[serde(default, deserialize_with = "deserialize_visibility_or_default")]
    pub visibility: Visibility,

    /// Sorts features in ascending order by this value.
    #[serde(rename = "circle-sort-key", skip_serializing_if = "Option::is_none")]
    pub circle_sort_key: Option<Value>,
}

/// A `circle` layer draws circle symbols at points.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CircleLayer {
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
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_opt_f64"
    )]
    pub minzoom: Option<f64>,

    /// Maximum zoom level at which to show this layer.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_opt_f64"
    )]
    pub maxzoom: Option<f64>,

    /// Filter expression to select features from the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,

    /// Layout properties.
    #[serde(default)]
    pub layout: CircleLayout,

    /// Paint properties.
    #[serde(default)]
    pub paint: CirclePaint,
}

/// Paint properties for a `heatmap` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct HeatmapPaint {
    /// Radius of influence of one heatmap point. Supports expressions.
    #[serde(rename = "heatmap-radius", skip_serializing_if = "Option::is_none")]
    pub heatmap_radius: Option<Value>,

    /// Weight of each individual data point. Supports expressions.
    #[serde(rename = "heatmap-weight", skip_serializing_if = "Option::is_none")]
    pub heatmap_weight: Option<Value>,

    /// Controls the intensity of the heatmap. Supports expressions.
    #[serde(rename = "heatmap-intensity", skip_serializing_if = "Option::is_none")]
    pub heatmap_intensity: Option<Value>,

    /// Defines the colour of each pixel based on its density value.
    #[serde(rename = "heatmap-color", skip_serializing_if = "Option::is_none")]
    pub heatmap_color: Option<Value>,

    /// Opacity of the entire heatmap layer. Supports expressions.
    #[serde(rename = "heatmap-opacity", skip_serializing_if = "Option::is_none")]
    pub heatmap_opacity: Option<Value>,

    /// Emission strength of the heatmap colour.
    #[serde(
        rename = "heatmap-emissive-strength",
        skip_serializing_if = "Option::is_none"
    )]
    pub heatmap_emissive_strength: Option<Value>,
}

/// Layout properties for a `heatmap` layer (visibility only).
pub type HeatmapLayout = CommonLayout;

/// A `heatmap` layer renders a heatmap.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HeatmapLayer {
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
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_opt_f64"
    )]
    pub minzoom: Option<f64>,

    /// Maximum zoom level at which to show this layer.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_opt_f64"
    )]
    pub maxzoom: Option<f64>,

    /// Filter expression to select features from the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,

    /// Layout properties.
    #[serde(default)]
    pub layout: HeatmapLayout,

    /// Paint properties.
    #[serde(default)]
    pub paint: HeatmapPaint,
}
