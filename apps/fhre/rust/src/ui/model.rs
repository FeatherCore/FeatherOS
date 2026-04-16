//! 3D Model Loading and Management
//!
//! 提供简单的 OBJ 格式模型加载和默认 3D 模型数据。
//! 支持顶点、法线和面数据。

use crate::math::{Vec3, Color};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::vec;
use alloc::format;

/// 3D 模型数据
#[derive(Debug, Clone, PartialEq)]
pub struct Model3D {
    /// 顶点列表
    pub vertices: Vec<Vec3>,
    /// 法线列表
    pub normals: Vec<Vec3>,
    /// 面列表（每个面是顶点索引的列表）
    pub faces: Vec<Vec<usize>>,
    /// 面颜色（每个面的颜色）
    pub face_colors: Vec<Color>,
    /// 模型名称
    pub name: String,
}

impl Model3D {
    /// 创建空模型
    pub fn new(name: &str) -> Self {
        Self {
            vertices: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
            face_colors: Vec::new(),
            name: String::from(name),
        }
    }

    /// 从 OBJ 格式字符串加载模型
    /// 
    /// 支持简单的 OBJ 格式：
    /// - v x y z (顶点)
    /// - vn x y z (法线)
    /// - f v1 v2 v3 ... (面，顶点索引从1开始)
    pub fn from_obj_str(obj_data: &str, name: &str) -> Self {
        let mut model = Self::new(name);
        
        for line in obj_data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            
            match parts[0] {
                "v" => {
                    // 顶点: v x y z
                    if parts.len() >= 4 {
                        if let (Ok(x), Ok(y), Ok(z)) = (
                            parts[1].parse::<f32>(),
                            parts[2].parse::<f32>(),
                            parts[3].parse::<f32>()
                        ) {
                            model.vertices.push(Vec3::new(x, y, z));
                        }
                    }
                }
                "vn" => {
                    // 法线: vn x y z
                    if parts.len() >= 4 {
                        if let (Ok(x), Ok(y), Ok(z)) = (
                            parts[1].parse::<f32>(),
                            parts[2].parse::<f32>(),
                            parts[3].parse::<f32>()
                        ) {
                            model.normals.push(Vec3::new(x, y, z));
                        }
                    }
                }
                "f" => {
                    // 面: f v1 v2 v3 ... (支持 v/vt/vn 格式，只取顶点索引)
                    let mut face = Vec::new();
                    for i in 1..parts.len() {
                        let part = parts[i];
                        // 处理 v/vt/vn 格式，只取 v 部分
                        let v_str = part.split('/').next().unwrap_or(part);
                        if let Ok(v_idx) = v_str.parse::<usize>() {
                            // OBJ 索引从 1 开始，转换为 0 开始
                            face.push(v_idx.saturating_sub(1));
                        }
                    }
                    if face.len() >= 3 {
                        model.faces.push(face);
                        // 默认颜色为白色
                        model.face_colors.push(Color::WHITE);
                    }
                }
                _ => {}
            }
        }
        
        model
    }

    /// 设置所有面的颜色
    pub fn set_all_face_colors(&mut self, color: Color) {
        for c in &mut self.face_colors {
            *c = color;
        }
    }

    /// 设置特定面的颜色
    pub fn set_face_color(&mut self, face_idx: usize, color: Color) {
        if face_idx < self.face_colors.len() {
            self.face_colors[face_idx] = color;
        }
    }

    /// 缩放模型
    pub fn scale(&mut self, scale: f32) {
        for v in &mut self.vertices {
            v.x *= scale;
            v.y *= scale;
            v.z *= scale;
        }
    }

    /// 获取模型中心点
    pub fn get_center(&self) -> Vec3 {
        if self.vertices.is_empty() {
            return Vec3::ZERO;
        }
        
        let mut sum = Vec3::ZERO;
        for v in &self.vertices {
            sum.x += v.x;
            sum.y += v.y;
            sum.z += v.z;
        }
        
        Vec3::new(
            sum.x / self.vertices.len() as f32,
            sum.y / self.vertices.len() as f32,
            sum.z / self.vertices.len() as f32,
        )
    }

    /// 将模型中心移动到原点
    pub fn center_at_origin(&mut self) {
        let center = self.get_center();
        for v in &mut self.vertices {
            v.x -= center.x;
            v.y -= center.y;
            v.z -= center.z;
        }
    }

    /// 获取顶点数量
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 获取面数量
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }
}

