//! Types shared between Dart and Rust for Galileo Flutter integration.
//! All types here are used by flutter_rust_bridge_codegen.

use flutter_rust_bridge::frb;
use galileo::control::{MouseButton, MouseButtonState, MouseButtonsState, MouseEvent, UserEvent};
use galileo::galileo_types;
use galileo_types::cartesian::{Point2, Vector2};

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

impl MapSize {
    pub fn as_galileo(&self) -> galileo_types::cartesian::Size<u32> {
        galileo_types::cartesian::Size::new(self.width, self.height)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapInitConfig {
    pub latlon: (f64, f64),
    pub zoom_level: u32,
    pub map_size: MapSize,
    /// Frames per second for the render loop (default: 30)
    pub fps: u32,
    /// Enable multisampling anti-aliasing
    pub enable_multisampling: bool,
    /// Background color as RGBA (0.0-1.0 range)
    pub background_color: (f32, f32, f32, f32),
}

impl Default for MapInitConfig {
    fn default() -> Self {
        Self {
            latlon: (0.0, 0.0),
            zoom_level: 10,
            map_size: MapSize {
                width: 800,
                height: 600,
            },
            fps: 30,
            enable_multisampling: true,
            background_color: (0.1, 0.2, 0.3, 1.0),
        }
    }
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

// Mirror types for UserEvent and its inner fields

// Mirror for Point2<f64>
#[frb(mirror(Point2))]
pub struct _Point2 {
    pub x: f64,
    pub y: f64,
}

// Mirror for Vector2<f64>
#[frb(mirror(Vector2<f64>))]
pub struct _Vector2f64 {
    pub dx: f64,
    pub dy: f64,
}

// Mirror for MouseButton
#[frb(mirror(MouseButton))]
pub enum _MouseButton {
    Left,
    Middle,
    Right,
    Other,
}

// Mirror for MouseButtonState
#[frb(mirror(MouseButtonState))]
pub enum _MouseButtonState {
    Pressed,
    Released,
}

// Mirror for MouseButtonsState
#[frb(mirror(MouseButtonsState))]
pub struct _MouseButtonsState {
    pub left: MouseButtonState,
    pub middle: MouseButtonState,
    pub right: MouseButtonState,
}

// Mirror for MouseEvent
#[frb(mirror(MouseEvent))]
pub struct _MouseEvent {
    pub screen_pointer_position: Point2,
    pub buttons: MouseButtonsState,
}

// Mirror for UserEvent
#[frb(mirror(UserEvent))]
pub enum _UserEvent {
    ButtonPressed(MouseButton, MouseEvent),
    ButtonReleased(MouseButton, MouseEvent),
    Click(MouseButton, MouseEvent),
    DoubleClick(MouseButton, MouseEvent),
    PointerMoved(MouseEvent),
    DragStarted(MouseButton, MouseEvent),
    Drag(MouseButton, Vector2<f64>, MouseEvent),
    DragEnded(MouseButton, MouseEvent),
    Scroll(f64, MouseEvent),
    Zoom(f64, Point2),
}
