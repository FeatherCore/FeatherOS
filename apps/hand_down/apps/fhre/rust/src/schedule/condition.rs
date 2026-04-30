//! Schedule Condition Implementation
//!
//! Provides conditions for controlling when systems run, aligned with Bevy's run conditions.

use crate::main_world::MainWorld;
use crate::resources::Resource;
use alloc::boxed::Box;

/// A condition that determines if a system should run
///
/// Returns true if the system should run, false otherwise.
pub type Condition = Box<dyn Fn(&MainWorld) -> bool>;

/// Common conditions
pub mod common_conditions {
    use super::*;

    /// Run if a resource exists
    pub fn resource_exists<T: Resource>(world: &MainWorld) -> bool {
        world.resources().get::<T>().is_some()
    }

    /// Run if a resource has changed
    pub fn resource_changed<T: Resource>(world: &MainWorld) -> bool {
        // TODO: Implement change detection for resources
        world.resources().get::<T>().is_some()
    }

    /// Run if a resource equals a specific value
    pub fn resource_equals<T: Resource + PartialEq + Clone>(value: T) -> Condition {
        Box::new(move |world| {
            world.resources()
                .get::<T>()
                .map(|r| *r == value)
                .unwrap_or(false)
        })
    }

    /// Always run
    pub fn always(_world: &MainWorld) -> bool {
        true
    }

    /// Never run
    pub fn never(_world: &MainWorld) -> bool {
        false
    }
}

/// Combinators for conditions
pub fn and(a: Condition, b: Condition) -> Condition {
    Box::new(move |world| a(world) && b(world))
}

pub fn or(a: Condition, b: Condition) -> Condition {
    Box::new(move |world| a(world) || b(world))
}

pub fn not(condition: Condition) -> Condition {
    Box::new(move |world| !condition(world))
}
