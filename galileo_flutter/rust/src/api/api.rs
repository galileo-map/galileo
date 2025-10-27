//! Main API for Galileo Flutter integration with texture rendering.
//!
//! This module provides the interface between Dart and Rust for
//! managing Galileo maps in Flutter applications with real texture rendering.

use flutter_rust_bridge::frb;
use galileo::control::UserEventHandler;
use galileo::layer::raster_tile_layer::RasterTileLayerBuilder;
use galileo::layer::vector_tile_layer::VectorTileLayerBuilder;
use galileo::layer::vector_tile_layer::style::VectorTileStyle;
use log::{debug, info};
use std::sync::atomic::Ordering;

use crate::api::dart_types::*;
use crate::core::map_session::{MapSession, SessionID};
use crate::core::{init_logger, IS_INITIALIZED, SESSIONS, TOKIO_RUNTIME};

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
    init_logger();
    info!("Galileo Flutter plugin initialized with FFI and texture support");
    IS_INITIALIZED.store(true, Ordering::SeqCst);
}

#[derive(Clone, Debug)]
pub struct CreateNewSessionResponse {
    pub session_id: u32,
    pub texture_id: i64,
}

pub fn create_new_map_session(
    engine_handle: i64,
    config: MapInitConfig,
) -> anyhow::Result<CreateNewSessionResponse> {
    info!("create_new_map_session was called");

    let session = TOKIO_RUNTIME
        .get()
        .unwrap()
        .block_on(MapSession::new(engine_handle, config))?;
    info!("New map session created with ID {}", session.session_id);
    Ok(CreateNewSessionResponse {
        session_id: session.session_id,
        texture_id: session.get_flutter_texture_id().unwrap(),
    })
}

/// Triggers a map update and re-render.
pub fn request_map_redraw(session_id: SessionID) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;
    TOKIO_RUNTIME.get().unwrap().block_on(session.redraw())
}

/// Marks the session as alive (called periodically from Flutter)
pub fn mark_session_alive(session_id: SessionID) {
    if let Some(session) = SESSIONS.lock().get(&session_id) {
        session.mark_alive();
        debug!("Session {} marked as alive", session_id);
    }
}

/// Destroys all streams for a given engine
pub fn destroy_all_engine_sessions(engine_id: i64) {
    debug!("destroy_engine_streams called for engine {}", engine_id);

    // Find and remove all sessions for this engine
    let mut sessions_to_remove = Vec::new();
    {
        let sessions = SESSIONS.lock();
        for (session_id, session) in sessions.iter() {
            if session.engine_handle == engine_id {
                sessions_to_remove.push(*session_id);
            }
        }
    }

    let handles: Vec<_> = sessions_to_remove
        .into_iter()
        .map(|session_id| {
            std::thread::spawn(move || {
                destroy_session(session_id);
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
}

/// Destroys a specific session
pub fn destroy_session(session_id: SessionID) {
    debug!("destroy_session called for session {}", session_id);
    if let Some(session) = SESSIONS.lock().remove(&session_id) {
        TOKIO_RUNTIME.get().unwrap().block_on(session.terminate());

        info!("Session {} destroyed with full cleanup", session_id);
        ()
    }
    info!("Session {session_id} does not exist")
}

/// Replaces {z}, {x}, {y} with tile indices
fn create_url_source(url_template: String) -> impl Fn(&galileo::tile_schema::TileIndex) -> String {
    move |index: &galileo::tile_schema::TileIndex| {
        url_template
            .replace("{z}", &index.z.to_string())
            .replace("{x}", &index.x.to_string())
            .replace("{y}", &index.y.to_string())
    }
}

/// Adds a layer to a session
pub fn add_session_layer(session_id: SessionID, layer_config: LayerConfig) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    match layer_config {
        LayerConfig::Osm => {
            let layer = RasterTileLayerBuilder::new_osm()
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create OSM layer: {}", e))?;
            session.add_layer(layer);
        }
        LayerConfig::RasterTiles {
            url_template: _,
            attribution: _,
        } => {
            // For now, just return OSM layer for custom tile providers
            // TODO: Implement custom URL tile providers
            let layer = RasterTileLayerBuilder::new_osm()
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create OSM layer: {}", e))?;
            session.add_layer(layer);
        }
        LayerConfig::VectorTiles {
            url_template,
            style_json,
            attribution,
        } => {
            let style: VectorTileStyle = serde_json::from_str(&style_json)
                .map_err(|e| anyhow::anyhow!("Failed to parse vector tile style: {}", e))?;
            
            let mut builder = VectorTileLayerBuilder::new_rest(create_url_source(url_template))
                .with_style(style)
                .with_tile_schema(galileo::TileSchema::web(18));
                // .with_file_cache_checked(".tile_cache");
            
            if let Some(attr) = attribution {
                builder = builder.with_attribution(attr, "".to_string());
            }
            
            let layer = builder.build()
                .map_err(|e| anyhow::anyhow!("Failed to create vector tile layer: {}", e))?;
            session.add_layer(layer);
        }
    }

    Ok(())
}

pub fn get_map_viewport(session_id: SessionID) -> Option<MapViewport> {
    if let Some(session) = SESSIONS.lock().get(&session_id) {
        return session.get_viewport();
    }
    None
}

pub fn handle_event_for_session(session_id: SessionID, event: UserEvent) {
    let galileo_event = event.to_galileo();

    let session = {
        let sessions = SESSIONS.lock();
        sessions.get(&session_id).cloned()
    };

    if let Some(session) = session {
        let mut map = session.map.lock();
        session.controller.handle(&galileo_event, &mut map);
    }
}

pub fn resize_session(session_id: SessionID, new_size: MapSize) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;
    
    TOKIO_RUNTIME
        .get()
        .unwrap()
        .block_on(session.resize(new_size))
}