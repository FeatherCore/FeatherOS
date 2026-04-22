//! Taskbar and window switcher.

use alloc::vec::Vec;
use fhre::{Color, RenderCommand, Vec2};
use fhre::math::Rect;

use crate::theme::{ThemePalette, shell_palette};
use crate::window::{Window, WindowId, WindowState};

/// Taskbar configuration.
pub struct TaskbarConfig {
    pub height: f32,
    pub background_color: Color,
    pub button_color: Color,
    pub button_hover_color: Color,
    pub position_bottom: bool,
}

impl Default for TaskbarConfig {
    fn default() -> Self {
        let palette = shell_palette();
        Self {
            height: 52.0,
            background_color: palette.background,
            button_color: Color::rgb(44, 56, 80),
            button_hover_color: palette.accent_hover,
            position_bottom: true,
        }
    }
}

impl TaskbarConfig {
    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.background_color = palette.background;
        self.button_hover_color = palette.accent_hover;
    }
}

/// Taskbar interaction result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskbarAction {
    ToggleLauncher,
    FocusWindow(WindowId),
}

/// Taskbar button.
pub struct TaskbarButton {
    pub text: &'static str,
    pub rect: Rect,
    pub window_id: Option<WindowId>,
    pub is_hovered: bool,
    pub is_active: bool,
    pub accent_color: Color,
}

impl TaskbarButton {
    pub fn new(text: &'static str, rect: Rect, window_id: Option<WindowId>) -> Self {
        Self {
            text,
            rect,
            window_id,
            is_hovered: false,
            is_active: false,
            accent_color: Color::rgb(88, 142, 255),
        }
    }

    pub fn generate_render_commands(&self, config: &TaskbarConfig) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        let background = if self.is_active {
            self.accent_color
        } else if self.is_hovered {
            config.button_hover_color
        } else {
            config.button_color
        };

        commands.push(RenderCommand::draw_rect_rounded(self.rect, background, 10.0));
        commands.push(RenderCommand::draw_text(
            Vec2::new(self.rect.x + 12.0, self.rect.y + self.rect.height * 0.55),
            self.text,
            Color::WHITE,
            13.0,
        ));
        commands
    }
}

/// Taskbar.
pub struct Taskbar {
    config: TaskbarConfig,
    rect: Rect,
    start_button: TaskbarButton,
    window_buttons: Vec<TaskbarButton>,
    launcher_open: bool,
}

impl Taskbar {
    pub fn new(config: TaskbarConfig) -> Self {
        Self {
            config,
            rect: Rect::ZERO,
            start_button: TaskbarButton::new("Start", Rect::ZERO, None),
            window_buttons: Vec::new(),
            launcher_open: false,
        }
    }

    pub fn init(&mut self, screen_size: Vec2) {
        let y = if self.config.position_bottom {
            screen_size.y - self.config.height
        } else {
            0.0
        };

        self.rect = Rect::new(0.0, y, screen_size.x, self.config.height);
        self.start_button.rect = Rect::new(10.0, y + 8.0, 88.0, self.config.height - 16.0);
        self.recalculate_button_positions();
    }

    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.config.apply_theme(palette);
        self.start_button.accent_color = palette.accent;
    }

    pub fn update(&mut self, _delta_time: f32) {}

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn height(&self) -> f32 {
        self.config.height
    }

    pub fn set_launcher_open(&mut self, open: bool) {
        self.launcher_open = open;
        self.start_button.is_active = open;
    }

    pub fn sync_windows(&mut self, windows: &[Window], active_window: Option<WindowId>) {
        self.window_buttons.clear();
        for window in windows {
            if window.state == WindowState::Closed {
                continue;
            }

            let mut button = TaskbarButton::new(window.title, Rect::ZERO, Some(window.id));
            button.is_active = active_window == Some(window.id) && window.state != WindowState::Minimized;
            button.accent_color = window.accent_color;
            self.window_buttons.push(button);
        }
        self.recalculate_button_positions();
    }

    pub fn contains(&self, point: Vec2) -> bool {
        self.rect.contains(point)
    }

    pub fn handle_mouse_move(&mut self, position: Vec2) {
        self.start_button.is_hovered = self.start_button.rect.contains(position);
        for button in &mut self.window_buttons {
            button.is_hovered = button.rect.contains(position);
        }
    }

    pub fn handle_click(&mut self, position: Vec2) -> Option<TaskbarAction> {
        if self.start_button.rect.contains(position) {
            return Some(TaskbarAction::ToggleLauncher);
        }

        for button in &self.window_buttons {
            if button.rect.contains(position) {
                if let Some(window_id) = button.window_id {
                    return Some(TaskbarAction::FocusWindow(window_id));
                }
            }
        }

        None
    }

    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        commands.push(RenderCommand::draw_rect(self.rect, self.config.background_color));
        commands.push(RenderCommand::draw_line_thick(
            Vec2::new(self.rect.x, self.rect.y),
            Vec2::new(self.rect.right(), self.rect.y),
            shell_palette().border,
            1.0,
        ));
        commands.extend(self.start_button.generate_render_commands(&self.config));
        for button in &self.window_buttons {
            commands.extend(button.generate_render_commands(&self.config));
        }
        commands
    }

    fn recalculate_button_positions(&mut self) {
        let button_width = 128.0;
        for (index, button) in self.window_buttons.iter_mut().enumerate() {
            button.rect = Rect::new(
                self.start_button.rect.right() + 10.0 + index as f32 * (button_width + 6.0),
                self.rect.y + 8.0,
                button_width,
                self.config.height - 16.0,
            );
        }
    }
}
