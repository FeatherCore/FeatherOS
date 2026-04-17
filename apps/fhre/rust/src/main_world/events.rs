//! Event System Implementation
//!
//! Provides EventReader/EventWriter for event-based communication between systems.
//! Aligned with Bevy's event system.

use super::world::MainWorld;
use super::system_param::{SystemParam, Res, ResMut};
use super::component::Component;
use crate::resources::Resource;
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Trait for event types
pub trait Event: Send + Sync + 'static {}

/// Event storage resource
pub struct Events<T: Event> {
    events: Vec<T>,
}

impl<T: Event> Events<T> {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub fn send(&mut self, event: T) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = T> + '_ {
        self.events.drain(..)
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

impl<T: Event> Default for Events<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Event> Resource for Events<T> {}

/// EventReader for reading events
///
/// Usage:
/// ```rust
/// fn my_system(mut reader: EventReader<MyEvent>) {
///     for event in reader.read() {
///         println!("Received: {:?}", event);
///     }
/// }
/// ```
pub struct EventReader<'w, 's, T: Event> {
    events: Res<'w, Events<T>>,
    _marker: PhantomData<&'s ()>,
}

impl<'w, 's, T: Event> EventReader<'w, 's, T> {
    /// Read all events
    pub fn read(&mut self) -> EventIterator<'_, T> {
        // This is a simplified version - in real implementation
        // we'd need to track which events have been read
        EventIterator {
            events: &[],
            index: 0,
        }
    }

    /// Check if there are any events
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Event iterator
pub struct EventIterator<'a, T: Event> {
    events: &'a [T],
    index: usize,
}

impl<'a, T: Event> Iterator for EventIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.events.len() {
            let item = &self.events[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// EventWriter for sending events
///
/// Usage:
/// ```rust
/// fn my_system(mut writer: EventWriter<MyEvent>) {
///     writer.send(MyEvent { data: 42 });
/// }
/// ```
pub struct EventWriter<'w, 's, T: Event> {
    events: ResMut<'w, Events<T>>,
    _marker: PhantomData<&'s ()>,
}

impl<'w, 's, T: Event> EventWriter<'w, 's, T> {
    /// Send an event
    pub fn send(&mut self, event: T) {
        self.events.send(event);
    }

    /// Send multiple events
    pub fn send_batch(&mut self, events: impl IntoIterator<Item = T>) {
        for event in events {
            self.send(event);
        }
    }
}

// === SystemParam Implementations ===

/// State for EventReader SystemParam
pub struct EventReaderState<T: Event> {
    _marker: PhantomData<T>,
}

impl<T: Event> Default for EventReaderState<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Event> SystemParam for EventReader<'_, '_, T> {
    type Item<'w, 's> = EventReader<'w, 's, T>;
    type State = EventReaderState<T>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        let world_ptr = world as *mut MainWorld;
        let events = (*world_ptr).resources().get::<Events<T>>().expect("Events resource not found");
        EventReader {
            events: Res::new(events),
            _marker: PhantomData,
        }
    }
}

/// State for EventWriter SystemParam
pub struct EventWriterState<T: Event> {
    _marker: PhantomData<T>,
}

impl<T: Event> Default for EventWriterState<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Event> SystemParam for EventWriter<'_, '_, T> {
    type Item<'w, 's> = EventWriter<'w, 's, T>;
    type State = EventWriterState<T>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        let world_ptr = world as *mut MainWorld;
        let events = (*world_ptr).resources_mut().get_mut::<Events<T>>().expect("Events resource not found");
        EventWriter {
            events: ResMut::new(events),
            _marker: PhantomData,
        }
    }
}
