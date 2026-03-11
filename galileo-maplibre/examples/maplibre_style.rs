use galileo::MapBuilder;
use galileo_maplibre::MaplibreLayer;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    run()
}

pub(crate) fn run() {
    galileo_egui::InitBuilder::new(create_map())
        .init()
        .expect("failed to initialize");
}

fn create_map() -> galileo::Map {
    let style_json = include_str!("../data/maptiler_fmt.json");
    let layer = MaplibreLayer::from_json(style_json).expect("failed to parse style");

    MapBuilder::default()
        .with_latlon(37.566, 128.9784)
        .with_z_level(8)
        .with_layer(layer)
        .build()
}
