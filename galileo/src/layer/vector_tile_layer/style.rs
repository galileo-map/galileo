//! See [`VectorTileStyle`].

use std::borrow::Cow;

use galileo_mvt::{MvtFeature, MvtValue};
use serde::{Deserialize, Serialize};

use crate::Color;
use crate::expr::{
    BoolExpr, ColorExpr, ExprFeature, ExprGeometryType, ExprValue, ExprView, NumExpr, TypedExpr,
};
use crate::render::point_paint::PointPaint;
use crate::render::text::{
    FontStyle, FontWeight, HorizontalAlignment, TextStyle, VerticalAlignment,
};
use crate::render::{LineCap, LinePaint, PolygonPaint};

/// Style of a vector tile layer. This specifies how each feature in a tile should be rendered.
///
/// <div class="warning">This exact type is experimental and is likely to change in near future.</div>
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTileStyle {
    /// Rules for feature to be drawn. Rules are traversed in sequence until a rule that corresponds to a current feature
    /// is found, and that rule is used for drawing. If no rule corresponds to the feature, default symbol is used.
    pub rules: Vec<StyleRule>,

    /// Background color of tiles.
    pub background: ColorExpr,
}

/// A rule that specifies what kind of features can be drawing with the given symbol.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyleRule {
    /// If set, a feature must belong to the set layer. If not set, rule is applied to all layers, that don't have
    /// a rule (e.g. this will be used as a default style).
    pub layer_name: Option<String>,
    /// If set, the rule will only be applied at resolutions lower than this value.
    pub max_resolution: Option<f64>,
    /// If set, the rule will only be applied at resolutions higher than this value.
    pub min_resolution: Option<f64>,
    /// Specifies a set of attributes of a feature that must have the given values for this rule to be applied.
    #[serde(default)]
    pub filter: Option<BoolExpr>,
    /// Symbol to draw a feature with.
    #[serde(default)]
    pub symbol: VectorTileSymbol,
}

impl StyleRule {
    /// Returns true, if the rule should be applied for the given feature.
    pub fn applies(&self, feature: &MvtFeature, resolution: f64, z_index: u32) -> bool {
        let Some(expr) = &self.filter else {
            return true;
        };

        let expr_view = ExprView {
            resolution,
            z_index: Some(z_index),
        };

        expr.eval(feature, expr_view)
    }
}

impl ExprFeature for MvtFeature {
    fn property(&self, property_name: &str) -> ExprValue<'_> {
        self.properties
            .get(property_name)
            .map(|v| v.into())
            .unwrap_or(ExprValue::Null)
    }

    fn geom_type(&self) -> ExprGeometryType {
        match self.geometry {
            galileo_mvt::MvtGeometry::Point(_) => ExprGeometryType::Point,
            galileo_mvt::MvtGeometry::LineString(_) => ExprGeometryType::Line,
            galileo_mvt::MvtGeometry::Polygon(_) => ExprGeometryType::Polygon,
        }
    }
}

impl<'a> From<&'a MvtValue> for ExprValue<'a> {
    fn from(value: &'a MvtValue) -> Self {
        match value {
            MvtValue::String(v) => Self::String(Cow::Borrowed(v)),
            MvtValue::Float(v) => Self::Number(*v as f64),
            MvtValue::Double(v) => Self::Number(*v),
            MvtValue::Int64(v) => Self::Number(*v as f64),
            MvtValue::Uint64(v) => Self::Number(*v as f64),
            MvtValue::Bool(v) => Self::Boolean(*v),
            MvtValue::Unknown => Self::Null,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTilePointSymbol {
    /// Size of the point.
    pub size: NumExpr,
    /// Color of the point.
    pub color: ColorExpr,
}

impl VectorTilePointSymbol {
    pub(crate) fn to_paint(&self, feature: &MvtFeature, view: ExprView) -> Option<PointPaint<'_>> {
        Some(PointPaint::circle(
            self.color.eval(feature, view)?,
            self.size.eval(feature, view)? as f32,
        ))
    }
}

/// Symbol for line geometries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTileLineSymbol {
    /// Width of the line in pixels.
    pub width: NumExpr,
    /// Color of the line in pixels.
    pub stroke_color: ColorExpr,
    /// Parameters of dash array for the line.
    ///
    /// Sets length of "dash - gap - dash - ..." of widths of the line. If the specification contains not even number of
    /// values, the whole pattern is repeated twice when applied.
    pub dasharray: Option<TypedExpr<Vec<f64>>>,
}

