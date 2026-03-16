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

use galileo::layer::vector_tile_layer::style::{PropertyFilter, PropertyFilterOperator};
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::layer::log_unsupported;

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
                    "linear" => Ok(Interpolation::Linear),
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
pub enum Expr {
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
        bindings: Vec<(String, Box<Expr>)>,
        /// The body expression evaluated in the binding scope.
        body: Box<Expr>,
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
        input: Box<Expr>,
        /// Output when the input is below the first stop.
        default_output: Box<Expr>,
        /// `(threshold, output)` pairs in ascending order.
        stops: Vec<(Expr, Expr)>,
    },

    /// `["interpolate", interp, input, stop_input_1, stop_output_1, ...]`
    ///
    /// Continuously interpolates between stop outputs.  Output type must be
    /// `number`, `array<number>`, `color`, `array<color>`, or `projection`.
    Interpolate {
        /// The interpolation curve.
        interpolation: Interpolation,
        /// The numeric input expression.
        input: Box<Expr>,
        /// `(threshold, output)` pairs in ascending order.
        stops: Vec<(Expr, Expr)>,
    },

    /// `["interpolate-hcl", interp, input, stop_input_1, stop_output_1, ...]`
    ///
    /// Like [`Interpolate`](Expr::Interpolate) but performed in the
    /// Hue-Chroma-Luminance color space.  Output must be `color`.
    InterpolateHcl {
        /// The interpolation curve.
        interpolation: Interpolation,
        /// The numeric input expression.
        input: Box<Expr>,
        /// `(threshold, output)` pairs in ascending order.
        stops: Vec<(Expr, Expr)>,
    },

    /// `["interpolate-lab", interp, input, stop_input_1, stop_output_1, ...]`
    ///
    /// Like [`Interpolate`](Expr::Interpolate) but performed in the CIELAB
    /// color space.  Output must be `color`.
    InterpolateLab {
        /// The interpolation curve.
        interpolation: Interpolation,
        /// The numeric input expression.
        input: Box<Expr>,
        /// `(threshold, output)` pairs in ascending order.
        stops: Vec<(Expr, Expr)>,
    },

    /// `["get", property]` or `["get", property, object_expr]`
    ///
    /// Retrieves a feature property (or a property from `object_expr`).
    Get {
        /// The property name expression (typically a string literal).
        property: Box<Expr>,
        /// Optional object to retrieve the property from.
        object: Option<Box<Expr>>,
    },

    /// `["has", property]` or `["has", property, object_expr]`
    ///
    /// Tests whether a feature property (or a property in `object_expr`)
    /// exists.
    Has {
        /// The property name expression.
        property: Box<Expr>,
        /// Optional object to test.
        object: Option<Box<Expr>>,
    },

    /// `["!has", property]` or `["!has", property, object_expr]`
    ///
    /// Tests whether a feature property (or a property in `object_expr`)
    /// does not exist.
    NotHas {
        /// The property name expression.
        property: Box<Expr>,
        /// Optional object to test.
        object: Option<Box<Expr>>,
    },

    /// `["at", index, array]`
    ///
    /// Retrieves the item at `index` from `array`.
    At {
        /// The zero-based index expression.
        index: Box<Expr>,
        /// The array expression.
        array: Box<Expr>,
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
        item: Box<Expr>,
        /// The haystack expression (array or string).
        array: Vec<Expr>,
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
        item: Box<Expr>,
        /// The haystack expression (array or string).
        array: Vec<Expr>,
    },

    /// `["index-of", item, array_or_string]` or with an optional `from_index`.
    ///
    /// Returns the first index at which `item` appears, or -1.
    IndexOf {
        /// The needle expression.
        item: Box<Expr>,
        /// The haystack expression.
        array_or_string: Box<Expr>,
        /// Optional start index for the search.
        from_index: Option<Box<Expr>>,
    },

    /// `["slice", array_or_string, start]` or `["slice", ..., start, end]`.
    ///
    /// Returns a subarray or substring.
    Slice {
        /// The source expression.
        array_or_string: Box<Expr>,
        /// The inclusive start index.
        start: Box<Expr>,
        /// The exclusive end index (optional).
        end: Option<Box<Expr>>,
    },

    /// `["length", array_or_string]`
    ///
    /// Returns the length of an array or string.
    Length(Box<Expr>),

    /// `["global-state", property_name]`
    ///
    /// Retrieves a property from global state set via platform APIs.
    GlobalState(Box<Expr>),

    /// `["case", cond_1, out_1, ..., cond_n, out_n, fallback]`
    ///
    /// Returns the output for the first condition that evaluates to `true`.
    Case {
        /// `(condition, output)` pairs evaluated in order.
        branches: Vec<(Box<Expr>, Box<Expr>)>,
        /// Output when no condition matches.
        fallback: Box<Expr>,
    },

    /// `["match", input, label_1, out_1, ..., label_n, out_n, fallback]`
    ///
    /// Returns the output for the first label that equals `input`.  A label
    /// may be a single value or an array of values.
    Match {
        /// The input expression.
        input: Box<Expr>,
        /// `(labels, output)` pairs; each label list contains one or more
        /// literal values.
        branches: Vec<(Vec<Value>, Box<Expr>)>,
        /// Output when no label matches.
        fallback: Box<Expr>,
    },

    /// `["coalesce", expr_1, ..., expr_n]`
    ///
    /// Evaluates each expression in order and returns the first non-null result.
    Coalesce(Vec<Expr>),

    /// `["all", input_1, ..., input_n]` — logical AND (short-circuit).
    All(Vec<Expr>),

    /// `["any", input_1, ..., input_n]` — logical OR (short-circuit).
    Any(Vec<Expr>),

    /// `["!", input]` — logical NOT.
    Not(Box<Expr>),

    /// `["==", a, b]` (with optional collator) — equality comparison.
    Eq(Box<Expr>, Box<Expr>),

    /// `["!=", a, b]` — inequality comparison.
    Ne(Box<Expr>, Box<Expr>),

    /// `[">", a, b]` — greater than.
    Gt(Box<Expr>, Box<Expr>),

    /// `[">=", a, b]` — greater than or equal.
    Gte(Box<Expr>, Box<Expr>),

    /// `["<", a, b]` — less than.
    Lt(Box<Expr>, Box<Expr>),

    /// `["<=", a, b]` — less than or equal.
    Lte(Box<Expr>, Box<Expr>),

    /// `["within", geojson]`
    ///
    /// Returns `true` if the feature is fully inside the given GeoJSON geometry.
    Within(Value),

    /// `["+", n_1, ..., n_n]` — sum.
    Add(Vec<Expr>),

    /// `["*", n_1, ..., n_n]` — product.
    Mul(Vec<Expr>),

    /// `["-", a, b]` or `["-", a]` — subtraction or negation.
    Sub(Box<Expr>, Option<Box<Expr>>),

    /// `["/", a, b]` — division.
    Div(Box<Expr>, Box<Expr>),

    /// `["%", a, b]` — modulo.
    Mod(Box<Expr>, Box<Expr>),

    /// `["^", base, exp]` — exponentiation.
    Pow(Box<Expr>, Box<Expr>),

    /// `["sqrt", n]` — square root.
    Sqrt(Box<Expr>),

    /// `["abs", n]` — absolute value.
    Abs(Box<Expr>),

    /// `["ceil", n]` — ceiling.
    Ceil(Box<Expr>),

    /// `["floor", n]` — floor.
    Floor(Box<Expr>),

    /// `["round", n]` — round to nearest integer (half away from zero).
    Round(Box<Expr>),

    /// `["min", n_1, ..., n_n]` — minimum.
    Min(Vec<Expr>),

    /// `["max", n_1, ..., n_n]` — maximum.
    Max(Vec<Expr>),

    /// `["log2", n]` — base-2 logarithm.
    Log2(Box<Expr>),

    /// `["log10", n]` — base-10 logarithm.
    Log10(Box<Expr>),

    /// `["ln", n]` — natural logarithm.
    Ln(Box<Expr>),

    /// `["sin", n]` — sine (radians).
    Sin(Box<Expr>),

    /// `["cos", n]` — cosine (radians).
    Cos(Box<Expr>),

    /// `["tan", n]` — tangent (radians).
    Tan(Box<Expr>),

    /// `["asin", n]` — arcsine.
    Asin(Box<Expr>),

    /// `["acos", n]` — arccosine.
    Acos(Box<Expr>),

    /// `["atan", n]` — arctangent.
    Atan(Box<Expr>),

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
    Rgb(Box<Expr>, Box<Expr>, Box<Expr>),

    /// `["rgba", r, g, b, a]` — constructs a color from RGBA components.
    Rgba(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),

    /// `["to-rgba", color]` — returns `[r, g, b, a]` components of a color.
    ToRgba(Box<Expr>),

    /// `["literal", value]` — wraps a JSON array or object as a literal.
    LiteralExpr(Value),

    /// `["typeof", value]` — returns a string describing the type of `value`.
    TypeOf(Box<Expr>),

    /// `["to-string", value]` — converts `value` to a string.
    ToString(Box<Expr>),

    /// `["to-number", value_1, ..., value_n]` — converts to a number.
    ToNumber(Vec<Expr>),

    /// `["to-boolean", value]` — converts to a boolean.
    ToBoolean(Box<Expr>),

    /// `["to-color", value_1, ..., value_n]` — converts to a color.
    ToColor(Vec<Expr>),

    /// `["number", value_1, ..., value_n]` — asserts input is a number.
    NumberAssertion(Vec<Expr>),

    /// `["string", value_1, ..., value_n]` — asserts input is a string.
    StringAssertion(Vec<Expr>),

    /// `["boolean", value_1, ..., value_n]` — asserts input is a boolean.
    BooleanAssertion(Vec<Expr>),

    /// `["object", value_1, ..., value_n]` — asserts input is an object.
    ObjectAssertion(Vec<Expr>),

    /// `["array", value]` / `["array", type, value]` / `["array", type, length, value]`
    ///
    /// Asserts input is an array, optionally with element type and length.
    ArrayAssertion {
        /// Optional element type assertion (`"string"`, `"number"`, `"boolean"`).
        element_type: Option<String>,
        /// Optional expected array length.
        length: Option<u64>,
        /// The value to assert.
        value: Box<Expr>,
    },

    /// `["collator", options]` — returns a locale-aware collator.
    Collator(Value),

    /// `["format", input_1, style_1?, ..., input_n, style_n?]`
    ///
    /// Returns a `formatted` string for rich text labels.  Each section is a
    /// `(content, style_overrides)` pair.
    Format(Vec<(Box<Expr>, Option<Value>)>),

    /// `["image", name]` — returns an image for use in icons/patterns.
    Image(Box<Expr>),

    /// `["number-format", input, options]` — formats a number as a string.
    NumberFormat {
        /// The number expression to format.
        input: Box<Expr>,
        /// Formatting options object (locale, currency, min/max fraction digits, etc.).
        options: Value,
    },

    /// `["get", ...]` — see [`Expr::Get`] (aliased here for feature state).
    ///
    /// `["feature-state", property]` — retrieves a property from feature state.
    FeatureState(Box<Expr>),

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
    Upcase(Box<Expr>),

    /// `["downcase", s]` — converts string to lowercase.
    Downcase(Box<Expr>),

    /// `["concat", s_1, ..., s_n]` — concatenates strings.
    Concat(Vec<Expr>),

    /// `["is-supported-script", s]` — returns `true` if the string renders legibly.
    IsSupportedScript(Box<Expr>),

    /// `["resolved-locale", collator]` — returns the IETF tag of the locale in use.
    ResolvedLocale(Box<Expr>),

    /// `["split", input, separator]` — splits a string into an array.
    Split(Box<Expr>, Box<Expr>),

    /// `["join", array, separator]` — joins an array into a string.
    Join(Box<Expr>, Box<Expr>),

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

