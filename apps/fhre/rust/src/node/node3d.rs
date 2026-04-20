//! Transform 组件
//!
//! FHRE 是纯 3D 引擎，所有实体都在 3D 空间中。
//! 2D 只是 3D 的特例：z=0 平面上的对象，通过正交相机投影到屏幕。
//!
//! # 设计理念
//!
//! - 统一使用 Transform (Transform3D) 作为唯一变换组件
//! - 2D 元素：position.z = 0，rotation.x/y = 0，scale.z = 1
//! - UI 层级：通过 z_order 或 position.z 区分
//! - 相机系统：正交相机用于 UI/2D，透视相机用于 3D 场景

use crate::Component;
use crate::math::{Vec3, Vec2, Rect};

/// Transform 组件（统一 3D 变换）
///
/// FHRE 的唯一变换组件，所有实体都使用此组件。
/// 2D 元素只需将 z 分量设为 0。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// 位置 (x, y, z)
    pub position: Vec3,
    /// 旋转（欧拉角，弧度）
    pub rotation: Vec3,
    /// 缩放
    pub scale: Vec3,
    /// 锚点（相对于自身的偏移，默认中心）
    pub anchor: Vec3,
}

/// Transform3D 是 Transform 的别名，保持 API 兼容
pub type Transform3D = Transform;

