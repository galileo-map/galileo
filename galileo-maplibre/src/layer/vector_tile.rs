use galileo::Color;
use galileo::galileo_types::cartesian::{CartesianPoint2d, Point2, Rect};
use galileo::galileo_types::geo::impls::GeoPoint2d;
use galileo::galileo_types::geo::{Crs, NewGeoPoint, Projection};
use galileo::layer::VectorTileLayer;
use galileo::layer::vector_tile_layer::VectorTileLayerBuilder;
use galileo::layer::vector_tile_layer::expressions::{
    ExponentialInterpolationArgs, InterpolateExpression, InterpolationArgs, OperationBase,
    StepValue, StyleValue,
};
use galileo::layer::vector_tile_layer::style::{
    StyleRule, VectorTilePolygonSymbol, VectorTileStyle, VectorTileSymbol,
};
use galileo::tile_schema::{TileSchema, TileSchemaBuilder, VerticalDirection};
use serde::Deserialize;
use serde_json::Value;

use crate::layer::UNSUPPORTED;
use crate::style::layer::{FillLayer, Layer as MaplibreStyleLayer};
use crate::style::source::{TileScheme, VectorSource};
use crate::style::value::{FunctionStop, MlStyleValue};

/// Tries to create a [`VectorTileLayer`] from a Maplibre vector source and the style layers that
/// reference it. Returns `None` if the source cannot be used (e.g. no tile URLs available).
pub fn try_create(
    source_name: &str,
    source: &VectorSource,
    layers: &[&MaplibreStyleLayer],
) -> Option<VectorTileLayer> {
    let tile_urls = match source.tiles.as_deref() {
        Some([_, ..]) => source.tiles.clone().unwrap(),
        _ => {
            log::debug!(
                "{UNSUPPORTED} Vector source '{source_name}' has no tile URLs; skipping. \
                 Open a GitHub issue or PR if support is desirable."
            );
            return None;
        }
    };

    let rules = build_rules(layers);
    let background = get_background(layers);
    let style = VectorTileStyle { rules, background };

    let tile_schema = build_tile_schema(source)?;

    VectorTileLayerBuilder::new_rest(move |index| {
        // When multiple URLs are provided they are equivalent mirrors; balance across them
        // using (x + y) mod n, which distributes evenly and is stable per tile.
        let url = &tile_urls[(index.x + index.y).rem_euclid(tile_urls.len() as i32) as usize];
        url.replace("{z}", &index.z.to_string())
            .replace("{x}", &index.x.to_string())
            .replace("{y}", &index.y.to_string())
    })
    .with_tile_schema(tile_schema)
    .with_style(style)
    .build()
    .ok()
}

/// Finds background layer and returns its color.
///
/// Maptiler supports having background layer in any position, and just adds filling of the
/// entire tile. WE don't support this currently, and always put background at the back.
fn get_background(layers: &[&MaplibreStyleLayer]) -> StyleValue<Color> {
    const DEFAULT_TILE_BACKGROUND: StyleValue<Color> = StyleValue::Simple(Color::TRANSPARENT);

    let layer = match layers {
        [] => return DEFAULT_TILE_BACKGROUND,
        [MaplibreStyleLayer::Background(layer), ..] => layer,
        layers => {
            let bg_layer = layers.iter().find_map(|l| {
                if let MaplibreStyleLayer::Background(background) = l {
                    Some(background)
                } else {
                    None
                }
            });

            if let Some(layer) = bg_layer {
                log::debug!(
                    "{UNSUPPORTED} Background layer '{}' is in not the first layer in the list. \
                    This is not yet supported. Background will be applied to the bottom of the tile.",
                    layer.id,
                );

                layer
            } else {
                return DEFAULT_TILE_BACKGROUND;
            }
        }
    };

    let Some(color) = &layer.paint.background_color else {
        return DEFAULT_TILE_BACKGROUND;
    };

    get_color_value(color, layer.paint.background_opacity.as_ref())
        .unwrap_or(DEFAULT_TILE_BACKGROUND)
}

