//! Input Plugin and Window Runner
//!
//! Bridges raw window events to ECS resources.
//! This is platform-specific, not part of FHRE core.

use fhre::{App, resources::Resource, Events, InputPlugin, WindowInputEvents, MousePosition, Plugin};
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

/// Input plugin that bridges raw events to ECS resources
pub struct PlatformInputPlugin<B: InputBridge> {
    bridge: B,
}

impl<B: InputBridge> PlatformInputPlugin<B> {
    pub fn new(bridge: B) -> Self {
        Self { bridge }
    }
}

impl<B: InputBridge + 'static> Plugin for PlatformInputPlugin<B> {
    fn build(&self, _app: &mut App) {}
}

impl<B: InputBridge + 'static> InputPlugin for PlatformInputPlugin<B> {
    fn bridge(&self, app: &mut App, events: &WindowInputEvents) {
        self.bridge_keyboard_input(app, &events.keyboard_events);
        self.bridge_mouse_input(app, &events.mouse_button_events);
        self.bridge_mouse_position(app, &events.mouse_button_events, &events.mouse_motion_events);
    }
}

impl<B: InputBridge> PlatformInputPlugin<B> {
    fn bridge_keyboard_input(&self, app: &mut App, events: &[fhre::window::KeyboardEvent]) {
        if let Some(key_input) = app.main_world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
            key_input.clear();
            for event in events {
                if let Some(kc) = self.bridge.map_keycode(event.keycode) {
                    if event.pressed {
                        key_input.press(kc);
                    } else {
                        key_input.release(kc);
                    }
                }
            }
        }
    }
    
    fn bridge_mouse_input(&self, app: &mut App, events: &[fhre::window::MouseButtonEvent]) {
        if let Some(mouse_input) = app.main_world.resources_mut().get_mut::<ButtonInput<MouseButton>>() {
            mouse_input.clear();
            for event in events {
                if let Some(btn) = self.bridge.map_mouse_button(event.button) {
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
        &self, 
        app: &mut App, 
        button_events: &[fhre::window::MouseButtonEvent], 
        motion_events: &[fhre::window::MouseMotionEvent]
    ) {
        if let Some(mouse_pos) = app.main_world.resources_mut().get_mut::<MousePosition>() {
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

/// Create a platform input plugin with default mappings
pub fn default_input_plugin() -> PlatformInputPlugin<DefaultInputBridge> {
    PlatformInputPlugin::new(DefaultInputBridge)
}
