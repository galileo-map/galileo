//! Main API for Galileo Flutter integration with texture rendering.
//!
//! This module provides the interface between Dart and Rust for
//! managing Galileo maps in Flutter applications with real texture rendering.

use flutter_rust_bridge::frb;
use galileo::galileo_types::cartesian::Size;
use galileo::galileo_types::geo::impls::GeoPoint2d;
use galileo::galileo_types::geo::{GeoPoint, NewGeoPoint};
use galileo::layer::raster_tile_layer::RasterTileLayerBuilder;
use galileo::{Map, MapBuilder};
use irondash_texture::{BoxedPixelData, PayloadProvider, SimplePixelData, Texture};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::api::dart_types::*;
use crate::core::pixel_buffer::PixelBuffer;
use crate::core::render_loop::RenderLoop;
use crate::core::windowless_renderer::WindowlessRenderer;
use crate::core::{IS_INITIALIZED, TOKIO_RUNTIME};

type SessionID = i64;

lazy_static::lazy_static! {
    static ref SESSION_COUNTER: Mutex<SessionID> = Mutex::new(0);
    static ref SESSIONS: Mutex<HashMap<SessionID, Arc<MapSession>>> = Mutex::new(HashMap::new());
    static ref RENDER_TASK_HANDLES: Mutex<HashMap<i64, tokio::task::JoinHandle<()>>> = Mutex::new(HashMap::new());
}

/// Internal map session that manages the Galileo map with rendering.
struct MapSession {
    session_id: SessionID,
    map: Arc<Mutex<Map>>,
    renderer: Arc<Mutex<WindowlessRenderer>>,
    render_loop: Arc<RenderLoop>,
    pixel_buffer: Arc<Mutex<PixelBuffer>>,
    texture_provider: Arc<TexturePixelProvider>,
    flutter_texture: Arc<Mutex<Texture<BoxedPixelData>>>,
    texture_id: i64,
    engine_handle: i64,
    is_alive: Arc<Mutex<bool>>,
    render_commands: Arc<Mutex<mpsc::UnboundedSender<RenderMessage>>>,
}

/// Messages for the rendering task
#[derive(Debug, Clone)]
enum RenderMessage {
    RenderFrame,
    Resize(MapSize),
    UpdateMap,
    Stop,
}

/// Texture pixel provider that implements irondash's PayloadProvider
struct TexturePixelProvider {
    pixel_data: Arc<Mutex<Vec<u8>>>,
    size: Arc<Mutex<MapSize>>,
}

impl TexturePixelProvider {
    fn new(size: MapSize) -> Self {
        let pixel_count = (size.width * size.height * 4) as usize;
        Self {
            pixel_data: Arc::new(Mutex::new(vec![0u8; pixel_count])),
            size: Arc::new(Mutex::new(size)),
        }
    }

    fn update_pixels(&self, new_pixels: Vec<u8>) {
        let mut pixels = self.pixel_data.lock().unwrap();
        *pixels = new_pixels;
    }

    fn resize(&self, new_size: MapSize) {
        let mut size = self.size.lock().unwrap();
        *size = new_size;

        let pixel_count = (new_size.width * new_size.height * 4) as usize;
        let mut pixels = self.pixel_data.lock().unwrap();
        pixels.clear();
        pixels.resize(pixel_count, 0);
    }
}

impl PayloadProvider<BoxedPixelData> for TexturePixelProvider {
    fn get_payload(&self) -> BoxedPixelData {
        let pixels = self.pixel_data.lock().unwrap();
        let size = self.size.lock().unwrap();

        SimplePixelData::new_boxed(size.width as i32, size.height as i32, pixels.clone())
    }
}

// Ensure MapSession is Send + Sync for thread safety
unsafe impl Send for MapSession {}
unsafe impl Sync for MapSession {}

#[frb(init)]
pub fn init_galileo_flutter() {
    flutter_rust_bridge::setup_default_user_utils();
}

