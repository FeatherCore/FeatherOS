//! System Implementation
//!
//! Systems are functions that operate on entities and components.
/// They are the "S" in ECS (Entity-Component-System).

use super::world::MainWorld;
use alloc::vec::Vec;

/// System trait - Executable system
/// 
/// Systems are functions that operate on the world.
/// They can query entities, modify components, and spawn/despawn entities.
pub trait System {
    /// Run the system
    fn run(&mut self, world: &mut MainWorld);
}

/// IntoSystem trait - Convert functions into systems
/// 
/// This allows regular functions to be used as systems.
pub trait IntoSystem {
    type System: System;

    /// Convert into a system
    fn into_system(self) -> Self::System;
}

/// Function system wrapper
pub struct FunctionSystem<F> {
    func: F,
}

impl<F> System for FunctionSystem<F>
where
    F: FnMut(&mut MainWorld),
{
    fn run(&mut self, world: &mut MainWorld) {
        (self.func)(world);
    }
}

impl<F> IntoSystem for F
where
    F: FnMut(&mut MainWorld) + 'static,
{
    type System = FunctionSystem<F>;

    fn into_system(self) -> Self::System {
        FunctionSystem { func: self }
    }
}

/// System function that updates positions based on velocity
pub fn velocity_system(world: &mut MainWorld) {
    use super::component::{Transform, Velocity};
    use super::entity::Entity;
    
    // Collect updates first to avoid borrow issues
    let updates: Vec<(u64, f32, f32)> = {
        let mut updates = Vec::new();
        for (entity, transform) in world.query::<Transform>() {
            if let Some(velocity) = world.get_component::<Velocity>(entity) {
                let new_x = transform.position.x + velocity.linear.x;
                let new_y = transform.position.y + velocity.linear.y;
                updates.push((entity.id(), new_x, new_y));
            }
        }
        updates
    };
    
    // Apply updates
    for (id, x, y) in updates {
        if let Some(transform) = world.get_component_mut::<Transform>(Entity::new(id)) {
            transform.position.x = x;
            transform.position.y = y;
        }
    }
}

/// System function that rotates sprites
pub fn rotation_system(world: &mut MainWorld) {
    use super::component::{Transform, Velocity};
    use super::entity::Entity;
    
    let updates: Vec<(u64, f32)> = {
        let mut updates = Vec::new();
        for (entity, transform) in world.query::<Transform>() {
            if let Some(velocity) = world.get_component::<Velocity>(entity) {
                let new_rotation = transform.rotation + velocity.angular;
                updates.push((entity.id(), new_rotation));
            }
        }
        updates
    };
    
    for (id, rotation) in updates {
        if let Some(transform) = world.get_component_mut::<Transform>(Entity::new(id)) {
            transform.rotation = rotation;
        }
    }
}
