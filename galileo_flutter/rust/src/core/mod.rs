//! Core implementation modules for Galileo Flutter integration.
//!
//! This module contains the internal implementation details for:
//! - Windowless wgpu rendering setup
//! - FPS-controlled render loops
//! - Pixel buffer management for texture copying
//! - Integration with irondash textures

pub mod windowless_renderer;
pub mod render_loop;
pub mod pixel_buffer;
pub mod flutter;


use tokio::runtime::Runtime;
pub use windowless_renderer::WindowlessRenderer;
pub use render_loop::RenderLoop;
pub use pixel_buffer::PixelBuffer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use log::debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

lazy_static::lazy_static! {
    pub static ref IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
    static ref WORKER_GUARD: std::sync::Mutex<Option<tracing_appender::non_blocking::WorkerGuard>> = std::sync::Mutex::new(None);
    pub static ref TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::from(
        tokio::runtime::Builder::new_current_thread()
            .worker_threads(4)
            .enable_all()
            .build().unwrap()
    );
}

pub(crate) fn init_logger() {
    let is_initialized = IS_INITIALIZED.load(Ordering::SeqCst);
    if is_initialized {
        return;
    }
    let file_appender = tracing_appender::rolling::daily("./logs", "galileo_flutter.log");
    let (non_blocking_file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_file_writer)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false);
    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_ansi(false);
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info")) // Default to info level if RUST_LOG is not set
        .unwrap();
    // 5. Combine the layers and initialize the global subscriber
    tracing_subscriber::registry()
        .with(env_filter) // Apply the environment filter
        .with(console_layer) // Add the stdout layer
        .with(file_layer) // Add the file layer
        .try_init()
        .unwrap(); // Set as the global default subscriber

    // leak the guard to keep the file writer alive
    WORKER_GUARD.lock().unwrap().replace(guard);
    // Default utilities - feel free to custom
    flutter_rust_bridge::setup_default_user_utils();
    debug!("Done initializing");
}
