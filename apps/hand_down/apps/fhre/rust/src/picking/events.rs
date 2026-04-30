//! Pointer Events
//!
//! Event types for pointer interactions, inspired by Bevy's event system.

use crate::Entity;
use crate::math::Vec2;
use crate::event::Event;
use super::{PointerId, PointerLocation, HitData};

#[derive(Debug, Clone, Copy)]
pub struct Pointer<E> {
    pub entity: Entity,
    pub pointer_id: PointerId,
    pub pointer_location: PointerLocation,
    pub event: E,
}

impl<E> core::ops::Deref for Pointer<E> {
    type Target = E;
    
    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl<E: Clone> Pointer<E> {
    pub fn new(id: PointerId, location: PointerLocation, event: E, entity: Entity) -> Self {
        Self {
            pointer_id: id,
            pointer_location: location,
            event,
            entity,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Over {
    pub hit: HitData,
}

impl Event for Pointer<Over> {}

#[derive(Debug, Clone, Copy)]
pub struct Out {
    pub hit: HitData,
}

impl Event for Pointer<Out> {}

#[derive(Debug, Clone, Copy)]
pub struct Enter {
    pub hit: HitData,
}

impl Event for Pointer<Enter> {}

#[derive(Debug, Clone, Copy)]
pub struct Leave {
    pub hit: HitData,
}

impl Event for Pointer<Leave> {}

#[derive(Debug, Clone, Copy)]
pub struct Press {
    pub hit: HitData,
    pub button: super::PointerButton,
}

impl Event for Pointer<Press> {}

#[derive(Debug, Clone, Copy)]
pub struct Release {
    pub hit: HitData,
    pub button: super::PointerButton,
}

impl Event for Pointer<Release> {}

#[derive(Debug, Clone, Copy)]
pub struct Click {
    pub hit: HitData,
    pub button: super::PointerButton,
}

impl Event for Pointer<Click> {}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub hit: HitData,
    pub delta: Vec2,
}

impl Event for Pointer<Move> {}

#[derive(Debug, Clone, Copy)]
pub struct DragStart {
    pub hit: HitData,
    pub button: super::PointerButton,
}

impl Event for Pointer<DragStart> {}

#[derive(Debug, Clone, Copy)]
pub struct Drag {
    pub hit: HitData,
    pub delta: Vec2,
    pub button: super::PointerButton,
}

impl Event for Pointer<Drag> {}

#[derive(Debug, Clone, Copy)]
pub struct DragEnd {
    pub hit: HitData,
    pub button: super::PointerButton,
}

impl Event for Pointer<DragEnd> {}
