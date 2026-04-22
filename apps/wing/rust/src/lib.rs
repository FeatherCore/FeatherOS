//! Wing Desktop Environment
//!
//! A lightweight desktop shell built on top of FHRE.

#![no_std]

extern crate alloc;

pub mod components;
pub mod desktop;
pub mod extract;
pub mod icon;
pub mod input;
pub mod launcher;
pub mod plugin;
pub mod resources;
pub mod systems;
pub mod taskbar;
pub mod theme;
pub mod widgets;
pub mod wallpaper;
pub mod window;

pub use components::{ButtonWidget, DesktopIconComponent, DesktopRoot, DesktopWallpaper, IconGlyph, Label, LauncherEntry, LauncherPanel, ScrollAreaWidget, TaskbarItem, TaskbarRoot, TextFieldWidget, WidgetLayoutNode, WidgetNodeComponent, WindowChrome, WindowContentRoot, WindowFocus, WindowFrame};
pub use desktop::{Desktop, DesktopConfig};
pub use extract::queue_wing_shell;
pub use icon::{DesktopIcon, IconGrid};
pub use input::{ButtonInput, KeyCode, MouseButton, MouseWheel};
pub use launcher::{AppInfo, AppLauncher};
pub use plugin::WingDesktopPlugin;
pub use resources::{DesktopMetrics, DragTransaction, FocusState, LauncherState, LayoutInvalidation, SelectionState, TaskbarState, TextInputState, ThemeState, WindowManagerState, WingDesktopState, WingRuntime};
pub use taskbar::{Taskbar, TaskbarConfig};
pub use theme::{ThemePalette, WingTheme, shell_palette};
pub use widgets::{TextEditCommand, WidgetAction, WidgetEvent, WidgetEventKind, WidgetId, WidgetInteractionState, WidgetKind, WidgetLayout, WidgetNode, WidgetResponse, WidgetTreeNode};
pub use wallpaper::{Wallpaper, WallpaperConfig};
pub use window::{Window, WindowId, WindowManager, WindowState, WindowWidgetEvent};

use alloc::vec::Vec;
use fhre::{RenderCommand, Vec2};
use taskbar::TaskbarAction;

/// Wing application context.
pub struct Wing {
    pub window_manager: WindowManager,
    pub desktop: Desktop,
    pub taskbar: Taskbar,
    pub launcher: AppLauncher,
    pub widget_events: Vec<WindowWidgetEvent>,
    pub theme: WingTheme,
    pub screen_size: Vec2,
}

impl Wing {
    /// Create a new Wing desktop environment.
    pub fn new(width: f32, height: f32) -> Self {
        let screen_size = Vec2::new(width, height);
        let launcher = AppLauncher::new();
        let mut desktop = Desktop::new(screen_size);
        desktop.set_apps(launcher.apps());
        let theme = WingTheme::default();
        desktop.apply_theme(theme.shell);

        Self {
            window_manager: WindowManager::new(screen_size),
            desktop,
            taskbar: Taskbar::new(TaskbarConfig::default()),
            launcher,
            widget_events: Vec::new(),
            theme,
            screen_size,
        }
    }

    /// Initialize the desktop environment.
    pub fn init(&mut self) {
        self.set_theme(self.theme);
        self.desktop.init();
        self.taskbar.init(self.screen_size);
        self.launcher
            .set_layout(self.screen_size, self.taskbar.rect());
        self.desktop.set_reserved_bottom(self.taskbar.height());
        self.desktop.arrange_icons();
    }

    /// Resize the desktop shell.
    pub fn resize(&mut self, width: f32, height: f32) {
        self.screen_size = Vec2::new(width, height);
        self.window_manager.resize_screen(self.screen_size);
        self.desktop.resize(self.screen_size, self.taskbar.height());
        self.taskbar.init(self.screen_size);
        self.launcher
            .set_layout(self.screen_size, self.taskbar.rect());
    }

    /// Register an application in both launcher and desktop.
    pub fn register_app(&mut self, app_info: AppInfo) {
        self.launcher.add_app(app_info);
        self.desktop.set_apps(self.launcher.apps());
        self.desktop.apply_theme(self.theme.shell);
    }

    pub fn set_theme(&mut self, theme: WingTheme) {
        self.theme = theme;
        self.desktop.apply_theme(theme.shell);
        self.taskbar.apply_theme(theme.shell);
        self.launcher.apply_theme(theme.shell);
        self.window_manager.apply_theme(theme.shell);
    }

    /// Update the desktop environment.
    pub fn update(&mut self, delta_time: f32) {
        self.window_manager.update(delta_time);
        self.desktop.update(delta_time);
        self.taskbar.update(delta_time);

        self.widget_events = self.window_manager.drain_widget_events();
        let pending_actions: Vec<(WindowId, WidgetAction)> = self
            .widget_events
            .iter()
            .filter_map(|event| event.action.map(|action| (event.window_id, action)))
            .collect();
        for (window_id, action) in pending_actions {
            self.handle_widget_action(window_id, action);
        }

        let active_window = self.window_manager.focused_window();
        self.taskbar.sync_windows(self.window_manager.windows(), active_window);
        self.taskbar.set_launcher_open(self.launcher.is_open());
    }

    pub fn drain_widget_events(&mut self) -> Vec<WindowWidgetEvent> {
        core::mem::take(&mut self.widget_events)
    }

    /// Launch an application.
    pub fn launch_app(&mut self, app_info: &AppInfo) -> Option<WindowId> {
        let window_id = self.window_manager.create_app_window(app_info);
        self.taskbar.sync_windows(
            self.window_manager.windows(),
            self.window_manager.focused_window(),
        );
        Some(window_id)
    }

