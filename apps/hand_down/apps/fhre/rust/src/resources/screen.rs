//! Screen Canvas Resource
//!
//! Defines the primary screen canvas (幕布) — a 3D object in the main world
//! that serves as the camera's projection plane.
//!
//! # Architecture
//!
//! The screen canvas is a flat plane in the 3D main world, positioned in front
//! of the camera. The camera captures the 3D scene and projects it onto this
//! canvas. The canvas content is then presented to the user via a presentation
//! window (X11, framebuffer, etc.).
//!
//! By default, the canvas is positioned at the camera's look-at target and
//! sized to fill the presentation window 1:1. The canvas can be manipulated
//! (moved closer/further from camera, rotated, etc.) to change what the user
//! sees in the presentation window.
//!
//! ```text
//! Camera (3D) ──looks at──▶ Screen Canvas (幕布, 3D plane)
//!                                │
//!                                │ 1:1 default overlap
//!                                ▼
//!                           Presentation Window (X11/FB)
//! ```

use crate::math::{Rect, Vec2, Vec3};
use crate::node::Transform3D;

/// Primary Screen Canvas Resource
///
/// The screen canvas is the camera's projection plane in the 3D main world.
/// It defines where the camera's output is rendered and how it maps to the
/// presentation window.
///
/// # Default Behavior
///
/// - Canvas is positioned at z=0 (camera's default look-at target)
/// - Canvas size matches the presentation window (1:1 pixel mapping)
/// - Camera looks at the canvas center
///
/// # Manipulation
///
/// The canvas has a `Transform3D` that can be modified:
/// - Move closer to camera → zoom-in effect on presentation window
/// - Move further from camera → zoom-out effect
/// - Rotate → tilted view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimaryScreen {
    /// Screen width in pixels (presentation window size)
    pub width: u32,
    /// Screen height in pixels (presentation window size)
    pub height: u32,
    /// Full screen viewport
    pub viewport: Rect,
    /// Pixel density (for high-DPI displays)
    pub pixel_density: f32,
    /// Whether the screen is fullscreen
    pub fullscreen: bool,
    /// 3D transform of the canvas in the main world
    ///
    /// Default: positioned at (width/2, height/2, 0) — the camera's look-at target.
    /// The canvas plane faces the camera (normal along +Z).
    pub transform: Transform3D,
}

impl PrimaryScreen {
    /// Create a new primary screen canvas with given dimensions
    ///
    /// The canvas is positioned at (width/2, height/2, 0) by default,
    /// which aligns with the camera's default look-at target.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            viewport: Rect::new(0.0, 0.0, width as f32, height as f32),
            pixel_density: 1.0,
            fullscreen: false,
            transform: Transform3D::from_position(
                width as f32 / 2.0,
                height as f32 / 2.0,
                0.0,
            ),
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

    /// Get screen center point (2D, canvas-local)
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

    /// Set custom 3D transform for the canvas (chainable)
    pub fn with_transform(mut self, transform: Transform3D) -> Self {
        self.transform = transform;
        self
    }

    /// Move the canvas closer to the camera (zoom-in effect)
    ///
    /// Increases the Z component of the canvas position.
    pub fn move_closer(&mut self, delta: f32) {
        self.transform.position.z += delta;
    }

    /// Move the canvas further from the camera (zoom-out effect)
    ///
    /// Decreases the Z component of the canvas position.
    pub fn move_further(&mut self, delta: f32) {
        self.transform.position.z -= delta;
    }

    /// Get the canvas position in 3D world space
    pub fn position(&self) -> Vec3 {
        self.transform.position
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
        Self::new(800, 600)
    }
}

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