impl Expr {
    /// Returns the operator name for this expression, or `None` for
    /// [`Expr::Literal`] and [`Expr::LiteralExpr`].
    ///
    /// Useful for quick operator-based dispatch in the renderer.
    pub fn operator(&self) -> Option<&str> {
        Some(match self {
            // Variable binding
            Expr::Let { .. } => "let",
            Expr::Var(_) => "var",
            // Ramps / curves
            Expr::Step { .. } => "step",
            Expr::Interpolate { .. } => "interpolate",
            Expr::InterpolateHcl { .. } => "interpolate-hcl",
            Expr::InterpolateLab { .. } => "interpolate-lab",
            // Lookup
            Expr::Get { .. } => "get",
            Expr::Has { .. } => "has",
            Expr::NotHas { .. } => "!has",
            Expr::At { .. } => "at",
            Expr::In { .. } => "in",
            Expr::NotIn { .. } => "!in",
            Expr::IndexOf { .. } => "index-of",
            Expr::Slice { .. } => "slice",
            Expr::Length(_) => "length",
            Expr::GlobalState(_) => "global-state",
            // Decision
            Expr::Case { .. } => "case",
            Expr::Match { .. } => "match",
            Expr::Coalesce(_) => "coalesce",
            Expr::All(_) => "all",
            Expr::Any(_) => "any",
            Expr::Not(_) => "!",
            Expr::Eq(_, _) => "==",
            Expr::Ne(_, _) => "!=",
            Expr::Gt(_, _) => ">",
            Expr::Gte(_, _) => ">=",
            Expr::Lt(_, _) => "<",
            Expr::Lte(_, _) => "<=",
            Expr::Within(_) => "within",
            // Math
            Expr::Add(_) => "+",
            Expr::Mul(_) => "*",
            Expr::Sub(_, _) => "-",
            Expr::Div(_, _) => "/",
            Expr::Mod(_, _) => "%",
            Expr::Pow(_, _) => "^",
            Expr::Sqrt(_) => "sqrt",
            Expr::Abs(_) => "abs",
            Expr::Ceil(_) => "ceil",
            Expr::Floor(_) => "floor",
            Expr::Round(_) => "round",
            Expr::Min(_) => "min",
            Expr::Max(_) => "max",
            Expr::Log2(_) => "log2",
            Expr::Log10(_) => "log10",
            Expr::Ln(_) => "ln",
            Expr::Sin(_) => "sin",
            Expr::Cos(_) => "cos",
            Expr::Tan(_) => "tan",
            Expr::Asin(_) => "asin",
            Expr::Acos(_) => "acos",
            Expr::Atan(_) => "atan",
            Expr::Ln2 => "ln2",
            Expr::Pi => "pi",
            Expr::E => "e",
            Expr::Distance(_) => "distance",
            // Color
            Expr::Rgb(_, _, _) => "rgb",
            Expr::Rgba(_, _, _, _) => "rgba",
            Expr::ToRgba(_) => "to-rgba",
            // Type operators
            Expr::LiteralExpr(_) => "literal",
            Expr::TypeOf(_) => "typeof",
            Expr::ToString(_) => "to-string",
            Expr::ToNumber(_) => "to-number",
            Expr::ToBoolean(_) => "to-boolean",
            Expr::ToColor(_) => "to-color",
            Expr::NumberAssertion(_) => "number",
            Expr::StringAssertion(_) => "string",
            Expr::BooleanAssertion(_) => "boolean",
            Expr::ObjectAssertion(_) => "object",
            Expr::ArrayAssertion { .. } => "array",
            Expr::Collator(_) => "collator",
            Expr::Format(_) => "format",
            Expr::Image(_) => "image",
            Expr::NumberFormat { .. } => "number-format",
            // Feature data
            Expr::FeatureState(_) => "feature-state",
            Expr::GeometryType => "geometry-type",
            Expr::Id => "id",
            Expr::Properties => "properties",
            Expr::Accumulated => "accumulated",
            Expr::LineProgress => "line-progress",
            // Camera
            Expr::Zoom => "zoom",
            // Heatmap
            Expr::HeatmapDensity => "heatmap-density",
            // Color relief
            Expr::Elevation => "elevation",
            // String
            Expr::Upcase(_) => "upcase",
            Expr::Downcase(_) => "downcase",
            Expr::Concat(_) => "concat",
            Expr::IsSupportedScript(_) => "is-supported-script",
            Expr::ResolvedLocale(_) => "resolved-locale",
            Expr::Split(_, _) => "split",
            Expr::Join(_, _) => "join",
            // Unknown
            Expr::Unknown { operator, .. } => operator.as_str(),
            // No operator for bare literals
            Expr::Literal(_) => return None,
        })
    }

