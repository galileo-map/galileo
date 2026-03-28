//! Typed representation of MapLibre style expressions.
//!
//! A MapLibre expression is a JSON array whose first element is an operator
//! string and whose remaining elements are arguments.  Arguments may themselves
//! be expressions (nested arrays), or bare JSON primitives.
//!
//! This module provides [`Expr`], a recursive enum with one variant per
//! operator group, plus a [`Literal`](Expr::Literal) variant for bare
//! primitives.
//!
//! # Deserialization
//!
//! [`Expr`] implements [`serde::Deserialize`] with a custom visitor.  A JSON
//! array is parsed as an expression (the first element names the operator); any
//! other JSON value is stored as [`Expr::Literal`].
//!
//! # Operator reference
//!
//! The full operator list is drawn from the
//! [MapLibre Style Spec — Expressions](https://maplibre.org/maplibre-style-spec/expressions/)
//! page.  Each variant's doc-comment lists the corresponding operator string(s).

use std::fmt;

use galileo::expr::{
    ControlPoint, CubicBezierInterpolation, ExponentialInterpolation, Expr, ExprGeometryType,
    ExprValue, LinearInterpolation, MatchCase, MatchExpr,
};
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::layer::{UNSUPPORTED, log_unsupported};
use crate::style::color::parse_css_color;

/// The interpolation curve used by [`Expr::Interpolate`] and its color-space
/// variants.
///
/// Encoded in JSON as a nested array: `["linear"]`, `["exponential", base]`,
/// or `["cubic-bezier", x1, y1, x2, y2]`.
#[derive(Debug, Clone, PartialEq)]
pub enum Interpolation {
    /// Linear interpolation: `["linear"]`.
    Linear,
    /// Exponential interpolation: `["exponential", base]`.
    Exponential {
        /// The exponential base — controls how quickly the output increases.
        /// Values close to 1 behave linearly; higher values accelerate toward
        /// the high end.
        base: f64,
    },
    /// Cubic Bézier interpolation: `["cubic-bezier", x1, y1, x2, y2]`.
    CubicBezier {
        /// X-coordinate of the first control point (must be in [0, 1]).
        x1: f64,
        /// Y-coordinate of the first control point.
        y1: f64,
        /// X-coordinate of the second control point (must be in [0, 1]).
        x2: f64,
        /// Y-coordinate of the second control point.
        y2: f64,
    },
}

// Custom deserialization: the JSON shape is `["linear"]`, `["exponential", N]`,
// or `["cubic-bezier", x1, y1, x2, y2]` — a flat array, not a struct.
impl<'de> Deserialize<'de> for Interpolation {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct InterpVisitor;

        impl<'de> Visitor<'de> for InterpVisitor {
            type Value = Interpolation;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str(
                    r#"an interpolation array: ["linear"], ["exponential", base], or ["cubic-bezier", x1, y1, x2, y2]"#,
                )
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                match name.as_str() {
                    "linear" => {
                        // Drain any extra elements (some styles pass a padding
                        // argument, e.g. `["linear", 1]`, which must be ignored).
                        while seq.next_element::<Value>()?.is_some() {}
                        Ok(Interpolation::Linear)
                    }
                    "exponential" => {
                        let base: f64 = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                        Ok(Interpolation::Exponential { base })
                    }
                    "cubic-bezier" => {
                        let x1: f64 = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                        let y1: f64 = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                        let x2: f64 = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(3, &self))?;
                        let y2: f64 = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(4, &self))?;
                        Ok(Interpolation::CubicBezier { x1, y1, x2, y2 })
                    }
                    other => Err(de::Error::unknown_variant(
                        other,
                        &["linear", "exponential", "cubic-bezier"],
                    )),
                }
            }
        }

        // Interpolation can appear either as a nested array inside a larger
        // expression array (where it arrives as a `Value`), or be deserialized
        // directly.  Handle both.
        d.deserialize_seq(InterpVisitor)
    }
}

impl Serialize for Interpolation {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        match self {
            Interpolation::Linear => {
                let mut seq = s.serialize_seq(Some(1))?;
                seq.serialize_element("linear")?;
                seq.end()
            }
            Interpolation::Exponential { base } => {
                let mut seq = s.serialize_seq(Some(2))?;
                seq.serialize_element("exponential")?;
                seq.serialize_element(base)?;
                seq.end()
            }
            Interpolation::CubicBezier { x1, y1, x2, y2 } => {
                let mut seq = s.serialize_seq(Some(5))?;
                seq.serialize_element("cubic-bezier")?;
                seq.serialize_element(x1)?;
                seq.serialize_element(y1)?;
                seq.serialize_element(x2)?;
                seq.serialize_element(y2)?;
                seq.end()
            }
        }
    }
}

/// A MapLibre style expression.
///
/// Expressions are the modern (post-v0.41.0) way to compute style property
/// values from the current zoom level and/or feature properties.  They are
/// represented in JSON as arrays: `["operator", arg1, arg2, ...]`.
///
/// # Deserialization
///
/// - A JSON **array** is parsed as an expression; the first element names the
///   operator and the remainder are arguments.
/// - Any other JSON value (number, string, bool, null, object) is stored as
///   [`Expr::Literal`].
///
/// Unknown operators are preserved as [`Expr::Unknown`] so that future spec
/// additions do not cause parse failures.
#[derive(Debug, Clone, PartialEq)]
pub enum MlExpr {
    /// A bare JSON primitive (number, string, boolean, null, object).
    ///
    /// Used wherever a sub-expression argument position holds a literal value
    /// rather than a nested expression.
    Literal(Value),

    /// `["let", var_name_1, var_value_1, ..., result_expr]`
    ///
    /// Binds named variables then evaluates the result expression in that
    /// scope.  Variable names must be string literals; their values are
    /// arbitrary expressions.
    Let {
        /// Alternating `(name, value)` pairs.
        bindings: Vec<(String, Box<MlExpr>)>,
        /// The body expression evaluated in the binding scope.
        body: Box<MlExpr>,
    },

    /// `["var", name]`
    ///
    /// References a variable bound by an enclosing `let` expression.
    Var(String),

    /// `["step", input, default_output, stop_input_1, stop_output_1, ...]`
    ///
    /// Piecewise-constant: returns the output of the last stop whose input
    /// is ≤ the evaluated `input`, or `default_output` if the input is less
    /// than the first stop.
    Step {
        /// The numeric input expression (e.g. `["zoom"]` or `["get", "pop"]`).
        input: Box<MlExpr>,
        /// Output when the input is below the first stop.
        default_output: Box<MlExpr>,
        /// `(threshold, output)` pairs in ascending order.
        stops: Vec<(f64, MlExpr)>,
    },

    /// `["interpolate", interp, input, stop_input_1, stop_output_1, ...]`
    ///
    /// Continuously interpolates between stop outputs.  Output type must be
    /// `number`, `array<number>`, `color`, `array<color>`, or `projection`.
    Interpolate {
        /// The interpolation curve.
        interpolation: Interpolation,
        /// The numeric input expression.
        input: Box<MlExpr>,
        /// `(threshold, output)` pairs in ascending order.
        stops: Vec<(f64, MlExpr)>,
    },

    /// `["interpolate-hcl", interp, input, stop_input_1, stop_output_1, ...]`
    ///
    /// Like [`Interpolate`](Expr::Interpolate) but performed in the
    /// Hue-Chroma-Luminance color space.  Output must be `color`.
    InterpolateHcl {
        /// The interpolation curve.
        interpolation: Interpolation,
        /// The numeric input expression.
        input: Box<MlExpr>,
        /// `(threshold, output)` pairs in ascending order.
        stops: Vec<(f64, MlExpr)>,
    },

