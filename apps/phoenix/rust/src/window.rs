//! Window Management Module
//!
//! Provides window creation, management, and rendering for applications.

use alloc::vec::Vec;
use alloc::string::String;
use fhre::math::{Color, Vec2, Rect};
use fhre::render_world::RenderCommand;

/// Unique window identifier
pub type WindowId = u32;

/// Window state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// Window is normal (active and visible)
    Normal,
    /// Window is minimized
    Minimized,
    /// Window is maximized
    Maximized,
    /// Window is closed
    Closed,
}

/// Window structure
pub struct Window {
    /// Window ID
    pub id: WindowId,
    /// Window title
    pub title: String,
    /// Window position (top-left corner)
    pub position: Vec2,
    /// Window size
    pub size: Vec2,
    /// Window state
    pub state: WindowState,
    /// Is window focused
    pub is_focused: bool,
    /// Window content color (for demo)
    pub content_color: Color,
    /// Window border color
    pub border_color: Color,
    /// Window title bar height
    pub title_bar_height: f32,
    /// Window border width
    pub border_width: f32,
}

impl Window {
    /// Create a new window
    pub fn new(id: WindowId, title: &str, size: Vec2, position: Vec2) -> Self {
        Self {
            id,
            title: String::from(title),
            position,
            size,
            state: WindowState::Normal,
            is_focused: false,
            content_color: Color::rgb(240, 240, 240),
            border_color: Color::rgb(100, 100, 100),
            title_bar_height: 24.0,
            border_width: 2.0,
        }
    }

    /// Get the full window rectangle (including border and title bar)
    pub fn get_rect(&self) -> Rect {
        Rect::new(
            self.position.x - self.border_width,
            self.position.y - self.title_bar_height - self.border_width,
            self.size.x + self.border_width * 2.0,
            self.size.y + self.title_bar_height + self.border_width * 2.0,
        )
    }

    /// Get the content area rectangle
    pub fn get_content_rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.size.x,
            self.size.y,
        )
    }

    /// Get the title bar rectangle
    pub fn get_title_bar_rect(&self) -> Rect {
        Rect::new(
            self.position.x - self.border_width,
            self.position.y - self.title_bar_height - self.border_width,
            self.size.x + self.border_width * 2.0,
            self.title_bar_height,
        )
    }

    /// Check if point is inside window
    pub fn contains(&self, point: Vec2) -> bool {
        self.get_rect().contains(point)
    }

    /// Check if point is on title bar
    pub fn is_on_title_bar(&self, point: Vec2) -> bool {
        self.get_title_bar_rect().contains(point)
    }

    /// Generate render commands for this window
    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        if self.state == WindowState::Minimized {
            return commands;
        }

        // Draw window border/background
        let rect = self.get_rect();
        commands.push(RenderCommand::DrawRect {
            rect,
            color: if self.is_focused {
                Color::rgb(60, 120, 180) // Focused border
            } else {
                Color::rgb(120, 120, 120) // Unfocused border
            },
        });

        // Draw title bar
        let title_bar = self.get_title_bar_rect();
        commands.push(RenderCommand::DrawRect {
            rect: title_bar,
            color: if self.is_focused {
                Color::rgb(60, 120, 180)
            } else {
                Color::rgb(160, 160, 160)
            },
        });

        // Draw content area
        let content = self.get_content_rect();
        commands.push(RenderCommand::DrawRect {
            rect: content,
            color: self.content_color,
        });

        // Draw close button (simple red square)
        let close_btn_size = 16.0;
        let close_btn = RenderCommand::DrawRect {
            rect: Rect::new(
                rect.x + rect.width - close_btn_size - 4.0,
                rect.y + 4.0,
                close_btn_size,
                close_btn_size,
            ),
            color: Color::rgb(220, 80, 80),
        };
        commands.push(close_btn);

        commands
    }
}