    pub fn to_prop_filters(&self) -> Vec<PropertyFilter> {
        type PF = PropertyFilterOperator;

        fn get_prop(prop: &Expr) -> Option<String> {
            if let Expr::Literal(Value::String(property_name)) = prop {
                if property_name == "$type" {
                    // TODO: this is supposed to check geometry type of the layer,
                    // but in Galileo this is done by the symbol. Need to check if
                    // it is possible in Maplibre to draw a layer with wrong geometry
                    // type. If so, do we need to do anything for it?
                    return None;
                }

                Some(property_name.clone())
            } else {
                log_unsupported!(format!("'{self:?}' expression"));
                None
            }
        }

        fn get_val(value: &Expr) -> Option<String> {
            match value {
                Expr::Literal(Value::String(s)) => Some(s.clone()),
                Expr::Literal(Value::Number(v)) => Some(v.to_string()),
                Expr::Literal(Value::Bool(v)) => Some(v.to_string()),
                Expr::Literal(Value::Null) => Some("null".to_string()),
                _ => {
                    log_unsupported!(format!("'{self:?}' expression"));
                    None
                }
            }
        }

        fn op(prop: &Expr, value: &Expr, f: impl FnOnce(String) -> PF) -> Vec<PropertyFilter> {
            let Some(property_name) = get_prop(prop) else {
                return Vec::new();
            };

            let Some(val) = get_val(value) else {
                return Vec::new();
            };

            vec![PropertyFilter {
                property_name: property_name.to_owned(),
                operator: f(val),
            }]
        }

        fn ex(prop: &Expr, obj: &Option<Box<Expr>>, operator: PF) -> Vec<PropertyFilter> {
            if obj.is_some() {
                log_unsupported!(format!("'{self:?}' expression"));
            }

            let Some(property_name) = get_prop(prop) else {
                return Vec::new();
            };

            vec![PropertyFilter {
                property_name,
                operator,
            }]
        }

        fn con(
            prop: &Expr,
            vals: &[Expr],
            f: impl FnOnce(Vec<String>) -> PF,
        ) -> Vec<PropertyFilter> {
            let Some(property_name) = get_prop(prop) else {
                return Vec::new();
            };

            let mut values = vec![];
            for val in vals {
                let Some(val) = get_val(val) else {
                    return Vec::new();
                };

                values.push(val);
            }

            vec![PropertyFilter {
                property_name: property_name.to_owned(),
                operator: f(values),
            }]
        }

        match self {
            Expr::All(parts) => parts.iter().flat_map(Self::to_prop_filters).collect(),
            Expr::Eq(prop, value) => op(prop, value, PF::Equal),
            Expr::Ne(prop, value) => op(prop, value, PF::NotEqual),
            Expr::Gt(prop, value) => op(prop, value, PF::GreaterThan),
            Expr::Gte(prop, value) => op(prop, value, PF::GreaterThanOrEqual),
            Expr::Lt(prop, value) => op(prop, value, PF::LessThan),
            Expr::Lte(prop, value) => op(prop, value, PF::LessThanOrEqual),
            Expr::Has { property, object } => ex(property, object, PF::Exist),
            Expr::NotHas { property, object } => ex(property, object, PF::NotExist),
            Expr::In { item, array } => con(item, array, PF::OneOf),
            Expr::NotIn { item, array } => con(item, array, PF::OneOf),
            _ => {
                log_unsupported!(format!("'{self:?}' expression"));
                Vec::new()
            }
        }
    }
}