    /// `["interpolate-lab", interp, input, stop_input_1, stop_output_1, ...]`
    ///
    /// Like [`Interpolate`](Expr::Interpolate) but performed in the CIELAB
    /// color space.  Output must be `color`.
    InterpolateLab {
        /// The interpolation curve.
        interpolation: Interpolation,
        /// The numeric input expression.
        input: Box<MlExpr>,
        /// `(threshold, output)` pairs in ascending order.
        stops: Vec<(f64, MlExpr)>,
    },

    /// `["get", property]` or `["get", property, object_expr]`
    ///
    /// Retrieves a feature property (or a property from `object_expr`).
    Get {
        /// The property name expression (typically a string literal).
        property: String,
        /// Optional object to retrieve the property from.
        object: Option<Box<MlExpr>>,
    },

    /// `["has", property]` or `["has", property, object_expr]`
    ///
    /// Tests whether a feature property (or a property in `object_expr`)
    /// exists.
    Has {
        /// The property name expression.
        property: Box<MlExpr>,
        /// Optional object to test.
        object: Option<Box<MlExpr>>,
    },

    /// `["!has", property]` or `["!has", property, object_expr]`
    ///
    /// Tests whether a feature property (or a property in `object_expr`)
    /// does not exist.
    NotHas {
        /// The property name expression.
        property: Box<MlExpr>,
        /// Optional object to test.
        object: Option<Box<MlExpr>>,
    },

    /// `["at", index, array]`
    ///
    /// Retrieves the item at `index` from `array`.
    At {
        /// The zero-based index expression.
        index: Box<MlExpr>,
        /// The array expression.
        array: Box<MlExpr>,
    },

    /// `["in", item, array]`
    ///
    /// Tests whether `item` is in `array`.
    //
    /// Docs above are written according to the specification, but in actual usage
    /// I've only seen `in` expression to be specified as: `["in", "property_name", "value1", "value2"]`.
    /// Galileo only support this non-spec variant.
    In {
        /// The needle expression.
        item: Box<MlExpr>,
        /// The haystack expression (array or string).
        array: Vec<MlExpr>,
    },

    /// `["!in", item, array]`
    ///
    /// Tests whether `item` is not in `array_or_string`.
    ///
    /// This property is not part of the official specification, but is widely used in
    /// styles. According to spec this is supposed to be specified as
    /// `["!", ["in", item, array]]`
    NotIn {
        /// The needle expression.
        item: Box<MlExpr>,
        /// The haystack expression (array or string).
        array: Vec<MlExpr>,
    },

    /// `["index-of", item, array_or_string]` or with an optional `from_index`.
    ///
    /// Returns the first index at which `item` appears, or -1.
    IndexOf {
        /// The needle expression.
        item: Box<MlExpr>,
        /// The haystack expression.
        array_or_string: Box<MlExpr>,
        /// Optional start index for the search.
        from_index: Option<Box<MlExpr>>,
    },

    /// `["slice", array_or_string, start]` or `["slice", ..., start, end]`.
    ///
    /// Returns a subarray or substring.
    Slice {
        /// The source expression.
        array_or_string: Box<MlExpr>,
        /// The inclusive start index.
        start: Box<MlExpr>,
        /// The exclusive end index (optional).
        end: Option<Box<MlExpr>>,
    },

    /// `["length", array_or_string]`
    ///
    /// Returns the length of an array or string.
    Length(Box<MlExpr>),

    /// `["global-state", property_name]`
    ///
    /// Retrieves a property from global state set via platform APIs.
    GlobalState(Box<MlExpr>),

    /// `["case", cond_1, out_1, ..., cond_n, out_n, fallback]`
    ///
    /// Returns the output for the first condition that evaluates to `true`.
    Case {
        /// `(condition, output)` pairs evaluated in order.
        branches: Vec<(Box<MlExpr>, Box<MlExpr>)>,
        /// Output when no condition matches.
        fallback: Box<MlExpr>,
    },

    /// `["match", input, label_1, out_1, ..., label_n, out_n, fallback]`
    ///
    /// Returns the output for the first label that equals `input`.  A label
    /// may be a single value or an array of values.
    Match {
        /// The input expression.
        input: Box<MlExpr>,
        /// `(labels, output)` pairs.  Each label is the raw JSON value as it
        /// appeared in the source — either a scalar (e.g. `"residential"`) or
        /// an array of scalars (e.g. `["motorway", "trunk"]`).  Preserving the
        /// original form is required for lossless round-trip serialization.
        branches: Vec<(Value, Box<MlExpr>)>,
        /// Output when no label matches.
        fallback: Box<MlExpr>,
    },

    /// `["coalesce", expr_1, ..., expr_n]`
    ///
    /// Evaluates each expression in order and returns the first non-null result.
    Coalesce(Vec<MlExpr>),

    /// `["all", input_1, ..., input_n]` — logical AND (short-circuit).
    All(Vec<MlExpr>),

    /// `["any", input_1, ..., input_n]` — logical OR (short-circuit).
    Any(Vec<MlExpr>),

    /// `["!", input]` — logical NOT.
    Not(Box<MlExpr>),

    /// `["==", a, b]` (with optional collator) — equality comparison.
    Eq(Box<MlExpr>, Box<MlExpr>),

    /// `["!=", a, b]` — inequality comparison.
    Ne(Box<MlExpr>, Box<MlExpr>),

    /// `[">", a, b]` — greater than.
    Gt(Box<MlExpr>, Box<MlExpr>),

    /// `[">=", a, b]` — greater than or equal.
    Gte(Box<MlExpr>, Box<MlExpr>),

    /// `["<", a, b]` — less than.
    Lt(Box<MlExpr>, Box<MlExpr>),

    /// `["<=", a, b]` — less than or equal.
    Lte(Box<MlExpr>, Box<MlExpr>),

    /// `["within", geojson]`
    ///
    /// Returns `true` if the feature is fully inside the given GeoJSON geometry.
    Within(Value),

    /// `["+", n_1, ..., n_n]` — sum.
    Add(Vec<MlExpr>),

    /// `["*", n_1, ..., n_n]` — product.
    Mul(Vec<MlExpr>),

    /// `["-", a, b]` or `["-", a]` — subtraction or negation.
    Sub(Box<MlExpr>, Option<Box<MlExpr>>),

    /// `["/", a, b]` — division.
    Div(Box<MlExpr>, Box<MlExpr>),

    /// `["%", a, b]` — modulo.
    Mod(Box<MlExpr>, Box<MlExpr>),

    /// `["^", base, exp]` — exponentiation.
    Pow(Box<MlExpr>, Box<MlExpr>),

    /// `["sqrt", n]` — square root.
    Sqrt(Box<MlExpr>),

    /// `["abs", n]` — absolute value.
    Abs(Box<MlExpr>),

    /// `["ceil", n]` — ceiling.
    Ceil(Box<MlExpr>),

    /// `["floor", n]` — floor.
    Floor(Box<MlExpr>),

    /// `["round", n]` — round to nearest integer (half away from zero).
    Round(Box<MlExpr>),

    /// `["min", n_1, ..., n_n]` — minimum.
    Min(Vec<MlExpr>),

    /// `["max", n_1, ..., n_n]` — maximum.
    Max(Vec<MlExpr>),

    /// `["log2", n]` — base-2 logarithm.
    Log2(Box<MlExpr>),

    /// `["log10", n]` — base-10 logarithm.
    Log10(Box<MlExpr>),

    /// `["ln", n]` — natural logarithm.
    Ln(Box<MlExpr>),

    /// `["sin", n]` — sine (radians).
    Sin(Box<MlExpr>),

    /// `["cos", n]` — cosine (radians).
    Cos(Box<MlExpr>),

    /// `["tan", n]` — tangent (radians).
    Tan(Box<MlExpr>),

    /// `["asin", n]` — arcsine.
    Asin(Box<MlExpr>),

    /// `["acos", n]` — arccosine.
    Acos(Box<MlExpr>),

    /// `["atan", n]` — arctangent.
    Atan(Box<MlExpr>),

    /// `["ln2"]` — the mathematical constant ln(2).
    Ln2,

