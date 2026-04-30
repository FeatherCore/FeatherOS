//! Soccer Ball Component - Truncated Icosahedron
//!
//! A 3D soccer ball UI component based on a truncated icosahedron geometry.
//! The truncated icosahedron has 12 pentagonal faces and 20 hexagonal faces.

use fhre::{Component, Color, Vec3, Vec2, Mat4, Transform3D, View, RenderCommand, RenderComponent, AnimationReceiver, AnimationProperty, Handle, Image};
use alloc::vec::Vec;

// ============================================================
// Geometry Constants
// ============================================================

/// Number of vertices in a truncated icosahedron
const NUM_VERTICES: usize = 60;

/// Number of hexagonal faces
const NUM_HEXAGONS: usize = 20;

/// Number of pentagonal faces
const NUM_PENTAGONS: usize = 12;

/// Number of vertices per hexagon
const VERTS_PER_HEXAGON: usize = 6;

/// Number of vertices per pentagon
const VERTS_PER_PENTAGON: usize = 5;

/// Original OBJ bounding box diameter (approximate)
const ORIGINAL_DIAMETER: f32 = 300.0;

/// Scale factor to align soccer ball diameter with cube space diagonal
/// Cube space diagonal = size * sqrt(3)
const DIAGONAL_SCALE: f32 = 1.732051; // sqrt(3)

// ============================================================
// Soccer Ball Component
// ============================================================

/// A 3D soccer ball component with per-face colors and rotation
#[derive(Debug, Clone, PartialEq)]
pub struct SoccerBall {
    /// Radius of the soccer ball in pixels
    pub size: f32,
    /// Colors for each of the 12 pentagonal faces
    pub pentagon_colors: [Color; NUM_PENTAGONS],
    /// Colors for each of the 20 hexagonal faces
    pub hexagon_colors: [Color; NUM_HEXAGONS],
    /// Texture handles for pentagon faces (optional)
    pub pentagon_textures: [Option<Handle<Image>>; NUM_PENTAGONS],
    /// Texture handles for hexagon faces (optional)
    pub hexagon_textures: [Option<Handle<Image>>; NUM_HEXAGONS],
    /// Rotation angles in degrees (Euler angles)
    pub rotation: Vec3,
    /// Whether to draw wireframe overlay
    pub wireframe: bool,
    /// Wireframe line color
    pub wireframe_color: Color,
}

impl SoccerBall {
    /// Create a new soccer ball with colorful faces
    pub fn new(size: f32) -> Self {
        // 12 pentagons with bright colors
        let pentagon_colors = [
            Color::rgb(255, 100, 100),  // Red
            Color::rgb(100, 255, 100),  // Green
            Color::rgb(100, 100, 255),  // Blue
            Color::rgb(255, 255, 100),  // Yellow
            Color::rgb(255, 100, 255),  // Magenta
            Color::rgb(100, 255, 255),  // Cyan
            Color::rgb(255, 200, 100),  // Orange
            Color::rgb(200, 100, 255),  // Pink
            Color::rgb(100, 255, 200),  // Mint
            Color::rgb(255, 150, 150),  // Light Red
            Color::rgb(150, 255, 150),  // Light Green
            Color::rgb(150, 150, 255),  // Light Blue
        ];
        
        // 20 hexagons with darker tones
        let hexagon_colors = [
            Color::rgb(200, 50, 50),    // Dark Red
            Color::rgb(50, 200, 50),    // Dark Green
            Color::rgb(50, 50, 200),    // Dark Blue
            Color::rgb(200, 200, 50),   // Dark Yellow
            Color::rgb(200, 50, 200),   // Dark Magenta
            Color::rgb(50, 200, 200),   // Dark Cyan
            Color::rgb(200, 150, 50),   // Dark Orange
            Color::rgb(150, 50, 200),   // Dark Pink
            Color::rgb(50, 200, 150),   // Dark Mint
            Color::rgb(150, 100, 100),  // Brown Red
            Color::rgb(100, 150, 100),  // Brown Green
            Color::rgb(100, 100, 150),  // Brown Blue
            Color::rgb(150, 150, 50),   // Olive
            Color::rgb(150, 50, 150),   // Dark Purple
            Color::rgb(50, 150, 150),   // Dark Teal
            Color::rgb(200, 100, 50),   // Red Orange
            Color::rgb(100, 200, 50),   // Yellow Green
            Color::rgb(50, 100, 200),   // Blue Purple
            Color::rgb(200, 50, 100),   // Red Purple
            Color::rgb(100, 50, 200),   // Purple Blue
        ];
        
        Self {
            size,
            pentagon_colors,
            hexagon_colors,
            pentagon_textures: Default::default(),
            hexagon_textures: Default::default(),
            rotation: Vec3::ZERO,
            wireframe: false,
            wireframe_color: Color::WHITE,
        }
    }

