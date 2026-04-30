//! Render Object Implementation
//!
//! Render objects are extracted from Main World components
//! and used to generate draw commands.

use crate::math::{Color, Rect, Vec2, Vec3};

/// Render Object - An object ready for rendering
///
/// These are created during the Extract phase from
/// Main World entities with Transform and Sprite components.
#[derive(Clone, Debug)]
pub struct RenderObject {
    /// Position in world space
    pub position: Vec3,
    /// Rotation in radians
    pub rotation: f32,
    /// Scale
    pub scale: Vec2,
    /// Sprite color
    pub color: Color,
    /// Sprite size
    pub size: Vec2,
    /// Visibility
    pub visible: bool,
    /// Z-order (depth)
    pub z_order: i32,
}

impl RenderObject {
    /// Create a new render object
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            color: Color::WHITE,
            size: Vec2::new(32.0, 32.0),
            visible: true,
            z_order: 0,
        }
    }

    /// Create from position
    pub fn from_position(x: f32, y: f32) -> Self {
        Self {
            position: Vec3::new(x, y, 0.0),
            rotation: 0.0,
            scale: Vec2::ONE,
            color: Color::WHITE,
            size: Vec2::new(32.0, 32.0),
            visible: true,
            z_order: 0,
        }
    }

    /// Set position
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Vec3::new(x, y, z);
        self
    }

    /// Set rotation
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set scale
    pub fn with_scale(mut self, x: f32, y: f32) -> Self {
        self.scale = Vec2::new(x, y);
        self
    }

    /// Set color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Set visibility
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set Z-order
    pub fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    /// Get the screen-space bounding rectangle
    pub fn get_bounds(&self) -> Rect {
        let half_width = (self.size.x * self.scale.x) / 2.0;
        let half_height = (self.size.y * self.scale.y) / 2.0;
        
        Rect::new(
            self.position.x - half_width,
            self.position.y - half_height,
            self.size.x * self.scale.x,
            self.size.y * self.scale.y,
        )
    }

    /// Get the four corners of the sprite (for rotation)
    pub fn get_corners(&self) -> [Vec2; 4] {
        let bounds = self.get_bounds();
        let center = Vec2::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        );
        
        let half_width = bounds.width / 2.0;
        let half_height = bounds.height / 2.0;
        
        let cos = libm::cosf(self.rotation);
        let sin = libm::sinf(self.rotation);
        
        // Local corners relative to center
        let local_corners = [
            Vec2::new(-half_width, -half_height), // Top-left
            Vec2::new(half_width, -half_height),  // Top-right
            Vec2::new(half_width, half_height),   // Bottom-right
            Vec2::new(-half_width, half_height),  // Bottom-left
        ];
        
        // Rotate and translate
        let mut world_corners = [Vec2::ZERO; 4];
        for (i, local) in local_corners.iter().enumerate() {
            world_corners[i] = Vec2::new(
                center.x + local.x * cos - local.y * sin,
                center.y + local.x * sin + local.y * cos,
            );
        }
        
        world_corners
    }
}

impl Default for RenderObject {
    fn default() -> Self {
        Self::new()
    }
}


