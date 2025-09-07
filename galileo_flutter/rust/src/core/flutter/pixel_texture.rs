use galileo::galileo_types;
use galileo::layer::raster_tile_layer::RasterTileLayerBuilder;
use irondash_texture::{BoxedPixelData, PayloadProvider, SendableTexture, SimplePixelData, Texture};
use log::{debug, error, info, warn};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::api::dart_types::{MapInitConfig, MapSize};
use crate::core::galileo_ref::create_galileo_map;
use crate::core::{
    MapSession, PixelBuffer, SessionID, WindowlessRenderer, SESSIONS, TOKIO_RUNTIME
};
use crate::utils::invoke_on_platform_main_thread;

/// Texture pixel provider that implements irondash's PayloadProvider
struct PixelTextureProvider {
    pixel_data: Arc<Mutex<Vec<u8>>>,
    size: Arc<Mutex<MapSize>>,
}

impl PixelTextureProvider {
    fn new(size: MapSize) -> Self {
        let pixel_count = (size.width * size.height * 4) as usize;
        Self {
            pixel_data: Arc::new(Mutex::new(vec![0u8; pixel_count])),
            size: Arc::new(Mutex::new(size)),
        }
    }

    pub fn update_pixels(&self, new_pixels: Vec<u8>) {
        let mut pixels = self.pixel_data.lock();
        *pixels = new_pixels;
    }

    pub fn resize(&self, new_size: MapSize) {
        let mut size = self.size.lock();
        *size = new_size;

        let pixel_count = (new_size.width * new_size.height * 4) as usize;
        let mut pixels = self.pixel_data.lock();
        pixels.clear();
        pixels.resize(pixel_count, 0);
    }
}

impl PayloadProvider<BoxedPixelData> for PixelTextureProvider {
    fn get_payload(&self) -> BoxedPixelData {
        let pixels = self.pixel_data.lock();
        let size = self.size.lock();

        SimplePixelData::new_boxed(size.width as i32, size.height as i32, pixels.clone())
    }
}

pub type SharedPixelTextureProvider = Arc<PixelTextureProvider>;

/// Creates a Flutter texture using irondash with proper provider.
pub async fn create_flutter_texture(
    engine_handle: i64,
    provider: Arc<PixelTextureProvider>,
) -> anyhow::Result<SharedPixelTextureProvider> {
    // Create boxed provider for irondash
    let (sendable_texture, texture_id) =
        invoke_on_platform_main_thread(move || -> anyhow::Result<_> {
            let texture =
                irondash_texture::Texture::new_with_provider(engine_handle, payload_holder)?;
            let texture_id = texture.id();
            Ok((texture.into_sendable_texture(), texture_id))
        })?;

    Ok(texture)
}


pub type SharedSendableTexture<T> = Arc<SendableTexture<T>>;
pub type SharedSendablePixelTexture = SharedSendableTexture<Box<PixelTextureProvider>>;




