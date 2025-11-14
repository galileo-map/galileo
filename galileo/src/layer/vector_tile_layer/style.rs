//! See [`VectorTileStyle`].

use galileo_mvt::{MvtFeature, MvtGeometry};
use serde::{Deserialize, Serialize};

use crate::render::point_paint::PointPaint;
use crate::render::text::TextStyle;
use crate::render::{LineCap, LinePaint, PolygonPaint};
use crate::Color;

/// Style of a vector tile layer. This specifies how each feature in a tile should be rendered.
///
/// <div class="warning">This exact type is experimental and is likely to change in near future.</div>
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTileStyle {
    /// Rules for feature to be drawn. Rules are traversed in sequence until a rule that corresponds to a current feature
    /// is found, and that rule is used for drawing. If no rule corresponds to the feature, default symbol is used.
    pub rules: Vec<StyleRule>,

    /// Background color of tiles.
    pub background: Color,
}

impl VectorTileStyle {
    /// Get a rule for the given feature.
    pub fn get_style_rule(&self, layer_name: &str, feature: &MvtFeature) -> Option<&StyleRule> {
        self.rules.iter().find(|&rule| {
            let correct_geometry_type = match feature.geometry {
                MvtGeometry::Point(_)
                    if matches!(
                        rule.symbol,
                        VectorTileSymbol::Point(_) | VectorTileSymbol::Label(_)
                    ) =>
                {
                    true
                }
                MvtGeometry::LineString(_) if matches!(rule.symbol, VectorTileSymbol::Line(_)) => {
                    true
                }
                MvtGeometry::Polygon(_) if matches!(rule.symbol, VectorTileSymbol::Polygon(_)) => {
                    true
                }
                _ => false,
            };

            if !correct_geometry_type {
                return false;
            }

            if rule.layer_name.as_ref().is_some_and(|v| v != layer_name) {
                return false;
            }

            let filter_check_passed = rule.properties.iter().all(|filter| {
                let value = feature.properties.get(&filter.property_name);
                match (&filter.operator, value) {
                    (PropertyFilterOperator::Equal(value), Some(v)) => v.eq_str(value),
                    (PropertyFilterOperator::NotEqual(value), Some(v)) => !v.eq_str(value),
                    (PropertyFilterOperator::NotEqual(_), None) => true,
                    (PropertyFilterOperator::GreaterThan(value), Some(v)) => {
                        compare_numeric(v, value, |a, b| a > b)
                    }
                    (PropertyFilterOperator::LessThan(value), Some(v)) => {
                        compare_numeric(v, value, |a, b| a < b)
                    }
                    (PropertyFilterOperator::GreaterThanOrEqual(value), Some(v)) => {
                        compare_numeric(v, value, |a, b| a >= b)
                    }
                    (PropertyFilterOperator::LessThanOrEqual(value), Some(v)) => {
                        compare_numeric(v, value, |a, b| a <= b)
                    }
                    (PropertyFilterOperator::OneOf(values), Some(v)) => {
                        values.iter().any(|candidate| v.eq_str(candidate))
                    }
                    (PropertyFilterOperator::NotOneOf(values), Some(v)) => {
                        !values.iter().any(|candidate| v.eq_str(candidate))
                    }
                    (PropertyFilterOperator::Exist, Some(_)) => true,
                    (PropertyFilterOperator::NotExist, None) => true,

                    _ => false,
                }
            });

            filter_check_passed
        })
    }
}

fn compare_numeric(a: &galileo_mvt::MvtValue, b: &str, cmp: impl Fn(f64, f64) -> bool) -> bool {
    if let Some(a_num) = a.as_f64() {
        if let Ok(b_num) = b.parse::<f64>() {
            return cmp(a_num, b_num);
        }
    }

    false
}

/// A rule that specifies what kind of features can be drawing with the given symbol.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyleRule {
    /// If set, a feature must belong to the set layer. If not set, layer is not checked.
    pub layer_name: Option<String>,
    /// Specifies a set of attributes of a feature that must have the given values for this rule to be applied.
    #[serde(default)]
    pub properties: Vec<PropertyFilter>,
    /// Symbol to draw a feature with.
    #[serde(default)]
    pub symbol: VectorTileSymbol,
}

/// A filter that checks if a feature's property matches specific criteria.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropertyFilter {
    /// Name of the property to check.
    pub property_name: String,
    /// Operator and value(s) to compare the property against.
    pub operator: PropertyFilterOperator,
}

/// Operators for filtering feature properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyFilterOperator {
    /// Property value must equal the given string.
    Equal(String),
    /// Property value must not equal the given string.
    NotEqual(String),
    /// Property value (as number) must be greater than the given value.
    GreaterThan(String),
    /// Property value (as number) must be less than the given value.
    LessThan(String),
    /// Property value (as number) must be greater than or equal to the given value.
    GreaterThanOrEqual(String),
    /// Property value (as number) must be less than or equal to the given value.
    LessThanOrEqual(String),
    /// Property value must be one of the given values.
    OneOf(Vec<String>),
    /// Property value must not be one of the given values.
    NotOneOf(Vec<String>),
    /// Property must exist (regardless of value).
    Exist,
    /// Property must not exist.
    NotExist,
}

