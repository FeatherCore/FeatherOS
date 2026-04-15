//! Node3D 组件
//!
//! 提供 3D 变换和属性，用于 3D 游戏对象和 2.5D UI 效果。

use crate::Component;
use crate::math::{Vec3, Vec2};

/// 3D 变换组件
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3D {
    /// 3D 位置 (x, y, z)
    pub position: Vec3,
    /// 3D 旋转（欧拉角，弧度）
    pub rotation: Vec3,
    /// 3D 缩放
    pub scale: Vec3,
    /// 锚点（相对于自身的偏移，默认中心）
    pub anchor: Vec3,
}

impl Transform3D {
    /// 创建默认变换
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            anchor: Vec3::ZERO, // 中心点
        }
    }

    /// 从位置创建
    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            anchor: Vec3::ZERO,
        }
    }

    /// 从 Vec3 位置创建
    pub fn from_vec3(position: Vec3) -> Self {
        Self {
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            anchor: Vec3::ZERO,
        }
    }

    /// 设置位置
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Vec3::new(x, y, z);
        self
    }

    /// 设置旋转（欧拉角）
    pub fn with_rotation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.rotation = Vec3::new(x, y, z);
        self
    }

    /// 设置缩放
    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = Vec3::new(x, y, z);
        self
    }

    /// 设置统一缩放
    pub fn with_uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vec3::splat(scale);
        self
    }

    /// 设置锚点
    pub fn with_anchor(mut self, x: f32, y: f32, z: f32) -> Self {
        self.anchor = Vec3::new(x, y, z);
        self
    }

    /// 获取变换矩阵（用于渲染）
    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        // 简化的 3D 变换矩阵计算
        let cx = libm::cosf(self.rotation.x);
        let sx = libm::sinf(self.rotation.x);
        let cy = libm::cosf(self.rotation.y);
        let sy = libm::sinf(self.rotation.y);
        let cz = libm::cosf(self.rotation.z);
        let sz = libm::sinf(self.rotation.z);

        // 旋转矩阵 (Z * Y * X)
        let rot = [
            [cy * cz, -cy * sz, sy, 0.0],
            [sx * sy * cz + cx * sz, -sx * sy * sz + cx * cz, -sx * cy, 0.0],
            [-cx * sy * cz + sx * sz, cx * sy * sz + sx * cz, cx * cy, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        // 缩放和平移
        [
            [rot[0][0] * self.scale.x, rot[0][1] * self.scale.y, rot[0][2] * self.scale.z, self.position.x],
            [rot[1][0] * self.scale.x, rot[1][1] * self.scale.y, rot[1][2] * self.scale.z, self.position.y],
            [rot[2][0] * self.scale.x, rot[2][1] * self.scale.y, rot[2][2] * self.scale.z, self.position.z],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    /// 平移
    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        self.position.x += dx;
        self.position.y += dy;
        self.position.z += dz;
    }

    /// 旋转
    pub fn rotate(&mut self, dx: f32, dy: f32, dz: f32) {
        self.rotation.x += dx;
        self.rotation.y += dy;
        self.rotation.z += dz;
    }

    /// 缩放
    pub fn scale_by(&mut self, factor: f32) {
        self.scale.x *= factor;
        self.scale.y *= factor;
        self.scale.z *= factor;
    }

    /// 看向目标点
    pub fn look_at(&mut self, target: Vec3) {
        let direction = target - self.position;
        self.rotation.y = libm::atan2f(direction.x, direction.z);
        self.rotation.x = libm::atan2f(-direction.y, libm::sqrtf(direction.x * direction.x + direction.z * direction.z));
    }
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Transform3D {
    fn type_name() -> &'static str {
        "Transform3D"
    }
}

/// Node3D 组件
///
/// 组合了 3D 变换和边界框信息
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node3D {
    /// 本地变换
    pub local: Transform3D,
    /// 全局变换（计算后）
    pub global: Transform3D,
    /// 尺寸
    pub size: Vec3,
}

impl Node3D {
    /// 创建新的 Node3D
    pub fn new() -> Self {
        Self {
            local: Transform3D::new(),
            global: Transform3D::new(),
            size: Vec3::ZERO,
        }
    }

    /// 从位置创建
    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            local: Transform3D::from_position(x, y, z),
            global: Transform3D::from_position(x, y, z),
            size: Vec3::ZERO,
        }
    }

    /// 从位置和尺寸创建
    pub fn with_size(mut self, width: f32, height: f32, depth: f32) -> Self {
        self.size = Vec3::new(width, height, depth);
        self
    }

    /// 设置本地位置
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.local.position = Vec3::new(x, y, z);
    }

    /// 设置本地旋转
    pub fn set_rotation(&mut self, x: f32, y: f32, z: f32) {
        self.local.rotation = Vec3::new(x, y, z);
    }

    /// 设置本地缩放
    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.local.scale = Vec3::new(x, y, z);
    }

    /// 更新全局变换（由系统调用）
    pub fn update_global(&mut self, parent_global: &Transform3D) {
        // 组合父变换和本地变换
        self.global.position = parent_global.position + self.local.position;
        self.global.rotation = Vec3::new(
            parent_global.rotation.x + self.local.rotation.x,
            parent_global.rotation.y + self.local.rotation.y,
            parent_global.rotation.z + self.local.rotation.z,
        );
        self.global.scale = Vec3::new(
            parent_global.scale.x * self.local.scale.x,
            parent_global.scale.y * self.local.scale.y,
            parent_global.scale.z * self.local.scale.z,
        );
        self.global.anchor = self.local.anchor;
    }

    /// 投影到 2D 屏幕坐标
    pub fn project_to_2d(&self, camera_position: Vec3, screen_size: Vec2) -> Vec2 {
        // 简化的透视投影
        let relative_pos = self.global.position - camera_position;
        let distance = relative_pos.z.max(0.1); // 避免除以零
        
        let screen_x = (relative_pos.x / distance) * screen_size.x * 0.5 + screen_size.x * 0.5;
        let screen_y = (relative_pos.y / distance) * screen_size.y * 0.5 + screen_size.y * 0.5;
        
        Vec2::new(screen_x, screen_y)
    }
}

