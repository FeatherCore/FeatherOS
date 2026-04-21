//! Cube Component
//!
//! A simple 3D cube UI component for demonstrating 3D rendering effects.

use fhre::{Component, Color, Vec3, Vec2, Mat4, Transform3D, View, RenderCommand, RenderComponent, AnimationReceiver, AnimationProperty, TextureRegion, Handle, Image};
use alloc::vec::Vec;
use alloc::vec;

// ============================================================
// Cube Face Indices
// ============================================================

/// Index of the front face (Z+)
const FACE_FRONT: isize = 0;
/// Index of the back face (Z-)
const FACE_BACK: isize = 1;
/// Index of the top face (Y+)
const FACE_TOP: isize = 2;
/// Index of the bottom face (Y-)
const FACE_BOTTOM: isize = 3;
/// Index of the left face (X-)
const FACE_LEFT: isize = 4;
/// Index of the right face (X+)
const FACE_RIGHT: isize = 5;

/// Total number of faces on a cube
const NUM_FACES: usize = 6;

/// Number of vertices on a cube
const NUM_VERTICES: usize = 8;

// ============================================================
// Cube Face Enum
// ============================================================

/// Identifies each face of the cube
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeFace {
    Front = FACE_FRONT,
    Back = FACE_BACK,
    Top = FACE_TOP,
    Bottom = FACE_BOTTOM,
    Left = FACE_LEFT,
    Right = FACE_RIGHT,
}

// ============================================================
// Cube Component
// ============================================================

/// A 3D cube component with per-face colors and rotation
#[derive(Debug, Clone, PartialEq)]
pub struct Cube {
    /// Edge length of the cube in pixels
    pub size: f32,
    /// Color for each of the 6 faces
    pub face_colors: [Color; NUM_FACES],
    /// Texture handle for each of the 6 faces (optional)
    pub face_textures: [Option<Handle<Image>>; NUM_FACES],
    /// Rotation angles in degrees (Euler angles)
    pub rotation: Vec3,
    /// Whether to draw wireframe overlay
    pub wireframe: bool,
    /// Wireframe line color
    pub wireframe_color: Color,
}

impl Cube {
    /// Create a new cube with the given edge size
    pub fn new(size: f32) -> Self {
        Self {
            size,
            face_colors: [
                Color::rgb(255, 100, 100),
                Color::rgb(100, 255, 100),
                Color::rgb(100, 100, 255),
                Color::rgb(255, 255, 100),
                Color::rgb(255, 100, 255),
                Color::rgb(100, 255, 255),
            ],
            face_textures: Default::default(),
            rotation: Vec3::ZERO,
            wireframe: false,
            wireframe_color: Color::WHITE,
        }
    }

    /// Set all faces to a single color
    pub fn with_color(mut self, color: Color) -> Self {
        self.face_colors = [color; NUM_FACES];
        self
    }

    /// Set individual face colors
    pub fn with_face_colors(mut self, colors: [Color; NUM_FACES]) -> Self {
        self.face_colors = colors;
        self
    }

    /// Set individual face textures
    pub fn with_face_textures_handles(mut self, textures: [Handle<Image>; NUM_FACES]) -> Self {
        self.face_textures = textures.map(Some);
        self
    }

    /// Set initial rotation angles
    pub fn with_rotation(mut self, rotation: Vec3) -> Self {
        self.rotation = rotation;
        self
    }

    /// Enable wireframe rendering
    pub fn with_wireframe(mut self, enabled: bool, color: Color) -> Self {
        self.wireframe = enabled;
        self.wireframe_color = color;
        self
    }

    /// Get the 8 vertices of the cube in local space
    pub fn get_vertices(&self) -> [Vec3; NUM_VERTICES] {
        let s = self.size * 0.5;
        [
            Vec3::new(-s, -s, -s),
            Vec3::new( s, -s, -s),
            Vec3::new( s,  s, -s),
            Vec3::new(-s,  s, -s),
            Vec3::new(-s, -s,  s),
            Vec3::new( s, -s,  s),
            Vec3::new( s,  s,  s),
            Vec3::new(-s,  s,  s),
        ]
    }

    /// Get the 6 faces as vertex index quads
    /// Each face is defined by 4 vertex indices in counter-clockwise order
    pub fn get_faces(&self) -> [[usize; 4]; NUM_FACES] {
        [
            [4, 5, 6, 7],  // Front (Z+)
            [1, 0, 3, 2],  // Back (Z-)
            [3, 2, 6, 7],  // Top (Y+)
            [0, 1, 5, 4],  // Bottom (Y-)
            [0, 4, 7, 3],  // Left (X-)
            [1, 2, 6, 5],  // Right (X+)
        ]
    }

