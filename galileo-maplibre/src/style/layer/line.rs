//! Line layer types.

use serde::Deserialize;
use serde_json::Value;

use super::common::{Visibility, deserialize_visibility_or_default};
use crate::style::deserialize_opt_f64;

/// Paint properties for a `line` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct LinePaint {
    /// Line stroke colour. Supports expressions.
    #[serde(rename = "line-color", skip_serializing_if = "Option::is_none")]
    pub line_color: Option<Value>,

    /// Line stroke opacity. Supports expressions.
    #[serde(rename = "line-opacity", skip_serializing_if = "Option::is_none")]
    pub line_opacity: Option<Value>,

    /// Line stroke width in pixels. Supports expressions.
    #[serde(rename = "line-width", skip_serializing_if = "Option::is_none")]
    pub line_width: Option<Value>,

    /// Line blur. Supports expressions.
    #[serde(rename = "line-blur", skip_serializing_if = "Option::is_none")]
    pub line_blur: Option<Value>,

    /// Dash pattern for the line. Supports expressions.
    #[serde(rename = "line-dasharray", skip_serializing_if = "Option::is_none")]
    pub line_dasharray: Option<Value>,

    /// Gap width for a casing effect. Supports expressions.
    #[serde(rename = "line-gap-width", skip_serializing_if = "Option::is_none")]
    pub line_gap_width: Option<Value>,

    /// A gradient used to color a line feature.
    #[serde(rename = "line-gradient", skip_serializing_if = "Option::is_none")]
    pub line_gradient: Option<Value>,

    /// Name of image in sprite to use for drawing line pattern.
    #[serde(rename = "line-pattern", skip_serializing_if = "Option::is_none")]
    pub line_pattern: Option<Value>,

    /// Translation of the line pixels. Supports expressions.
    #[serde(rename = "line-translate", skip_serializing_if = "Option::is_none")]
    pub line_translate: Option<Value>,

    /// Control whether the translation is relative to the map or viewport.
    #[serde(
        rename = "line-translate-anchor",
        skip_serializing_if = "Option::is_none"
    )]
    pub line_translate_anchor: Option<Value>,

    /// Emission strength of the line colour.
    #[serde(
        rename = "line-emissive-strength",
        skip_serializing_if = "Option::is_none"
    )]
    pub line_emissive_strength: Option<Value>,

    /// Stroke offset relative to the line's direction. Supports expressions.
    #[serde(rename = "line-offset", skip_serializing_if = "Option::is_none")]
    pub line_offset: Option<Value>,
}

/// Layout properties for a `line` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct LineLayout {
    /// Whether this layer is displayed.
    #[serde(default, deserialize_with = "deserialize_visibility_or_default")]
    pub visibility: Visibility,

    /// The display of line endings. Supports expressions.
    #[serde(rename = "line-cap", skip_serializing_if = "Option::is_none")]
    pub line_cap: Option<Value>,

    /// The display of lines when joining. Supports expressions.
    #[serde(rename = "line-join", skip_serializing_if = "Option::is_none")]
    pub line_join: Option<Value>,

    /// Used to automatically convert Miter joins to Bevel joins for sharp angles.
    #[serde(rename = "line-miter-limit", skip_serializing_if = "Option::is_none")]
    pub line_miter_limit: Option<Value>,

    /// Used to automatically convert Round joins to Miter joins for slight angles.
    #[serde(rename = "line-round-limit", skip_serializing_if = "Option::is_none")]
    pub line_round_limit: Option<Value>,

    /// Sorts features in ascending order by value. Features with a higher sort
    /// key will appear above features with a lower sort key.
    #[serde(rename = "line-sort-key", skip_serializing_if = "Option::is_none")]
    pub line_sort_key: Option<Value>,
}

/// A `line` layer draws stroked lines.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LineLayer {
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
    pub layout: LineLayout,

    /// Paint properties.
    #[serde(default)]
    pub paint: LinePaint,
}
