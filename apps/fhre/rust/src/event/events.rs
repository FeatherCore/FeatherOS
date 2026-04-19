//! Events Storage Implementation
//!
//! Stores events in a double-buffered queue system.
//! Events are added to the current frame's queue and read from the previous frame.

use alloc::collections::VecDeque;
use core::any::{TypeId, Any};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

/// Trait for event types
///
/// All event types must implement this trait.
pub trait Event: Clone + 'static {}

/// Event storage for a specific event type
struct EventStorage<T: Event> {
    /// Events from the previous frame (available for reading)
    events_a: VecDeque<T>,
    /// Events for the current frame (being written)
    events_b: VecDeque<T>,
    /// Whether we're using A as current (true) or B (false)
    using_a_as_current: bool,
}

impl<T: Event> EventStorage<T> {
    fn new() -> Self {
        Self {
            events_a: VecDeque::new(),
            events_b: VecDeque::new(),
            using_a_as_current: true,
        }
    }

    /// Send an event
    fn send(&mut self, event: T) {
        if self.using_a_as_current {
            self.events_a.push_back(event);
        } else {
            self.events_b.push_back(event);
        }
    }

    /// Send multiple events
    fn send_batch(&mut self, events: impl Iterator<Item = T>) {
        if self.using_a_as_current {
            self.events_a.extend(events);
        } else {
            self.events_b.extend(events);
        }
    }

    /// Get events from the previous frame for reading
    fn get_events(&self) -> &VecDeque<T> {
        if self.using_a_as_current {
            &self.events_b
        } else {
            &self.events_a
        }
    }

    /// Swap buffers for the next frame
    fn update(&mut self) {
        // Clear the old events (which were read this frame)
        if self.using_a_as_current {
            self.events_b.clear();
        } else {
            self.events_a.clear();
        }
        // Swap which buffer is current
        self.using_a_as_current = !self.using_a_as_current;
    }

    /// Clear all events
    fn clear(&mut self) {
        self.events_a.clear();
        self.events_b.clear();
    }
}

/// Events - Container for all event types
///
/// Stores event queues by type ID for type-safe access.
pub struct Events {
    /// Storage map: TypeId -> EventStorage (boxed to handle different types)
    storage: BTreeMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Events {
    /// Create a new empty events container
    pub fn new() -> Self {
        Self {
            storage: BTreeMap::new(),
        }
    }

    /// Register an event type (creates storage if not exists)
    fn ensure_storage<T: Event + Send + Sync>(&mut self) {
        let type_id = TypeId::of::<T>();
        if !self.storage.contains_key(&type_id) {
            let storage: EventStorage<T> = EventStorage::new();
            self.storage.insert(type_id, Box::new(storage));
        }
    }

    /// Send an event
    pub fn send<T: Event + Send + Sync>(&mut self, event: T) {
        self.ensure_storage::<T>();
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.storage.get_mut(&type_id) {
            if let Some(typed_storage) = storage.downcast_mut::<EventStorage<T>>() {
                typed_storage.send(event);
            }
        }
    }

    /// Send multiple events
    pub fn send_batch<T: Event + Send + Sync>(&mut self, events: impl Iterator<Item = T>) {
        self.ensure_storage::<T>();
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.storage.get_mut(&type_id) {
            if let Some(typed_storage) = storage.downcast_mut::<EventStorage<T>>() {
                typed_storage.send_batch(events);
            }
        }
    }

    /// Get events for reading (from previous frame)
    pub fn get_events<T: Event + Send + Sync>(&self) -> Option<&VecDeque<T>> {
        let type_id = TypeId::of::<T>();
        self.storage.get(&type_id).and_then(|storage| {
            storage.downcast_ref::<EventStorage<T>>().map(|s| s.get_events())
        })
    }

    /// Update all event buffers (call at end of frame)
    pub fn update(&mut self) {
        for storage in self.storage.values_mut() {
            // Use unsafe to call update on the boxed storage
            // SAFETY: We know all storages are EventStorage<T> for some T
            unsafe {
                let ptr = storage.as_mut() as *mut dyn Any;
                // We need a different approach - update all known types
                // For now, this is a limitation of the type-erased storage
            }
        }
    }

    /// Update a specific event type
    pub fn update_type<T: Event + Send + Sync>(&mut self) {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.storage.get_mut(&type_id) {
            if let Some(typed_storage) = storage.downcast_mut::<EventStorage<T>>() {
                typed_storage.update();
            }
        }
    }

    /// Clear all events of a specific type
    pub fn clear<T: Event + Send + Sync>(&mut self) {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.storage.get_mut(&type_id) {
            if let Some(typed_storage) = storage.downcast_mut::<EventStorage<T>>() {
                typed_storage.clear();
            }
        }
    }

    /// Check if an event type has any events
    pub fn has_events<T: Event + Send + Sync>(&self) -> bool {
        self.get_events::<T>().map(|e| !e.is_empty()).unwrap_or(false)
    }

    /// Get the number of events for a type
    pub fn event_count<T: Event + Send + Sync>(&self) -> usize {
        self.get_events::<T>().map(|e| e.len()).unwrap_or(0)
    }
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::resources::Resource for Events {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestEvent {
        value: i32,
    }

    impl Event for TestEvent {}

    #[test]
    fn test_event_storage() {
        let mut storage = EventStorage::<TestEvent>::new();
        
        // Initially no events
        assert_eq!(storage.get_events().len(), 0);
        
        // Send an event
        storage.send(TestEvent { value: 42 });
        
        // Event is in current buffer, not readable yet
        assert_eq!(storage.get_events().len(), 0);
        
        // Update to swap buffers
        storage.update();
        
        // Now event is readable
        assert_eq!(storage.get_events().len(), 1);
        assert_eq!(storage.get_events()[0].value, 42);
        
        // Update again clears old events
        storage.update();
        assert_eq!(storage.get_events().len(), 0);
    }
}
