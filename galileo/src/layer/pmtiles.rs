//! PMTiles support for Galileo.
//!
//! Specification: https://docs.protomaps.com/pmtiles/
//!
//! Example: load a tile from an open PMTiles source
//!
//! ```no_run
//! use pmtiles::{AsyncPmTilesReader, reqwest::Client, TileCoord};
//! use galileo::layer::pmtiles::PmtilesTileLoader;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Client::builder().use_rustls_tls().build()?;
//! let url = "https://demo-bucket.protomaps.com/v4.pmtiles";
//! let reader = AsyncPmTilesReader::new_with_cached_url(pmtiles::NoCache, client, url).await?;
//! let loader = PmtilesTileLoader::new(reader);
//! // Fetch the root tile 0/0/0
//! let _bytes = loader.load(galileo::tile_schema::TileIndex::new(0, 0, 0)).await;
//! # Ok(())
//! # }
//! ```

use std::io::Read;

use bytes::Bytes;
use flate2::read::GzDecoder;
use galileo_mvt::MvtTile;
use log::error;
use pmtiles::{DirectoryCache, TileCoord};

use crate::decoded_image::DecodedImage;
use crate::error::GalileoError;
use crate::layer::raster_tile_layer::RasterTileLoader;
use crate::layer::vector_tile_layer::VectorTileLayer;
use crate::layer::vector_tile_layer::VectorTileLayerBuilder;
use crate::layer::vector_tile_layer::style::VectorTileStyle;
use crate::layer::vector_tile_layer::tile_provider::loader::{TileLoadError, VectorTileLoader};
use crate::platform::PlatformService;
use crate::tile_schema::TileIndex;

// Use directory cache implementations provided by the `pmtiles` crate (e.g. `NoCache`).

/// Tile loader for PMTiles format using an async backend (e.g., HTTP)
pub struct PmtilesTileLoader<B = pmtiles::HttpBackend, C = pmtiles::NoCache> {
    reader: pmtiles::AsyncPmTilesReader<B, C>,
}

impl<B, C> PmtilesTileLoader<B, C>
where
    B: pmtiles::AsyncBackend + Send + Sync,
    C: DirectoryCache + Send + Sync,
{
    /// Creates a new PMTiles tile loader with the given reader
    pub fn new(reader: pmtiles::AsyncPmTilesReader<B, C>) -> Self {
        Self { reader }
    }

    async fn get_tile(&self, index: TileIndex) -> Result<Bytes, GalileoError> {
        let coord = TileCoord::new(index.z as u8, index.x as u32, index.y as u32)
            .ok_or(GalileoError::NotFound)?;

        self.reader
            .get_tile(coord)
            .await
            .map_err(|_| GalileoError::NotFound)?
            .ok_or(GalileoError::NotFound)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl<B, C> RasterTileLoader for PmtilesTileLoader<B, C>
where
    B: pmtiles::AsyncBackend + Send + Sync,
    C: DirectoryCache + Send + Sync + maybe_sync::MaybeSend + maybe_sync::MaybeSync,
{
    async fn load(&self, index: TileIndex) -> Result<DecodedImage, GalileoError> {
        let bytes = self.get_tile(index).await?;
        crate::platform::instance().decode_image(bytes).await
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl<B, C> VectorTileLoader for PmtilesTileLoader<B, C>
where
    B: pmtiles::AsyncBackend + Send + Sync,
    C: DirectoryCache + Send + Sync + maybe_sync::MaybeSend + maybe_sync::MaybeSync,
{
    async fn load(&self, index: TileIndex) -> Result<MvtTile, TileLoadError> {
        let bytes = self
            .get_tile(index)
            .await
            .map_err(|_| TileLoadError::Network)?;

        // Check if this is GZIP compressed data
        let decompressed_bytes = if bytes.len() > 2 && bytes[0..2] == [0x1F, 0x8B] {
            // GZIP compressed data - decompress it
            let mut decoder = GzDecoder::new(&bytes[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| {
                error!("PMTiles: GZIP decompression error: {:?}", e);
                TileLoadError::Decoding
            })?;
            Bytes::from(decompressed)
        } else {
            // Not compressed, use as-is
            bytes
        };

        MvtTile::decode(decompressed_bytes, false).map_err(|e| {
            error!("PMTiles: Vector tile decoding error: {:?}", e);
            TileLoadError::Decoding
        })
    }
}

/// Convenience helper to build a vector tile layer from a PMTiles URL using the given HTTP client and style.
///
/// This keeps the `pmtiles`-specific types inside the crate so examples and applications
/// don't need to depend on the `pmtiles` crate directly.
pub async fn build_vector_layer_from_url(
    client: pmtiles::reqwest::Client,
    url: impl pmtiles::reqwest::IntoUrl,
    tile_schema: crate::TileSchema,
    style: VectorTileStyle,
) -> Result<VectorTileLayer, GalileoError> {
    let reader = pmtiles::AsyncPmTilesReader::new_with_cached_url(pmtiles::NoCache, client, url)
        .await
        .map_err(|_| GalileoError::IO)?;
    let loader = PmtilesTileLoader::new(reader);
    VectorTileLayerBuilder::new_pmtiles(loader, tile_schema)
        .with_style(style)
        .build()
}
