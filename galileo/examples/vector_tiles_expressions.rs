//! This example shows how to create and work with vector
//! tile layers with style strings containing interpolate and step like expressions

use std::sync::Arc;

use egui::FontDefinitions;
use galileo::expr::{
    ControlPoint, CubicBezierInterpolation, ExponentialInterpolation, Expr, LinearInterpolation,
};
use galileo::layer::VectorTileLayer;
use galileo::layer::data_provider::remove_parameters_modifier;
use galileo::layer::vector_tile_layer::VectorTileLayerBuilder;
use galileo::layer::vector_tile_layer::style::{
    StyleRule, VectorTilePolygonSymbol, VectorTileStyle, VectorTileSymbol,
};
use galileo::render::text::RustybuzzRasterizer;
use galileo::render::text::text_service::TextService;
use galileo::tile_schema::{TileIndex, TileSchema, TileSchemaBuilder};
use galileo::{Color, MapBuilder};
use galileo_egui::{EguiMap, EguiMapState};
use parking_lot::RwLock;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    run()
}

struct App {
    map: EguiMapState,
    layer: Arc<RwLock<VectorTileLayer>>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            EguiMap::new(&mut self.map).show_ui(ui);
        });

        egui::Window::new("Buttons")
            .title_bar(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Linear Interpolation").clicked() {
                        self.set_style(with_overlay_rule(linear_interpolation_style()));
                    }
                    if ui.button("Exponential Interpolation").clicked() {
                        self.set_style(with_overlay_rule(exponential_interpolation_style()));
                    }
                    if ui.button("Cubic Interpolation").clicked() {
                        self.set_style(with_overlay_rule(cubic_interpolation_style()));
                    }
                });
            });
    }
}

fn with_overlay_rule(overlay: StyleRule) -> VectorTileStyle {
    let mut base: VectorTileStyle =
        serde_json::from_str(include_str!("data/vt_style.json")).expect("invalid style json");
    base.rules = vec![overlay];
    base
}

impl App {
    fn new(egui_map_state: EguiMapState, layer: Arc<RwLock<VectorTileLayer>>) -> Self {
        let fonts = FontDefinitions::default();
        let provider = RustybuzzRasterizer::default();

        let text_service = TextService::initialize(provider);
        for font in fonts.font_data.values() {
            text_service.load_font(Arc::new(font.font.to_vec()));
        }

        Self {
            map: egui_map_state,
            layer,
        }
    }

    fn set_style(&mut self, style: VectorTileStyle) {
        let mut layer = self.layer.write();
        if style != *layer.style() {
            layer.update_style(style);
            self.map.request_redraw();
        }
    }
}

pub(crate) fn run() {
    let Some(api_key) = std::option_env!("VT_API_KEY") else {
        panic!("Set the MapTiler API key into VT_API_KEY library when building this example");
    };

    let style =
        serde_json::from_str(include_str!("data/vt_style.json")).expect("invalid style json");
    let layer = VectorTileLayerBuilder::new_rest(move |&index: &TileIndex| {
        format!(
            "https://api.maptiler.com/tiles/v3-openmaptiles/{z}/{x}/{y}.pbf?key={api_key}",
            z = index.z,
            x = index.x,
            y = index.y
        )
    })
    .with_style(style)
    .with_tile_schema(tile_schema())
    .with_file_cache_modifier_checked(".tile_cache", Box::new(remove_parameters_modifier))
    .with_attribution(
        "© MapTiler© OpenStreetMap contributors".to_string(),
        "https://www.maptiler.com/copyright/".to_string(),
    )
    .build()
    .expect("failed to create layer");

    let layer = Arc::new(RwLock::new(layer));

    let map = MapBuilder::default().with_layer(layer.clone()).build();
    galileo_egui::InitBuilder::new(map)
        .with_app_builder(|egui_map_state, _| Box::new(App::new(egui_map_state, layer)))
        .init()
        .expect("failed to initialize");
}

#[allow(clippy::unwrap_used)]
fn linear_interpolation_style() -> StyleRule {
    StyleRule {
        layer_name: None,
        max_resolution: None,
        min_resolution: None,
        filter: None,
        symbol: VectorTileSymbol::Polygon(VectorTilePolygonSymbol {
            fill_color: Expr::InterpolateLinear(Box::new(LinearInterpolation {
                input: Expr::Zoom,
                control_points: vec![
                    ControlPoint {
                        input: 2.0.into(),
                        output: Color::try_from_hex("#81C4ec").unwrap().into(),
                    },
                    ControlPoint {
                        input: 5.0.into(),
                        output: Color::try_from_hex("#29546d").unwrap().into(),
                    },
                    ControlPoint {
                        input: 8.0.into(),
                        output: Color::try_from_hex("#3d835c").unwrap().into(),
                    },
                ],
            }))
            .into(),
        }),
    }
}

#[allow(clippy::unwrap_used)]
fn exponential_interpolation_style() -> StyleRule {
    StyleRule {
        layer_name: None,
        max_resolution: None,
        min_resolution: None,
        filter: None,
        symbol: VectorTileSymbol::Polygon(VectorTilePolygonSymbol {
            fill_color: Expr::InterpolateExp(Box::new(ExponentialInterpolation {
                base: 2.0,
                input: Expr::Zoom,
                control_points: vec![
                    ControlPoint {
                        input: 2.0.into(),
                        output: Color::try_from_hex("#81C4ec").unwrap().into(),
                    },
                    ControlPoint {
                        input: 5.0.into(),
                        output: Color::try_from_hex("#29546d").unwrap().into(),
                    },
                    ControlPoint {
                        input: 8.0.into(),
                        output: Color::try_from_hex("#3d835c").unwrap().into(),
                    },
                ],
            }))
            .into(),
        }),
    }
}

#[allow(clippy::unwrap_used)]
fn cubic_interpolation_style() -> StyleRule {
    StyleRule {
        layer_name: None,
        max_resolution: None,
        min_resolution: None,
        filter: None,
        symbol: VectorTileSymbol::Polygon(VectorTilePolygonSymbol {
            fill_color: Expr::InterpolateCubicBezier(Box::new(CubicBezierInterpolation {
                curve_params: [0.25, 0.0, 0.75, 1.0],
                input: Expr::Zoom,
                control_points: vec![
                    ControlPoint {
                        input: 2.0.into(),
                        output: Color::try_from_hex("#81C4ec").unwrap().into(),
                    },
                    ControlPoint {
                        input: 5.0.into(),
                        output: Color::try_from_hex("#29546d").unwrap().into(),
                    },
                    ControlPoint {
                        input: 8.0.into(),
                        output: Color::try_from_hex("#3d835c").unwrap().into(),
                    },
                ],
            }))
            .into(),
        }),
    }
}

fn tile_schema() -> TileSchema {
    TileSchemaBuilder::web_mercator(2..16)
        .rect_tile_size(1024)
        .build()
        .expect("invalid tile schema")
}
