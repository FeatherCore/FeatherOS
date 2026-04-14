//! Main World Implementation
//!
//! The Main World is the primary ECS world containing game entities,
//! components, and systems. This is where the game logic runs.

use super::entity::Entity;
use super::component::Component;
use super::system::{System, IntoSystem};
use crate::resources::Resources;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::any::{TypeId, Any};

/// Main World - Container for ECS data and systems
pub struct MainWorld {
    /// Next entity ID to allocate
    next_entity_id: u64,
    /// Active entities
    entities: Vec<Entity>,
    /// Component storage: TypeId -> EntityId -> Component
    components: BTreeMap<TypeId, BTreeMap<u64, Box<dyn Any>>>,
    /// Systems to run
    systems: Vec<Box<dyn System>>,
    /// Global resources
    resources: Resources,
}

impl MainWorld {
    /// Create a new empty Main World
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            components: BTreeMap::new(),
            systems: Vec::new(),
            resources: Resources::new(),
        }
    }

    /// Spawn a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }

    /// Despawn an entity and all its components
    pub fn despawn(&mut self, entity: Entity) {
        // Remove from entities list
        if let Some(pos) = self.entities.iter().position(|e| e.id() == entity.id()) {
            self.entities.swap_remove(pos);
        }
        
        // Remove all components for this entity
        for (_, storage) in self.components.iter_mut() {
            storage.remove(&entity.id());
        }
    }

    /// Insert a component for an entity
    pub fn insert_component<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self.components
            .entry(type_id)
            .or_insert_with(BTreeMap::new);
        storage.insert(entity.id(), Box::new(component));
    }

    /// Get a component reference
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .and_then(|storage| storage.get(&entity.id()))
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable component reference
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|storage| storage.get_mut(&entity.id()))
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|storage| storage.remove(&entity.id()))
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Query all entities with a specific component type
    pub fn query<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let type_id = TypeId::of::<T>();
        let storage = self.components.get(&type_id);
        
        self.entities.iter()
            .filter_map(move |entity| {
                storage?.get(&entity.id())
                    .and_then(|boxed| boxed.downcast_ref::<T>())
                    .map(|comp| (*entity, comp))
            })
    }

    /// Query all entities with a specific component type (mutable)
    /// Returns a Vec instead of an iterator to avoid lifetime issues
    pub fn query_mut<T: Component>(&mut self) -> Vec<(Entity, &mut T)> {
        let type_id = TypeId::of::<T>();
        let entity_ids: Vec<u64> = self.entities.iter().map(|e| e.id()).collect();
        
        let mut results = Vec::new();
        
        // Get all components for this type first
        if let Some(storage) = self.components.get_mut(&type_id) {
            for id in entity_ids {
                // Use raw pointer to avoid borrow checker issues
                // SAFETY: We know the storage exists and we're only accessing each id once
                if let Some(component) = unsafe {
                    let ptr = storage as *mut BTreeMap<u64, Box<dyn Any>>;
                    (*ptr).get_mut(&id)
                        .and_then(|boxed| boxed.downcast_mut::<T>())
                } {
                    results.push((Entity::new(id), component));
                }
            }
        }
        
        results
    }

    /// Add a system to the world
    pub fn add_system<S: IntoSystem>(&mut self, system: S) 
    where
        <S as IntoSystem>::System: 'static,
    {
        self.systems.push(Box::new(system.into_system()));
    }

    /// Run all systems
    pub fn run_systems(&mut self) {
        // Take systems out temporarily to avoid borrow issues
        let mut systems: Vec<Box<dyn System>> = Vec::new();
        core::mem::swap(&mut systems, &mut self.systems);
        
        // Run each system
        for system in systems.iter_mut() {
            system.run(self);
        }
        
        // Put systems back
        self.systems = systems;
    }

    /// Get reference to resources
    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    /// Get mutable reference to resources
    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }

    /// Get all entities
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Clear all entities and components
    pub fn clear(&mut self) {
        self.entities.clear();
        self.components.clear();
    }
}

impl Default for MainWorld {
    fn default() -> Self {
        Self::new()
    }
}
