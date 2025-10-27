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
        .with_z_level(3) // z-level in config is too big, so only blue ocean is drawn, so for
                         // testing change it to just 3
        // .with_z_level(config.z_level) // Initial zoom level
        .with_latlon(config.latlon.0, config.latlon.1)
        .build();
    Ok(map)
}

pub fn create_galileo_map_v2(config: &MapInitConfig) -> anyhow::Result<Map> {
    let map = MapBuilder::default()
        .with_latlon(config.latlon.0, config.latlon.1)
        .with_z_level(config.zoom_level)
        .build();
    Ok(map)
}
