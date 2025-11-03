//! Service for text rendering.

use std::sync::{Arc, OnceLock};

use galileo_types::cartesian::Vector2;
use parking_lot::RwLock;
use rustybuzz::ttf_parser::FaceParsingError;
use thiserror::Error;

use super::font_provider::FontProvider;
use crate::render::text::font_provider::DefaultFontProvider;
use crate::render::text::{TextRasterizer, TextShaping, TextStyle};

static INSTANCE: OnceLock<TextService> = OnceLock::new();

/// Error from a font service
#[derive(Debug, Error)]
pub enum FontServiceError {
    /// Error parsing font face file
    #[error(transparent)]
    FaceParsingError(#[from] FaceParsingError),

    /// Font file not found
    #[error("font is not loaded")]
    FontNotFound,

    /// Font service is not initialized
    #[error("font service is not initialize")]
    NotInitialized,
}

/// Provides common access to underlying text shaping engine implementation.
pub struct TextService {
    pub(crate) rasterizer: RwLock<Box<dyn TextRasterizer + Send + Sync>>,
    font_provider: Box<dyn FontProvider + Send + Sync>,
}

impl TextService {
    /// Initializes the font service with the given provider.
    pub fn initialize(provider: impl TextRasterizer + Send + Sync + 'static) -> &'static Self {
        if INSTANCE.get().is_some() {
            log::warn!(
                "Font service is already initialized. Second initialization call is ignored."
            );
        }

        INSTANCE.get_or_init(|| {
            log::info!("Initializing TextService");

            Self {
                rasterizer: RwLock::new(Box::new(provider)),
                font_provider: Box::new(DefaultFontProvider::new()),
            }
        })
    }
    
    /// Ensures the font service is initialized with default settings if not already initialized.
    /// This is a fallback for cases where explicit initialization was not called.
    #[cfg(not(target_arch = "wasm32"))]
    fn ensure_initialized() -> &'static Self {
        use crate::render::text::RustybuzzRasterizer;
        
        INSTANCE.get_or_init(|| {
            log::warn!(
                "TextService was not explicitly initialized. Initializing with default settings. \
                For better control, call TextService::initialize() during application startup."
            );

            let service = Self {
                rasterizer: RwLock::new(Box::new(RustybuzzRasterizer::default())),
                font_provider: Box::new(DefaultFontProvider::new()),
            };
            
            // Load system fonts on Windows
            #[cfg(target_os = "windows")]
            {
                log::info!("Loading fonts from C:/Windows/Fonts");
                service.font_provider.load_fonts_folder("C:/Windows/Fonts".into());
            }
            
            // Load system fonts on macOS
            #[cfg(target_os = "macos")]
            {
                log::info!("Loading fonts from /System/Library/Fonts");
                service.font_provider.load_fonts_folder("/System/Library/Fonts".into());
                service.font_provider.load_fonts_folder("/Library/Fonts".into());
            }
            
            // Load system fonts on Linux
            #[cfg(target_os = "linux")]
            {
                log::info!("Loading fonts from /usr/share/fonts");
                service.font_provider.load_fonts_folder("/usr/share/fonts".into());
            }
            
            service
        })
    }
    
    #[cfg(target_arch = "wasm32")]
    fn ensure_initialized() -> &'static Self {
        use crate::render::text::RustybuzzRasterizer;
        
        INSTANCE.get_or_init(|| {
            log::warn!(
                "TextService was not explicitly initialized. Initializing with default settings. \
                For better control, call TextService::initialize() during application startup."
            );

            Self {
                rasterizer: RwLock::new(Box::new(RustybuzzRasterizer::default())),
                font_provider: Box::new(DefaultFontProvider::new()),
            }
        })
    }

    /// Returns static instance of the service if it was initialized.
    pub fn instance() -> Option<&'static Self> {
        INSTANCE.get()
    }

    /// Shape the given text input with the given style.
    pub fn shape(
        text: &str,
        style: &TextStyle,
        offset: Vector2<f32>,
        dpi_scale_factor: f32,
    ) -> Result<TextShaping, FontServiceError> {
        // Get existing instance or auto-initialize with defaults
        let service = Self::instance().unwrap_or_else(|| {
            log::warn!("TextService::shape() called before initialization. Auto-initializing with defaults.");
            Self::ensure_initialized()
        });

        service.rasterizer.read().shape(
            text,
            style,
            offset,
            &*service.font_provider,
            dpi_scale_factor,
        )
    }

    /// Load all fonts from the given directory (recursevly).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_fonts(&self, folder_path: impl AsRef<std::path::Path>) {
        self.font_provider
            .load_fonts_folder(folder_path.as_ref().into());
    }

    /// Loads the font faces from the given font binary data.
    pub fn load_font(&self, font_data: Arc<Vec<u8>>) {
        self.load_font_internal(font_data, true);
    }

    pub(crate) fn load_font_internal(&self, font_data: Arc<Vec<u8>>, notify_workers: bool) {
        self.font_provider.load_font_data(font_data.clone());

        if notify_workers {
            #[cfg(target_arch = "wasm32")]
            crate::async_runtime::spawn(async {
                crate::platform::web::web_workers::WebWorkerService::instance()
                    .load_font(font_data)
                    .await
            });
        }
    }
}