impl Serialize for Expr {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // Convert to a serde_json::Value first, then serialize that.
        // This lets us produce the exact JSON array format `["op", arg1, ...]`.
        let v = expr_to_value(self);
        v.serialize(s)
    }
}

/// Convert an [`Expr`] to a [`Value`] that matches the MapLibre JSON array
/// format for expressions.
fn expr_to_value(e: &Expr) -> Value {
    match e {
        Expr::Literal(v) => v.clone(),
        Expr::LiteralExpr(v) => Value::Array(vec![Value::String("literal".into()), v.clone()]),

        // Variable binding
        Expr::Let { bindings, body } => {
            let mut arr = vec![Value::String("let".into())];
            for (name, val) in bindings {
                arr.push(Value::String(name.clone()));
                arr.push(expr_to_value(val));
            }
            arr.push(expr_to_value(body));
            Value::Array(arr)
        }
        Expr::Var(name) => Value::Array(vec![
            Value::String("var".into()),
            Value::String(name.clone()),
        ]),

        // Ramps / curves
        Expr::Step {
            input,
            default_output,
            stops,
        } => {
            let mut arr = vec![
                Value::String("step".into()),
                expr_to_value(input),
                expr_to_value(default_output),
            ];
            for (inp, out) in stops {
                arr.push(expr_to_value(inp));
                arr.push(expr_to_value(out));
            }
            Value::Array(arr)
        }
        Expr::Interpolate {
            interpolation,
            input,
            stops,
        } => interp_to_array("interpolate", interpolation, input, stops),
        Expr::InterpolateHcl {
            interpolation,
            input,
            stops,
        } => interp_to_array("interpolate-hcl", interpolation, input, stops),
        Expr::InterpolateLab {
            interpolation,
            input,
            stops,
        } => interp_to_array("interpolate-lab", interpolation, input, stops),

        // Lookup
        Expr::Get { property, object } => opt_object_array("get", property, object.as_deref()),
        Expr::Has { property, object } => opt_object_array("has", property, object.as_deref()),
        Expr::NotHas { property, object } => opt_object_array("!has", property, object.as_deref()),
        Expr::At { index, array } => Value::Array(vec![
            Value::String("at".into()),
            expr_to_value(index),
            expr_to_value(array),
        ]),
        Expr::In { item, array } => {
            let mut arr = vec![Value::String("in".into()), expr_to_value(item)];
            for val in array {
                arr.push(expr_to_value(val));
            }

            Value::Array(arr)
        }
        Expr::NotIn { item, array } => {
            let mut arr = vec![Value::String("!in".into()), expr_to_value(item)];
            for val in array {
                arr.push(expr_to_value(val));
            }

            Value::Array(arr)
        }
        Expr::IndexOf {
            item,
            array_or_string,
            from_index,
        } => {
            let mut arr = vec![
                Value::String("index-of".into()),
                expr_to_value(item),
                expr_to_value(array_or_string),
            ];
            if let Some(fi) = from_index {
                arr.push(expr_to_value(fi));
            }
            Value::Array(arr)
        }
        Expr::Slice {
            array_or_string,
            start,
            end,
        } => {
            let mut arr = vec![
                Value::String("slice".into()),
                expr_to_value(array_or_string),
                expr_to_value(start),
            ];
            if let Some(e) = end {
                arr.push(expr_to_value(e));
            }
            Value::Array(arr)
        }
        Expr::Length(inner) => {
            Value::Array(vec![Value::String("length".into()), expr_to_value(inner)])
        }
        Expr::GlobalState(inner) => Value::Array(vec![
            Value::String("global-state".into()),
            expr_to_value(inner),
        ]),

        // Decision
        Expr::Case { branches, fallback } => {
            let mut arr = vec![Value::String("case".into())];
            for (cond, out) in branches {
                arr.push(expr_to_value(cond));
                arr.push(expr_to_value(out));
            }
            arr.push(expr_to_value(fallback));
            Value::Array(arr)
        }
        Expr::Match {
            input,
            branches,
            fallback,
        } => {
            let mut arr = vec![Value::String("match".into()), expr_to_value(input)];
            for (labels, out) in branches {
                if labels.len() == 1 {
                    arr.push(labels[0].clone());
                } else {
                    arr.push(Value::Array(labels.clone()));
                }
                arr.push(expr_to_value(out));
            }
            arr.push(expr_to_value(fallback));
            Value::Array(arr)
        }
        Expr::Coalesce(args) => variadic_array("coalesce", args),
        Expr::All(args) => variadic_array("all", args),
        Expr::Any(args) => variadic_array("any", args),
        Expr::Not(inner) => Value::Array(vec![Value::String("!".into()), expr_to_value(inner)]),
        Expr::Eq(a, b) => binary_array("==", a, b),
        Expr::Ne(a, b) => binary_array("!=", a, b),
        Expr::Gt(a, b) => binary_array(">", a, b),
        Expr::Gte(a, b) => binary_array(">=", a, b),
        Expr::Lt(a, b) => binary_array("<", a, b),
        Expr::Lte(a, b) => binary_array("<=", a, b),
        Expr::Within(v) => Value::Array(vec![Value::String("within".into()), v.clone()]),

        // Math
        Expr::Add(args) => variadic_array("+", args),
        Expr::Mul(args) => variadic_array("*", args),
        Expr::Sub(a, None) => Value::Array(vec![Value::String("-".into()), expr_to_value(a)]),
        Expr::Sub(a, Some(b)) => binary_array("-", a, b),
        Expr::Div(a, b) => binary_array("/", a, b),
        Expr::Mod(a, b) => binary_array("%", a, b),
        Expr::Pow(a, b) => binary_array("^", a, b),
        Expr::Sqrt(inner) => unary_array("sqrt", inner),
        Expr::Abs(inner) => unary_array("abs", inner),
        Expr::Ceil(inner) => unary_array("ceil", inner),
        Expr::Floor(inner) => unary_array("floor", inner),
        Expr::Round(inner) => unary_array("round", inner),
        Expr::Min(args) => variadic_array("min", args),
        Expr::Max(args) => variadic_array("max", args),
        Expr::Log2(inner) => unary_array("log2", inner),
        Expr::Log10(inner) => unary_array("log10", inner),
        Expr::Ln(inner) => unary_array("ln", inner),
        Expr::Sin(inner) => unary_array("sin", inner),
        Expr::Cos(inner) => unary_array("cos", inner),
        Expr::Tan(inner) => unary_array("tan", inner),
        Expr::Asin(inner) => unary_array("asin", inner),
        Expr::Acos(inner) => unary_array("acos", inner),
        Expr::Atan(inner) => unary_array("atan", inner),
        Expr::Ln2 => Value::Array(vec![Value::String("ln2".into())]),
        Expr::Pi => Value::Array(vec![Value::String("pi".into())]),
        Expr::E => Value::Array(vec![Value::String("e".into())]),
        Expr::Distance(v) => Value::Array(vec![Value::String("distance".into()), v.clone()]),

        // Color
        Expr::Rgb(r, g, b) => Value::Array(vec![
            Value::String("rgb".into()),
            expr_to_value(r),
            expr_to_value(g),
            expr_to_value(b),
        ]),
        Expr::Rgba(r, g, b, a) => Value::Array(vec![
            Value::String("rgba".into()),
            expr_to_value(r),
            expr_to_value(g),
            expr_to_value(b),
            expr_to_value(a),
        ]),
        Expr::ToRgba(inner) => unary_array("to-rgba", inner),

        // Type operators
        Expr::TypeOf(inner) => unary_array("typeof", inner),
        Expr::ToString(inner) => unary_array("to-string", inner),
        Expr::ToNumber(args) => variadic_array("to-number", args),
        Expr::ToBoolean(inner) => unary_array("to-boolean", inner),
        Expr::ToColor(args) => variadic_array("to-color", args),
        Expr::NumberAssertion(args) => variadic_array("number", args),
        Expr::StringAssertion(args) => variadic_array("string", args),
        Expr::BooleanAssertion(args) => variadic_array("boolean", args),
        Expr::ObjectAssertion(args) => variadic_array("object", args),
        Expr::ArrayAssertion {
            element_type,
            length,
            value,
        } => {
            let mut arr = vec![Value::String("array".into())];
            if let Some(et) = element_type {
                arr.push(Value::String(et.clone()));
                if let Some(len) = length {
                    arr.push(Value::Number((*len).into()));
                }
            }
            arr.push(expr_to_value(value));
            Value::Array(arr)
        }
        Expr::Collator(opts) => Value::Array(vec![Value::String("collator".into()), opts.clone()]),
        Expr::Format(sections) => {
            let mut arr = vec![Value::String("format".into())];
            for (content, style) in sections {
                arr.push(expr_to_value(content));
                if let Some(s) = style {
                    arr.push(s.clone());
                }
            }
            Value::Array(arr)
        }
        Expr::Image(inner) => unary_array("image", inner),
        Expr::NumberFormat { input, options } => Value::Array(vec![
            Value::String("number-format".into()),
            expr_to_value(input),
            options.clone(),
        ]),

        // Feature data
        Expr::FeatureState(inner) => unary_array("feature-state", inner),
        Expr::GeometryType => Value::Array(vec![Value::String("geometry-type".into())]),
        Expr::Id => Value::Array(vec![Value::String("id".into())]),
        Expr::Properties => Value::Array(vec![Value::String("properties".into())]),
        Expr::Accumulated => Value::Array(vec![Value::String("accumulated".into())]),
        Expr::LineProgress => Value::Array(vec![Value::String("line-progress".into())]),

        // Camera
        Expr::Zoom => Value::Array(vec![Value::String("zoom".into())]),

        // Heatmap
        Expr::HeatmapDensity => Value::Array(vec![Value::String("heatmap-density".into())]),

        // Color relief
        Expr::Elevation => Value::Array(vec![Value::String("elevation".into())]),

        // String
        Expr::Upcase(inner) => unary_array("upcase", inner),
        Expr::Downcase(inner) => unary_array("downcase", inner),
        Expr::Concat(args) => variadic_array("concat", args),
        Expr::IsSupportedScript(inner) => unary_array("is-supported-script", inner),
        Expr::ResolvedLocale(inner) => unary_array("resolved-locale", inner),
        Expr::Split(a, b) => binary_array("split", a, b),
        Expr::Join(a, b) => binary_array("join", a, b),

        // Unknown
        Expr::Unknown { operator, args } => {
            let mut arr = vec![Value::String(operator.clone())];
            arr.extend(args.iter().cloned());
            Value::Array(arr)
        }
    }
}

