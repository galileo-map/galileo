#![allow(missing_docs)] //TODO: temporary allow

use std::borrow::Cow;
use std::marker::PhantomData;

use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serialize};

mod interpolation;
pub use interpolation::*;
mod match_expr;
pub use match_expr::*;

mod value;
use serde_json::Value;
pub use value::{ExprGeometryType, ExprValue};

use crate::Color;

pub mod parser;

/// An expression that can be evaluated against a feature and a view to produce an [`ExprValue`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    All(Vec<Expr>),
    Any(Vec<Expr>),
    Not(Box<Expr>),

    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),

    Get(String),
    In {
        needle: Box<Expr>,
        haystack: Vec<Expr>,
    },

    GeomType,
    Zoom,

    Linear(Box<LinearInterpolation>),
    Exponential(Box<ExponentialInterpolation>),
    CubicBezier(Box<CubicBezierInterpolation>),

    Match(Box<MatchExpr>),

    WithOpacity(WithOpacityExpr),

    #[serde(untagged)]
    Value(ExprValue<'static>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TypedExpr<Out>(Expr, PhantomData<Out>);

pub type ColorExpr = TypedExpr<Color>;
pub type NumExpr = TypedExpr<f64>;
pub type BoolExpr = TypedExpr<bool>;

impl<T> Default for TypedExpr<T> {
    fn default() -> Self {
        Self(Expr::Value(ExprValue::Null), PhantomData)
    }
}

impl<Out> TypedExpr<Out> {
    pub const fn new(expr: Expr) -> Self {
        Self(expr, PhantomData)
    }
}

impl TypedExpr<Color> {
    pub fn eval(&self, f: &impl ExprFeature, v: ExprView) -> Option<Color> {
        self.0.eval(f, v).as_color()
    }
}

impl TypedExpr<f64> {
    pub fn eval(&self, f: &impl ExprFeature, v: ExprView) -> Option<f64> {
        self.0.eval(f, v).as_number()
    }
}

impl TypedExpr<bool> {
    pub fn eval(&self, f: &impl ExprFeature, v: ExprView) -> bool {
        self.0.eval(f, v).to_bool()
    }
}

impl TypedExpr<Vec<f64>> {
    pub fn eval<'a>(&'a self, f: &'a impl ExprFeature, v: ExprView) -> Option<Cow<'a, [f64]>> {
        match self.0.eval(f, v) {
            ExprValue::NumArray(arr) => Some(arr),
            _ => None,
        }
    }
}

impl<Out> From<Expr> for TypedExpr<Out> {
    fn from(value: Expr) -> Self {
        Self::new(value)
    }
}

impl From<Color> for TypedExpr<Color> {
    fn from(value: Color) -> Self {
        Expr::Value(value.into()).into()
    }
}

impl From<f64> for TypedExpr<f64> {
    fn from(value: f64) -> Self {
        Expr::Value(value.into()).into()
    }
}

impl From<Color> for Expr {
    fn from(value: Color) -> Self {
        Expr::Value(value.into())
    }
}

impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Expr::Value(value.into())
    }
}

impl From<String> for Expr {
    fn from(value: String) -> Self {
        Expr::Value(value.into())
    }
}

impl From<bool> for Expr {
    fn from(value: bool) -> Self {
        Expr::Value(value.into())
    }
}

impl From<Vec<f64>> for Expr {
    fn from(value: Vec<f64>) -> Self {
        Expr::Value(value.into())
    }
}

impl<'de, Out> Deserialize<'de> for TypedExpr<Out> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Buffer into a generic value so we can inspect the shape without consuming the input.
        let value = serde_json::Value::deserialize(deserializer)?;

        let expr = match value {
            Value::Number(v) => v
                .as_f64()
                .ok_or_else(|| serde::de::Error::custom(format!("invalid number value: {v}")))?
                .into(),
            Value::Bool(v) => v.into(),
            Value::Null => ExprValue::Null.into(),
            Value::String(s) => parser::parse_expr(&s).map_err(|e| {
                let errors = e.into_iter().map(|v| v.to_string());
                let errors = Itertools::intersperse(errors, ", ".to_string());
                serde::de::Error::custom(format!(
                    "expression parse error: {}",
                    errors.collect::<String>(),
                ))
            })?,
            v => Expr::deserialize(v).map_err(serde::de::Error::custom)?,
        };

        Ok(expr.into())
    }
}

pub trait ExprFeature {
    fn property(&self, property_name: &str) -> ExprValue<'_>;
    fn geom_type(&self) -> ExprGeometryType;
}

