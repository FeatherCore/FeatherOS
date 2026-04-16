//! Wallpaper Module
//!
//! Manages desktop wallpaper and background effects.

use fhre::math::{Color, Vec2, Rect};
use fhre::render_world::RenderCommand;
use alloc::vec::Vec;

/// Wallpaper configuration
pub struct WallpaperConfig {
    /// Background color
    pub background_color: Color,
    /// Gradient start color (if using gradient)
    pub gradient_start: Color,
    /// Gradient end color (if using gradient)
    pub gradient_end: Color,
    /// Use gradient
    pub use_gradient: bool,
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        Self {
            background_color: Color::rgb(30, 30, 30),
            gradient_start: Color::rgb(20, 30, 50),
            gradient_end: Color::rgb(50, 40, 60),
            use_gradient: true,
        }
    }
}

/// Wallpaper
pub struct Wallpaper {
    /// Configuration
    config: WallpaperConfig,
}

impl Wallpaper {
    /// Create a new wallpaper
    pub fn new(config: WallpaperConfig) -> Self {
        Self { config }
    }

    /// Generate render commands for the wallpaper
    pub fn generate_render_commands(&self, screen_size: Vec2) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        if self.config.use_gradient {
            // Simple gradient effect using multiple horizontal lines
            let steps = 20;
            let step_height = screen_size.y / steps as f32;

            for i in 0..steps {
                let t = i as f32 / steps as f32;
                let color = Color::rgb(
                    (self.config.gradient_start.r as f32 * (1.0 - t) + self.config.gradient_end.r as f32 * t) as u8,
                    (self.config.gradient_start.g as f32 * (1.0 - t) + self.config.gradient_end.g as f32 * t) as u8,
                    (self.config.gradient_start.b as f32 * (1.0 - t) + self.config.gradient_end.b as f32 * t) as u8,
                );

                commands.push(RenderCommand::DrawRect {
                    rect: Rect::new(0.0, i as f32 * step_height, screen_size.x, step_height + 1.0),
                    color,
                });
            }
        } else {
            // Solid color
            commands.push(RenderCommand::DrawRect {
                rect: Rect::new(0.0, 0.0, screen_size.x, screen_size.y),
                color: self.config.background_color,
            });
        }

        commands
    }
}

impl Default for Wallpaper {
    fn default() -> Self {
        Self::new(WallpaperConfig::default())
    }
}
