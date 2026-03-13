//! Generic property value types for MapLibre style layers.
//!
//! Every paint and layout property in the MapLibre style specification has a
//! **known output type** — for example `line-color` always produces a color,
//! `line-width` always produces a number, and `text-optional` always produces
//! a boolean.  The JSON representation of that value can take three forms:
//!
//! 1. **Literal** — a plain JSON primitive already of the correct type
//!    (e.g. `"#ff0000"` for color, `2.0` for number, `true` for bool).
//! 2. **Expression** — a JSON array whose first element is an operator string
//!    (e.g. `["interpolate", ["linear"], ["zoom"], 5, 1, 10, 4]`).
//!    Expressions are the modern (post-v0.41.0) way to compute a value from
//!    map state (zoom level) or feature properties.
//! 3. **Function** — a JSON object with a `"stops"` key (and optionally
//!    `"base"`, `"property"`, `"type"`, `"default"`, `"colorSpace"`).
//!    Functions are the legacy (pre-v0.41.0) equivalent of expressions and
//!    are deprecated but still widely used in real-world styles.
//!
//! # Type parameters
//!
//! [`StyleValue<T>`] is generic over the output type `T`.  The concrete
//! instantiations used across layer properties are:
//!
//! | Rust type | MapLibre spec type | JSON representation |
//! |-----------|-------------------|---------------------|
//! | [`f64`]   | `number`          | JSON number         |
//! | [`bool`]  | `boolean`         | JSON boolean        |
//! | [`String`]| `string`          | JSON string         |
//!
//! # Serde strategy
//!
//! `StyleValue<T>` is deserialized with `#[serde(untagged)]`.  Serde tries
//! each variant in order:
//! - `Literal(T)` — succeeds when the JSON is a primitive that directly
//!   deserializes as `T` (number → `f64`, string → `String`, bool → `bool`).
//! - `Expression(Expr)` — succeeds when the JSON is an array.
//! - `Function(Function<T>)` — succeeds when the JSON is an object (it
//!   must have a `"stops"` key; other objects are an error).

use galileo::Color;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::expression::Expr;
use crate::style::color::parse_css_color;

/// The interpolation type for a legacy [`Function`].
///
/// Governs how the function interpolates between stops.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionType {
    /// Passes the input through unchanged (no stops required).
    Identity,
    /// Interpolates exponentially between stops (default for numeric output).
    Exponential,
    /// Returns the output of the nearest stop below the input.
    Interval,
    /// Returns the output for the stop whose input equals the feature value.
    Categorical,
}

/// The color space used for interpolating color outputs in a legacy [`Function`].
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorSpace {
    /// Interpolate in RGB color space.
    #[default]
    Rgb,
    /// Interpolate in CIELAB color space.
    Lab,
    /// Interpolate in Hue-Chroma-Luminance color space.
    Hcl,
}

/// A single stop in a legacy [`Function`].
///
/// A stop maps an input value (zoom level or feature property value) to an
/// output value of type `T`.  Both are stored as raw [`Value`] because:
/// - The input can be a plain number *or* a `{"zoom": N, "value": V}` object
///   (for zoom-and-property functions).
/// - The output must be appropriate for `T` but is easier to keep as raw JSON
///   here and let the renderer coerce it.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionStop<T> {
    /// The input threshold — a zoom level (`number`) or a zoom-and-property
    /// object (`{"zoom": number, "value": any}`).
    pub input: f64,
    /// The output value at this stop.
    pub output: T,
}

impl<T: Serialize> Serialize for FunctionStop<T> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(2))?;
        seq.serialize_element(&self.input)?;
        seq.serialize_element(&self.output)?;
        seq.end()
    }
}

macro_rules! impl_function_stop_deserialize {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl<'de> Deserialize<'de> for FunctionStop<$ty> {
                fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                    struct StopVisitor;

                    impl<'de> Visitor<'de> for StopVisitor {
                        type Value = FunctionStop<$ty>;

                        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                            f.write_str("a [input, output] stop pair")
                        }

                        fn visit_seq<A: SeqAccess<'de>>(
                            self,
                            mut seq: A,
                        ) -> Result<Self::Value, A::Error> {
                            let input = seq
                                .next_element::<f64>()?
                                .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                            let output = seq
                                .next_element::<$ty>()?
                                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                            Ok(FunctionStop { input, output })
                        }
                    }

                    d.deserialize_seq(StopVisitor)
                }
            }
        )+
    };
}

