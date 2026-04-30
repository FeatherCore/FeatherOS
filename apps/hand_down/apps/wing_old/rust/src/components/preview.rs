//! Preview effects used by app switcher cards.

extern crate alloc;

use alloc::vec::Vec;

use fhre::{Color, Component, Mat4, Vec2, Vec3};

const NUM_VERTICES: usize = 60;
const NUM_HEXAGONS: usize = 20;
const NUM_PENTAGONS: usize = 12;
const VERTS_PER_HEXAGON: usize = 6;
const VERTS_PER_PENTAGON: usize = 5;
const ORIGINAL_DIAMETER: f32 = 300.0;
const DIAGONAL_SCALE: f32 = 1.732051;

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewSoccerBall {
    pub size_ratio: f32,
    pub rotation: Vec3,
    pub pentagon_colors: [Color; NUM_PENTAGONS],
    pub hexagon_colors: [Color; NUM_HEXAGONS],
    pub wireframe: bool,
    pub wireframe_color: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectedPreviewFace {
    pub face_index: usize,
    pub vertices: Vec<Vec2>,
    pub center: Vec2,
    pub depth: f32,
    pub color: Color,
}

const SURFACE_FACE_INDICES: [usize; 8] = [0, 1, 4, 2, 5, 8, 9, 11];

impl PreviewSoccerBall {
    pub fn new(size_ratio: f32) -> Self {
        Self {
            size_ratio,
            rotation: Vec3::ZERO,
            pentagon_colors: [Color::rgb(18, 22, 30); NUM_PENTAGONS],
            hexagon_colors: [Color::rgb(244, 247, 252); NUM_HEXAGONS],
            wireframe: true,
            wireframe_color: Color::new(80, 92, 112, 180),
        }
    }

    pub fn for_stack_index(stack_index: u8) -> Self {
        Self::new(0.48).with_rotation(Vec3::new(
            28.0 + stack_index as f32 * 8.0,
            stack_index as f32 * 24.0,
            12.0,
        ))
    }

    pub fn for_app_switcher() -> Self {
        Self::new(1.0).with_rotation(Vec3::new(28.0, 0.0, 12.0))
    }

    pub fn with_rotation(mut self, rotation: Vec3) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn rotate(&mut self, delta: Vec3) {
        self.rotation.x = wrap_degrees(self.rotation.x + delta.x);
        self.rotation.y = wrap_degrees(self.rotation.y + delta.y);
        self.rotation.z = wrap_degrees(self.rotation.z + delta.z);
    }

    pub fn projected_faces(&self, center: Vec2, base_size: f32, alpha: u8) -> Vec<ProjectedPreviewFace> {
        let size = base_size * self.size_ratio;
        if size <= 1.0 || alpha == 0 {
            return Vec::new();
        }

        let vertices = self.vertices(size);
        let rotation = rotation_matrix(self.rotation);
        let distance = size * 5.0;

        let mut projected: Vec<Vec2> = Vec::with_capacity(NUM_VERTICES);
        let mut depth: Vec<f32> = Vec::with_capacity(NUM_VERTICES);

        for vertex in vertices {
            let rotated = rotation.mul_vec3(vertex);
            let perspective = distance / (distance - rotated.z).max(1.0);
            projected.push(Vec2::new(
                center.x + rotated.x * perspective,
                center.y - rotated.y * perspective,
            ));
            depth.push(rotated.z);
        }

        let mut faces: Vec<(f32, usize, Vec<Vec2>, Color)> = Vec::new();

        for (face_idx, face) in self.hexagon_faces().iter().enumerate() {
            let verts = face.iter().map(|&idx| projected[idx]).collect();
            let avg_z = average_depth(face, &depth);
            let shade = depth_shade(avg_z, size);
            faces.push((avg_z, face_idx, verts, shade_color(self.hexagon_colors[face_idx], shade, alpha)));
        }

        for (face_idx, face) in self.pentagon_faces().iter().enumerate() {
            let verts = face.iter().map(|&idx| projected[idx]).collect();
            let avg_z = average_depth(face, &depth);
            let shade = depth_shade(avg_z, size);
            faces.push((
                avg_z,
                NUM_HEXAGONS + face_idx,
                verts,
                shade_color(self.pentagon_colors[face_idx], shade, alpha),
            ));
        }

        faces.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));

        faces
            .into_iter()
            .map(|(depth, face_index, vertices, color)| ProjectedPreviewFace {
                face_index,
                center: polygon_center(&vertices),
                vertices,
                depth,
                color,
            })
            .collect()
    }

    fn vertices(&self, size: f32) -> [Vec3; NUM_VERTICES] {
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

        let target_diameter = size * DIAGONAL_SCALE;
        let scale = target_diameter / ORIGINAL_DIAMETER;
        let mut vertices = [Vec3::ZERO; NUM_VERTICES];

        for (i, (x, y, z)) in raw_vertices.iter().enumerate() {
            vertices[i] = Vec3::new(x * scale, y * scale, z * scale);
        }

        vertices
    }

    fn hexagon_faces(&self) -> [[usize; VERTS_PER_HEXAGON]; NUM_HEXAGONS] {
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

    fn pentagon_faces(&self) -> [[usize; VERTS_PER_PENTAGON]; NUM_PENTAGONS] {
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
}

impl Default for PreviewSoccerBall {
    fn default() -> Self {
        Self::new(0.48)
    }
}

impl Component for PreviewSoccerBall {
    fn type_name() -> &'static str {
        "PreviewSoccerBall"
    }
}