/// Initialize the Galileo Flutter plugin with FFI pointer for irondash
pub fn galileo_flutter_init(ffi_ptr: i64) {
    if IS_INITIALIZED.load(Ordering::SeqCst) {
        return;
    }

    // Initialize irondash FFI
    irondash_dart_ffi::irondash_init_ffi(ffi_ptr as *mut std::ffi::c_void);

    info!("Galileo Flutter plugin initialized with FFI and texture support");
    IS_INITIALIZED.store(true, Ordering::SeqCst);
}

/// Updates the session counter and returns a new session ID
pub fn create_new_session() -> i64 {
    let mut session_counter = SESSION_COUNTER.lock().unwrap();
    *session_counter += 1;
    *session_counter
}

/// Creates a new Galileo map session with full texture rendering.
pub fn create_new_galileo_map(
    session_id: i64,
    engine_handle: i64,
    size: MapSize,
    config: RenderConfig,
) -> anyhow::Result<i64> {
    info!(
        "Creating Galileo map with full rendering for session {}, engine {}, size {}x{}",
        session_id, engine_handle, size.width, size.height
    );

    let texture_id = session_id * 1000;

    TOKIO_RUNTIME.get().unwrap().handle().block_on(async {
        create_map_session_async(session_id, engine_handle, size, config).await
    })?;

    // Return texture ID immediately
    Ok(texture_id)
}

/// Async function to create the actual map session with full rendering.
async fn create_map_session_async(
    session_id: i64,
    engine_handle: i64,
    size: MapSize,
    config: RenderConfig,
) -> anyhow::Result<()> {
    // Create windowless renderer
    let renderer_size = Size::new(size.width, size.height);
    let renderer = WindowlessRenderer::new(renderer_size)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create renderer: {}", e))?;
    let renderer = Arc::new(Mutex::new(renderer));

    // Create Galileo map with OSM layer
    let map = create_galileo_map_with_layers()?;
    let map = Arc::new(Mutex::new(map));

    // Create pixel buffer for GPU-CPU data transfer
    let device = {
        let renderer_guard = renderer.lock().unwrap();
        Arc::new(renderer_guard.device().clone())
    };
    let queue = {
        let renderer_guard = renderer.lock().unwrap();
        Arc::new(renderer_guard.queue().clone())
    };
    let pixel_buffer = Arc::new(Mutex::new(PixelBuffer::new(device, queue, size)));

    // Create render loop
    let render_loop = Arc::new(RenderLoop::new(config));

    // Create texture provider and Flutter texture
    let texture_provider = Arc::new(TexturePixelProvider::new(size));
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
        render_loop: render_loop.clone(),
        pixel_buffer: pixel_buffer.clone(),
        texture_provider: texture_provider.clone(),
        flutter_texture: flutter_texture.clone(),
        texture_id: session_id * 1000,
        engine_handle,
        is_alive: Arc::new(Mutex::new(true)),
        render_commands,
    });

    // Store session
    {
        let mut sessions = SESSIONS.lock().unwrap();
        sessions.insert(session_id, session.clone());
    }

    // Start render loop
    render_loop
        .start()
        .map_err(|e| anyhow::anyhow!("Failed to start render loop: {}", e))?;

    // Start rendering task
    let render_handle = TOKIO_RUNTIME
        .get()
        .unwrap()
        .spawn(render_task(session.clone(), render_rx));

    // Store render task handle
    {
        let mut handles = RENDER_TASK_HANDLES.lock().unwrap();
        handles.insert(session_id, render_handle);
    }

    info!(
        "Map session {} created with full rendering pipeline",
        session_id
    );
    Ok(())
}

/// Creates a Galileo map with default OSM layer.
fn create_galileo_map_with_layers() -> anyhow::Result<Map> {
    // Add OpenStreetMap layer
    let osm_layer = RasterTileLayerBuilder::new_osm()
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create OSM layer: {}", e))?;

    // Set initial viewport (center on world)
    let map = MapBuilder::default()
        .with_layer(osm_layer)
        .with_latlon(0.0, 0.0) // Center on equator/prime meridian
        .with_z_level(2) // Initial zoom level
        .build();

    Ok(map)
}





