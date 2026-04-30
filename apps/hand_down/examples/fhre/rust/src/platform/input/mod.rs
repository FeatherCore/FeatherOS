//! Platform Input Types
//!
//! Input types specific to this platform (NuttX + X11 simulation).
//! These are NOT part of FHRE core - they belong to the platform layer.

mod button_input;
mod keyboard;
mod mouse;

pub use button_input::ButtonInput;
pub use keyboard::{KeyCode, Key};
pub use mouse::MouseButton;

extern crate alloc;
