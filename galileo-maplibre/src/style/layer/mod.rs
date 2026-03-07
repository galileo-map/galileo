//! Style layer definitions.
//!
//! Each MapLibre layer type has its own submodule. This module re-exports all
//! public types and provides the [`Layer`] enum used in [`crate::style::MaplibreStyle`].

pub mod background;
pub mod common;
pub mod fill;
pub mod line;
pub mod misc;
pub mod point;
pub mod raster;
pub mod symbol;

pub use background::{BackgroundLayer, BackgroundLayout, BackgroundPaint};
pub use common::{CommonLayout, Visibility};
pub use fill::{
    FillExtrusionLayer, FillExtrusionLayout, FillExtrusionPaint, FillLayer, FillLayout, FillPaint,
};
pub use line::{LineLayer, LineLayout, LinePaint};
pub use misc::{ClipLayer, SkyLayer, SkyLayout, SkyPaint, SlotLayer};
pub use point::{
    CircleLayer, CircleLayout, CirclePaint, HeatmapLayer, HeatmapLayout, HeatmapPaint,
};
pub use raster::{
    HillshadeLayer, HillshadeLayout, HillshadePaint, RasterLayer, RasterLayout, RasterPaint,
};
use serde::Deserialize;
pub use symbol::{SymbolLayer, SymbolLayout, SymbolPaint};

/// A map layer — one of the supported rendering layer types.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum Layer {
    /// A background fill layer.
    #[serde(rename = "background")]
    Background(BackgroundLayer),
    /// A filled polygon layer.
    #[serde(rename = "fill")]
    Fill(FillLayer),
    /// A stroked line layer.
    #[serde(rename = "line")]
    Line(Box<LineLayer>),
    /// An icon / text label layer.
    #[serde(rename = "symbol")]
    Symbol(Box<SymbolLayer>),
    /// A raster tile layer.
    #[serde(rename = "raster")]
    Raster(RasterLayer),
    /// A circle layer.
    #[serde(rename = "circle")]
    Circle(CircleLayer),
    /// An extruded polygon (3D) layer.
    #[serde(rename = "fill-extrusion")]
    FillExtrusion(FillExtrusionLayer),
    /// A heatmap layer.
    #[serde(rename = "heatmap")]
    Heatmap(HeatmapLayer),
    /// A client-side hillshade layer.
    #[serde(rename = "hillshade")]
    Hillshade(HillshadeLayer),
    /// A sky / atmosphere dome layer.
    #[serde(rename = "sky")]
    Sky(SkyLayer),
    /// An insertion-point layer for imported styles.
    #[serde(rename = "slot")]
    Slot(SlotLayer),
    /// A clipping mask layer.
    #[serde(rename = "clip")]
    Clip(ClipLayer),
}

#[cfg(test)]
mod tests {
    use common::{DEFAULT_MAXZOOM, DEFAULT_MINZOOM};

    use super::*;

    #[test]
    fn parse_background_layer() {
        let json = r#"{
            "id": "Background",
            "type": "background",
            "layout": {"visibility": "visible"},
            "paint": {"background-color": "hsl(47,79%,94%)"}
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Background(bg) = layer else {
            panic!("expected Background")
        };
        assert_eq!(bg.id, "Background");
        assert_eq!(bg.layout.visibility, Visibility::Visible);
        assert_eq!(bg.minzoom, DEFAULT_MINZOOM);
        assert_eq!(bg.maxzoom, DEFAULT_MAXZOOM);
    }

    #[test]
    fn parse_fill_layer() {
        let json = r#"{
            "id": "Meadow",
            "type": "fill",
            "source": "maptiler_planet",
            "source-layer": "globallandcover",
            "maxzoom": 8,
            "layout": {"visibility": "visible"},
            "paint": {"fill-color": "hsl(75,51%,85%)", "fill-opacity": 0.5},
            "filter": ["==", "class", "grass"]
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Fill(f) = layer else {
            panic!("expected Fill")
        };
        assert_eq!(f.id, "Meadow");
        assert_eq!(f.source.as_deref(), Some("maptiler_planet"));
        assert_eq!(f.source_layer.as_deref(), Some("globallandcover"));
        assert_eq!(f.maxzoom, 8.0);
        assert!(f.filter.is_some());
    }

