//! Change Detection Implementation
//!
//! Provides change detection for components, aligned with Bevy's change detection system.
//! Tracks when components are added, modified, or removed.

use super::component::Component;
use super::entity::Entity;
use alloc::collections::BTreeMap;
use core::any::TypeId;

/// Change ticks for a component
#[derive(Clone, Copy, Debug)]
pub struct ChangeTicks {
    /// Tick when component was added
    pub added: u64,
    /// Tick when component was last modified
    pub changed: u64,
}

impl ChangeTicks {
    pub fn new(current_tick: u64) -> Self {
        Self {
            added: current_tick,
            changed: current_tick,
        }
    }

    /// Mark as changed at current tick
    pub fn mark_changed(&mut self, current_tick: u64) {
        self.changed = current_tick;
    }

    /// Check if component was added since last system run
    pub fn is_added(&self, last_run_tick: u64, current_tick: u64) -> bool {
        self.added >= last_run_tick && self.added < current_tick
    }

    /// Check if component was changed since last system run
    pub fn is_changed(&self, last_run_tick: u64, current_tick: u64) -> bool {
        self.changed >= last_run_tick && self.changed < current_tick
    }
}

/// Change detection storage for all components
pub struct ChangeDetection {
    /// Current tick counter
    current_tick: u64,
    /// Component change tracking: (Entity, ComponentType) -> ChangeTicks
    component_ticks: BTreeMap<(Entity, TypeId), ChangeTicks>,
}

impl ChangeDetection {
    pub fn new() -> Self {
        Self {
            current_tick: 1,
            component_ticks: BTreeMap::new(),
        }
    }

    /// Increment tick counter (called each frame)
    pub fn increment_tick(&mut self) {
        self.current_tick = self.current_tick.wrapping_add(1);
    }

    /// Get current tick
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Mark component as added
    pub fn mark_added<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        let ticks = ChangeTicks::new(self.current_tick);
        self.component_ticks.insert((entity, type_id), ticks);
    }

    /// Mark component as changed
    pub fn mark_changed<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        if let Some(ticks) = self.component_ticks.get_mut(&(entity, type_id)) {
            ticks.mark_changed(self.current_tick);
        }
    }

    /// Remove component tracking
    pub fn remove<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        self.component_ticks.remove(&(entity, type_id));
    }

    /// Check if component was added since last run
    pub fn is_added<T: Component>(&self, entity: Entity, last_run_tick: u64) -> bool {
        let type_id = TypeId::of::<T>();
        self.component_ticks
            .get(&(entity, type_id))
            .map(|ticks| ticks.is_added(last_run_tick, self.current_tick))
            .unwrap_or(false)
    }

    /// Check if component was changed since last run
    pub fn is_changed<T: Component>(&self, entity: Entity, last_run_tick: u64) -> bool {
        let type_id = TypeId::of::<T>();
        self.component_ticks
            .get(&(entity, type_id))
            .map(|ticks| ticks.is_changed(last_run_tick, self.current_tick))
            .unwrap_or(false)
    }

    /// Get change ticks for a component
    pub fn get_ticks<T: Component>(&self, entity: Entity) -> Option<&ChangeTicks> {
        let type_id = TypeId::of::<T>();
        self.component_ticks.get(&(entity, type_id))
    }
}

impl Default for ChangeDetection {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper for mutable component access with change detection
///
/// Usage:
/// ```rust
/// fn my_system(mut transforms: Query<Mut<Transform>>) {
///     for mut transform in transforms.iter_mut() {
///         transform.position.x += 1.0; // Automatically marks as changed
///     }
/// }
/// ```
pub struct Mut<'a, T: Component> {
    value: &'a mut T,
    entity: Entity,
    change_detection: &'a mut ChangeDetection,
    _phantom: core::marker::PhantomData<T>,
}

impl<'a, T: Component> Mut<'a, T> {
    pub fn new(
        value: &'a mut T,
        entity: Entity,
        change_detection: &'a mut ChangeDetection,
    ) -> Self {
        Self {
            value,
            entity,
            change_detection,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Check if component was added this frame
    pub fn is_added(&self) -> bool {
        // TODO: Need to pass last_run_tick somehow
        false
    }

    /// Check if component was changed this frame
    pub fn is_changed(&self) -> bool {
        // TODO: Need to pass last_run_tick somehow
        false
    }
}

impl<'a, T: Component> core::ops::Deref for Mut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: Component> core::ops::DerefMut for Mut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Mark as changed when mutably dereferenced
        self.change_detection.mark_changed::<T>(self.entity);
        self.value
    }
}

/// Ref wrapper for immutable component access with change detection info
pub struct Ref<'a, T: Component> {
    value: &'a T,
    entity: Entity,
    change_detection: &'a ChangeDetection,
    _phantom: core::marker::PhantomData<T>,
}

impl<'a, T: Component> Ref<'a, T> {
    pub fn new(value: &'a T, entity: Entity, change_detection: &'a ChangeDetection) -> Self {
        Self {
            value,
            entity,
            change_detection,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Check if component was added this frame
    pub fn is_added(&self) -> bool {
        // TODO: Need to pass last_run_tick somehow
        false
    }

    /// Check if component was changed this frame
    pub fn is_changed(&self) -> bool {
        // TODO: Need to pass last_run_tick somehow
        false
    }
}

impl<'a, T: Component> core::ops::Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}
