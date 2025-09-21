//! View a PMTiles dataset as a map window.
//! Run with: cargo run --example pmtiles --features pmtiles

use galileo::{Lod, Map, MapBuilder, TileSchema};
use galileo_types::cartesian::{Point2, Rect};
use galileo_types::geo::Crs;
use galileo::layer::pmtiles::build_vector_layer_from_url;
use galileo::layer::vector_tile_layer::style::VectorTileStyle;
use pmtiles::reqwest::Client;

#[tokio::main]
async fn main() {
    let tile_schema = tile_schema();
    let style: VectorTileStyle = serde_json::from_str(include_str!("data/pmtiles_style.json"))
        .expect("invalid style json");
    let client = Client::new();
    let url = "https://demo-bucket.protomaps.com/v4.pmtiles";
    let vt_layer = build_vector_layer_from_url(client, url, tile_schema.clone(), style)
        .await
        .expect("failed to create layer");

    let map: Map = MapBuilder::default()
        .with_latlon(37.566, 128.9784)
        .with_z_level(4)
        .with_layer(vt_layer)
        .build();

    galileo_egui::InitBuilder::new(map)
        .init()
        .expect("failed to initialize");
}

fn tile_schema() -> TileSchema {
    // Match the schema used in the main project (1024px tiles, WebMercator)
    const ORIGIN: Point2 = Point2::new(-20037508.342787, 20037508.342787);
    const TOP_RESOLUTION: f64 = 156543.03392800014 / 4.0;

    let mut lods = vec![Lod::new(TOP_RESOLUTION, 0).expect("invalid config")];
    for i in 1..16 {
        lods.push(
            Lod::new(lods[(i - 1) as usize].resolution() / 2.0, i).expect("invalid tile schema"),
        );
    }

    TileSchema {
        origin: ORIGIN,
        bounds: Rect::new(
            -20037508.342787,
            -20037508.342787,
            20037508.342787,
            20037508.342787,
        ),
        lods: lods.into_iter().collect(),
        tile_width: 1024,
        tile_height: 1024,
        y_direction: galileo::tile_schema::VerticalDirection::TopToBottom,
        crs: Crs::EPSG3857,
    }
}