impl Transform {
    pub const ZERO: Self = Self {
        position: Vec3::ZERO,
        rotation: Vec3::ZERO,
        scale: Vec3::ONE,
        anchor: Vec3::ZERO,
    };

    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            anchor: Vec3::ZERO,
        }
    }

    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            anchor: Vec3::ZERO,
        }
    }

    pub fn from_vec3(position: Vec3) -> Self {
        Self {
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            anchor: Vec3::ZERO,
        }
    }

    pub fn from_2d(x: f32, y: f32) -> Self {
        Self {
            position: Vec3::new(x, y, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            anchor: Vec3::ZERO,
        }
    }

    pub fn from_vec2(position: Vec2) -> Self {
        Self {
            position: Vec3::new(position.x, position.y, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            anchor: Vec3::ZERO,
        }
    }

    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Vec3::new(x, y, z);
        self
    }

    pub fn with_position_2d(mut self, x: f32, y: f32) -> Self {
        self.position = Vec3::new(x, y, self.position.z);
        self
    }

    pub fn with_rotation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.rotation = Vec3::new(x, y, z);
        self
    }

    pub fn with_rotation_2d(mut self, rotation: f32) -> Self {
        self.rotation = Vec3::new(0.0, 0.0, rotation);
        self
    }

    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = Vec3::new(x, y, z);
        self
    }

    pub fn with_scale_2d(mut self, x: f32, y: f32) -> Self {
        self.scale = Vec3::new(x, y, 1.0);
        self
    }

    pub fn with_uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vec3::splat(scale);
        self
    }

    pub fn with_anchor(mut self, x: f32, y: f32, z: f32) -> Self {
        self.anchor = Vec3::new(x, y, z);
        self
    }

    pub fn with_anchor_2d(mut self, x: f32, y: f32) -> Self {
        self.anchor = Vec3::new(x, y, 0.0);
        self
    }

    pub fn to_matrix(&self) -> [[f32; 4]; 4] {
        let cx = libm::cosf(self.rotation.x);
        let sx = libm::sinf(self.rotation.x);
        let cy = libm::cosf(self.rotation.y);
        let sy = libm::sinf(self.rotation.y);
        let cz = libm::cosf(self.rotation.z);
        let sz = libm::sinf(self.rotation.z);

        let rot = [
            [cy * cz, -cy * sz, sy, 0.0],
            [sx * sy * cz + cx * sz, -sx * sy * sz + cx * cz, -sx * cy, 0.0],
            [-cx * sy * cz + sx * sz, cx * sy * sz + sx * cz, cx * cy, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        [
            [rot[0][0] * self.scale.x, rot[0][1] * self.scale.y, rot[0][2] * self.scale.z, self.position.x],
            [rot[1][0] * self.scale.x, rot[1][1] * self.scale.y, rot[1][2] * self.scale.z, self.position.y],
            [rot[2][0] * self.scale.x, rot[2][1] * self.scale.y, rot[2][2] * self.scale.z, self.position.z],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    pub fn to_matrix_2d(&self) -> [[f32; 3]; 3] {
        let cos = libm::cosf(self.rotation.z);
        let sin = libm::sinf(self.rotation.z);

        [
            [cos * self.scale.x, -sin * self.scale.y, self.position.x],
            [sin * self.scale.x, cos * self.scale.y, self.position.y],
            [0.0, 0.0, 1.0],
        ]
    }

    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        self.position.x += dx;
        self.position.y += dy;
        self.position.z += dz;
    }

    pub fn translate_2d(&mut self, dx: f32, dy: f32) {
        self.position.x += dx;
        self.position.y += dy;
    }

    pub fn rotate(&mut self, dx: f32, dy: f32, dz: f32) {
        self.rotation.x += dx;
        self.rotation.y += dy;
        self.rotation.z += dz;
    }

    pub fn rotate_2d(&mut self, angle: f32) {
        self.rotation.z += angle;
    }

    pub fn scale_by(&mut self, factor: f32) {
        self.scale.x *= factor;
        self.scale.y *= factor;
        self.scale.z *= factor;
    }

    pub fn look_at(&mut self, target: Vec3) {
        let direction = target - self.position;
        self.rotation.y = libm::atan2f(direction.x, direction.z);
        self.rotation.x = libm::atan2f(-direction.y, libm::sqrtf(direction.x * direction.x + direction.z * direction.z));
    }

    pub fn is_2d(&self) -> bool {
        self.position.z == 0.0
            && self.rotation.x == 0.0
            && self.rotation.y == 0.0
            && self.scale.z == 1.0
    }

    pub fn xy(&self) -> Vec2 {
        Vec2::new(self.position.x, self.position.y)
    }

    pub fn rotation_2d(&self) -> f32 {
        self.rotation.z
    }

    pub fn scale_xy(&self) -> Vec2 {
        Vec2::new(self.scale.x, self.scale.y)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Transform {
    fn type_name() -> &'static str {
        "Transform"
    }
}

/// 全局变换（由系统计算）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform(pub Transform);

impl Default for GlobalTransform {
    fn default() -> Self {
        Self(Transform::new())
    }
}

impl Component for GlobalTransform {
    fn type_name() -> &'static str {
        "GlobalTransform"
    }
}

/// Node 组件 - 组合变换和边界框
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node3D {
    pub local: Transform,
    pub global: Transform,
    pub size: Vec3,
}

impl Node3D {
    pub fn new() -> Self {
        Self {
            local: Transform::new(),
            global: Transform::new(),
            size: Vec3::ZERO,
        }
    }

    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            local: Transform::from_position(x, y, z),
            global: Transform::from_position(x, y, z),
            size: Vec3::ZERO,
        }
    }

    pub fn from_2d(x: f32, y: f32) -> Self {
        Self {
            local: Transform::from_2d(x, y),
            global: Transform::from_2d(x, y),
            size: Vec3::ZERO,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32, depth: f32) -> Self {
        self.size = Vec3::new(width, height, depth);
        self
    }

    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.local.position = Vec3::new(x, y, z);
    }

    pub fn set_rotation(&mut self, x: f32, y: f32, z: f32) {
        self.local.rotation = Vec3::new(x, y, z);
    }

    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) {
        self.local.scale = Vec3::new(x, y, z);
    }

    pub fn update_global(&mut self, parent_global: &Transform) {
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

    pub fn project_to_2d(&self, camera_position: Vec3, screen_size: Vec2) -> Vec2 {
        let relative_pos = self.global.position - camera_position;
        let distance = relative_pos.z.max(0.1);
        
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

/// Camera Component - Attach to entity for multi-camera support
///
/// Note: For the main camera, use `resources::Camera` as a Resource.
/// This component is for entities that represent camera objects in the scene.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraComponent {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub orthographic: bool,
    pub ortho_size: f32,
    pub background_color: crate::math::Color,
    pub viewport: Rect,
    pub depth: i32,
    pub culling_mask: u32,
}

/// Camera3D is an alias for CameraComponent
pub type Camera3D = CameraComponent;

/// Deprecated: Use CameraComponent instead
#[deprecated(since = "2.3.0", note = "Use CameraComponent instead to avoid confusion with resources::Camera")]
pub type Camera = CameraComponent;

impl CameraComponent {
    pub fn new() -> Self {
        Self {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
            orthographic: false,
            ortho_size: 1.0,
            background_color: crate::math::Color::BLACK,
            viewport: Rect::new(0.0, 0.0, 1.0, 1.0),
            depth: 0,
            culling_mask: 0xFFFFFFFF,
        }
    }

    pub fn perspective(fov: f32) -> Self {
        Self::new().with_fov(fov)
    }

    pub fn orthographic(size: f32) -> Self {
        Self {
            orthographic: true,
            ortho_size: size,
            ..Self::new()
        }
    }

    pub fn ui_camera() -> Self {
        Self {
            orthographic: true,
            ortho_size: 1.0,
            near: -1000.0,
            far: 1000.0,
            ..Self::new()
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

    pub fn with_viewport(mut self, viewport: Rect) -> Self {
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

impl Default for CameraComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for CameraComponent {
    fn type_name() -> &'static str {
        "CameraComponent"
    }
}

/// 光源组件
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    pub light_type: LightType,
    pub color: crate::math::Color,
    pub intensity: f32,
    pub range: f32,
    pub spot_angle: f32,
}

/// Light3D 是 Light 的别名
pub type Light3D = Light;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    Directional,
    Point,
    Spot,
    Ambient,
}

impl Light {
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

    pub fn ambient() -> Self {
        Self {
            light_type: LightType::Ambient,
            color: crate::math::Color::WHITE,
            intensity: 0.3,
            range: 0.0,
            spot_angle: 0.0,
        }
    }
}

impl Component for Light {
    fn type_name() -> &'static str {
        "Light"
    }
}

/// 边界框组件
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub local: Rect,
    pub global: Rect,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            local: Rect::ZERO,
            global: Rect::ZERO,
        }
    }
}

impl Component for BoundingBox {
    fn type_name() -> &'static str {
        "BoundingBox"
    }
}
