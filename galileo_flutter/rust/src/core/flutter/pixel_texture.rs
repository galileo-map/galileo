

/// Creates a Flutter texture using irondash with proper provider.
async fn create_flutter_texture(
    engine_handle: i64,
    provider: Arc<TexturePixelProvider>,
) -> anyhow::Result<Texture <BoxedPixelData>> {
    // Create boxed provider for irondash
    let boxed_provider: Arc<dyn PayloadProvider<BoxedPixelData>> = provider;

    let texture = Texture::new_with_provider(engine_handle, boxed_provider)
        .map_err(|e| anyhow::anyhow!("Failed to create Flutter texture: {:?}", e))?;

    Ok(texture)
}

/// Helper function to safely read pixels without holding mutex across await
async fn read_pixels_from_buffer(pixel_buffer: Arc<Mutex<PixelBuffer>>) -> anyhow::Result<Vec<u8>> {
    // Use spawn_blocking to handle the async operation safely
    let handle = TOKIO_RUNTIME.get().unwrap().handle().clone();
    let result = tokio::task::spawn_blocking(move || {
        // Get a runtime handle for the blocking context
        handle.block_on(async move {
            let mut buffer = pixel_buffer.lock().unwrap();
            let pixels = buffer.read_pixels().await?;
            Ok::<Vec<u8>, anyhow::Error>(pixels.to_vec())
        })
    })
    .await;

    result.map_err(|e| anyhow::anyhow!("Join error: {}", e))?
}



/// Renders a single frame for the session.
async fn render_frame(session: &Arc<MapSession>) -> anyhow::Result<()> {
    // Render the map to wgpu texture
    {
        let mut renderer = session.renderer.lock().unwrap();
        let map = session.map.lock().unwrap();

        renderer
            .render_map(&map)
            .map_err(|e| anyhow::anyhow!("Failed to render map: {}", e))?;
    }

    // Copy texture to staging buffer
    let target_texture = {
        let renderer = session.renderer.lock().unwrap();
        renderer
            .target_texture()
            .ok_or_else(|| anyhow::anyhow!("No target texture available"))?
            .clone()
    };

    {
        let mut pixel_buffer = session.pixel_buffer.lock().unwrap();
        pixel_buffer
            .copy_from_texture(&target_texture)
            .map_err(|e| anyhow::anyhow!("Failed to copy texture to buffer: {}", e))?;
    }

    // Read pixels from staging buffer (use helper to avoid async mutex issues)
    let pixels = read_pixels_from_buffer(session.pixel_buffer.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to read pixels: {}", e))?;

    // Update texture provider
    session.texture_provider.update_pixels(pixels);

    // Mark frame available for Flutter
    {
        let flutter_texture = session.flutter_texture.lock().unwrap();
        flutter_texture
            .mark_frame_available()
            .map_err(|e| anyhow::anyhow!("Failed to mark frame available: {:?}", e))?;
    }

    Ok(())
}


/// Resizes the rendering session.
async fn resize_session(session: &Arc<MapSession>, new_size: MapSize) -> anyhow::Result<()> {
    info!(
        "Resizing session {} to {}x{}",
        session.session_id, new_size.width, new_size.height
    );

    // Resize renderer
    {
        let mut renderer = session.renderer.lock().unwrap();
        let size = Size::new(new_size.width, new_size.height);
        renderer
            .resize(size)
            .map_err(|e| anyhow::anyhow!("Failed to resize renderer: {}", e))?;
    }

    // Resize pixel buffer
    {
        let mut pixel_buffer = session.pixel_buffer.lock().unwrap();
        pixel_buffer
            .resize(new_size)
            .map_err(|e| anyhow::anyhow!("Failed to resize pixel buffer: {}", e))?;
    }

    // Resize texture provider
    session.texture_provider.resize(new_size);

    // Trigger render to fill new size
    render_frame(session).await?;

    Ok(())
}