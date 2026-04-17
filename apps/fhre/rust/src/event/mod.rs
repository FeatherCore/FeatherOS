//! Event System for FHRE
//!
//! A simplified event system inspired by Bevy's Event/Messaging system.
//! Events are used to communicate between systems and from platform input to game logic.

mod events;
mod event_writer;
mod event_reader;
mod system_param;

pub use events::{Events, Event};
pub use event_writer::EventWriter;
pub use event_reader::EventReader;
pub use system_param::{EventReaderState, EventWriterState};
