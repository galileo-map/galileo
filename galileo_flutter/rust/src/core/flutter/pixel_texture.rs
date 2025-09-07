use galileo::galileo_types;
use galileo::layer::raster_tile_layer::RasterTileLayerBuilder;
use irondash_texture::{
    BoxedPixelData, PayloadProvider, PixelDataProvider, SendableTexture, SimplePixelData, Texture,
};
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
    MapSession, PixelBuffer, SessionID, WindowlessRenderer, SESSIONS, TOKIO_RUNTIME,
};
use crate::utils::invoke_on_platform_main_thread;

/// medium that we will use to set pixel data
pub struct PixelPayloadHolder {
    pixel_data: Arc<Mutex<Vec<u8>>>,
    size: Arc<Mutex<MapSize>>,
}

impl PixelPayloadHolder {
    pub fn new(size: MapSize) -> Arc<Self> {
        let pixel_count = (size.width * size.height * 4) as usize;
        Arc::new(Self {
            pixel_data: Arc::new(Mutex::new(vec![0u8; pixel_count])),
            size: Arc::new(Mutex::new(size)),
        })
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

impl PayloadProvider<BoxedPixelData> for PixelPayloadHolder {
    fn get_payload(&self) -> BoxedPixelData {
        let pixels = self.pixel_data.lock();
        let size = self.size.lock();

        SimplePixelData::new_boxed(size.width as i32, size.height as i32, pixels.clone())
    }
}

pub type SharedPixelPayloadHolder = Arc<PixelPayloadHolder>;

/// Creates a Flutter texture using irondash with proper provider.
pub fn create_flutter_texture(
    engine_handle: i64,
    payload_holder: Arc<PixelPayloadHolder>,
) -> anyhow::Result<(SharedSendablePixelTexture, i64)> {
    // Create boxed provider for irondash
    let (sendable_texture, texture_id) =
        invoke_on_platform_main_thread(move || -> anyhow::Result<_> {
            let texture =
                irondash_texture::Texture::new_with_provider(engine_handle, payload_holder)?;
            let texture_id = texture.id();
            Ok((texture.into_sendable_texture(), texture_id))
        })?;

    Ok((sendable_texture, texture_id))
}

pub type SharedSendableTexture<T> = Arc<SendableTexture<T>>;
pub type SharedSendablePixelTexture = SharedSendableTexture<Box<dyn PixelDataProvider>>;
