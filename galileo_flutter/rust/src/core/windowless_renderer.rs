
use galileo::galileo_types::cartesian::Size;
use galileo::render::WgpuRenderer;
use log::debug;
use parking_lot::Mutex;
use std::sync::Arc;
use galileo::Map;
use wgpu::{
    Device, Extent3d, Queue, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView,
};

/// Error types for windowless renderer operations.
#[derive(Debug, thiserror::Error)]
pub enum WindowlessRendererError {
    #[error("Failed to create wgpu adapter")]
    AdapterCreationFailed,
    #[error("Failed to create wgpu device: {0}")]
    DeviceCreationFailed(#[from] wgpu::RequestDeviceError),
    #[error("Failed to create texture: {0}")]
    TextureCreationFailed(String),
    #[error("Renderer not initialized")]
    NotInitialized,
    #[error("Invalid size: width={0}, height={1}")]
    InvalidSize(u32, u32),
}

/// Windowless wgpu renderer that creates a device without a surface.
///
/// This renderer follows the windowless pattern from learn-wgpu:
/// 1. Create instance, adapter, device, and queue
/// 2. Create a render target texture
/// 3. Initialize Galileo's WgpuRenderer with the device and texture
pub struct WindowlessRenderer {
    galileo_renderer: WgpuRenderer,
    size: Size<u32>,
}

impl WindowlessRenderer {
    pub async fn new(size: Size<u32>) -> Result<Self, WindowlessRendererError> {
        debug!("Initializing renderer with size {size:?}");

        if size.width() == 0 || size.height() == 0 {
            return Err(WindowlessRendererError::InvalidSize(
                size.width(),
                size.height(),
            ));
        }
        let mut renderer = WgpuRenderer::new_with_texture_rt(size.clone())
            .await
            .expect("failed to create renderer");
        
        renderer.set_horizon_options(None);

        debug!("Renderer initialized");

        Ok( Self {
            galileo_renderer: renderer,
            size,
        })
    }

    /// Resizes the renderer and recreates the target texture.
    pub fn resize(&mut self, new_size: Size<u32>) -> Result<(), WindowlessRendererError> {
        self.galileo_renderer.resize(new_size);
        self.size = new_size;
        Ok(())
    }

    /// Gets the current size of the renderer.
    pub fn size(&self) -> Size<u32> {
        self.size
    }

    pub async fn render(&self, map: &Map) -> Vec<u8>{
        debug!("Redner is called for galileo map");
        
        self.galileo_renderer.trim_textures();

        self.galileo_renderer.render(&map).expect("failed to render the map");
    
        self.galileo_renderer.get_image()
            .await
            .expect("failed to get image bitmap from texture")
            
    }

}

/// Thread-safe wrapper for WindowlessRenderer.
/// This allows the renderer to be shared between the main thread and render thread.
pub type SharedWindowlessRenderer = Arc<Mutex<WindowlessRenderer>>;

impl std::fmt::Debug for WindowlessRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowlessRenderer")
            .field("size", &self.size)
            .finish()
    }
}
