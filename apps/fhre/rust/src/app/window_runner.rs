//! Window Runner Implementation
//!
//! Provides a runner that works with a window stored as an ECS Resource.
//! The application creates its own window and inserts it as a Resource.
//!
//! # Example
//!
//! ```rust
//! // Create window externally
//! let window = X11Window::new(640, 480, "FHRE Demo");
//!
//! // Insert as Resource
//! let mut app = App::new(640, 480);
//! app.insert_resource(window);
//! app.set_runner(WindowRunner::<X11Window>::new());
//! app.run();
//! ```

use super::{App, AppRunner, AppExit};
use alloc::boxed::Box;

/// Trait for windows that can be used with WindowRunner
///
/// Implement this trait for your window type and insert it as a Resource.
pub trait WindowResource: crate::resources::Resource {
    /// Check if the window is still open
    fn is_open(&self) -> bool;

    /// Get framebuffer dimensions
    fn dimensions(&self) -> (u32, u32);

    /// Present framebuffer to window
    fn present(&mut self, framebuffer: &[u8]);

    /// Collect input events
    fn collect_events(&mut self) -> WindowEvents;
}

/// Collection of window events
#[derive(Debug, Default, Clone)]
pub struct WindowEvents {
    pub mouse_motions: alloc::vec::Vec<MouseMotionEvent>,
    pub mouse_buttons: alloc::vec::Vec<MouseButtonEvent>,
    pub keyboard: alloc::vec::Vec<KeyboardEvent>,
    pub close_requested: bool,
}

/// Mouse motion event
#[derive(Debug, Clone, Copy)]
pub struct MouseMotionEvent {
    pub x: f32,
    pub y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
}

/// Mouse button event
#[derive(Debug, Clone, Copy)]
pub struct MouseButtonEvent {
    pub button: u32,
    pub pressed: bool,
    pub x: f32,
    pub y: f32,
}

/// Keyboard event
#[derive(Debug, Clone, Copy)]
pub struct KeyboardEvent {
    pub keycode: u32,
    pub pressed: bool,
}

/// Runner that uses a window from ECS Resources
///
/// The window type `W` must implement `WindowResource` and be inserted as a Resource
/// before calling `app.run()`.
///
/// # Type Parameters
///
/// * `W` - The window type, must implement `WindowResource`
pub struct WindowRunner<W: WindowResource> {
    target_fps: u32,
    _marker: core::marker::PhantomData<W>,
}

impl<W: WindowResource> WindowRunner<W> {
    /// Create a new window runner
    pub fn new() -> Self {
        Self {
            target_fps: 60,
            _marker: core::marker::PhantomData,
        }
    }

    /// Set target FPS
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }
}

impl<W: WindowResource> Default for WindowRunner<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: WindowResource + 'static> AppRunner for WindowRunner<W> {
    fn run(self: Box<Self>, mut app: App) -> AppExit {
        let target_delta = 1.0 / self.target_fps as f32;

        loop {
            // Check if app should exit
            if !app.is_running() {
                return AppExit::Success;
            }

            // Get window from ECS resources
            let window_open = {
                if let Some(window) = app.main_world.resources().get::<W>() {
                    window.is_open()
                } else {
                    // Window resource not found, exit
                    return AppExit::Success;
                }
            };

            if !window_open {
                return AppExit::Success;
            }

            // Collect events from window
            let events = {
                if let Some(window) = app.main_world.resources_mut().get_mut::<W>() {
                    window.collect_events()
                } else {
                    WindowEvents::default()
                }
            };

            // TODO: Send events to FHRE's input system
            // This requires converting WindowEvents to FHRE's internal input format

            // Update the app (runs systems + renders)
            app.update();

            // Present framebuffer to window
            // TODO: Get framebuffer from app's render_world and present it
            // For now, this is a placeholder

            // Frame timing
            let elapsed = unsafe {
                extern "C" { fn clock() -> i64; }
                clock()
            };
            let elapsed_secs = elapsed as f32 / 1_000_000.0;
            if elapsed_secs < target_delta {
                let sleep_us = ((target_delta - elapsed_secs) * 1_000_000.0) as u32;
                unsafe {
                    extern "C" { fn usleep(usec: u32) -> i32; }
                    usleep(sleep_us);
                }
            }
        }
    }
}

/// Create a window runner for a specific window type
pub fn window_runner<W: WindowResource + 'static>() -> Box<dyn AppRunner> {
    Box::new(WindowRunner::<W>::new())
}
