//! Camera Resource
//!
//! Declarative camera configuration for FHRE's single 3D camera architecture.
//! Users configure the camera via `insert_resource(Camera::new(...))` in their App builder.
//!
//! # Architecture
//!
//! FHRE uses a single 3D perspective camera as the default camera. The main world
//! is entirely 3D. The camera captures the 3D scene and projects it onto a 2D screen
//! canvas (幕布), which is then presented to the user via a presentation window
//! (X11, framebuffer, etc.).
//!
//! - 3D objects (Cube, SoccerBall): rendered via Camera MVP → screen canvas coordinates
//! - 2D UI (Button): drawn directly in screen canvas pixel coordinates
//! - Both share the same screen canvas coordinate system
//!
//! # Example
//! ```ignore
//! App::new(640, 480)
//!     .add_plugins(DefaultPlugins)
//!     .insert_resource(Camera::perspective_3d(
//!         Vec3::new(320.0, 160.0, 600.0),   // position
//!         Vec3::new(320.0, 160.0, 0.0),      // look-at target
//!         45.0,                                // FOV in degrees
//!     ))
//!     .run();
//! ```

use crate::math::{Vec3, Mat4};
use alloc::string::String;

/// Camera projection type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProjectionType {
    /// Perspective projection for 3D scenes
    Perspective {
        fov_degrees: f32,
        near: f32,
        far: f32,
    },
}

impl ProjectionType {
    pub fn perspective(fov_degrees: f32) -> Self {
        Self::Perspective { fov_degrees, near: 0.1, far: 1000.0 }
    }

    pub fn perspective_full(fov_degrees: f32, near: f32, far: f32) -> Self {
        Self::Perspective { fov_degrees, near, far }
    }

    pub fn is_perspective(&self) -> bool {
        true
    }

    pub fn build_projection_matrix(&self, width: f32, height: f32) -> Mat4 {
        match self {
            ProjectionType::Perspective { fov_degrees, near, far } => {
                let aspect = width / height;
                Mat4::perspective_rh(fov_degrees.to_radians(), aspect, *near, *far)
            }
        }
    }
}

/// Main 3D Camera Resource
///
/// The single 3D perspective camera in FHRE's single-camera architecture.
/// Configured declaratively via `insert_resource()`.
///
/// In the extract phase, the camera's look-at target is overridden to track
/// the PrimaryScreen canvas position, ensuring the camera always points at
/// the screen canvas regardless of canvas movement.
///
/// # Default values
/// When created via `Camera::default()` or `Camera::default_3d()`:
/// - Position: (width/2, height/2, 600) — behind the canvas center
/// - Target: (width/2, height/2, 0) — canvas center
/// - FOV: 45°
#[derive(Clone, Debug)]
pub struct Camera {
    /// Camera position in world space
    pub position: Vec3,
    /// Look-at target point in world space
    pub target: Vec3,
    /// Up vector (typically Y-up)
    pub up: Vec3,
    /// Projection type (always Perspective in single-camera architecture)
    pub projection: ProjectionType,
    /// Camera name/label (for debugging and multi-camera)
    pub name: String,
}

impl Camera {
    /// Create a perspective 3D camera (Bevy-style convenience)
    ///
    /// Uses sensible defaults:
    /// - up = (0, 1, 0) Y-is-up
    /// - name = "Main3D"
    pub fn perspective_3d(position: Vec3, target: Vec3, fov_degrees: f32) -> Self {
        Self {
            position,
            target,
            up: Vec3::new(0.0, 1.0, 0.0),
            projection: ProjectionType::perspective(fov_degrees),
            name: "Main3D".into(),
        }
    }

    /// Create a default 3D camera centered on screen canvas
    ///
    /// Position: (width/2, height/2, 600) — behind the canvas center
    /// Target:   (width/2, height/2, 0)   — canvas center (screen canvas position)
    /// FOV: 45°
    ///
    /// Note: In the extract phase, the target is overridden to track the
    /// actual PrimaryScreen.transform.position, so this default target
    /// only matters when PrimaryScreen is not available.
    pub fn default_3d(width: u32, height: u32) -> Self {
        let w = width as f32;
        let h = height as f32;
        Self::perspective_3d(
            Vec3::new(w / 2.0, h / 2.0, 600.0),
            Vec3::new(w / 2.0, h / 2.0, 0.0),
            45.0,
        )
    }

    /// Build view matrix from current position/target/up
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Build full MVP matrix for given screen dimensions
    pub fn build_view_projection(&self, width: f32, height: f32) -> Mat4 {
        let proj = self.projection.build_projection_matrix(width, height);
        let view = self.view_matrix();
        proj * view
    }

    /// Set custom up vector (chainable)
    pub fn with_up(mut self, up: Vec3) -> Self {
        self.up = up;
        self
    }

    /// Set custom name (chainable)
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.into();
        self
    }

    /// Set custom near/far planes for perspective (chainable)
    pub fn with_depth_planes(mut self, near: f32, far: f32) -> Self {
        if let ProjectionType::Perspective { ref mut fov_degrees, .. } = self.projection {
            self.projection = ProjectionType::perspective_full(*fov_degrees, near, far);
        }
        self
    }
}

impl Default for Camera {
    fn default() -> Self {
        // Will be overridden by CameraPlugin with actual screen size
        Self::default_3d(800, 600)
    }
}

impl super::Resource for Camera {}
