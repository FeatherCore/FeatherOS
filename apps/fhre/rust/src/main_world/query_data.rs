//! QueryData Implementation
//!
//! Provides support for querying multiple components in a single Query.
//! This enables Bevy-style queries like `Query<(&A, &mut B)>`.

use super::component::Component;
use super::entity::Entity;
use super::world::MainWorld;
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Trait for types that can be fetched from a Query
///
/// This is the core trait that enables multi-component queries.
/// Implement this for component references, tuples, and other queryable types.
///
/// # Examples
///
/// ```rust
/// // Single component
/// Query<&Transform>
///
/// // Multiple components (tuple)
/// Query<(&Transform, &Velocity)>
///
/// // Mutable access
/// Query<&mut Transform>
///
/// // Mixed access
/// Query<(&Transform, &mut Velocity)>
/// ```
pub trait QueryData {
    /// The item type returned by the query
    type Item<'w>;

    /// Fetch the data for a specific entity
    ///
    /// # Safety
    /// This function uses unsafe internally. The caller must ensure:
    /// - The entity has all required components
    /// - No aliasing mutable references exist
    unsafe fn fetch<'w>(world: &'w MainWorld, entity: Entity) -> Option<Self::Item<'w>>;

    /// Check if an entity has all required components
    fn matches(world: &MainWorld, entity: Entity) -> bool;
}

// Implement QueryData for immutable component references
impl<T: Component> QueryData for &T {
    type Item<'w> = &'w T;

    unsafe fn fetch<'w>(world: &'w MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        world.get_component::<T>(entity)
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

// Implement QueryData for mutable component references
impl<T: Component> QueryData for &mut T {
    type Item<'w> = &'w mut T;

    unsafe fn fetch<'w>(world: &'w MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        // SAFETY: We use raw pointer to bypass borrow checker
        // The caller must ensure no aliasing mutable references exist
        let world_ptr = world as *const MainWorld as *mut MainWorld;
        (*world_ptr).get_component_mut::<T>(entity)
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

// Implement QueryData for tuples (up to 4 elements)
// This enables Query<(&A, &B)>, Query<(&mut A, &B, &C)>, etc.

impl<A: QueryData> QueryData for (A,) {
    type Item<'w> = (A::Item<'w>,);

    unsafe fn fetch<'w>(world: &'w MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        let a = A::fetch(world, entity)?;
        Some((a,))
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        A::matches(world, entity)
    }
}

impl<A: QueryData, B: QueryData> QueryData for (A, B) {
    type Item<'w> = (A::Item<'w>, B::Item<'w>);

    unsafe fn fetch<'w>(world: &'w MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        // SAFETY: We fetch each component separately
        // The caller must ensure no aliasing mutable references exist
        let world_ptr = world as *const MainWorld as *mut MainWorld;
        let a = A::fetch(&*world_ptr, entity)?;
        let b = B::fetch(&*world_ptr, entity)?;
        Some((a, b))
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        A::matches(world, entity) && B::matches(world, entity)
    }
}

impl<A: QueryData, B: QueryData, C: QueryData> QueryData for (A, B, C) {
    type Item<'w> = (A::Item<'w>, B::Item<'w>, C::Item<'w>);

    unsafe fn fetch<'w>(world: &'w MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        let world_ptr = world as *const MainWorld as *mut MainWorld;
        let a = A::fetch(&*world_ptr, entity)?;
        let b = B::fetch(&*world_ptr, entity)?;
        let c = C::fetch(&*world_ptr, entity)?;
        Some((a, b, c))
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        A::matches(world, entity) && B::matches(world, entity) && C::matches(world, entity)
    }
}

impl<A: QueryData, B: QueryData, C: QueryData, D: QueryData> QueryData for (A, B, C, D) {
    type Item<'w> = (A::Item<'w>, B::Item<'w>, C::Item<'w>, D::Item<'w>);

    unsafe fn fetch<'w>(world: &'w MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        let world_ptr = world as *const MainWorld as *mut MainWorld;
        let a = A::fetch(&*world_ptr, entity)?;
        let b = B::fetch(&*world_ptr, entity)?;
        let c = C::fetch(&*world_ptr, entity)?;
        let d = D::fetch(&*world_ptr, entity)?;
        Some((a, b, c, d))
    }

    fn matches(world: &MainWorld, entity: Entity) -> bool {
        A::matches(world, entity)
            && B::matches(world, entity)
            && C::matches(world, entity)
            && D::matches(world, entity)
    }
}

// Implement QueryData for Entity (just returns the entity itself)
impl QueryData for Entity {
    type Item<'w> = Entity;

    unsafe fn fetch<'w>(_world: &'w MainWorld, entity: Entity) -> Option<Self::Item<'w>> {
        Some(entity)
    }

    fn matches(_world: &MainWorld, _entity: Entity) -> bool {
        true
    }
}

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