/// Main rendering task that handles the actual rendering pipeline.
async fn render_task(
    session: Arc<MapSession>,
    mut render_rx: mpsc::UnboundedReceiver<RenderMessage>,
) {
    let mut render_interval = tokio::time::interval(
        std::time::Duration::from_millis(33), // ~30 FPS
    );

    info!("Starting render task for session {}", session.session_id);

    loop {
        tokio::select! {
            // Handle render messages
            message = render_rx.recv() => {
                match message {
                    Some(RenderMessage::RenderFrame) => {
                        if let Err(e) = render_frame(&session).await {
                            warn!("Failed to render frame for session {}: {}", session.session_id, e);
                        }
                    }
                    Some(RenderMessage::Resize(new_size)) => {
                        if let Err(e) = resize_session(&session, new_size).await {
                            warn!("Failed to resize session {}: {}", session.session_id, e);
                        }
                    }
                    Some(RenderMessage::UpdateMap) => {
                        // Trigger a new render frame
                        if let Err(e) = render_frame(&session).await {
                            warn!("Failed to render updated map for session {}: {}", session.session_id, e);
                        }
                    }
                    Some(RenderMessage::Stop) | None => {
                        info!("Stopping render task for session {}", session.session_id);
                        break;
                    }
                }
            }

            // Regular frame rendering
            _ = render_interval.tick() => {
                // Check if session is still alive
                {
                    let is_alive = session.is_alive.lock().unwrap();
                    if !*is_alive {
                        break;
                    }
                }

                // Render frame at regular intervals
                if let Err(e) = render_frame(&session).await {
                    warn!("Failed to render regular frame for session {}: {}", session.session_id, e);
                }
            }
        }
    }

    info!("Render task completed for session {}", session.session_id);
}


/// Triggers a map update and re-render.
fn trigger_map_update(session_id: i64) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    let render_commands = session.render_commands.lock().unwrap();
    render_commands
        .send(RenderMessage::UpdateMap)
        .map_err(|e| anyhow::anyhow!("Failed to send update message: {}", e))?;

    Ok(())
}

/// Event handling functions that work with simple coordinate mapping

pub fn handle_session_touch_event(session_id: i64, event: TouchEvent) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock().unwrap();
    let _session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    // Simple touch handling - for now just trigger a re-render
    debug!(
        "Touch event for session {}: {:?} at ({}, {})",
        session_id, event.event_type, event.x, event.y
    );

    // Trigger re-render
    trigger_map_update(session_id)?;

    Ok(())
}

pub fn handle_session_pan_event(session_id: i64, event: PanEvent) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    // Simple pan handling - modify map center based on delta
    debug!(
        "Pan event for session {}: {:?} delta=({}, {})",
        session_id, event.event_type, event.delta_x, event.delta_y
    );

    if let PanEventType::Update = event.event_type {
        let mut map = session.map.lock().unwrap();
        let current_view = map.view();

        // Calculate new position based on pan delta
        // This is a simplified implementation - in a real app you'd convert screen coordinates to map coordinates
        let current_pos = current_view
            .position()
            .unwrap_or_else(|| GeoPoint2d::latlon(0.0, 0.0));
        let delta_scale = 0.0001; // Simple scaling factor
        let new_lat = current_pos.lat() - event.delta_y * delta_scale;
        let new_lon = current_pos.lon() + event.delta_x * delta_scale;

        let new_center = GeoPoint2d::latlon(new_lat, new_lon);
        let new_view = current_view.with_position(&new_center);
        map.set_view(new_view);
    }

    // Trigger re-render
    trigger_map_update(session_id)?;

    Ok(())
}

pub fn handle_session_scale_event(session_id: i64, event: ScaleEvent) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    // Simple zoom handling - modify resolution based on scale
    debug!(
        "Scale event for session {}: scale={} at ({}, {})",
        session_id, event.scale, event.focal_x, event.focal_y
    );

    {
        let mut map = session.map.lock().unwrap();
        let current_view = map.view();
        let current_resolution = current_view.resolution();

        // Apply scale change (inverted because smaller resolution = more zoom)
        let scale_factor = 1.0 / event.scale.max(0.1).min(10.0);
        let new_resolution = (current_resolution * scale_factor).max(0.1);

        let new_view = current_view.with_resolution(new_resolution);
        map.set_view(new_view);
    }

    // Trigger re-render
    trigger_map_update(session_id)?;

    Ok(())
}

