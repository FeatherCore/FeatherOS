//! Configuration Resources
//!
//! Global configuration settings for rendering and window.

use crate::math::Color;

/// RenderConfig - Rendering configuration
///
/// Global settings that affect how the renderer operates.
#[derive(Clone, Debug)]
pub struct RenderConfig {
    /// Screen width
    pub width: u32,
    /// Screen height
    pub height: u32,
    /// Target FPS
    pub target_fps: u32,
    /// VSync enabled
    pub vsync: bool,
    /// Clear color
    pub clear_color: Color,
    /// Maximum draw calls per frame
    pub max_draw_calls: usize,
    /// Enable debug rendering
    pub debug_render: bool,
}

impl RenderConfig {
    /// Create a new render config with default values
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            target_fps: 60,
            vsync: true,
            clear_color: Color::BLACK,
            max_draw_calls: 1000,
            debug_render: false,
        }
    }

    /// Create a config with specific dimensions
    pub fn with_size(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Self::new()
        }
    }

    /// Set dimensions
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Get aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }

    /// Get target frame time
    pub fn target_frame_time(&self) -> f32 {
        1.0 / self.target_fps as f32
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Resource for RenderConfig {}

/// WindowConfig - Window/Display configuration
///
/// Settings for the display window or framebuffer.
#[derive(Clone, Debug)]
pub struct WindowConfig {
    /// Window title (for windowed platforms)
    pub title: &'static str,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Fullscreen mode
    pub fullscreen: bool,
    /// Resizable window
    pub resizable: bool,
    /// Borderless window
    pub borderless: bool,
    /// MSAA samples
    pub msaa: u32,
}

impl WindowConfig {
    /// Create a new window config with default values
    pub fn new() -> Self {
        Self {
            title: "FHRE Application",
            width: 800,
            height: 600,
            fullscreen: false,
            resizable: false,
            borderless: false,
            msaa: 0,
        }
    }

    /// Create a config with specific dimensions
    pub fn with_size(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..Self::new()
        }
    }

    /// Set title
    pub fn with_title(mut self, title: &'static str) -> Self {
        self.title = title;
        self
    }

    /// Set fullscreen
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Set resizable
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Resource for WindowConfig {}
