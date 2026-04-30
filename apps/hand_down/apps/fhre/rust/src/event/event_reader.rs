//! Event Reader
//!
//! Used to read events from systems.

use super::events::{Events, Event};
use alloc::vec::Vec;

/// EventReader - Reads events of a specific type
///
/// Use this in systems to receive events.
pub struct EventReader<'a, T: Event> {
    events: &'a Events,
    _phantom: core::marker::PhantomData<T>,
}

impl<'a, T: Event + Clone + Send + Sync> EventReader<'a, T> {
    /// Create a new event reader
    pub fn new(events: &'a Events) -> Self {
        Self {
            events,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Get all events as a vector
    pub fn read(&self) -> Vec<T> {
        self.events.get_events::<T>()
            .map(|queue| queue.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Check if there are any events
    pub fn is_empty(&self) -> bool {
        self.events.get_events::<T>().map(|q| q.is_empty()).unwrap_or(true)
    }

    /// Get the number of events
    pub fn len(&self) -> usize {
        self.events.get_events::<T>().map(|q| q.len()).unwrap_or(0)
    }
}
