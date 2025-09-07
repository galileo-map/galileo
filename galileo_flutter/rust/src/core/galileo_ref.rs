use galileo::{
    layer::{raster_tile_layer::RasterTileLayerBuilder, Layer},
    Map, MapBuilder,
};

use crate::api::dart_types::{MapInitConfig, MapSize};

/// Creates a Galileo map with default OSM layer.
pub fn create_galileo_map(
    config: &MapInitConfig,
    layer: impl Layer + 'static,
) -> anyhow::Result<Map> {
    // Set initial viewport (center on world)
    let map = MapBuilder::default()
        .with_latlon(0.0, 0.0) // Center on equator/prime meridian
        .with_layer(layer)
        .with_z_level(config.zoom_level) // Initial zoom level
        .with_latlon(config.latlon.0, config.latlon.1)
        .build();
    Ok(map)
}