impl PropertyFilterOperator {
    /// Parse a property filter operator from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - The operator string (e.g., "==", "!=", ">", "<", "in", "exist")
    /// * `rhs` - The right-hand side value(s) to compare against
    ///
    /// # Returns
    ///
    /// `Some(PropertyFilterOperator)` if the operator is valid, `None` otherwise.
    pub fn from_str(s: &str, rhs: &str) -> Option<Self> {
        match s {
            "==" => Some(Self::Equal(rhs.to_string())),
            "!=" => Some(Self::NotEqual(rhs.to_string())),
            ">" => Some(Self::GreaterThan(rhs.to_string())),
            "<" => Some(Self::LessThan(rhs.to_string())),
            ">=" => Some(Self::GreaterThanOrEqual(rhs.to_string())),
            "<=" => Some(Self::LessThanOrEqual(rhs.to_string())),
            "in" => Some(Self::OneOf(
                rhs.split(',').map(|s| s.trim().to_string()).collect(),
            )),
            "not in" => Some(Self::NotOneOf(
                rhs.split(',').map(|s| s.trim().to_string()).collect(),
            )),
            "exist" => Some(Self::Exist),
            "not_exist" => Some(Self::NotExist),
            _ => None,
        }
    }
}

impl std::fmt::Display for PropertyFilterOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Equal(v) => write!(f, "== {}", v),
            Self::NotEqual(v) => write!(f, "!= {}", v),
            Self::GreaterThan(v) => write!(f, "> {}", v),
            Self::LessThan(v) => write!(f, "< {}", v),
            Self::GreaterThanOrEqual(v) => write!(f, ">= {}", v),
            Self::LessThanOrEqual(v) => write!(f, "<= {}", v),
            Self::OneOf(values) => write!(f, "in [{}]", values.join(", ")),
            Self::NotOneOf(values) => write!(f, "not in [{}]", values.join(", ")),
            Self::Exist => write!(f, "exist"),
            Self::NotExist => write!(f, "not exist"),
        }
    }
}

/// Symbol of an object in a vector tile.
///
/// An the object has incompatible type with the symbol, the object is not renderred.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VectorTileSymbol {
    /// Do not render object.
    #[default]
    None,
    /// Symbol for a point object.
    #[serde(rename = "point")]
    Point(VectorTilePointSymbol),
    /// Symbol for a line object.
    #[serde(rename = "line")]
    Line(VectorTileLineSymbol),
    /// Symbol for a polygon object.
    #[serde(rename = "polygon")]
    Polygon(VectorTilePolygonSymbol),
    /// Symbol for a point object that is renderred as a text label.
    #[serde(rename = "label")]
    Label(VectorTileLabelSymbol),
}

impl VectorTileSymbol {
    /// Get the line symbol if this is a line symbol.
    pub(crate) fn line(&self) -> Option<&VectorTileLineSymbol> {
        match self {
            Self::Line(symbol) => Some(symbol),
            _ => None,
        }
    }

    /// Get the polygon symbol if this is a polygon symbol.
    pub(crate) fn polygon(&self) -> Option<&VectorTilePolygonSymbol> {
        match self {
            Self::Polygon(symbol) => Some(symbol),
            _ => None,
        }
    }

    /// Get the point symbol if this is a point symbol.
    pub(crate) fn point(&self) -> Option<&VectorTilePointSymbol> {
        match self {
            Self::Point(symbol) => Some(symbol),
            _ => None,
        }
    }

    /// Get the label symbol if this is a label symbol.
    pub(crate) fn label(&self) -> Option<&VectorTileLabelSymbol> {
        match self {
            Self::Label(symbol) => Some(symbol),
            _ => None,
        }
    }
}

/// Symbol for point geometries.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTilePointSymbol {
    /// Size of the point.
    pub size: f64,
    /// Color of the point.
    pub color: Color,
}

impl From<VectorTilePointSymbol> for PointPaint<'_> {
    fn from(value: VectorTilePointSymbol) -> Self {
        PointPaint::circle(value.color, value.size as f32)
    }
}

/// Symbol for line geometries.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTileLineSymbol {
    /// Width of the line in pixels.
    pub width: f64,
    /// Color of the line in pixels.
    pub stroke_color: Color,
}

impl From<VectorTileLineSymbol> for LinePaint {
    fn from(value: VectorTileLineSymbol) -> Self {
        Self {
            color: value.stroke_color,
            width: value.width,
            offset: 0.0,
            line_cap: LineCap::Butt,
        }
    }
}

/// Symbol for polygon geometries.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTilePolygonSymbol {
    /// Color of the fill of polygon.
    pub fill_color: Color,
}

impl From<VectorTilePolygonSymbol> for PolygonPaint {
    fn from(value: VectorTilePolygonSymbol) -> Self {
        Self {
            color: value.fill_color,
        }
    }
}

/// Symbol of a point geometry that is renderred as text label on the map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTileLabelSymbol {
    /// Text of the label with substitutes for feature attributes.
    pub pattern: String,
    /// Style of the text.
    pub text_style: TextStyle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_serialization_point() {
        let symbol = VectorTileSymbol::Point(VectorTilePointSymbol {
            size: 10.0,
            color: Color::BLACK,
        });

        let _json = serde_json::to_string_pretty(&symbol).unwrap();

        let value = serde_json::to_value(&symbol).unwrap();
        assert!(value.as_object().unwrap().get("point").is_some());
        assert!(value.as_object().unwrap().get("polygon").is_none());
    }

    #[test]
    fn serialize_with_bincode() {
        let rule = StyleRule {
            layer_name: None,
            properties: vec![],
            symbol: VectorTileSymbol::None,
        };

        let serialized = bincode::serde::encode_to_vec(&rule, bincode::config::standard()).unwrap();
        let _: (StyleRule, _) =
            bincode::serde::decode_from_slice(&serialized, bincode::config::standard()).unwrap();
    }
}
