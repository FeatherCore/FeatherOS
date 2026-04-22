//! Desktop icons.

use alloc::vec::Vec;
use fhre::{Color, RenderCommand, Vec2};
use fhre::math::Rect;

use crate::launcher::AppInfo;
use crate::theme::{ThemePalette, shell_palette};

/// Desktop icon.
#[derive(Clone, Debug)]
pub struct DesktopIcon {
    pub app_info: AppInfo,
    pub position: Vec2,
    pub size: f32,
    pub is_selected: bool,
    pub theme: ThemePalette,
}

impl DesktopIcon {
    pub fn new(app_info: AppInfo, position: Vec2, size: f32) -> Self {
        Self {
            app_info,
            position,
            size,
            is_selected: false,
            theme: shell_palette(),
        }
    }

    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.theme = palette;
    }

    pub fn get_rect(&self) -> Rect {
        Rect::from_center_size(self.position, Vec2::new(self.size, self.size))
    }

    pub fn label_rect(&self) -> Rect {
        Rect::new(
            self.position.x - self.size * 0.75,
            self.position.y + self.size * 0.5 + 6.0,
            self.size * 1.5,
            18.0,
        )
    }

    pub fn contains(&self, point: Vec2) -> bool {
        self.get_rect().contains(point)
    }

    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        let rect = self.get_rect();
        let frame = rect.inset(6.0, 6.0);

        if self.is_selected {
            commands.push(RenderCommand::draw_rect_rounded(
                Rect::new(rect.x - 8.0, rect.y - 8.0, rect.width + 16.0, rect.height + 32.0),
                self.theme.accent_pressed,
                14.0,
            ));
        }

        commands.push(RenderCommand::draw_rect_rounded(rect, self.theme.surface_alt, 14.0));
        commands.push(RenderCommand::draw_rect_rounded(frame, self.app_info.icon_color, 10.0));
        commands.push(RenderCommand::draw_line_thick(
            Vec2::new(frame.x + 8.0, frame.y + frame.height * 0.35),
            Vec2::new(frame.right() - 8.0, frame.y + frame.height * 0.35),
            Color::rgb(255, 255, 255),
            3.0,
        ));
        commands.push(RenderCommand::draw_line_thick(
            Vec2::new(frame.x + 8.0, frame.y + frame.height * 0.62),
            Vec2::new(frame.right() - 14.0, frame.y + frame.height * 0.62),
            Color::rgb(255, 255, 255),
            3.0,
        ));
        commands.push(RenderCommand::draw_text(
            Vec2::new(self.label_rect().x + 4.0, self.label_rect().y + 12.0),
            self.app_info.name,
            Color::WHITE,
            12.0,
        ));
        commands
    }
}

/// Grid layout for desktop icons.
#[derive(Clone, Copy, Debug)]
pub struct IconGrid {
    screen_size: Vec2,
    cell_size: f32,
    spacing: f32,
    top_padding: f32,
    left_padding: f32,
    reserved_bottom: f32,
}

impl IconGrid {
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            screen_size,
            cell_size: 92.0,
            spacing: 22.0,
            top_padding: 28.0,
            left_padding: 22.0,
            reserved_bottom: 0.0,
        }
    }

    pub fn set_screen_size(&mut self, screen_size: Vec2) {
        self.screen_size = screen_size;
    }

    pub fn set_cell_metrics(&mut self, cell_size: f32, spacing: f32) {
        self.cell_size = cell_size;
        self.spacing = spacing;
    }

    pub fn set_reserved_bottom(&mut self, reserved_bottom: f32) {
        self.reserved_bottom = reserved_bottom;
    }

    pub fn get_grid_position(&self, row: u32, col: u32) -> Vec2 {
        Vec2::new(
            self.left_padding + self.cell_size * 0.5 + col as f32 * (self.cell_size + self.spacing),
            self.top_padding + self.cell_size * 0.5 + row as f32 * (self.cell_size + self.spacing),
        )
    }

    pub fn max_rows(&self) -> u32 {
        let usable_height = (self.screen_size.y - self.reserved_bottom - self.top_padding).max(self.cell_size);
        (usable_height / (self.cell_size + self.spacing)).max(1.0) as u32
    }

    pub fn max_cols(&self) -> u32 {
        let usable_width = (self.screen_size.x - self.left_padding).max(self.cell_size);
        (usable_width / (self.cell_size + self.spacing)).max(1.0) as u32
    }
}
