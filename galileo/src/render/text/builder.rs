//! Builder for `TextStyle`.
use serde::{Deserialize, Serialize};

use crate::layer::vector_tile_layer::expressions::StyleValue;
use crate::render::text::{
    FontStyle, FontWeight, HorizontalAlignment, TextStyle, VerticalAlignment,
};
use crate::Color;

/// Raw Style of a text label that generates a `TextStyle`.
/// This allows for interpolation of fields like font_size,font_color,outline_color,outline_width.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextStyleBuilder {
    /// Name of the font to use.
    pub font_family: Vec<String>,
    /// Size of the font in pixels.
    pub font_size: StyleValue<f32>,
    /// Color of the font.
    #[serde(default = "default_font_color_style")]
    pub font_color: StyleValue<Color>,
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
    pub outline_width: StyleValue<f32>,
    /// Color of the outline around the letters.
    #[serde(default = "default_outline_color_style")]
    pub outline_color: StyleValue<Color>,
}

impl TextStyleBuilder {
    /// This method returns the value of the struct `TextStyle` on the basis of the
    /// current resolution level.
    pub fn get_value(self, current_resolution: f64) -> TextStyle {
        TextStyle {
            font_family: self.font_family,
            font_size: self.font_size.get_value(current_resolution),
            font_color: self.font_color.get_value(current_resolution),
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
            weight: self.weight,
            style: self.style,
            outline_width: self.outline_width.get_value(current_resolution),
            outline_color: self.outline_color.get_value(current_resolution),
        }
    }
}

fn default_font_color_style() -> StyleValue<Color> {
    Color::BLACK.into()
}

fn default_outline_color_style() -> StyleValue<Color> {
    Color::TRANSPARENT.into()
}

fn default_outline_width_style() -> StyleValue<f32> {
    0.0.into()
}
