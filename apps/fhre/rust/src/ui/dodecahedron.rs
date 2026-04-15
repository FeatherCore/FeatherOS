//! Dodecahedron Component
//!
//! 3D 正十二面体 UI 组件，用于展示 3D 球体效果。
//! 正十二面体有12个五边形面，20个顶点，30条边。
//! 基于 HoneyGUI 的 gui_soccer 组件设计。

use crate::Component;
use crate::math::{Color, Vec3};

/// 正十二面体组件
#[derive(Debug, Clone, PartialEq)]
pub struct Dodecahedron {
    /// 正十二面体尺寸（边长）
    pub size: f32,
    /// 每个面的颜色（12个五边形面）
    pub face_colors: [Color; 12],
    /// 旋转角度（欧拉角，度）
    pub rotation: Vec3,
    /// 线框模式
    pub wireframe: bool,
    /// 线框颜色
    pub wireframe_color: Color,
}

/// 黄金比例
const PHI: f32 = 1.618033988749895;
/// 黄金比例的倒数
const INV_PHI: f32 = 0.618033988749895;

impl Dodecahedron {
    /// 创建新正十二面体
    pub fn new(size: f32) -> Self {
        Self {
            size,
            face_colors: [
                Color::rgb(255, 100, 100), Color::rgb(100, 255, 100), 
                Color::rgb(100, 100, 255), Color::rgb(255, 255, 100), 
                Color::rgb(255, 100, 255), Color::rgb(100, 255, 255),
                Color::rgb(255, 150, 100), Color::rgb(150, 255, 100), 
                Color::rgb(100, 150, 255), Color::rgb(255, 200, 100),
                Color::rgb(200, 100, 255), Color::rgb(100, 255, 200),
            ],
            rotation: Vec3::ZERO,
            wireframe: false,
            wireframe_color: Color::WHITE,
        }
    }

    /// 设置所有面为同一颜色
    pub fn with_color(mut self, color: Color) -> Self {
        self.face_colors = [color; 12];
        self
    }

