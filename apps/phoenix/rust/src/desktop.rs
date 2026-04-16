//! Desktop Module
//!
//! Manages the desktop background, icons, and wallpaper.

use alloc::vec::Vec;
use fhre::math::{Color, Vec2, Rect};
use fhre::render_world::RenderCommand;
use crate::icon::{DesktopIcon, IconGrid};
use crate::wallpaper::Wallpaper;

/// Desktop configuration
pub struct DesktopConfig {
    /// Background color
    pub background_color: Color,
    /// Icon grid spacing
    pub icon_spacing: f32,
    /// Icon size
    pub icon_size: f32,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            background_color: Color::rgb(30, 30, 30),
            icon_spacing: 80.0,
            icon_size: 64.0,
        }
    }
}

/// Desktop environment
pub struct Desktop {
    /// Screen size
    screen_size: Vec2,
    /// Configuration
    config: DesktopConfig,
    /// Wallpaper
    wallpaper: Wallpaper,
    /// Icon grid
    icon_grid: IconGrid,
    /// Desktop icons
    icons: Vec<DesktopIcon>,
}

impl Desktop {
    /// Create a new desktop
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            screen_size,
            config: DesktopConfig::default(),
            wallpaper: Wallpaper::default(),
            icon_grid: IconGrid::new(screen_size),
            icons: Vec::new(),
        }
    }

    /// Initialize the desktop
    pub fn init(&mut self) {
        // Add default icons
        self.add_default_icons();
    }

    /// Add default desktop icons
    fn add_default_icons(&mut self) {
        // Add some example icons
        let icon_positions = [
            (100.0, 100.0, "Home"),
            (100.0, 200.0, "Documents"),
            (100.0, 300.0, "Settings"),
            (100.0, 400.0, "Trash"),
        ];

        for (x, y, name) in icon_positions.iter() {
            let icon = DesktopIcon::new(
                String::from(*name),
                Vec2::new(*x, *y),
                self.config.icon_size,
            );
            self.icons.push(icon);
        }
    }

    /// Update the desktop
    pub fn update(&mut self, _delta_time: f32) {
        // Update animations, etc.
    }

    /// Get icon at position
    pub fn get_icon_at(&self, position: Vec2) -> Option<&DesktopIcon> {
        self.icons.iter().find(|icon| icon.contains(position))
    }

    /// Generate render commands for the desktop
    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        // Draw wallpaper/background
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(0.0, 0.0, self.screen_size.x, self.screen_size.y),
            color: self.config.background_color,
        });

        // Draw icons
        for icon in &self.icons {
            commands.extend(icon.generate_render_commands());
        }

        commands
    }
}
