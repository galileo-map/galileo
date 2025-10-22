pub mod flutter;
pub mod galileo_ref;
pub mod map_session;
pub mod pixel_buffer;
pub mod windowless_renderer;

use galileo::galileo_types;
use log::debug;
use parking_lot::Mutex;
pub use pixel_buffer::PixelBuffer;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
pub use windowless_renderer::WindowlessRenderer;

use crate::api::dart_types::MapSize;
use crate::core::flutter::pixel_texture::SharedPixelPayloadHolder;
use crate::core::map_session::{MapSession, SessionID};
impl MapSize {
    pub fn as_galileo(&self) -> galileo_types::cartesian::Size<u32> {
        galileo_types::cartesian::Size::new(self.width, self.height)
    }
}

lazy_static::lazy_static! {
    pub static ref IS_INITIALIZED: AtomicBool = AtomicBool::new(false);
    pub static ref WORKER_GUARD: std::sync::Mutex<Option<tracing_appender::non_blocking::WorkerGuard>> = std::sync::Mutex::new(None);
    pub static ref TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::from(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build().unwrap()
    );
    pub static ref SESSION_COUNTER: AtomicU32 = AtomicU32::new(0);
    pub static ref SESSIONS: Mutex<HashMap<SessionID, Arc<MapSession>>> = Mutex::new(HashMap::new());
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