    #[test]
    fn parse_line_layer() {
        let json = r##"{
            "id": "Road",
            "type": "line",
            "source": "maptiler_planet",
            "source-layer": "transportation",
            "layout": {"line-cap": "round", "line-join": "round", "visibility": "visible"},
            "paint": {"line-color": "#fff", "line-width": 2}
        }"##;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Line(l) = layer else {
            panic!("expected Line")
        };
        assert_eq!(l.id, "Road");
        assert!(l.layout.line_cap.is_some());
        assert!(l.layout.line_join.is_some());
    }

    #[test]
    fn parse_symbol_layer() {
        let json = r##"{
            "id": "Labels",
            "type": "symbol",
            "source": "maptiler_planet",
            "source-layer": "place",
            "layout": {
                "text-field": "{name}",
                "text-font": ["Open Sans Regular"],
                "text-size": 12,
                "visibility": "visible"
            },
            "paint": {
                "text-color": "#333",
                "text-halo-color": "#fff",
                "text-halo-width": 1
            }
        }"##;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Symbol(s) = layer else {
            panic!("expected Symbol")
        };
        assert_eq!(s.id, "Labels");
        assert!(s.layout.text_field.is_some());
        assert!(s.layout.text_font.is_some());
    }

    #[test]
    fn parse_raster_layer() {
        let json = r#"{
            "id": "Satellite",
            "type": "raster",
            "source": "satellite",
            "paint": {"raster-opacity": 1}
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Raster(r) = layer else {
            panic!("expected Raster")
        };
        assert_eq!(r.id, "Satellite");
        assert!(r.paint.raster_opacity.is_some());
    }

    #[test]
    fn parse_circle_layer() {
        let json = r##"{
            "id": "Points",
            "type": "circle",
            "source": "my-source",
            "paint": {"circle-color": "#f00", "circle-radius": 5}
        }"##;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Circle(c) = layer else {
            panic!("expected Circle")
        };
        assert_eq!(c.id, "Points");
        assert!(c.paint.circle_color.is_some());
    }

    #[test]
    fn parse_fill_extrusion_layer() {
        let json = r##"{
            "id": "Buildings",
            "type": "fill-extrusion",
            "source": "maptiler_planet",
            "source-layer": "building",
            "paint": {
                "fill-extrusion-color": "#aaa",
                "fill-extrusion-height": 10,
                "fill-extrusion-base": 0,
                "fill-extrusion-opacity": 0.6
            }
        }"##;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::FillExtrusion(fe) = layer else {
            panic!("expected FillExtrusion")
        };
        assert_eq!(fe.id, "Buildings");
        assert!(fe.paint.fill_extrusion_height.is_some());
    }

    #[test]
    fn parse_heatmap_layer() {
        let json = r#"{
            "id": "Density",
            "type": "heatmap",
            "source": "earthquakes",
            "paint": {"heatmap-radius": 30, "heatmap-opacity": 0.8}
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Heatmap(h) = layer else {
            panic!("expected Heatmap")
        };
        assert_eq!(h.id, "Density");
        assert!(h.paint.heatmap_opacity.is_some());
    }

    #[test]
    fn parse_hillshade_layer() {
        let json = r#"{
            "id": "Terrain",
            "type": "hillshade",
            "source": "dem",
            "paint": {"hillshade-exaggeration": 0.5}
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Hillshade(h) = layer else {
            panic!("expected Hillshade")
        };
        assert_eq!(h.id, "Terrain");
        assert!(h.paint.hillshade_exaggeration.is_some());
    }

    #[test]
    fn parse_sky_layer() {
        let json = r#"{
            "id": "Sky",
            "type": "sky",
            "paint": {"sky-type": "atmosphere", "sky-opacity": 1}
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Sky(s) = layer else {
            panic!("expected Sky")
        };
        assert_eq!(s.id, "Sky");
        assert!(s.paint.sky_opacity.is_some());
    }

    #[test]
    fn parse_slot_layer() {
        let json = r#"{"id": "bottom", "type": "slot"}"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Slot(s) = layer else {
            panic!("expected Slot")
        };
        assert_eq!(s.id, "bottom");
    }

    #[test]
    fn parse_clip_layer() {
        let json = r#"{
            "id": "Clip",
            "type": "clip",
            "source": "composite",
            "source-layer": "building",
            "clip-layer-types": ["3d", "symbols"]
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Clip(c) = layer else {
            panic!("expected Clip")
        };
        assert_eq!(c.id, "Clip");
        assert_eq!(
            c.clip_layer_types.as_deref(),
            Some(["3d".to_string(), "symbols".to_string()].as_slice())
        );
    }

    #[test]
    fn unknown_layer_type_returns_error() {
        let json = r#"{"id": "x", "type": "unknown"}"#;
        assert!(serde_json::from_str::<Layer>(json).is_err());
    }

    #[test]
    fn invalid_minzoom_falls_back_to_default() {
        let json = r#"{"id": "x", "type": "fill", "source": "s", "minzoom": "bad"}"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Fill(f) = layer else {
            panic!("expected Fill")
        };
        assert_eq!(f.minzoom, DEFAULT_MINZOOM);
    }

    #[test]
    fn invalid_maxzoom_falls_back_to_default() {
        let json = r#"{"id": "x", "type": "line", "source": "s", "maxzoom": "bad"}"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Line(l) = layer else {
            panic!("expected Line")
        };
        assert_eq!(l.maxzoom, DEFAULT_MAXZOOM);
    }

    #[test]
    fn invalid_visibility_falls_back_to_default() {
        let json = r#"{
            "id": "x",
            "type": "fill",
            "source": "s",
            "layout": {"visibility": "bad-value"}
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Fill(f) = layer else {
            panic!("expected Fill")
        };
        assert_eq!(f.layout.visibility, Visibility::Visible);
    }

    #[test]
    fn missing_layout_uses_defaults() {
        let json = r#"{"id": "x", "type": "background"}"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        let Layer::Background(bg) = layer else {
            panic!("expected Background")
        };
        assert_eq!(bg.layout.visibility, Visibility::Visible);
        assert!(bg.paint.background_color.is_none());
    }
}
