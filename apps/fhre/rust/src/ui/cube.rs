//! Cube Component
//!
//! 简单的 3D 立方体 UI 组件，用于展示 3D 效果。

use crate::Component;
use crate::math::{Color, Vec3};
use alloc::vec::Vec;

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

impl crate::animation::AnimationReceiver for Cube {
    fn apply_animation(&mut self, property: crate::animation::AnimationProperty, value: f32) {
        use crate::animation::AnimationProperty;
        match property {
            // === 3D Rotation (Euler angles in degrees) ===
            AnimationProperty::RotationX => { self.rotation.x = value; }
            AnimationProperty::RotationY => { self.rotation.y = value; }
            AnimationProperty::RotationZ => { self.rotation.z = value; }

            // === Scale (uniform: size field) ===
            AnimationProperty::ScaleX | AnimationProperty::ScaleY | AnimationProperty::ScaleZ => {
                self.size = value;
            }

            // === Translation (position is in Transform3D, not Cube) ===
            AnimationProperty::TranslationX | AnimationProperty::TranslationY | AnimationProperty::TranslationZ => {}

            // === Unsupported for Cube ===
            _ => {}
        }
    }
}

impl crate::render_world::RenderComponent for Cube {
    fn generate_render_commands(&self, transform: &crate::node::Transform3D, view: &crate::render_world::View) -> Vec<crate::render_world::RenderCommand> {
        use crate::math::{Vec2, Vec3, Mat4};
        use crate::render_world::RenderCommand;
        use alloc::vec::Vec;

        let mut commands = Vec::new();

        // 获取立方体的顶点和面
        let vertices = self.get_vertices();
        let faces = self.get_faces();

        // 构建立方体的旋转矩阵
        let rot_x = Mat4::from_rotation_x(self.rotation.x.to_radians());
        let rot_y = Mat4::from_rotation_y(self.rotation.y.to_radians());
        let rot_z = Mat4::from_rotation_z(self.rotation.z.to_radians());
        let rotation = rot_z.mul(&rot_y).mul(&rot_x);

        // 变换顶点到世界空间
        let mut world_vertices: Vec<Vec3> = Vec::with_capacity(8);
        for v in &vertices {
            let rotated = rotation.mul_vec3(*v);
            let world_pos = rotated + transform.position;
            world_vertices.push(world_pos);
        }

        // 投影顶点到屏幕空间并计算 view-space Z
        let mut screen_vertices: Vec<Vec2> = Vec::with_capacity(8);
        let mut view_z: Vec<f32> = Vec::with_capacity(8);
        
        for world_pos in &world_vertices {
            // 变换到 view space（相机相对坐标）
            let view_pos = view.view.mul_vec3(*world_pos);
            view_z.push(view_pos.z);
            
            if let Some((x, y)) = view.world_to_screen(*world_pos) {
                screen_vertices.push(Vec2::new(x, y));
            } else {
                // 顶点在相机后面，跳过这个立方体
                screen_vertices.clear();
                break;
            }
        }

        if screen_vertices.len() != 8 {
            return commands;
        }

        // 收集所有可见的面及其深度
        let mut visible_faces: Vec<(usize, f32, [Vec2; 4])> = Vec::new();

        for (face_idx, face) in faces.iter().enumerate() {
            // 获取面的四个顶点（屏幕坐标）
            let v0 = screen_vertices[face[0]];
            let v1 = screen_vertices[face[1]];
            let v2 = screen_vertices[face[2]];
            let v3 = screen_vertices[face[3]];

            // 背面剔除：计算面的法向量（在屏幕空间）
            let edge1 = v1 - v0;
            let edge2 = v3 - v0;
            let _cross_z = edge1.x * edge2.y - edge1.y * edge2.x;

            // 背面剔除：不启用
            // 原因：前面的面可能带透明度，需要绘制背面才能正确显示
            // 画家算法（按深度排序）已足够处理遮挡关系
            if true {
                // 计算面的平均 view-space Z（用于排序）
                // view-space Z 越小表示越近（相机在 +Z 看向 -Z）
                let avg_view_z = (view_z[face[0]] 
                                + view_z[face[1]] 
                                + view_z[face[2]] 
                                + view_z[face[3]]) / 4.0;
                
                visible_faces.push((face_idx, avg_view_z, [v0, v1, v2, v3]));
            }
        }

        // 按 view-space Z 排序（远的先画，Z 值小的先画）
        // 在右手坐标系 view space 中，相机看向 -Z
        // Z 值越小（越负）表示越远，Z 值越大（越接近 0）表示越近
        // 画家算法：先画远的（Z 值小的/更负的），后画近的（Z 值大的/接近0的）
        visible_faces.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // 绘制可见的面（从远到近）
        for (face_idx, _, [v0, v1, v2, v3]) in visible_faces {
            let color = self.face_colors[face_idx];

            // 使用多边形填充四边形（避免三角形拼接缝隙）
            commands.push(RenderCommand::DrawPolygon {
                vertices: alloc::vec![v0, v1, v2, v3],
                color,
            });

            // Draw wireframe if enabled
            if self.wireframe {
                let wf_color = self.wireframe_color;

                commands.push(RenderCommand::DrawLine {
                    start: v0,
                    end: v1,
                    color: wf_color,
                    thickness: 1.0,
                });
                commands.push(RenderCommand::DrawLine {
                    start: v1,
                    end: v2,
                    color: wf_color,
                    thickness: 1.0,
                });
                commands.push(RenderCommand::DrawLine {
                    start: v2,
                    end: v3,
                    color: wf_color,
                    thickness: 1.0,
                });
                commands.push(RenderCommand::DrawLine {
                    start: v3,
                    end: v0,
                    color: wf_color,
                    thickness: 1.0,
                });
            }
        }

        commands
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