impl Default for Model3D {
    fn default() -> Self {
        Self::new("unnamed")
    }
}

/// 默认模型库
pub mod default_models {
    use super::*;

    /// 创建立方体模型
    /// 
    /// 创建一个轴对齐的立方体，边长为 2.0（从 -1 到 1）
    pub fn create_cube() -> Model3D {
        let obj_data = r#"
# Cube model
# 8 vertices
v -1.0 -1.0 -1.0
v  1.0 -1.0 -1.0
v  1.0  1.0 -1.0
v -1.0  1.0 -1.0
v -1.0 -1.0  1.0
v  1.0 -1.0  1.0
v  1.0  1.0  1.0
v -1.0  1.0  1.0

# 6 faces (quads)
f 1 2 3 4
f 5 8 7 6
f 1 5 6 2
f 2 6 7 3
f 3 7 8 4
f 4 8 5 1
"#;

        let mut model = Model3D::from_obj_str(obj_data, "cube");
        
        // 设置立方体面的颜色（每个面不同颜色）
        let colors = [
            Color::RED,      // 前面
            Color::GREEN,    // 后面
            Color::BLUE,     // 底面
            Color::YELLOW,   // 右面
            Color::CYAN,     // 顶面
            Color::MAGENTA,  // 左面
        ];
        
        for (i, color) in colors.iter().enumerate() {
            if i < model.face_colors.len() {
                model.face_colors[i] = *color;
            }
        }
        
        model
    }

