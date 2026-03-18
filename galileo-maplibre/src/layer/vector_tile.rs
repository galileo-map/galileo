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
    StyleRule, VectorTileLineSymbol, VectorTilePolygonSymbol, VectorTileStyle, VectorTileSymbol,
};
use galileo::tile_schema::{TileSchema, TileSchemaBuilder, VerticalDirection};
use serde::Deserialize;

use crate::layer::{UNSUPPORTED, log_unsupported_field};
use crate::style::color::MlColor;
use crate::style::layer::{FillLayer, Layer as MaplibreStyleLayer, LineLayer};
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

    let tile_schema = build_tile_schema(source)?;

    let rules = build_rules(layers, &tile_schema);
    let background = get_background(layers);
    let style = VectorTileStyle { rules, background };

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
    .with_fade_in_duration(Default::default())
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

    get_color_value(
        &layer.paint.background_color,
        &layer.paint.background_opacity,
    )
    .unwrap_or(DEFAULT_TILE_BACKGROUND)
}

fn get_color_value(
    color: &MlStyleValue<MlColor>,
    opacity: &MlStyleValue<f64>,
) -> Option<StyleValue<Color>> {
    let galileo_color = get_galileo_value(color)?;
    let galileo_opacity = get_galileo_value(opacity).unwrap_or(StyleValue::Simple(1.0));

    match (galileo_color, galileo_opacity) {
        (StyleValue::Simple(c), StyleValue::Simple(o)) => Some((*c).with_alpha_float(o).into()),
        (StyleValue::Simple(c), StyleValue::Interpolate(o)) => Some(StyleValue::Interpolate(
            o.map(|opacity| (*c).with_alpha_float(opacity)),
        )),
        (StyleValue::Simple(c), StyleValue::Steps(o)) => Some(StyleValue::Steps(
            o.map(|opacity| (*c).with_alpha_float(opacity)),
        )),
        (StyleValue::Interpolate(c), StyleValue::Simple(o)) => Some(StyleValue::Interpolate(
            c.map(|color| (*color).with_alpha_float(o)),
        )),
        (StyleValue::Steps(c), StyleValue::Simple(o)) => Some(StyleValue::Steps(
            c.map(|color| (*color).with_alpha_float(o)),
        )),
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
fn build_rules(layers: &[&MaplibreStyleLayer], tile_schema: &TileSchema) -> Vec<StyleRule> {
    let mut rules = Vec::new();
    for &layer in layers {
        match layer {
            MaplibreStyleLayer::Background(_) => {
                // Handled by `get_background` function
                continue;
            }
            MaplibreStyleLayer::Fill(fill) => {
                if let Some(rule) = fill_rule(fill, tile_schema) {
                    rules.push(rule);
                }
            }
            MaplibreStyleLayer::Line(line) => {
                if let Some(rule) = line_rule(line, tile_schema) {
                    rules.push(rule);
                }
            }
            other => {
                log::debug!(
                    "{UNSUPPORTED} Maplibre layer type '{}' (id: '{}') inside a vector source \
                     is not yet supported. Open a GitHub issue or PR if support is desirable.",
                    other.type_name(),
                    other.id(),
                );

                continue;
            }
        }

        log::trace!(
            "Maplibre layer '{}' of type '{}' is added as a VT style rule",
            layer.id(),
            layer.type_name()
        );
    }
    rules
}

/// Converts a [`FillLayer`] to a [`StyleRule`], or logs and returns `None` if unsupported.
fn fill_rule(fill: &FillLayer, tile_schema: &TileSchema) -> Option<StyleRule> {
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

    let fill_color = &fill.paint.fill_color;
    let fill_opacity = &fill.paint.fill_opacity;
    let color = get_color_value(fill_color, fill_opacity)?;

    log_unsupported_field!(fill.paint.fill_antialias);
    log_unsupported_field!(fill.paint.fill_outline_color);
    log_unsupported_field!(fill.paint.fill_pattern);
    log_unsupported_field!(fill.paint.fill_translate);
    log_unsupported_field!(fill.paint.fill_translate_anchor);
    log_unsupported_field!(fill.paint.fill_emissive_strength);

    let min_resolution = fill
        .maxzoom
        .and_then(|lod| tile_schema.lod_resolution(lod.round() as u32));
    let max_resolution = fill
        .minzoom
        .and_then(|lod| tile_schema.lod_resolution(lod.round() as u32));
    let filter = fill.filter.as_ref().and_then(|v| v.to_galileo_expr());

    Some(StyleRule {
        layer_name: Some(source_layer),
        symbol: VectorTileSymbol::Polygon(VectorTilePolygonSymbol { fill_color: color }),
        min_resolution,
        max_resolution,
        filter: filter.map(Into::into),
    })
}

/// Converts a [`LineLayer`] to a [`StyleRule`], or logs and returns `None` if unsupported.
fn line_rule(line: &LineLayer, tile_schema: &TileSchema) -> Option<StyleRule> {
    if line.paint.line_dasharray.is_some() {
        log::debug!(
            "{UNSUPPORTED} Line dasharray is not supported yet; skipping layer {}",
            line.id
        );
        return None;
    }

    log_unsupported_field!(line.paint.line_blur);
    log_unsupported_field!(line.paint.line_gap_width);
    log_unsupported_field!(line.paint.line_gradient);
    log_unsupported_field!(line.paint.line_pattern);
    log_unsupported_field!(line.paint.line_translate);
    log_unsupported_field!(line.paint.line_translate_anchor);
    log_unsupported_field!(line.paint.line_emissive_strength);
    log_unsupported_field!(line.paint.line_offset);

    let source_layer = match &line.source_layer {
        Some(l) => l.clone(),
        None => {
            log::debug!(
                "{UNSUPPORTED} Line layer '{}' has no source-layer; skipping.",
                line.id
            );
            return None;
        }
    };

    let stroke_color = &line.paint.line_color;
    let stroke_opacity = &line.paint.line_opacity;
    let color = get_color_value(stroke_color, stroke_opacity).unwrap_or(Color::TRANSPARENT.into());
    let stroke_width = &line.paint.line_width;
    let width = get_galileo_value(stroke_width).unwrap_or(1.0.into());

    let min_resolution = line
        .maxzoom
        .and_then(|lod| tile_schema.lod_resolution(lod.round() as u32));
    let max_resolution = line
        .minzoom
        .and_then(|lod| tile_schema.lod_resolution(lod.round() as u32));
    let filter = line.filter.as_ref().and_then(|v| v.to_galileo_expr());

    Some(StyleRule {
        layer_name: Some(source_layer),
        symbol: VectorTileSymbol::Line(VectorTileLineSymbol {
            width,
            stroke_color: color,
        }),
        min_resolution,
        max_resolution,
        filter: filter.map(Into::into),
    })
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