    /// Close a window.
    pub fn close_window(&mut self, window_id: WindowId) {
        self.window_manager.close_window(window_id);
        self.taskbar.sync_windows(
            self.window_manager.windows(),
            self.window_manager.focused_window(),
        );
    }

    /// Handle pointer movement.
    pub fn handle_mouse_move(&mut self, position: Vec2) {
        self.taskbar.handle_mouse_move(position);
        self.launcher.handle_mouse_move(position);
        self.window_manager.handle_mouse_move(position);
    }

    /// Handle primary click.
    pub fn handle_click(&mut self, position: Vec2) {
        if self.taskbar.contains(position) {
            if let Some(action) = self.taskbar.handle_click(position) {
                self.handle_taskbar_action(action);
            }
            return;
        }

        if self.launcher.is_open() {
            if self.launcher.contains(position) {
                if let Some(app) = self.launcher.handle_click(position) {
                    let _ = self.launch_app(&app);
                }
                self.taskbar.set_launcher_open(self.launcher.is_open());
                return;
            }

            self.launcher.close();
            self.taskbar.set_launcher_open(false);
        }

        if let Some(app) = self.desktop.handle_click(position) {
            let _ = self.launch_app(&app);
            return;
        }

        if let Some(window_id) = self.window_manager.get_window_at(position) {
            self.window_manager.focus_window(window_id);
            let close_requested = self.window_manager.handle_click(window_id, position);
            if close_requested {
                self.taskbar.sync_windows(
                    self.window_manager.windows(),
                    self.window_manager.focused_window(),
                );
            }
            return;
        }

        self.window_manager.clear_focus();
        self.desktop.clear_selection();
    }

    /// Handle pointer release.
    pub fn handle_mouse_release(&mut self, position: Vec2) {
        self.window_manager.handle_mouse_release(position);
        self.window_manager.stop_dragging();
    }

    pub fn handle_text_edit(&mut self, command: TextEditCommand) -> bool {
        self.window_manager.handle_text_edit(command)
    }

    pub fn handle_scroll(&mut self, position: Vec2, delta: i32) {
        self.window_manager.handle_scroll(position, delta);
    }

    /// Handle dragging.
    pub fn handle_drag(&mut self, delta: Vec2) {
        if let Some(dragging_window) = self.window_manager.dragging_window() {
            self.window_manager.move_window(dragging_window, delta);
        }
    }

    /// Build the frame's render commands.
    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        commands.push(RenderCommand::clear(self.theme.shell.background));
        commands.extend(self.desktop.generate_render_commands());
        commands.extend(self.window_manager.generate_render_commands());
        commands.extend(self.launcher.generate_render_commands());
        commands.extend(self.taskbar.generate_render_commands());
        commands
    }

    fn handle_taskbar_action(&mut self, action: TaskbarAction) {
        match action {
            TaskbarAction::ToggleLauncher => {
                self.launcher.toggle();
                self.taskbar.set_launcher_open(self.launcher.is_open());
            }
            TaskbarAction::FocusWindow(window_id) => {
                self.window_manager.toggle_window_visibility(window_id);
                self.taskbar.sync_windows(
                    self.window_manager.windows(),
                    self.window_manager.focused_window(),
                );
            }
        }
    }

    fn handle_widget_action(&mut self, window_id: WindowId, action: WidgetAction) {
        match action {
            WidgetAction::ToggleLauncher => {
                self.launcher.toggle();
                self.taskbar.set_launcher_open(self.launcher.is_open());
            }
            WidgetAction::CycleTheme => {
                let next = if self.theme == crate::theme::WingTheme::aurora() {
                    crate::theme::WingTheme::dusk()
                } else {
                    crate::theme::WingTheme::aurora()
                };
                self.set_theme(next);
            }
            WidgetAction::ApplySelectedTheme => {
                let selected_theme = self
                    .widget_events
                    .iter()
                    .rev()
                    .find(|event| event.window_id == window_id && event.action == Some(WidgetAction::ApplySelectedTheme))
                    .and_then(|event| self.window_manager.get_widget(window_id, event.event.widget_id))
                    .and_then(|widget| match &widget.kind {
                        WidgetKind::Dropdown { options, selected, .. } => options.get(*selected).copied(),
                        _ => None,
                    });

                match selected_theme {
                    Some("Dusk") => self.set_theme(crate::theme::WingTheme::dusk()),
                    Some("System") => self.set_theme(crate::theme::WingTheme::aurora()),
                    _ => self.set_theme(crate::theme::WingTheme::aurora()),
                }
            }
            WidgetAction::ApplyButtonMatrixSelection => {
                let widget_id = self
                    .widget_events
                    .iter()
                    .rev()
                    .find(|event| event.window_id == window_id && event.action == Some(WidgetAction::ApplyButtonMatrixSelection))
                    .map(|event| event.event.widget_id);
                if let Some(widget_id) = widget_id {
                    self.window_manager.apply_button_matrix_selection(window_id, widget_id);
                }
            }
            WidgetAction::CloseWindow => self.close_window(window_id),
            WidgetAction::MinimizeWindow => self.window_manager.toggle_window_visibility(window_id),
            WidgetAction::MaximizeWindow => self.window_manager.toggle_maximized(window_id),
            WidgetAction::LaunchAppNamed(name) => {
                if let Some(app) = self.launcher.find_app(name) {
                    let _ = self.launch_app(&app);
                }
            }
        }
    }
}

/// Wing error types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WingError {
    WindowNotFound,
    AppNotFound,
    InvalidOperation,
    AllocationFailed,
}

/// Result type for Wing operations.
pub type Result<T> = core::result::Result<T, WingError>;
