//! Window System for FHRE
//!
//! Provides platform-agnostic window abstraction traits and event types.
//! Platform-specific implementations (X11, Wayland, etc.) should be provided
//! by the application layer (e.g., examples/demo).
//!
//! # Architecture
//!
//! - **Window trait**: Abstract interface for platform windows
//! - **WindowInputEvents**: Standardized input event types
//! - **WindowRunner**: Handles the main loop with any WindowResource implementation
//!
//! # Usage (Application Layer)
//!
//! ```rust
//! // Implement Window trait for your platform
//! struct MyX11Window { ... }
//!
//! impl fhre::window::Window for MyX11Window {
//!     fn is_running(&self) -> bool { ... }
//!     fn collect_input_events(&mut self) -> WindowInputEvents { ... }
//!     fn present(&mut self, framebuffer: &[u8]) { ... }
//!     fn dimensions(&self) -> (u32, u32) { ... }
//! }
//!
//! // Use in your app
//! let mut app = App::new(640, 480);
//! let mut window = MyX11Window::new(640, 480, "App");
//! loop {
//!     let events = window.collect_input_events();
//!     if !window.is_running() { break; }
//!     app.update_and_render();
//!     window.present(app.framebuffer());
//! }
//! ```

/// Trait for window implementations
///
/// Platform-specific code should implement this trait.
/// The implementation lives in the application layer (e.g., examples).
pub trait Window {
    /// Check if the window is still running
    fn is_running(&self) -> bool;

    /// Collect input events from the window
    fn collect_input_events(&mut self) -> WindowInputEvents;

    /// Present the framebuffer to the window
    fn present(&mut self, framebuffer: &[u8]);

    /// Get window dimensions
    fn dimensions(&self) -> (u32, u32);
}

/// Input events from window
#[derive(Debug, Clone, Default)]
pub struct WindowInputEvents {
    pub mouse_button_events: alloc::vec::Vec<MouseButtonEvent>,
    pub mouse_motion_events: alloc::vec::Vec<MouseMotionEvent>,
    pub mouse_wheel_events: alloc::vec::Vec<MouseWheelEvent>,
    pub keyboard_events: alloc::vec::Vec<KeyboardEvent>,
}

/// Mouse button event
#[derive(Debug, Clone, Copy)]
pub struct MouseButtonEvent {
    pub button: u32,
    pub pressed: bool,
    pub x: i32,
    pub y: i32,
}

/// Mouse motion event
#[derive(Debug, Clone, Copy)]
pub struct MouseMotionEvent {
    pub x: i32,
    pub y: i32,
    pub delta_x: i32,
    pub delta_y: i32,
}

/// Mouse wheel event
#[derive(Debug, Clone, Copy)]
pub struct MouseWheelEvent {
    pub direction: i32,
    pub x: i32,
    pub y: i32,
}

/// Keyboard event
#[derive(Debug, Clone, Copy)]
pub struct KeyboardEvent {
    pub keycode: u32,
    pub pressed: bool,
}
