//! Commands Implementation
//!
//! Provides deferred entity/component operations, aligned with Bevy's Commands.
//! Commands are queued during system execution and applied at the end of the stage.

use super::world::MainWorld;
use super::entity::Entity;
use super::component::Component;
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::marker::PhantomData;

/// Commands for spawning/despawning entities and inserting/removing components
///
/// Usage:
/// ```rust
/// fn my_system(mut commands: Commands) {
///     // Spawn a new entity
///     let entity = commands.spawn_empty();
///     
///     // Spawn with components
///     commands.spawn((Transform::default(), Sprite::default()));
///     
///     // Insert component to existing entity
///     commands.insert(entity, Transform::default());
///     
///     // Despawn entity
///     commands.despawn(entity);
/// }
/// ```
pub struct Commands<'w, 's> {
    spawn_queue: Vec<SpawnCommand>,
    despawn_queue: Vec<Entity>,
    insert_queue: Vec<InsertCommand>,
    state: Option<&'s mut CommandsState>,
    _world: PhantomData<&'w ()>,
}

struct SpawnCommand {
    placeholder: Entity,
}

struct InsertCommand {
    placeholder: Entity,
    component_type_id: core::any::TypeId,
    component_ptr: *mut u8,
    drop_fn: unsafe fn(*mut u8),
    insert_fn: unsafe fn(*mut u8, Entity, &mut MainWorld),
}

// SAFETY: InsertCommand only stores Send components
unsafe impl Send for InsertCommand {}

impl<'w, 's> Commands<'w, 's> {
    /// Create new Commands with a reference to state
    pub fn new(state: &'s mut CommandsState) -> Self {
        // Take ownership of commands from state
        let spawn_queue = core::mem::take(&mut state.spawn_queue);
        let despawn_queue = core::mem::take(&mut state.despawn_queue);
        let insert_queue = core::mem::take(&mut state.insert_queue);
        
        Self {
            spawn_queue,
            despawn_queue,
            insert_queue,
            state: Some(state),
            _world: PhantomData,
        }
    }

    /// Spawn a new empty entity and return a placeholder ID
    /// 
    /// The placeholder will be replaced with the real entity ID when commands are applied.
    pub fn spawn_empty(&mut self) -> Entity {
        static mut NEXT_PLACEHOLDER: u64 = 0;
        let placeholder = unsafe {
            NEXT_PLACEHOLDER += 1;
            Entity::new(NEXT_PLACEHOLDER)
        };
        self.spawn_queue.push(SpawnCommand { placeholder });
        placeholder
    }

    /// Spawn a new entity with a component
    pub fn spawn_one<C: Component>(&mut self, component: C) -> Entity {
        let placeholder = self.spawn_empty();
        self.insert(placeholder, component);
        placeholder
    }

    /// Start building a new entity (returns EntityCommands for builder pattern)
    pub fn spawn(&mut self) -> EntityCommands<'_> {
        let placeholder = self.spawn_empty();
        // SAFETY: EntityCommands lifetime is bound to Commands lifetime
        // The 'static lifetime on EntityCommands is a workaround for the type system
        let commands_static = unsafe { 
            &mut *(self as *mut Commands<'w, 's> as *mut Commands<'static, 'static>)
        };
        EntityCommands::new(placeholder, commands_static)
    }

    /// Insert a component to an entity (using placeholder)
    pub fn insert<C: Component>(&mut self, placeholder: Entity, component: C) {
        let component_ptr = Box::into_raw(Box::new(component)) as *mut u8;
        
        unsafe fn drop_fn<C>(ptr: *mut u8) {
            drop(Box::from_raw(ptr as *mut C));
        }
        
        unsafe fn insert_fn<C: Component>(ptr: *mut u8, entity: Entity, world: &mut MainWorld) {
            let component = *Box::from_raw(ptr as *mut C);
            world.insert_component(entity, component);
        }
        
        self.insert_queue.push(InsertCommand {
            placeholder,
            component_type_id: core::any::TypeId::of::<C>(),
            component_ptr,
            drop_fn: drop_fn::<C>,
            insert_fn: insert_fn::<C>,
        });
    }

    /// Despawn an entity
    pub fn despawn(&mut self, entity: Entity) {
        self.despawn_queue.push(entity);
    }

    /// Apply all queued commands to the world
    pub fn apply(&mut self, world: &mut MainWorld) {
        // Create a mapping from placeholder entities to real entities
        let mut entity_map: alloc::collections::BTreeMap<u64, Entity> = alloc::collections::BTreeMap::new();
        
        // Process spawns first and build the mapping
        for cmd in self.spawn_queue.drain(..) {
            let real_entity = world.spawn();
            entity_map.insert(cmd.placeholder.id(), real_entity);
        }
        
        // Process inserts - map placeholder entities to real ones
        for cmd in self.insert_queue.drain(..) {
            unsafe {
                // Get the real entity from the mapping
                let real_entity = if let Some(&entity) = entity_map.get(&cmd.placeholder.id()) {
                    entity
                } else {
                    // If not in mapping, use the placeholder (might be an existing entity)
                    cmd.placeholder
                };
                
                (cmd.insert_fn)(cmd.component_ptr, real_entity, world);
            }
        }
        
        // Process despawns last
        for entity in self.despawn_queue.drain(..) {
            world.despawn(entity);
        }
    }

    /// Check if there are any pending commands
    pub fn is_empty(&self) -> bool {
        self.spawn_queue.is_empty() && self.despawn_queue.is_empty() && self.insert_queue.is_empty()
    }

    /// Move commands back to state
    fn move_to_state(&mut self) {
        if let Some(ref mut state) = self.state {
            state.spawn_queue.append(&mut self.spawn_queue);
            state.despawn_queue.append(&mut self.despawn_queue);
            state.insert_queue.append(&mut self.insert_queue);
        }
    }
}

