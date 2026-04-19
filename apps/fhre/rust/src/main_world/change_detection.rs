//! Change Detection Implementation
//!
//! Provides change detection for components, aligned with Bevy's change detection system.
//! Tracks when components are added, modified, or removed.
//!
//! # How it works
//!
//! Each frame, `ChangeDetection::increment_tick()` advances the tick counter.
//! When a component is inserted, `mark_added` records the current tick.
//! When a component is mutably accessed via `Mut<T>`, `mark_changed` records the tick.
//! `Mut<T>::is_added()` / `is_changed()` compare against `last_run_tick` to determine
//! if the change happened since the last time this system ran.

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
    /// Last tick when systems started running (updated at start of each frame)
    last_run_tick: u64,
    /// Component change tracking: (Entity, ComponentType) -> ChangeTicks
    component_ticks: BTreeMap<(Entity, TypeId), ChangeTicks>,
}

impl ChangeDetection {
    pub fn new() -> Self {
        Self {
            current_tick: 1,
            last_run_tick: 0,
            component_ticks: BTreeMap::new(),
        }
    }

    /// Increment tick counter and update last_run_tick (called each frame)
    pub fn increment_tick(&mut self) {
        self.last_run_tick = self.current_tick;
        self.current_tick = self.current_tick.wrapping_add(1);
    }

    /// Get current tick
    pub fn current_tick(&self) -> u64 {
        self.current_tick
    }

    /// Get last run tick (tick at the start of the current frame)
    pub fn last_run_tick(&self) -> u64 {
        self.last_run_tick
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
/// When `DerefMut` is called, the component is automatically marked as changed.
/// Use `is_added()` / `is_changed()` to check if the component was modified
/// since the last time this system ran.
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

    /// Check if component was added since last system run
    pub fn is_added(&self) -> bool {
        self.change_detection.is_added::<T>(self.entity, self.change_detection.last_run_tick)
    }

    /// Check if component was changed since last system run
    pub fn is_changed(&self) -> bool {
        self.change_detection.is_changed::<T>(self.entity, self.change_detection.last_run_tick)
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
        self.change_detection.mark_changed::<T>(self.entity);
        self.value
    }
}

/// Ref wrapper for immutable component access with change detection info
///
/// Use `is_added()` / `is_changed()` to check if the component was modified
/// since the last time this system ran, without triggering change marks.
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

    /// Check if component was added since last system run
    pub fn is_added(&self) -> bool {
        self.change_detection.is_added::<T>(self.entity, self.change_detection.last_run_tick)
    }

    /// Check if component was changed since last system run
    pub fn is_changed(&self) -> bool {
        self.change_detection.is_changed::<T>(self.entity, self.change_detection.last_run_tick)
    }
}

impl<'a, T: Component> core::ops::Deref for Ref<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

// === SystemParam implementations for Mut<T> and Ref<T> ===

/// State for Mut<T> SystemParam
pub struct MutState<T: Component> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: Component> Default for MutState<T> {
    fn default() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

/// State for Ref<T> SystemParam  
pub struct RefState<T: Component> {
    _marker: core::marker::PhantomData<T>,
}

impl<T: Component> Default for RefState<T> {
    fn default() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

// Note: Mut<T> and Ref<T> as SystemParam would require a way to specify
// which entity to query. In Bevy, this is done via Query<Mut<T>>.
// For now, we provide Query<Mut<T>> support through the Query system.