    /// Set standard black/white soccer ball colors
    pub fn with_standard_colors(mut self) -> Self {
        self.pentagon_colors = [Color::BLACK; NUM_PENTAGONS];
        self.hexagon_colors = [Color::WHITE; NUM_HEXAGONS];
        self
    }

    /// Set texture handles for pentagon faces
    pub fn with_pentagon_textures_handles(mut self, textures: [Option<Handle<Image>>; NUM_PENTAGONS]) -> Self {
        self.pentagon_textures = textures;
        self
    }

    /// Set texture handles for hexagon faces
    pub fn with_hexagon_textures_handles(mut self, textures: [Option<Handle<Image>>; NUM_HEXAGONS]) -> Self {
        self.hexagon_textures = textures;
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

    /// Get the 60 vertices of the truncated icosahedron in local space
    ///
    /// Data source: `/home/uan-wsl2/codes/FeatherOS/docs/截角二十面体.obj`
    /// The soccer ball diameter is scaled to match the cube's space diagonal.
    pub fn get_vertices(&self) -> [Vec3; NUM_VERTICES] {
        // Raw vertex data from OBJ file (indices converted from 1-based to 0-based)
        let raw_vertices: [(f32, f32, f32); NUM_VERTICES] = [
            (-0.000370, 81.976440, 128.476242),
            (0.000376, 24.522030, 150.415939),
            (49.755840, 103.923996, 99.751915),
            (99.511360, 68.417496, 92.966797),
            (30.751171, 139.436325, 53.274368),
            (61.501007, 139.441406, 0.012680),
            (-30.750563, 139.436584, 53.273998),
            (-61.501610, 139.441132, 0.013036),
            (-49.756062, 103.924431, 99.751335),
            (-99.511086, 68.417061, 92.967354),
            (-49.755829, -10.985058, 143.631348),
            (-99.511322, 10.961706, 114.906982),
            (-30.751173, -68.437439, 132.653534),
            (-61.501022, -103.941841, 92.951599),
            (30.750572, -68.437866, 132.653442),
            (61.501610, -103.941399, 92.951675),
            (49.756081, -10.985776, 143.631210),
            (99.511116, 10.962403, 114.907127),
            (130.262299, 68.421860, 39.704739),
            (111.257088, 103.934052, -6.771563),
            (149.267181, 10.969494, 28.727331),
            (149.267166, -10.969493, -28.727345),
            (130.261917, -24.542421, 75.204964),
            (111.257454, -81.993843, 64.226685),
            (30.750444, -139.445938, 53.249565),
            (-30.750448, -139.446228, 53.248871),
            (61.501007, -139.441391, -0.012670),
            (30.751173, -139.436325, -53.274357),
            (111.257065, -103.934044, 6.771553),
            (130.262268, -68.421860, -39.704750),
            (130.261887, 24.542418, -75.204956),
            (111.257454, 81.993858, -64.226685),
            (99.511093, -10.962395, -114.907127),
            (49.756058, 10.985778, -143.631226),
            (99.511322, -68.417488, -92.966774),
            (49.755833, -103.924004, -99.751892),
            (-30.750574, -139.436584, -53.274025),
            (-61.501625, -139.441132, -0.013040),
            (-49.756077, -103.924416, -99.751350),
            (-99.511116, -68.417046, -92.967361),
            (-0.000389, -81.976425, -128.476257),
            (0.000365, -24.522007, -150.415924),
            (30.750565, 68.437881, -132.653458),
            (61.501625, 103.941422, -92.951683),
            (-30.751173, 68.437454, -132.653564),
            (-61.501026, 103.941841, -92.951599),
            (-49.755848, 10.985067, -143.631348),
            (-99.511345, -10.961706, -114.906982),
            (-130.261917, -68.422287, -39.705238),
            (-111.257469, -103.933601, 6.772027),
            (-149.267181, -10.970186, -28.727079),
            (-149.267166, 10.970192, 28.727062),
            (-130.262283, 24.542418, -75.204308),
            (-111.257088, 81.993858, -64.227333),
            (-30.750448, 139.446228, -53.248871),
            (30.750448, 139.445969, -53.249573),
            (-111.257462, 103.933617, -6.772024),
            (-130.261887, 68.422295, 39.705223),
            (-130.262283, -24.542408, 75.204300),
            (-111.257088, -81.993843, 64.227325),
        ];
        
        // Scale to match cube diagonal
        let target_diameter = self.size * DIAGONAL_SCALE;
        let scale = target_diameter / ORIGINAL_DIAMETER;
        
        let mut vertices = [Vec3::ZERO; NUM_VERTICES];
        for (i, (x, y, z)) in raw_vertices.iter().enumerate() {
            vertices[i] = Vec3::new(x * scale, y * scale, z * scale);
        }
        
        vertices
    }

    /// Get the 20 hexagonal faces as vertex index arrays
    ///
    /// Data source: `/home/uan-wsl2/codes/FeatherOS/docs/截角二十面体.obj`
    /// OBJ indices converted from 1-based to 0-based.
    pub fn get_hexagon_faces(&self) -> [[usize; VERTS_PER_HEXAGON]; NUM_HEXAGONS] {
        [
            [1, 16, 17, 3, 2, 0],
            [3, 18, 19, 5, 4, 2],
            [5, 55, 54, 7, 6, 4],
            [7, 56, 57, 9, 8, 6],
            [9, 11, 10, 1, 0, 8],
            [11, 58, 59, 13, 12, 10],
            [13, 25, 24, 15, 14, 12],
            [15, 23, 22, 17, 16, 14],
            [18, 20, 21, 30, 31, 19],
            [20, 22, 23, 28, 29, 21],
            [25, 37, 36, 27, 26, 24],
            [27, 35, 34, 29, 28, 26],
            [30, 32, 33, 42, 43, 31],
            [32, 34, 35, 40, 41, 33],
            [37, 49, 48, 39, 38, 36],
            [39, 47, 46, 41, 40, 38],
            [42, 44, 45, 54, 55, 43],
            [44, 46, 47, 52, 53, 45],
            [49, 59, 58, 51, 50, 48],
            [51, 57, 56, 53, 52, 50],
        ]
    }

    /// Get the 12 pentagonal faces as vertex index arrays
    ///
    /// Data source: `/home/uan-wsl2/codes/FeatherOS/docs/截角二十面体.obj`
    /// OBJ indices converted from 1-based to 0-based.
    pub fn get_pentagon_faces(&self) -> [[usize; VERTS_PER_PENTAGON]; NUM_PENTAGONS] {
        [
            [2, 4, 6, 8, 0],
            [10, 12, 14, 16, 1],
            [17, 22, 20, 18, 3],
            [15, 24, 26, 28, 23],
            [29, 34, 32, 30, 21],
            [27, 36, 38, 40, 35],
            [41, 46, 44, 42, 33],
            [39, 48, 50, 52, 47],
            [53, 56, 7, 54, 45],
            [51, 58, 11, 9, 57],
            [59, 49, 37, 25, 13],
            [5, 19, 31, 43, 55],
        ]
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

impl Default for SoccerBall {
    fn default() -> Self {
        Self::new(100.0)
    }
}

impl Component for SoccerBall {
    fn type_name() -> &'static str {
        "SoccerBall"
    }
}

impl AnimationReceiver for SoccerBall {
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

impl RenderComponent for SoccerBall {
    fn generate_render_commands(&self, transform: &Transform3D, view: &View) -> Vec<RenderCommand> {
        let mut commands = Vec::new();

        let vertices = self.get_vertices();
        let hexagons = self.get_hexagon_faces();
        let pentagons = self.get_pentagon_faces();

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

        // Collect all faces with their average depth
        let mut all_faces: Vec<(usize, bool, f32, Vec<Vec2>)> = Vec::new();

        // Process hexagonal faces
        for (face_idx, face) in hexagons.iter().enumerate() {
            let verts: Vec<Vec2> = face.iter()
                .map(|&idx| screen_vertices[idx])
                .collect();
            let avg_z: f32 = face.iter()
                .map(|&idx| view_z[idx])
                .sum::<f32>() / VERTS_PER_HEXAGON as f32;
            all_faces.push((face_idx, false, avg_z, verts));
        }

        // Process pentagonal faces
        for (face_idx, face) in pentagons.iter().enumerate() {
            let verts: Vec<Vec2> = face.iter()
                .map(|&idx| screen_vertices[idx])
                .collect();
            let avg_z: f32 = face.iter()
                .map(|&idx| view_z[idx])
                .sum::<f32>() / VERTS_PER_PENTAGON as f32;
            all_faces.push((face_idx, true, avg_z, verts));
        }

        // Sort by depth (far to near for painter's algorithm)
        // In view space, camera looks toward -Z, so larger Z = farther
        all_faces.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // Render all faces
        for (face_idx, is_pentagon, _, verts) in all_faces {
            if is_pentagon {
                let color = self.pentagon_colors[face_idx];
                
                // Note: Textures are handled by extract/queue phase, not RenderComponent
                commands.push(RenderCommand::DrawPolygon {
                    vertices: verts.clone(),
                    color,
                });

                if self.wireframe {
                    let wf_color = self.wireframe_color;
                    for i in 0..VERTS_PER_PENTAGON {
                        commands.push(RenderCommand::DrawLine {
                            start: verts[i],
                            end: verts[(i + 1) % VERTS_PER_PENTAGON],
                            color: wf_color,
                            thickness: 1.0,
                        });
                    }
                }
            } else {
                let color = self.hexagon_colors[face_idx];
                
                // Note: Textures are handled by extract/queue phase, not RenderComponent
                commands.push(RenderCommand::DrawPolygon {
                    vertices: verts.clone(),
                    color,
                });

                if self.wireframe {
                    let wf_color = self.wireframe_color;
                    for i in 0..VERTS_PER_HEXAGON {
                        commands.push(RenderCommand::DrawLine {
                            start: verts[i],
                            end: verts[(i + 1) % VERTS_PER_HEXAGON],
                            color: wf_color,
                            thickness: 1.0,
                        });
                    }
                }
            }
        }

        commands
    }
}

impl SoccerBall {
    /// Generate UV coordinates for a pentagon face
    fn generate_pentagon_uvs() -> Vec<Vec2> {
        let angle_step = core::f32::consts::TAU / 5.0;
        let start_angle = -core::f32::consts::FRAC_PI_2;
        
        (0..5)
            .map(|i| {
                let angle = start_angle + (i as f32 + 0.5) * angle_step;
                Vec2::new(
                    0.5 + 0.5 * libm::cosf(angle),
                    0.5 + 0.5 * libm::sinf(angle),
                )
            })
            .collect()
    }

    /// Generate UV coordinates for a hexagon face
    fn generate_hexagon_uvs() -> Vec<Vec2> {
        let angle_step = core::f32::consts::TAU / 6.0;
        let start_angle = 0.0;
        
        (0..6)
            .map(|i| {
                let angle = start_angle + (i as f32 + 0.5) * angle_step;
                Vec2::new(
                    0.5 + 0.5 * libm::cosf(angle),
                    0.5 + 0.5 * libm::sinf(angle),
                )
            })
            .collect()
    }
}
