//! Types shared between Dart and Rust for Galileo Flutter integration.
//! All types here are used by flutter_rust_bridge_codegen.

use flutter_rust_bridge::frb;

/// Geographic position with latitude and longitude coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapPosition {
    pub latitude: f64,
    pub longitude: f64,
}

/// Map viewport configuration including center, zoom, and rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapViewport {
    pub center: MapPosition,
    pub zoom: f64,
    pub rotation: f64,
}

/// Physical size of the map in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapSize {
    pub width: u32,
    pub height: u32,
}

/// Configuration for the rendering system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderConfig {
    /// Frames per second for the render loop (default: 30)
    pub fps: u32,
    /// Enable multisampling anti-aliasing
    pub enable_multisampling: bool,
    /// Background color as RGBA (0.0-1.0 range)
    pub background_color: (f32, f32, f32, f32),
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            fps: 30,
            enable_multisampling: true,
            background_color: (0.1, 0.2, 0.3, 1.0),
        }
    }
}

/// Touch event from Flutter gesture detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchEvent {
    /// X coordinate relative to the map widget
    pub x: f64,
    /// Y coordinate relative to the map widget
    pub y: f64,
    /// Type of touch event
    pub event_type: TouchEventType,
}

/// Types of touch events that can occur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TouchEventType {
    Down,
    Move,
    Up,
    Cancel,
}

/// Scroll/zoom event from Flutter gesture detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollEvent {
    /// X coordinate relative to the map widget
    pub x: f64,
    /// Y coordinate relative to the map widget
    pub y: f64,
    /// Horizontal scroll delta
    pub delta_x: f64,
    /// Vertical scroll delta (used for zoom)
    pub delta_y: f64,
}

/// Pan gesture event from Flutter gesture detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanEvent {
    /// X coordinate relative to the map widget
    pub x: f64,
    /// Y coordinate relative to the map widget
    pub y: f64,
    /// Change in X since last event
    pub delta_x: f64,
    /// Change in Y since last event
    pub delta_y: f64,
    /// Type of pan event
    pub event_type: PanEventType,
}

/// Types of pan events that can occur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanEventType {
    Start,
    Update,
    End,
}

/// Scale/pinch gesture event from Flutter gesture detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleEvent {
    /// Focal point X coordinate relative to the map widget
    pub focal_x: f64,
    /// Focal point Y coordinate relative to the map widget
    pub focal_y: f64,
    /// Scale factor (1.0 = no change)
    pub scale: f64,
    /// Rotation in radians
    pub rotation: f64,
    /// Type of scale event
    pub event_type: ScaleEventType,
}

/// Types of scale events that can occur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleEventType {
    Start,
    Update,
    End,
}

/// Layer configuration for different types of map layers.
#[derive(Debug, Clone, PartialEq)]
pub enum LayerConfig {
    /// OpenStreetMap raster tile layer
    Osm,
    /// Custom raster tile layer with URL template
    RasterTiles {
        url_template: String,
        attribution: Option<String>,
    },
}

// Opaque types for complex Rust objects that shouldn't be exposed to Dart directly
/// Opaque handle to the Galileo map session.
/// This contains the actual map state and rendering context.
#[frb(opaque)]
pub struct GalileoMapSession {
    // Internal implementation will be in the implementation file
}

/// Opaque handle to a Flutter texture.
/// This manages the texture that will be displayed in the Flutter widget.
#[frb(opaque)]
pub struct TextureHandle {
    // Internal texture management
}