fn get_color_value(
    color: &MlStyleValue<Color>,
    opacity: Option<&MlStyleValue<f64>>,
) -> Option<StyleValue<Color>> {
    let galileo_color = get_galileo_value(color)?;
    let Some(galileo_opacity) = opacity.and_then(get_galileo_value) else {
        return Some(galileo_color);
    };

    match (galileo_color, galileo_opacity) {
        (StyleValue::Simple(c), StyleValue::Simple(o)) => Some(c.with_alpha_float(o).into()),
        (StyleValue::Simple(c), StyleValue::Interpolate(o)) => Some(StyleValue::Interpolate(
            o.map(|opacity| c.with_alpha_float(opacity)),
        )),
        (StyleValue::Simple(c), StyleValue::Steps(o)) => Some(StyleValue::Steps(
            o.map(|opacity| c.with_alpha_float(opacity)),
        )),
        (StyleValue::Interpolate(c), StyleValue::Simple(o)) => Some(StyleValue::Interpolate(
            c.map(|color| color.with_alpha_float(o)),
        )),
        (StyleValue::Steps(c), StyleValue::Simple(o)) => {
            Some(StyleValue::Steps(c.map(|color| color.with_alpha_float(o))))
        }
        _ => {
            log::debug!(
                "{UNSUPPORTED} Color values with both color and opacity interpolation are not yet supported",
            );
            None
        }
    }
}

fn get_galileo_value<T: Copy + Default + std::fmt::Debug>(
    value: &MlStyleValue<T>,
) -> Option<StyleValue<T>>
where
    for<'de> FunctionStop<T>: Deserialize<'de>,
{
    match value {
        MlStyleValue::Literal(v) => Some((*v).into()),
        MlStyleValue::Expression(expr) => {
            log::debug!(
                "{UNSUPPORTED} Expressions of type {:?} are not yet supported",
                expr.operator(),
            );
            None
        }
        MlStyleValue::Function(function) => {
            let steps = function.stops.iter().map(|stop| StepValue {
                basis: stop.input,
                step_value: stop.output,
            });
            let args = InterpolationArgs::Exponential(
                ExponentialInterpolationArgs::new(function.base, steps).ok()?,
            );
            Some(StyleValue::Interpolate(InterpolateExpression::new(
                args,
                OperationBase::Zlevel,
            )))
        }
    }
}

/// Builds a Web Mercator [`TileSchema`] from a vector source's zoom range, scheme, and bounds.
fn build_tile_schema(source: &VectorSource) -> Option<TileSchema> {
    let min_z = source.minzoom as u32;
    let max_z = source.maxzoom as u32;

    let y_direction = match source.scheme {
        TileScheme::Xyz => VerticalDirection::TopToBottom,
        TileScheme::Tms => VerticalDirection::BottomToTop,
    };

    let mut builder = TileSchemaBuilder::web_mercator(min_z..=max_z)
        .rect_tile_size(1024)
        .y_direction(y_direction);

    if let Some(bounds) = source.bounds
        && let Some(rect) = wgs84_bounds_to_mercator(bounds)
    {
        builder = builder.tile_bounds(rect);
    }

    builder.build().ok()
}

/// Converts each supported style layer into a [`StyleRule`], logging unsupported ones.
fn build_rules(layers: &[&MaplibreStyleLayer]) -> Vec<StyleRule> {
    let mut rules = Vec::new();
    for &layer in layers {
        match layer {
            MaplibreStyleLayer::Fill(fill) => {
                if let Some(rule) = fill_rule(fill) {
                    rules.push(rule);
                    log::debug!(
                        "Maplibre layer '{}' of type '{}' is added as a VT style rule",
                        layer.id(),
                        layer.type_name()
                    );
                }
            }
            other => {
                log::debug!(
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
            log::debug!(
                "{UNSUPPORTED} Fill layer '{}' has no source-layer; skipping.",
                fill.id
            );
            return None;
        }
    };

    let fill_color = fill.paint.fill_color?;
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

/// Extracts a number from a paint property that must be a literal JSON number.
/// Logs and returns `None` for expressions or functions; silently returns `None` for absent values.
fn extract_literal_f64(value: &Option<Value>, layer_id: &str, prop: &str) -> Option<f64> {
    match value {
        None => None,
        Some(Value::Number(n)) => n.as_f64(),
        Some(_) => {
            log::debug!(
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

/// Converts WGS84 bounding box `[west, south, east, north]` (degrees) to a Web Mercator [`Rect`]
/// in projected meters, suitable for use with [`TileSchemaBuilder::tile_bounds`].
fn wgs84_bounds_to_mercator(bounds: [f64; 4]) -> Option<Rect> {
    let projection: Box<dyn Projection<InPoint = GeoPoint2d, OutPoint = Point2>> =
        Crs::EPSG3857.get_projection()?;
    let [west, south, east, north] = bounds;
    let sw = projection.project(&GeoPoint2d::latlon(south, west))?;
    let ne = projection.project(&GeoPoint2d::latlon(north, east))?;
    Some(Rect::new(sw.x(), sw.y(), ne.x(), ne.y()))
}
