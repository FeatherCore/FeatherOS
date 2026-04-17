//! Mouse Input Types
//!
//! Mouse button, motion, and scroll events and resources.

use crate::math::Vec2;
use crate::resources::Resource;

/// Mouse buttons
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

/// State of a button
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
}

/// Mouse button input event
#[derive(Clone, Debug)]
pub struct MouseButtonInput {
    pub button: MouseButton,
    pub state: ButtonState,
}

impl crate::event::Event for MouseButtonInput {}
unsafe impl Send for MouseButtonInput {}
unsafe impl Sync for MouseButtonInput {}

/// Mouse motion event (relative movement)
#[derive(Clone, Debug)]
pub struct MouseMotion {
    pub delta: Vec2,
}

impl crate::event::Event for MouseMotion {}
unsafe impl Send for MouseMotion {}
unsafe impl Sync for MouseMotion {}

/// Mouse scroll unit
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseScrollUnit {
    Line,
    Pixel,
}

/// Mouse wheel event
#[derive(Clone, Debug)]
pub struct MouseWheel {
    pub unit: MouseScrollUnit,
    pub x: f32,
    pub y: f32,
}

impl crate::event::Event for MouseWheel {}
unsafe impl Send for MouseWheel {}
unsafe impl Sync for MouseWheel {}

/// Mouse input resource - combines all mouse state
#[derive(Clone, Debug)]
pub struct MouseInput {
    /// Current mouse position
    pub position: Vec2,
    /// Previous mouse position
    pub last_position: Vec2,
}

impl MouseInput {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            last_position: Vec2::ZERO,
        }
    }

    /// Update position and return delta
    pub fn update_position(&mut self, new_position: Vec2) -> Vec2 {
        self.last_position = self.position;
        self.position = new_position;
        self.position - self.last_position
    }

    /// Get the delta from last position
    pub fn delta(&self) -> Vec2 {
        self.position - self.last_position
    }
}

impl Default for MouseInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for MouseInput {}

/// Accumulated mouse motion (reset each frame)
#[derive(Clone, Debug)]
pub struct AccumulatedMouseMotion {
    pub delta: Vec2,
}

impl AccumulatedMouseMotion {
    pub fn new() -> Self {
        Self {
            delta: Vec2::ZERO,
        }
    }

    pub fn clear(&mut self) {
        self.delta = Vec2::ZERO;
    }

    pub fn accumulate(&mut self, delta: Vec2) {
        self.delta += delta;
    }
}

impl Default for AccumulatedMouseMotion {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for AccumulatedMouseMotion {}

/// Accumulated mouse scroll (reset each frame)
#[derive(Clone, Debug)]
pub struct AccumulatedMouseScroll {
    pub unit: MouseScrollUnit,
    pub delta: Vec2,
}

impl AccumulatedMouseScroll {
    pub fn new() -> Self {
        Self {
            unit: MouseScrollUnit::Line,
            delta: Vec2::ZERO,
        }
    }

    pub fn clear(&mut self) {
        self.delta = Vec2::ZERO;
    }

    pub fn accumulate(&mut self, delta: Vec2, unit: MouseScrollUnit) {
        self.unit = unit;
        self.delta += delta;
    }
}

impl Default for AccumulatedMouseScroll {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for AccumulatedMouseScroll {}
