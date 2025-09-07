//! Main API for Galileo Flutter integration with texture rendering.
//!
//! This module provides the interface between Dart and Rust for
//! managing Galileo maps in Flutter applications with real texture rendering.

use flutter_rust_bridge::frb;
use galileo::control::UserEvent;
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
use crate::core::map_session::SessionID;
use crate::core::{IS_INITIALIZED, SESSIONS, SESSION_COUNTER, TOKIO_RUNTIME};

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

/// Triggers a map update and re-render.
fn request_map_redraw(session_id: SessionID) -> anyhow::Result<()> {
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
        session.terminate();
        info!("Session {} destroyed with full cleanup", session_id);
        ()
    }
    info!("Session {session_id} does not exist")
}

/// Adds a layer to a session
pub fn add_session_layer(session_id: SessionID, layer_config: LayerConfig) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock();
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

    session.add_layer(layer);

    Ok(())
}
