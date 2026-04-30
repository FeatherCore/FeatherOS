//! Window System for FHRE
//!
//! Provides platform-agnostic window abstraction traits and event types.
//! Platform-specific implementations should be provided by the application layer.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Application Layer                         │
//! │  - Implements Window trait                                   │
//! │  - Provides InputBridge to convert raw events to resources   │
//! │  - Uses WindowRunner for main loop                           │
//! └─────────────────────────────────────────────────────────────┘
//!                           │
//!                           ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      FHRE Core                               │
//! │  - Window trait (collect raw events)                        │
//! │  - MousePosition resource                                   │
//! │  - Picking system (abstract pointer events)                 │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::resources::Resource;
use alloc::vec::Vec;

/// Mouse position resource
#[derive(Clone, Copy, Debug, Default)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

impl Resource for MousePosition {}

/// Trait for window implementations
///
/// Platform-specific code should implement this trait.
pub trait Window {
    /// Check if the window is still running
    fn is_running(&self) -> bool;

    /// Collect input events from the window
    fn collect_input_events(&mut self) -> WindowInputEvents;

    /// Present the framebuffer to the window
    fn present(&mut self, framebuffer: &[u32]);

    /// Get window dimensions
    fn dimensions(&self) -> (u32, u32);
}

/// Input events from window (raw platform events)
#[derive(Debug, Clone, Default)]
pub struct WindowInputEvents {
    pub mouse_button_events: Vec<MouseButtonEvent>,
    pub mouse_motion_events: Vec<MouseMotionEvent>,
    pub mouse_wheel_events: Vec<MouseWheelEvent>,
    pub keyboard_events: Vec<KeyboardEvent>,
}

/// Mouse button event (raw)
#[derive(Debug, Clone, Copy)]
pub struct MouseButtonEvent {
    pub button: u32,
    pub pressed: bool,
    pub x: i32,
    pub y: i32,
}

/// Mouse motion event (raw)
#[derive(Debug, Clone, Copy)]
pub struct MouseMotionEvent {
    pub x: i32,
    pub y: i32,
    pub delta_x: i32,
    pub delta_y: i32,
}

/// Mouse wheel event (raw)
#[derive(Debug, Clone, Copy)]
pub struct MouseWheelEvent {
    pub direction: i32,
    pub x: i32,
    pub y: i32,
}

/// Keyboard event (raw)
#[derive(Debug, Clone, Copy)]
pub struct KeyboardEvent {
    pub keycode: u32,
    pub pressed: bool,
}

use crate::app::App;
use crate::plugin::Plugin;

/// Trait for input plugins that bridge raw events to ECS resources
///
/// Platform implementations should implement this trait to convert
/// platform-specific input events to FHRE input resources.
pub trait InputPlugin: Plugin {
    /// Bridge raw window events to ECS resources
    ///
    /// Called each frame before `update_and_render()`.
    fn bridge(&self, app: &mut App, events: &WindowInputEvents);
}
