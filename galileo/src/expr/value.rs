use serde::{Deserialize, Serialize};

use crate::Color;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExprGeometryType {
    None,
    Point,
    Line,
    Polygon,
    MultiLine,
    MultiPolygon,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprValue<S> {
    Null,
    Boolean(bool),
    Number(f64),
    Color(Color),
    String(S),
    GeomType(ExprGeometryType),
}

impl<S> ExprValue<S> {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ExprValue::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            ExprValue::Number(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_color(&self) -> Option<Color> {
        match self {
            ExprValue::Color(v) => Some(*v),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> bool {
        self.as_bool().unwrap_or(false)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl ExprValue<String> {
    pub(super) fn borrowed(&self) -> ExprValue<&str> {
        match self {
            ExprValue::Null => ExprValue::Null,
            ExprValue::Boolean(v) => ExprValue::Boolean(*v),
            ExprValue::Number(v) => ExprValue::Number(*v),
            ExprValue::Color(v) => ExprValue::Color(*v),
            ExprValue::String(v) => ExprValue::String(v),
            ExprValue::GeomType(v) => ExprValue::GeomType(*v),
        }
    }
}

impl<S> From<bool> for ExprValue<S> {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl<S> From<f64> for ExprValue<S> {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl<S> From<ExprGeometryType> for ExprValue<S> {
    fn from(value: ExprGeometryType) -> Self {
        Self::GeomType(value)
    }
}

impl<S> From<Color> for ExprValue<S> {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}

impl<S: PartialOrd> PartialOrd for ExprValue<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (ExprValue::Boolean(v1), ExprValue::Boolean(v2)) => v1.partial_cmp(v2),
            (ExprValue::Number(v1), ExprValue::Number(v2)) => v1.partial_cmp(v2),
            (ExprValue::String(v1), ExprValue::String(v2)) => v1.partial_cmp(v2),
            _ => None,
        }
    }
}
