use std::any::Any;
use std::sync::Arc;

use galileo::layer::attribution::Attribution;
use galileo::layer::Layer;
use galileo::render::Canvas;
use galileo::{MapView, Messenger};

use crate::MaplibreStyle;

/// A Galileo [`Layer`] that renders a Maplibre style definition.
///
/// Internally owns one or more Galileo layers derived from the style's sources and renders them
/// sequentially. Construct with [`MaplibreLayer::from_json`].
pub struct MaplibreLayer {
    #[allow(dead_code)]
    style: MaplibreStyle,
    messenger: Option<Arc<dyn Messenger>>,
}

impl MaplibreLayer {
    /// Parses a Maplibre style JSON string and creates a new layer.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let style = serde_json::from_str(json)?;
        Ok(Self {
            style,
            messenger: None,
        })
    }
}

impl Layer for MaplibreLayer {
    fn render(&self, _view: &MapView, _canvas: &mut dyn Canvas) {}

    fn prepare(&self, _view: &MapView) {}

    fn set_messenger(&mut self, messenger: Box<dyn Messenger>) {
        self.messenger = Some(Arc::from(messenger));
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn attribution(&self) -> Option<Attribution> {
        None
    }
}
