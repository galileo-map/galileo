#![allow(missing_docs)] //TODO: temporary allow

use serde::{Deserialize, Serialize};

mod value;
pub use value::{ExprGeometryType, ExprValue};

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
}

pub trait ExprFeature {
    fn property(&self, property_name: &str) -> ExprValue<&str>;
    fn geom_type(&self) -> ExprGeometryType;
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
        }
    }
}

impl From<ExprValue<String>> for Expr {
    fn from(value: ExprValue<String>) -> Self {
        Self::Literal(value)
    }
}
