//! Application Launcher Module
//!
//! Provides an application launcher/menu for starting applications.

use alloc::string::String;
use alloc::vec::Vec;
use fhre::math::{Color, Vec2, Rect};
use fhre::render_world::RenderCommand;
use crate::window::WindowId;

/// Application information
#[derive(Clone, Debug)]
pub struct AppInfo {
    /// Application name
    pub name: &'static str,
    /// Application description
    pub description: &'static str,
    /// Default window size
    pub default_size: Vec2,
    /// Initial window position
    pub initial_position: Vec2,
    /// Application icon (placeholder)
    pub icon_color: Color,
}

impl AppInfo {
    /// Create a new application info
    pub fn new(
        name: &'static str,
        description: &'static str,
        default_size: Vec2,
        initial_position: Vec2,
        icon_color: Color,
    ) -> Self {
        Self {
            name,
            description,
            default_size,
            initial_position,
            icon_color,
        }
    }
}

/// Launcher item
pub struct LauncherItem {
    /// Application info
    pub app_info: AppInfo,
    /// Item rectangle
    pub rect: Rect,
    /// Is hovered
    pub is_hovered: bool,
}

impl LauncherItem {
    /// Create a new launcher item
    pub fn new(app_info: AppInfo, rect: Rect) -> Self {
        Self {
            app_info,
            rect,
            is_hovered: false,
        }
    }

    /// Generate render commands
    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        let color = if self.is_hovered {
            Color::rgb(80, 80, 80)
        } else {
            Color::rgb(60, 60, 60)
        };

        commands.push(RenderCommand::DrawRect {
            rect: self.rect,
            color,
        });

        // Draw icon placeholder
        let icon_size = 32.0;
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(
                self.rect.x + 8.0,
                self.rect.y + (self.rect.height - icon_size) / 2.0,
                icon_size,
                icon_size,
            ),
            color: self.app_info.icon_color,
        });

        commands
    }
}

/// Application launcher
pub struct AppLauncher {
    /// Is launcher visible
    is_open: bool,
    /// Launcher rectangle
    rect: Rect,
    /// Launcher items
    items: Vec<LauncherItem>,
    /// Selected item index
    selected_index: Option<usize>,
}

impl AppLauncher {
    /// Create a new application launcher
    pub fn new() -> Self {
        let mut launcher = Self {
            is_open: false,
            rect: Rect::new(0.0, 0.0, 300.0, 400.0),
            items: Vec::new(),
            selected_index: None,
        };

        launcher.add_default_apps();
        launcher
    }

    /// Add default applications
    fn add_default_apps(&mut self) {
        let apps = [
            AppInfo::new(
                "Calculator",
                "Simple calculator application",
                Vec2::new(300.0, 400.0),
                Vec2::new(100.0, 100.0),
                Color::rgb(100, 200, 100),
            ),
            AppInfo::new(
                "Text Editor",
                "Edit text files",
                Vec2::new(600.0, 400.0),
                Vec2::new(150.0, 150.0),
                Color::rgb(100, 100, 200),
            ),
            AppInfo::new(
                "File Manager",
                "Browse and manage files",
                Vec2::new(500.0, 400.0),
                Vec2::new(200.0, 100.0),
                Color::rgb(200, 200, 100),
            ),
            AppInfo::new(
                "Settings",
                "System settings",
                Vec2::new(400.0, 500.0),
                Vec2::new(250.0, 150.0),
                Color::rgb(200, 100, 200),
            ),
            AppInfo::new(
                "Terminal",
                "Command line interface",
                Vec2::new(500.0, 300.0),
                Vec2::new(300.0, 200.0),
                Color::rgb(50, 50, 50),
            ),
        ];

        let item_height = 48.0;
        for (i, app) in apps.iter().enumerate() {
            let item = LauncherItem::new(
                app.clone(),
                Rect::new(
                    self.rect.x + 8.0,
                    self.rect.y + 8.0 + i as f32 * (item_height + 4.0),
                    self.rect.width - 16.0,
                    item_height,
                ),
            );
            self.items.push(item);
        }
    }

    /// Toggle launcher visibility
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Open launcher
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Close launcher
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Check if launcher is open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get selected application
    pub fn get_selected_app(&self) -> Option<&AppInfo> {
        self.selected_index.map(|index| &self.items[index].app_info)
    }

    /// Handle mouse move
    pub fn handle_mouse_move(&mut self, position: Vec2) {
        if !self.is_open {
            return;
        }

        for (i, item) in self.items.iter_mut().enumerate() {
            item.is_hovered = item.rect.contains(position);
            if item.is_hovered {
                self.selected_index = Some(i);
            }
        }
    }

    /// Handle click
    pub fn handle_click(&mut self, position: Vec2) -> Option<&AppInfo> {
        if !self.is_open {
            return None;
        }

        for item in &self.items {
            if item.rect.contains(position) {
                self.close();
                return Some(&item.app_info);
            }
        }

        None
    }

    /// Generate render commands
    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        if !self.is_open {
            return commands;
        }

        // Draw launcher background
        commands.push(RenderCommand::DrawRect {
            rect: self.rect,
            color: Color::rgb(40, 40, 40),
        });

        // Draw launcher border
        let border_width = 2.0;
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(self.rect.x, self.rect.y, self.rect.width, border_width),
            color: Color::rgb(100, 100, 100),
        });
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(self.rect.x, self.rect.y + self.rect.height - border_width, self.rect.width, border_width),
            color: Color::rgb(100, 100, 100),
        });
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(self.rect.x, self.rect.y, border_width, self.rect.height),
            color: Color::rgb(100, 100, 100),
        });
        commands.push(RenderCommand::DrawRect {
            rect: Rect::new(self.rect.x + self.rect.width - border_width, self.rect.y, border_width, self.rect.height),
            color: Color::rgb(100, 100, 100),
        });

        // Draw items
        for item in &self.items {
            commands.extend(item.generate_render_commands());
        }

        commands
    }
}

impl Default for AppLauncher {
    fn default() -> Self {
        Self::new()
    }
}
