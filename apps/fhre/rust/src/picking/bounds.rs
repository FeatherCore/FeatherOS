//! Pickable Bounds

use crate::{Component, math::Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PickableBounds {
    pub width: f32,
    pub height: f32,
}

impl PickableBounds {
    pub fn from_size(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn contains_point(&self, center: Vec2, point: Vec2) -> bool {
        let half_w = self.width / 2.0;
        let half_h = self.height / 2.0;
        point.x >= center.x - half_w
            && point.x <= center.x + half_w
            && point.y >= center.y - half_h
            && point.y <= center.y + half_h
    }
}

impl Component for PickableBounds {
    fn type_name() -> &'static str { "PickableBounds" }
}
