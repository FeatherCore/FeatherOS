//! SystemParam Implementation
//!
//! Provides declarative system parameters like Bevy's Res, ResMut, Query.
//! This enables pure ECS style programming.

use super::world::MainWorld;
use super::entity::Entity;
use super::component::Component;
use crate::resources::Resource;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// SystemParam trait - Types that can be used as system parameters
///
/// This is the core trait that enables declarative ECS.
/// Implement this for types that can be extracted from the World during system execution.
pub trait SystemParam {
    /// The item type that will be passed to the system function
    type Item<'w, 's>;
    /// State that persists between system runs
    type State: Default;

    /// Get the parameter item from the world
    ///
    /// # Safety
    /// This function uses unsafe internally to bypass borrow checker.
    /// The caller must ensure that:
    /// - No aliasing mutable references exist
    /// - The world is valid for the lifetime of the item
    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's>;
}

/// Read-only resource access
///
/// Use this to access resources in a system:
/// ```rust
/// fn my_system(time: Res<Time>) {
///     println!("Time: {:?}", time.delta());
/// }
/// ```
pub struct Res<'w, T: 'static> {
    value: &'w T,
}

impl<'w, T: 'static> Res<'w, T> {
    /// Create a new Res wrapper
    pub fn new(value: &'w T) -> Self {
        Self { value }
    }
}

impl<'w, T: 'static> Deref for Res<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T: 'static> Clone for Res<'_, T> {
    fn clone(&self) -> Self {
        Self { value: self.value }
    }
}

impl<T: 'static> Copy for Res<'_, T> {}

/// State for Res SystemParam
pub struct ResState<T: 'static> {
    _marker: PhantomData<T>,
}

impl<T: 'static> Default for ResState<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: 'static + Resource> SystemParam for Res<'_, T> {
    type Item<'w, 's> = Res<'w, T>;
    type State = ResState<T>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        // Use raw pointer to bypass borrow checker
        let world_ptr = world as *mut MainWorld;
        let resources = (*world_ptr).resources();
        let value = resources
            .get::<T>()
            .expect("Resource not found");
        Res { value }
    }
}

/// Mutable resource access
///
/// Use this to mutate resources in a system:
/// ```rust
/// fn my_system(mut state: ResMut<DemoState>) {
///     state.rotation_y += 1.0;
/// }
/// ```
pub struct ResMut<'w, T: 'static> {
    value: &'w mut T,
}

impl<'w, T: 'static> ResMut<'w, T> {
    /// Create a new ResMut wrapper
    pub fn new(value: &'w mut T) -> Self {
        Self { value }
    }

    /// Get a mutable reference to the inner value
    pub fn into_inner(self) -> &'w mut T {
        self.value
    }
}

impl<'w, T: 'static> Deref for ResMut<'w, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: 'static> DerefMut for ResMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

/// State for ResMut SystemParam
pub struct ResMutState<T: 'static> {
    _marker: PhantomData<T>,
}

impl<T: 'static> Default for ResMutState<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: 'static + Resource> SystemParam for ResMut<'_, T> {
    type Item<'w, 's> = ResMut<'w, T>;
    type State = ResMutState<T>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        // Use raw pointer to bypass borrow checker
        let world_ptr = world as *mut MainWorld;
        let resources = (*world_ptr).resources_mut();
        let value = resources
            .get_mut::<T>()
            .expect("Resource not found");
        ResMut { value }
    }
}

/// Basic Query for components (without filter support)
///
/// Use `FilteredQuery` for Bevy-style filtering:
/// ```rust
/// fn my_system(query: Query<&Transform>) {
///     for transform in &query {
///         println!("Position: {:?}", transform.position);
///     }
/// }
/// ```
///
/// Note: This is being replaced by FilteredQuery which supports filters.
/// Use the type alias `Query<T, F>` from `main_world` module.
pub struct BasicQuery<'w, 's, T: Component> {
    items: Vec<(Entity, *mut T)>,
    _marker: PhantomData<(&'w (), &'s ())>,
}