    /// 创建足球模型（截角二十面体）
    /// 
    /// 使用标准的截角二十面体 OBJ 数据
    pub fn create_soccer_ball() -> Model3D {
        // 截角二十面体的 OBJ 数据（内嵌）
        // 60 个顶点，12 个五边形，20 个六边形
        let obj_data = r#"# WaveFront *.obj file
v -0.000370 81.976 128.476
v 0.000376 24.522 150.416
v 49.756 103.924 99.752
v 99.511 68.417 92.967
v 30.751 139.436 53.274
v 61.501 139.441 0.013
v -30.751 139.437 53.274
v -61.502 139.441 0.013
v -49.756 103.924 99.751
v -99.511 68.417 92.967
v -49.756 -10.985 143.631
v -99.511 10.962 114.907
v -30.751 -68.437 132.654
v -61.501 -103.942 92.952
v 30.751 -68.438 132.653
v 61.502 -103.941 92.952
v 49.756 -10.986 143.631
v 99.511 10.962 114.907
v 130.262 68.422 39.705
v 111.257 103.934 -6.772
v 149.267 10.969 28.727
v 149.267 -10.969 -28.727
v 130.262 -24.542 75.205
v 111.257 -81.994 64.227
v 30.750 -139.446 53.250
v -30.750 -139.446 53.249
v 61.501 -139.441 -0.013
v 30.751 -139.436 -53.274
v 111.257 -103.934 6.772
v 130.262 -68.422 -39.705
v 130.262 24.542 -75.205
v 111.257 81.994 -64.227
v 99.511 -10.962 -114.907
v 49.756 10.986 -143.631
v 99.511 -68.417 -92.967
v 49.756 -103.924 -99.752
v -30.751 -139.437 -53.274
v -61.502 -139.441 -0.013
v -49.756 -103.924 -99.751
v -99.511 -68.417 -92.967
v -0.000389 -81.976 -128.476
v 0.000365 -24.522 -150.416
v 30.751 68.438 -132.653
v 61.502 103.941 -92.952
v -30.751 68.437 -132.654
v -61.501 103.942 -92.952
v -49.756 10.985 -143.631
v -99.511 -10.962 -114.907
v -130.262 -68.422 -39.705
v -111.257 -103.934 6.772
v -149.267 -10.970 -28.727
v -149.267 10.970 28.727
v -130.262 24.542 -75.204
v -111.257 81.994 -64.227
v -30.750 139.446 -53.249
v 30.750 139.446 -53.250
v -111.257 103.934 -6.772
v -130.262 68.422 39.705
v -130.262 -24.542 75.204
v -111.257 -81.994 64.227
f 2 17 18 4 3 1
f 4 19 20 6 5 3
f 6 56 55 8 7 5
f 8 57 58 10 9 7
f 10 12 11 2 1 9
f 12 59 60 14 13 11
f 14 26 25 16 15 13
f 16 24 23 18 17 15
f 19 21 22 31 32 20
f 21 23 24 29 30 22
f 26 38 37 28 27 25
f 28 36 35 30 29 27
f 31 33 34 43 44 32
f 33 35 36 41 42 34
f 38 50 49 40 39 37
f 40 48 47 42 41 39
f 43 45 46 55 56 44
f 45 47 48 53 54 46
f 50 60 59 52 51 49
f 52 58 57 54 53 51
f 3 5 7 9 1
f 11 13 15 17 2
f 18 23 21 19 4
f 16 25 27 29 24
f 30 35 33 31 22
f 28 37 39 41 36
f 42 47 45 43 34
f 40 49 51 53 48
f 54 57 8 55 46
f 52 59 12 10 58
f 60 50 38 26 14
f 6 20 32 44 56
"#;
        
        let mut model = Model3D::from_obj_str(obj_data, "soccer_ball");
        
        // 设置足球颜色：12个五边形为黑色，20个六边形为白色
        // 前12个面是五边形，后20个面是六边形
        for i in 0..model.face_colors.len() {
            if i < 12 {
                model.face_colors[i] = Color::BLACK;  // 五边形
            } else {
                model.face_colors[i] = Color::WHITE;  // 六边形
            }
        }
        
        model
    }

    /// 创建球体模型（细分二十面体）
    /// 
    /// 创建一个细分级别的球体
    pub fn create_sphere(subdivisions: u32) -> Model3D {
        // 从基础二十面体开始
        let mut model = create_icosahedron();
        
        // 细分指定次数
        for _ in 0..subdivisions {
            subdivide_sphere(&mut model);
        }
        
        // 归一化所有顶点到球面
        for v in &mut model.vertices {
            let len = libm::sqrtf(v.x * v.x + v.y * v.y + v.z * v.z);
            if len > 0.0 {
                v.x /= len;
                v.y /= len;
                v.z /= len;
            }
        }
        
        model.name = String::from("sphere");
        model
    }

