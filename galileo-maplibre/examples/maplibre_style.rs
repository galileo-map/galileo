use galileo::MapBuilder;
use galileo_maplibre::MaplibreLayer;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    env_logger::init();

    let map = tokio::runtime::Runtime::new()
        .expect("failed to create tokio runtime")
        .block_on(create_map());

    galileo_egui::InitBuilder::new(map)
        .with_logging(false)
        .init()
        .expect("failed to initialize");
}

async fn create_map() -> galileo::Map {
    let Some(api_key) = std::option_env!("VT_API_KEY") else {
        panic!(
            "Set the MapTiler API key into VT_API_KEY environment variable when building this example"
        );
    };

    let style_json = include_str!("../data/maptiler_fmt.json").replace("{VT_API_KEY}", api_key);
    let layer = MaplibreLayer::from_json(&style_json)
        .await
        .expect("failed to parse style");

    MapBuilder::default()
        .with_latlon(37.566, 128.9784)
        .with_z_level(8)
        .with_layer(layer)
        .build()
}