fn unary_array(op: &str, inner: &Expr) -> Value {
    Value::Array(vec![Value::String(op.into()), expr_to_value(inner)])
}

fn binary_array(op: &str, a: &Expr, b: &Expr) -> Value {
    Value::Array(vec![
        Value::String(op.into()),
        expr_to_value(a),
        expr_to_value(b),
    ])
}

fn variadic_array(op: &str, args: &[Expr]) -> Value {
    let mut arr = vec![Value::String(op.into())];
    for a in args {
        arr.push(expr_to_value(a));
    }
    Value::Array(arr)
}

fn opt_object_array(op: &str, property: &Expr, object: Option<&Expr>) -> Value {
    let mut arr = vec![Value::String(op.into()), expr_to_value(property)];
    if let Some(obj) = object {
        arr.push(expr_to_value(obj));
    }
    Value::Array(arr)
}

fn interp_to_array(
    op: &str,
    interpolation: &Interpolation,
    input: &Expr,
    stops: &[(Expr, Expr)],
) -> Value {
    let mut arr = vec![
        Value::String(op.into()),
        serde_json::to_value(interpolation).unwrap_or(Value::Null),
        expr_to_value(input),
    ];
    for (inp, out) in stops {
        arr.push(expr_to_value(inp));
        arr.push(expr_to_value(out));
    }
    Value::Array(arr)
}

