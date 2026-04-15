//! Soccer Ball Component - Truncated Icosahedron
//!
//! 3D 足球 UI 组件，基于截角二十面体（Truncated Icosahedron）。
//! 截角二十面体有32个面：12个正五边形 + 20个正六边形
//! 这是经典足球的几何形状。

use crate::Component;
use crate::math::{Color, Vec3};

/// 足球组件（截角二十面体）
#[derive(Debug, Clone, PartialEq)]
pub struct SoccerBall {
    /// 足球尺寸（半径）
    pub size: f32,
    /// 12个五边形面的颜色
    pub pentagon_colors: [Color; 12],
    /// 20个六边形面的颜色
    pub hexagon_colors: [Color; 20],
    /// 旋转角度（欧拉角，度）
    pub rotation: Vec3,
    /// 线框模式
    pub wireframe: bool,
    /// 线框颜色
    pub wireframe_color: Color,
}

impl SoccerBall {
    /// 创建新足球
    pub fn new(size: f32) -> Self {
        // 彩色配色以便区分各个面
        let pentagon_colors = [
            Color::rgb(255, 100, 100), // 红
            Color::rgb(100, 255, 100), // 绿
            Color::rgb(100, 100, 255), // 蓝
            Color::rgb(255, 255, 100), // 黄
            Color::rgb(255, 100, 255), // 紫
            Color::rgb(100, 255, 255), // 青
            Color::rgb(255, 150, 100), // 橙
            Color::rgb(150, 255, 100), // 浅绿
            Color::rgb(100, 150, 255), // 浅蓝
            Color::rgb(255, 200, 100), // 金黄
            Color::rgb(200, 100, 255), // 紫红
            Color::rgb(100, 255, 200), // 青绿
        ];
        
        let hexagon_colors = [
            Color::rgb(220, 220, 220), Color::rgb(200, 200, 200),
            Color::rgb(180, 180, 180), Color::rgb(160, 160, 160),
            Color::rgb(140, 140, 140), Color::rgb(120, 120, 120),
            Color::rgb(220, 200, 200), Color::rgb(200, 220, 200),
            Color::rgb(200, 200, 220), Color::rgb(220, 220, 180),
            Color::rgb(220, 180, 220), Color::rgb(180, 220, 220),
            Color::rgb(240, 220, 200), Color::rgb(220, 240, 200),
            Color::rgb(200, 220, 240), Color::rgb(240, 200, 220),
            Color::rgb(220, 200, 240), Color::rgb(200, 240, 220),
            Color::rgb(240, 240, 200), Color::rgb(200, 240, 240),
        ];
        
        Self {
            size,
            pentagon_colors,
            hexagon_colors,
            rotation: Vec3::ZERO,
            wireframe: true,
            wireframe_color: Color::BLACK,
        }
    }

