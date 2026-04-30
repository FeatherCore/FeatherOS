//! QueryData Implementation
//!
//! Provides support for querying multiple components in a single Query.
//! This enables Bevy-style queries like `Query<(&A, &mut B)>`.

use super::component::Component;
use super::entity::Entity;
use super::world::MainWorld;
use super::change_detection::{Mut, Ref};
use alloc::vec::Vec;
use core::marker::PhantomData;

pub trait QueryData {
    type Item<'w>;

    unsafe fn fetch<'w>(world: &'w mut MainWorld, entity: Entity) -> Option<Self::Item<'w>>;

    fn matches(world: &MainWorld, entity: Entity) -> bool;
}

impl<T: Component> QueryData for &T {
    type Item<'w> = &'w T;

    unsafe fn fetch<'w>(world: &'w mut MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        world.get_component::<T>(entity)
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

impl<T: Component> QueryData for &mut T {
    type Item<'w> = &'w mut T;

    unsafe fn fetch<'w>(world: &'w mut MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        world.get_component_mut::<T>(entity)
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

impl QueryData for Entity {
    type Item<'w> = Entity;

    unsafe fn fetch<'w>(_world: &'w mut MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        Some(entity)
    }

    fn matches(_world: &MainWorld, _entity: Entity) -> bool {
        true
    }
}

impl<T: Component> QueryData for Mut<'static, T> {
    type Item<'w> = Mut<'w, T>;

    unsafe fn fetch<'w>(world: &'w mut MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        let world_ptr = world as *mut MainWorld;
        let component = (*world_ptr).get_component_mut::<T>(entity)?;
        let change_detection = (*world_ptr).change_detection_mut() as *mut _;
        Some(Mut::new(component, entity, &mut *change_detection))
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

impl<T: Component> QueryData for Ref<'static, T> {
    type Item<'w> = Ref<'w, T>;

    unsafe fn fetch<'w>(world: &'w mut MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        let world_ptr = world as *mut MainWorld;
        let component = (*world_ptr).get_component::<T>(entity)?;
        let change_detection = (*world_ptr).change_detection() as *const _;
        Some(Ref::new(component, entity, &*change_detection))
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

macro_rules! impl_tuple_query_data {
    () => {
        impl QueryData for () {
            type Item<'w> = ();

            unsafe fn fetch<'w>(_world: &'w mut MainWorld, _entity: Entity) -> Option<Self::Item<'w>> {
                Some(())
            }

            fn matches(_world: &MainWorld, _entity: Entity) -> bool {
                true
            }
        }
    };
    ($($T:ident),*) => {
        impl<$($T: QueryData),*> QueryData for ($($T,)*) {
            type Item<'w> = ($($T::Item<'w>,)*);

            #[allow(unused_variables)]
            unsafe fn fetch<'w>(world: &'w mut MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
                let world_ptr = world as *mut MainWorld;
                Some(($($T::fetch(&mut *world_ptr, entity)?,)*))
            }

            fn matches(world: &MainWorld, entity: Entity) -> bool {
                true $(&& $T::matches(world, entity))*
            }
        }
    };
}

crate::all_tuples!(impl_tuple_query_data, 0, 15, T);

/// A Query that can fetch multiple components using QueryData
///
/// This is the advanced Query type that supports multi-component queries:
///
/// ```rust
/// // Single component
/// fn system1(query: Query<&Transform>) {
///     for transform in &query {
///         // ...
///     }
/// }
///
/// // Multiple components
/// fn system2(query: Query<(&Transform, &Velocity)>) {
///     for (transform, velocity) in &query {
///         // ...
///     }
/// }
///
/// // Mutable access
/// fn system3(query: Query<(&mut Transform, &Velocity)>) {
///     for (mut transform, velocity) in &query {
///         transform.position += velocity.value * delta_time;
///     }
/// }
///
/// // With Entity
/// fn system4(query: Query<(Entity, &Transform, &mut Velocity)>) {
///     for (entity, transform, mut velocity) in &query {
///         // Access entity ID and components
///     }
/// }
/// ```
pub struct MultiQuery<'w, 's, D: QueryData, F: super::query_filter::QueryFilter = ()> {
    items: Vec<(Entity, D::Item<'w>)>,
    _marker: PhantomData<(&'w (), &'s (), F)>,
}

impl<'w, 's, D: QueryData, F: super::query_filter::QueryFilter> MultiQuery<'w, 's, D, F> {
    /// Create a new MultiQuery
    pub fn new(world: &'w mut MainWorld) -> Self {
        let items: Vec<(Entity, D::Item<'w>)> = unsafe {
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

                // Check if entity matches the QueryData requirements
                if D::matches(&*world_ptr, entity) {
                    // Apply the filter
                    if F::matches(entity, &*world_ptr) {
                        if let Some(item) = D::fetch(&mut *world_ptr, entity) {
                            items.push((entity, item));
                        }
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

    /// Get the items slice for iteration
    pub fn items(&self) -> &[(Entity, D::Item<'w>)] {
        &self.items
    }

    /// Get the items slice mutably for iteration
    pub fn items_mut(&mut self) -> &mut [(Entity, D::Item<'w>)] {
        &mut self.items
    }

    /// Get a single item
    pub fn single(&self) -> Option<&D::Item<'w>> {
        self.items.first().map(|(_, item)| item)
    }

    /// Get a single item mutably
    pub fn single_mut(&mut self) -> Option<&mut D::Item<'w>> {
        self.items.first_mut().map(|(_, item)| item)
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

// SystemParam implementation for MultiQuery
use super::system_param::SystemParam;

/// State for MultiQuery SystemParam
pub struct MultiQueryState<D: QueryData, F: super::query_filter::QueryFilter> {
    _marker: PhantomData<(D, F)>,
}

impl<D: QueryData, F: super::query_filter::QueryFilter> Default for MultiQueryState<D, F> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<D: QueryData, F: super::query_filter::QueryFilter> SystemParam for MultiQuery<'_, '_, D, F> {
    type Item<'w, 's> = MultiQuery<'w, 's, D, F>;
    type State = MultiQueryState<D, F>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        MultiQuery::new(world)
    }
}
