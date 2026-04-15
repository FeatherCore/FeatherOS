//! Cube Component
//!
//! 简单的 3D 立方体 UI 组件，用于展示 3D 效果。

use crate::Component;
use crate::math::{Color, Vec3};

/// 立方体面的索引
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeFace {
    Front = 0,
    Back = 1,
    Top = 2,
    Bottom = 3,
    Left = 4,
    Right = 5,
}

/// 立方体组件
#[derive(Debug, Clone, PartialEq)]
pub struct Cube {
    /// 立方体尺寸（边长）
    pub size: f32,
    /// 每个面的颜色
    pub face_colors: [Color; 6],
    /// 旋转角度（欧拉角，度）
    pub rotation: Vec3,
    /// 线框模式
    pub wireframe: bool,
    /// 线框颜色
    pub wireframe_color: Color,
}

impl Cube {
    /// 创建新立方体
    pub fn new(size: f32) -> Self {
        Self {
            size,
            face_colors: [
                Color::rgb(255, 100, 100), // Front - red
                Color::rgb(100, 255, 100), // Back - green
                Color::rgb(100, 100, 255), // Top - blue
                Color::rgb(255, 255, 100), // Bottom - yellow
                Color::rgb(255, 100, 255), // Left - magenta
                Color::rgb(100, 255, 255), // Right - cyan
            ],
            rotation: Vec3::ZERO,
            wireframe: false,
            wireframe_color: Color::WHITE,
        }
    }

    /// 设置所有面为同一颜色
    pub fn with_color(mut self, color: Color) -> Self {
        self.face_colors = [color; 6];
        self
    }

    /// 设置各个面的颜色
    pub fn with_face_colors(mut self, colors: [Color; 6]) -> Self {
        self.face_colors = colors;
        self
    }

    /// 设置旋转角度
    pub fn with_rotation(mut self, rotation: Vec3) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置线框模式
    pub fn with_wireframe(mut self, enabled: bool, color: Color) -> Self {
        self.wireframe = enabled;
        self.wireframe_color = color;
        self
    }

    /// 获取立方体的 8 个顶点（本地坐标）
    /// 
    /// 顶点顺序：
    /// 0: -x, -y, -z (左下后)
    /// 1: +x, -y, -z (右下后)
    /// 2: +x, +y, -z (右上后)
    /// 3: -x, +y, -z (左上后)
    /// 4: -x, -y, +z (左下前)
    /// 5: +x, -y, +z (右下前)
    /// 6: +x, +y, +z (右上前)
    /// 7: -x, +y, +z (左上前)
    pub fn get_vertices(&self) -> [Vec3; 8] {
        let s = self.size * 0.5;
        [
            Vec3::new(-s, -s, -s), // 0
            Vec3::new(s, -s, -s),  // 1
            Vec3::new(s, s, -s),   // 2
            Vec3::new(-s, s, -s),  // 3
            Vec3::new(-s, -s, s),  // 4
            Vec3::new(s, -s, s),   // 5
            Vec3::new(s, s, s),    // 6
            Vec3::new(-s, s, s),   // 7
        ]
    }

    /// 获取立方体的 6 个面（每个面由 4 个顶点索引组成）
    /// 
    /// 面的顶点顺序为逆时针（从外部看）
    pub fn get_faces(&self) -> [[usize; 4]; 6] {
        [
            [4, 5, 6, 7], // Front (Z+)
            [1, 0, 3, 2], // Back (Z-)
            [3, 2, 6, 7], // Top (Y+)
            [0, 1, 5, 4], // Bottom (Y-)
            [0, 4, 7, 3], // Left (X-)
            [1, 2, 6, 5], // Right (X+)
        ]
    }

    /// 获取面的颜色
    pub fn get_face_color(&self, face: CubeFace) -> Color {
        self.face_colors[face as usize]
    }

    /// 旋转立方体
    pub fn rotate(&mut self, delta: Vec3) {
        self.rotation.x += delta.x;
        self.rotation.y += delta.y;
        self.rotation.z += delta.z;
    }

    /// 设置旋转
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

/// 旋转的立方体组件（自动旋转）
#[derive(Debug, Clone, PartialEq)]
pub struct RotatingCube {
    pub cube: Cube,
    pub rotation_speed: Vec3,
}

impl RotatingCube {
    pub fn new(size: f32) -> Self {
        Self {
            cube: Cube::new(size),
            rotation_speed: Vec3::new(0.0, 30.0, 0.0),
        }
    }

    pub fn with_speed(mut self, speed: Vec3) -> Self {
        self.rotation_speed = speed;
        self
    }

    /// 更新旋转
    pub fn update(&mut self, delta_time: f32) {
        self.cube.rotation.x += self.rotation_speed.x * delta_time;
        self.cube.rotation.y += self.rotation_speed.y * delta_time;
        self.cube.rotation.z += self.rotation_speed.z * delta_time;
    }
}

impl Default for RotatingCube {
    fn default() -> Self {
        Self::new(100.0)
    }
}

impl Component for RotatingCube {
    fn type_name() -> &'static str {
        "RotatingCube"
    }
}