pub struct EmptyExprFeature;
impl ExprFeature for EmptyExprFeature {
    fn property(&self, _property_name: &str) -> ExprValue<'_> {
        ExprValue::Null
    }

    fn geom_type(&self) -> ExprGeometryType {
        ExprGeometryType::None
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ExprView {
    pub resolution: f64,
    pub z_index: Option<u32>,
}

impl Expr {
    pub fn eval<'a>(&'a self, f: &'a impl ExprFeature, v: ExprView) -> ExprValue<'a> {
        match self {
            Expr::Value(x) => x.borrowed(),
            Expr::All(exprs) => exprs.iter().all(|expr| expr.eval(f, v).to_bool()).into(),
            Expr::Any(exprs) => exprs.iter().any(|expr| expr.eval(f, v).to_bool()).into(),
            Expr::Not(expr) => (!expr.eval(f, v).to_bool()).into(),
            Expr::Eq(lhs, rhs) => (lhs.eval(f, v) == rhs.eval(f, v)).into(),
            Expr::Ne(lhs, rhs) => (lhs.eval(f, v) != rhs.eval(f, v)).into(),
            Expr::Gt(lhs, rhs) => (lhs.eval(f, v) > rhs.eval(f, v)).into(),
            Expr::Gte(lhs, rhs) => (lhs.eval(f, v) >= rhs.eval(f, v)).into(),
            Expr::Lt(lhs, rhs) => (lhs.eval(f, v) < rhs.eval(f, v)).into(),
            Expr::Lte(lhs, rhs) => (lhs.eval(f, v) <= rhs.eval(f, v)).into(),
            Expr::Get(prop) => f.property(prop),
            Expr::In { needle, haystack } => {
                let value = needle.eval(f, v);
                if value.is_null() {
                    return false.into();
                };

                haystack.iter().any(|expr| expr.eval(f, v) == value).into()
            }
            Expr::GeomType => f.geom_type().into(),
            Expr::Zoom => v
                .z_index
                .map(|z_level| (z_level as f64).into())
                .unwrap_or(ExprValue::Null),
            Expr::Linear(ip) => ip.eval(f, v),
            Expr::Exponential(ip) => ip.eval(f, v),
            Expr::CubicBezier(ip) => ip.eval(f, v),
            Expr::Match(m) => m.eval(f, v),
            Expr::WithOpacity(wo) => wo.eval(f, v),
        }
    }
}

impl From<ExprValue<'static>> for Expr {
    fn from(value: ExprValue<'static>) -> Self {
        Self::Value(value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WithOpacityExpr {
    pub color: Box<Expr>,
    pub opacity: Box<Expr>,
}

impl WithOpacityExpr {
    pub fn eval<'a>(&'a self, f: &'a impl ExprFeature, v: ExprView) -> ExprValue<'a> {
        let Some(color) = self.color.eval(f, v).as_color() else {
            return ExprValue::Null;
        };

        let Some(opacity) = self.opacity.eval(f, v).as_f64() else {
            return ExprValue::Null;
        };

        ExprValue::Color(color.with_alpha_float(opacity))
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn deserialize_expr_from_string() {
        let json = r#""kind == \"road\"""#;
        let expr: BoolExpr = serde_json::from_str(json).unwrap();
        assert_eq!(
            expr.0,
            Expr::Eq(
                Box::new(Expr::Get("kind".to_string())),
                Box::new(Expr::Value("road".to_string().into())),
            )
        );
    }

    #[test]
    fn deserialize_expr_from_num() {
        let json = "42";
        let expr: NumExpr = serde_json::from_str(json).unwrap();
        assert_eq!(expr.0, Expr::Value(ExprValue::Number(42.0)));
    }

    #[test]
    fn deserialize_expr_from_object() {
        let json = r#"{"Get": "kind"}"#;
        let expr: NumExpr = serde_json::from_str(json).unwrap();
        assert_eq!(expr.0, Expr::Get("kind".to_string()));
    }

    #[test]
    fn deserialize_bad_string_returns_error() {
        let json = r#""@@@ not valid @@@""#;
        assert!(serde_json::from_str::<NumExpr>(json).is_err());
    }

    #[test]
    fn serialization_of_complex_expressions() {
        let expr = Expr::Exponential(Box::new(ExponentialInterpolation {
            base: 2.0,
            input: Expr::Zoom,
            control_points: vec![
                ControlPoint {
                    input: 10.0.into(),
                    output: Color::BLACK.into(),
                },
                ControlPoint {
                    input: 20.0.into(),
                    output: Color::RED.into(),
                },
            ],
        }));

        let json = serde_json::to_string_pretty(&expr).unwrap();
        let deser: Expr = serde_json::from_str(&json).unwrap();
        assert_eq!(deser, expr);

        assert_snapshot!(json, @r##"
        {
          "Exponential": {
            "base": 2.0,
            "input": "Zoom",
            "control_points": [
              {
                "input": 10.0,
                "output": "#000000FF"
              },
              {
                "input": 20.0,
                "output": "#FF0000FF"
              }
            ]
          }
        }
        "##);
    }
}
