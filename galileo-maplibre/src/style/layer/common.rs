//! Shared types and deserialisation helpers used by all layer submodules.

use serde::Deserialize;
use serde_json::Value;

use super::super::helpers::deserialize_f64_with_fallback;

pub(super) const DEFAULT_MINZOOM: f64 = 0.0;
pub(super) const DEFAULT_MAXZOOM: f64 = 24.0;

pub(super) fn default_layer_minzoom() -> f64 {
    DEFAULT_MINZOOM
}

pub(super) fn default_layer_maxzoom() -> f64 {
    DEFAULT_MAXZOOM
}

pub(super) fn deserialize_minzoom<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_MINZOOM, "minzoom")
}

pub(super) fn deserialize_maxzoom<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_f64_with_fallback(deserializer, DEFAULT_MAXZOOM, "maxzoom")
}

pub(super) fn deserialize_visibility_or_default<'de, D>(
    deserializer: D,
) -> Result<Visibility, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(Visibility::default()),
        v => serde_json::from_value::<Visibility>(v.clone()).or_else(|err| {
            log::warn!("Invalid visibility {v}: {err}; using default");
            Ok(Visibility::default())
        }),
    }
}

/// Whether a layer is shown or hidden.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// The layer is shown (default).
    #[default]
    Visible,
    /// The layer is hidden.
    None,
}

/// Layout properties shared by all layer types that have only a `visibility` field.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct CommonLayout {
    /// Whether this layer is displayed.
    #[serde(default, deserialize_with = "deserialize_visibility_or_default")]
    pub visibility: Visibility,
}
