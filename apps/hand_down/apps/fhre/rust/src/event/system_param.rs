//! Event SystemParam Implementation
//!
//! Provides SystemParam implementations for EventReader and EventWriter.

use super::{Events, Event, EventReader, EventWriter};
use crate::main_world::MainWorld;
use crate::SystemParam;
use core::marker::PhantomData;

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

impl<T: Event + Clone + Send + Sync> SystemParam for EventReader<'_, T> {
    type Item<'w, 's> = EventReader<'s, T>;
    type State = EventReaderState<T>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        let world_ptr = world as *mut MainWorld;
        let events = (*world_ptr).resources().get::<Events>().expect("Events resource not found");
        EventReader::new(events)
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

impl<T: Event + Send + Sync> SystemParam for EventWriter<'_, T> {
    type Item<'w, 's> = EventWriter<'s, T>;
    type State = EventWriterState<T>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        let world_ptr = world as *mut MainWorld;
        let events = (*world_ptr).resources_mut().get_mut::<Events>().expect("Events resource not found");
        EventWriter::new(events)
    }
}
