//! Input System for FHRE
//!
//! A complete input system inspired by Bevy's input handling.
//! Provides mouse, keyboard, and touch input support through ECS resources.

mod button_input;
mod mouse;
mod keyboard;
mod input_systems;
mod plugin;

pub use button_input::ButtonInput;
pub use mouse::{
    MouseButton, MouseButtonInput, ButtonState as MouseButtonState,
    MouseMotion, MouseWheel, MouseScrollUnit, MouseInput,
    AccumulatedMouseMotion, AccumulatedMouseScroll,
};
pub use keyboard::{
    KeyCode, Key, KeyboardInput, KeyState as KeyboardKeyState, KeyboardFocusLost,
};
pub use input_systems::{
    mouse_button_input_system,
    accumulate_mouse_motion_system,
    accumulate_mouse_scroll_system,
    keyboard_input_system,
    input_systems_update,
};
pub use plugin::{InputPlugin, InputAppExt};
