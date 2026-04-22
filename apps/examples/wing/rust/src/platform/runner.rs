//! Input plugin and platform event bridge.

use fhre::{App, InputPlugin, MousePosition, Plugin, WindowInputEvents};

use super::input::{ButtonInput, KeyCode, MouseButton, MouseWheel};

pub trait InputBridge {
    fn map_keycode(&self, platform_keycode: u32) -> Option<KeyCode>;
    fn map_mouse_button(&self, platform_button: u32) -> Option<MouseButton>;
}

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
        self.bridge_mouse_wheel(app, &events.mouse_wheel_events);
    }
}

impl<B: InputBridge> PlatformInputPlugin<B> {
    fn bridge_keyboard_input(&self, app: &mut App, events: &[fhre::window::KeyboardEvent]) {
        if let Some(key_input) = app.main_world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
            key_input.clear();
            for event in events {
                if let Some(keycode) = self.bridge.map_keycode(event.keycode) {
                    if event.pressed {
                        key_input.press(keycode);
                    } else {
                        key_input.release(keycode);
                    }
                }
            }
        }
    }

    fn bridge_mouse_input(&self, app: &mut App, events: &[fhre::window::MouseButtonEvent]) {
        if let Some(mouse_input) = app.main_world.resources_mut().get_mut::<ButtonInput<MouseButton>>() {
            mouse_input.clear();
            for event in events {
                if let Some(button) = self.bridge.map_mouse_button(event.button) {
                    if event.pressed {
                        if !mouse_input.pressed(button) {
                            mouse_input.press(button);
                        }
                    } else {
                        mouse_input.release(button);
                    }
                }
            }
        }
    }

    fn bridge_mouse_position(
        &self,
        app: &mut App,
        button_events: &[fhre::window::MouseButtonEvent],
        motion_events: &[fhre::window::MouseMotionEvent],
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

    fn bridge_mouse_wheel(&self, app: &mut App, events: &[fhre::window::MouseWheelEvent]) {
        if let Some(mouse_wheel) = app.main_world.resources_mut().get_mut::<MouseWheel>() {
            mouse_wheel.clear();
            for event in events {
                mouse_wheel.delta += event.direction;
            }
        }
    }
}
