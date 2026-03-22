#![allow(missing_docs)] //TODO: temporary allow

use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize};

mod interpolation;
pub use interpolation::*;
mod match_expr;
pub use match_expr::*;

mod value;
pub use value::{ExprGeometryType, ExprValue};

use crate::Color;

pub mod parser;

/// An expression that can be evaluated against a feature and a view to produce an [`ExprValue`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Literal(ExprValue<String>),

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

    InterpolateLinear(Box<LinearInterpolation>),
    InterpolateExp(Box<ExponentialInterpolation>),
    InterpolateCubicBezier(Box<CubicBezierInterpolation>),

    Match(Box<MatchExpr>),

    WithOpacity(WithOpacityExpr),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TypedExpr<Out>(Expr, PhantomData<Out>);

pub type ColorExpr = TypedExpr<Color>;
pub type NumExpr = TypedExpr<f64>;
pub type BoolExpr = TypedExpr<bool>;

impl<T> Default for TypedExpr<T> {
    fn default() -> Self {
        Self(Expr::Literal(ExprValue::Null), PhantomData)
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

impl<Out> From<Expr> for TypedExpr<Out> {
    fn from(value: Expr) -> Self {
        Self::new(value)
    }
}

impl From<Color> for TypedExpr<Color> {
    fn from(value: Color) -> Self {
        Expr::Literal(value.into()).into()
    }
}

impl From<f64> for TypedExpr<f64> {
    fn from(value: f64) -> Self {
        Expr::Literal(value.into()).into()
    }
}

impl From<Color> for Expr {
    fn from(value: Color) -> Self {
        Expr::Literal(value.into())
    }
}

impl From<f64> for Expr {
    fn from(value: f64) -> Self {
        Expr::Literal(value.into())
    }
}

impl<'de, Out> Deserialize<'de> for TypedExpr<Out> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Buffer into a generic value so we can inspect the shape without consuming the input.
        let value = serde_json::Value::deserialize(deserializer)?;

        if let serde_json::Value::String(ref s) = value {
            return parser::parse_expr(s)
                .map(|(_, expr)| expr.into())
                .map_err(|e| serde::de::Error::custom(format!("expression parse error: {e}")));
        }

        Expr::deserialize(value)
            .map(Into::into)
            .map_err(serde::de::Error::custom)
    }
}

pub trait ExprFeature {
    fn property(&self, property_name: &str) -> ExprValue<&str>;
    fn geom_type(&self) -> ExprGeometryType;
}

pub struct EmptyExprFeature;
impl ExprFeature for EmptyExprFeature {
    fn property(&self, _property_name: &str) -> ExprValue<&str> {
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
    pub fn eval<'a>(&'a self, f: &'a impl ExprFeature, v: ExprView) -> ExprValue<&'a str> {
        match self {
            Expr::Literal(x) => x.borrowed(),
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
            Expr::InterpolateLinear(ip) => ip.eval(f, v),
            Expr::InterpolateExp(ip) => ip.eval(f, v),
            Expr::InterpolateCubicBezier(ip) => ip.eval(f, v),
            Expr::Match(m) => m.eval(f, v),
            Expr::WithOpacity(wo) => wo.eval(f, v),
        }
    }
}

impl From<ExprValue<String>> for Expr {
    fn from(value: ExprValue<String>) -> Self {
        Self::Literal(value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WithOpacityExpr {
    pub color: Box<Expr>,
    pub opacity: Box<Expr>,
}

impl WithOpacityExpr {
    pub fn eval<'a>(&'a self, f: &'a impl ExprFeature, v: ExprView) -> ExprValue<&'a str> {
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
    use super::*;

    #[test]
    fn deserialize_expr_from_string() {
        let json = r#""kind == \"road\"""#;
        let expr: BoolExpr = serde_json::from_str(json).unwrap();
        assert_eq!(
            expr.0,
            Expr::Eq(
                Box::new(Expr::Get("kind".to_string())),
                Box::new(Expr::Literal(ExprValue::String("road".to_string()))),
            )
        );
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
}
