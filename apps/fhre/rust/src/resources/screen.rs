//! Screen Resource
//!
//! Defines the primary screen and viewport management.
//! The primary screen is the default render target for FHRE.

use crate::math::{Rect, Vec2};

/// Primary Screen Resource
///
/// This is the default render target for FHRE.
/// It defines the main window/screen dimensions and properties.
/// All rendering happens to this screen by default unless otherwise specified.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimaryScreen {
    /// Screen width in pixels
    pub width: u32,
    /// Screen height in pixels
    pub height: u32,
    /// Full screen viewport
    pub viewport: Rect,
    /// Pixel density (for high-DPI displays)
    pub pixel_density: f32,
    /// Whether the screen is fullscreen
    pub fullscreen: bool,
}

impl PrimaryScreen {
    /// Create a new primary screen with given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            viewport: Rect::new(0.0, 0.0, width as f32, height as f32),
            pixel_density: 1.0,
            fullscreen: false,
        }
    }

    /// Get screen dimensions as (width, height)
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get screen size as Vec2
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    /// Get screen center point
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.width as f32 / 2.0, self.height as f32 / 2.0)
    }

    /// Get aspect ratio (width / height)
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Check if a point is within screen bounds
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= 0.0
            && point.x < self.width as f32
            && point.y >= 0.0
            && point.y < self.height as f32
    }

    /// Convert normalized coordinates (0-1) to screen coordinates
    pub fn normalized_to_screen(&self, normalized: Vec2) -> Vec2 {
        Vec2::new(
            normalized.x * self.width as f32,
            normalized.y * self.height as f32,
        )
    }

    /// Convert screen coordinates to normalized coordinates (0-1)
    pub fn screen_to_normalized(&self, screen: Vec2) -> Vec2 {
        Vec2::new(
            screen.x / self.width as f32,
            screen.y / self.height as f32,
        )
    }

    /// Set pixel density for high-DPI displays
    pub fn with_pixel_density(mut self, density: f32) -> Self {
        self.pixel_density = density;
        self
    }

    /// Set fullscreen mode
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Resize the screen (e.g., when window is resized)
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.viewport = Rect::new(0.0, 0.0, width as f32, height as f32);
    }
}

impl Default for PrimaryScreen {
    fn default() -> Self {
        // Default to 800x600, will be overridden by actual display
        Self::new(800, 600)
    }
}

// PrimaryScreen is a Resource
impl crate::resources::Resource for PrimaryScreen {}

/// Screen Coordinate Helper
///
/// Helper struct for converting between different coordinate systems
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenCoordinate {
    /// Pixel coordinates (0,0) at top-left
    Pixels(Vec2),
    /// Normalized coordinates (0-1) from top-left
    Normalized(Vec2),
    /// Center-relative coordinates (0,0) at screen center
    CenterRelative(Vec2),
}

impl ScreenCoordinate {
    /// Convert to pixel coordinates
    pub fn to_pixels(&self, screen: &PrimaryScreen) -> Vec2 {
        match self {
            ScreenCoordinate::Pixels(p) => *p,
            ScreenCoordinate::Normalized(n) => screen.normalized_to_screen(*n),
            ScreenCoordinate::CenterRelative(c) => {
                let center = screen.center();
                Vec2::new(center.x + c.x, center.y + c.y)
            }
        }
    }

    /// Create from pixel coordinates
    pub fn from_pixels(x: f32, y: f32) -> Self {
        ScreenCoordinate::Pixels(Vec2::new(x, y))
    }

    /// Create from normalized coordinates (0-1)
    pub fn from_normalized(x: f32, y: f32) -> Self {
        ScreenCoordinate::Normalized(Vec2::new(x, y))
    }

    /// Create from center-relative coordinates
    pub fn from_center(x: f32, y: f32) -> Self {
        ScreenCoordinate::CenterRelative(Vec2::new(x, y))
    }
}

/// Viewport Configuration
///
/// Defines a sub-region of the screen for rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportConfig {
    /// Viewport rectangle in screen coordinates
    pub rect: Rect,
    /// Whether to clear this viewport before rendering
    pub clear: bool,
    /// Clear color
    pub clear_color: crate::math::Color,
}

impl ViewportConfig {
    /// Create a full-screen viewport
    pub fn fullscreen(screen: &PrimaryScreen) -> Self {
        Self {
            rect: screen.viewport,
            clear: true,
            clear_color: crate::math::Color::BLACK,
        }
    }

    /// Create a viewport from normalized coordinates (0-1)
    pub fn from_normalized(
        screen: &PrimaryScreen,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            rect: Rect::new(
                x * screen.width as f32,
                y * screen.height as f32,
                width * screen.width as f32,
                height * screen.height as f32,
            ),
            clear: true,
            clear_color: crate::math::Color::BLACK,
        }
    }

    /// Set whether to clear this viewport
    pub fn with_clear(mut self, clear: bool) -> Self {
        self.clear = clear;
        self
    }

    /// Set clear color
    pub fn with_clear_color(mut self, color: crate::math::Color) -> Self {
        self.clear_color = color;
        self
    }
}
