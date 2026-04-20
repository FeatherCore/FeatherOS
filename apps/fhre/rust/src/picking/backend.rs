//! Picking Backend

use crate::Entity;
use crate::math::Vec2;
use super::pointer::PointerId;

#[derive(Debug, Clone, Copy)]
pub struct HitData {
    pub position: Vec2,
    pub depth: f32,
}

#[derive(Debug, Clone)]
pub struct PointerHits {
    pub pointer: PointerId,
    pub entity: Entity,
    pub hit: HitData,
    pub order: f32,
}
