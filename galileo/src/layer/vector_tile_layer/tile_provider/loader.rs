//! Vector tile loader stuff.

use bytes::Bytes;
use galileo_mvt::MvtTile;
use maybe_sync::{MaybeSend, MaybeSync};

use crate::error::GalileoError;
use crate::layer::data_provider::{PersistentCacheController, UrlSource};
use crate::platform::PlatformService;
use crate::tile_schema::TileIndex;

/// Error that can occur when trying to load a vector tile.
pub enum TileLoadError {
    /// Could not connect to the remote server.
    Network,
    /// Tile with the given index does not exist.
    DoesNotExist,
    /// Failed to decode vector tile from the binary data.
    Decoding,
}

/// Loader for vector tiles.
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait VectorTileLoader: MaybeSend + MaybeSync {
    /// Load tile with the given index.
    async fn load(&self, index: TileIndex) -> Result<MvtTile, TileLoadError>;
}

/// Load the tile from the Web.
pub struct WebVtLoader {
    cache: Option<Box<dyn PersistentCacheController<str, Bytes>>>,
    url_source: Box<dyn UrlSource<TileIndex>>,
    offline_mode: bool,
}

impl WebVtLoader {
    /// Create a new instance.
    pub fn new(
        cache: Option<Box<dyn PersistentCacheController<str, Bytes>>>,
        url_source: impl UrlSource<TileIndex> + 'static,
        offline_mode: bool,
    ) -> Self {
        Self {
            cache,
            url_source: Box::new(url_source),
            offline_mode,
        }
    }

    async fn load_raw(&self, url: &str) -> Result<Bytes, TileLoadError> {
        if let Some(data) = self.cache.as_ref().and_then(|cache| cache.get(url)) {
            log::info!("✓ Cache HIT for: {url}");
            return Ok(data);
        }

        if self.offline_mode {
            log::info!("Offline mode: tile not in cache for: {url}");
            return Err(TileLoadError::DoesNotExist);
        }

        log::info!("⬇ Downloading from network: {url}");
        let bytes = crate::platform::instance()
            .load_bytes_from_url(url)
            .await
            .map_err(|err| match err {
                GalileoError::NotFound => {
                    log::warn!("✗ Tile not found (404): {url}");
                    TileLoadError::DoesNotExist
                }
                _ => {
                    log::error!("✗ Network error loading tile: {url}");
                    TileLoadError::Network
                }
            })?;

        log::info!("✓ Downloaded {} bytes from: {url}", bytes.len());

        if let Some(cache) = &self.cache {
            match cache.insert(url, &bytes) {
                Ok(()) => log::info!("✓ Cached tile: {url}"),
                Err(error) => log::info!("✗ Failed to cache tile {url}: {error:?}"),
            }
        }

        Ok(bytes)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl VectorTileLoader for WebVtLoader {
    async fn load(&self, index: TileIndex) -> Result<MvtTile, TileLoadError> {
        let url = (self.url_source)(&index);

        log::trace!("Loading tile {index:?} from url {url}");
        let bytes = self.load_raw(&url).await?;

        log::trace!("Tile {index:?} loaded. Byte size: {}", bytes.len());

        let mvt = MvtTile::decode(bytes, false).map_err(|_| TileLoadError::Decoding)?;

        log::trace!("Tile {index:?} successfully decoded");

        Ok(mvt)
    }
}
