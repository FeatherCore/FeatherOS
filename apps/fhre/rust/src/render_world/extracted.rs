//! Extracted Components
//!
//! These are the components that get extracted from Main World to Render World.
//! They are simplified versions of Main World components, optimized for rendering.

use crate::{Component, math::{Vec3, Mat4}, node::Transform3D};
use alloc::vec::Vec;

/// Extracted transform - simplified transform for rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtractedTransform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Component for ExtractedTransform {
    fn type_name() -> &'static str { "ExtractedTransform" }
}

impl From<&Transform3D> for ExtractedTransform {
    fn from(transform: &Transform3D) -> Self {
        Self {
            position: transform.position,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }
}

/// Extracted mesh - mesh data for rendering
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedMesh {
    /// Vertices in local space
    pub vertices: Vec<Vec3>,
    /// Face indices (each face is a list of vertex indices)
    pub faces: Vec<Vec<usize>>,
    /// Face colors
    pub face_colors: Vec<crate::math::Color>,
    /// World position
    pub position: Vec3,
    /// Rotation in degrees (Euler angles)
    pub rotation: Vec3,
    /// Wireframe mode
    pub wireframe: bool,
    pub wireframe_color: crate::math::Color,
}

impl Component for ExtractedMesh {
    fn type_name() -> &'static str { "ExtractedMesh" }
}

/// Extracted UI element - 2D UI for rendering
#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedUI {
    pub position: crate::math::Vec2,
    pub width: f32,
    pub height: f32,
    pub color: crate::math::Color,
}

impl Component for ExtractedUI {
    fn type_name() -> &'static str { "ExtractedUI" }
}

/// Extracted view - camera/view data for rendering
#[derive(Debug, Clone)]
pub struct ExtractedView {
    pub projection: Mat4,
    pub view: Mat4,
    pub view_projection: Mat4,
    pub camera_position: Vec3,
    pub viewport: crate::math::Rect,
}

impl Component for ExtractedView {
    fn type_name() -> &'static str { "ExtractedView" }
}