impl VectorTileLineSymbol {
    pub(crate) fn to_paint<'a>(
        &'a self,
        feature: &'a MvtFeature,
        view: ExprView,
    ) -> Option<LinePaint<'a>> {
        Some(LinePaint {
            color: self.stroke_color.eval(feature, view)?,
            width: self.width.eval(feature, view)?,
            offset: 0.0,
            line_cap: LineCap::Butt,
            dasharray: self.dasharray.as_ref().and_then(|v| v.eval(feature, view)),
        })
    }
}

/// Symbol for polygon geometries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTilePolygonSymbol {
    /// Color of the fill of polygon.
    pub fill_color: ColorExpr,
}

impl VectorTilePolygonSymbol {
    pub(crate) fn to_paint(&self, feature: &MvtFeature, view: ExprView) -> Option<PolygonPaint> {
        Some(PolygonPaint {
            color: self.fill_color.eval(feature, view)?,
        })
    }
}

/// Symbol of a point geometry that is renderred as text label on the map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorTileLabelSymbol {
    /// Text of the label with substitutes for feature attributes.
    pub pattern: String,
    /// Style of the text.
    pub text_style: VtTextStyle,
}

/// Raw Style of a text label that generates a `TextStyle`.
/// This allows for interpolation of fields like font_size,font_color,outline_color,outline_width.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VtTextStyle {
    /// Name of the font to use.
    pub font_family: Vec<String>,
    /// Size of the font in pixels.
    pub font_size: NumExpr,
    /// Color of the font.
    #[serde(default = "default_font_color_style")]
    pub font_color: ColorExpr,
    /// Alignment of label along horizontal axis.
    #[serde(default)]
    pub horizontal_alignment: HorizontalAlignment,
    /// Alignment of label along vertical axis.
    #[serde(default)]
    pub vertical_alignment: VerticalAlignment,
    /// Weight of the font.
    #[serde(default)]
    pub weight: FontWeight,
    /// sTyle of the font.
    #[serde(default)]
    pub style: FontStyle,
    /// Width of the outline around the letters.
    #[serde(default = "default_outline_width_style")]
    pub outline_width: NumExpr,
    /// Color of the outline around the letters.
    #[serde(default = "default_outline_color_style")]
    pub outline_color: ColorExpr,
}

impl VtTextStyle {
    /// This method returns the value of the struct `TextStyle` on the basis of the
    /// current resolution level.
    pub fn get_value(self, feature: &MvtFeature, view: ExprView) -> Option<TextStyle> {
        Some(TextStyle {
            font_family: self.font_family,
            font_size: self.font_size.eval(feature, view)? as f32,
            font_color: self.font_color.eval(feature, view)?,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
            weight: self.weight,
            style: self.style,
            outline_width: self.outline_width.eval(feature, view)? as f32,
            outline_color: self.outline_color.eval(feature, view)?,
        })
    }
}

fn default_font_color_style() -> ColorExpr {
    Color::BLACK.into()
}

fn default_outline_color_style() -> ColorExpr {
    Color::TRANSPARENT.into()
}

fn default_outline_width_style() -> NumExpr {
    0.0.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_serialization_point() {
        let symbol = VectorTileSymbol::Point(VectorTilePointSymbol {
            size: 10.0.into(),
            color: Color::BLACK.into(),
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
            min_resolution: None,
            max_resolution: None,
            filter: None,
            symbol: VectorTileSymbol::None,
        };

        let serialized = bincode::serde::encode_to_vec(&rule, bincode::config::standard()).unwrap();
        let _: (StyleRule, _) =
            bincode::serde::decode_from_slice(&serialized, bincode::config::standard()).unwrap();
    }
}