/// Window manager
pub struct WindowManager {
    /// All windows
    windows: Vec<Window>,
    /// Next window ID
    next_id: WindowId,
    /// Currently focused window
    focused_window: Option<WindowId>,
    /// Window being dragged
    pub dragging_window: Option<WindowId>,
    /// Screen size
    screen_size: Vec2,
    /// Window stack order (front to back)
    window_order: Vec<WindowId>,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
            focused_window: None,
            dragging_window: None,
            screen_size,
            window_order: Vec::new(),
        }
    }

    /// Create a new window
    pub fn create_window(&mut self, title: &str, size: Vec2, position: Vec2) -> WindowId {
        let id = self.next_id;
        self.next_id += 1;

        let window = Window::new(id, title, size, position);
        self.windows.push(window);
        
        // Add to front of window order
        self.window_order.insert(0, id);
        
        // Focus the new window
        self.focus_window(id);

        id
    }

    /// Close a window
    pub fn close_window(&mut self, window_id: WindowId) {
        if let Some(index) = self.windows.iter().position(|w| w.id == window_id) {
            self.windows.remove(index);
        }
        
        if let Some(index) = self.window_order.iter().position(|&id| id == window_id) {
            self.window_order.remove(index);
        }

        if self.focused_window == Some(window_id) {
            self.focused_window = self.window_order.first().copied();
        }
    }

    /// Get a window by ID
    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == window_id)
    }

    /// Get a mutable window by ID
    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == window_id)
    }

    /// Focus a window
    pub fn focus_window(&mut self, window_id: WindowId) {
        // Unfocus current window
        if let Some(current_id) = self.focused_window {
            if let Some(window) = self.get_window_mut(current_id) {
                window.is_focused = false;
            }
        }

        // Focus new window
        if let Some(window) = self.get_window_mut(window_id) {
            window.is_focused = true;
            self.focused_window = Some(window_id);
            
            // Move to front of window order
            if let Some(index) = self.window_order.iter().position(|&id| id == window_id) {
                self.window_order.remove(index);
                self.window_order.insert(0, window_id);
            }
        }
    }

    /// Move a window
    pub fn move_window(&mut self, window_id: WindowId, delta: Vec2) {
        if let Some(window) = self.get_window_mut(window_id) {
            window.position.x += delta.x;
            window.position.y += delta.y;
            
            // Clamp to screen bounds
            window.position.x = window.position.x.max(0.0).min(self.screen_size.x - window.size.x);
            window.position.y = window.position.y.max(window.title_bar_height).min(self.screen_size.y);
        }
    }

    /// Get window at position (from front to back)
    pub fn get_window_at(&self, point: Vec2) -> Option<WindowId> {
        for &window_id in &self.window_order {
            if let Some(window) = self.get_window(window_id) {
                if window.contains(point) && window.state != WindowState::Minimized {
                    return Some(window_id);
                }
            }
        }
        None
    }

    /// Handle click on window
    pub fn handle_click(&mut self, window_id: WindowId, position: Vec2) {
        if let Some(window) = self.get_window(window_id) {
            if window.is_on_title_bar(position) {
                // Check if clicked on close button
                let rect = window.get_rect();
                let close_btn_x = rect.x + rect.width - 20.0;
                let close_btn_y = rect.y + 4.0;
                
                if position.x >= close_btn_x && position.x <= close_btn_x + 16.0 &&
                   position.y >= close_btn_y && position.y <= close_btn_y + 16.0 {
                    self.close_window(window_id);
                    return;
                }
                
                // Start dragging
                self.dragging_window = Some(window_id);
            }
        }
    }

    /// Stop dragging
    pub fn stop_dragging(&mut self) {
        self.dragging_window = None;
    }

    /// Update all windows
    pub fn update(&mut self, _delta_time: f32) {
        // Update window animations, etc.
    }

    /// Generate render commands for all windows (back to front)
    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        
        // Render windows from back to front
        for &window_id in self.window_order.iter().rev() {
            if let Some(window) = self.get_window(window_id) {
                commands.extend(window.generate_render_commands());
            }
        }
        
        commands
    }
}
