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
//! | [`Color`] | `color`           | JSON string         |
//!
//! # Serde strategy
//!
//! `StyleValue<T>` is deserialized with `#[serde(untagged)]`.  Serde tries
//! each variant in order:
//! - `Literal(T)` — succeeds when the JSON is a primitive that directly
//!   deserializes as `T` (number → `f64`, string → `Color`/`String`, bool →
//!   `bool`).
//! - `Expression(Expr)` — succeeds when the JSON is an array.
//! - `Function(Function<T>)` — succeeds when the JSON is an object (it
//!   must have a `"stops"` key; other objects are an error).

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::expression::Expr;

/// A MapLibre color value.
///
/// Colors appear in the JSON as strings in one of several CSS-compatible
/// formats: hex (`#rrggbb`, `#rgb`, `#rrggbbaa`), `rgb(r,g,b)`,
/// `rgba(r,g,b,a)`, `hsl(h,s%,l%)`, `hsla(h,s%,l%,a)`, or a named CSS
/// color keyword such as `"red"`.
///
/// This type stores the original string verbatim.  Parsing to RGBA components
/// is the responsibility of the renderer and is not performed here.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Color(pub String);

impl Color {
    /// Create a `Color` from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the underlying color string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for Color {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for Color {
    fn from(s: String) -> Self {
        Self(s)
    }
}

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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorSpace {
    /// Interpolate in RGB color space.
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionStop {
    /// The input threshold — a zoom level (`number`) or a zoom-and-property
    /// object (`{"zoom": number, "value": any}`).
    pub input: Value,
    /// The output value at this stop.
    pub output: Value,
}

// Custom deserialization: stops are `[input, output]` arrays in JSON.
mod function_stops_serde {
    use std::fmt;

    use serde::de::{SeqAccess, Visitor};
    use serde::{Deserializer, Serializer};
    use serde_json::Value;

    use super::FunctionStop;

    pub fn serialize<S>(stops: &Vec<FunctionStop>, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = s.serialize_seq(Some(stops.len()))?;
        for stop in stops {
            seq.serialize_element(&[&stop.input, &stop.output])?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Vec<FunctionStop>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StopsVisitor;

        impl<'de> Visitor<'de> for StopsVisitor {
            type Value = Vec<FunctionStop>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an array of [input, output] stop pairs")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut stops = Vec::new();
                while let Some(pair) = seq.next_element::<[Value; 2]>()? {
                    let [input, output] = pair;
                    stops.push(FunctionStop { input, output });
                }
                Ok(stops)
            }
        }

        d.deserialize_seq(StopsVisitor)
    }
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
pub struct Function {
    /// The stop pairs `[input, output]`.  Required for all types except
    /// `identity`.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        with = "function_stops_serde"
    )]
    pub stops: Vec<FunctionStop>,

    /// The exponential base for `exponential` interpolation.  Default is `1`
    /// (linear).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<f64>,

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
    pub default: Option<Value>,

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
pub enum StyleValue<T> {
    /// A literal value of the expected type, present directly in the JSON.
    Literal(T),
    /// A modern expression: a JSON array whose first element is an operator
    /// string.  The renderer evaluates this at runtime to produce a `T`.
    Expression(Expr),
    /// A legacy zoom/property function: a JSON object with a `"stops"` key.
    /// The renderer evaluates this at runtime to produce a `T`.
    Function(Function),
}

impl<T> StyleValue<T> {
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
    pub fn as_function(&self) -> Option<&Function> {
        match self {
            Self::Function(f) => Some(f),
            _ => None,
        }
    }
}

impl From<f64> for StyleValue<f64> {
    fn from(v: f64) -> Self {
        Self::Literal(v)
    }
}

impl From<bool> for StyleValue<bool> {
    fn from(v: bool) -> Self {
        Self::Literal(v)
    }
}

impl From<String> for StyleValue<String> {
    fn from(v: String) -> Self {
        Self::Literal(v)
    }
}

impl From<&str> for StyleValue<String> {
    fn from(v: &str) -> Self {
        Self::Literal(v.to_owned())
    }
}

impl From<Color> for StyleValue<Color> {
    fn from(v: Color) -> Self {
        Self::Literal(v)
    }
}

impl<'a> From<&'a str> for StyleValue<Color> {
    fn from(v: &'a str) -> Self {
        Self::Literal(Color::from(v))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn color_from_hex() {
        let c = Color::from("#ff0000");
        assert_eq!(c.as_str(), "#ff0000");
        assert_eq!(c.to_string(), "#ff0000");
    }

    #[test]
    fn color_from_rgb() {
        let c = Color::new("rgb(255,0,0)");
        assert_eq!(c.as_str(), "rgb(255,0,0)");
    }

    #[test]
    fn color_from_hsl() {
        let c = Color::new("hsl(47,79%,94%)");
        assert_eq!(c.as_str(), "hsl(47,79%,94%)");
    }

    #[test]
    fn color_from_named() {
        let c = Color::from("red");
        assert_eq!(c.as_str(), "red");
    }

    #[test]
    fn color_serde_roundtrip() {
        let c = Color::from("#aabbcc");
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"#aabbcc\"");
        let back: Color = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }

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
        let v: StyleValue<f64> = serde_json::from_value(original.clone()).unwrap();
        assert!(v.is_expression());
        assert_eq!(
            v.as_expression().and_then(|e| e.operator()),
            Some("interpolate")
        );
    }

    #[test]
    fn function_stops_only() {
        let f: Function = serde_json::from_value(json!({"stops": [[5, 1.0], [10, 4.0]]})).unwrap();
        assert_eq!(f.stops.len(), 2);
        assert_eq!(f.stops[0].input, json!(5));
        assert_eq!(f.stops[0].output, json!(1.0));
        assert!(f.base.is_none());
        assert!(f.property.is_none());
        assert!(f.function_type.is_none());
        assert!(f.default.is_none());
        assert!(f.color_space.is_none());
    }

    #[test]
    fn function_with_base() {
        let f: Function =
            serde_json::from_value(json!({"base": 1.4, "stops": [[5, 1.0], [10, 4.0]]})).unwrap();
        assert_eq!(f.base, Some(1.4));
        assert_eq!(f.stops.len(), 2);
    }

    #[test]
    fn function_with_property() {
        let f: Function = serde_json::from_value(json!({
            "property": "temperature",
            "stops": [[0, "blue"], [100, "red"]]
        }))
        .unwrap();
        assert_eq!(f.property.as_deref(), Some("temperature"));
        assert_eq!(f.stops[0].output, json!("blue"));
    }

    #[test]
    fn function_with_type_and_colorspace() {
        let f: Function = serde_json::from_value(json!({
            "type": "exponential",
            "colorSpace": "lab",
            "stops": [[0, "#fff"], [10, "#000"]]
        }))
        .unwrap();
        assert_eq!(f.function_type, Some(FunctionType::Exponential));
        assert_eq!(f.color_space, Some(ColorSpace::Lab));
    }

    #[test]
    fn function_with_default() {
        let f: Function = serde_json::from_value(json!({
            "stops": [[0, 1.0]],
            "default": 0.5
        }))
        .unwrap();
        assert_eq!(f.default, Some(json!(0.5)));
    }

    #[test]
    fn function_color_stops() {
        let f: Function = serde_json::from_value(
            json!({"stops": [[6, "hsl(47,79%,94%)"], [14, "hsl(42,49%,93%)"]]}),
        )
        .unwrap();
        assert_eq!(f.stops.len(), 2);
        assert_eq!(f.stops[0].input, json!(6));
        assert_eq!(f.stops[0].output, json!("hsl(47,79%,94%)"));
    }

    #[test]
    fn function_roundtrip() {
        let original = json!({"base": 1.5, "stops": [[0, "#fff"], [10, "#000"]]});
        let f: Function = serde_json::from_value(original.clone()).unwrap();
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
            let f: Function =
                serde_json::from_value(json!({"type": s, "stops": [[0, 1]]})).unwrap();
            assert_eq!(f.function_type, Some(expected));
        }
    }

    #[test]
    fn f64_literal() {
        let v: StyleValue<f64> = serde_json::from_value(json!(2.0)).unwrap();
        assert!(v.is_literal());
        assert_eq!(v.as_literal(), Some(&2.0));
        assert!(!v.is_expression());
        assert!(!v.is_function());
    }

    #[test]
    fn f64_expression() {
        let raw = json!(["interpolate", ["linear"], ["zoom"], 5, 1.0, 10, 4.0]);
        let v: StyleValue<f64> = serde_json::from_value(raw).unwrap();
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
        let v: StyleValue<f64> = serde_json::from_value(raw).unwrap();
        assert!(v.is_function());
        assert_eq!(v.as_function().and_then(|f| f.base), Some(1.4));
        assert!(!v.is_literal());
        assert!(!v.is_expression());
    }

    #[test]
    fn f64_step_expression() {
        let v: StyleValue<f64> =
            serde_json::from_value(json!(["step", ["zoom"], 0.5, 12, 1.0, 15, 2.0])).unwrap();
        assert_eq!(v.as_expression().and_then(|e| e.operator()), Some("step"));
    }

    #[test]
    fn bool_literal_true() {
        let v: StyleValue<bool> = serde_json::from_value(json!(true)).unwrap();
        assert_eq!(v.as_literal(), Some(&true));
    }

    #[test]
    fn bool_literal_false() {
        let v: StyleValue<bool> = serde_json::from_value(json!(false)).unwrap();
        assert_eq!(v.as_literal(), Some(&false));
    }

    #[test]
    fn bool_expression() {
        let v: StyleValue<bool> = serde_json::from_value(json!(["has", "name"])).unwrap();
        assert!(v.is_expression());
        assert_eq!(v.as_expression().and_then(|e| e.operator()), Some("has"));
    }

    #[test]
    fn string_literal() {
        let v: StyleValue<String> = serde_json::from_value(json!("butt")).unwrap();
        assert_eq!(v.as_literal(), Some(&"butt".to_owned()));
    }

    #[test]
    fn string_expression() {
        let v: StyleValue<String> = serde_json::from_value(json!(["get", "name"])).unwrap();
        assert!(v.is_expression());
        assert_eq!(v.as_expression().and_then(|e| e.operator()), Some("get"));
    }

    #[test]
    fn color_literal_hex() {
        let v: StyleValue<Color> = serde_json::from_value(json!("#ff0000")).unwrap();
        assert_eq!(v.as_literal(), Some(&Color::from("#ff0000")));
    }

    #[test]
    fn color_literal_hsl() {
        let v: StyleValue<Color> = serde_json::from_value(json!("hsl(47,79%,94%)")).unwrap();
        assert_eq!(v.as_literal().map(|c| c.as_str()), Some("hsl(47,79%,94%)"));
    }

    #[test]
    fn color_expression() {
        let v: StyleValue<Color> = serde_json::from_value(json!([
            "match",
            ["get", "class"],
            "residential",
            "#f00",
            "#000"
        ]))
        .unwrap();
        assert!(v.is_expression());
        assert_eq!(v.as_expression().and_then(|e| e.operator()), Some("match"));
    }

    #[test]
    fn color_function_stops() {
        let raw = json!({"stops": [[6, "hsl(47,79%,94%)"], [14, "hsl(42,49%,93%)"]]});
        let v: StyleValue<Color> = serde_json::from_value(raw).unwrap();
        assert!(v.is_function());
        let f = v.as_function().unwrap();
        assert_eq!(f.stops.len(), 2);
        assert_eq!(f.stops[0].output, json!("hsl(47,79%,94%)"));
    }

    #[test]
    fn color_function_with_base() {
        let raw = json!({"base": 1.0, "stops": [[6, "hsl(47,79%,94%)"], [14, "#aaa"]]});
        let v: StyleValue<Color> = serde_json::from_value(raw).unwrap();
        let f = v.as_function().unwrap();
        assert_eq!(f.base, Some(1.0));
    }

    #[test]
    fn maptiler_background_color_function() {
        // From maptiler_fmt.json — background-color with zoom stops
        let raw = json!({"stops": [[6, "hsl(47,79%,94%)"], [14, "hsl(42,49%,93%)"]]});
        let v: StyleValue<Color> = serde_json::from_value(raw).unwrap();
        let f = v.as_function().unwrap();
        assert_eq!(f.stops[0].input, json!(6));
        assert_eq!(f.stops[1].input, json!(14));
    }

    #[test]
    fn maptiler_fill_opacity_function() {
        // From maptiler_fmt.json — fill-opacity with zoom stops
        let raw = json!({"stops": [[0, 1], [8, 0.1]]});
        let v: StyleValue<f64> = serde_json::from_value(raw).unwrap();
        let f = v.as_function().unwrap();
        assert_eq!(f.stops[0].output, json!(1));
        assert_eq!(f.stops[1].output, json!(0.1));
    }

    #[test]
    fn roundtrip_f64_literal() {
        let original = json!(1.5);
        let v: StyleValue<f64> = serde_json::from_value(original.clone()).unwrap();
        let back = serde_json::to_value(&v).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn roundtrip_color_literal() {
        let original = json!("#aabbcc");
        let v: StyleValue<Color> = serde_json::from_value(original.clone()).unwrap();
        let back = serde_json::to_value(&v).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn roundtrip_expression() {
        let original = json!(["interpolate", ["linear"], ["zoom"], 5, 1.0, 10, 4.0]);
        let v: StyleValue<f64> = serde_json::from_value(original.clone()).unwrap();
        let back = serde_json::to_value(&v).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn roundtrip_function() {
        let original = json!({"base": 1.5, "stops": [[0, "#fff"], [10, "#000"]]});
        let v: StyleValue<Color> = serde_json::from_value(original.clone()).unwrap();
        let back = serde_json::to_value(&v).unwrap();
        assert_eq!(back, original);
    }

    #[test]
    fn from_impls() {
        assert_eq!(StyleValue::from(2.5_f64).as_literal(), Some(&2.5));
        assert_eq!(StyleValue::from(true).as_literal(), Some(&true));
        assert_eq!(
            StyleValue::from("butt").as_literal(),
            Some(&"butt".to_owned())
        );
        assert_eq!(
            StyleValue::from(Color::from("red")).as_literal(),
            Some(&Color::from("red"))
        );
        // &str → StyleValue<Color>
        let sc: StyleValue<Color> = StyleValue::from("blue");
        assert_eq!(sc.as_literal(), Some(&Color::from("blue")));
    }
}
