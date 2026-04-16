//! Desktop Icon Module
//!
//! Manages desktop icons and their interactions.

use alloc::string::String;
use alloc::vec::Vec;
use fhre::math::{Color, Vec2, Rect};
use fhre::render_world::RenderCommand;

/// Desktop icon
pub struct DesktopIcon {
    /// Icon name
    pub name: String,
    /// Icon position
    pub position: Vec2,
    /// Icon size
    pub size: f32,
    /// Icon color
    pub color: Color,
    /// Is selected
    pub is_selected: bool,
    /// Click handler
    pub on_click: Option<fn()>,
}

impl DesktopIcon {
    /// Create a new desktop icon
    pub fn new(name: String, position: Vec2, size: f32) -> Self {
        Self {
            name,
            position,
            size,
            color: Color::rgb(200, 200, 200),
            is_selected: false,
            on_click: None,
        }
    }

    /// Get icon rectangle
    pub fn get_rect(&self) -> Rect {
        Rect::new(
            self.position.x - self.size / 2.0,
            self.position.y - self.size / 2.0,
            self.size,
            self.size,
        )
    }

    /// Check if point is inside icon
    pub fn contains(&self, point: Vec2) -> bool {
        self.get_rect().contains(point)
    }

    /// Handle click
    pub fn on_click(&self) {
        if let Some(handler) = self.on_click {
            handler();
        }
    }

    /// Generate render commands for this icon
    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        let rect = self.get_rect();

        // Draw icon background
        commands.push(RenderCommand::DrawRect {
            rect,
            color: if self.is_selected {
                Color::rgb(100, 150, 200)
            } else {
                self.color
            },
        });

        // Draw icon border
        let border_width = 2.0;
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(rect.x, rect.y, rect.width, border_width),
            color: Color::rgb(150, 150, 150),
        });
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(rect.x, rect.y + rect.height - border_width, rect.width, border_width),
            color: Color::rgb(150, 150, 150),
        });
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(rect.x, rect.y, border_width, rect.height),
            color: Color::rgb(150, 150, 150),
        });
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(rect.x + rect.width - border_width, rect.y, border_width, rect.height),
            color: Color::rgb(150, 150, 150),
        });

        commands
    }
}

/// Icon grid for organizing desktop icons
pub struct IconGrid {
    /// Screen size
    screen_size: Vec2,
    /// Grid cell size
    cell_size: f32,
    /// Grid spacing
    spacing: f32,
}

impl IconGrid {
    /// Create a new icon grid
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            screen_size,
            cell_size: 80.0,
            spacing: 20.0,
        }
    }

    /// Get grid position for an icon
    pub fn get_grid_position(&self, row: u32, col: u32) -> Vec2 {
        Vec2::new(
            self.spacing + col as f32 * (self.cell_size + self.spacing),
            self.spacing + row as f32 * (self.cell_size + self.spacing),
        )
    }

    /// Get grid cell at position
    pub fn get_cell_at(&self, position: Vec2) -> Option<(u32, u32)> {
        let col = (position.x / (self.cell_size + self.spacing)) as u32;
        let row = (position.y / (self.cell_size + self.spacing)) as u32;

        let max_cols = (self.screen_size.x / (self.cell_size + self.spacing)) as u32;
        let max_rows = (self.screen_size.y / (self.cell_size + self.spacing)) as u32;

        if col < max_cols && row < max_rows {
            Some((row, col))
        } else {
            None
        }
    }
}
