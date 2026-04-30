//! Event Writer
//!
//! Used to send events from systems.

use super::events::{Events, Event};

/// EventWriter - Sends events of a specific type
///
/// Use this in systems to emit events.
pub struct EventWriter<'a, T: Event> {
    events: &'a mut Events,
    _phantom: core::marker::PhantomData<T>,
}

impl<'a, T: Event + Send + Sync> EventWriter<'a, T> {
    /// Create a new event writer
    pub fn new(events: &'a mut Events) -> Self {
        Self {
            events,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Send a single event
    pub fn send(&mut self, event: T) {
        self.events.send(event);
    }

    /// Send multiple events at once
    pub fn send_batch(&mut self, events: impl Iterator<Item = T>) {
        self.events.send_batch::<T>(events);
    }
}
