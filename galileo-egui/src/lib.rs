//! Integration between [Galileo](https://docs.rs/galileo/latest/galileo/) and
//! [EGUI](https://docs.rs/galileo/latest/egui).
//!
//! This crate provides a widget [`EguiMap`] for `egui` to render a Galileo map into egui
//! application.
//!
//! With `init` feature you else get an [`InitBuilder`] that can help you set up a simple
//! application with a map. This struct is mainly meant to be used in development environments or
//! for simple examples.

use galileo::render::HorizonOptions;

mod egui_map;
pub use egui_map::{EguiMap, EguiMapState};

#[cfg(feature = "init")]
mod init;
#[cfg(feature = "init")]
pub use init::InitBuilder;

/// Options of the map
pub struct EguiMapOptions {
    pub(crate) horizon_options: Option<HorizonOptions>,
}

impl Default for EguiMapOptions {
    fn default() -> Self {
        Self {
            horizon_options: Some(HorizonOptions::default()),
        }
    }
}
