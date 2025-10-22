use crate::core::galileo_ref::create_galileo_map;
pub use crate::core::pixel_buffer::PixelBuffer;
use crate::core::{WindowlessRenderer, SESSIONS, SESSION_COUNTER, TOKIO_RUNTIME};
use crate::utils::invoke_on_platform_main_thread;
use anyhow::anyhow;
use galileo::galileo_types;
use galileo::layer::raster_tile_layer::RasterTileLayerBuilder;
use log::{debug, error, info, trace, warn};
use parking_lot::Mutex;
use parking_lot::{RwLock, RwLockReadGuard};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::api::dart_types::{LayerConfig, MapInitConfig, MapSize, MapViewport};
use crate::core::flutter::pixel_texture::{
    create_flutter_texture, PixelPayloadHolder, SharedPixelPayloadHolder,
    SharedSendablePixelTexture,
};

pub type SessionID = u32;

struct FlutterCtx {
    payload_holder: SharedPixelPayloadHolder,
    sendable_texture: SharedSendablePixelTexture,
    pub texture_id: i64,
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
    pub session_id: SessionID,
    pub map: Arc<Mutex<galileo::Map>>,
    renderer: Arc<Mutex<WindowlessRenderer>>,
    /// this is optional because we wanna drop this on the platform thread.
    flutter_ctx: RwLock<Option<FlutterCtx>>,
    pub engine_handle: i64,
    is_alive: AtomicBool,
    pub controller: galileo::control::MapController,
    is_first_render: AtomicBool,
    last_rendered_time: Mutex<Option<Instant>>,
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
            // .with_file_cache_checked(".tile_cache")
            .build()
            .expect("failed to create layer");

        // If we don't set fade in duration to 0, when the image is first drawn, all tiles will
        // be transparent.
        osm.set_fade_in_duration(Duration::default());
        let size = config.map_size;

        let map = create_galileo_map(&config, osm)?;
        let map = Arc::new(Mutex::new(map));

        let flutter_ctx = FlutterCtx::new(engine_handle, size)?;

        let session = Arc::new(MapSession {
            session_id,
            map: map.clone(),
            renderer: renderer.clone(),
            flutter_ctx: RwLock::new(Some(flutter_ctx)),
            engine_handle,
            is_alive: AtomicBool::new(true),
            controller: galileo::control::MapController::default(),
            is_first_render: AtomicBool::new(true),
            last_rendered_time: Mutex::new(None),
        });
        // set session as message callback for galileo
        {
            #[derive(Clone)]
            struct _SessionWrapper(Arc<MapSession>);

            impl galileo::Messenger for _SessionWrapper {
                fn request_redraw(&self) {
                    let session = self.0.clone();

                    // spawn in a separate thread
                    std::thread::spawn(move || {
                        TOKIO_RUNTIME.get().unwrap().block_on(async move {
                            session._draw_no_res().await;
                        });
                    });
                }
            }

            let messenger = _SessionWrapper(session.clone());

            let mut map = map.lock();
            for layer in map.layers_mut().iter_mut() {
                layer.set_messenger(Box::new(messenger.clone()));
            }

            map.set_messenger(Some(messenger));
        }

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

    /// Checks if we can render the map to avoid unnecessary re-renders.
    pub fn can_render(&self) -> bool {
        const SKIP_RENDER_INTERVAL: Duration = Duration::from_millis(16); // ~60fps
        
        let mut last_time = self.last_rendered_time.lock();
        match *last_time {
            None => {
                *last_time = Some(Instant::now());
                true
            }
            Some(last) => {
                let elapsed = last.elapsed();
                if elapsed >= SKIP_RENDER_INTERVAL {
                    *last_time = Some(Instant::now());
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn get_flutter_texture_id(&self) -> Option<i64> {
        Some(self.flutter_ctx.read().as_ref()?.texture_id)
    }

    pub fn add_layer(&self, layer: impl galileo::layer::Layer + 'static) {
        let mut map = self.map.lock();
        map.layers_mut().push(layer);
        map.redraw();
    }

    /// Renders a single frame for the session.
    pub async fn redraw(&self) -> anyhow::Result<()> {
        // Render the map to wgpu texture
        trace!("map session request redraw was called");
        let flctx = self.flutter_ctx.read();
        let flutter_ctx = flctx
            .as_ref()
            .ok_or(anyhow!("flutter context not available"))?;

        let is_first_render = self.is_first_render.swap(false, Ordering::Relaxed);

        let pixels = {
            let mut renderer = self.renderer.lock();
            let mut map = self.map.lock();
            
            map.animate();
            
            // check size changed
            let renderer_size = renderer.size().cast();
            if map.view().size() != renderer_size {
                map.set_size(renderer_size);
            }

            debug!("Rendering map size: {:?} to surface size: {:?}", map.view().size(), renderer.size());
            debug!("Map view is: {:?}", map.view());
            map.load_layers();
            if is_first_render {
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
            renderer.render(&map).await
        };

        // Update texture provider
        flutter_ctx.payload_holder.update_pixels(pixels);
        // Mark frame available for Flutter
        flutter_ctx.sendable_texture.mark_frame_available();
        Ok(())
    }

    pub fn get_viewport(&self) -> Option<MapViewport> {
        self.map
            .lock()
            .view()
            .get_bbox()
            .as_ref()
            .map(MapViewport::from_rect)
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
            todo!("resize")
        }
        let flctx = self.flutter_ctx.read();
        let flutter_ctx = flctx
            .as_ref()
            .ok_or(anyhow!("flutter context not available"))?;

        // Resize texture provider
        flutter_ctx.payload_holder.resize(new_size);

        // Trigger render to fill new size
        self.redraw().await?;

        Ok(())
    }
    async fn _draw_no_res(&self) {
        self.redraw().await.inspect_err(|err| error!("{err}"));
    }
    pub async fn terminate(self: Arc<Self>) {
        self.is_alive.store(false, Ordering::SeqCst);
        let max_retries = 10;
        let mut retries = 0;

        while retries < max_retries {
            let self_clone = self.clone();
            if invoke_on_platform_main_thread(move || {
                // drop the flutter texture in the platform thread
                let mut flctx = self_clone.flutter_ctx.write();
                if let Some(ctx) = flctx.take() {
                    let mut ref_count = Arc::strong_count(&ctx.payload_holder);
                    ref_count += Arc::strong_count(&ctx.sendable_texture);
                    info!("flutter ref count sum: {}", ref_count);
                    if ref_count == 2 {
                        drop(ctx);
                    }
                    return true;
                }

                return false;
            }) {
                return;
            } else {
                retries += 1;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}
/// Updates the session counter and returns a new session ID
fn create_new_session() -> SessionID {
    SESSION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}
