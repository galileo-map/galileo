use galileo::layer::vector_tile_layer::style::{
    StyleRule, VectorTilePolygonSymbol, VectorTileStyle, VectorTileSymbol,
};
use galileo::layer::vector_tile_layer::VectorTileLayerBuilder;
use galileo::layer::VectorTileLayer;
use galileo::tile_schema::TileSchemaBuilder;
use galileo::Color;
use serde_json::Value;

use crate::color::parse_css_color;
use crate::layer::UNSUPPORTED;
use crate::style::layer::{FillLayer, Layer as MaplibreStyleLayer};
use crate::style::source::VectorSource;

/// Tries to create a [`VectorTileLayer`] from a Maplibre vector source and the style layers that
/// reference it. Returns `None` if the source cannot be used (e.g. no direct tile URLs).
pub fn try_create(
    source_name: &str,
    source: &VectorSource,
    layers: &[&MaplibreStyleLayer],
) -> Option<VectorTileLayer> {
    let tile_url = match tile_url_template(source) {
        Some(url) => url,
        None => {
            log::info!(
                "{UNSUPPORTED} Vector source '{source_name}' uses a TileJSON URL rather than \
                 direct tile URLs, which is not yet supported. Open a GitHub issue or PR if \
                 support is desirable."
            );
            return None;
        }
    };

    let rules = build_rules(layers);
    let style = VectorTileStyle {
        rules,
        background: Color::TRANSPARENT,
    };

    let min_z = source.minzoom as u32;
    let max_z = source.maxzoom as u32;
    let tile_schema = TileSchemaBuilder::web_mercator(min_z..=max_z)
        .build()
        .ok()?;

    VectorTileLayerBuilder::new_rest(move |index| {
        tile_url
            .replace("{z}", &index.z.to_string())
            .replace("{x}", &index.x.to_string())
            .replace("{y}", &index.y.to_string())
    })
    .with_tile_schema(tile_schema)
    .with_style(style)
    .build()
    .ok()
}

/// Returns the first direct tile URL template from the source, if available.
fn tile_url_template(source: &VectorSource) -> Option<String> {
    source.tiles.as_ref()?.first().cloned()
}

/// Converts each supported style layer into a [`StyleRule`], logging unsupported ones.
fn build_rules(layers: &[&MaplibreStyleLayer]) -> Vec<StyleRule> {
    let mut rules = Vec::new();
    for &layer in layers {
        match layer {
            MaplibreStyleLayer::Fill(fill) => {
                if let Some(rule) = fill_rule(fill) {
                    rules.push(rule);
                }
            }
            other => {
                log::info!(
                    "{UNSUPPORTED} Maplibre layer type '{}' (id: '{}') inside a vector source \
                     is not yet supported. Open a GitHub issue or PR if support is desirable.",
                    other.type_name(),
                    other.id(),
                );
            }
        }
    }
    rules
}

/// Converts a [`FillLayer`] to a [`StyleRule`], or logs and returns `None` if unsupported.
fn fill_rule(fill: &FillLayer) -> Option<StyleRule> {
    let source_layer = match &fill.source_layer {
        Some(l) => l.clone(),
        None => {
            log::info!(
                "{UNSUPPORTED} Fill layer '{}' has no source-layer; skipping.",
                fill.id
            );
            return None;
        }
    };

    let fill_color = extract_literal_color(&fill.paint.fill_color, &fill.id, "fill-color")?;
    let opacity = extract_literal_f64(&fill.paint.fill_opacity, &fill.id, "fill-opacity");
    let color = apply_opacity(fill_color, opacity.unwrap_or(1.0));

    Some(StyleRule {
        layer_name: Some(source_layer),
        symbol: VectorTileSymbol::Polygon(VectorTilePolygonSymbol {
            fill_color: color.into(),
        }),
        ..Default::default()
    })
}

/// Extracts a color from a paint property that must be a literal CSS color string.
/// Logs and returns `None` for expressions, functions, or unparsable values.
fn extract_literal_color(value: &Option<Value>, layer_id: &str, prop: &str) -> Option<Color> {
    match value {
        None => None,
        Some(Value::String(s)) => match parse_css_color(s) {
            Some(c) => Some(c),
            None => {
                log::warn!(
                    "Failed to parse color '{s}' for property '{prop}' in layer '{layer_id}'."
                );
                None
            }
        },
        Some(_) => {
            log::info!(
                "{UNSUPPORTED} Property '{prop}' in layer '{layer_id}' uses an expression or \
                 function, which is not yet supported. Open a GitHub issue or PR if support is \
                 desirable."
            );
            None
        }
    }
}

/// Extracts a number from a paint property that must be a literal JSON number.
/// Logs and returns `None` for expressions or functions; silently returns `None` for absent values.
fn extract_literal_f64(value: &Option<Value>, layer_id: &str, prop: &str) -> Option<f64> {
    match value {
        None => None,
        Some(Value::Number(n)) => n.as_f64(),
        Some(_) => {
            log::info!(
                "{UNSUPPORTED} Property '{prop}' in layer '{layer_id}' uses an expression or \
                 function, which is not yet supported. Open a GitHub issue or PR if support is \
                 desirable."
            );
            None
        }
    }
}

fn apply_opacity(color: Color, opacity: f64) -> Color {
    Color::rgba(
        color.r(),
        color.g(),
        color.b(),
        (color.a() as f64 * opacity).round() as u8,
    )
}
