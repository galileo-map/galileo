use std::borrow::Cow;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExprValue<'a> {
    Null,
    Boolean(bool),
    Number(f64),
    Color(Color),
    String(Cow<'a, str>),
    GeomType(ExprGeometryType),
    NumArray(Cow<'a, [f64]>),
}

impl ExprValue<'_> {
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

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ExprValue::Number(v) => Some(*v),
            _ => None,
        }
    }

    pub fn to_bool(&self) -> bool {
        self.as_bool().unwrap_or(false)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn borrowed<'a>(&'a self) -> ExprValue<'a> {
        match self {
            ExprValue::Null => ExprValue::Null,
            ExprValue::Boolean(v) => ExprValue::Boolean(*v),
            ExprValue::Number(v) => ExprValue::Number(*v),
            ExprValue::Color(v) => ExprValue::Color(*v),
            ExprValue::String(cow) => ExprValue::String(Cow::Borrowed(cow)),
            ExprValue::GeomType(v) => ExprValue::GeomType(*v),
            ExprValue::NumArray(cow) => ExprValue::NumArray(Cow::Borrowed(cow)),
        }
    }

    pub fn owned(&self) -> ExprValue<'static> {
        match self {
            ExprValue::Null => ExprValue::Null,
            ExprValue::Boolean(v) => ExprValue::Boolean(*v),
            ExprValue::Number(v) => ExprValue::Number(*v),
            ExprValue::Color(v) => ExprValue::Color(*v),
            ExprValue::String(cow) => ExprValue::String(Cow::Owned(cow.to_string())),
            ExprValue::GeomType(v) => ExprValue::GeomType(*v),
            ExprValue::NumArray(cow) => ExprValue::NumArray(Cow::Owned(cow.to_vec())),
        }
    }
}

impl From<bool> for ExprValue<'_> {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<f64> for ExprValue<'_> {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<ExprGeometryType> for ExprValue<'_> {
    fn from(value: ExprGeometryType) -> Self {
        Self::GeomType(value)
    }
}

impl From<Color> for ExprValue<'_> {
    fn from(value: Color) -> Self {
        Self::Color(value)
    }
}

impl From<String> for ExprValue<'_> {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}

impl From<Vec<f64>> for ExprValue<'_> {
    fn from(value: Vec<f64>) -> Self {
        Self::NumArray(value.into())
    }
}

impl PartialOrd for ExprValue<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (ExprValue::Boolean(v1), ExprValue::Boolean(v2)) => v1.partial_cmp(v2),
            (ExprValue::Number(v1), ExprValue::Number(v2)) => v1.partial_cmp(v2),
            (ExprValue::String(v1), ExprValue::String(v2)) => v1.partial_cmp(v2),
            _ => None,
        }
    }
}
