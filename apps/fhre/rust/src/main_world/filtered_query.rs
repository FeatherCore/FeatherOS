//! FilteredQuery Implementation
//!
//! Provides Query with integrated filter support, aligned with Bevy's Query system.
//! Usage: `Query<&Transform, With<Velocity>>` or `Query<(&A, &B), (With<C>, Without<D>)>`

use super::component::Component;
use super::entity::Entity;
use super::world::MainWorld;
use super::query_filter::QueryFilter;
use super::system_param::SystemParam;
use alloc::vec::Vec;
use core::marker::PhantomData;

/// A Query that supports filtering entities based on component presence
///
/// This is the main query type that supports Bevy-style filtering:
///
/// ```rust
/// // Query all entities with Transform
/// fn system1(query: Query<&Transform>) {
///     for transform in &query {
///         // ...
///     }
/// }
///
/// // Query entities with Transform that also have Velocity
/// fn system2(query: Query<&Transform, With<Velocity>>) {
///     for transform in &query {
///         // Entity has both Transform and Velocity
///     }
/// }
///
/// // Query entities with Transform but NOT Kinematic
/// fn system3(query: Query<&Transform, Without<Kinematic>>) {
///     for transform in &query {
///         // Entity has Transform but not Kinematic
///     }
/// }
///
/// // Query with multiple filters (AND logic)
/// fn system4(query: Query<&Transform, (With<Velocity>, Without<Static>)>) {
///     for transform in &query {
///         // Entity has Transform and Velocity, but not Static
///     }
/// }
/// ```
pub struct FilteredQuery<'w, 's, T: Component, F: QueryFilter = ()> {
    items: Vec<(Entity, *mut T)>,
    _marker: PhantomData<(&'w (), &'s (), F)>,
}

impl<'w, 's, T: Component, F: QueryFilter> FilteredQuery<'w, 's, T, F> {
    /// Create a new FilteredQuery with the given filter
    pub fn new(world: &'w mut MainWorld) -> Self {
        let items: Vec<(Entity, *mut T)> = unsafe {
            let world_ptr = world as *mut MainWorld;
            let mut items = Vec::new();

            // Get all entities
            let entity_ids: Vec<u64> = (*world_ptr).entities()
                .iter()
                .map(|e| e.id())
                .collect();

            // Filter and collect matching entities
            for id in entity_ids {
                let entity = Entity::new(id);

                // Check if entity has the queried component
                if let Some(component) = (*world_ptr).get_component_mut::<T>(entity) {
                    // Apply the filter
                    if F::matches(entity, &*world_ptr) {
                        items.push((entity, component as *mut T));
                    }
                }
            }

            items
        };

        Self {
            items,
            _marker: PhantomData,
        }
    }

    /// Iterate over all matching components (immutable)
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|(_, ptr)| unsafe {
            &**ptr
        })
    }

    /// Iterate over all matching components (mutable)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().map(|(_, ptr)| unsafe {
            &mut **ptr
        })
    }

    /// Iterate over entities and components (immutable)
    pub fn iter_with_entities(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.items.iter().map(|(entity, ptr)| (*entity, unsafe {
            &**ptr
        }))
    }

    /// Iterate over entities and components (mutable)
    pub fn iter_mut_with_entities(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.items.iter_mut().map(|(entity, ptr)| (*entity, unsafe {
            &mut **ptr
        }))
    }

    /// Get a single component
    pub fn single(&self) -> Option<&T> {
        self.iter().next()
    }

    /// Get a single component mutably
    pub fn single_mut(&mut self) -> Option<&mut T> {
        self.iter_mut().next()
    }

    /// Get component for a specific entity
    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.items.iter()
            .find(|(e, _)| e.id() == entity.id())
            .map(|(_, ptr)| unsafe { &**ptr })
    }

    /// Get component mutably for a specific entity
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.items.iter_mut()
            .find(|(e, _)| e.id() == entity.id())
            .map(|(_, ptr)| unsafe { &mut **ptr })
    }

    /// Check if query has any matching entities
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of matching entities
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl<'w, 's, T: Component, F: QueryFilter> IntoIterator for &'w FilteredQuery<'w, 's, T, F> {
    type Item = &'w T;
    type IntoIter = FilteredQueryIter<'w, 's, T, F>;

    fn into_iter(self) -> Self::IntoIter {
        FilteredQueryIter {
            query: self,
            index: 0,
        }
    }
}

impl<'w, 's, T: Component, F: QueryFilter> IntoIterator for &'w mut FilteredQuery<'w, 's, T, F> {
    type Item = &'w mut T;
    type IntoIter = FilteredQueryIterMut<'w, 's, T, F>;

    fn into_iter(self) -> Self::IntoIter {
        FilteredQueryIterMut {
            query: self,
            index: 0,
        }
    }
}

/// Immutable filtered query iterator
pub struct FilteredQueryIter<'w, 's, T: Component, F: QueryFilter> {
    query: &'w FilteredQuery<'w, 's, T, F>,
    index: usize,
}

impl<'w, 's, T: Component, F: QueryFilter> Iterator for FilteredQueryIter<'w, 's, T, F> {
    type Item = &'w T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.query.items.len() {
            let item = unsafe { &*self.query.items[self.index].1 };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// Mutable filtered query iterator
pub struct FilteredQueryIterMut<'w, 's, T: Component, F: QueryFilter> {
    query: &'w mut FilteredQuery<'w, 's, T, F>,
    index: usize,
}

impl<'w, 's, T: Component, F: QueryFilter> Iterator for FilteredQueryIterMut<'w, 's, T, F> {
    type Item = &'w mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.query.items.len() {
            let ptr = self.query.items[self.index].1;
            let item = unsafe { &mut *ptr };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// State for FilteredQuery SystemParam
pub struct FilteredQueryState<T: Component, F: QueryFilter> {
    _marker: PhantomData<(T, F)>,
}

impl<T: Component, F: QueryFilter> Default for FilteredQueryState<T, F> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Component, F: QueryFilter> SystemParam for FilteredQuery<'_, '_, T, F> {
    type Item<'w, 's> = FilteredQuery<'w, 's, T, F>;
    type State = FilteredQueryState<T, F>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        FilteredQuery::new(world)
    }
}

// Re-export common filter types for convenience
pub use super::query_filter::{With, Without, Or, And};
