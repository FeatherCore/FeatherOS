//! Window System for FHRE
//!
//! Provides platform-specific window creation and event loop management.
//! Currently supports X11 for SIM platform.

#[cfg(feature = "sim")]
mod x11;

#[cfg(feature = "sim")]
pub use x11::{X11Window, WindowInputEvents, MouseButtonEvent, MouseMotionEvent, MouseWheelEvent, KeyboardEvent};

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