    /// 创建彩色足球（每个面不同颜色）
    pub fn with_colors(mut self, pentagon_colors: [Color; 12], hexagon_colors: [Color; 20]) -> Self {
        self.pentagon_colors = pentagon_colors;
        self.hexagon_colors = hexagon_colors;
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

    /// 获取截角二十面体的 60 个顶点（本地坐标）
    /// 
    /// 使用预计算的归一化顶点坐标（单位球面上）
    pub fn get_vertices(&self) -> [Vec3; 60] {
        let scale = self.size * 0.5;
        
        // 预计算的归一化顶点（单位球面上）
        // 基于标准截角二十面体，所有顶点到原点距离为 1.0
        let unit_vertices: [(f32, f32, f32); 60] = [
            (0.0, 0.178411, 0.983969), (0.0, 0.178411, -0.983969),
            (0.0, -0.178411, 0.983969), (0.0, -0.178411, -0.983969),
            (0.178411, 0.983969, 0.0), (0.178411, -0.983969, 0.0),
            (-0.178411, 0.983969, 0.0), (-0.178411, -0.983969, 0.0),
            (0.983969, 0.0, 0.178411), (0.983969, 0.0, -0.178411),
            (-0.983969, 0.0, 0.178411), (-0.983969, 0.0, -0.178411),
            
            (0.356822, 0.715591, 0.603414), (0.356822, 0.715591, -0.603414),
            (0.356822, -0.715591, 0.603414), (0.356822, -0.715591, -0.603414),
            (-0.356822, 0.715591, 0.603414), (-0.356822, 0.715591, -0.603414),
            (-0.356822, -0.715591, 0.603414), (-0.356822, -0.715591, -0.603414),
            (0.715591, 0.603414, 0.356822), (0.715591, 0.603414, -0.356822),
            (0.715591, -0.603414, 0.356822), (0.715591, -0.603414, -0.356822),
            (-0.715591, 0.603414, 0.356822), (-0.715591, 0.603414, -0.356822),
            (-0.715591, -0.603414, 0.356822), (-0.715591, -0.603414, -0.356822),
            (0.603414, 0.356822, 0.715591), (0.603414, 0.356822, -0.715591),
            (0.603414, -0.356822, 0.715591), (0.603414, -0.356822, -0.715591),
            (-0.603414, 0.356822, 0.715591), (-0.603414, 0.356822, -0.715591),
            (-0.603414, -0.356822, 0.715591), (-0.603414, -0.356822, -0.715591),
            
            (0.356822, 0.268328, 0.894427), (0.356822, 0.268328, -0.894427),
            (0.356822, -0.268328, 0.894427), (0.356822, -0.268328, -0.894427),
            (-0.356822, 0.268328, 0.894427), (-0.356822, 0.268328, -0.894427),
            (-0.356822, -0.268328, 0.894427), (-0.356822, -0.268328, -0.894427),
            (0.268328, 0.894427, 0.356822), (0.268328, 0.894427, -0.356822),
            (0.268328, -0.894427, 0.356822), (0.268328, -0.894427, -0.356822),
            (-0.268328, 0.894427, 0.356822), (-0.268328, 0.894427, -0.356822),
            (-0.268328, -0.894427, 0.356822), (-0.268328, -0.894427, -0.356822),
            (0.894427, 0.356822, 0.268328), (0.894427, 0.356822, -0.268328),
            (0.894427, -0.356822, 0.268328), (0.894427, -0.356822, -0.268328),
            (-0.894427, 0.356822, 0.268328), (-0.894427, 0.356822, -0.268328),
            (-0.894427, -0.356822, 0.268328), (-0.894427, -0.356822, -0.268328),
        ];
        
        let mut vertices = [Vec3::ZERO; 60];
        for (i, (x, y, z)) in unit_vertices.iter().enumerate() {
            vertices[i] = Vec3::new(*x * scale, *y * scale, *z * scale);
        }
        
        vertices
    }

    /// 获取12个五边形面（每个面由5个顶点索引组成）
    /// 
    /// 基于标准截角二十面体的五边形面定义
    pub fn get_pentagon_faces(&self) -> [[usize; 5]; 12] {
        [
            [0, 28, 36, 12, 4],       // 五边形 0
            [1, 5, 13, 37, 29],       // 五边形 1
            [2, 6, 16, 40, 32],       // 五边形 2
            [3, 33, 41, 17, 7],       // 五边形 3
            [8, 9, 21, 45, 20],       // 五边形 4
            [10, 11, 25, 49, 24],     // 五边形 5
            [14, 15, 23, 47, 22],     // 五边形 6
            [18, 19, 27, 51, 26],     // 五边形 7
            [30, 31, 35, 43, 42],     // 五边形 8
            [34, 46, 44, 52, 54],     // 五边形 9
            [38, 39, 55, 53, 50],     // 五边形 10
            [48, 56, 58, 59, 57],     // 五边形 11
        ]
    }

    /// 获取20个六边形面（每个面由6个顶点索引组成）
    pub fn get_hexagon_faces(&self) -> [[usize; 6]; 20] {
        [
            [0, 4, 8, 20, 44, 52],       // 六边形 0
            [0, 52, 28, 32, 40, 36],     // 六边形 1
            [2, 32, 28, 36, 16, 6],      // 六边形 2
            [2, 6, 10, 24, 48, 40],      // 六边形 3
            [4, 12, 20, 8, 9, 21],       // 六边形 4
            [4, 21, 13, 5, 1, 0],        // 六边形 5
            [6, 16, 12, 4, 0, 2],        // 六边形 6
            [8, 9, 23, 47, 46, 22],      // 六边形 7
            [10, 11, 25, 49, 48, 24],    // 六边形 8
            [12, 16, 17, 25, 24, 10],    // 六边形 9
            [14, 18, 19, 27, 26, 22],    // 六边形 10
            [15, 14, 22, 26, 50, 51],    // 六边形 11
            [17, 16, 40, 48, 56, 57],    // 六边形 12
            [19, 18, 42, 58, 59, 27],    // 六边形 13
            [21, 20, 44, 45, 37, 13],    // 六边形 14
            [23, 22, 46, 47, 55, 39],    // 六边形 15
            [25, 17, 57, 41, 33, 49],    // 六边形 16
            [27, 59, 51, 43, 35, 19],    // 六边形 17
            [29, 37, 45, 53, 21, 5],     // 六边形 18
            [31, 15, 39, 55, 54, 34],    // 六边形 19
        ]
    }

    /// 旋转足球
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

/// 旋转的足球组件（自动旋转）
#[derive(Debug, Clone, PartialEq)]
pub struct RotatingSoccerBall {
    pub soccer_ball: SoccerBall,
    pub rotation_speed: Vec3,
}

impl RotatingSoccerBall {
    pub fn new(size: f32) -> Self {
        Self {
            soccer_ball: SoccerBall::new(size),
            rotation_speed: Vec3::new(0.0, 1.0, 0.0),
        }
    }

    pub fn with_rotation_speed(mut self, speed: Vec3) -> Self {
        self.rotation_speed = speed;
        self
    }

    pub fn update(&mut self, delta_time: f32) {
        let delta = self.rotation_speed * delta_time * 60.0;
        self.soccer_ball.rotate(delta);
    }
}

impl Default for RotatingSoccerBall {
    fn default() -> Self {
        Self::new(100.0)
    }
}

impl Component for RotatingSoccerBall {
    fn type_name() -> &'static str {
        "RotatingSoccerBall"
    }
}
