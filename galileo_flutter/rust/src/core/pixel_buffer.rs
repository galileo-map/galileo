//! Pixel buffer management for copying texture data between Galileo and Flutter.
//!
//! This module handles the conversion and copying of pixel data from Galileo's
//! wgpu textures to Flutter's texture system using irondash. It ensures proper
//! format conversion (to RGBA) and efficient memory management.

use parking_lot::Mutex;
use std::sync::Arc;
use wgpu::{Buffer, Device, Queue};

use crate::api::dart_types::MapSize;

/// Error types for pixel buffer operations.
#[derive(Debug, thiserror::Error)]
pub enum PixelBufferError {
    #[error("Buffer mapping failed")]
    MappingFailed,
    #[error("Invalid buffer size for dimensions {width}x{height}")]
    InvalidBufferSize { width: u32, height: u32 },
    #[error("Format conversion error: {0}")]
    FormatConversionError(String),
    #[error("Irondash texture error: {0}")]
    IrondashError(String),
    #[error("Buffer not ready for reading")]
    BufferNotReady,
}

/// Manages pixel buffers for copying texture data from wgpu to Flutter.
///
/// This struct handles:
/// - Creating and managing wgpu staging buffers
/// - Converting pixel formats (ensuring RGBA output)
/// - Interfacing with irondash texture system
/// - Memory-efficient buffer reuse
pub struct PixelBuffer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    staging_buffer: Option<Buffer>,
    size: MapSize,
    pixel_data: Vec<u8>,
}

impl PixelBuffer {
    /// Creates a new pixel buffer for the given size.
    ///
    /// The buffer is initially empty and will be allocated when first used.
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, size: MapSize) -> Self {
        Self {
            device,
            queue,
            staging_buffer: None,
            size,
            pixel_data: Vec::new(),
        }
    }

    /// Resizes the pixel buffer to accommodate a new texture size.
    ///
    /// This will recreate the staging buffer if the size has changed.
    pub fn resize(&mut self, new_size: MapSize) -> Result<(), PixelBufferError> {
        if self.size != new_size {
            self.size = new_size;
            self.staging_buffer = None; // Will be recreated on next use
            self.pixel_data.clear();
        }
        Ok(())
    }

    /// Gets or creates a staging buffer for the current size.
    fn ensure_staging_buffer(&mut self) -> Result<&Buffer, PixelBufferError> {
        if self.staging_buffer.is_none() {
            let buffer_size = (4 * self.size.width * self.size.height) as wgpu::BufferAddress;

            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                label: Some("Galileo Flutter Pixel Staging Buffer"),
                mapped_at_creation: false,
            });

            self.staging_buffer = Some(buffer);
        }

        Ok(self.staging_buffer.as_ref().unwrap())
    }

    /// Copies texture data to the staging buffer.
    ///
    /// This creates a command encoder and copies the texture data to our
    /// staging buffer where it can be read by the CPU.
    pub fn copy_from_texture(&mut self, texture: &wgpu::Texture) -> Result<(), PixelBufferError> {
        // Copy size values to avoid borrowing issues
        let width = self.size.width;
        let height = self.size.height;

        // Ensure staging buffer exists first
        self.ensure_staging_buffer()?;

        // Now get a reference to the buffer and device separately
        let staging_buffer = self.staging_buffer.as_ref().unwrap();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Galileo Flutter Pixel Copy Encoder"),
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
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));
        Ok(())
    }

    /// Reads the pixel data from the staging buffer.
    ///
    /// This maps the buffer for reading and copies the data to our internal
    /// pixel data vector. The data is guaranteed to be in RGBA format.
    pub async fn read_pixels(&mut self) -> Result<&[u8], PixelBufferError> {
        let staging_buffer = self
            .staging_buffer
            .as_ref()
            .ok_or(PixelBufferError::BufferNotReady)?;

        let buffer_slice = staging_buffer.slice(..);

        // Create a channel for async buffer mapping
        let (sender, receiver) = futures::channel::oneshot::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        // Poll the device to process the mapping request
        let _ = self.device.poll(wgpu::MaintainBase::Wait);

        // Wait for mapping to complete
        receiver
            .await
            .map_err(|_| PixelBufferError::MappingFailed)?
            .map_err(|_| PixelBufferError::MappingFailed)?;

        // Get the mapped data
        let data = buffer_slice.get_mapped_range();

        // Copy data to our internal buffer
        let expected_size = (4 * self.size.width * self.size.height) as usize;
        if data.len() != expected_size {
            return Err(PixelBufferError::InvalidBufferSize {
                width: self.size.width,
                height: self.size.height,
            });
        }

        self.pixel_data.clear();
        self.pixel_data.extend_from_slice(&data);

        // Unmap the buffer
        drop(data);
        staging_buffer.unmap();

        // Convert to RGBA if needed (wgpu texture should already be RGBA8UnormSrgb)
        self.ensure_rgba_format()?;

        Ok(&self.pixel_data)
    }

    /// Ensures the pixel data is in RGBA format.
    ///
    /// This is a placeholder for format conversion logic. Since we're using
    /// RGBA8UnormSrgb textures, this should be a no-op, but we include it
    /// for future compatibility.
    fn ensure_rgba_format(&mut self) -> Result<(), PixelBufferError> {
        // TODO: Implement format conversion if needed
        // For now, we assume the texture is already in RGBA8UnormSrgb format
        Ok(())
    }

    /// Gets the current size of the pixel buffer.
    pub fn size(&self) -> MapSize {
        self.size
    }

    /// Gets the expected size in bytes for the current buffer.
    pub fn expected_byte_size(&self) -> usize {
        (4 * self.size.width * self.size.height) as usize
    }

    /// Gets a reference to the current pixel data.
    ///
    /// Returns None if no pixel data has been read yet.
    pub fn pixel_data(&self) -> Option<&[u8]> {
        if self.pixel_data.is_empty() {
            None
        } else {
            Some(&self.pixel_data)
        }
    }

    /// Clears the internal pixel data buffer.
    pub fn clear(&mut self) {
        self.pixel_data.clear();
    }
}

