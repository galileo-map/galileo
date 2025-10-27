//! Types shared between Dart and Rust for Galileo Flutter integration.
//! All types here are used by flutter_rust_bridge_codegen.

use flutter_rust_bridge::frb;
use galileo::galileo_types;

/// Geographic position with latitude and longitude coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapPosition {
    pub latitude: f64,
    pub longitude: f64,
}

/// Map viewport configuration including center, zoom, and rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapViewport {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl MapViewport{
    #[frb(ignore)]
    pub fn from_rect(rect: &galileo_types::cartesian::Rect) -> Self {
        Self {
            x_min: rect.x_min(),
            x_max: rect.x_max(),
            y_min: rect.y_min(),
            y_max: rect.y_max(),
        }
    }
}


/// Physical size of the map in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapSize {
    pub width: u32,
    pub height: u32,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapInitConfig {
    pub latlon: (f64, f64),
    pub zoom_level: u32,
    pub map_size: MapSize,
    /// Frames per second for the render loop (default: 30)
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
    VectorTiles {
        url_template: String,
        style_json: String,
        attribution: Option<String>,
    }
}

// Manual type definitions for Dart-friendly versions

/// 2D point in cartesian coordinate space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    #[frb(ignore)]
    pub fn to_galileo(&self) -> galileo_types::cartesian::Point2<f64> {
        galileo_types::cartesian::Point2::new(self.x, self.y)
    }
}

/// 2D vector in cartesian coordinate space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub dx: f64,
    pub dy: f64,
}

impl Vector2 {
    #[frb(ignore)]
    pub fn to_galileo(&self) -> galileo_types::cartesian::Vector2<f64> {
        galileo_types::cartesian::Vector2::new(self.dx, self.dy)
    }
}

/// Mouse button enum.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseButton {
    /// The button you click when you want to shoot.
    Left,
    /// The button you click when you want to reload.
    Middle,
    /// The button you click when you want to hit with a rifle handle.
    Right,
    /// The button you click when you are a pro gamer and want to look cool.
    Other,
}

impl MouseButton {
    #[frb(ignore)]
    pub fn to_galileo(&self) -> galileo::control::MouseButton {
        match self {
            MouseButton::Left => galileo::control::MouseButton::Left,
            MouseButton::Middle => galileo::control::MouseButton::Middle,
            MouseButton::Right => galileo::control::MouseButton::Right,
            MouseButton::Other => galileo::control::MouseButton::Other,
        }
    }
}

/// Mouse button state.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MouseButtonState {
    /// Button is pressed.
    Pressed,
    /// Button is not pressed.
    Released,
}

impl MouseButtonState {
    #[frb(ignore)]
    pub fn to_galileo(&self) -> galileo::control::MouseButtonState {
        match self {
            MouseButtonState::Pressed => galileo::control::MouseButtonState::Pressed,
            MouseButtonState::Released => galileo::control::MouseButtonState::Released,
        }
    }
}

/// State of all mouse buttons.
#[derive(Debug, Copy, Clone)]
pub struct MouseButtonsState {
    /// State of the left mouse button.
    pub left: MouseButtonState,
    /// State of the middle mouse button.
    pub middle: MouseButtonState,
    /// State of the right mouse button.
    pub right: MouseButtonState,
}

impl MouseButtonsState {
    #[frb(ignore)]
    pub fn to_galileo(&self) -> galileo::control::MouseButtonsState {
        galileo::control::MouseButtonsState {
            left: self.left.to_galileo(),
            middle: self.middle.to_galileo(),
            right: self.right.to_galileo(),
        }
    }
}

/// State of the mouse at the moment of the event.
#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// Pointer position on the screen in pixels from the top-left corner.
    pub screen_pointer_position: Point2,
    /// State of the mouse buttons.
    pub buttons: MouseButtonsState,
}

impl MouseEvent {
    #[frb(ignore)]
    pub fn to_galileo(&self) -> galileo::control::MouseEvent {
        galileo::control::MouseEvent {
            screen_pointer_position: self.screen_pointer_position.to_galileo(),
            buttons: self.buttons.to_galileo(),
        }
    }
}

/// User interaction event.
#[derive(Debug, Clone)]
pub enum UserEvent {
    /// A mouse button was pressed.
    ButtonPressed(MouseButton, MouseEvent),
    /// A mouse button was released.
    ButtonReleased(MouseButton, MouseEvent),
    /// A mouse button was clicked.
    Click(MouseButton, MouseEvent),
    /// A double click was done.
    DoubleClick(MouseButton, MouseEvent),
    /// Mouse pointer moved.
    PointerMoved(MouseEvent),
    /// Drag started.
    DragStarted(MouseButton, MouseEvent),
    /// Mouse pointer moved after drag started was consumed.
    Drag(MouseButton, Vector2, MouseEvent),
    /// Mouse button was released while dragging.
    DragEnded(MouseButton, MouseEvent),
    /// Scroll event is called.
    Scroll(f64, MouseEvent),
    /// Zoom is called around a point.
    Zoom(f64, Point2),
}

impl UserEvent {
    #[frb(ignore)]
    pub fn to_galileo(&self) -> galileo::control::UserEvent {
        match self {
            UserEvent::ButtonPressed(button, event) => {
                galileo::control::UserEvent::ButtonPressed(button.to_galileo(), event.to_galileo())
            }
            UserEvent::ButtonReleased(button, event) => {
                galileo::control::UserEvent::ButtonReleased(button.to_galileo(), event.to_galileo())
            }
            UserEvent::Click(button, event) => {
                galileo::control::UserEvent::Click(button.to_galileo(), event.to_galileo())
            }
            UserEvent::DoubleClick(button, event) => {
                galileo::control::UserEvent::DoubleClick(button.to_galileo(), event.to_galileo())
            }
            UserEvent::PointerMoved(event) => {
                galileo::control::UserEvent::PointerMoved(event.to_galileo())
            }
            UserEvent::DragStarted(button, event) => {
                galileo::control::UserEvent::DragStarted(button.to_galileo(), event.to_galileo())
            }
            UserEvent::Drag(button, vector, event) => {
                galileo::control::UserEvent::Drag(button.to_galileo(), vector.to_galileo(), event.to_galileo())
            }
            UserEvent::DragEnded(button, event) => {
                galileo::control::UserEvent::DragEnded(button.to_galileo(), event.to_galileo())
            }
            UserEvent::Scroll(delta, event) => {
                galileo::control::UserEvent::Scroll(*delta, event.to_galileo())
            }
            UserEvent::Zoom(delta, point) => {
                galileo::control::UserEvent::Zoom(*delta, point.to_galileo())
            }
        }
    }
}
