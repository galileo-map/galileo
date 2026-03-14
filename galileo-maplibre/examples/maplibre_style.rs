use galileo::MapBuilder;
use galileo_maplibre::MaplibreLayer;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let style_path = std::env::args()
        .nth(1)
        .unwrap_or("galileo-maplibre/data/maptiler_fmt.json".to_string());

    env_logger::init();

    let map = tokio::runtime::Runtime::new()
        .expect("failed to create tokio runtime")
        .block_on(create_map(&style_path));

    galileo_egui::InitBuilder::new(map)
        .with_logging(false)
        .init()
        .expect("failed to initialize");
}

async fn create_map(style_path: &str) -> galileo::Map {
    let Some(api_key) = std::option_env!("VT_API_KEY") else {
        panic!(
            "Set the MapTiler API key into VT_API_KEY environment variable when building this example"
        );
    };

    let style_json = std::fs::read_to_string(style_path)
        .unwrap_or_else(|err| panic!("failed to load style json file '{style_path}': {err}"));
    let layer = MaplibreLayer::from_json(&style_json.replace("{VT_API_KEY}", api_key))
        .await
        .expect("failed to parse style");

    MapBuilder::default()
        .with_latlon(37.566, 128.9784)
        .with_z_level(8)
        .with_layer(layer)
        .build()
}