impl<'de> Deserialize<'de> for Expr {
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
pub(crate) fn expr_from_value(v: Value) -> Result<Expr, String> {
    match v {
        Value::Array(arr) => parse_expr_array(arr),
        other => Ok(Expr::Literal(other)),
    }
}

fn box_expr(v: Value) -> Result<Box<Expr>, String> {
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
fn parse_expr_array(mut arr: Vec<Value>) -> Result<Expr, String> {
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
            return Ok(Expr::Literal(Value::Array(full)));
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
            Ok(Expr::Let { bindings, body })
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
            Ok(Expr::Var(name))
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
            Ok(Expr::Step {
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
                "interpolate" => Ok(Expr::Interpolate {
                    interpolation,
                    input,
                    stops,
                }),
                "interpolate-hcl" => Ok(Expr::InterpolateHcl {
                    interpolation,
                    input,
                    stops,
                }),
                _ => Ok(Expr::InterpolateLab {
                    interpolation,
                    input,
                    stops,
                }),
            }
        }

        "get" => {
            let property = box_expr(take_arg(&mut args, 1, "get")?)?;
            let object = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(Expr::Get { property, object })
        }

        "has" => {
            let property = box_expr(take_arg(&mut args, 1, "has")?)?;
            let object = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(Expr::Has { property, object })
        }

        "!has" => {
            let property = box_expr(take_arg(&mut args, 1, "!has")?)?;
            let object = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(Expr::NotHas { property, object })
        }

        "at" => {
            let index = box_expr(take_arg(&mut args, 1, "at")?)?;
            let array = box_expr(take_arg(&mut args, 2, "at")?)?;
            Ok(Expr::At { index, array })
        }

        "in" => {
            let item = box_expr(take_arg(&mut args, 1, "!in")?)?;
            let mut vals = vec![];
            let mut index = 2;
            while !args.is_empty() {
                vals.push(expr_from_value(take_arg(&mut args, index, "in")?)?);
                index += 1;
            }

            Ok(Expr::In { item, array: vals })
        }

        "!in" => {
            let item = box_expr(take_arg(&mut args, 1, "!in")?)?;
            let mut vals = vec![];
            let mut index = 2;
            while !args.is_empty() {
                vals.push(expr_from_value(take_arg(&mut args, index, "in")?)?);
                index += 1;
            }

            Ok(Expr::NotIn { item, array: vals })
        }

        "index-of" => {
            let item = box_expr(take_arg(&mut args, 1, "index-of")?)?;
            let array_or_string = box_expr(take_arg(&mut args, 2, "index-of")?)?;
            let from_index = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(Expr::IndexOf {
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
            Ok(Expr::Slice {
                array_or_string,
                start,
                end,
            })
        }

        "length" => Ok(Expr::Length(box_expr(take_arg(&mut args, 1, "length")?)?)),

        "global-state" => Ok(Expr::GlobalState(box_expr(take_arg(
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
            Ok(Expr::Case { branches, fallback })
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
                let label_val = args.remove(0);
                let labels = match label_val {
                    Value::Array(arr) => arr,
                    scalar => vec![scalar],
                };
                let out = box_expr(args.remove(0))?;
                branches.push((labels, out));
            }
            Ok(Expr::Match {
                input,
                branches,
                fallback,
            })
        }

        "coalesce" => Ok(Expr::Coalesce(parse_variadic(args)?)),
        "all" => Ok(Expr::All(parse_variadic(args)?)),
        "any" => Ok(Expr::Any(parse_variadic(args)?)),

        "!" => Ok(Expr::Not(box_expr(take_arg(&mut args, 1, "!")?)?)),

        "==" => Ok(Expr::Eq(
            box_expr(take_arg(&mut args, 1, "==")?)?,
            box_expr(take_arg(&mut args, 2, "==")?)?,
        )),
        "!=" => Ok(Expr::Ne(
            box_expr(take_arg(&mut args, 1, "!=")?)?,
            box_expr(take_arg(&mut args, 2, "!=")?)?,
        )),
        ">" => Ok(Expr::Gt(
            box_expr(take_arg(&mut args, 1, ">")?)?,
            box_expr(take_arg(&mut args, 2, ">")?)?,
        )),
        ">=" => Ok(Expr::Gte(
            box_expr(take_arg(&mut args, 1, ">=")?)?,
            box_expr(take_arg(&mut args, 2, ">=")?)?,
        )),
        "<" => Ok(Expr::Lt(
            box_expr(take_arg(&mut args, 1, "<")?)?,
            box_expr(take_arg(&mut args, 2, "<")?)?,
        )),
        "<=" => Ok(Expr::Lte(
            box_expr(take_arg(&mut args, 1, "<=")?)?,
            box_expr(take_arg(&mut args, 2, "<=")?)?,
        )),

        "within" => Ok(Expr::Within(take_arg(&mut args, 1, "within")?)),

        "+" => Ok(Expr::Add(parse_variadic(args)?)),
        "*" => Ok(Expr::Mul(parse_variadic(args)?)),

        "-" => {
            let a = box_expr(take_arg(&mut args, 1, "-")?)?;
            let b = if args.is_empty() {
                None
            } else {
                Some(box_expr(args.remove(0))?)
            };
            Ok(Expr::Sub(a, b))
        }

        "/" => Ok(Expr::Div(
            box_expr(take_arg(&mut args, 1, "/")?)?,
            box_expr(take_arg(&mut args, 2, "/")?)?,
        )),
        "%" => Ok(Expr::Mod(
            box_expr(take_arg(&mut args, 1, "%")?)?,
            box_expr(take_arg(&mut args, 2, "%")?)?,
        )),
        "^" => Ok(Expr::Pow(
            box_expr(take_arg(&mut args, 1, "^")?)?,
            box_expr(take_arg(&mut args, 2, "^")?)?,
        )),

        "sqrt" => Ok(Expr::Sqrt(box_expr(take_arg(&mut args, 1, "sqrt")?)?)),
        "abs" => Ok(Expr::Abs(box_expr(take_arg(&mut args, 1, "abs")?)?)),
        "ceil" => Ok(Expr::Ceil(box_expr(take_arg(&mut args, 1, "ceil")?)?)),
        "floor" => Ok(Expr::Floor(box_expr(take_arg(&mut args, 1, "floor")?)?)),
        "round" => Ok(Expr::Round(box_expr(take_arg(&mut args, 1, "round")?)?)),

        "min" => Ok(Expr::Min(parse_variadic(args)?)),
        "max" => Ok(Expr::Max(parse_variadic(args)?)),

        "log2" => Ok(Expr::Log2(box_expr(take_arg(&mut args, 1, "log2")?)?)),
        "log10" => Ok(Expr::Log10(box_expr(take_arg(&mut args, 1, "log10")?)?)),
        "ln" => Ok(Expr::Ln(box_expr(take_arg(&mut args, 1, "ln")?)?)),
        "sin" => Ok(Expr::Sin(box_expr(take_arg(&mut args, 1, "sin")?)?)),
        "cos" => Ok(Expr::Cos(box_expr(take_arg(&mut args, 1, "cos")?)?)),
        "tan" => Ok(Expr::Tan(box_expr(take_arg(&mut args, 1, "tan")?)?)),
        "asin" => Ok(Expr::Asin(box_expr(take_arg(&mut args, 1, "asin")?)?)),
        "acos" => Ok(Expr::Acos(box_expr(take_arg(&mut args, 1, "acos")?)?)),
        "atan" => Ok(Expr::Atan(box_expr(take_arg(&mut args, 1, "atan")?)?)),

        "ln2" => Ok(Expr::Ln2),
        "pi" => Ok(Expr::Pi),
        "e" => Ok(Expr::E),

        "distance" => Ok(Expr::Distance(take_arg(&mut args, 1, "distance")?)),

        "rgb" => Ok(Expr::Rgb(
            box_expr(take_arg(&mut args, 1, "rgb")?)?,
            box_expr(take_arg(&mut args, 2, "rgb")?)?,
            box_expr(take_arg(&mut args, 3, "rgb")?)?,
        )),
        "rgba" => Ok(Expr::Rgba(
            box_expr(take_arg(&mut args, 1, "rgba")?)?,
            box_expr(take_arg(&mut args, 2, "rgba")?)?,
            box_expr(take_arg(&mut args, 3, "rgba")?)?,
            box_expr(take_arg(&mut args, 4, "rgba")?)?,
        )),
        "to-rgba" => Ok(Expr::ToRgba(box_expr(take_arg(&mut args, 1, "to-rgba")?)?)),

        "literal" => Ok(Expr::LiteralExpr(take_arg(&mut args, 1, "literal")?)),
        "typeof" => Ok(Expr::TypeOf(box_expr(take_arg(&mut args, 1, "typeof")?)?)),
        "to-string" => Ok(Expr::ToString(box_expr(take_arg(
            &mut args,
            1,
            "to-string",
        )?)?)),
        "to-number" => Ok(Expr::ToNumber(parse_variadic(args)?)),
        "to-boolean" => Ok(Expr::ToBoolean(box_expr(take_arg(
            &mut args,
            1,
            "to-boolean",
        )?)?)),
        "to-color" => Ok(Expr::ToColor(parse_variadic(args)?)),

        "number" => Ok(Expr::NumberAssertion(parse_variadic(args)?)),
        "string" => Ok(Expr::StringAssertion(parse_variadic(args)?)),
        "boolean" => Ok(Expr::BooleanAssertion(parse_variadic(args)?)),
        "object" => Ok(Expr::ObjectAssertion(parse_variadic(args)?)),

        "array" => {
            // ["array", value] | ["array", type, value] | ["array", type, len, value]
            match args.len() {
                1 => {
                    let value = box_expr(args.remove(0))?;
                    Ok(Expr::ArrayAssertion {
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
                    Ok(Expr::ArrayAssertion {
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
                    Ok(Expr::ArrayAssertion {
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

        "collator" => Ok(Expr::Collator(take_arg(&mut args, 1, "collator")?)),

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
            Ok(Expr::Format(sections))
        }

        "image" => Ok(Expr::Image(box_expr(take_arg(&mut args, 1, "image")?)?)),

        "number-format" => {
            let input = box_expr(take_arg(&mut args, 1, "number-format")?)?;
            let options = take_arg(&mut args, 2, "number-format")?;
            Ok(Expr::NumberFormat { input, options })
        }

        "feature-state" => Ok(Expr::FeatureState(box_expr(take_arg(
            &mut args,
            1,
            "feature-state",
        )?)?)),
        "geometry-type" => Ok(Expr::GeometryType),
        "id" => Ok(Expr::Id),
        "properties" => Ok(Expr::Properties),
        "accumulated" => Ok(Expr::Accumulated),
        "line-progress" => Ok(Expr::LineProgress),

        "zoom" => Ok(Expr::Zoom),

        "heatmap-density" => Ok(Expr::HeatmapDensity),

        "elevation" => Ok(Expr::Elevation),

        "upcase" => Ok(Expr::Upcase(box_expr(take_arg(&mut args, 1, "upcase")?)?)),
        "downcase" => Ok(Expr::Downcase(box_expr(take_arg(
            &mut args, 1, "downcase",
        )?)?)),
        "concat" => Ok(Expr::Concat(parse_variadic(args)?)),
        "is-supported-script" => Ok(Expr::IsSupportedScript(box_expr(take_arg(
            &mut args,
            1,
            "is-supported-script",
        )?)?)),
        "resolved-locale" => Ok(Expr::ResolvedLocale(box_expr(take_arg(
            &mut args,
            1,
            "resolved-locale",
        )?)?)),
        "split" => Ok(Expr::Split(
            box_expr(take_arg(&mut args, 1, "split")?)?,
            box_expr(take_arg(&mut args, 2, "split")?)?,
        )),
        "join" => Ok(Expr::Join(
            box_expr(take_arg(&mut args, 1, "join")?)?,
            box_expr(take_arg(&mut args, 2, "join")?)?,
        )),

        other => Ok(Expr::Unknown {
            operator: other.to_owned(),
            args,
        }),
    }
}

/// Parse pairs of `(input, output)` from a flat argument list, used by
/// `step` and the `interpolate` family.
fn parse_stop_pairs(args: &mut Vec<Value>, op: &str) -> Result<Vec<(Expr, Expr)>, String> {
    if !args.len().is_multiple_of(2) {
        return Err(format!(
            "expression \"{op}\": stop arguments must come in (input, output) pairs, got {} args",
            args.len()
        ));
    }
    let mut stops = Vec::new();
    while args.len() >= 2 {
        let input = expr_from_value(args.remove(0))?;
        let output = expr_from_value(args.remove(0))?;
        stops.push((input, output));
    }
    Ok(stops)
}

/// Parse a variable-length list of expression arguments.
fn parse_variadic(args: Vec<Value>) -> Result<Vec<Expr>, String> {
    args.into_iter().map(expr_from_value).collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn parse(v: serde_json::Value) -> Expr {
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
            Expr::Step { default_output, .. } => {
                assert!(matches!(*default_output, Expr::Literal(Value::Number(_))));
            }
            other => panic!("expected Step, got {other:?}"),
        }
    }

    #[test]
    fn literal_string_as_arg() {
        // Strings appear as Literal args inside expressions (e.g. get property)
        let e = parse(json!(["get", "name"]));
        match e {
            Expr::Get { property, .. } => {
                assert!(matches!(*property, Expr::Literal(Value::String(_))));
            }
            other => panic!("expected Get, got {other:?}"),
        }
    }

    #[test]
    fn literal_bool_as_arg() {
        // Booleans appear as Literal args inside expressions
        let e = parse(json!(["!", false]));
        assert!(matches!(e, Expr::Not(inner) if matches!(*inner, Expr::Literal(Value::Bool(_)))));
    }

    #[test]
    fn zoom() {
        assert_eq!(parse(json!(["zoom"])), Expr::Zoom);
    }

    #[test]
    fn geometry_type() {
        assert_eq!(parse(json!(["geometry-type"])), Expr::GeometryType);
    }

    #[test]
    fn id_expr() {
        assert_eq!(parse(json!(["id"])), Expr::Id);
    }

    #[test]
    fn heatmap_density() {
        assert_eq!(parse(json!(["heatmap-density"])), Expr::HeatmapDensity);
    }

    #[test]
    fn get_no_object() {
        let e = parse(json!(["get", "name"]));
        assert!(matches!(e, Expr::Get { object: None, .. }));
    }

    #[test]
    fn get_with_object() {
        let e = parse(json!(["get", "name", ["properties"]]));
        assert!(matches!(
            e,
            Expr::Get {
                object: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn has_expr() {
        let e = parse(json!(["has", "population"]));
        assert!(matches!(e, Expr::Has { object: None, .. }));
    }

    #[test]
    fn step_expr() {
        let e = parse(json!(["step", ["zoom"], 0.5, 12, 1.0, 15, 2.0]));
        match e {
            Expr::Step { stops, .. } => assert_eq!(stops.len(), 2),
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
            Expr::Interpolate {
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
            Expr::Interpolate { interpolation, .. } => {
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
        assert!(matches!(e, Expr::InterpolateHcl { .. }));
    }

    #[test]
    fn add_expr() {
        let e = parse(json!(["+", 1, 2, 3]));
        match e {
            Expr::Add(args) => assert_eq!(args.len(), 3),
            other => panic!("expected Add, got {other:?}"),
        }
    }

    #[test]
    fn sub_unary() {
        let e = parse(json!(["-", ["get", "val"]]));
        assert!(matches!(e, Expr::Sub(_, None)));
    }

    #[test]
    fn sub_binary() {
        let e = parse(json!(["-", 10, 3]));
        assert!(matches!(e, Expr::Sub(_, Some(_))));
    }

    #[test]
    fn math_constants() {
        assert_eq!(parse(json!(["ln2"])), Expr::Ln2);
        assert_eq!(parse(json!(["pi"])), Expr::Pi);
        assert_eq!(parse(json!(["e"])), Expr::E);
    }

    #[test]
    fn case_expr() {
        let e = parse(json!(["case", ["has", "name"], "yes", "no"]));
        match e {
            Expr::Case { branches, .. } => assert_eq!(branches.len(), 1),
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
            Expr::Match { branches, .. } => {
                assert_eq!(branches.len(), 2);
                assert_eq!(branches[0].0, vec![json!("residential")]);
            }
            other => panic!("expected Match, got {other:?}"),
        }
    }

    #[test]
    fn match_expr_array_labels() {
        let e = parse(json!(["match", ["get", "n"], [1, 2], "low", "other"]));
        match e {
            Expr::Match { branches, .. } => {
                assert_eq!(branches[0].0, vec![json!(1), json!(2)]);
            }
            other => panic!("expected Match, got {other:?}"),
        }
    }

    #[test]
    fn all_any_not() {
        assert!(matches!(parse(json!(["all", true, false])), Expr::All(_)));
        assert!(matches!(parse(json!(["any", true, false])), Expr::Any(_)));
        assert!(matches!(parse(json!(["!", true])), Expr::Not(_)));
    }

    #[test]
    fn comparison_ops() {
        assert!(matches!(parse(json!(["==", 1, 1])), Expr::Eq(_, _)));
        assert!(matches!(parse(json!(["!=", 1, 2])), Expr::Ne(_, _)));
        assert!(matches!(parse(json!([">", 2, 1])), Expr::Gt(_, _)));
        assert!(matches!(parse(json!([">=", 2, 2])), Expr::Gte(_, _)));
        assert!(matches!(parse(json!(["<", 1, 2])), Expr::Lt(_, _)));
        assert!(matches!(parse(json!(["<=", 1, 2])), Expr::Lte(_, _)));
    }

    #[test]
    fn literal_expr() {
        let e = parse(json!(["literal", [1, 2, 3]]));
        assert!(matches!(e, Expr::LiteralExpr(Value::Array(_))));
    }

    #[test]
    fn array_assertion_bare() {
        let e = parse(json!(["array", ["get", "coords"]]));
        assert!(matches!(
            e,
            Expr::ArrayAssertion {
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
            Expr::ArrayAssertion {
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
            Expr::ArrayAssertion {
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
            Expr::Concat(args) => assert_eq!(args.len(), 3),
            other => panic!("expected Concat, got {other:?}"),
        }
    }

    #[test]
    fn upcase_downcase() {
        assert!(matches!(parse(json!(["upcase", "hi"])), Expr::Upcase(_)));
        assert!(matches!(
            parse(json!(["downcase", "HI"])),
            Expr::Downcase(_)
        ));
    }

    #[test]
    fn let_var_binding() {
        let e = parse(json!(["let", "x", 42, ["var", "x"]]));
        match e {
            Expr::Let { bindings, body } => {
                assert_eq!(bindings.len(), 1);
                assert_eq!(bindings[0].0, "x");
                assert!(matches!(*body, Expr::Var(ref s) if s == "x"));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn unknown_operator() {
        let e = parse(json!(["future-operator", 1, 2]));
        match e {
            Expr::Unknown { operator, args } => {
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
            Expr::Gte(left, right) => {
                assert!(matches!(*left, Expr::Get { .. }));
                assert!(matches!(*right, Expr::Literal(_)));
            }
            other => panic!("expected Gte, got {other:?}"),
        }
    }

    #[test]
    fn rgb_expr() {
        let e = parse(json!(["rgb", 255, 0, 128]));
        assert!(matches!(e, Expr::Rgb(_, _, _)));
    }

    #[test]
    fn feature_state_expr() {
        let e = parse(json!(["feature-state", "hover"]));
        assert!(matches!(e, Expr::FeatureState(_)));
    }

    #[test]
    fn format_expr() {
        let e = parse(json!(["format", ["get", "name"], {"font-scale": 0.8}, "\n", {}]));
        match e {
            Expr::Format(sections) => {
                assert_eq!(sections.len(), 2);
                assert!(sections[0].1.is_some()); // has style override
                assert!(sections[1].1.is_some()); // has empty style override
            }
            other => panic!("expected Format, got {other:?}"),
        }
    }
}
