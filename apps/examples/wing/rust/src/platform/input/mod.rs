//! Platform input types.

mod button_input;
mod keyboard;
mod mouse;

pub use button_input::ButtonInput;
pub use keyboard::KeyCode;
pub use mouse::MouseButton;

extern crate alloc;
