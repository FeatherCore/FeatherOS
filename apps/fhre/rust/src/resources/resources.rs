//! Resource Management Implementation
//!
//! Resources are global data that can be accessed by systems.
//! They are stored in a type-safe container.

use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use core::any::{TypeId, Any};

/// Resource trait - Marker trait for resource types
///
/// All resource types must implement this trait.
pub trait Resource: 'static + Send + Sync {}

/// Resources - Container for global data
///
/// Stores resources by type ID for type-safe access.
pub struct Resources {
    /// Storage map: TypeId -> Resource
    storage: BTreeMap<TypeId, Box<dyn Any>>,
}

impl Resources {
    /// Create a new empty resources container
    pub fn new() -> Self {
        Self {
            storage: BTreeMap::new(),
        }
    }

    /// Insert a resource
    pub fn insert<T: Resource>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.storage.insert(type_id, Box::new(resource));
    }

    /// Get a reference to a resource
    pub fn get<T: Resource>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.storage
            .get(&type_id)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable reference to a resource
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.storage
            .get_mut(&type_id)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove a resource
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.storage
            .remove(&type_id)
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Check if a resource exists
    pub fn contains<T: Resource>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.storage.contains_key(&type_id)
    }

    /// Clear all resources
    pub fn clear(&mut self) {
        self.storage.clear();
    }

    /// Get or insert a resource
    pub fn get_or_insert<T: Resource + Default>(&mut self) -> &mut T {
        let type_id = TypeId::of::<T>();
        if !self.storage.contains_key(&type_id) {
            self.insert(T::default());
        }
        self.get_mut::<T>().unwrap()
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

/// Res<T> - Resource reference wrapper
///
/// Used as a system parameter to access resources.
pub struct Res<'a, T: Resource> {
    resource: &'a T,
}

impl<'a, T: Resource> Res<'a, T> {
    /// Create a new resource reference
    pub fn new(resource: &'a T) -> Self {
        Self { resource }
    }

    /// Get the resource reference
    pub fn get(&self) -> &T {
        self.resource
    }
}

impl<'a, T: Resource> core::ops::Deref for Res<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

/// ResMut<T> - Mutable resource reference wrapper
///
/// Used as a system parameter to mutably access resources.
pub struct ResMut<'a, T: Resource> {
    resource: &'a mut T,
}

impl<'a, T: Resource> ResMut<'a, T> {
    /// Create a new mutable resource reference
    pub fn new(resource: &'a mut T) -> Self {
        Self { resource }
    }

    /// Get the mutable resource reference
    pub fn get_mut(&mut self) -> &mut T {
        self.resource
    }
}

impl<'a, T: Resource> core::ops::Deref for ResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource
    }
}

impl<'a, T: Resource> core::ops::DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource
    }
}
