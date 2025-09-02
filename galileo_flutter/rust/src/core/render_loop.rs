//! FPS-controlled render loop for Galileo Flutter integration.
//!
//! This module implements a render loop that runs on a separate thread and
//! renders the Galileo map at a configurable frame rate. It handles:
//! - Timing control for consistent FPS
//! - Texture copying from Galileo to Flutter
//! - Render state management (start/stop/pause)

use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio::time::{interval, MissedTickBehavior};

use crate::api::dart_types::{RenderConfig, MapSize};


/// Commands that can be sent to the render loop.
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// Start the render loop
    Start,
    /// Stop the render loop
    Stop,
    /// Pause rendering (but keep loop alive)
    Pause,
    /// Resume rendering
    Resume,
    /// Change FPS setting
    SetFps(u32),
    /// Resize the rendering target
    Resize(MapSize),
    /// Request a single frame render
    RequestFrame,
}

/// Current state of the render loop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderState {
    Stopped,
    Running,
    Paused,
}

/// Error types for render loop operations.
#[derive(Debug, thiserror::Error)]
pub enum RenderLoopError {
    #[error("Render loop is not running")]
    NotRunning,
    #[error("Invalid FPS value: {0}")]
    InvalidFps(u32),
    #[error("Channel send error: {0}")]
    ChannelSendError(String),
    #[error("Renderer error: {0}")]
    RendererError(String),
}

/// FPS-controlled render loop that manages Galileo map rendering.
///
/// The render loop runs on a separate async task and can be controlled
/// via commands sent through a channel. It maintains a consistent frame
/// rate and handles texture updates to Flutter.
pub struct RenderLoop {
    command_sender: mpsc::UnboundedSender<RenderCommand>,
    state: Arc<Mutex<RenderState>>,
    config: Arc<Mutex<RenderConfig>>,
}

impl RenderLoop {
    /// Creates a new render loop with the given configuration.
    ///
    /// The render loop starts in a stopped state and must be explicitly
    /// started using the `start()` method.
    pub fn new(config: RenderConfig) -> Self {
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let state = Arc::new(Mutex::new(RenderState::Stopped));
        let config = Arc::new(Mutex::new(config));

        // Start the render loop task
        let task_state = state.clone();
        let task_config = config.clone();
        tokio::spawn(async move {
            Self::render_task(command_receiver, task_state, task_config).await;
        });

        Self {
            command_sender,
            state,
            config,
        }
    }

    /// Starts the render loop.
    pub fn start(&self) -> Result<(), RenderLoopError> {
        self.send_command(RenderCommand::Start)
    }

    /// Stops the render loop.
    pub fn stop(&self) -> Result<(), RenderLoopError> {
        self.send_command(RenderCommand::Stop)
    }

    /// Pauses the render loop (keeps loop alive but stops rendering).
    pub fn pause(&self) -> Result<(), RenderLoopError> {
        self.send_command(RenderCommand::Pause)
    }

    /// Resumes the render loop from paused state.
    pub fn resume(&self) -> Result<(), RenderLoopError> {
        self.send_command(RenderCommand::Resume)
    }

    /// Sets the target FPS for the render loop.
    pub fn set_fps(&self, fps: u32) -> Result<(), RenderLoopError> {
        if fps == 0 || fps > 120 {
            return Err(RenderLoopError::InvalidFps(fps));
        }
        self.send_command(RenderCommand::SetFps(fps))
    }

    /// Resizes the rendering target.
    pub fn resize(&self, size: MapSize) -> Result<(), RenderLoopError> {
        self.send_command(RenderCommand::Resize(size))
    }

    /// Requests a single frame to be rendered immediately.
    pub fn request_frame(&self) -> Result<(), RenderLoopError> {
        self.send_command(RenderCommand::RequestFrame)
    }

    /// Gets the current state of the render loop.
    pub fn state(&self) -> RenderState {
        *self.state.lock()
    }

