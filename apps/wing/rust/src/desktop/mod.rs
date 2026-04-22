//! Desktop workspace.

use alloc::vec::Vec;
use fhre::{Color, RenderCommand, Vec2};

use crate::icon::{DesktopIcon, IconGrid};
use crate::launcher::{AppInfo, LauncherItem};
use crate::theme::{ThemePalette, shell_palette};
use crate::wallpaper::Wallpaper;

/// Desktop configuration.
pub struct DesktopConfig {
    pub background_color: Color,
    pub icon_spacing: f32,
    pub icon_size: f32,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            background_color: shell_palette().background,
            icon_spacing: 22.0,
            icon_size: 68.0,
        }
    }
}

impl DesktopConfig {
    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.background_color = palette.background;
    }
}

/// Desktop environment.
pub struct Desktop {
    screen_size: Vec2,
    config: DesktopConfig,
    wallpaper: Wallpaper,
    icon_grid: IconGrid,
    icons: Vec<DesktopIcon>,
    selected_icon: Option<usize>,
}

impl Desktop {
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            screen_size,
            config: DesktopConfig::default(),
            wallpaper: Wallpaper::default(),
            icon_grid: IconGrid::new(screen_size),
            icons: Vec::new(),
            selected_icon: None,
        }
    }

    pub fn init(&mut self) {
        self.arrange_icons();
    }

    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.config.apply_theme(palette);
        self.wallpaper.apply_theme(palette);
        for icon in &mut self.icons {
            icon.apply_theme(palette);
        }
    }

    pub fn resize(&mut self, screen_size: Vec2, reserved_bottom: f32) {
        self.screen_size = screen_size;
        self.icon_grid.set_screen_size(screen_size);
        self.icon_grid.set_reserved_bottom(reserved_bottom);
        self.arrange_icons();
    }

    pub fn set_reserved_bottom(&mut self, reserved_bottom: f32) {
        self.icon_grid.set_reserved_bottom(reserved_bottom);
    }

    pub fn set_apps(&mut self, apps: &[LauncherItem]) {
        self.icons.clear();
        for app in apps {
            let mut icon = DesktopIcon::new(
                app.app_info.clone(),
                Vec2::ZERO,
                self.config.icon_size,
            );
            icon.apply_theme(shell_palette());
            self.icons.push(icon);
        }
        self.arrange_icons();
    }

    pub fn arrange_icons(&mut self) {
        self.icon_grid
            .set_cell_metrics(self.config.icon_size + 24.0, self.config.icon_spacing);
        let max_rows = self.icon_grid.max_rows().max(1);

        for (index, icon) in self.icons.iter_mut().enumerate() {
            let row = (index as u32) % max_rows;
            let col = (index as u32) / max_rows;
            icon.position = self.icon_grid.get_grid_position(row, col);
            icon.size = self.config.icon_size;
        }
    }

    pub fn update(&mut self, _delta_time: f32) {}

    pub fn get_icon_at(&self, position: Vec2) -> Option<usize> {
        self.icons.iter().position(|icon| icon.contains(position))
    }

    pub fn handle_click(&mut self, position: Vec2) -> Option<AppInfo> {
        self.clear_selection();
        let index = self.get_icon_at(position)?;
        self.icons[index].is_selected = true;
        self.selected_icon = Some(index);
        Some(self.icons[index].app_info.clone())
    }

    pub fn clear_selection(&mut self) {
        self.selected_icon = None;
        for icon in &mut self.icons {
            icon.is_selected = false;
        }
    }

    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = self.wallpaper.generate_render_commands(self.screen_size);
        for icon in &self.icons {
            commands.extend(icon.generate_render_commands());
        }
        commands
    }
}
