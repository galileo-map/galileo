//! Symbol layer types.

use serde::Deserialize;
use serde_json::Value;

use super::common::{
    Visibility, default_layer_maxzoom, default_layer_minzoom, deserialize_maxzoom,
    deserialize_minzoom, deserialize_visibility_or_default,
};

/// Paint properties for a `symbol` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct SymbolPaint {
    /// Icon colour. Supports expressions.
    #[serde(rename = "icon-color", skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<Value>,

    /// Icon halo colour. Supports expressions.
    #[serde(rename = "icon-halo-color", skip_serializing_if = "Option::is_none")]
    pub icon_halo_color: Option<Value>,

    /// Width of the halo around the icon. Supports expressions.
    #[serde(rename = "icon-halo-width", skip_serializing_if = "Option::is_none")]
    pub icon_halo_width: Option<Value>,

    /// Fade out the halo towards the outside. Supports expressions.
    #[serde(rename = "icon-halo-blur", skip_serializing_if = "Option::is_none")]
    pub icon_halo_blur: Option<Value>,

    /// Icon opacity. Supports expressions.
    #[serde(rename = "icon-opacity", skip_serializing_if = "Option::is_none")]
    pub icon_opacity: Option<Value>,

    /// Icon translation offset. Supports expressions.
    #[serde(rename = "icon-translate", skip_serializing_if = "Option::is_none")]
    pub icon_translate: Option<Value>,

    /// Control whether the icon translation is relative to map or viewport.
    #[serde(
        rename = "icon-translate-anchor",
        skip_serializing_if = "Option::is_none"
    )]
    pub icon_translate_anchor: Option<Value>,

    /// Emission strength of the icon colour.
    #[serde(
        rename = "icon-emissive-strength",
        skip_serializing_if = "Option::is_none"
    )]
    pub icon_emissive_strength: Option<Value>,

    /// Text colour. Supports expressions.
    #[serde(rename = "text-color", skip_serializing_if = "Option::is_none")]
    pub text_color: Option<Value>,

    /// Text halo colour. Supports expressions.
    #[serde(rename = "text-halo-color", skip_serializing_if = "Option::is_none")]
    pub text_halo_color: Option<Value>,

    /// Width of the halo around text. Supports expressions.
    #[serde(rename = "text-halo-width", skip_serializing_if = "Option::is_none")]
    pub text_halo_width: Option<Value>,

    /// Fade out the halo towards the outside. Supports expressions.
    #[serde(rename = "text-halo-blur", skip_serializing_if = "Option::is_none")]
    pub text_halo_blur: Option<Value>,

    /// Text opacity. Supports expressions.
    #[serde(rename = "text-opacity", skip_serializing_if = "Option::is_none")]
    pub text_opacity: Option<Value>,

    /// Text translation offset. Supports expressions.
    #[serde(rename = "text-translate", skip_serializing_if = "Option::is_none")]
    pub text_translate: Option<Value>,

    /// Control whether the text translation is relative to map or viewport.
    #[serde(
        rename = "text-translate-anchor",
        skip_serializing_if = "Option::is_none"
    )]
    pub text_translate_anchor: Option<Value>,

    /// Emission strength of the text colour.
    #[serde(
        rename = "text-emissive-strength",
        skip_serializing_if = "Option::is_none"
    )]
    pub text_emissive_strength: Option<Value>,
}

