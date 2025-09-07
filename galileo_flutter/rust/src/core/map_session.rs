use galileo::galileo_types;
use galileo::layer::raster_tile_layer::RasterTileLayerBuilder;
use log::{debug, info, warn};
use parking_lot::Mutex;
use crate::api::api::create_new_session;
use crate::core::galileo_ref::create_galileo_map;
pub use crate::core::pixel_buffer::PixelBuffer;
use crate::core::{WindowlessRenderer, RENDER_TASK_HANDLES, SESSION_COUNTER, TOKIO_RUNTIME};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::api::dart_types::{MapInitConfig, MapSize, RenderMessage};
use crate::core::flutter::pixel_texture::{SharedPixelTextureProvider, SharedSendablePixelTexture};



pub type SessionID = u32;

/// Internal map session that manages the Galileo map with rendering.
pub struct MapSession {
    session_id: SessionID,
    pub map: Arc<Mutex<galileo::Map>>,
    renderer: Arc<Mutex<WindowlessRenderer>>,
    wgpu_pixel_buffer: Arc<Mutex<PixelBuffer>>,
    texture_provider: SharedPixelTextureProvider,
    flutter_sendable_texture: SharedSendablePixelTexture,
    pub engine_handle: i64,
    is_alive: AtomicBool,
}

// Ensure MapSession is Send + Sync for thread safety
unsafe impl Send for MapSession {}
unsafe impl Sync for MapSession {}
impl MapSession {
    
    pub async fn new(
        engine_handle: i64,
        config: MapInitConfig,
        
    ) -> Arc<MapSession>{
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
        
            let flutter_pixel_buffer = Arc::new(Mutex::new(PixelBuffer::new(device, queue, size)));
        
            // Create render loop
        
            // Create texture provider and Flutter texture
            let texture_provider = Arc::new(PixelTextureProvider::new(size));
            let flutter_texture = create_flutter_texture(engine_handle, texture_provider.clone()).await?;
            let flutter_texture = Arc::new(Mutex::new(flutter_texture));
        
            // Create render task communication channel
            let (render_tx, render_rx) = mpsc::unbounded_channel();
            let render_commands = Arc::new(Mutex::new(render_tx));
        
            // Create session
            let session = Arc::new(MapSession {
                session_id,
                map: map.clone(),
                renderer: renderer.clone(),
                wgpu_pixel_buffer: flutter_pixel_buffer.clone(),
                texture_provider: texture_provider.clone(),
                flutter_sendable_texture: flutter_texture.clone(),
                engine_handle,
                is_alive: AtomicBool::new(true),
                render_commands_tx: render_commands,
            });
        
            // Store session
            {
                let mut sessions = SESSIONS.lock();
                sessions.insert(session_id, session.clone());
            }
        
            // Start rendering task
            let render_handle = TOKIO_RUNTIME
                .get()
                .unwrap()
                .spawn(render_task(session.clone(), render_rx));
        
            // Store render task handle
            {
                let mut handles = RENDER_TASK_HANDLES.lock();
                handles.insert(session_id, render_handle);
            }
        
            info!(
                "Map session {} created with full rendering pipeline",
                session_id
            );
        
()
        
    }
    
    pub fn is_alive(&self) -> bool {
        self.is_alive.load(Ordering::SeqCst)
    }
    pub fn mark_alive(&self) {
        self.is_alive.store(true, Ordering::SeqCst);
    }



           /// Renders a single frame for the session.
           pub async fn render_frame(&self) -> anyhow::Result<()> {
               // Render the map to wgpu texture
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
               self.texture_provider.update_pixels(pixels);
           
               // Mark frame available for Flutter
               {
                   let flutter_texture = self.flutter_sendable_texture.lock();
                   flutter_texture
                       .mark_frame_available()
                       .map_err(|e| anyhow::anyhow!("Failed to mark frame available: {:?}", e))?;
               }
           
               Ok(())
           }
           
           /// Resizes the rendering session.
           pub async fn resize(&self, new_size: MapSize) -> anyhow::Result<()> {
               info!(
                   "Resizing session {} to {}x{}",
                   self.session_id, new_size.width, new_size.height
               );
           
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
               self.texture_provider.resize(new_size);
           
               // Trigger render to fill new size
               self.render_frame().await?;
           
               Ok(())
           }
           
           
           
    pub fn terminate(&self) {
        self.is_alive.store(false, Ordering::SeqCst);
        let render_commands = self.render_commands_tx.lock();
        let _ = render_commands.send(RenderMessage::Stop);
    }
}

async fn read_pixels_from_buffer(pixel_buffer: &PixelBuffer) -> anyhow::Result<Vec<u8>> {
    // Use spawn_blocking to handle the async operation safely
    let handle = TOKIO_RUNTIME.get().unwrap().handle().clone();
    let result = tokio::task::spawn_blocking(move || {
        // Get a runtime handle for the blocking context
        handle.block_on(async move {
            let mut buffer = pixel_buffer.lock();
            let pixels = buffer.read_pixels().await?;
            Ok::<Vec<u8>, anyhow::Error>(pixels.to_vec())
        })
    })
    .await;

    result.map_err(|e| anyhow::anyhow!("Join error: {}", e))?
}

/// Updates the session counter and returns a new session ID
fn create_new_session() -> SessionID {
    SESSION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}