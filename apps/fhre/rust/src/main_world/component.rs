//! Component Implementation
//!
//! Components are data containers attached to entities.
//! They represent properties like position, velocity, sprite, etc.
//!
//! For transforms, use `Transform` from the `node` module.

use crate::math::{Vec2, Color};

/// Component trait - Marker trait for component types
/// 
/// All component types must implement this trait.
/// Components should be simple data structures.
pub trait Component: 'static + Send + Sync {
    /// Get the type name of this component
    fn type_name() -> &'static str
    where
        Self: Sized;
}

/// Sprite component - Visual representation
#[derive(Clone, Debug, PartialEq)]
pub struct Sprite {
    pub color: Color,
    pub width: f32,
    pub height: f32,
    pub visible: bool,
}

impl Sprite {
    /// Create a new sprite with default values
    pub fn new() -> Self {
        Self {
            color: Color::WHITE,
            width: 32.0,
            height: 32.0,
            visible: true,
        }
    }

    /// Create a sprite with specific size
    pub fn with_size(width: f32, height: f32) -> Self {
        Self {
            color: Color::WHITE,
            width,
            height,
            visible: true,
        }
    }

    /// Create a sprite with specific color
    pub fn with_color(color: Color) -> Self {
        Self {
            color,
            width: 32.0,
            height: 32.0,
            visible: true,
        }
    }

    /// Create a sprite with size and color
    pub fn new_with_color(width: f32, height: f32, color: Color) -> Self {
        Self {
            color,
            width,
            height,
            visible: true,
        }
    }

    /// Set color
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Set size
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl Default for Sprite {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Sprite {
    fn type_name() -> &'static str {
        "Sprite"
    }
}

/// Velocity component - For physics/movement
#[derive(Clone, Debug, PartialEq)]
pub struct Velocity {
    pub linear: Vec2,
    pub angular: f32,
}

impl Velocity {
    /// Create zero velocity
    pub fn new() -> Self {
        Self {
            linear: Vec2::new(0.0, 0.0),
            angular: 0.0,
        }
    }

    /// Create velocity from components
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self {
            linear: Vec2::new(x, y),
            angular: 0.0,
        }
    }

    /// Create velocity with angular component
    pub fn with_angular(x: f32, y: f32, angular: f32) -> Self {
        Self {
            linear: Vec2::new(x, y),
            angular,
        }
    }

    /// Set linear velocity
    pub fn set_linear(&mut self, x: f32, y: f32) {
        self.linear.x = x;
        self.linear.y = y;
    }

    /// Add to linear velocity
    pub fn add_linear(&mut self, dx: f32, dy: f32) {
        self.linear.x += dx;
        self.linear.y += dy;
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Velocity {
    fn type_name() -> &'static str {
        "Velocity"
    }
}