impl Default for Node3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Node3D {
    fn type_name() -> &'static str {
        "Node3D"
    }
}

/// 全局变换 3D（由系统计算）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform3D(pub Transform3D);

impl Default for GlobalTransform3D {
    fn default() -> Self {
        Self(Transform3D::new())
    }
}

impl Component for GlobalTransform3D {
    fn type_name() -> &'static str {
        "GlobalTransform3D"
    }
}

/// 摄像机组件
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera3D {
    /// 视野角度（度）
    pub fov: f32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 背景颜色
    pub background_color: crate::math::Color,
    /// 是否正交投影
    pub orthographic: bool,
    /// 正交投影尺寸
    pub orthographic_size: f32,
    /// 视口（归一化坐标）
    pub viewport: crate::math::Rect,
    /// 渲染深度（排序用）
    pub depth: i32,
    /// 裁剪掩码
    pub culling_mask: u32,
}

impl Camera3D {
    pub fn new() -> Self {
        Self {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            background_color: crate::math::Color::BLACK,
            orthographic: false,
            orthographic_size: 5.0,
            viewport: crate::math::Rect::new(0.0, 0.0, 1.0, 1.0),
            depth: 0,
            culling_mask: 0xFFFFFFFF,
        }
    }

    pub fn with_fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }

    pub fn with_clip(mut self, near: f32, far: f32) -> Self {
        self.near = near;
        self.far = far;
        self
    }

    pub fn orthographic(mut self, size: f32) -> Self {
        self.orthographic = true;
        self.orthographic_size = size;
        self
    }

    pub fn with_viewport(mut self, viewport: crate::math::Rect) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn with_depth(mut self, depth: i32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_culling_mask(mut self, mask: u32) -> Self {
        self.culling_mask = mask;
        self
    }

    pub fn with_background_color(mut self, color: crate::math::Color) -> Self {
        self.background_color = color;
        self
    }
}

impl Default for Camera3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Camera3D {
    fn type_name() -> &'static str {
        "Camera3D"
    }
}

/// 光源组件
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light3D {
    /// 光源类型
    pub light_type: LightType,
    /// 颜色
    pub color: crate::math::Color,
    /// 强度
    pub intensity: f32,
    /// 范围（点光源和聚光灯）
    pub range: f32,
    /// 聚光角度（聚光灯）
    pub spot_angle: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Directional, // 平行光
    Point,       // 点光源
    Spot,        // 聚光灯
    Ambient,     // 环境光
}

impl Light3D {
    pub fn directional() -> Self {
        Self {
            light_type: LightType::Directional,
            color: crate::math::Color::WHITE,
            intensity: 1.0,
            range: 0.0,
            spot_angle: 0.0,
        }
    }

    pub fn point() -> Self {
        Self {
            light_type: LightType::Point,
            color: crate::math::Color::WHITE,
            intensity: 1.0,
            range: 10.0,
            spot_angle: 0.0,
        }
    }

    pub fn spot() -> Self {
        Self {
            light_type: LightType::Spot,
            color: crate::math::Color::WHITE,
            intensity: 1.0,
            range: 10.0,
            spot_angle: 45.0,
        }
    }
}

impl Component for Light3D {
    fn type_name() -> &'static str {
        "Light3D"
    }
}