/// Resize a session
pub fn resize_session_size(session_id: i64, size: MapSize) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    let render_commands = session.render_commands.lock().unwrap();
    render_commands
        .send(RenderMessage::Resize(size))
        .map_err(|e| anyhow::anyhow!("Failed to send resize message: {}", e))?;

    Ok(())
}

/// Marks the session as alive (called periodically from Flutter)
pub fn mark_session_alive(session_id: i64) {
    if let Some(session) = SESSIONS.lock().unwrap().get(&session_id) {
        let mut is_alive = session.is_alive.lock().unwrap();
        *is_alive = true;
        debug!("Session {} marked as alive", session_id);
    }
}

/// Destroys all streams for a given engine
pub fn destroy_all_engine_sessions(engine_id: i64) {
    debug!("destroy_engine_streams called for engine {}", engine_id);

    // Find and remove all sessions for this engine
    let mut sessions_to_remove = Vec::new();
    {
        let sessions = SESSIONS.lock().unwrap();
        for (session_id, session) in sessions.iter() {
            if session.engine_handle == engine_id {
                sessions_to_remove.push(*session_id);
            }
        }
    }

    for session_id in sessions_to_remove {
        destroy_session(session_id);
    }
}

/// Destroys a specific session
pub fn destroy_session(session_id: i64) {
    debug!("destroy_session called for session {}", session_id);
    
    // Stop render task first
    if let Some(session) = SESSIONS.lock().unwrap().remove(&session_id) {
        // Mark as not alive
        {
            let mut is_alive = session.is_alive.lock().unwrap();
            *is_alive = false;
        }

        // Stop render loop
        if let Err(e) = session.render_loop.stop() {
            warn!(
                "Failed to stop render loop for session {}: {}",
                session_id, e
            );
        }

        // Send stop message to render task
        {
            let render_commands = session.render_commands.lock().unwrap();
            let _ = render_commands.send(RenderMessage::Stop);
        }
    }


    // Wait for render task to complete and remove handle
    {
        let mut handles = RENDER_TASK_HANDLES.lock().unwrap();
        if let Some(handle) = handles.remove(&session_id) {
            handle.abort(); // Force abort if not completed
        }
    }

    info!("Session {} destroyed with full cleanup", session_id);
}

/// Gets the current viewport for a session
pub fn get_session_viewport(session_id: i64) -> anyhow::Result<MapViewport> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    let map = session.map.lock().unwrap();
    let view = map.view();

    // Get center position - need to project from current CRS to lat/lon
    let center_point = view
        .position()
        .unwrap_or_else(|| GeoPoint2d::latlon(0.0, 0.0));

    Ok(MapViewport {
        center: MapPosition {
            latitude: center_point.lat(),
            longitude: center_point.lon(),
        },
        zoom: view.resolution(),
        rotation: 0.0, // Galileo doesn't have rotation yet
    })
}

/// Sets the viewport for a session
pub fn set_session_viewport(session_id: i64, viewport: MapViewport) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    {
        let mut map = session.map.lock().unwrap();
        let center = GeoPoint2d::latlon(viewport.center.latitude, viewport.center.longitude);

        // Update the map view
        let new_view = map
            .view()
            .with_position(&center)
            .with_resolution(viewport.zoom);
        map.set_view(new_view);
    }

    // Trigger re-render
    trigger_map_update(session_id)?;

    Ok(())
}

/// Adds a layer to a session
pub fn add_session_layer(session_id: i64, layer_config: LayerConfig) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock().unwrap();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    let layer = match layer_config {
        LayerConfig::Osm => RasterTileLayerBuilder::new_osm()
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create OSM layer: {}", e))?,
        LayerConfig::RasterTiles {
            url_template: _,
            attribution: _,
        } => {
            // For now, just return OSM layer for custom tile providers
            // TODO: Implement custom URL tile providers
            RasterTileLayerBuilder::new_osm()
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create OSM layer: {}", e))?
        }
    };

    {
        let mut map = session.map.lock().unwrap();
        map.layers_mut().push(layer);
    }

    // Trigger re-render to show new layer
    trigger_map_update(session_id)?;

    Ok(())
}
