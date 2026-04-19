//! Query Filter Implementation
//!
//! Provides filters for Query: With<T>, Without<T>, and Or<...>
//! Aligned with Bevy's query filter system.

use super::component::Component;
use super::entity::Entity;
use super::world::MainWorld;
use core::marker::PhantomData;

/// Trait for query filters
///
/// Filters determine which entities match a query based on component presence.
pub trait QueryFilter {
    /// Check if an entity matches this filter
    fn matches(entity: Entity, world: &MainWorld) -> bool;
}

/// Filter that requires a component to be present
///
/// Usage:
/// ```rust
/// // Query entities that have both Transform and Velocity
/// fn my_system(query: Query<(Transform, Velocity), (With<Transform>, With<Velocity>)>) {
///     for (transform, velocity) in query.iter() {
///         // ...
///     }
/// }
/// ```
pub struct With<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for With<T> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

/// Filter that requires a component to be absent
///
/// Usage:
/// ```rust
/// // Query entities that have Transform but NOT Velocity
/// fn my_system(query: Query<Transform, Without<Velocity>>) {
///     for transform in query.iter() {
///         // These entities don't have Velocity
///     }
/// }
/// ```
pub struct Without<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for Without<T> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        world.get_component::<T>(entity).is_none()
    }
}

/// Filter that matches if ANY of the inner filters match (OR logic)
///
/// Usage:
/// ```rust
/// // Query entities that have either Transform OR Velocity
/// fn my_system(query: Query<Entity, Or<(With<Transform>, With<Velocity>)>>) {
///     for entity in query.iter() {
///         // Entity has at least one of the components
///     }
/// }
/// ```
pub struct Or<T> {
    _marker: PhantomData<T>,
}

impl<F1: QueryFilter, F2: QueryFilter> QueryFilter for Or<(F1, F2)> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        F1::matches(entity, world) || F2::matches(entity, world)
    }
}

impl<F1: QueryFilter, F2: QueryFilter, F3: QueryFilter> QueryFilter for Or<(F1, F2, F3)> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        F1::matches(entity, world) || F2::matches(entity, world) || F3::matches(entity, world)
    }
}

/// Filter that matches if ALL of the inner filters match (AND logic)
///
/// This is the default behavior when multiple filters are specified.
pub struct And<T> {
    _marker: PhantomData<T>,
}

impl<F1: QueryFilter, F2: QueryFilter> QueryFilter for And<(F1, F2)> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        F1::matches(entity, world) && F2::matches(entity, world)
    }
}

impl<F1: QueryFilter, F2: QueryFilter, F3: QueryFilter> QueryFilter for And<(F1, F2, F3)> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        F1::matches(entity, world) && F2::matches(entity, world) && F3::matches(entity, world)
    }
}

/// Tuple implementations for combining filters
impl<F1: QueryFilter, F2: QueryFilter> QueryFilter for (F1, F2) {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        F1::matches(entity, world) && F2::matches(entity, world)
    }
}

impl<F1: QueryFilter, F2: QueryFilter, F3: QueryFilter> QueryFilter for (F1, F2, F3) {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        F1::matches(entity, world) && F2::matches(entity, world) && F3::matches(entity, world)
    }
}

impl<F1: QueryFilter, F2: QueryFilter, F3: QueryFilter, F4: QueryFilter> QueryFilter for (F1, F2, F3, F4) {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        F1::matches(entity, world) 
            && F2::matches(entity, world) 
            && F3::matches(entity, world)
            && F4::matches(entity, world)
    }
}

/// Unit type implements QueryFilter (always matches)
impl QueryFilter for () {
    fn matches(_entity: Entity, _world: &MainWorld) -> bool {
        true
    }
}

/// Filter that matches entities where component T was added since last run
///
/// Usage:
/// ```rust
/// // Query entities where Transform was just added
/// fn my_system(query: Query<&Transform, Added<Transform>>) {
///     for transform in &query {
///         // This Transform was added this frame
///     }
/// }
/// ```
pub struct Added<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for Added<T> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        let last_run_tick = world.change_detection().last_run_tick();
        world.change_detection().is_added::<T>(entity, last_run_tick)
    }
}

/// Filter that matches entities where component T was changed since last run
///
/// Usage:
/// ```rust
/// // Query entities where Transform was modified
/// fn my_system(query: Query<&Transform, Changed<Transform>>) {
///     for transform in &query {
///         // This Transform was modified this frame
///     }
/// }
/// ```
pub struct Changed<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for Changed<T> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        let last_run_tick = world.change_detection().last_run_tick();
        world.change_detection().is_changed::<T>(entity, last_run_tick)
    }
}
