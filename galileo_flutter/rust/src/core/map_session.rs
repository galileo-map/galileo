use crate::core::galileo_ref::create_galileo_map;
pub use crate::core::pixel_buffer::PixelBuffer;
use crate::core::{WindowlessRenderer, SESSIONS, SESSION_COUNTER, TOKIO_RUNTIME};
use galileo::galileo_types;
use galileo::layer::raster_tile_layer::RasterTileLayerBuilder;
use log::{debug, error, info, warn};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::api::dart_types::{MapInitConfig, MapSize};
use crate::core::flutter::pixel_texture::{
    create_flutter_texture, PixelPayloadHolder, SharedPixelPayloadHolder,
    SharedSendablePixelTexture,
};

pub type SessionID = u32;

struct FlutterCtx {
    payload_holder: SharedPixelPayloadHolder,
    sendable_texture: SharedSendablePixelTexture,
    texture_id: i64,
}

impl FlutterCtx {
    fn new(engine_handle: i64, size: MapSize) -> anyhow::Result<Self> {
        // Create texture provider and Flutter texture
        let pixel_texture_payload_holder = PixelPayloadHolder::new(size);
        let (sendable_texture, texture_id) =
            create_flutter_texture(engine_handle, pixel_texture_payload_holder.clone())?;
        Ok(FlutterCtx {
            payload_holder: pixel_texture_payload_holder,
            sendable_texture,
            texture_id: texture_id,
        })
    }
}
/// Internal map session that manages the Galileo map with rendering.
pub struct MapSession {
    session_id: SessionID,
    pub map: Arc<Mutex<galileo::Map>>,
    renderer: Arc<Mutex<WindowlessRenderer>>,
    wgpu_pixel_buffer: Arc<Mutex<PixelBuffer>>,
    /// this is optional because we wanna drop this on the platform thread.
    flutter_ctx: Option<FlutterCtx>,
    pub engine_handle: i64,
    is_alive: AtomicBool,
}

// Ensure MapSession is Send + Sync for thread safety
unsafe impl Send for MapSession {}
unsafe impl Sync for MapSession {}
impl MapSession {
    pub async fn new(engine_handle: i64, config: MapInitConfig) -> anyhow::Result<Arc<MapSession>> {
        let session_id = create_new_session();
        // Create windowless renderer
        let renderer_size = config.map_size.as_galileo();
        let renderer = WindowlessRenderer::new(renderer_size)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create renderer: {}", e))?;
        let renderer = Arc::new(Mutex::new(renderer));

        // Create OSM layer for background
        let mut osm = RasterTileLayerBuilder::new_osm()
            .with_file_cache_checked(".tile_cache")
            .build()
            .expect("failed to create layer");

        // If we don't set fade in duration to 0, when the image is first drawn, all tiles will
        // be transparent.
        osm.set_fade_in_duration(Duration::default());
        let size = config.map_size;

        let map = create_galileo_map(&config, osm)?;
        let map = Arc::new(Mutex::new(map));

        // Create pixel buffer for GPU-CPU data transfer
        let device = {
            let renderer_guard = renderer.lock();
            Arc::new(renderer_guard.device().clone())
        };
        let queue = {
            let renderer_guard = renderer.lock();
            Arc::new(renderer_guard.queue().clone())
        };

        let wgpu_pixel_buffer = Arc::new(Mutex::new(PixelBuffer::new(device, queue, size)));

        let flutter_ctx = FlutterCtx::new(engine_handle, size)?;

        let session = Arc::new(MapSession {
            session_id,
            map: map.clone(),
            renderer: renderer.clone(),
            flutter_ctx: Some(flutter_ctx),
            wgpu_pixel_buffer: wgpu_pixel_buffer.clone(),
            engine_handle,
            is_alive: AtomicBool::new(true),
        });

        {
            let mut sessions = SESSIONS.lock();
            sessions.insert(session_id, session.clone());
        }
        Ok(session)
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::SeqCst)
    }
    pub fn mark_alive(&self) {
        self.is_alive.store(true, Ordering::SeqCst);
    }

    fn get_flutter_ctx(&self) -> anyhow::Result<&FlutterCtx> {
        self.flutter_ctx
            .as_ref()
            .ok_or(anyhow::anyhow!("Flutter context not available"))
    }
    
    pub async fn handle_mouse_event(&self) -> anyhow::Result<()>{
        let map = self.map.lock();
        map.set_messenger(messenger);
        
        Ok(())
    }
    
    /// Renders a single frame for the session.
    pub async fn redraw(&self) -> anyhow::Result<()> {
        // Render the map to wgpu texture
        let flutter_ctx = self.get_flutter_ctx()?;
        {
            let mut renderer = self.renderer.lock();
            let map = self.map.lock();

            renderer
                .render_map(&map)
                .map_err(|e| anyhow::anyhow!("Failed to render map: {}", e))?;
        }

        // Copy texture to staging buffer
        let wgpu_texture = {
            let renderer = self.renderer.lock();
            renderer
                .target_texture()
                .ok_or_else(|| anyhow::anyhow!("No target texture available"))?
                .clone()
        };

        let pixels = {
            let mut pixel_buffer = self.wgpu_pixel_buffer.lock();
            pixel_buffer
                .copy_from_texture(&wgpu_texture)
                .map_err(|e| anyhow::anyhow!("Failed to copy texture to buffer: {}", e))?;
            pixel_buffer.read_pixels().await.map(|px| px.to_vec())?
        };

        // Update texture provider
        flutter_ctx.payload_holder.update_pixels(pixels);
        // Mark frame available for Flutter
        flutter_ctx.sendable_texture.mark_frame_available();
        Ok(())
    }

    /// Resizes the rendering session.
    pub async fn resize(&self, new_size: MapSize) -> anyhow::Result<()> {
        info!(
            "Resizing session {} to {}x{}",
            self.session_id, new_size.width, new_size.height
        );
        let flutter_ctx = self.get_flutter_ctx()?;

        // Resize renderer
        {
            let mut renderer = self.renderer.lock();
            let size = galileo_types::cartesian::Size::new(new_size.width, new_size.height);
            renderer
                .resize(size)
                .map_err(|e| anyhow::anyhow!("Failed to resize renderer: {}", e))?;
        }

        // Resize pixel buffer
        {
            let mut pixel_buffer = self.wgpu_pixel_buffer.lock();
            pixel_buffer
                .resize(new_size)
                .map_err(|e| anyhow::anyhow!("Failed to resize pixel buffer: {}", e))?;
        }

        // Resize texture provider
        flutter_ctx.payload_holder.resize(new_size);

        // Trigger render to fill new size
        self.redraw().await?;

        Ok(())
    }
    async fn _draw_no_res(&self){
             self.redraw().await.inspect_err(|err|{
                error!("{err}")
            });
             
    }
    pub fn terminate(&self) {
        self.is_alive.store(false, Ordering::SeqCst);
    }
}

impl galileo::Messenger for MapSession{

    fn request_redraw(&self) {
        TOKIO_RUNTIME.get().unwrap().block_on(
           self._draw_no_res()
            
        )
    }
    
}

/// Updates the session counter and returns a new session ID
fn create_new_session() -> SessionID {
    SESSION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}