impl std::fmt::Debug for PixelBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PixelBuffer")
            .field("size", &self.size)
            .field("has_staging_buffer", &self.staging_buffer.is_some())
            .field("pixel_data_len", &self.pixel_data.len())
            .field("expected_byte_size", &self.expected_byte_size())
            .finish()
    }
}

/// Shared pixel buffer that can be used across threads.
pub type SharedPixelBuffer = Arc<Mutex<PixelBuffer>>;

/// Utility functions for pixel format conversion.
pub mod format_utils {
    /// Converts BGRA pixel data to RGBA format in-place.
    pub fn bgra_to_rgba(pixels: &mut [u8]) {
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Swap B and R channels
        }
    }

    /// Converts RGB pixel data to RGBA format by adding alpha channel.
    pub fn rgb_to_rgba(rgb_pixels: &[u8]) -> Vec<u8> {
        let mut rgba_pixels = Vec::with_capacity(rgb_pixels.len() / 3 * 4);

        for chunk in rgb_pixels.chunks_exact(3) {
            rgba_pixels.extend_from_slice(chunk);
            rgba_pixels.push(255); // Full alpha
        }

        rgba_pixels
    }

    /// Validates that pixel data has the expected size for given dimensions.
    pub fn validate_pixel_data_size(pixels: &[u8], width: u32, height: u32, channels: u32) -> bool {
        let expected_size = (width * height * channels) as usize;
        pixels.len() == expected_size
    }
}

#[cfg(test)]
mod tests {
    use super::format_utils::*;

    #[test]
    fn test_bgra_to_rgba_conversion() {
        let mut pixels = vec![
            255, 0, 0, 255, // Blue pixel in BGRA
            0, 255, 0, 255, // Green pixel in BGRA
            0, 0, 255, 255, // Red pixel in BGRA
        ];

        bgra_to_rgba(&mut pixels);

        assert_eq!(
            pixels,
            vec![
                0, 0, 255, 255, // Red pixel in RGBA
                0, 255, 0, 255, // Green pixel in RGBA
                255, 0, 0, 255, // Blue pixel in RGBA
            ]
        );
    }

    #[test]
    fn test_rgb_to_rgba_conversion() {
        let rgb_pixels = vec![255, 0, 0, 0, 255, 0, 0, 0, 255];
        let rgba_pixels = rgb_to_rgba(&rgb_pixels);

        assert_eq!(
            rgba_pixels,
            vec![
                255, 0, 0, 255, // Red with full alpha
                0, 255, 0, 255, // Green with full alpha
                0, 0, 255, 255, // Blue with full alpha
            ]
        );
    }

    #[test]
    fn test_validate_pixel_data_size() {
        let pixels = vec![0u8; 1920 * 1080 * 4];
        assert!(validate_pixel_data_size(&pixels, 1920, 1080, 4));

        let wrong_size_pixels = vec![0u8; 1920 * 1080 * 3];
        assert!(!validate_pixel_data_size(&wrong_size_pixels, 1920, 1080, 4));
    }
}
