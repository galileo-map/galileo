use std::any::Any;
use std::sync::Arc;

use galileo::layer::attribution::Attribution;
use galileo::layer::Layer;
use galileo::render::Canvas;
use galileo::{MapView, Messenger};

use crate::style::layer::Layer as MaplibreStyleLayer;
use crate::MaplibreStyle;

pub mod vector_tile;

pub(crate) const UNSUPPORTED: &str = "[maplibre:unsupported]";

/// A Galileo [`Layer`] that renders a Maplibre style definition.
///
/// Internally owns one or more Galileo layers derived from the style's sources and renders them
/// sequentially. Construct with [`MaplibreLayer::from_json`].
pub struct MaplibreLayer {
    inner: Vec<Box<dyn Layer>>,
    messenger: Option<Arc<dyn Messenger>>,
}

impl MaplibreLayer {
    /// Parses a Maplibre style JSON string and creates a new layer.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let style: MaplibreStyle = serde_json::from_str(json)?;
        Ok(Self::from_style(style))
    }

    /// Creates a new layer from a parsed [`MaplibreStyle`].
    pub fn from_style(style: MaplibreStyle) -> Self {
        let inner = build_inner_layers(&style);
        Self {
            inner,
            messenger: None,
        }
    }
}

/// Groups style layers by source (preserving first-seen order) and tries to create a Galileo
/// layer for each source.
fn build_inner_layers(style: &MaplibreStyle) -> Vec<Box<dyn Layer>> {
    // Collect sources in the order they first appear across layers.
    let mut source_order: Vec<&str> = Vec::new();
    for map_layer in &style.layers {
        if let Some(src) = map_layer.source() {
            if !source_order.contains(&src) {
                source_order.push(src);
            }
        } else {
            log::info!(
                "{UNSUPPORTED} Maplibre layer type '{}' (id: '{}') is not yet supported. \
                 Open a GitHub issue or PR if support is desirable.",
                map_layer.type_name(),
                map_layer.id(),
            );
        }
    }

    let mut inner: Vec<Box<dyn Layer>> = Vec::new();

    for source_name in source_order {
        let Some(source) = style.sources.get(source_name) else {
            log::warn!(
                "Maplibre source '{source_name}' referenced by layers but not defined; skipping."
            );
            continue;
        };

        let source_layers: Vec<&MaplibreStyleLayer> = style
            .layers
            .iter()
            .filter(|l| l.source() == Some(source_name))
            .collect();

        if let Some(layer) = try_create_source_layer(source_name, source, &source_layers) {
            inner.push(layer);
        }
    }

    inner
}

fn try_create_source_layer(
    source_name: &str,
    source: &crate::style::source::Source,
    layers: &[&MaplibreStyleLayer],
) -> Option<Box<dyn Layer>> {
    use crate::style::source::Source;
    match source {
        Source::Vector(vector_source) => {
            vector_tile::try_create(source_name, vector_source, layers)
                .map(|l| Box::new(l) as Box<dyn Layer>)
        }
        _ => {
            log::info!(
                "{UNSUPPORTED} Maplibre source type for '{source_name}' is not yet supported. \
                 Open a GitHub issue or PR if support is desirable."
            );
            None
        }
    }
}

impl Layer for MaplibreLayer {
    fn render(&self, view: &MapView, canvas: &mut dyn Canvas) {
        for layer in &self.inner {
            layer.render(view, canvas);
        }
    }

    fn prepare(&self, view: &MapView) {
        for layer in &self.inner {
            layer.prepare(view);
        }
    }

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
