//! Pointer types for picking system
//!
//! Inspired by Bevy's pointer module.

use crate::math::Vec2;
use crate::event::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PointerId {
    Mouse,
    Touch(u64),
    Custom(u64),
}

impl Default for PointerId {
    fn default() -> Self {
        Self::Mouse
    }
}

impl PointerId {
    pub fn is_mouse(&self) -> bool {
        matches!(self, PointerId::Mouse)
    }
    
    pub fn is_touch(&self) -> bool {
        matches!(self, PointerId::Touch(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressDirection {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy)]
pub struct PointerPress {
    pub primary: bool,
    pub secondary: bool,
    pub middle: bool,
}

impl Default for PointerPress {
    fn default() -> Self {
        Self {
            primary: false,
            secondary: false,
            middle: false,
        }
    }
}

impl PointerPress {
    pub fn is_primary_pressed(&self) -> bool {
        self.primary
    }
    
    pub fn is_any_pressed(&self) -> bool {
        self.primary || self.secondary || self.middle
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointerLocation {
    pub position: Vec2,
}

impl Default for PointerLocation {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PointerAction {
    Press(PointerButton),
    Release(PointerButton),
    Move { delta: Vec2 },
    Cancel,
}

#[derive(Debug, Clone)]
pub struct PointerInput {
    pub pointer_id: PointerId,
    pub location: PointerLocation,
    pub action: PointerAction,
}

impl Event for PointerInput {}

impl PointerInput {
    pub fn new(pointer_id: PointerId, location: PointerLocation, action: PointerAction) -> Self {
        Self {
            pointer_id,
            location,
            action,
        }
    }
    
    pub fn button_just_pressed(&self, target_button: PointerButton) -> bool {
        if let PointerAction::Press(button) = self.action {
            button == target_button
        } else {
            false
        }
    }
    
    pub fn button_just_released(&self, target_button: PointerButton) -> bool {
        if let PointerAction::Release(button) = self.action {
            button == target_button
        } else {
            false
        }
    }
}
