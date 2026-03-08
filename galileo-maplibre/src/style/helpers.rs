//! Shared deserialisation helpers used across style submodules.

use serde::Deserialize;
use serde_json::Value;

/// Deserialise an `f64` from a JSON [`Value`], falling back to `fallback`
/// (and logging a warning) when the value is invalid or null.
pub(super) fn deserialize_f64_with_fallback<'de, D>(
    deserializer: D,
    fallback: f64,
    field_name: &str,
) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Null => Ok(fallback),
        v => serde_json::from_value::<f64>(v.clone()).or_else(|err| {
            log::warn!("Invalid {field_name} value {v}: {err}; using default ({fallback})");
            Ok(fallback)
        }),
    }
}
