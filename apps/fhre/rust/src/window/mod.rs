//! Window System for FHRE
//!
//! Provides platform-agnostic window abstraction traits and event types.
//! Platform-specific implementations (X11, Wayland, framebuffer, etc.) should be provided
//! by the application layer (e.g., examples/demo).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Application Layer                         │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
//! │  │ X11Window   │  │ FBWindow    │  │ OtherPlatformWindow │  │
//! │  │ (sim mode)  │  │ (embedded)  │  │ (user defined)      │  │
//! │  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
//! │         │                │                    │              │
//! │         └────────────────┼────────────────────┘              │
//! │                          │                                   │
//! │                          ▼                                   │
//! │                   impl Window trait                          │
//! └─────────────────────────────────────────────────────────────┘
//!                           │
//!                           ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      FHRE Core                               │
//! │  ┌─────────────────────────────────────────────────────┐    │
//! │  │ WindowRunner: unified main loop with input bridging  │    │
//! │  └─────────────────────────────────────────────────────┘    │
//! │  ┌─────────────────────────────────────────────────────┐    │
//! │  │ App: ECS world + rendering                           │    │
//! │  └─────────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage (Application Layer)
//!
//! ## 1. Implement Window trait for your platform
//!
//! ```rust
//! use fhre::window::{Window, WindowInputEvents, MouseButtonEvent, KeyboardEvent};
//!
//! struct MyPlatformWindow {
//!     // platform-specific fields
//! }
//!
//! impl MyPlatformWindow {
//!     pub fn new(width: u32, height: u32) -> Option<Self> {
//!         // Initialize platform window
//!         Some(Self { ... })
//!     }
//! }
//!
//! impl Window for MyPlatformWindow {
//!     fn is_running(&self) -> bool { true }
//!     
//!     fn collect_input_events(&mut self) -> WindowInputEvents {
//!         // Collect events from platform
//!         WindowInputEvents::default()
//!     }
//!     
//!     fn present(&mut self, framebuffer: &[u8]) {
//!         // Copy framebuffer to display
//!     }
//!     
//!     fn dimensions(&self) -> (u32, u32) { (640, 480) }
//! }
//! ```
//!
//! ## 2. Use WindowRunner for unified main loop
//!
//! ```rust
//! use fhre::{App, DefaultPlugins};
//! use fhre::window::{WindowRunner, WindowInputAdapter};
//!
//! fn main() {
//!     let mut app = App::new(640, 480);
//!     app.add_plugins(DefaultPlugins);
//!     
//!     let mut window = MyPlatformWindow::new(640, 480).unwrap();
//!     let input_adapter = MyInputAdapter; // implements WindowInputAdapter
//!     
//!     WindowRunner::new(&mut app, &mut window, &input_adapter)
//!         .with_frame_delay_ms(16)
//!         .run();
//! }
//! ```

use crate::App;
use crate::input::{ButtonInput, KeyCode, MouseButton};
use crate::resources::Resource;
use alloc::vec::Vec;

/// Default frame delay in milliseconds (~60 FPS)
pub const DEFAULT_FRAME_DELAY_MS: u32 = 16;

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
/// The implementation lives in the application layer (e.g., examples).
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

/// Trait for adapting platform-specific input to FHRE input
///
/// Different platforms have different key/button codes.
/// Implement this trait to map your platform's codes to FHRE's KeyCode/MouseButton.
pub trait WindowInputAdapter {
    /// Map platform keycode to FHRE KeyCode
    fn map_keycode(&self, platform_keycode: u32) -> Option<KeyCode>;
    
    /// Map platform mouse button to FHRE MouseButton
    fn map_mouse_button(&self, platform_button: u32) -> Option<MouseButton>;
}

/// Default input adapter (identity mapping for common keys)
pub struct DefaultInputAdapter;

impl WindowInputAdapter for DefaultInputAdapter {
    fn map_keycode(&self, _keycode: u32) -> Option<KeyCode> {
        None // No mapping by default
    }
    
    fn map_mouse_button(&self, button: u32) -> Option<MouseButton> {
        match button {
            1 => Some(MouseButton::Left),
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::Right),
            _ => None,
        }
    }
}

