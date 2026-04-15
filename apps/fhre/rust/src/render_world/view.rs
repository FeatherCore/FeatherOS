//! View Module
//!
//! Manages render targets and view configuration.
//! Simplified version of Bevy's view system for embedded systems.

use crate::math::{Mat4, Vec3, Rect};

/// View configuration - defines a render target
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// Viewport rectangle (x, y, width, height)
    pub viewport: Rect,
    /// Projection matrix
    pub projection: Mat4,
    /// View matrix (camera transform)
    pub view: Mat4,
    /// View-projection matrix (cached)
    pub view_projection: Mat4,
    /// Camera position in world space
    pub camera_position: Vec3,
    /// Near plane distance
    pub near: f32,
    /// Far plane distance
    pub far: f32,
    /// Whether this is an orthographic projection
    pub orthographic: bool,
}

impl View {
    /// Create a new 2D orthographic view
    pub fn new_2d(width: f32, height: f32) -> Self {
        let viewport = Rect::new(0.0, 0.0, width, height);
        let projection = Mat4::orthographic_rh(0.0, width, height, 0.0, -1000.0, 1000.0);
        let view = Mat4::IDENTITY;
        let view_projection = projection * view;
        
        Self {
            viewport,
            projection,
            view,
            view_projection,
            camera_position: Vec3::ZERO,
            near: -1000.0,
            far: 1000.0,
            orthographic: true,
        }
    }

    /// Create a new 3D perspective view
    pub fn new_3d(width: f32, height: f32, fov_degrees: f32) -> Self {
        let viewport = Rect::new(0.0, 0.0, width, height);
        let aspect_ratio = width / height;
        let projection = Mat4::perspective_rh(
            fov_degrees.to_radians(),
            aspect_ratio,
            0.1,
            1000.0,
        );
        let view = Mat4::IDENTITY;
        let view_projection = projection * view;
        
        Self {
            viewport,
            projection,
            view,
            view_projection,
            camera_position: Vec3::new(0.0, 0.0, 10.0),
            near: 0.1,
            far: 1000.0,
            orthographic: false,
        }
    }

    /// Set camera position
    pub fn set_camera_position(&mut self, position: Vec3) {
        self.camera_position = position;
        self.update_view_matrix();
    }

    /// Set camera look-at target
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        self.view = Mat4::look_at_rh(self.camera_position, target, up);
        self.update_view_projection();
    }

    /// Update view matrix from camera position and orientation
    fn update_view_matrix(&mut self) {
        // Default: camera looks at origin from positive Z
        self.view = Mat4::look_at_rh(
            self.camera_position,
            Vec3::ZERO,
            Vec3::Y,
        );
        self.update_view_projection();
    }

    /// Update view-projection matrix
    fn update_view_projection(&mut self) {
        self.view_projection = self.projection * self.view;
    }

    /// Set viewport
    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.viewport = Rect::new(x, y, width, height);
    }

    /// Get viewport width
    pub fn width(&self) -> f32 {
        self.viewport.width
    }

    /// Get viewport height
    pub fn height(&self) -> f32 {
        self.viewport.height
    }

    /// Project world position to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec3) -> Option<(f32, f32)> {
        // Transform to clip space
        let clip_pos = self.view_projection * world_pos.extend(1.0);
        
        // For perspective projection, check if behind camera
        if clip_pos.w <= 0.0 {
            return None;
        }
        
        // Perspective divide
        let ndc_x = clip_pos.x / clip_pos.w;
        let ndc_y = clip_pos.y / clip_pos.w;
        
        // Check if within NDC bounds (-1 to 1)
        if ndc_x < -1.0 || ndc_x > 1.0 || ndc_y < -1.0 || ndc_y > 1.0 {
            // Still return the value, but it might be off-screen
        }
        
        // Convert to screen coordinates
        let screen_x = (ndc_x + 1.0) * 0.5 * self.viewport.width + self.viewport.x;
        let screen_y = (1.0 - ndc_y) * 0.5 * self.viewport.height + self.viewport.y;
        
        Some((screen_x, screen_y))
    }

    /// Unproject screen coordinates to world ray
    pub fn screen_to_world_ray(&self, screen_x: f32, screen_y: f32) -> (Vec3, Vec3) {
        // Convert to NDC
        let ndc_x = (screen_x - self.viewport.x) / self.viewport.width * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y - self.viewport.y) / self.viewport.height * 2.0;
        
        // Ray origin (camera position)
        let origin = self.camera_position;
        
        // Ray direction
        let clip_pos = Vec3::new(ndc_x, ndc_y, 1.0);
        let direction = if let Some(inv_view_proj) = self.view_projection.inverse() {
            let world_pos = (inv_view_proj * clip_pos.extend(1.0)).truncate();
            (world_pos - origin).normalize()
        } else {
            // Fallback: use view direction
            Vec3::new(0.0, 0.0, -1.0)
        };
        
        (origin, direction)
    }
}

impl Default for View {
    fn default() -> Self {
        Self::new_2d(800.0, 600.0)
    }
}

/// View target - describes where to render
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewTarget {
    /// Render to screen/framebuffer
    Screen,
    /// Render to texture (for post-processing)
    Texture(u32), // texture ID
}

impl Default for ViewTarget {
    fn default() -> Self {
        ViewTarget::Screen
    }
}

/// Clear configuration - defines how to clear the view
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClearConfig {
    /// Whether to clear color
    pub clear_color: bool,
    /// Color to clear with
    pub color: crate::math::Color,
    /// Whether to clear depth
    pub clear_depth: bool,
    /// Depth value to clear with
    pub depth: f32,
}

impl ClearConfig {
    /// Create clear config with color
    pub fn color(color: crate::math::Color) -> Self {
        Self {
            clear_color: true,
            color,
            clear_depth: true,
            depth: 1.0,
        }
    }

    /// Create clear config with depth only
    pub fn depth(depth: f32) -> Self {
        Self {
            clear_color: false,
            color: crate::math::Color::BLACK,
            clear_depth: true,
            depth,
        }
    }

    /// Don't clear
    pub fn none() -> Self {
        Self {
            clear_color: false,
            color: crate::math::Color::BLACK,
            clear_depth: false,
            depth: 1.0,
        }
    }
}

impl Default for ClearConfig {
    fn default() -> Self {
        Self::color(crate::math::Color::BLACK)
    }
}

/// View bundle - combines view, target, and clear config
#[derive(Debug, Clone)]
pub struct ViewBundle {
    /// View configuration
    pub view: View,
    /// Render target
    pub target: ViewTarget,
    /// Clear configuration
    pub clear: ClearConfig,
}

impl ViewBundle {
    /// Create a new view bundle for screen rendering
    pub fn new_screen(width: f32, height: f32) -> Self {
        Self {
            view: View::new_2d(width, height),
            target: ViewTarget::Screen,
            clear: ClearConfig::default(),
        }
    }

    /// Create a new view bundle for texture rendering
    pub fn new_texture(width: f32, height: f32, texture_id: u32) -> Self {
        Self {
            view: View::new_2d(width, height),
            target: ViewTarget::Texture(texture_id),
            clear: ClearConfig::default(),
        }
    }

    /// Set clear color
    pub fn with_clear_color(mut self, color: crate::math::Color) -> Self {
        self.clear = ClearConfig::color(color);
        self
    }

    /// Disable clearing
    pub fn without_clear(mut self) -> Self {
        self.clear = ClearConfig::none();
        self
    }
}

impl Default for ViewBundle {
    fn default() -> Self {
        Self::new_screen(800.0, 600.0)
    }
}