impl_function_stop_deserialize!(f64, bool, String, Value);

impl<'de> Deserialize<'de> for FunctionStop<Color> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct StopVisitor;

        impl<'de> Visitor<'de> for StopVisitor {
            type Value = FunctionStop<Color>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a [input, output] stop pair")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let input = seq
                    .next_element::<f64>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let color_str = seq
                    .next_element::<&str>()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let color = parse_css_color(color_str).unwrap_or(Color::TRANSPARENT);
                Ok(FunctionStop {
                    input,
                    output: color,
                })
            }
        }

        d.deserialize_seq(StopVisitor)
    }
}

const DEFAULT_BASE: f64 = 1.0;
fn default_base() -> f64 {
    DEFAULT_BASE
}

fn de_base<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    use crate::style::helpers::deserialize_f64_with_fallback;
    deserialize_f64_with_fallback(deserializer, DEFAULT_BASE, "base")
}

/// A legacy MapLibre zoom/property function (deprecated since v0.41.0).
///
/// Still widely used in real-world styles.  The renderer evaluates this at
/// runtime by interpolating between stops to produce a value of type `T`.
///
/// # JSON shape
/// ```json
/// {"stops": [[5, 1], [10, 4]]}
/// {"base": 1.4, "stops": [[5, 1], [10, 4]]}
/// {"property": "temperature", "stops": [[0, "blue"], [100, "red"]]}
/// {"base": 1, "stops": [[{"zoom": 0, "value": 0}, 0], [{"zoom": 20, "value": 5}, 20]]}
/// ```
///
/// The `default` and `stops` outputs carry raw [`Value`] rather than `T`
/// because the stop output type can only be verified by the renderer at
/// runtime (e.g. for `identity` functions, the type is not statically known).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function<T>
where
    for<'a> FunctionStop<T>: Deserialize<'a>,
{
    /// The stop pairs `[input, output]`.  Required for all types except
    /// `identity`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stops: Vec<FunctionStop<T>>,

    /// The exponential base for `exponential` interpolation.  Default is `1`
    /// (linear).
    #[serde(default = "default_base", deserialize_with = "de_base")]
    pub base: f64,

    /// The feature property to use as the function input.  If absent, the
    /// function input is the current zoom level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,

    /// The interpolation type.  Defaults to `exponential` for numeric
    /// properties and `interval` for others.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub function_type: Option<FunctionType>,

    /// Fallback output value used when the input does not match any stop or
    /// when the feature property is missing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<T>,

    /// Color space for interpolating color outputs.
    #[serde(rename = "colorSpace", skip_serializing_if = "Option::is_none")]
    pub color_space: Option<ColorSpace>,
}

/// A style property value that ultimately produces a `T`.
///
/// See the [module documentation](self) for the full design rationale.
///
/// # Deserialization
///
/// ```json
/// // Literal f64
/// 2.0
///
/// // Literal Color
/// "#ff0000"
///
/// // Expression (modern) — JSON array starting with an operator string
/// ["interpolate", ["linear"], ["zoom"], 5, 1, 10, 4]
///
/// // Function (legacy) — JSON object with a "stops" key
/// {"base": 1.4, "stops": [[5, 1], [10, 4]]}
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MlStyleValue<T: Default>
where
    for<'a> FunctionStop<T>: Deserialize<'a>,
{
    /// A literal value of the expected type, present directly in the JSON.
    Literal(T),
    /// A modern expression: a JSON array whose first element is an operator
    /// string.  The renderer evaluates this at runtime to produce a `T`.
    Expression(Expr),
    /// A legacy zoom/property function: a JSON object with a `"stops"` key.
    /// The renderer evaluates this at runtime to produce a `T`.
    Function(Function<T>),
}

impl<T: Default> MlStyleValue<T>
where
    for<'a> FunctionStop<T>: Deserialize<'a>,
{
    /// Returns `true` if this is a [`StyleValue::Literal`].
    pub fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    /// Returns the literal value, or `None` if this is not a literal.
    pub fn as_literal(&self) -> Option<&T> {
        match self {
            Self::Literal(v) => Some(v),
            _ => None,
        }
    }

    /// Returns `true` if this is a [`StyleValue::Expression`].
    pub fn is_expression(&self) -> bool {
        matches!(self, Self::Expression(_))
    }

    /// Returns the expression, or `None` if this is not an expression.
    pub fn as_expression(&self) -> Option<&Expr> {
        match self {
            Self::Expression(e) => Some(e),
            _ => None,
        }
    }

    /// Returns `true` if this is a [`StyleValue::Function`].
    pub fn is_function(&self) -> bool {
        matches!(self, Self::Function(_))
    }

    /// Returns the function, or `None` if this is not a function.
    pub fn as_function(&self) -> Option<&Function<T>> {
        match self {
            Self::Function(f) => Some(f),
            _ => None,
        }
    }
}

