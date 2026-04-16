//! Phoenix Desktop Environment
//!
//! A desktop shell for FeatherOS based on FHRE (Feather Hybrid Rendering Engine).
//!
//! Phoenix provides:
//! - Window management (create, move, resize, close windows)
//! - Desktop wallpaper and icon grid
//! - Taskbar with application launcher
//! - System tray
//! - Multi-tasking support
//!
//! Architecture:
//! ```
//! ┌─────────────────────────────────────────┐
//! │         User Applications               │
//! ├─────────────────────────────────────────┤
//! │         Phoenix Desktop Shell           │
//! │  (Window Manager + Desktop + Taskbar)   │
//! ├─────────────────────────────────────────┤
//! │         FHRE (Graphics Library)         │
//! ├─────────────────────────────────────────┤
//! │         NuttX (Operating System)        │
//! └─────────────────────────────────────────┘
//! ```

#![no_std]
#![feature(lang_items)]

extern crate alloc;

pub mod desktop;
pub mod window;
pub mod taskbar;
pub mod launcher;
pub mod wallpaper;
pub mod icon;

pub use desktop::{Desktop, DesktopConfig};
pub use window::{Window, WindowId, WindowState, WindowManager};
pub use taskbar::{Taskbar, TaskbarConfig};
pub use launcher::{AppLauncher, AppInfo};
pub use wallpaper::{Wallpaper, WallpaperConfig};
pub use icon::{DesktopIcon, IconGrid};

use fhre::math::{Color, Vec2};
use fhre::node::{Node, Transform2D};

/// Phoenix application context
///
/// This is the main entry point for the Phoenix desktop environment.
/// It manages all desktop components and coordinates between applications.
pub struct Phoenix {
    /// Window manager
    pub window_manager: WindowManager,
    /// Desktop configuration
    pub desktop: Desktop,
    /// Taskbar
    pub taskbar: Taskbar,
    /// Application launcher
    pub launcher: AppLauncher,
    /// Screen dimensions
    pub screen_size: Vec2,
}

impl Phoenix {
    /// Create a new Phoenix desktop environment
    pub fn new(width: f32, height: f32) -> Self {
        let screen_size = Vec2::new(width, height);
        
        Self {
            window_manager: WindowManager::new(screen_size),
            desktop: Desktop::new(screen_size),
            taskbar: Taskbar::new(TaskbarConfig::default()),
            launcher: AppLauncher::new(),
            screen_size,
        }
    }

    /// Initialize the desktop environment
    pub fn init(&mut self) {
        // Initialize desktop components
        self.desktop.init();
        self.taskbar.init(self.screen_size);
    }

    /// Update the desktop environment (called every frame)
    pub fn update(&mut self, delta_time: f32) {
        // Update window manager
        self.window_manager.update(delta_time);
        
        // Update desktop
        self.desktop.update(delta_time);
        
        // Update taskbar
        self.taskbar.update(delta_time);
    }

    /// Launch an application
    pub fn launch_app(&mut self, app_info: &AppInfo) -> Option<WindowId> {
        // Create a new window for the application
        let window_id = self.window_manager.create_window(
            app_info.name,
            app_info.default_size,
            app_info.initial_position,
        );
        
        // Add to taskbar
        self.taskbar.add_app(app_info.clone(), window_id);
        
        Some(window_id)
    }

    /// Close a window
    pub fn close_window(&mut self, window_id: WindowId) {
        self.window_manager.close_window(window_id);
        self.taskbar.remove_window(window_id);
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, position: Vec2) {
        // Check if clicked on taskbar
        if self.taskbar.contains(position) {
            self.taskbar.handle_click(position);
            return;
        }
        
        // Check if clicked on desktop icons
        if let Some(icon) = self.desktop.get_icon_at(position) {
            icon.on_click();
            return;
        }
        
        // Check if clicked on a window
        if let Some(window_id) = self.window_manager.get_window_at(position) {
            self.window_manager.focus_window(window_id);
            self.window_manager.handle_click(window_id, position);
        }
    }

    /// Handle mouse drag
    pub fn handle_drag(&mut self, position: Vec2, delta: Vec2) {
        if let Some(dragging_window) = self.window_manager.dragging_window {
            self.window_manager.move_window(dragging_window, delta);
        }
    }

    /// Toggle application launcher
    pub fn toggle_launcher(&mut self) {
        self.launcher.toggle();
    }
}

/// Phoenix error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhoenixError {
    /// Window not found
    WindowNotFound,
    /// Application not found
    AppNotFound,
    /// Invalid operation
    InvalidOperation,
    /// Resource allocation failed
    AllocationFailed,
}

/// Result type for Phoenix operations
pub type Result<T> = core::result::Result<T, PhoenixError>;
