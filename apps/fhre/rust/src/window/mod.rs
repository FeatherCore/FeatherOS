//! Window System for FHRE
//!
//! Provides platform-specific window creation and event loop management.
//! Currently supports X11 for SIM platform.
//!
//! # Architecture
//!
//! - **WindowPlugin**: Creates and manages the X11 window as a Plugin
//! - **WindowRunner**: Handles the main loop with window event processing
//! - **X11Window**: Platform-specific implementation of WindowResource
//!
//! # Usage
//!
//! ```rust
//! use fhre::window::WindowPlugin;
//!
//! App::new(640, 480)
//!     .add_plugin(WindowPlugin::new(640, 480, "My App"))
//!     .run();
//! ```

#[cfg(feature = "sim")]
mod x11;

#[cfg(feature = "sim")]
pub use x11::X11Window;

use crate::app::{App, AppRunner};
use crate::app::window_runner::WindowRunner;
use alloc::string::String;

/// Window Plugin for FHRE
///
/// This plugin creates an X11 window and sets up a WindowRunner to handle
/// the main application loop. It automatically:
/// - Creates an X11 window with the specified dimensions
/// - Inserts the window as an ECS Resource
/// - Sets up a WindowRunner for the main loop
///
/// # Example
///
/// ```rust
/// App::new(640, 480)
///     .add_plugin(WindowPlugin::new(640, 480, "FHRE Demo"))
///     .run();
/// ```
pub struct WindowPlugin {
    width: u32,
    height: u32,
    title: String,
    target_fps: u32,
}

impl WindowPlugin {
    /// Create a new WindowPlugin
    ///
    /// # Arguments
    /// * `width` - Window width in pixels
    /// * `height` - Window height in pixels
    /// * `title` - Window title
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        Self {
            width,
            height,
            title: String::from(title),
            target_fps: 60,
        }
    }

    /// Set the target FPS (default: 60)
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }
}

#[cfg(feature = "sim")]
use crate::plugin::Plugin;

#[cfg(feature = "sim")]
impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        unsafe {
            extern "C" { fn printf(format: *const u8, ...) -> i32; }
            printf(b"[WINDOW_PLUGIN] Creating X11 window %dx%d: %s\n\0".as_ptr(), 
                   self.width, self.height, self.title.as_ptr());
        }

        // Create X11 window
        #[cfg(feature = "sim")]
        {
            let window = match X11Window::new(self.width, self.height, &self.title) {
                Some(w) => w,
                None => {
                    unsafe {
                        extern "C" { fn printf(format: *const u8, ...) -> i32; }
                        printf(b"[WINDOW_PLUGIN] ERROR: Failed to create X11 window\n\0".as_ptr());
                    }
                    return;
                }
            };

            // Insert window as resource
            app.insert_resource(window);

            // Set up WindowRunner
            let runner = WindowRunner::<X11Window>::new()
                .with_target_fps(self.target_fps);
            app.set_runner(runner);
        }

        unsafe {
            extern "C" { fn printf(format: *const u8, ...) -> i32; }
            printf(b"[WINDOW_PLUGIN] Window plugin initialized\n\0".as_ptr());
        }
    }
}

/// Trait for window implementations
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

/// Input events from window (re-export for convenience)
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
