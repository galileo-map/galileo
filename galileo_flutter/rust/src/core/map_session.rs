use crate::core::galileo_ref::create_galileo_map_v2;
pub use crate::core::pixel_buffer::PixelBuffer;
use crate::core::{WindowlessRenderer, SESSIONS, SESSION_COUNTER, TOKIO_RUNTIME};
use crate::utils::invoke_on_platform_main_thread;
use anyhow::anyhow;
use galileo::{DummyMessenger, galileo_types};
use log::{debug, error, info};
use parking_lot::RwLock;
use tokio::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::api::dart_types::{MapInitConfig, MapSize, MapViewport};
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
    requires_redraw: AtomicBool,
    redraw_scheduled: AtomicBool,
}

// Ensure MapSession is Send + Sync for thread safety
unsafe impl Send for MapSession {}
unsafe impl Sync for MapSession {}

#[derive(Clone)]
struct SessionMessenger(Arc<MapSession>);

impl galileo::Messenger for SessionMessenger {
    fn request_redraw(&self) {
        self.0.requires_redraw.store(true, Ordering::Relaxed);

        if self.0.redraw_scheduled.swap(true, Ordering::Acquire) {
            return;
        }

        let session = self.0.clone();
        if let Some(runtime) = TOKIO_RUNTIME.get() {
            let _ = runtime.spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(16)).await; // throttle to ~60fps
                    if !session.requires_redraw.swap(false, Ordering::Relaxed) {
                        session.redraw_scheduled.store(false, Ordering::Release);
                        break;
                    }

                    session._draw_no_res().await;
                }
            });
        }
    }
}

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
        // let mut osm = RasterTileLayerBuilder::new_osm()
        //     // .with_file_cache_checked(".tile_cache")
        //     .build()
        //     .expect("failed to create layer");

        // // If we don't set fade in duration to 0, when the image is first drawn, all tiles will
        // // be transparent.
        // osm.set_fade_in_duration(Duration::default());
        let size = config.map_size;

        // let map = create_galileo_map(&config, osm)?;
        let map = create_galileo_map_v2(&config)?;
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
            requires_redraw: AtomicBool::new(false),
            redraw_scheduled: AtomicBool::new(false),
        });
        // set session as message callback for galileo
        {
            let messenger = SessionMessenger(session.clone());

            let mut map = map.lock().await;
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

    pub fn get_flutter_texture_id(&self) -> Option<i64> {
        Some(self.flutter_ctx.read().as_ref()?.texture_id)
    }

    pub async fn add_layer(&self, mut layer: impl galileo::layer::Layer + 'static) {
        if let Some(session) = SESSIONS.lock().get(&self.session_id).cloned() {
            let messenger: SessionMessenger = SessionMessenger(session);
            layer.set_messenger(Box::new(messenger));
        }

        let mut map = self.map.lock().await;
        map.layers_mut().push(layer);
        map.redraw();
    }

    /// Renders a single frame for the session.
    pub async fn redraw(&self) -> anyhow::Result<()> {
        let (payload_holder, sendable_texture) = {
            let flctx = self.flutter_ctx.read();
            let flutter_ctx = flctx
                .as_ref()
                .ok_or(anyhow!("flutter context not available"))?;
            (flutter_ctx.payload_holder.clone(), flutter_ctx.sendable_texture.clone())
        };

        let is_first_render = self.is_first_render.swap(false, Ordering::Relaxed);

        if is_first_render {
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }

        let pixels = {
            let renderer = self.renderer.lock().await;
            let mut map = self.map.lock().await;

            map.animate();

            // check size changed
            let renderer_size = renderer.size().cast();
            if map.view().size() != renderer_size {
                map.set_size(renderer_size);
            }

            debug!(
                "Rendering map size: {:?} to surface size: {:?}",
                map.view().size(),
                renderer.size()
            );
            debug!("Map view is: {:?}", map.view());
            map.load_layers();
            renderer.render(&map).await
        };

        // Update texture provider
        payload_holder.update_pixels(pixels);
        // Mark frame available for Flutter
        sendable_texture.mark_frame_available();
        Ok(())
    }

    pub async fn get_viewport(&self) -> Option<MapViewport> {
        self.map
            .lock()
            .await
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
        let size = galileo_types::cartesian::Size::new(new_size.width, new_size.height);

        // Resize renderer
        {
            let mut renderer = self.renderer.lock().await;
            renderer
                .resize(size)
                .map_err(|e| anyhow::anyhow!("Failed to resize renderer: {}", e))?;
        }

        // Resize pixel buffer
        {
            let mut map = self.map.lock().await;
            map.set_size(size.cast::<f64>());
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
        
        // clear all layers
        {
            let mut map = self.map.lock().await;
            for layer in map.layers_mut().iter_mut() {
                layer.set_messenger(Box::new(DummyMessenger {}));
            }
            map.set_messenger(None::<DummyMessenger>);
            map.layers_mut().clear();
        }
        
        tokio::time::sleep(Duration::from_millis(500)).await;

        let flctx = self.flutter_ctx.write().take();
        if let Some(ctx) = flctx {
            let _ = Arc::strong_count(&ctx.payload_holder);
            let _ = Arc::strong_count(&ctx.sendable_texture);

            invoke_on_platform_main_thread(move || {
                drop(ctx);
            });
        }
    }
}
/// Updates the session counter and returns a new session ID
fn create_new_session() -> SessionID {
    SESSION_COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}