    /// `["pi"]` — the mathematical constant π.
    Pi,

    /// `["e"]` — the mathematical constant e.
    E,

    /// `["distance", geojson]`
    ///
    /// Returns the shortest distance in metres between the feature and the
    /// given GeoJSON geometry.
    Distance(Value),

    /// `["rgb", r, g, b]` — constructs a color from RGB components (0–255).
    Rgb(Box<MlExpr>, Box<MlExpr>, Box<MlExpr>),

    /// `["rgba", r, g, b, a]` — constructs a color from RGBA components.
    Rgba(Box<MlExpr>, Box<MlExpr>, Box<MlExpr>, Box<MlExpr>),

    /// `["to-rgba", color]` — returns `[r, g, b, a]` components of a color.
    ToRgba(Box<MlExpr>),

    /// `["literal", value]` — wraps a JSON array or object as a literal.
    LiteralExpr(Value),

    /// `["typeof", value]` — returns a string describing the type of `value`.
    TypeOf(Box<MlExpr>),

    /// `["to-string", value]` — converts `value` to a string.
    ToString(Box<MlExpr>),

    /// `["to-number", value_1, ..., value_n]` — converts to a number.
    ToNumber(Vec<MlExpr>),

    /// `["to-boolean", value]` — converts to a boolean.
    ToBoolean(Box<MlExpr>),

    /// `["to-color", value_1, ..., value_n]` — converts to a color.
    ToColor(Vec<MlExpr>),

    /// `["number", value_1, ..., value_n]` — asserts input is a number.
    NumberAssertion(Vec<MlExpr>),

    /// `["string", value_1, ..., value_n]` — asserts input is a string.
    StringAssertion(Vec<MlExpr>),

    /// `["boolean", value_1, ..., value_n]` — asserts input is a boolean.
    BooleanAssertion(Vec<MlExpr>),

    /// `["object", value_1, ..., value_n]` — asserts input is an object.
    ObjectAssertion(Vec<MlExpr>),

    /// `["array", value]` / `["array", type, value]` / `["array", type, length, value]`
    ///
    /// Asserts input is an array, optionally with element type and length.
    ArrayAssertion {
        /// Optional element type assertion (`"string"`, `"number"`, `"boolean"`).
        element_type: Option<String>,
        /// Optional expected array length.
        length: Option<u64>,
        /// The value to assert.
        value: Box<MlExpr>,
    },

    /// `["collator", options]` — returns a locale-aware collator.
    Collator(Value),

    /// `["format", input_1, style_1?, ..., input_n, style_n?]`
    ///
    /// Returns a `formatted` string for rich text labels.  Each section is a
    /// `(content, style_overrides)` pair.
    Format(Vec<(Box<MlExpr>, Option<Value>)>),

    /// `["image", name]` — returns an image for use in icons/patterns.
    Image(Box<MlExpr>),

    /// `["number-format", input, options]` — formats a number as a string.
    NumberFormat {
        /// The number expression to format.
        input: Box<MlExpr>,
        /// Formatting options object (locale, currency, min/max fraction digits, etc.).
        options: Value,
    },

    /// `["get", ...]` — see [`Expr::Get`] (aliased here for feature state).
    ///
    /// `["feature-state", property]` — retrieves a property from feature state.
    FeatureState(Box<MlExpr>),

    /// `["geometry-type"]` — returns `"Point"`, `"LineString"`, or `"Polygon"`.
    GeometryType,

    /// `["id"]` — returns the feature's id.
    Id,

    /// `["properties"]` — returns the feature's properties object.
    Properties,

    /// `["accumulated"]` — gets the accumulated cluster property value.
    Accumulated,

    /// `["line-progress"]` — gets the progress along a gradient line.
    LineProgress,

    /// `["zoom"]` — the current map zoom level.
    Zoom,

    /// `["heatmap-density"]` — kernel density estimate at the current pixel.
    HeatmapDensity,

    /// `["elevation"]` — elevation in metres at the current pixel.
    Elevation,

    /// `["upcase", s]` — converts string to uppercase.
    Upcase(Box<MlExpr>),

    /// `["downcase", s]` — converts string to lowercase.
    Downcase(Box<MlExpr>),

    /// `["concat", s_1, ..., s_n]` — concatenates strings.
    Concat(Vec<MlExpr>),

    /// `["is-supported-script", s]` — returns `true` if the string renders legibly.
    IsSupportedScript(Box<MlExpr>),

    /// `["resolved-locale", collator]` — returns the IETF tag of the locale in use.
    ResolvedLocale(Box<MlExpr>),

    /// `["split", input, separator]` — splits a string into an array.
    Split(Box<MlExpr>, Box<MlExpr>),

    /// `["join", array, separator]` — joins an array into a string.
    Join(Box<MlExpr>, Box<MlExpr>),

    /// An unrecognised operator.
    ///
    /// Preserved so that unknown/future operators do not cause a parse failure.
    /// The first element is the operator string; the rest are raw argument values.
    Unknown {
        /// The unrecognised operator name.
        operator: String,
        /// The raw JSON arguments.
        args: Vec<Value>,
    },
}

impl MlExpr {
    /// Returns the operator name for this expression, or `None` for
    /// [`Expr::Literal`] and [`Expr::LiteralExpr`].
    ///
    /// Useful for quick operator-based dispatch in the renderer.
    pub fn operator(&self) -> Option<&str> {
        Some(match self {
            // Variable binding
            MlExpr::Let { .. } => "let",
            MlExpr::Var(_) => "var",
            // Ramps / curves
            MlExpr::Step { .. } => "step",
            MlExpr::Interpolate { .. } => "interpolate",
            MlExpr::InterpolateHcl { .. } => "interpolate-hcl",
            MlExpr::InterpolateLab { .. } => "interpolate-lab",
            // Lookup
            MlExpr::Get { .. } => "get",
            MlExpr::Has { .. } => "has",
            MlExpr::NotHas { .. } => "!has",
            MlExpr::At { .. } => "at",
            MlExpr::In { .. } => "in",
            MlExpr::NotIn { .. } => "!in",
            MlExpr::IndexOf { .. } => "index-of",
            MlExpr::Slice { .. } => "slice",
            MlExpr::Length(_) => "length",
            MlExpr::GlobalState(_) => "global-state",
            // Decision
            MlExpr::Case { .. } => "case",
            MlExpr::Match { .. } => "match",
            MlExpr::Coalesce(_) => "coalesce",
            MlExpr::All(_) => "all",
            MlExpr::Any(_) => "any",
            MlExpr::Not(_) => "!",
            MlExpr::Eq(_, _) => "==",
            MlExpr::Ne(_, _) => "!=",
            MlExpr::Gt(_, _) => ">",
            MlExpr::Gte(_, _) => ">=",
            MlExpr::Lt(_, _) => "<",
            MlExpr::Lte(_, _) => "<=",
            MlExpr::Within(_) => "within",
            // Math
            MlExpr::Add(_) => "+",
            MlExpr::Mul(_) => "*",
            MlExpr::Sub(_, _) => "-",
            MlExpr::Div(_, _) => "/",
            MlExpr::Mod(_, _) => "%",
            MlExpr::Pow(_, _) => "^",
            MlExpr::Sqrt(_) => "sqrt",
            MlExpr::Abs(_) => "abs",
            MlExpr::Ceil(_) => "ceil",
            MlExpr::Floor(_) => "floor",
            MlExpr::Round(_) => "round",
            MlExpr::Min(_) => "min",
            MlExpr::Max(_) => "max",
            MlExpr::Log2(_) => "log2",
            MlExpr::Log10(_) => "log10",
            MlExpr::Ln(_) => "ln",
            MlExpr::Sin(_) => "sin",
            MlExpr::Cos(_) => "cos",
            MlExpr::Tan(_) => "tan",
            MlExpr::Asin(_) => "asin",
            MlExpr::Acos(_) => "acos",
            MlExpr::Atan(_) => "atan",
            MlExpr::Ln2 => "ln2",
            MlExpr::Pi => "pi",
            MlExpr::E => "e",
            MlExpr::Distance(_) => "distance",
            // Color
            MlExpr::Rgb(_, _, _) => "rgb",
            MlExpr::Rgba(_, _, _, _) => "rgba",
            MlExpr::ToRgba(_) => "to-rgba",
            // Type operators
            MlExpr::LiteralExpr(_) => "literal",
            MlExpr::TypeOf(_) => "typeof",
            MlExpr::ToString(_) => "to-string",
            MlExpr::ToNumber(_) => "to-number",
            MlExpr::ToBoolean(_) => "to-boolean",
            MlExpr::ToColor(_) => "to-color",
            MlExpr::NumberAssertion(_) => "number",
            MlExpr::StringAssertion(_) => "string",
            MlExpr::BooleanAssertion(_) => "boolean",
            MlExpr::ObjectAssertion(_) => "object",
            MlExpr::ArrayAssertion { .. } => "array",
            MlExpr::Collator(_) => "collator",
            MlExpr::Format(_) => "format",
            MlExpr::Image(_) => "image",
            MlExpr::NumberFormat { .. } => "number-format",
            // Feature data
            MlExpr::FeatureState(_) => "feature-state",
            MlExpr::GeometryType => "geometry-type",
            MlExpr::Id => "id",
            MlExpr::Properties => "properties",
            MlExpr::Accumulated => "accumulated",
            MlExpr::LineProgress => "line-progress",
            // Camera
            MlExpr::Zoom => "zoom",
            // Heatmap
            MlExpr::HeatmapDensity => "heatmap-density",
            // Color relief
            MlExpr::Elevation => "elevation",
            // String
            MlExpr::Upcase(_) => "upcase",
            MlExpr::Downcase(_) => "downcase",
            MlExpr::Concat(_) => "concat",
            MlExpr::IsSupportedScript(_) => "is-supported-script",
            MlExpr::ResolvedLocale(_) => "resolved-locale",
            MlExpr::Split(_, _) => "split",
            MlExpr::Join(_, _) => "join",
            // Unknown
            MlExpr::Unknown { operator, .. } => operator.as_str(),
            // No operator for bare literals
            MlExpr::Literal(_) => return None,
        })
    }

