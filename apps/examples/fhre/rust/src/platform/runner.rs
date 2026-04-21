//! Input Bridge and Window Runner
//!
//! Bridges raw window events to ECS resources.
//! This is platform-specific, not part of FHRE core.

use fhre::{App, resources::Resource, Events};
use fhre::window::{Window, WindowInputEvents, MousePosition};
use super::input::{ButtonInput, KeyCode, MouseButton};

extern crate alloc;

/// Trait for adapting platform-specific input to input resources
///
/// Different platforms have different key/button codes.
pub trait InputBridge {
    /// Map platform keycode to KeyCode
    fn map_keycode(&self, platform_keycode: u32) -> Option<KeyCode>;
    
    /// Map platform mouse button to MouseButton
    fn map_mouse_button(&self, platform_button: u32) -> Option<MouseButton>;
}

/// Default input bridge for common mappings
pub struct DefaultInputBridge;

impl InputBridge for DefaultInputBridge {
    fn map_keycode(&self, _keycode: u32) -> Option<KeyCode> {
        None
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

/// Window runner with input bridging
///
/// Provides a standard main loop that:
/// 1. Collects input events from the window
/// 2. Bridges events to ButtonInput resources
/// 3. Runs app.update_and_render()
/// 4. Presents the framebuffer to the window
pub struct WindowRunner<'a, W: Window, B: InputBridge> {
    app: &'a mut App,
    window: &'a mut W,
    input_bridge: &'a B,
    frame_delay_ms: u32,
}

impl<'a, W: Window, B: InputBridge> WindowRunner<'a, W, B> {
    pub fn new(app: &'a mut App, window: &'a mut W, input_bridge: &'a B) -> Self {
        Self {
            app,
            window,
            input_bridge,
            frame_delay_ms: 16,
        }
    }
    
    pub fn with_frame_delay_ms(mut self, ms: u32) -> Self {
        self.frame_delay_ms = ms;
        self
    }
    
    pub fn run(&mut self) {
        extern "C" {
            fn usleep(usec: u32) -> i32;
            fn sched_yield() -> i32;
        }
        
        let mut frame_count: u32 = 0;
        
        loop {
            frame_count += 1;
            
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
            
            if let Some(events) = self.app.main_world.resources_mut().get_mut::<Events>() {
                events.update();
            }
            
            unsafe {
                usleep(self.frame_delay_ms * 1000);
            }
            
            if false {  // TEMP: test
                break;
            }
        }
    }
    
    fn bridge_keyboard_input(&mut self, events: &[fhre::window::KeyboardEvent]) {
        if let Some(key_input) = self.app.main_world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
            key_input.clear();
            for event in events {
                if let Some(kc) = self.input_bridge.map_keycode(event.keycode) {
                    if event.pressed {
                        key_input.press(kc);
                    } else {
                        key_input.release(kc);
                    }
                }
            }
        }
    }
    
    fn bridge_mouse_input(&mut self, events: &[fhre::window::MouseButtonEvent]) {
        if let Some(mouse_input) = self.app.main_world.resources_mut().get_mut::<ButtonInput<MouseButton>>() {
            mouse_input.clear();
            for event in events {
                if let Some(btn) = self.input_bridge.map_mouse_button(event.button) {
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
    
    fn bridge_mouse_position(
        &mut self, 
        button_events: &[fhre::window::MouseButtonEvent], 
        motion_events: &[fhre::window::MouseMotionEvent]
    ) {
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