/// Unified window runner with input bridging
///
/// Provides a standard main loop that:
/// 1. Collects input events from the window
/// 2. Bridges events to FHRE's ButtonInput resources
/// 3. Runs app.update_and_render()
/// 4. Presents the framebuffer to the window
pub struct WindowRunner<'a, W: Window, A: WindowInputAdapter> {
    app: &'a mut App,
    window: &'a mut W,
    input_adapter: &'a A,
    frame_delay_ms: u32,
}

impl<'a, W: Window, A: WindowInputAdapter> WindowRunner<'a, W, A> {
    /// Create a new window runner
    pub fn new(app: &'a mut App, window: &'a mut W, input_adapter: &'a A) -> Self {
        Self {
            app,
            window,
            input_adapter,
            frame_delay_ms: DEFAULT_FRAME_DELAY_MS,
        }
    }
    
    /// Set frame delay in milliseconds (default: 16ms ≈ 60fps)
    pub fn with_frame_delay_ms(mut self, ms: u32) -> Self {
        self.frame_delay_ms = ms;
        self
    }
    
    /// Run the main loop
    /// 
    /// This will:
    /// 1. Collect input events from window
    /// 2. Bridge events to ButtonInput resources
    /// 3. Call app.update_and_render()
    /// 4. Present framebuffer to window
    /// 5. Sleep for frame_delay_ms
    /// 6. Repeat until window.is_running() returns false
    pub fn run(&mut self) {
        extern "C" {
            fn usleep(usec: u32) -> i32;
            fn sched_yield() -> i32;
        }
        
        loop {
            unsafe { sched_yield(); }
            
            let events = self.window.collect_input_events();
            
            if !self.window.is_running() {
                break;
            }
            
            self.bridge_keyboard_input(&events.keyboard_events);
            self.bridge_mouse_input(&events.mouse_button_events);
            self.bridge_mouse_position(&events.mouse_button_events, &events.mouse_motion_events);
            
            self.app.update_and_render();
            
            self.window.present(self.app.framebuffer());
            
            unsafe {
                usleep(self.frame_delay_ms * 1000);
            }
        }
    }
    
    /// Bridge keyboard events to ButtonInput<KeyCode> resource
    fn bridge_keyboard_input(&mut self, events: &[KeyboardEvent]) {
        if let Some(key_input) = self.app.main_world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
            key_input.clear();
            for event in events {
                if let Some(kc) = self.input_adapter.map_keycode(event.keycode) {
                    if event.pressed {
                        key_input.press(kc);
                    } else {
                        key_input.release(kc);
                    }
                }
            }
        }
    }
    
    fn bridge_mouse_input(&mut self, events: &[MouseButtonEvent]) {
        if let Some(mouse_input) = self.app.main_world.resources_mut().get_mut::<ButtonInput<MouseButton>>() {
            mouse_input.clear();
            for event in events {
                if let Some(btn) = self.input_adapter.map_mouse_button(event.button) {
                    if event.pressed {
                        if !mouse_input.pressed(btn) {
                            mouse_input.press(btn);
                        }
                    } else {
                        mouse_input.release(btn);
                    }
                }
            }
        }
    }
    
    fn bridge_mouse_position(&mut self, button_events: &[MouseButtonEvent], motion_events: &[MouseMotionEvent]) {
        if let Some(mouse_pos) = self.app.main_world.resources_mut().get_mut::<MousePosition>() {
            for event in motion_events {
                mouse_pos.x = event.x;
                mouse_pos.y = event.y;
            }
            for event in button_events {
                mouse_pos.x = event.x;
                mouse_pos.y = event.y;
            }
        }
    }
}

/// Input events from window
#[derive(Debug, Clone, Default)]
pub struct WindowInputEvents {
    pub mouse_button_events: Vec<MouseButtonEvent>,
    pub mouse_motion_events: Vec<MouseMotionEvent>,
    pub mouse_wheel_events: Vec<MouseWheelEvent>,
    pub keyboard_events: Vec<KeyboardEvent>,
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
