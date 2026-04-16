//! Soccer Ball Component - Truncated Icosahedron
//!
//! 3D 足球 UI 组件，基于截角二十面体（Truncated Icosahedron）。
//! 使用 `/home/uan-wsl2/codes/FeatherOS/docs/截角二十面体.obj` 的精确数据。

use crate::Component;
use crate::math::{Color, Vec3};
use alloc::vec::Vec;

/// 足球组件（基于截角二十面体）
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
    /// 创建新足球（彩色版本，每个面不同颜色）
    pub fn new(size: f32) -> Self {
        // 12个五边形 - 使用不同的鲜艳颜色
        let pentagon_colors = [
            Color::rgb(255, 100, 100), // 红
            Color::rgb(100, 255, 100), // 绿
            Color::rgb(100, 100, 255), // 蓝
            Color::rgb(255, 255, 100), // 黄
            Color::rgb(255, 100, 255), // 紫
            Color::rgb(100, 255, 255), // 青
            Color::rgb(255, 200, 100), // 橙
            Color::rgb(200, 100, 255), // 粉
            Color::rgb(100, 255, 200), // 薄荷
            Color::rgb(255, 150, 150), // 浅红
            Color::rgb(150, 255, 150), // 浅绿
            Color::rgb(150, 150, 255), // 浅蓝
        ];
        
        // 20个六边形 - 使用不同的深色调
        let hexagon_colors = [
            Color::rgb(200, 50, 50),   // 深红
            Color::rgb(50, 200, 50),   // 深绿
            Color::rgb(50, 50, 200),   // 深蓝
            Color::rgb(200, 200, 50),  // 深黄
            Color::rgb(200, 50, 200),  // 深紫
            Color::rgb(50, 200, 200),  // 深青
            Color::rgb(200, 150, 50),  // 深橙
            Color::rgb(150, 50, 200),  // 深粉
            Color::rgb(50, 200, 150),  // 深薄荷
            Color::rgb(150, 100, 100), // 棕红
            Color::rgb(100, 150, 100), // 棕绿
            Color::rgb(100, 100, 150), // 棕蓝
            Color::rgb(150, 150, 50),  // 橄榄
            Color::rgb(150, 50, 150),  // 深紫红
            Color::rgb(50, 150, 150),  // 深青绿
            Color::rgb(200, 100, 50),  // 红橙
            Color::rgb(100, 200, 50),  // 黄绿
            Color::rgb(50, 100, 200),  // 蓝紫
            Color::rgb(200, 50, 100),  // 红紫
            Color::rgb(100, 50, 200),  // 紫蓝
        ];
        
        Self {
            size,
            pentagon_colors,
            hexagon_colors,
            rotation: Vec3::ZERO,
            wireframe: false,
            wireframe_color: Color::WHITE,
        }
    }

    /// 创建标准黑白足球
    pub fn with_standard_colors(mut self) -> Self {
        self.pentagon_colors = [Color::BLACK; 12];
        self.hexagon_colors = [Color::WHITE; 20];
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
    /// 数据来源: `/home/uan-wsl2/codes/FeatherOS/docs/截角二十面体.obj`
    /// OBJ 文件中的顶点坐标（已缩放到合适大小）
    /// 
    /// 注意：为了与立方体对齐，足球的直径 = 立方体的空间对角线 = size * sqrt(3)
    /// 原始 OBJ 数据的包围盒约为 [-150, 150]，所以缩放因子为 size * 0.866 / 150
    pub fn get_vertices(&self) -> [Vec3; 60] {
        // 原始 OBJ 顶点数据（从文件读取）
        // 注意：OBJ 文件中的顶点索引从 1 开始，这里保持原始坐标
        let raw_vertices: [(f32, f32, f32); 60] = [
            (-0.000370, 81.976440, 128.476242),      // 1 -> 0
            (0.000376, 24.522030, 150.415939),       // 2 -> 1
            (49.755840, 103.923996, 99.751915),      // 3 -> 2
            (99.511360, 68.417496, 92.966797),       // 4 -> 3
            (30.751171, 139.436325, 53.274368),      // 5 -> 4
            (61.501007, 139.441406, 0.012680),       // 6 -> 5
            (-30.750563, 139.436584, 53.273998),     // 7 -> 6
            (-61.501610, 139.441132, 0.013036),      // 8 -> 7
            (-49.756062, 103.924431, 99.751335),     // 9 -> 8
            (-99.511086, 68.417061, 92.967354),      // 10 -> 9
            (-49.755829, -10.985058, 143.631348),    // 11 -> 10
            (-99.511322, 10.961706, 114.906982),     // 12 -> 11
            (-30.751173, -68.437439, 132.653534),    // 13 -> 12
            (-61.501022, -103.941841, 92.951599),    // 14 -> 13
            (30.750572, -68.437866, 132.653442),     // 15 -> 14
            (61.501610, -103.941399, 92.951675),     // 16 -> 15
            (49.756081, -10.985776, 143.631210),     // 17 -> 16
            (99.511116, 10.962403, 114.907127),      // 18 -> 17
            (130.262299, 68.421860, 39.704739),      // 19 -> 18
            (111.257088, 103.934052, -6.771563),     // 20 -> 19
            (149.267181, 10.969494, 28.727331),      // 21 -> 20
            (149.267166, -10.969493, -28.727345),    // 22 -> 21
            (130.261917, -24.542421, 75.204964),     // 23 -> 22
            (111.257454, -81.993843, 64.226685),     // 24 -> 23
            (30.750444, -139.445938, 53.249565),     // 25 -> 24
            (-30.750448, -139.446228, 53.248871),    // 26 -> 25
            (61.501007, -139.441391, -0.012670),     // 27 -> 26
            (30.751173, -139.436325, -53.274357),    // 28 -> 27
            (111.257065, -103.934044, 6.771553),     // 29 -> 28
            (130.262268, -68.421860, -39.704750),    // 30 -> 29
            (130.261887, 24.542418, -75.204956),     // 31 -> 30
            (111.257454, 81.993858, -64.226685),     // 32 -> 31
            (99.511093, -10.962395, -114.907127),    // 33 -> 32
            (49.756058, 10.985778, -143.631226),     // 34 -> 33
            (99.511322, -68.417488, -92.966774),     // 35 -> 34
            (49.755833, -103.924004, -99.751892),    // 36 -> 35
            (-30.750574, -139.436584, -53.274025),   // 37 -> 36
            (-61.501625, -139.441132, -0.013040),    // 38 -> 37
            (-49.756077, -103.924416, -99.751350),   // 39 -> 38
            (-99.511116, -68.417046, -92.967361),    // 40 -> 39
            (-0.000389, -81.976425, -128.476257),    // 41 -> 40
            (0.000365, -24.522007, -150.415924),     // 42 -> 41
            (30.750565, 68.437881, -132.653458),     // 43 -> 42
            (61.501625, 103.941422, -92.951683),     // 44 -> 43
            (-30.751173, 68.437454, -132.653564),    // 45 -> 44
            (-61.501026, 103.941841, -92.951599),    // 46 -> 45
            (-49.755848, 10.985067, -143.631348),    // 47 -> 46
            (-99.511345, -10.961706, -114.906982),   // 48 -> 47
            (-130.261917, -68.422287, -39.705238),   // 49 -> 48
            (-111.257469, -103.933601, 6.772027),    // 50 -> 49
            (-149.267181, -10.970186, -28.727079),   // 51 -> 50
            (-149.267166, 10.970192, 28.727062),     // 52 -> 51
            (-130.262283, 24.542418, -75.204308),    // 53 -> 52
            (-111.257088, 81.993858, -64.227333),    // 54 -> 53
            (-30.750448, 139.446228, -53.248871),    // 55 -> 54
            (30.750448, 139.445969, -53.249573),     // 56 -> 55
            (-111.257462, 103.933617, -6.772024),    // 57 -> 56
            (-130.261887, 68.422295, 39.705223),     // 58 -> 57
            (-130.262283, -24.542408, 75.204300),    // 59 -> 58
            (-111.257088, -81.993843, 64.227325),    // 60 -> 59
        ];
        
        // 计算缩放因子，使足球直径与立方体空间对角线对齐
        // 原始数据的包围盒约为 [-150, 150]，直径约为 300
        // 目标直径 = size * sqrt(3) ≈ size * 1.732
        let target_diameter = self.size * 1.732051; // sqrt(3)
        let original_diameter = 300.0; // 近似原始直径
        let scale = target_diameter / original_diameter;
        
        // 缩放顶点
        let mut vertices = [Vec3::ZERO; 60];
        for (i, (x, y, z)) in raw_vertices.iter().enumerate() {
            vertices[i] = Vec3::new(x * scale, y * scale, z * scale);
        }
        
        vertices
    }

    /// 获取20个六边形面（每个面由6个顶点索引组成）
    /// 
    /// 数据来源: `/home/uan-wsl2/codes/FeatherOS/docs/截角二十面体.obj`
    /// OBJ 文件中的面索引从1开始，这里已转换为从0开始
    pub fn get_hexagon_faces(&self) -> [[usize; 6]; 20] {
        [
            [1, 16, 17, 3, 2, 0],           // 六边形 0 (f 2 17 18 4 3 1)
            [3, 18, 19, 5, 4, 2],           // 六边形 1 (f 4 19 20 6 5 3)
            [5, 55, 54, 7, 6, 4],           // 六边形 2 (f 6 56 55 8 7 5)
            [7, 56, 57, 9, 8, 6],           // 六边形 3 (f 8 57 58 10 9 7)
            [9, 11, 10, 1, 0, 8],           // 六边形 4 (f 10 12 11 2 1 9)
            [11, 58, 59, 13, 12, 10],       // 六边形 5 (f 12 59 60 14 13 11)
            [13, 25, 24, 15, 14, 12],       // 六边形 6 (f 14 26 25 16 15 13)
            [15, 23, 22, 17, 16, 14],       // 六边形 7 (f 16 24 23 18 17 15)
            [18, 20, 21, 30, 31, 19],       // 六边形 8 (f 19 21 22 31 32 20)
            [20, 22, 23, 28, 29, 21],       // 六边形 9 (f 21 23 24 29 30 22)
            [25, 37, 36, 27, 26, 24],       // 六边形 10 (f 26 38 37 28 27 25)
            [27, 35, 34, 29, 28, 26],       // 六边形 11 (f 28 36 35 30 29 27)
            [30, 32, 33, 42, 43, 31],       // 六边形 12 (f 31 33 34 43 44 32)
            [32, 34, 35, 40, 41, 33],       // 六边形 13 (f 33 35 36 41 42 34)
            [37, 49, 48, 39, 38, 36],       // 六边形 14 (f 38 50 49 40 39 37)
            [39, 47, 46, 41, 40, 38],       // 六边形 15 (f 40 48 47 42 41 39)
            [42, 44, 45, 54, 55, 43],       // 六边形 16 (f 43 45 46 55 56 44)
            [44, 46, 47, 52, 53, 45],       // 六边形 17 (f 45 47 48 53 54 46)
            [49, 59, 58, 51, 50, 48],       // 六边形 18 (f 50 60 59 52 51 49)
            [51, 57, 56, 53, 52, 50],       // 六边形 19 (f 52 58 57 54 53 51)
        ]
    }

    /// 获取12个五边形面（每个面由5个顶点索引组成）
    /// 
    /// 数据来源: `/home/uan-wsl2/codes/FeatherOS/docs/截角二十面体.obj`
    /// OBJ 文件中的面索引从1开始，这里已转换为从0开始
    pub fn get_pentagon_faces(&self) -> [[usize; 5]; 12] {
        [
            [2, 4, 6, 8, 0],                // 五边形 0 (f 3 5 7 9 1)
            [10, 12, 14, 16, 1],            // 五边形 1 (f 11 13 15 17 2)
            [17, 22, 20, 18, 3],            // 五边形 2 (f 18 23 21 19 4)
            [15, 24, 26, 28, 23],           // 五边形 3 (f 16 25 27 29 24)
            [29, 34, 32, 30, 21],           // 五边形 4 (f 30 35 33 31 22)
            [27, 36, 38, 40, 35],           // 五边形 5 (f 28 37 39 41 36)
            [41, 46, 44, 42, 33],           // 五边形 6 (f 42 47 45 43 34)
            [39, 48, 50, 52, 47],           // 五边形 7 (f 40 49 51 53 48)
            [53, 56, 7, 54, 45],            // 五边形 8 (f 54 57 8 55 46)
            [51, 58, 11, 9, 57],            // 五边形 9 (f 52 59 12 10 58)
            [59, 49, 37, 25, 13],           // 五边形 10 (f 60 50 38 26 14)
            [5, 19, 31, 43, 55],            // 五边形 11 (f 6 20 32 44 56)
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

impl crate::render_world::RenderComponent for SoccerBall {
    fn generate_render_commands(&self, transform: &crate::node::Transform3D, view: &crate::render_world::View) -> Vec<crate::render_world::RenderCommand> {
        use crate::math::{Vec2, Vec3, Mat4};
        use crate::render_world::RenderCommand;

        let mut commands = Vec::new();

        // 获取预定义的顶点和面数据
        let vertices = self.get_vertices();
        let hexagons = self.get_hexagon_faces();
        let pentagons = self.get_pentagon_faces();

        // 构建旋转矩阵
        let rot_x = Mat4::from_rotation_x(self.rotation.x.to_radians());
        let rot_y = Mat4::from_rotation_y(self.rotation.y.to_radians());
        let rot_z = Mat4::from_rotation_z(self.rotation.z.to_radians());
        let rotation = rot_z.mul(&rot_y).mul(&rot_x);

        // 变换顶点到世界空间
        let mut world_vertices: Vec<Vec3> = Vec::with_capacity(60);
        for v in &vertices {
            let rotated = rotation.mul_vec3(*v);
            let world_pos = rotated + transform.position;
            world_vertices.push(world_pos);
        }

        // 投影顶点到屏幕空间并计算 view-space Z
        let mut screen_vertices: Vec<Vec2> = Vec::with_capacity(60);
        let mut view_z: Vec<f32> = Vec::with_capacity(60);
        
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

        if screen_vertices.len() != 60 {
            return commands;
        }

        // 收集所有面及其深度
        let mut all_faces: Vec<(usize, bool, f32, Vec<Vec2>)> = Vec::new();

        // 处理20个六边形面
        for (face_idx, face) in hexagons.iter().enumerate() {
            let verts: Vec<Vec2> = face.iter()
                .map(|&idx| screen_vertices[idx])
                .collect();
            // 使用面的平均 Z 值来排序（更准确地表示面的整体深度）
            let avg_z: f32 = face.iter()
                .map(|&idx| view_z[idx])
                .sum::<f32>() / 6.0;
            all_faces.push((face_idx, false, avg_z, verts));
        }

        // 处理12个五边形面
        for (face_idx, face) in pentagons.iter().enumerate() {
            let verts: Vec<Vec2> = face.iter()
                .map(|&idx| screen_vertices[idx])
                .collect();
            // 使用面的平均 Z 值来排序
            let avg_z: f32 = face.iter()
                .map(|&idx| view_z[idx])
                .sum::<f32>() / 5.0;
            all_faces.push((face_idx, true, avg_z, verts));
        }

        // 按 view-space Z 排序（远的先画，Z 值大的先画）
        // 注意：在 view space 中，相机看向 -Z 方向
        // Z 值越大（越接近0）表示越远，Z 值越小（越负）表示越近
        // 画家算法：先画远的（Z 值大的/接近0的），后画近的（Z 值小的/更负的）
        // 降序排序：从大到小（远的先画）
        all_faces.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // 绘制所有面
        for (face_idx, is_pentagon, _, verts) in all_faces {
            if is_pentagon {
                // 绘制五边形（使用多边形填充，避免三角形拼接缝隙）
                let color = self.pentagon_colors[face_idx];
                
                // 使用 DrawPolygon 直接填充五边形
                commands.push(RenderCommand::DrawPolygon {
                    vertices: verts.clone(),
                    color,
                });

                // 绘制五边形线框
                if self.wireframe {
                    let wf_color = self.wireframe_color;
                    let v = &verts;
                    for i in 0..5 {
                        commands.push(RenderCommand::DrawLine {
                            start: v[i],
                            end: v[(i + 1) % 5],
                            color: wf_color,
                            thickness: 1.0,
                        });
                    }
                }
            } else {
                // 绘制六边形（使用多边形填充，避免三角形拼接缝隙）
                let color = self.hexagon_colors[face_idx];
                
                // 使用 DrawPolygon 直接填充六边形
                commands.push(RenderCommand::DrawPolygon {
                    vertices: verts.clone(),
                    color,
                });

                // 绘制六边形线框
                if self.wireframe {
                    let wf_color = self.wireframe_color;
                    let v = &verts;
                    for i in 0..6 {
                        commands.push(RenderCommand::DrawLine {
                            start: v[i],
                            end: v[(i + 1) % 6],
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