impl<'w, 's, T: Component> BasicQuery<'w, 's, T> {
    /// Create a new Query
    pub fn new(world: &'w mut MainWorld) -> Self {
        // Collect all matching entities first
        let items: Vec<(Entity, *mut T)> = unsafe {
            let world_ptr = world as *mut MainWorld;
            let mut items = Vec::new();
            
            // Get entity IDs first
            let entity_ids: Vec<u64> = (*world_ptr).entities()
                .iter()
                .map(|e| e.id())
                .collect();
            
            // Then get components
            for id in entity_ids {
                if let Some(component) = (*world_ptr).get_component_mut::<T>(Entity::new(id)) {
                    items.push((Entity::new(id), component as *mut T));
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

impl<'w, 's, T: Component> IntoIterator for &'w BasicQuery<'w, 's, T> {
    type Item = &'w T;
    type IntoIter = QueryIter<'w, 's, T>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIter {
            query: self,
            index: 0,
        }
    }
}

impl<'w, 's, T: Component> IntoIterator for &'w mut BasicQuery<'w, 's, T> {
    type Item = &'w mut T;
    type IntoIter = QueryIterMut<'w, 's, T>;

    fn into_iter(self) -> Self::IntoIter {
        QueryIterMut {
            query: self,
            index: 0,
        }
    }
}

/// Immutable query iterator
pub struct QueryIter<'w, 's, T: Component> {
    query: &'w BasicQuery<'w, 's, T>,
    index: usize,
}

impl<'w, 's, T: Component> Iterator for QueryIter<'w, 's, T> {
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

/// Mutable query iterator
pub struct QueryIterMut<'w, 's, T: Component> {
    query: &'w mut BasicQuery<'w, 's, T>,
    index: usize,
}

impl<'w, 's, T: Component> Iterator for QueryIterMut<'w, 's, T> {
    type Item = &'w mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.query.items.len() {
            // SAFETY: Each item is accessed only once
            let ptr = self.query.items[self.index].1;
            let item = unsafe { &mut *ptr };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

/// State for BasicQuery SystemParam
pub struct BasicQueryState<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> Default for BasicQueryState<T> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: Component> SystemParam for BasicQuery<'_, '_, T> {
    type Item<'w, 's> = BasicQuery<'w, 's, T>;
    type State = BasicQueryState<T>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        BasicQuery::new(world)
    }
}

// Tuple implementations for SystemParam

impl SystemParam for () {
    type Item<'w, 's> = ();
    type State = ();

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        _world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        ()
    }
}

impl<A: SystemParam> SystemParam for (A,) {
    type Item<'w, 's> = (A::Item<'w, 's>,);
    type State = (A::State,);

    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        let (a,) = state;
        (A::get_param(a, world),)
    }
}

impl<A: SystemParam, B: SystemParam> SystemParam for (A, B) {
    type Item<'w, 's> = (A::Item<'w, 's>, B::Item<'w, 's>);
    type State = (A::State, B::State);

    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        // SAFETY: We use raw pointers to bypass borrow checker
        // This is safe because each param accesses different parts of the world
        let world_ptr = world as *mut MainWorld;
        let (a, b) = state;
        let a_item = A::get_param(a, &mut *world_ptr);
        let b_item = B::get_param(b, &mut *world_ptr);
        (a_item, b_item)
    }
}

impl<A: SystemParam, B: SystemParam, C: SystemParam> SystemParam for (A, B, C) {
    type Item<'w, 's> = (A::Item<'w, 's>, B::Item<'w, 's>, C::Item<'w, 's>);
    type State = (A::State, B::State, C::State);

    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        let world_ptr = world as *mut MainWorld;
        let (a, b, c) = state;
        let a_item = A::get_param(a, &mut *world_ptr);
        let b_item = B::get_param(b, &mut *world_ptr);
        let c_item = C::get_param(c, &mut *world_ptr);
        (a_item, b_item, c_item)
    }
}

impl<A: SystemParam, B: SystemParam, C: SystemParam, D: SystemParam> SystemParam for (A, B, C, D) {
    type Item<'w, 's> = (A::Item<'w, 's>, B::Item<'w, 's>, C::Item<'w, 's>, D::Item<'w, 's>);
    type State = (A::State, B::State, C::State, D::State);

    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        let world_ptr = world as *mut MainWorld;
        let (a, b, c, d) = state;
        let a_item = A::get_param(a, &mut *world_ptr);
        let b_item = B::get_param(b, &mut *world_ptr);
        let c_item = C::get_param(c, &mut *world_ptr);
        let d_item = D::get_param(d, &mut *world_ptr);
        (a_item, b_item, c_item, d_item)
    }
}

/// Local state for systems
///
/// `Local<T>` allows systems to have persistent state across multiple runs.
/// The state is initialized once when the system is first created.
///
/// Usage:
/// ```rust
/// fn my_system(mut counter: Local<u32>) {
///     *counter += 1;
///     println!("System ran {} times", *counter);
/// }
/// ```
pub struct Local<'s, T: 'static> {
    value: &'s mut T,
}

impl<'s, T: 'static> Local<'s, T> {
    /// Create a new Local wrapper
    pub fn new(value: &'s mut T) -> Self {
        Self { value }
    }
}

impl<'s, T: 'static> Deref for Local<'s, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'s, T: 'static> DerefMut for Local<'s, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

/// Trait for types that can be created from the World
pub trait FromWorld {
    fn from_world(world: &mut MainWorld) -> Self;
}

impl<T: Default> FromWorld for T {
    fn from_world(_world: &mut MainWorld) -> Self {
        Self::default()
    }
}

/// State for Local SystemParam
pub struct LocalState<T: 'static> {
    value: T,
}

impl<T: 'static + Default> Default for LocalState<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
        }
    }
}

impl<T: 'static + Default + Send> SystemParam for Local<'_, T> {
    type Item<'w, 's> = Local<'s, T>;
    type State = LocalState<T>;

    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        _world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        // Initialize on first access if needed
        Local { value: &mut state.value }
    }
}