    /// 设置各个面的颜色
    pub fn with_face_colors(mut self, colors: [Color; 12]) -> Self {
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

    /// 获取正十二面体的 20 个顶点（本地坐标）
    /// 
    /// 顶点基于黄金比例 PHI = (1 + sqrt(5)) / 2
    /// 20个顶点分为三组：
    /// - (±1, ±1, ±1) - 立方体的8个顶点
    /// - (0, ±PHI, ±1/PHI) - 4个顶点
    /// - (±1/PHI, 0, ±PHI) - 4个顶点  
    /// - (±PHI, ±1/PHI, 0) - 4个顶点
    pub fn get_vertices(&self) -> [Vec3; 20] {
        let s = self.size * 0.5;
        let phi = PHI * s;
        let inv_phi = INV_PHI * s;
        let one = s;
        
        [
            // (±1, ±1, ±1) - 立方体的8个顶点
            Vec3::new( one,  one,  one),  // 0
            Vec3::new( one,  one, -one),  // 1
            Vec3::new( one, -one,  one),  // 2
            Vec3::new( one, -one, -one),  // 3
            Vec3::new(-one,  one,  one),  // 4
            Vec3::new(-one,  one, -one),  // 5
            Vec3::new(-one, -one,  one),  // 6
            Vec3::new(-one, -one, -one),  // 7
            
            // (0, ±PHI, ±1/PHI) - 4个顶点
            Vec3::new(0.0,  phi,  inv_phi),  // 8
            Vec3::new(0.0,  phi, -inv_phi),  // 9
            Vec3::new(0.0, -phi,  inv_phi),  // 10
            Vec3::new(0.0, -phi, -inv_phi),  // 11
            
            // (±1/PHI, 0, ±PHI) - 4个顶点
            Vec3::new( inv_phi, 0.0,  phi),  // 12
            Vec3::new( inv_phi, 0.0, -phi),  // 13
            Vec3::new(-inv_phi, 0.0,  phi),  // 14
            Vec3::new(-inv_phi, 0.0, -phi),  // 15
            
            // (±PHI, ±1/PHI, 0) - 4个顶点
            Vec3::new( phi,  inv_phi, 0.0),  // 16
            Vec3::new( phi, -inv_phi, 0.0),  // 17
            Vec3::new(-phi,  inv_phi, 0.0),  // 18
            Vec3::new(-phi, -inv_phi, 0.0),  // 19
        ]
    }

    /// 获取正十二面体的 12 个五边形面（每个面由 5 个顶点索引组成）
    /// 
    /// 面的顶点顺序为逆时针（从外部看）
    pub fn get_faces(&self) -> [[usize; 5]; 12] {
        [
            // 围绕 +Z 轴的5个面
            [0, 4, 14, 12, 2],   // 面 0: 底部
            [0, 2, 17, 16, 1],   // 面 1
            [0, 1, 9, 8, 4],     // 面 2
            [1, 16, 13, 15, 5],  // 面 3
            [4, 8, 9, 5, 18],    // 面 4
            
            // 中间层
            [2, 12, 10, 11, 3],  // 面 5
            [2, 3, 17, 16, 0],   // 面 6 (修正)
            [3, 11, 7, 19, 6],   // 面 7
            [12, 14, 6, 10, 2],  // 面 8 (修正)
            
            // 围绕 -Z 轴的5个面
            [5, 15, 7, 19, 18],  // 面 9
            [6, 19, 7, 11, 10],  // 面 10: 顶部
            [13, 16, 17, 3, 15], // 面 11
        ]
    }

    /// 将五边形面分解为三角形（用于渲染）
    /// 每个五边形分解为3个三角形：
    /// - (v0, v1, v2)
    /// - (v0, v2, v3)
    /// - (v0, v3, v4)
    pub fn get_triangulated_faces(&self) -> [[usize; 3]; 36] {
        let faces = self.get_faces();
        let mut triangles = [[0; 3]; 36];
        
        for (i, face) in faces.iter().enumerate() {
            let base = i * 3;
            // 五边形扇形分解：以v0为顶点，连接到其他顶点
            triangles[base] = [face[0], face[1], face[2]];
            triangles[base + 1] = [face[0], face[2], face[3]];
            triangles[base + 2] = [face[0], face[3], face[4]];
        }
        
        triangles
    }

    /// 获取每个面对应的三角形索引范围
    /// 返回 [(triangle_start, triangle_count); 12]
    pub fn get_face_triangle_ranges(&self) -> [(usize, usize); 12] {
        [
            (0, 3), (3, 3), (6, 3), (9, 3),
            (12, 3), (15, 3), (18, 3), (21, 3),
            (24, 3), (27, 3), (30, 3), (33, 3),
        ]
    }

    /// 旋转正十二面体
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

impl Default for Dodecahedron {
    fn default() -> Self {
        Self::new(100.0)
    }
}

impl Component for Dodecahedron {
    fn type_name() -> &'static str {
        "Dodecahedron"
    }
}

/// 旋转的正十二面体组件（自动旋转）
#[derive(Debug, Clone, PartialEq)]
pub struct RotatingDodecahedron {
    pub dodecahedron: Dodecahedron,
    pub rotation_speed: Vec3,
}

impl RotatingDodecahedron {
    pub fn new(size: f32) -> Self {
        Self {
            dodecahedron: Dodecahedron::new(size),
            rotation_speed: Vec3::new(0.0, 1.0, 0.0),
        }
    }

    pub fn with_rotation_speed(mut self, speed: Vec3) -> Self {
        self.rotation_speed = speed;
        self
    }

    pub fn update(&mut self, delta_time: f32) {
        let delta = self.rotation_speed * delta_time * 60.0;
        self.dodecahedron.rotate(delta);
    }
}

impl Default for RotatingDodecahedron {
    fn default() -> Self {
        Self::new(100.0)
    }
}

impl Component for RotatingDodecahedron {
    fn type_name() -> &'static str {
        "RotatingDodecahedron"
    }
}
