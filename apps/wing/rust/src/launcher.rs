//! Application launcher.

use alloc::vec::Vec;
use fhre::{Color, RenderCommand, Vec2};
use fhre::math::Rect;

use crate::theme::{ThemePalette, shell_palette};

/// Application information.
#[derive(Clone, Debug, PartialEq)]
pub struct AppInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub default_size: Vec2,
    pub initial_position: Vec2,
    pub icon_color: Color,
}

impl AppInfo {
    pub const fn new(
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

/// Launcher item.
pub struct LauncherItem {
    pub app_info: AppInfo,
    pub rect: Rect,
    pub is_hovered: bool,
}

impl LauncherItem {
    pub fn new(app_info: AppInfo, rect: Rect) -> Self {
        Self {
            app_info,
            rect,
            is_hovered: false,
        }
    }

    pub fn generate_render_commands(&self, palette: ThemePalette) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        let background = if self.is_hovered {
            palette.accent_hover
        } else {
            Color::rgb(42, 48, 64)
        };

        commands.push(RenderCommand::draw_rect_rounded(self.rect, background, 10.0));
        commands.push(RenderCommand::draw_rect_rounded(
            Rect::new(self.rect.x + 10.0, self.rect.y + 8.0, 32.0, 32.0),
            self.app_info.icon_color,
            8.0,
        ));
        commands.push(RenderCommand::draw_text(
            Vec2::new(self.rect.x + 52.0, self.rect.y + 13.0),
            self.app_info.name,
            Color::WHITE,
            16.0,
        ));
        commands.push(RenderCommand::draw_text(
            Vec2::new(self.rect.x + 52.0, self.rect.y + 29.0),
            self.app_info.description,
            palette.text_muted,
            12.0,
        ));
        commands
    }
}

/// Application launcher.
pub struct AppLauncher {
    is_open: bool,
    rect: Rect,
    items: Vec<LauncherItem>,
    selected_index: Option<usize>,
    theme: ThemePalette,
}

impl AppLauncher {
    pub fn new() -> Self {
        let mut launcher = Self {
            is_open: false,
            rect: Rect::new(16.0, 120.0, 320.0, 360.0),
            items: Vec::new(),
            selected_index: None,
            theme: shell_palette(),
        };
        launcher.add_default_apps();
        launcher.relayout_items();
        launcher
    }

    pub fn apps(&self) -> &[LauncherItem] {
        &self.items
    }

    pub fn find_app(&self, name: &str) -> Option<AppInfo> {
        self.items
            .iter()
            .find(|item| item.app_info.name == name)
            .map(|item| item.app_info.clone())
    }

    pub fn add_app(&mut self, app_info: AppInfo) {
        self.items.push(LauncherItem::new(app_info, Rect::ZERO));
        self.relayout_items();
    }

    pub fn set_layout(&mut self, screen_size: Vec2, taskbar_rect: Rect) {
        let width = 340.0;
        let max_height = (screen_size.y - taskbar_rect.height - 48.0).max(220.0);
        let desired_height = (self.items.len() as f32 * 56.0 + 20.0).min(max_height);
        self.rect = Rect::new(16.0, screen_size.y - taskbar_rect.height - desired_height - 12.0, width, desired_height);
        self.relayout_items();
    }

    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.theme = palette;
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.selected_index = None;
        for item in &mut self.items {
            item.is_hovered = false;
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn contains(&self, position: Vec2) -> bool {
        self.is_open && self.rect.contains(position)
    }

    pub fn handle_mouse_move(&mut self, position: Vec2) {
        if !self.is_open {
            return;
        }

        self.selected_index = None;
        for (index, item) in self.items.iter_mut().enumerate() {
            item.is_hovered = item.rect.contains(position);
            if item.is_hovered {
                self.selected_index = Some(index);
            }
        }
    }

    pub fn handle_click(&mut self, position: Vec2) -> Option<AppInfo> {
        if !self.is_open {
            return None;
        }

        let selected = self
            .items
            .iter()
            .find(|item| item.rect.contains(position))
            .map(|item| item.app_info.clone());
        if selected.is_some() {
            self.close();
        }
        selected
    }

    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        if !self.is_open {
            return commands;
        }

        let palette = self.theme;

        commands.push(RenderCommand::draw_rect_rounded(
            self.rect,
            palette.background,
            16.0,
        ));
        commands.push(RenderCommand::draw_text(
            Vec2::new(self.rect.x + 16.0, self.rect.y + 16.0),
            "Applications",
            Color::WHITE,
            18.0,
        ));

        for item in &self.items {
            commands.extend(item.generate_render_commands(palette));
        }

        commands
    }

    fn add_default_apps(&mut self) {
        let apps = [
            AppInfo::new(
                "Calculator",
                "Quick arithmetic workspace",
                Vec2::new(300.0, 360.0),
                Vec2::new(96.0, 96.0),
                Color::rgb(88, 201, 126),
            ),
            AppInfo::new(
                "Files",
                "Browse project assets",
                Vec2::new(480.0, 340.0),
                Vec2::new(136.0, 120.0),
                Color::rgb(255, 194, 77),
            ),
            AppInfo::new(
                "Terminal",
                "Low level system console",
                Vec2::new(520.0, 300.0),
                Vec2::new(180.0, 160.0),
                Color::rgb(82, 88, 102),
            ),
            AppInfo::new(
                "Settings",
                "Shell and display preferences",
                Vec2::new(420.0, 420.0),
                Vec2::new(220.0, 110.0),
                Color::rgb(180, 136, 255),
            ),
            AppInfo::new(
                "Gallery",
                "Recent screenshots and media",
                Vec2::new(460.0, 320.0),
                Vec2::new(260.0, 144.0),
                Color::rgb(110, 187, 255),
            ),
        ];

        for app in apps {
            self.add_app(app);
        }
    }

    fn relayout_items(&mut self) {
        let item_height = 48.0;
        let start_y = self.rect.y + 44.0;
        for (index, item) in self.items.iter_mut().enumerate() {
            item.rect = Rect::new(
                self.rect.x + 10.0,
                start_y + index as f32 * (item_height + 8.0),
                self.rect.width - 20.0,
                item_height,
            );
        }
    }
}

impl Default for AppLauncher {
    fn default() -> Self {
        Self::new()
    }
}
