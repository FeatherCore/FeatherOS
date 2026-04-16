//! Taskbar Module
//!
//! Provides a taskbar with application launcher and window switcher.

use alloc::string::String;
use alloc::vec::Vec;
use fhre::math::{Color, Vec2, Rect};
use fhre::render_world::RenderCommand;
use crate::window::WindowId;
use crate::launcher::AppInfo;

/// Taskbar configuration
pub struct TaskbarConfig {
    /// Taskbar height
    pub height: f32,
    /// Taskbar color
    pub background_color: Color,
    /// Button color
    pub button_color: Color,
    /// Button hover color
    pub button_hover_color: Color,
    /// Position (true = bottom, false = top)
    pub position_bottom: bool,
}

impl Default for TaskbarConfig {
    fn default() -> Self {
        Self {
            height: 48.0,
            background_color: Color::rgb(40, 40, 40),
            button_color: Color::rgb(60, 60, 60),
            button_hover_color: Color::rgb(80, 80, 80),
            position_bottom: true,
        }
    }
}

/// Taskbar button
pub struct TaskbarButton {
    /// Button text
    pub text: String,
    /// Button rectangle
    pub rect: Rect,
    /// Associated window ID
    pub window_id: Option<WindowId>,
    /// Is hovered
    pub is_hovered: bool,
    /// Is active
    pub is_active: bool,
}

impl TaskbarButton {
    /// Create a new taskbar button
    pub fn new(text: String, rect: Rect, window_id: Option<WindowId>) -> Self {
        Self {
            text,
            rect,
            window_id,
            is_hovered: false,
            is_active: false,
        }
    }

    /// Generate render commands
    pub fn generate_render_commands(&self, config: &TaskbarConfig) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        let color = if self.is_active {
            Color::rgb(100, 150, 200)
        } else if self.is_hovered {
            config.button_hover_color
        } else {
            config.button_color
        };

        commands.push(RenderCommand::DrawRect {
            rect: self.rect,
            color,
        });

        commands
    }
}

/// Taskbar
pub struct Taskbar {
    /// Configuration
    config: TaskbarConfig,
    /// Taskbar rectangle
    rect: Rect,
    /// Start button
    start_button: TaskbarButton,
    /// Window buttons
    window_buttons: Vec<TaskbarButton>,
    /// Is launcher open
    launcher_open: bool,
}

impl Taskbar {
    /// Create a new taskbar
    pub fn new(config: TaskbarConfig) -> Self {
        Self {
            config,
            rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            start_button: TaskbarButton::new(
                String::from("Start"),
                Rect::new(0.0, 0.0, 80.0, 48.0),
                None,
            ),
            window_buttons: Vec::new(),
            launcher_open: false,
        }
    }

    /// Initialize the taskbar
    pub fn init(&mut self, screen_size: Vec2) {
        let y = if self.config.position_bottom {
            screen_size.y - self.config.height
        } else {
            0.0
        };

        self.rect = Rect::new(0.0, y, screen_size.x, self.config.height);
        self.start_button.rect = Rect::new(8.0, y + 4.0, 80.0, self.config.height - 8.0);
    }

    /// Update the taskbar
    pub fn update(&mut self, _delta_time: f32) {
        // Update animations, etc.
    }

    /// Add an application to the taskbar
    pub fn add_app(&mut self, app_info: AppInfo, window_id: WindowId) {
        let button_width = 120.0;
        let button_x = self.start_button.rect.x + self.start_button.rect.width + 8.0 +
            self.window_buttons.len() as f32 * (button_width + 4.0);

        let button = TaskbarButton::new(
            String::from(app_info.name),
            Rect::new(
                button_x,
                self.rect.y + 4.0,
                button_width,
                self.config.height - 8.0,
            ),
            Some(window_id),
        );

        self.window_buttons.push(button);
    }

    /// Remove a window from the taskbar
    pub fn remove_window(&mut self, window_id: WindowId) {
        if let Some(index) = self.window_buttons.iter().position(|b| b.window_id == Some(window_id)) {
            self.window_buttons.remove(index);
            // Recalculate positions
            self.recalculate_button_positions();
        }
    }

    /// Recalculate button positions
    fn recalculate_button_positions(&mut self) {
        let button_width = 120.0;
        for (i, button) in self.window_buttons.iter_mut().enumerate() {
            button.rect.x = self.start_button.rect.x + self.start_button.rect.width + 8.0 +
                i as f32 * (button_width + 4.0);
        }
    }

    /// Check if point is inside taskbar
    pub fn contains(&self, point: Vec2) -> bool {
        self.rect.contains(point)
    }

    /// Handle click
    pub fn handle_click(&mut self, position: Vec2) {
        // Check start button
        if self.start_button.rect.contains(position) {
            self.launcher_open = !self.launcher_open;
            return;
        }

        // Check window buttons
        for button in &mut self.window_buttons {
            if button.rect.contains(position) {
                button.is_active = !button.is_active;
                // TODO: Focus/minimize window
                break;
            }
        }
    }

    /// Generate render commands for the taskbar
    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        // Draw taskbar background
        commands.push(RenderCommand::DrawRect {
            rect: self.rect,
            color: self.config.background_color,
        });

        // Draw start button
        commands.extend(self.start_button.generate_render_commands(&self.config));

        // Draw window buttons
        for button in &self.window_buttons {
            commands.extend(button.generate_render_commands(&self.config));
        }

        commands
    }
}