/// Layout properties for a `symbol` layer.
#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct SymbolLayout {
    /// Whether this layer is displayed.
    #[serde(default, deserialize_with = "deserialize_visibility_or_default")]
    pub visibility: Visibility,

    /// Label placement relative to its geometry. Supports expressions.
    #[serde(rename = "symbol-placement", skip_serializing_if = "Option::is_none")]
    pub symbol_placement: Option<Value>,

    /// Distance between two symbol anchors. Supports expressions.
    #[serde(rename = "symbol-spacing", skip_serializing_if = "Option::is_none")]
    pub symbol_spacing: Option<Value>,

    /// Whether symbols can be placed at map edges.
    #[serde(rename = "symbol-avoid-edges", skip_serializing_if = "Option::is_none")]
    pub symbol_avoid_edges: Option<Value>,

    /// Determines whether overlapping symbols in the same layer are hidden.
    #[serde(rename = "symbol-sort-key", skip_serializing_if = "Option::is_none")]
    pub symbol_sort_key: Option<Value>,

    /// Controls the order in which overlapping symbols are rendered.
    #[serde(rename = "symbol-z-order", skip_serializing_if = "Option::is_none")]
    pub symbol_z_order: Option<Value>,

    /// If true, the icon will be visible even if it collides with other icons.
    #[serde(rename = "icon-allow-overlap", skip_serializing_if = "Option::is_none")]
    pub icon_allow_overlap: Option<Value>,

    /// Part of the icon placed nearest to the anchor.
    #[serde(rename = "icon-anchor", skip_serializing_if = "Option::is_none")]
    pub icon_anchor: Option<Value>,

    /// If true, other symbols can be visible even if they collide with the icon.
    #[serde(
        rename = "icon-ignore-placement",
        skip_serializing_if = "Option::is_none"
    )]
    pub icon_ignore_placement: Option<Value>,

    /// Name of an image in the sprite to use for drawing an icon.
    #[serde(rename = "icon-image", skip_serializing_if = "Option::is_none")]
    pub icon_image: Option<Value>,

    /// If true, the icon may be flipped to prevent text label being upside-down.
    #[serde(rename = "icon-keep-upright", skip_serializing_if = "Option::is_none")]
    pub icon_keep_upright: Option<Value>,

    /// Offset distance of icon from its anchor. Supports expressions.
    #[serde(rename = "icon-offset", skip_serializing_if = "Option::is_none")]
    pub icon_offset: Option<Value>,

    /// If true, text is optional when icon is blocked.
    #[serde(rename = "icon-optional", skip_serializing_if = "Option::is_none")]
    pub icon_optional: Option<Value>,

    /// Size of the additional area around the icon used to detect collisions.
    #[serde(rename = "icon-padding", skip_serializing_if = "Option::is_none")]
    pub icon_padding: Option<Value>,

    /// Rotates the icon clockwise. Supports expressions.
    #[serde(rename = "icon-rotate", skip_serializing_if = "Option::is_none")]
    pub icon_rotate: Option<Value>,

    /// In combination with `symbol-placement`, determines the rotation of the icon.
    #[serde(
        rename = "icon-rotation-alignment",
        skip_serializing_if = "Option::is_none"
    )]
    pub icon_rotation_alignment: Option<Value>,

    /// Scales the icon. Supports expressions.
    #[serde(rename = "icon-size", skip_serializing_if = "Option::is_none")]
    pub icon_size: Option<Value>,

    /// Positions icon relative to other icons.
    #[serde(rename = "icon-text-fit", skip_serializing_if = "Option::is_none")]
    pub icon_text_fit: Option<Value>,

    /// Padding around the text when `icon-text-fit` is used.
    #[serde(
        rename = "icon-text-fit-padding",
        skip_serializing_if = "Option::is_none"
    )]
    pub icon_text_fit_padding: Option<Value>,

    /// If true, the text is visible even if it collides.
    #[serde(rename = "text-allow-overlap", skip_serializing_if = "Option::is_none")]
    pub text_allow_overlap: Option<Value>,

    /// Part of the text placed nearest to the anchor. Supports expressions.
    #[serde(rename = "text-anchor", skip_serializing_if = "Option::is_none")]
    pub text_anchor: Option<Value>,

    /// Value to use for a text label. Supports expressions.
    #[serde(rename = "text-field", skip_serializing_if = "Option::is_none")]
    pub text_field: Option<Value>,

    /// Font stack for the glyphs. Supports expressions.
    #[serde(rename = "text-font", skip_serializing_if = "Option::is_none")]
    pub text_font: Option<Value>,

    /// If true, other symbols can be visible even if they collide with the text.
    #[serde(
        rename = "text-ignore-placement",
        skip_serializing_if = "Option::is_none"
    )]
    pub text_ignore_placement: Option<Value>,

    /// Text justification. Supports expressions.
    #[serde(rename = "text-justify", skip_serializing_if = "Option::is_none")]
    pub text_justify: Option<Value>,

    /// If true, the text may be flipped vertically when rotated.
    #[serde(rename = "text-keep-upright", skip_serializing_if = "Option::is_none")]
    pub text_keep_upright: Option<Value>,

    /// Text tracking. Supports expressions.
    #[serde(
        rename = "text-letter-spacing",
        skip_serializing_if = "Option::is_none"
    )]
    pub text_letter_spacing: Option<Value>,

    /// Text leading value. Supports expressions.
    #[serde(rename = "text-line-height", skip_serializing_if = "Option::is_none")]
    pub text_line_height: Option<Value>,

    /// Maximum angle change between adjacent characters. Supports expressions.
    #[serde(rename = "text-max-angle", skip_serializing_if = "Option::is_none")]
    pub text_max_angle: Option<Value>,

    /// The maximum line width for text wrapping. Supports expressions.
    #[serde(rename = "text-max-width", skip_serializing_if = "Option::is_none")]
    pub text_max_width: Option<Value>,

    /// Offset distance of text from its anchor. Supports expressions.
    #[serde(rename = "text-offset", skip_serializing_if = "Option::is_none")]
    pub text_offset: Option<Value>,

    /// If true, icons are optional when text is blocked.
    #[serde(rename = "text-optional", skip_serializing_if = "Option::is_none")]
    pub text_optional: Option<Value>,

    /// Size of the additional area around the text used to detect collisions.
    #[serde(rename = "text-padding", skip_serializing_if = "Option::is_none")]
    pub text_padding: Option<Value>,

    /// Radial offset of text. Supports expressions.
    #[serde(rename = "text-radial-offset", skip_serializing_if = "Option::is_none")]
    pub text_radial_offset: Option<Value>,

    /// In combination with `symbol-placement`, determines the rotation of the text.
    #[serde(
        rename = "text-rotation-alignment",
        skip_serializing_if = "Option::is_none"
    )]
    pub text_rotation_alignment: Option<Value>,

    /// Font size. Supports expressions.
    #[serde(rename = "text-size", skip_serializing_if = "Option::is_none")]
    pub text_size: Option<Value>,

    /// Specifies how to capitalize text. Supports expressions.
    #[serde(rename = "text-transform", skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<Value>,

    /// To increase the chance of placing high-priority labels on the map.
    #[serde(
        rename = "text-variable-anchor",
        skip_serializing_if = "Option::is_none"
    )]
    pub text_variable_anchor: Option<Value>,

    /// The property to use as a feature's writing direction.
    #[serde(rename = "text-writing-mode", skip_serializing_if = "Option::is_none")]
    pub text_writing_mode: Option<Value>,
}

/// A `symbol` layer draws icons or text labels at points or along lines.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SymbolLayer {
    /// Unique layer identifier.
    pub id: String,

    /// Source to use for this layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Layer within the source to use.
    #[serde(rename = "source-layer", skip_serializing_if = "Option::is_none")]
    pub source_layer: Option<String>,

    /// Arbitrary properties for tracking the layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,

    /// Minimum zoom level at which to show this layer.
    #[serde(
        default = "default_layer_minzoom",
        deserialize_with = "deserialize_minzoom"
    )]
    pub minzoom: f64,

    /// Maximum zoom level at which to show this layer.
    #[serde(
        default = "default_layer_maxzoom",
        deserialize_with = "deserialize_maxzoom"
    )]
    pub maxzoom: f64,

    /// Filter expression to select features from the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,

    /// Layout properties.
    #[serde(default)]
    pub layout: SymbolLayout,

    /// Paint properties.
    #[serde(default)]
    pub paint: SymbolPaint,
}