pub fn surface_index_for_face(face_index: usize, surface_count: usize) -> Option<usize> {
    SURFACE_FACE_INDICES
        .iter()
        .position(|&mapped_face| mapped_face == face_index)
        .filter(|&surface_index| surface_index < surface_count)
}

pub fn app_switcher_ball_center(screen_size: Vec2, progress: f32) -> Vec2 {
    let progress = progress.clamp(0.0, 1.0);
    Vec2::new(
        screen_size.x * 0.5,
        screen_size.y * 0.5 + (1.0 - progress) * screen_size.y * 0.28,
    )
}

pub fn app_switcher_ball_size(screen_size: Vec2, progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    let open_scale = 0.82 + progress * 0.18;
    screen_size.x.min(screen_size.y) * 0.42 * open_scale
}

pub fn point_in_polygon(point: Vec2, vertices: &[Vec2]) -> bool {
    if vertices.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = vertices.len() - 1;

    for i in 0..vertices.len() {
        let vi = vertices[i];
        let vj = vertices[j];
        let intersects = ((vi.y > point.y) != (vj.y > point.y))
            && (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x);

        if intersects {
            inside = !inside;
        }
        j = i;
    }

    inside
}

fn rotation_matrix(rotation: Vec3) -> Mat4 {
    let rot_x = Mat4::from_rotation_x(rotation.x.to_radians());
    let rot_y = Mat4::from_rotation_y(rotation.y.to_radians());
    let rot_z = Mat4::from_rotation_z(rotation.z.to_radians());
    rot_z.mul(&rot_y).mul(&rot_x)
}

fn average_depth<const N: usize>(face: &[usize; N], depth: &[f32]) -> f32 {
    face.iter().map(|&idx| depth[idx]).sum::<f32>() / N as f32
}

fn polygon_center(vertices: &[Vec2]) -> Vec2 {
    if vertices.is_empty() {
        return Vec2::ZERO;
    }

    let mut center = Vec2::ZERO;
    for vertex in vertices {
        center += *vertex;
    }
    center * (1.0 / vertices.len() as f32)
}

fn depth_shade(avg_z: f32, size: f32) -> f32 {
    let normalized = (avg_z / (size * 0.9)).clamp(-1.0, 1.0);
    (0.78 + normalized * 0.22).clamp(0.56, 1.05)
}

fn shade_color(color: Color, shade: f32, alpha: u8) -> Color {
    Color::new(
        (color.r as f32 * shade).clamp(0.0, 255.0) as u8,
        (color.g as f32 * shade).clamp(0.0, 255.0) as u8,
        (color.b as f32 * shade).clamp(0.0, 255.0) as u8,
        alpha,
    )
}

fn wrap_degrees(value: f32) -> f32 {
    if value >= 360.0 {
        value - 360.0
    } else if value < 0.0 {
        value + 360.0
    } else {
        value
    }
}
