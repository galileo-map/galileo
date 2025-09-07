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
    TOKIO_RUNTIME.get().unwrap().block_on(
    session.redraw()    
    )}

/// Event handling functions that work with simple coordinate mapping

pub fn handle_session_touch_event(session_id: SessionID, event: TouchEvent) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    // Simple touch handling - for now just trigger a re-render
    debug!(
        "Touch event for session {}: {:?} at ({}, {})",
        session_id, event.event_type, event.x, event.y
    );
      TODO
    )
}

pub fn handle_session_pan_event(session_id: i64, event: PanEvent) -> anyhow::Result<()> {
    let sessions = SESSIONS.lock();
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
    let sessions = SESSIONS.lock();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    // Simple zoom handling - modify resolution based on scale
    debug!(
        "Scale event for session {}: scale={} at ({}, {})",
        session_id, event.scale, event.focal_x, event.focal_y
    );

    {
        let mut map = session.map.lock();
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
    let sessions = SESSIONS.lock();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;
    session.send_render_message(RenderMessage::Resize(size))
}

/// Marks the session as alive (called periodically from Flutter)
pub fn mark_session_alive(session_id: i64) {
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
pub fn destroy_session(session_id: i64) {
    debug!("destroy_session called for session {}", session_id);
    if let Some(session) = SESSIONS.lock().remove(&session_id) {
        session.terminate();
        info!("Session {} destroyed with full cleanup", session_id);
        ()
    }
    info!("Session {session_id} does not exist")
}

/// Gets the current viewport for a session
pub fn get_session_viewport(session_id: i64) -> anyhow::Result<MapViewport> {
    let sessions = SESSIONS.lock();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    let map = session.map.lock();
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
    let sessions = SESSIONS.lock();
    let session = sessions
        .get(&session_id)
        .ok_or_else(|| anyhow::anyhow!("Session {} not found", session_id))?;

    {
        let mut map = session.map.lock();
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

    {
        let mut map = session.map.lock();
        map.layers_mut().push(layer);
    }

    // Trigger re-render to show new layer
    trigger_map_update(session_id)?;

    Ok(())
}
