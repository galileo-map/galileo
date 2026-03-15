//! Background layer types.

use serde::Deserialize;
use serde_json::Value;

use super::common::CommonLayout;
use crate::style::color::MlColor;
use crate::style::value::MlStyleValue;
use crate::style::{
    default_one, default_transparent, deser_default_one, deser_default_transparent,
    deserialize_opt_f64,
};

/// Paint properties for a background layer.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BackgroundPaint {
    /// The colour with which the background will be drawn.
    #[serde(
        rename = "background-color",
        default = "default_transparent",
        deserialize_with = "deser_default_transparent"
    )]
    pub background_color: MlStyleValue<MlColor>,

    /// The opacity at which the background will be drawn.
    #[serde(
        rename = "background-opacity",
        default = "default_one",
        deserialize_with = "deser_default_one"
    )]
    pub background_opacity: MlStyleValue<f64>,

    /// Name of image in sprite to use for drawing an image background.
    #[serde(rename = "background-pattern")]
    pub background_pattern: Option<Value>,
    /// Controls the intensity of light emitted on the source features.
    #[serde(rename = "background-emissive-strength")]
    pub background_emissive_strength: Option<Value>,
}

impl Default for BackgroundPaint {
    fn default() -> Self {
        Self {
            background_color: default_transparent(),
            background_opacity: default_one(),
            background_pattern: Default::default(),
            background_emissive_strength: Default::default(),
        }
    }
}

/// Layout properties for a background layer (visibility only).
pub type BackgroundLayout = CommonLayout;

/// A background layer fills the map with a single colour or pattern.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BackgroundLayer {
    /// Unique layer identifier.
    pub id: String,
    /// Arbitrary properties useful to track with the layer, but do not influence rendering.
    pub metadata: Option<Value>,
    /// The minimum zoom level for the layer (inclusive).
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_opt_f64"
    )]
    pub minzoom: Option<f64>,
    /// The maximum zoom level for the layer (exclusive).
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_opt_f64"
    )]
    pub maxzoom: Option<f64>,
    /// Layout properties for this layer.
    #[serde(default)]
    pub layout: BackgroundLayout,
    /// Paint properties for this layer.
    #[serde(default)]
    pub paint: BackgroundPaint,
}