    pub fn to_galileo_expr(&self) -> Option<Expr> {
        fn get_prop(prop: &MlExpr) -> Option<Expr> {
            if let MlExpr::Literal(Value::String(property_name)) = prop {
                if property_name == "$type" {
                    // TODO: this is supposed to check geometry type of the layer,
                    // but in Galileo this is done by the symbol. Need to check if
                    // it is possible in Maplibre to draw a layer with wrong geometry
                    // type. If so, do we need to do anything for it?
                    return None;
                }

                Some(Expr::Get(property_name.clone()))
            } else {
                log::debug!("{UNSUPPORTED} Expected '{prop:?}' to be a property name");
                None
            }
        }

        fn op(
            prop: &MlExpr,
            value: &MlExpr,
            f: impl FnOnce(Box<Expr>, Box<Expr>) -> Expr,
        ) -> Option<Expr> {
            let property = get_prop(prop)?;

            let val = match property {
                Expr::GeomType => match value {
                    MlExpr::Literal(Value::String(v)) if v.to_lowercase() == "point" => {
                        ExprValue::GeomType(ExprGeometryType::Point).into()
                    }
                    MlExpr::Literal(Value::String(v)) if v.to_lowercase() == "linestring" => {
                        ExprValue::GeomType(ExprGeometryType::Line).into()
                    }
                    MlExpr::Literal(Value::String(v)) if v.to_lowercase() == "multilinestring" => {
                        ExprValue::GeomType(ExprGeometryType::MultiLine).into()
                    }
                    MlExpr::Literal(Value::String(v)) if v.to_lowercase() == "polygon" => {
                        ExprValue::GeomType(ExprGeometryType::Polygon).into()
                    }
                    MlExpr::Literal(Value::String(v)) if v.to_lowercase() == "multipolygon" => {
                        ExprValue::GeomType(ExprGeometryType::MultiPolygon).into()
                    }
                    _ => {
                        log_unsupported!(format!("geometry type {value:?}"));
                        return None;
                    }
                },
                _ => value.to_galileo_expr()?,
            };

            Some(f(Box::new(property), Box::new(val)))
        }

        fn contains(prop: &MlExpr, vals: &[MlExpr]) -> Option<Expr> {
            let property = get_prop(prop)?;

            let mut values = vec![];
            for val in vals {
                let val = val.to_galileo_expr()?;
                values.push(val);
            }

            Some(Expr::In {
                needle: Box::new(property),
                haystack: values,
            })
        }

        fn literal(v: &Value) -> Option<ExprValue<String>> {
            match v {
                Value::Bool(v) => Some(ExprValue::Boolean(*v)),
                Value::Number(v) => Some(ExprValue::Number(v.as_f64()?)),
                Value::String(v) => Some(
                    parse_css_color(v)
                        .map(ExprValue::from)
                        .unwrap_or_else(|| ExprValue::String(v.clone())),
                ),
                Value::Null => Some(ExprValue::Null),
                _ => None,
            }
        }

        Some(match self {
            MlExpr::Literal(l) => literal(l)?.into(),
            MlExpr::All(parts) => Expr::All(
                parts
                    .iter()
                    .map(|v| v.to_galileo_expr())
                    .collect::<Option<Vec<_>>>()?,
            ),
            MlExpr::Eq(prop, value) => op(prop, value, Expr::Eq)?,
            MlExpr::Ne(prop, value) => op(prop, value, Expr::Ne)?,
            MlExpr::Gt(prop, value) => op(prop, value, Expr::Gt)?,
            MlExpr::Gte(prop, value) => op(prop, value, Expr::Gte)?,
            MlExpr::Lt(prop, value) => op(prop, value, Expr::Lt)?,
            MlExpr::Lte(prop, value) => op(prop, value, Expr::Lte)?,
            MlExpr::Has { property, .. } => Expr::Ne(
                Box::new(get_prop(property)?),
                Box::new(ExprValue::Null.into()),
            ),
            MlExpr::NotHas { property, .. } => Expr::Eq(
                Box::new(get_prop(property)?),
                Box::new(ExprValue::Null.into()),
            ),
            MlExpr::In { item, array } => contains(item, array)?,
            MlExpr::NotIn { item, array } => contains(item, array)?,
            MlExpr::Interpolate {
                interpolation,
                input,
                stops,
            } => interpolation_to_galileo(interpolation, input, stops)?,
            MlExpr::Zoom => Expr::Zoom,
            MlExpr::Get { property, object } => {
                if object.is_some() {
                    log_unsupported!(format!("get from an object"));
                    return None;
                }

                Expr::Get(property.clone())
            }
            MlExpr::Match {
                input,
                branches,
                fallback,
            } => Expr::Match(Box::new(MatchExpr {
                input: input.to_galileo_expr()?,
                cases: branches
                    .iter()
                    .map(|(values, out)| {
                        let entries = match values {
                            Value::Array(entries) => entries.clone(),
                            v => vec![v.clone()],
                        };

                        Some(MatchCase {
                            in_values: entries
                                .into_iter()
                                .map(|v| literal(&v))
                                .collect::<Option<Vec<_>>>()?,
                            out: out.to_galileo_expr()?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?,
                fallback: fallback.to_galileo_expr()?,
            })),
            _ => {
                log::debug!("{UNSUPPORTED} Expression {self:?} is not supported yet");
                return None;
            }
        })
    }
}

fn interpolation_to_galileo(
    interpolation: &Interpolation,
    input: &MlExpr,
    stops: &[(f64, MlExpr)],
) -> Option<Expr> {
    Some(match interpolation {
        Interpolation::Linear => Expr::Linear(Box::new(LinearInterpolation {
            input: input.to_galileo_expr()?,
            control_points: stops
                .iter()
                .map(|(input, output)| {
                    output.to_galileo_expr().map(|out| ControlPoint {
                        input: (*input).into(),
                        output: out,
                    })
                })
                .collect::<Option<Vec<_>>>()?,
        })),
        Interpolation::Exponential { base } => {
            Expr::Exponential(Box::new(ExponentialInterpolation {
                base: *base,
                input: input.to_galileo_expr()?,
                control_points: stops
                    .iter()
                    .map(|(input, output)| {
                        output.to_galileo_expr().map(|out| ControlPoint {
                            input: (*input).into(),
                            output: out,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?,
            }))
        }
        Interpolation::CubicBezier { x1, y1, x2, y2 } => {
            Expr::CubicBezier(Box::new(CubicBezierInterpolation {
                curve_params: [*x1, *y1, *x2, *y2],
                input: input.to_galileo_expr()?,
                control_points: stops
                    .iter()
                    .map(|(input, output)| {
                        output.to_galileo_expr().map(|out| ControlPoint {
                            input: (*input).into(),
                            output: out,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?,
            }))
        }
    })
}

impl<'de> Deserialize<'de> for MlExpr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // Only JSON arrays are valid top-level expressions.  Non-arrays
        // produce an error so that `StyleValue<T>` can fall through to the
        // `Function` or `Literal(T)` variant instead.
        let v = Value::deserialize(d)?;
        match v {
            Value::Array(arr) => parse_expr_array(arr).map_err(de::Error::custom),
            _ => Err(de::Error::custom("expression must be a JSON array")),
        }
    }
}

/// Parse an [`Expr`] from a [`Value`].
///
/// Arrays are interpreted as expressions; all other JSON values become
/// [`Expr::Literal`].
pub(crate) fn expr_from_value(v: Value) -> Result<MlExpr, String> {
    match v {
        Value::Array(arr) => parse_expr_array(arr),
        other => Ok(MlExpr::Literal(other)),
    }
}

fn box_expr(v: Value) -> Result<Box<MlExpr>, String> {
    expr_from_value(v).map(Box::new)
}

/// Consume an argument from a positioned argument list, returning a descriptive
/// error if the index is out of bounds.
fn take_arg(args: &mut Vec<Value>, pos: usize, op: &str) -> Result<Value, String> {
    if args.is_empty() {
        Err(format!(
            "expression \"{op}\": expected argument at position {pos} but got none"
        ))
    } else {
        Ok(args.remove(0))
    }
}

/// Parse `["operator", arg1, arg2, ...]` where `arr` is the full array
/// (operator is `arr[0]`).
fn parse_expr_array(mut arr: Vec<Value>) -> Result<MlExpr, String> {
    if arr.is_empty() {
        return Err("expression array is empty".into());
    }

    let op_val = arr.remove(0);
    let op = match &op_val {
        Value::String(s) => s.clone(),
        _ => {
            // Not a string operator → treat the whole original array as a literal.
            let mut full = vec![op_val];
            full.extend(arr);
            return Ok(MlExpr::Literal(Value::Array(full)));
        }
    };
    // `arr` now contains the arguments (op removed).
    let mut args = arr;

    match op.as_str() {
        "let" => {
            // ["let", name1, val1, ..., nameN, valN, body]
            // Must have at least 3 elements: one binding pair + body.
            if args.len() < 3 || args.len().is_multiple_of(2) {
                return Err(format!(
                    "expression \"let\": expected odd number of arguments ≥ 3, got {}",
                    args.len()
                ));
            }
            let body_val = args.pop().unwrap();
            let body = box_expr(body_val)?;
            let mut bindings = Vec::new();
            while args.len() >= 2 {
                let name = match args.remove(0) {
                    Value::String(s) => s,
                    other => {
                        return Err(format!(
                            "expression \"let\": binding name must be a string, got {other}"
                        ));
                    }
                };
                let val = box_expr(args.remove(0))?;
                bindings.push((name, val));
            }
            Ok(MlExpr::Let { bindings, body })
        }

        "var" => {
            let name = match take_arg(&mut args, 1, "var")? {
                Value::String(s) => s,
                other => {
                    return Err(format!(
                        "expression \"var\": name must be a string, got {other}"
                    ));
                }
            };
            Ok(MlExpr::Var(name))
        }

        "step" => {
            // ["step", input, default_output, stop1_in, stop1_out, ...]
            if args.len() < 2 {
                return Err(format!(
                    "expression \"step\": expected at least 2 arguments, got {}",
                    args.len()
                ));
            }
            let input = box_expr(args.remove(0))?;
            let default_output = box_expr(args.remove(0))?;
            let stops = parse_stop_pairs(&mut args, "step")?;
            Ok(MlExpr::Step {
                input,
                default_output,
                stops,
            })
        }

        "interpolate" | "interpolate-hcl" | "interpolate-lab" => {
            // ["interpolate", interp, input, stop1_in, stop1_out, ...]
            if args.len() < 2 {
                return Err(format!(
                    "expression \"{op}\": expected at least 2 arguments, got {}",
                    args.len()
                ));
            }
            let interp_val = args.remove(0);
            let interpolation = serde_json::from_value::<Interpolation>(interp_val)
                .map_err(|e| format!("expression \"{op}\": bad interpolation: {e}"))?;
            let input = box_expr(args.remove(0))?;
            let stops = parse_stop_pairs(&mut args, &op)?;
            match op.as_str() {
                "interpolate" => Ok(MlExpr::Interpolate {
                    interpolation,
                    input,
                    stops,
                }),
                "interpolate-hcl" => Ok(MlExpr::InterpolateHcl {
                    interpolation,
                    input,
                    stops,
                }),
                _ => Ok(MlExpr::InterpolateLab {
                    interpolation,
                    input,
                    stops,
                }),
            }
        }

        "get" => {
            let property = take_arg(&mut args, 1, "get")?
                .as_str()
                .ok_or_else(|| "Expected string value for get argument".to_string())?
                .to_owned();
            let object = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(MlExpr::Get { property, object })
        }

        "has" => {
            let property = box_expr(take_arg(&mut args, 1, "has")?)?;
            let object = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(MlExpr::Has { property, object })
        }

        "!has" => {
            let property = box_expr(take_arg(&mut args, 1, "!has")?)?;
            let object = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(MlExpr::NotHas { property, object })
        }

        "at" => {
            let index = box_expr(take_arg(&mut args, 1, "at")?)?;
            let array = box_expr(take_arg(&mut args, 2, "at")?)?;
            Ok(MlExpr::At { index, array })
        }

        "in" => {
            let item = box_expr(take_arg(&mut args, 1, "!in")?)?;
            let mut vals = vec![];
            let mut index = 2;
            while !args.is_empty() {
                vals.push(expr_from_value(take_arg(&mut args, index, "in")?)?);
                index += 1;
            }

            Ok(MlExpr::In { item, array: vals })
        }

        "!in" => {
            let item = box_expr(take_arg(&mut args, 1, "!in")?)?;
            let mut vals = vec![];
            let mut index = 2;
            while !args.is_empty() {
                vals.push(expr_from_value(take_arg(&mut args, index, "in")?)?);
                index += 1;
            }

            Ok(MlExpr::NotIn { item, array: vals })
        }

        "index-of" => {
            let item = box_expr(take_arg(&mut args, 1, "index-of")?)?;
            let array_or_string = box_expr(take_arg(&mut args, 2, "index-of")?)?;
            let from_index = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(MlExpr::IndexOf {
                item,
                array_or_string,
                from_index,
            })
        }

        "slice" => {
            let array_or_string = box_expr(take_arg(&mut args, 1, "slice")?)?;
            let start = box_expr(take_arg(&mut args, 2, "slice")?)?;
            let end = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(MlExpr::Slice {
                array_or_string,
                start,
                end,
            })
        }

        "length" => Ok(MlExpr::Length(box_expr(take_arg(&mut args, 1, "length")?)?)),

        "global-state" => Ok(MlExpr::GlobalState(box_expr(take_arg(
            &mut args,
            1,
            "global-state",
        )?)?)),

        "case" => {
            // ["case", cond1, out1, ..., condN, outN, fallback]
            if args.len() < 3 || args.len().is_multiple_of(2) {
                return Err(format!(
                    "expression \"case\": expected odd number of arguments ≥ 3, got {}",
                    args.len()
                ));
            }
            let fallback_val = args.pop().unwrap();
            let fallback = box_expr(fallback_val)?;
            let mut branches = Vec::new();
            while args.len() >= 2 {
                let cond = box_expr(args.remove(0))?;
                let out = box_expr(args.remove(0))?;
                branches.push((cond, out));
            }
            Ok(MlExpr::Case { branches, fallback })
        }

        "match" => {
            // ["match", input, label1, out1, ..., labelN, outN, fallback]
            if args.len() < 4 || !args.len().is_multiple_of(2) {
                return Err(format!(
                    "expression \"match\": expected even number of arguments ≥ 4, got {}",
                    args.len()
                ));
            }
            let input = box_expr(args.remove(0))?;
            let fallback_val = args.pop().unwrap();
            let fallback = box_expr(fallback_val)?;
            let mut branches = Vec::new();
            while args.len() >= 2 {
                let labels = args.remove(0);
                let out = box_expr(args.remove(0))?;
                branches.push((labels, out));
            }
            Ok(MlExpr::Match {
                input,
                branches,
                fallback,
            })
        }

        "coalesce" => Ok(MlExpr::Coalesce(parse_variadic(args)?)),
        "all" => Ok(MlExpr::All(parse_variadic(args)?)),
        "any" => Ok(MlExpr::Any(parse_variadic(args)?)),

        "!" => Ok(MlExpr::Not(box_expr(take_arg(&mut args, 1, "!")?)?)),

        "==" => Ok(MlExpr::Eq(
            box_expr(take_arg(&mut args, 1, "==")?)?,
            box_expr(take_arg(&mut args, 2, "==")?)?,
        )),
        "!=" => Ok(MlExpr::Ne(
            box_expr(take_arg(&mut args, 1, "!=")?)?,
            box_expr(take_arg(&mut args, 2, "!=")?)?,
        )),
        ">" => Ok(MlExpr::Gt(
            box_expr(take_arg(&mut args, 1, ">")?)?,
            box_expr(take_arg(&mut args, 2, ">")?)?,
        )),
        ">=" => Ok(MlExpr::Gte(
            box_expr(take_arg(&mut args, 1, ">=")?)?,
            box_expr(take_arg(&mut args, 2, ">=")?)?,
        )),
        "<" => Ok(MlExpr::Lt(
            box_expr(take_arg(&mut args, 1, "<")?)?,
            box_expr(take_arg(&mut args, 2, "<")?)?,
        )),
        "<=" => Ok(MlExpr::Lte(
            box_expr(take_arg(&mut args, 1, "<=")?)?,
            box_expr(take_arg(&mut args, 2, "<=")?)?,
        )),

        "within" => Ok(MlExpr::Within(take_arg(&mut args, 1, "within")?)),

        "+" => Ok(MlExpr::Add(parse_variadic(args)?)),
        "*" => Ok(MlExpr::Mul(parse_variadic(args)?)),

        "-" => {
            let a = box_expr(take_arg(&mut args, 1, "-")?)?;
            let b = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(MlExpr::Sub(a, b))
        }

        "/" => Ok(MlExpr::Div(
            box_expr(take_arg(&mut args, 1, "/")?)?,
            box_expr(take_arg(&mut args, 2, "/")?)?,
        )),
        "%" => Ok(MlExpr::Mod(
            box_expr(take_arg(&mut args, 1, "%")?)?,
            box_expr(take_arg(&mut args, 2, "%")?)?,
        )),
        "^" => Ok(MlExpr::Pow(
            box_expr(take_arg(&mut args, 1, "^")?)?,
            box_expr(take_arg(&mut args, 2, "^")?)?,
        )),

        "sqrt" => Ok(MlExpr::Sqrt(box_expr(take_arg(&mut args, 1, "sqrt")?)?)),
        "abs" => Ok(MlExpr::Abs(box_expr(take_arg(&mut args, 1, "abs")?)?)),
        "ceil" => Ok(MlExpr::Ceil(box_expr(take_arg(&mut args, 1, "ceil")?)?)),
        "floor" => Ok(MlExpr::Floor(box_expr(take_arg(&mut args, 1, "floor")?)?)),
        "round" => Ok(MlExpr::Round(box_expr(take_arg(&mut args, 1, "round")?)?)),

        "min" => Ok(MlExpr::Min(parse_variadic(args)?)),
        "max" => Ok(MlExpr::Max(parse_variadic(args)?)),

        "log2" => Ok(MlExpr::Log2(box_expr(take_arg(&mut args, 1, "log2")?)?)),
        "log10" => Ok(MlExpr::Log10(box_expr(take_arg(&mut args, 1, "log10")?)?)),
        "ln" => Ok(MlExpr::Ln(box_expr(take_arg(&mut args, 1, "ln")?)?)),
        "sin" => Ok(MlExpr::Sin(box_expr(take_arg(&mut args, 1, "sin")?)?)),
        "cos" => Ok(MlExpr::Cos(box_expr(take_arg(&mut args, 1, "cos")?)?)),
        "tan" => Ok(MlExpr::Tan(box_expr(take_arg(&mut args, 1, "tan")?)?)),
        "asin" => Ok(MlExpr::Asin(box_expr(take_arg(&mut args, 1, "asin")?)?)),
        "acos" => Ok(MlExpr::Acos(box_expr(take_arg(&mut args, 1, "acos")?)?)),
        "atan" => Ok(MlExpr::Atan(box_expr(take_arg(&mut args, 1, "atan")?)?)),

        "ln2" => Ok(MlExpr::Ln2),
        "pi" => Ok(MlExpr::Pi),
        "e" => Ok(MlExpr::E),

        "distance" => Ok(MlExpr::Distance(take_arg(&mut args, 1, "distance")?)),

        "rgb" => Ok(MlExpr::Rgb(
            box_expr(take_arg(&mut args, 1, "rgb")?)?,
            box_expr(take_arg(&mut args, 2, "rgb")?)?,
            box_expr(take_arg(&mut args, 3, "rgb")?)?,
        )),
        "rgba" => Ok(MlExpr::Rgba(
            box_expr(take_arg(&mut args, 1, "rgba")?)?,
            box_expr(take_arg(&mut args, 2, "rgba")?)?,
            box_expr(take_arg(&mut args, 3, "rgba")?)?,
            box_expr(take_arg(&mut args, 4, "rgba")?)?,
        )),
        "to-rgba" => Ok(MlExpr::ToRgba(box_expr(take_arg(
            &mut args, 1, "to-rgba",
        )?)?)),

        "literal" => Ok(MlExpr::LiteralExpr(take_arg(&mut args, 1, "literal")?)),
        "typeof" => Ok(MlExpr::TypeOf(box_expr(take_arg(&mut args, 1, "typeof")?)?)),
        "to-string" => Ok(MlExpr::ToString(box_expr(take_arg(
            &mut args,
            1,
            "to-string",
        )?)?)),
        "to-number" => Ok(MlExpr::ToNumber(parse_variadic(args)?)),
        "to-boolean" => Ok(MlExpr::ToBoolean(box_expr(take_arg(
            &mut args,
            1,
            "to-boolean",
        )?)?)),
        "to-color" => Ok(MlExpr::ToColor(parse_variadic(args)?)),

        "number" => Ok(MlExpr::NumberAssertion(parse_variadic(args)?)),
        "string" => Ok(MlExpr::StringAssertion(parse_variadic(args)?)),
        "boolean" => Ok(MlExpr::BooleanAssertion(parse_variadic(args)?)),
        "object" => Ok(MlExpr::ObjectAssertion(parse_variadic(args)?)),

        "array" => {
            // ["array", value] | ["array", type, value] | ["array", type, len, value]
            match args.len() {
                1 => {
                    let value = box_expr(args.remove(0))?;
                    Ok(MlExpr::ArrayAssertion {
                        element_type: None,
                        length: None,
                        value,
                    })
                }
                2 => {
                    let element_type = match args.remove(0) {
                        Value::String(s) => Some(s),
                        _ => None,
                    };
                    let value = box_expr(args.remove(0))?;
                    Ok(MlExpr::ArrayAssertion {
                        element_type,
                        length: None,
                        value,
                    })
                }
                3 => {
                    let element_type = match args.remove(0) {
                        Value::String(s) => Some(s),
                        _ => None,
                    };
                    let length = args.remove(0).as_u64();
                    let value = box_expr(args.remove(0))?;
                    Ok(MlExpr::ArrayAssertion {
                        element_type,
                        length,
                        value,
                    })
                }
                n => Err(format!(
                    "expression \"array\": expected 1–3 arguments, got {n}"
                )),
            }
        }

        "collator" => Ok(MlExpr::Collator(take_arg(&mut args, 1, "collator")?)),

        "format" => {
            // ["format", input_1, style_1?, input_2, style_2?, ...]
            // style overrides are objects; inputs are expressions
            let mut sections = Vec::new();
            while !args.is_empty() {
                let content = box_expr(args.remove(0))?;
                let style = if args.first().map(|v| v.is_object()).unwrap_or(false) {
                    Some(args.remove(0))
                } else {
                    None
                };
                sections.push((content, style));
            }
            Ok(MlExpr::Format(sections))
        }

        "image" => Ok(MlExpr::Image(box_expr(take_arg(&mut args, 1, "image")?)?)),

        "number-format" => {
            let input = box_expr(take_arg(&mut args, 1, "number-format")?)?;
            let options = take_arg(&mut args, 2, "number-format")?;
            Ok(MlExpr::NumberFormat { input, options })
        }

        "feature-state" => Ok(MlExpr::FeatureState(box_expr(take_arg(
            &mut args,
            1,
            "feature-state",
        )?)?)),
        "geometry-type" => Ok(MlExpr::GeometryType),
        "id" => Ok(MlExpr::Id),
        "properties" => Ok(MlExpr::Properties),
        "accumulated" => Ok(MlExpr::Accumulated),
        "line-progress" => Ok(MlExpr::LineProgress),

        "zoom" => Ok(MlExpr::Zoom),

        "heatmap-density" => Ok(MlExpr::HeatmapDensity),

        "elevation" => Ok(MlExpr::Elevation),

        "upcase" => Ok(MlExpr::Upcase(box_expr(take_arg(&mut args, 1, "upcase")?)?)),
        "downcase" => Ok(MlExpr::Downcase(box_expr(take_arg(
            &mut args, 1, "downcase",
        )?)?)),
        "concat" => Ok(MlExpr::Concat(parse_variadic(args)?)),
        "is-supported-script" => Ok(MlExpr::IsSupportedScript(box_expr(take_arg(
            &mut args,
            1,
            "is-supported-script",
        )?)?)),
        "resolved-locale" => Ok(MlExpr::ResolvedLocale(box_expr(take_arg(
            &mut args,
            1,
            "resolved-locale",
        )?)?)),
        "split" => Ok(MlExpr::Split(
            box_expr(take_arg(&mut args, 1, "split")?)?,
            box_expr(take_arg(&mut args, 2, "split")?)?,
        )),
        "join" => Ok(MlExpr::Join(
            box_expr(take_arg(&mut args, 1, "join")?)?,
            box_expr(take_arg(&mut args, 2, "join")?)?,
        )),

        other => Ok(MlExpr::Unknown {
            operator: other.to_owned(),
            args,
        }),
    }
}

/// Parse pairs of `(input, output)` from a flat argument list, used by
/// `step` and the `interpolate` family.
fn parse_stop_pairs(args: &mut Vec<Value>, op: &str) -> Result<Vec<(f64, MlExpr)>, String> {
    if !args.len().is_multiple_of(2) {
        return Err(format!(
            "expression \"{op}\": stop arguments must come in (input, output) pairs, got {} args",
            args.len()
        ));
    }
    let mut stops = Vec::new();
    while args.len() >= 2 {
        let input_arg = args.remove(0);
        let Some(input) = input_arg.as_f64() else {
            return Err(format!(
                "stop input must be a number literal, got {input_arg:?}"
            ));
        };
        let output = expr_from_value(args.remove(0))?;
        stops.push((input, output));
    }
    Ok(stops)
}

/// Parse a variable-length list of expression arguments.
fn parse_variadic(args: Vec<Value>) -> Result<Vec<MlExpr>, String> {
    args.into_iter().map(expr_from_value).collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(v: serde_json::Value) -> MlExpr {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn interpolation_linear() {
        let i: Interpolation = serde_json::from_value(json!(["linear"])).unwrap();
        assert_eq!(i, Interpolation::Linear);
    }

    #[test]
    fn interpolation_exponential() {
        let i: Interpolation = serde_json::from_value(json!(["exponential", 1.5])).unwrap();
        assert_eq!(i, Interpolation::Exponential { base: 1.5 });
    }

    #[test]
    fn interpolation_cubic_bezier() {
        let i: Interpolation =
            serde_json::from_value(json!(["cubic-bezier", 0.0, 0.0, 1.0, 1.0])).unwrap();
        assert_eq!(
            i,
            Interpolation::CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0
            }
        );
    }

    #[test]
    fn literal_number_as_arg() {
        // Numbers appear as Literal args inside expressions (e.g. stop inputs)
        let e = parse(json!(["step", ["zoom"], 0, 5, 1]));
        match e {
            MlExpr::Step { default_output, .. } => {
                assert!(matches!(*default_output, MlExpr::Literal(Value::Number(_))));
            }
            other => panic!("expected Step, got {other:?}"),
        }
    }

    #[test]
    fn literal_string_as_arg() {
        // Strings appear as Literal args inside expressions (e.g. get property)
        let e = parse(json!(["get", "name"]));
        match e {
            MlExpr::Get { property, .. } => {
                assert_eq!(property, "name");
            }
            other => panic!("expected Get, got {other:?}"),
        }
    }

    #[test]
    fn literal_bool_as_arg() {
        // Booleans appear as Literal args inside expressions
        let e = parse(json!(["!", false]));
        assert!(
            matches!(e, MlExpr::Not(inner) if matches!(*inner, MlExpr::Literal(Value::Bool(_))))
        );
    }

    #[test]
    fn zoom() {
        assert_eq!(parse(json!(["zoom"])), MlExpr::Zoom);
    }

    #[test]
    fn geometry_type() {
        assert_eq!(parse(json!(["geometry-type"])), MlExpr::GeometryType);
    }

    #[test]
    fn id_expr() {
        assert_eq!(parse(json!(["id"])), MlExpr::Id);
    }

    #[test]
    fn heatmap_density() {
        assert_eq!(parse(json!(["heatmap-density"])), MlExpr::HeatmapDensity);
    }

    #[test]
    fn get_no_object() {
        let e = parse(json!(["get", "name"]));
        assert!(matches!(e, MlExpr::Get { object: None, .. }));
    }

    #[test]
    fn get_with_object() {
        let e = parse(json!(["get", "name", ["properties"]]));
        assert!(matches!(
            e,
            MlExpr::Get {
                object: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn has_expr() {
        let e = parse(json!(["has", "population"]));
        assert!(matches!(e, MlExpr::Has { object: None, .. }));
    }

    #[test]
    fn step_expr() {
        let e = parse(json!(["step", ["zoom"], 0.5, 12, 1.0, 15, 2.0]));
        match e {
            MlExpr::Step { stops, .. } => assert_eq!(stops.len(), 2),
            other => panic!("expected Step, got {other:?}"),
        }
    }

    #[test]
    fn interpolate_linear_zoom() {
        let e = parse(json!([
            "interpolate",
            ["linear"],
            ["zoom"],
            5,
            1.0,
            10,
            4.0
        ]));
        match e {
            MlExpr::Interpolate {
                interpolation,
                stops,
                ..
            } => {
                assert_eq!(interpolation, Interpolation::Linear);
                assert_eq!(stops.len(), 2);
            }
            other => panic!("expected Interpolate, got {other:?}"),
        }
    }

    #[test]
    fn interpolate_exponential() {
        let e = parse(json!([
            "interpolate",
            ["exponential", 1.4],
            ["zoom"],
            5,
            1.0,
            10,
            4.0
        ]));
        match e {
            MlExpr::Interpolate { interpolation, .. } => {
                assert_eq!(interpolation, Interpolation::Exponential { base: 1.4 });
            }
            other => panic!("expected Interpolate, got {other:?}"),
        }
    }

    #[test]
    fn interpolate_hcl() {
        let e = parse(json!([
            "interpolate-hcl",
            ["linear"],
            ["zoom"],
            0,
            "#f00",
            10,
            "#00f"
        ]));
        assert!(matches!(e, MlExpr::InterpolateHcl { .. }));
    }

    #[test]
    fn add_expr() {
        let e = parse(json!(["+", 1, 2, 3]));
        match e {
            MlExpr::Add(args) => assert_eq!(args.len(), 3),
            other => panic!("expected Add, got {other:?}"),
        }
    }

    #[test]
    fn sub_unary() {
        let e = parse(json!(["-", ["get", "val"]]));
        assert!(matches!(e, MlExpr::Sub(_, None)));
    }

    #[test]
    fn sub_binary() {
        let e = parse(json!(["-", 10, 3]));
        assert!(matches!(e, MlExpr::Sub(_, Some(_))));
    }

    #[test]
    fn math_constants() {
        assert_eq!(parse(json!(["ln2"])), MlExpr::Ln2);
        assert_eq!(parse(json!(["pi"])), MlExpr::Pi);
        assert_eq!(parse(json!(["e"])), MlExpr::E);
    }

    #[test]
    fn case_expr() {
        let e = parse(json!(["case", ["has", "name"], "yes", "no"]));
        match e {
            MlExpr::Case { branches, .. } => assert_eq!(branches.len(), 1),
            other => panic!("expected Case, got {other:?}"),
        }
    }

    #[test]
    fn match_expr_scalar_labels() {
        let e = parse(json!([
            "match",
            ["get", "class"],
            "residential",
            "#f00",
            "commercial",
            "#0f0",
            "#000"
        ]));
        match e {
            MlExpr::Match { branches, .. } => {
                assert_eq!(branches.len(), 2);
                assert_eq!(branches[0].0, json!("residential"));
            }
            other => panic!("expected Match, got {other:?}"),
        }
    }

    #[test]
    fn match_expr_array_labels() {
        let e = parse(json!(["match", ["get", "n"], [1, 2], "low", "other"]));
        match e {
            MlExpr::Match { branches, .. } => {
                assert_eq!(branches[0].0, json!([1, 2]));
            }
            other => panic!("expected Match, got {other:?}"),
        }
    }

    #[test]
    fn all_any_not() {
        assert!(matches!(parse(json!(["all", true, false])), MlExpr::All(_)));
        assert!(matches!(parse(json!(["any", true, false])), MlExpr::Any(_)));
        assert!(matches!(parse(json!(["!", true])), MlExpr::Not(_)));
    }

    #[test]
    fn comparison_ops() {
        assert!(matches!(parse(json!(["==", 1, 1])), MlExpr::Eq(_, _)));
        assert!(matches!(parse(json!(["!=", 1, 2])), MlExpr::Ne(_, _)));
        assert!(matches!(parse(json!([">", 2, 1])), MlExpr::Gt(_, _)));
        assert!(matches!(parse(json!([">=", 2, 2])), MlExpr::Gte(_, _)));
        assert!(matches!(parse(json!(["<", 1, 2])), MlExpr::Lt(_, _)));
        assert!(matches!(parse(json!(["<=", 1, 2])), MlExpr::Lte(_, _)));
    }

    #[test]
    fn literal_expr() {
        let e = parse(json!(["literal", [1, 2, 3]]));
        assert!(matches!(e, MlExpr::LiteralExpr(Value::Array(_))));
    }

    #[test]
    fn array_assertion_bare() {
        let e = parse(json!(["array", ["get", "coords"]]));
        assert!(matches!(
            e,
            MlExpr::ArrayAssertion {
                element_type: None,
                length: None,
                ..
            }
        ));
    }

    #[test]
    fn array_assertion_typed() {
        let e = parse(json!(["array", "number", ["get", "coords"]]));
        match e {
            MlExpr::ArrayAssertion {
                element_type,
                length,
                ..
            } => {
                assert_eq!(element_type.as_deref(), Some("number"));
                assert!(length.is_none());
            }
            other => panic!("expected ArrayAssertion, got {other:?}"),
        }
    }

    #[test]
    fn array_assertion_typed_and_length() {
        let e = parse(json!(["array", "string", 3, ["literal", ["a", "b", "c"]]]));
        match e {
            MlExpr::ArrayAssertion {
                element_type,
                length,
                ..
            } => {
                assert_eq!(element_type.as_deref(), Some("string"));
                assert_eq!(length, Some(3));
            }
            other => panic!("expected ArrayAssertion, got {other:?}"),
        }
    }

    #[test]
    fn concat_expr() {
        let e = parse(json!(["concat", "hello", " ", "world"]));
        match e {
            MlExpr::Concat(args) => assert_eq!(args.len(), 3),
            other => panic!("expected Concat, got {other:?}"),
        }
    }

    #[test]
    fn upcase_downcase() {
        assert!(matches!(parse(json!(["upcase", "hi"])), MlExpr::Upcase(_)));
        assert!(matches!(
            parse(json!(["downcase", "HI"])),
            MlExpr::Downcase(_)
        ));
    }

    #[test]
    fn let_var_binding() {
        let e = parse(json!(["let", "x", 42, ["var", "x"]]));
        match e {
            MlExpr::Let { bindings, body } => {
                assert_eq!(bindings.len(), 1);
                assert_eq!(bindings[0].0, "x");
                assert!(matches!(*body, MlExpr::Var(ref s) if s == "x"));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn unknown_operator() {
        let e = parse(json!(["future-operator", 1, 2]));
        match e {
            MlExpr::Unknown { operator, args } => {
                assert_eq!(operator, "future-operator");
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn nested_get_in_comparison() {
        let e = parse(json!([">=", ["get", "mag"], 4]));
        match e {
            MlExpr::Gte(left, right) => {
                assert!(matches!(*left, MlExpr::Get { .. }));
                assert!(matches!(*right, MlExpr::Literal(_)));
            }
            other => panic!("expected Gte, got {other:?}"),
        }
    }

    #[test]
    fn rgb_expr() {
        let e = parse(json!(["rgb", 255, 0, 128]));
        assert!(matches!(e, MlExpr::Rgb(_, _, _)));
    }

    #[test]
    fn feature_state_expr() {
        let e = parse(json!(["feature-state", "hover"]));
        assert!(matches!(e, MlExpr::FeatureState(_)));
    }

    #[test]
    fn format_expr() {
        let e = parse(json!(["format", ["get", "name"], {"font-scale": 0.8}, "\n", {}]));
        match e {
            MlExpr::Format(sections) => {
                assert_eq!(sections.len(), 2);
                assert!(sections[0].1.is_some()); // has style override
                assert!(sections[1].1.is_some()); // has empty style override
            }
            other => panic!("expected Format, got {other:?}"),
        }
    }
}
