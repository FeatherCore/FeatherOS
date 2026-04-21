//! Wallpaper rendering.

use alloc::vec::Vec;
use fhre::{Color, RenderCommand, Vec2};
use fhre::math::Rect;

use crate::theme::{ThemePalette, shell_palette};

/// Wallpaper configuration.
pub struct WallpaperConfig {
    pub background_color: Color,
    pub gradient_start: Color,
    pub gradient_end: Color,
    pub accent_a: Color,
    pub accent_b: Color,
    pub use_gradient: bool,
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        let palette = shell_palette();
        Self {
            background_color: palette.background,
            gradient_start: Color::rgb(22, 34, 60),
            gradient_end: Color::rgb(8, 12, 24),
            accent_a: palette.accent,
            accent_b: Color::rgb(162, 102, 255),
            use_gradient: true,
        }
    }
}

/// Wallpaper state.
pub struct Wallpaper {
    config: WallpaperConfig,
}

impl Wallpaper {
    pub fn new(config: WallpaperConfig) -> Self {
        Self { config }
    }

    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.config.background_color = palette.background;
        self.config.accent_a = palette.accent;
    }

    pub fn generate_render_commands(&self, screen_size: Vec2) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        if self.config.use_gradient {
            let steps = 24;
            let step_height = screen_size.y / steps as f32;
            for index in 0..steps {
                let t = index as f32 / steps as f32;
                let color = Color::rgb(
                    (self.config.gradient_start.r as f32 * (1.0 - t) + self.config.gradient_end.r as f32 * t) as u8,
                    (self.config.gradient_start.g as f32 * (1.0 - t) + self.config.gradient_end.g as f32 * t) as u8,
                    (self.config.gradient_start.b as f32 * (1.0 - t) + self.config.gradient_end.b as f32 * t) as u8,
                );
                commands.push(RenderCommand::draw_rect(
                    Rect::new(0.0, index as f32 * step_height, screen_size.x, step_height + 1.0),
                    color,
                ));
            }
        } else {
            commands.push(RenderCommand::draw_rect(
                Rect::new(0.0, 0.0, screen_size.x, screen_size.y),
                self.config.background_color,
            ));
        }

        commands.push(RenderCommand::draw_rect_rounded(
            Rect::new(screen_size.x - 260.0, 48.0, 190.0, 190.0),
            self.config.accent_a,
            96.0,
        ));
        commands.push(RenderCommand::draw_rect_rounded(
            Rect::new(screen_size.x - 180.0, 92.0, 220.0, 220.0),
            self.config.accent_b,
            110.0,
        ));
        commands.push(RenderCommand::draw_text(
            Vec2::new(28.0, 42.0),
            "Wing",
            Color::WHITE,
            28.0,
        ));
        commands.push(RenderCommand::draw_text(
            Vec2::new(30.0, 62.0),
            "Feather desktop shell",
            shell_palette().text_muted,
            14.0,
        ));
        commands
    }
}

impl Default for Wallpaper {
    fn default() -> Self {
        Self::new(WallpaperConfig::default())
    }
}
