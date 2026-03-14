//! Sky, slot, and clip layer types.

use serde::Deserialize;
use serde_json::Value;

use super::common::CommonLayout;
use crate::style::deserialize_opt_f64;

/// Paint properties for a `sky` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct SkyPaint {
    /// Type of the sky rendering method.
    #[serde(rename = "sky-type", skip_serializing_if = "Option::is_none")]
    pub sky_type: Option<Value>,

    /// A colour used to seed the atmosphere simulation.
    #[serde(
        rename = "sky-atmosphere-color",
        skip_serializing_if = "Option::is_none"
    )]
    pub sky_atmosphere_color: Option<Value>,

    /// A colour applied to the atmosphere at halo radius from the sun.
    #[serde(
        rename = "sky-atmosphere-halo-color",
        skip_serializing_if = "Option::is_none"
    )]
    pub sky_atmosphere_halo_color: Option<Value>,

    /// Position of the sun relative to the map.
    #[serde(rename = "sky-atmosphere-sun", skip_serializing_if = "Option::is_none")]
    pub sky_atmosphere_sun: Option<Value>,

    /// Intensity of the sun as a light source in the atmosphere.
    #[serde(
        rename = "sky-atmosphere-sun-intensity",
        skip_serializing_if = "Option::is_none"
    )]
    pub sky_atmosphere_sun_intensity: Option<Value>,

    /// Defines a radial colour gradient.
    #[serde(rename = "sky-gradient", skip_serializing_if = "Option::is_none")]
    pub sky_gradient: Option<Value>,

    /// Centre of the sky gradient.
    #[serde(
        rename = "sky-gradient-center",
        skip_serializing_if = "Option::is_none"
    )]
    pub sky_gradient_center: Option<Value>,

    /// The angular distance from the gradient centre to its outer limits.
    #[serde(
        rename = "sky-gradient-radius",
        skip_serializing_if = "Option::is_none"
    )]
    pub sky_gradient_radius: Option<Value>,

    /// The opacity of the entire sky layer.
    #[serde(rename = "sky-opacity", skip_serializing_if = "Option::is_none")]
    pub sky_opacity: Option<Value>,
}

/// Layout properties for a `sky` layer (visibility only).
pub type SkyLayout = CommonLayout;

/// A `sky` layer renders a stylized spherical dome.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SkyLayer {
    /// Unique layer identifier.
    pub id: String,

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

    /// Layout properties.
    #[serde(default)]
    pub layout: SkyLayout,

    /// Paint properties.
    #[serde(default)]
    pub paint: SkyPaint,
}

/// A `slot` layer acts as an insertion point for layers coming from imported styles.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SlotLayer {
    /// Unique layer identifier.
    pub id: String,

    /// Arbitrary properties for tracking the layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// A `clip` layer removes 3D content (models, extrusions) and/or symbols
/// within the layer's geometry so that custom 3D content can be composited.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ClipLayer {
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
    #[serde(deserialize_with = "deserialize_opt_f64")]
    pub minzoom: Option<f64>,

    /// Maximum zoom level at which to show this layer.
    #[serde(deserialize_with = "deserialize_opt_f64")]
    pub maxzoom: Option<f64>,

    /// Filter expression to select features from the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,

    /// Types of content to clip: `"3d"` and/or `"symbols"`.
    #[serde(rename = "clip-layer-types", skip_serializing_if = "Option::is_none")]
    pub clip_layer_types: Option<Vec<String>>,
}
