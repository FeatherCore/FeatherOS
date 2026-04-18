//! Event System for FHRE
//!
//! A simplified event system inspired by Bevy's Event/Messaging system.
//! Events are used to communicate between systems and from platform input to game logic.
//!
//! # Architecture
//!
//! ```text
//! Platform (X11) → Events → System (EventReader) → Game Logic
//! Game Logic → EventWriter → Events → Other Systems
//! ```
//!
//! # Usage
//!
//! ```rust
//! // In a system
//! fn my_system(
//!     keyboard: EventReader<KeyboardInput>,
//!     mut mouse_motion: EventWriter<MouseMotion>,
//! ) {
//!     // Read events
//!     for event in keyboard.read() {
//!         // Handle keyboard input
//!     }
//!     
//!     // Write events
//!     mouse_motion.send(MouseMotion { x: 100.0, y: 200.0 });
//! }
//! ```

mod events;
mod event_writer;
mod event_reader;
mod system_param;

pub use events::{Events, Event};
pub use event_writer::EventWriter;
pub use event_reader::EventReader;
pub use system_param::{EventReaderState, EventWriterState};

use crate::plugin::Plugin;
use crate::app::App;
use crate::resources::Resource;

/// Standard input events for FHRE
pub mod input_events {
    use super::Event;
    use alloc::string::String;

    /// Keyboard input event
    #[derive(Clone, Debug)]
    pub struct KeyboardInput {
        /// Key code
        pub keycode: u32,
        /// Whether the key was pressed or released
        pub state: ButtonState,
        /// Character if this is a text input
        pub character: Option<char>,
    }

    impl Event for KeyboardInput {}

    /// Mouse button state
    #[derive(Clone, Debug, Copy, PartialEq, Eq)]
    pub enum ButtonState {
        Pressed,
        Released,
        Repeated,
    }

    /// Mouse button event
    #[derive(Clone, Debug)]
    pub struct MouseButtonInput {
        /// Which mouse button
        pub button: MouseButton,
        /// State of the button
        pub state: ButtonState,
        /// Position when the event occurred
        pub x: f32,
        pub y: f32,
    }

    impl Event for MouseButtonInput {}

    /// Mouse button identifiers
    #[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
    #[repr(u16)]
    pub enum MouseButton {
        Left = 1,
        Middle = 2,
        Right = 3,
        Back = 4,
        Forward = 5,
        Other(u16),
    }

    /// Mouse motion event
    #[derive(Clone, Debug, Copy)]
    pub struct MouseMotion {
        /// Current X position
        pub x: f32,
        /// Current Y position
        pub y: f32,
        /// Delta from last position
        pub delta_x: f32,
        /// Delta from last position
        pub delta_y: f32,
    }

    impl Event for MouseMotion {}

    /// Mouse wheel/scroll event
    #[derive(Clone, Debug, Copy)]
    pub struct MouseWheel {
        /// Scroll direction (-1 for down, +1 for up)
        pub delta: f32,
        /// X position during scroll
        pub x: f32,
        /// Y position during scroll
        pub y: f32,
    }

    impl Event for MouseWheel {}

    /// Window resize event
    #[derive(Clone, Debug, Copy)]
    pub struct WindowResized {
        /// New width in pixels
        pub width: u32,
        /// New height in pixels
        pub height: u32,
    }

    impl Event for WindowResized {}

    /// Window close requested event
    #[derive(Clone, Debug, Copy)]
    pub struct WindowCloseRequested;

    impl Event for WindowCloseRequested {}
}

// SAFETY: Events is only accessed on main thread in SIM platform
unsafe impl Send for Events {}
unsafe impl Sync for Events {}

/// Event Plugin - Registers the Events resource as a Plugin
///
/// This plugin initializes the global Events resource that all systems can use.
///
/// # Example
///
/// ```rust
/// App::new(640, 480)
///     .add_plugin(EventPlugin)  // Register Events resource
///     .run();
/// ```
pub struct EventPlugin;

impl EventPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EventPlugin {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: EventPlugin is stateless and safe to share
unsafe impl Send for EventPlugin {}
unsafe impl Sync for EventPlugin {}

impl Plugin for EventPlugin {
    fn build(&self, app: &mut App) {
        // Insert Events as a global resource
        app.insert_resource(Events::new());
        
        unsafe {
            extern "C" { fn printf(format: *const u8, ...) -> i32; }
            printf(b"[EVENT_PLUGIN] Initialized - Events resource registered\n\0".as_ptr());
        }
    }
}