impl From<f64> for MlStyleValue<f64> {
    fn from(v: f64) -> Self {
        Self::Literal(v)
    }
}

impl From<bool> for MlStyleValue<bool> {
    fn from(v: bool) -> Self {
        Self::Literal(v)
    }
}

impl From<String> for MlStyleValue<String> {
    fn from(v: String) -> Self {
        Self::Literal(v)
    }
}

impl From<&str> for MlStyleValue<String> {
    fn from(v: &str) -> Self {
        Self::Literal(v.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    #[test]
    fn expression_operator_and_args() {
        let e: Expr =
            serde_json::from_value(json!(["interpolate", ["linear"], ["zoom"], 5, 1, 10, 4]))
                .unwrap();
        assert_eq!(e.operator(), Some("interpolate"));
        // Interpolate with 2 stops
        assert!(matches!(e, Expr::Interpolate { ref stops, .. } if stops.len() == 2));
    }

    #[test]
    fn expression_step_operator() {
        let e: Expr =
            serde_json::from_value(json!(["step", ["zoom"], 0.5, 12, 1.0, 15, 2.0])).unwrap();
        assert_eq!(e.operator(), Some("step"));
    }

    #[test]
    fn expression_roundtrip() {
        // Round-trip via StyleValue so we verify end-to-end serde
        let original = json!(["interpolate", ["linear"], ["zoom"], 5, 1.0, 10, 4.0]);
        let v: MlStyleValue<f64> = serde_json::from_value(original.clone()).unwrap();
        assert!(v.is_expression());
        assert_eq!(
            v.as_expression().and_then(|e| e.operator()),
            Some("interpolate")
        );
    }

    #[test]
    fn function_stops_only() {
        let f: Function<Value> =
            serde_json::from_value(json!({"stops": [[5, 1.0], [10, 4.0]]})).unwrap();
        assert_eq!(f.stops.len(), 2);
        assert_eq!(f.stops[0].input, json!(5));
        assert_eq!(f.stops[0].output, json!(1.0));
        assert_eq!(f.base, 1.0);
        assert!(f.property.is_none());
        assert!(f.function_type.is_none());
        assert!(f.default.is_none());
        assert!(f.color_space.is_none());
    }

    #[test]
    fn function_with_base() {
        let f: Function<f64> =
            serde_json::from_value(json!({"base": 1.4, "stops": [[5, 1.0], [10, 4.0]]})).unwrap();
        assert_eq!(f.base, 1.4);
        assert_eq!(f.stops.len(), 2);
    }

    #[test]
    fn function_with_property() {
        let f: Function<Value> = serde_json::from_value(json!({
            "property": "temperature",
            "stops": [[0, "blue"], [100, "red"]]
        }))
        .unwrap();
        assert_eq!(f.property.as_deref(), Some("temperature"));
        assert_eq!(f.stops[0].output, json!("blue"));
    }

    #[test]
    fn function_with_default() {
        let f: Function<Value> = serde_json::from_value(json!({
            "stops": [[0, 1.0]],
            "default": 0.5
        }))
        .unwrap();
        assert_eq!(f.default, Some(json!(0.5)));
    }

    #[test]
    fn function_color_stops() {
        let f: Function<Value> = serde_json::from_value(
            json!({"stops": [[6, "hsl(47,79%,94%)"], [14, "hsl(42,49%,93%)"]]}),
        )
        .unwrap();
        assert_eq!(f.stops.len(), 2);
        assert_eq!(f.stops[0].input, 6.0);
        assert_eq!(f.stops[0].output, json!("hsl(47,79%,94%)"));
    }

    #[test]
    fn function_roundtrip() {
        let original = json!({"base": 1.5, "stops": [[0, "#fff"], [10, "#000"]]});
        let f: Function<Value> = serde_json::from_value(original.clone()).unwrap();
        let back = serde_json::to_value(&f).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn function_type_variants() {
        for (s, expected) in [
            ("identity", FunctionType::Identity),
            ("exponential", FunctionType::Exponential),
            ("interval", FunctionType::Interval),
            ("categorical", FunctionType::Categorical),
        ] {
            let f: Function<Value> =
                serde_json::from_value(json!({"type": s, "stops": [[0, 1]]})).unwrap();
            assert_eq!(f.function_type, Some(expected));
        }
    }

    #[test]
    fn f64_literal() {
        let v: MlStyleValue<f64> = serde_json::from_value(json!(2.0)).unwrap();
        assert!(v.is_literal());
        assert_eq!(v.as_literal(), Some(&2.0));
        assert!(!v.is_expression());
        assert!(!v.is_function());
    }

    #[test]
    fn f64_expression() {
        let raw = json!(["interpolate", ["linear"], ["zoom"], 5, 1.0, 10, 4.0]);
        let v: MlStyleValue<f64> = serde_json::from_value(raw).unwrap();
        assert!(v.is_expression());
        assert_eq!(
            v.as_expression().and_then(|e| e.operator()),
            Some("interpolate")
        );
        assert!(!v.is_literal());
        assert!(!v.is_function());
    }

    #[test]
    fn f64_function() {
        let raw = json!({"base": 1.4, "stops": [[5, 1.0], [10, 4.0]]});
        let v: MlStyleValue<f64> = serde_json::from_value(raw).unwrap();
        assert!(v.is_function());
        assert_eq!(v.as_function().map(|f| f.base), Some(1.4));
        assert!(!v.is_literal());
        assert!(!v.is_expression());
    }

    #[test]
    fn f64_step_expression() {
        let v: MlStyleValue<f64> =
            serde_json::from_value(json!(["step", ["zoom"], 0.5, 12, 1.0, 15, 2.0])).unwrap();
        assert_eq!(v.as_expression().and_then(|e| e.operator()), Some("step"));
    }

    #[test]
    fn bool_literal_true() {
        let v: MlStyleValue<bool> = serde_json::from_value(json!(true)).unwrap();
        assert_eq!(v.as_literal(), Some(&true));
    }

    #[test]
    fn bool_literal_false() {
        let v: MlStyleValue<bool> = serde_json::from_value(json!(false)).unwrap();
        assert_eq!(v.as_literal(), Some(&false));
    }

    #[test]
    fn bool_expression() {
        let v: MlStyleValue<bool> = serde_json::from_value(json!(["has", "name"])).unwrap();
        assert!(v.is_expression());
        assert_eq!(v.as_expression().and_then(|e| e.operator()), Some("has"));
    }

    #[test]
    fn string_literal() {
        let v: MlStyleValue<String> = serde_json::from_value(json!("butt")).unwrap();
        assert_eq!(v.as_literal(), Some(&"butt".to_owned()));
    }

    #[test]
    fn string_expression() {
        let v: MlStyleValue<String> = serde_json::from_value(json!(["get", "name"])).unwrap();
        assert!(v.is_expression());
        assert_eq!(v.as_expression().and_then(|e| e.operator()), Some("get"));
    }

    #[test]
    fn maptiler_fill_opacity_function() {
        // From maptiler_fmt.json — fill-opacity with zoom stops
        let raw = json!({"stops": [[0, 1], [8, 0.1]]});
        let v: MlStyleValue<f64> = serde_json::from_value(raw).unwrap();
        let f = v.as_function().unwrap();
        assert_eq!(f.stops[0].output, json!(1));
        assert_eq!(f.stops[1].output, json!(0.1));
    }

    #[test]
    fn roundtrip_f64_literal() {
        let original = json!(1.5);
        let v: MlStyleValue<f64> = serde_json::from_value(original.clone()).unwrap();
        let back = serde_json::to_value(&v).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn roundtrip_expression() {
        let original = json!(["interpolate", ["linear"], ["zoom"], 5, 1.0, 10, 4.0]);
        let v: MlStyleValue<f64> = serde_json::from_value(original.clone()).unwrap();
        let back = serde_json::to_value(&v).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn roundtrip_function() {
        let original = json!({"base": 1.5, "stops": [[0, 0.0], [10, 1.0]]});
        let v: MlStyleValue<f64> = serde_json::from_value(original.clone()).unwrap();
        let back = serde_json::to_value(&v).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn from_impls() {
        assert_eq!(MlStyleValue::from(2.5_f64).as_literal(), Some(&2.5));
        assert_eq!(MlStyleValue::from(true).as_literal(), Some(&true));
        assert_eq!(
            MlStyleValue::from("butt").as_literal(),
            Some(&"butt".to_owned())
        );
    }
}