    /// Get the color for a specific face
    pub fn get_face_color(&self, face: CubeFace) -> Color {
        self.face_colors[face as usize]
    }

    /// Add delta to current rotation
    pub fn rotate(&mut self, delta: Vec3) {
        self.rotation.x += delta.x;
        self.rotation.y += delta.y;
        self.rotation.z += delta.z;
    }

    /// Set rotation directly
    pub fn set_rotation(&mut self, rotation: Vec3) {
        self.rotation = rotation;
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new(100.0)
    }
}

impl Component for Cube {
    fn type_name() -> &'static str {
        "Cube"
    }
}

impl AnimationReceiver for Cube {
    fn apply_animation(&mut self, property: AnimationProperty, value: f32) {
        match property {
            AnimationProperty::RotationX => { self.rotation.x = value; }
            AnimationProperty::RotationY => { self.rotation.y = value; }
            AnimationProperty::RotationZ => { self.rotation.z = value; }
            AnimationProperty::ScaleX | AnimationProperty::ScaleY | AnimationProperty::ScaleZ => {
                self.size = value;
            }
            _ => {}
        }
    }
}

impl RenderComponent for Cube {
    fn generate_render_commands(&self, transform: &Transform3D, view: &View) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        let vertices = self.get_vertices();
        let faces = self.get_faces();

        // Build rotation matrix (Z * Y * X order)
        let rot_x = Mat4::from_rotation_x(self.rotation.x.to_radians());
        let rot_y = Mat4::from_rotation_y(self.rotation.y.to_radians());
        let rot_z = Mat4::from_rotation_z(self.rotation.z.to_radians());
        let rotation = rot_z.mul(&rot_y).mul(&rot_x);

        // Transform vertices to world space
        let mut world_vertices: Vec<Vec3> = Vec::with_capacity(NUM_VERTICES);
        for v in &vertices {
            let rotated = rotation.mul_vec3(*v);
            let world_pos = rotated + transform.position;
            world_vertices.push(world_pos);
        }

        // Project to screen space
        let mut screen_vertices: Vec<Vec2> = Vec::with_capacity(NUM_VERTICES);
        let mut view_z: Vec<f32> = Vec::with_capacity(NUM_VERTICES);
        
        for world_pos in &world_vertices {
            let view_pos = view.view.mul_vec3(*world_pos);
            view_z.push(view_pos.z);
            
            if let Some((x, y)) = view.world_to_screen(*world_pos) {
                screen_vertices.push(Vec2::new(x, y));
            } else {
                screen_vertices.clear();
                break;
            }
        }

        if screen_vertices.len() != NUM_VERTICES {
            return commands;
        }

        // Collect faces with their average depth for painter's algorithm
        let mut visible_faces: Vec<(usize, f32, [Vec2; 4])> = Vec::new();

        for (face_idx, face) in faces.iter().enumerate() {
            let v0 = screen_vertices[face[0]];
            let v1 = screen_vertices[face[1]];
            let v2 = screen_vertices[face[2]];
            let v3 = screen_vertices[face[3]];
            
            let avg_view_z = (view_z[face[0]] 
                            + view_z[face[1]] 
                            + view_z[face[2]] 
                            + view_z[face[3]]) / 4.0;
            
            visible_faces.push((face_idx, avg_view_z, [v0, v1, v2, v3]));
        }

        // Sort by depth (far to near for painter's algorithm)
        visible_faces.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Render each face
        for (face_idx, _, [v0, v1, v2, v3]) in visible_faces {
            let color = self.face_colors[face_idx];

            // Draw filled polygon
            commands.push(RenderCommand::DrawPolygon {
                vertices: vec![v0, v1, v2, v3],
                color,
            });

            // Draw wireframe if enabled
            if self.wireframe {
                let wf_color = self.wireframe_color;

                commands.push(RenderCommand::DrawLine {
                    start: v0, end: v1, color: wf_color, thickness: 1.0,
                });
                commands.push(RenderCommand::DrawLine {
                    start: v1, end: v2, color: wf_color, thickness: 1.0,
                });
                commands.push(RenderCommand::DrawLine {
                    start: v2, end: v3, color: wf_color, thickness: 1.0,
                });
                commands.push(RenderCommand::DrawLine {
                    start: v3, end: v0, color: wf_color, thickness: 1.0,
                });
            }
        }

        commands
    }
}