impl Drop for Commands<'_, '_> {
    fn drop(&mut self) {
        // Move commands back to state so they can be applied later
        self.move_to_state();
    }
}

/// A command that can be applied to the World
pub trait Command {
    fn apply(self, world: &mut MainWorld);
}

/// EntityCommands - Builder pattern for entity operations
///
/// Usage:
/// ```rust
/// fn my_system(mut commands: Commands) {
///     // Spawn with builder pattern
///     commands.spawn()
///         .insert(Transform::default())
///         .insert(Sprite::default());
/// }
/// ```
pub struct EntityCommands<'a> {
    placeholder: Entity,
    commands: &'a mut Commands<'static, 'static>,
}

impl<'a> EntityCommands<'a> {
    /// Create new EntityCommands
    pub fn new(placeholder: Entity, commands: &'a mut Commands<'static, 'static>) -> Self {
        Self { placeholder, commands }
    }

    /// Insert a component
    pub fn insert<C: Component>(mut self, component: C) -> Self {
        self.commands.insert(self.placeholder, component);
        self
    }

    /// Despawn the entity
    pub fn despawn(mut self) {
        self.commands.despawn(self.placeholder);
    }
}

// === SystemParam Implementation ===

use super::system_param::SystemParam;

/// State for Commands SystemParam
pub struct CommandsState {
    pub(crate) spawn_queue: Vec<SpawnCommand>,
    pub(crate) despawn_queue: Vec<Entity>,
    pub(crate) insert_queue: Vec<InsertCommand>,
}

impl Default for CommandsState {
    fn default() -> Self {
        Self {
            spawn_queue: Vec::new(),
            despawn_queue: Vec::new(),
            insert_queue: Vec::new(),
        }
    }
}

impl CommandsState {
    /// Apply all queued commands to the world
    pub fn apply(&mut self, world: &mut MainWorld) {
        // Create a mapping from placeholder entities to real entities
        let mut entity_map: alloc::collections::BTreeMap<u64, Entity> = alloc::collections::BTreeMap::new();
        
        // Process spawns first and build the mapping
        for cmd in self.spawn_queue.drain(..) {
            let real_entity = world.spawn();
            entity_map.insert(cmd.placeholder.id(), real_entity);
        }
        
        // Process inserts - map placeholder entities to real ones
        for cmd in self.insert_queue.drain(..) {
            unsafe {
                // Get the real entity from the mapping
                let real_entity = if let Some(&entity) = entity_map.get(&cmd.placeholder.id()) {
                    entity
                } else {
                    // If not in mapping, use the placeholder (might be an existing entity)
                    cmd.placeholder
                };
                
                (cmd.insert_fn)(cmd.component_ptr, real_entity, world);
            }
        }
        
        // Process despawns last
        for entity in self.despawn_queue.drain(..) {
            world.despawn(entity);
        }
    }

    /// Check if there are any pending commands
    pub fn is_empty(&self) -> bool {
        self.spawn_queue.is_empty() && self.despawn_queue.is_empty() && self.insert_queue.is_empty()
    }
}

impl SystemParam for Commands<'_, '_> {
    type Item<'w, 's> = Commands<'w, 's>;
    type State = CommandsState;

    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        _world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        Commands::new(state)
    }
}