    /// 创建正二十面体
    fn create_icosahedron() -> Model3D {
        let phi: f32 = 1.618033988749895;
        
        let obj_data = format!(r#"
# Icosahedron model
# 12 vertices
v 0.0 1.0 {phi}
v 0.0 1.0 -{phi}
v 0.0 -1.0 {phi}
v 0.0 -1.0 -{phi}
v 1.0 {phi} 0.0
v 1.0 -{phi} 0.0
v -1.0 {phi} 0.0
v -1.0 -{phi} 0.0
v {phi} 0.0 1.0
v {phi} 0.0 -1.0
v -{phi} 0.0 1.0
v -{phi} 0.0 -1.0

# 20 triangular faces
f 1 9 5
f 1 5 7
f 1 7 11
f 1 11 3
f 1 3 9
f 2 6 10
f 2 8 6
f 2 12 8
f 2 4 12
f 2 10 4
f 3 8 6
f 3 11 8
f 4 10 9
f 4 12 5
f 5 12 7
f 6 7 10
f 7 6 8
f 8 11 12
f 9 3 6
f 9 10 5
"#, phi = phi);

        Model3D::from_obj_str(&obj_data, "icosahedron")
    }

    /// 细分球体模型
    fn subdivide_sphere(model: &mut Model3D) {
        let mut new_faces: Vec<Vec<usize>> = Vec::new();
        let mut new_colors: Vec<Color> = Vec::new();
        
        for (face_idx, face) in model.faces.iter().enumerate() {
            if face.len() == 3 {
                // 三角形细分
                let v0 = face[0];
                let v1 = face[1];
                let v2 = face[2];
                
                // 计算边中点
                let m01 = model.vertices.len();
                model.vertices.push(midpoint(&model.vertices[v0], &model.vertices[v1]));
                
                let m12 = model.vertices.len();
                model.vertices.push(midpoint(&model.vertices[v1], &model.vertices[v2]));
                
                let m20 = model.vertices.len();
                model.vertices.push(midpoint(&model.vertices[v2], &model.vertices[v0]));
                
                // 创建4个新三角形
                new_faces.push(vec![v0, m01, m20]);
                new_faces.push(vec![v1, m12, m01]);
                new_faces.push(vec![v2, m20, m12]);
                new_faces.push(vec![m01, m12, m20]);
                
                // 复制颜色
                let color = model.face_colors.get(face_idx).copied().unwrap_or(Color::WHITE);
                new_colors.push(color);
                new_colors.push(color);
                new_colors.push(color);
                new_colors.push(color);
            } else {
                // 非三角形面保持不变
                new_faces.push(face.clone());
                new_colors.push(model.face_colors.get(face_idx).copied().unwrap_or(Color::WHITE));
            }
        }
        
        model.faces = new_faces;
        model.face_colors = new_colors;
    }

    /// 计算两个顶点的中点
    fn midpoint(a: &Vec3, b: &Vec3) -> Vec3 {
        Vec3::new(
            (a.x + b.x) * 0.5,
            (a.y + b.y) * 0.5,
            (a.z + b.z) * 0.5,
        )
    }
}

/// 3D 模型组件 - 用于在场景中使用模型
#[derive(Debug, Clone, PartialEq)]
pub struct ModelComponent {
    /// 模型数据
    pub model: Model3D,
    /// 缩放
    pub scale: f32,
    /// 旋转（欧拉角，度）
    pub rotation: Vec3,
    /// 线框模式
    pub wireframe: bool,
    /// 线框颜色
    pub wireframe_color: Color,
}

impl ModelComponent {
    /// 从模型创建组件
    pub fn new(model: Model3D) -> Self {
        Self {
            model,
            scale: 1.0,
            rotation: Vec3::ZERO,
            wireframe: false,
            wireframe_color: Color::BLACK,
        }
    }

    /// 创建立方体组件
    pub fn cube() -> Self {
        Self::new(default_models::create_cube())
    }

    /// 创建足球组件
    pub fn soccer_ball() -> Self {
        Self::new(default_models::create_soccer_ball())
    }

    /// 创建球体组件
    pub fn sphere(subdivisions: u32) -> Self {
        Self::new(default_models::create_sphere(subdivisions))
    }

    /// 设置缩放
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// 设置旋转
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

    /// 旋转模型
    pub fn rotate(&mut self, delta: Vec3) {
        self.rotation.x += delta.x;
        self.rotation.y += delta.y;
        self.rotation.z += delta.z;
    }

    /// 获取缩放后的顶点
    pub fn get_scaled_vertices(&self) -> Vec<Vec3> {
        self.model.vertices.iter().map(|v| {
            Vec3::new(v.x * self.scale, v.y * self.scale, v.z * self.scale)
        }).collect()
    }
}

impl Default for ModelComponent {
    fn default() -> Self {
        Self::cube()
    }
}

impl Component for ModelComponent {
    fn type_name() -> &'static str {
        "ModelComponent"
    }
}

use crate::Component;
