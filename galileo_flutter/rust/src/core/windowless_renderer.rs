//! Windowless wgpu renderer for Galileo Flutter integration.
//!
//! This module implements a windowless wgpu setup similar to the approach described
//! in the learn-wgpu tutorial. It creates a wgpu device and queue without a surface,
//! then initializes Galileo's WgpuRenderer with a custom texture.

use galileo::galileo_types::cartesian::Size;
use galileo::render::WgpuRenderer;
use parking_lot::Mutex;
use std::sync::Arc;
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
    device: Device,
    queue: Queue,
    galileo_renderer: Option<WgpuRenderer>,
    target_texture: Option<Texture>,
    target_texture_view: Option<TextureView>,
    size: Size<u32>,
}

impl WindowlessRenderer {
    /// Creates a new windowless renderer with the specified size.
    ///
    /// This is an async function that will:
    /// 1. Create a wgpu instance
    /// 2. Request an adapter (without a compatible surface)
    /// 3. Request a device and queue
    /// 4. Create the initial render target texture
    /// 5. Initialize Galileo's WgpuRenderer
    pub async fn new(size: Size<u32>) -> Result<Self, WindowlessRendererError> {
        if size.width() == 0 || size.height() == 0 {
            return Err(WindowlessRendererError::InvalidSize(
                size.width(),
                size.height(),
            ));
        }

        // Create wgpu instance
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Request adapter without a surface (windowless)
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None, // No surface needed for windowless rendering
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| WindowlessRendererError::AdapterCreationFailed)?;

        // Request device and queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Galileo Flutter Windowless Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let mut renderer = Self {
            device,
            queue,
            galileo_renderer: None,
            target_texture: None,
            target_texture_view: None,
            size,
        };

        // Create the initial render target texture
        renderer.create_target_texture()?;

        // Initialize Galileo renderer
        renderer.init_galileo_renderer()?;

        Ok(renderer)
    }

    /// Creates the target texture that Galileo will render to.
    ///
    /// The texture uses RGBA8UnormSrgb format to match Flutter's expectations
    /// and includes both RENDER_ATTACHMENT and COPY_SRC usage flags.
    fn create_target_texture(&mut self) -> Result<(), WindowlessRendererError> {
        let texture_desc = TextureDescriptor {
            size: Extent3d {
                width: self.size.width(),
                height: self.size.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb, // RGBA format for Flutter compatibility
            usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
            label: Some("Galileo Flutter Render Target"),
            view_formats: &[],
        };

        let texture = self.device.create_texture(&texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.target_texture = Some(texture);
        self.target_texture_view = Some(texture_view);

        Ok(())
    }

    /// Initializes Galileo's WgpuRenderer with our device and texture size.
    fn init_galileo_renderer(&mut self) -> Result<(), WindowlessRendererError> {
        let galileo_renderer = WgpuRenderer::new_with_device_and_texture(
            self.device.clone(),
            self.queue.clone(),
            self.size,
        );

        self.galileo_renderer = Some(galileo_renderer);
        Ok(())
    }

    /// Resizes the renderer and recreates the target texture.
    pub fn resize(&mut self, new_size: Size<u32>) -> Result<(), WindowlessRendererError> {
        if new_size.width() == 0 || new_size.height() == 0 {
            return Err(WindowlessRendererError::InvalidSize(
                new_size.width(),
                new_size.height(),
            ));
        }

        if self.size == new_size {
            return Ok(()); // No need to resize
        }

        self.size = new_size;

        // Recreate target texture with new size
        self.create_target_texture()?;

        // Resize Galileo renderer
        if let Some(ref mut galileo_renderer) = self.galileo_renderer {
            galileo_renderer.resize(new_size);
        }

        Ok(())
    }

    /// Gets a reference to the wgpu device.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Gets a reference to the wgpu queue.
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Gets a reference to the Galileo renderer.
    pub fn galileo_renderer(&self) -> Option<&WgpuRenderer> {
        self.galileo_renderer.as_ref()
    }

    /// Gets a mutable reference to the Galileo renderer.
    pub fn galileo_renderer_mut(&mut self) -> Option<&mut WgpuRenderer> {
        self.galileo_renderer.as_mut()
    }

    /// Gets the target texture view for rendering.
    pub fn target_texture_view(&self) -> Option<&TextureView> {
        self.target_texture_view.as_ref()
    }

    /// Gets the target texture for reading pixels.
    pub fn target_texture(&self) -> Option<&Texture> {
        self.target_texture.as_ref()
    }

    /// Gets the current size of the renderer.
    pub fn size(&self) -> Size<u32> {
        self.size
    }

    /// Renders the given Galileo map to the target texture.
    pub fn render_map(&mut self, map: &galileo::Map) -> Result<(), WindowlessRendererError> {
        let galileo_renderer = self
            .galileo_renderer
            .as_mut()
            .ok_or(WindowlessRendererError::NotInitialized)?;

        let texture_view = self
            .target_texture_view
            .as_ref()
            .ok_or(WindowlessRendererError::NotInitialized)?;

        galileo_renderer.render_to_texture_view(map, texture_view);
        Ok(())
    }

    /// Creates a staging buffer for copying texture data to CPU memory.
    /// This buffer can be used to read the rendered pixels.
    pub fn create_staging_buffer(&self) -> wgpu::Buffer {
        let buffer_size = (4 * self.size.width() * self.size.height()) as wgpu::BufferAddress;

        self.device.create_buffer(&wgpu::BufferDescriptor {
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: Some("Galileo Flutter Staging Buffer"),
            mapped_at_creation: false,
        })
    }

    /// Copies the rendered texture to a staging buffer for CPU access.
    pub fn copy_texture_to_buffer(
        &self,
        staging_buffer: &wgpu::Buffer,
    ) -> Result<(), WindowlessRendererError> {
        let texture = self
            .target_texture
            .as_ref()
            .ok_or(WindowlessRendererError::NotInitialized)?;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Galileo Flutter Copy Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * self.size.width()),
                    rows_per_image: Some(self.size.height()),
                },
            },
            Extent3d {
                width: self.size.width(),
                height: self.size.height(),
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

/// Thread-safe wrapper for WindowlessRenderer.
/// This allows the renderer to be shared between the main thread and render thread.
pub type SharedWindowlessRenderer = Arc<Mutex<WindowlessRenderer>>;

impl std::fmt::Debug for WindowlessRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowlessRenderer")
            .field("size", &self.size)
            .field("has_galileo_renderer", &self.galileo_renderer.is_some())
            .field("has_target_texture", &self.target_texture.is_some())
            .finish()
    }
}