    /// Gets the current render configuration.
    pub fn config(&self) -> RenderConfig {
        *self.config.lock()
    }

    /// Sends a command to the render loop.
    fn send_command(&self, command: RenderCommand) -> Result<(), RenderLoopError> {
        self.command_sender
            .send(command)
            .map_err(|e| RenderLoopError::ChannelSendError(e.to_string()))
    }

    /// Main render loop task that runs on a separate async task.
    async fn render_task(
        mut command_receiver: mpsc::UnboundedReceiver<RenderCommand>,
        state: Arc<Mutex<RenderState>>,
        config: Arc<Mutex<RenderConfig>>,
    ) {
        let mut current_fps = config.lock().fps;
        let mut frame_duration = Duration::from_secs_f64(1.0 / current_fps as f64);
        let mut interval_timer = interval(frame_duration);
        interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                // Handle commands from the main thread
                command = command_receiver.recv() => {
                    match command {
                        Some(RenderCommand::Start) => {
                            *state.lock() = RenderState::Running;
                            log::info!("Render loop started");
                        }
                        Some(RenderCommand::Stop) => {
                            *state.lock() = RenderState::Stopped;
                            log::info!("Render loop stopped");
                            break; // Exit the loop
                        }
                        Some(RenderCommand::Pause) => {
                            *state.lock() = RenderState::Paused;
                            log::info!("Render loop paused");
                        }
                        Some(RenderCommand::Resume) => {
                            *state.lock() = RenderState::Running;
                            log::info!("Render loop resumed");
                        }
                        Some(RenderCommand::SetFps(fps)) => {
                            let _ = current_fps;
                            current_fps = fps;
                            frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
                            interval_timer = interval(frame_duration);
                            interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

                            let mut cfg = config.lock();
                            cfg.fps = fps;
                            log::info!("Render loop FPS changed to {}", fps);
                        }
                        Some(RenderCommand::Resize(size)) => {
                            // TODO: Handle resize in Phase 2
                            log::info!("Render loop resize to {}x{}", size.width, size.height);
                        }
                        Some(RenderCommand::RequestFrame) => {
                            // Render a single frame immediately
                            if *state.lock() != RenderState::Stopped {
                                Self::render_frame().await;
                            }
                        }
                        None => {
                            // Channel closed, exit the loop
                            break;
                        }
                    }
                }

                // Handle frame timing
                _ = interval_timer.tick() => {
                    let current_state = *state.lock();
                    if current_state == RenderState::Running {
                        Self::render_frame().await;
                    }
                }
            }
        }

        log::info!("Render loop task exited");
    }

    /// Renders a single frame.
    /// This is a placeholder implementation that will be completed in Phase 2.
    async fn render_frame() {
        // TODO: Implement actual rendering in Phase 2
        // This should:
        // 1. Render the Galileo map to the target texture
        // 2. Copy pixels from the wgpu texture to a staging buffer
        // 3. Map the buffer and read the pixel data
        // 4. Convert to RGBA format if needed
        // 5. Update the Flutter texture via irondash

        log::trace!("Rendering frame (placeholder)");
    }
}

impl Drop for RenderLoop {
    fn drop(&mut self) {
        // Ensure the render loop stops when dropped
        let _ = self.send_command(RenderCommand::Stop);
    }
}

/// Statistics about render loop performance.
#[derive(Debug, Clone, Copy)]
pub struct RenderStats {
    /// Current frames per second being achieved
    pub actual_fps: f64,
    /// Target frames per second
    pub target_fps: u32,
    /// Average frame render time in milliseconds
    pub avg_frame_time_ms: f64,
    /// Number of frames rendered since last reset
    pub frame_count: u64,
    /// Number of dropped frames due to timing issues
    pub dropped_frames: u64,
}

impl RenderStats {
    pub fn new(target_fps: u32) -> Self {
        Self {
            actual_fps: 0.0,
            target_fps,
            avg_frame_time_ms: 0.0,
            frame_count: 0,
            dropped_frames: 0,
        }
    }
}
