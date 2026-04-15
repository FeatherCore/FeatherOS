//! Node2D 组件
//!
//! 提供 2D 变换和属性，用于 2D 游戏对象和 UI 控件。

use crate::Component;
use crate::math::{Vec2, Rect};

/// 2D 变换组件
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    /// 2D 位置 (x, y)
    pub position: Vec2,
    /// 2D 旋转（弧度）
    pub rotation: f32,
    /// 2D 缩放
    pub scale: Vec2,
    /// 锚点（相对于自身的偏移，默认中心）
    pub anchor: Vec2,
}

impl Transform2D {
    /// 创建默认变换
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
            anchor: Vec2::ZERO, // 中心点
        }
    }

    /// 从位置创建
    pub fn from_position(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            rotation: 0.0,
            scale: Vec2::ONE,
            anchor: Vec2::ZERO,
        }
    }

    /// 从 Vec2 位置创建
    pub fn from_vec2(position: Vec2) -> Self {
        Self {
            position,
            rotation: 0.0,
            scale: Vec2::ONE,
            anchor: Vec2::ZERO,
        }
    }

    /// 设置位置
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Vec2::new(x, y);
        self
    }

    /// 设置旋转
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置缩放
    pub fn with_scale(mut self, x: f32, y: f32) -> Self {
        self.scale = Vec2::new(x, y);
        self
    }

    /// 设置锚点
    pub fn with_anchor(mut self, x: f32, y: f32) -> Self {
        self.anchor = Vec2::new(x, y);
        self
    }

    /// 预定义锚点：中心
    pub fn anchor_center() -> Vec2 {
        Vec2::ZERO
    }

    /// 预定义锚点：左上角
    pub fn anchor_top_left() -> Vec2 {
        Vec2::new(-0.5, 0.5)
    }

    /// 预定义锚点：右上角
    pub fn anchor_top_right() -> Vec2 {
        Vec2::new(0.5, 0.5)
    }

    /// 预定义锚点：左下角
    pub fn anchor_bottom_left() -> Vec2 {
        Vec2::new(-0.5, -0.5)
    }

    /// 预定义锚点：右下角
    pub fn anchor_bottom_right() -> Vec2 {
        Vec2::new(0.5, -0.5)
    }

    /// 获取变换矩阵（用于渲染）
    pub fn to_matrix(&self) -> [[f32; 3]; 3] {
        let cos = libm::cosf(self.rotation);
        let sin = libm::sinf(self.rotation);

        // 2D 变换矩阵 (3x3 齐次坐标)
        [
            [cos * self.scale.x, -sin * self.scale.y, self.position.x],
            [sin * self.scale.x, cos * self.scale.y, self.position.y],
            [0.0, 0.0, 1.0],
        ]
    }

    /// 平移
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.position.x += dx;
        self.position.y += dy;
    }

    /// 旋转
    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
    }

    /// 缩放
    pub fn scale_by(&mut self, factor: f32) {
        self.scale.x *= factor;
        self.scale.y *= factor;
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Transform2D {
    fn type_name() -> &'static str {
        "Transform2D"
    }
}

/// Node2D 组件
///
/// 组合了 2D 变换和边界框信息
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node2D {
    /// 本地变换
    pub local: Transform2D,
    /// 全局变换（计算后）
    pub global: Transform2D,
    /// 边界框（本地坐标）
    pub local_bounds: Rect,
    /// 边界框（全局坐标，计算后）
    pub global_bounds: Rect,
    /// 尺寸
    pub size: Vec2,
}

impl Node2D {
    /// 创建新的 Node2D
    pub fn new() -> Self {
        Self {
            local: Transform2D::new(),
            global: Transform2D::new(),
            local_bounds: Rect::ZERO,
            global_bounds: Rect::ZERO,
            size: Vec2::ZERO,
        }
    }

    /// 从位置创建
    pub fn from_position(x: f32, y: f32) -> Self {
        Self {
            local: Transform2D::from_position(x, y),
            global: Transform2D::from_position(x, y),
            local_bounds: Rect::ZERO,
            global_bounds: Rect::ZERO,
            size: Vec2::ZERO,
        }
    }

    /// 从位置和尺寸创建
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self.local_bounds = Rect::from_center_size(Vec2::ZERO, self.size);
        self
    }

    /// 设置本地位置
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.local.position = Vec2::new(x, y);
    }

    /// 设置本地旋转
    pub fn set_rotation(&mut self, rotation: f32) {
        self.local.rotation = rotation;
    }

    /// 设置本地缩放
    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.local.scale = Vec2::new(x, y);
    }

    /// 更新全局变换（由系统调用）
    pub fn update_global(&mut self, parent_global: &Transform2D) {
        // 组合父变换和本地变换
        self.global.position = parent_global.position + self.local.position;
        self.global.rotation = parent_global.rotation + self.local.rotation;
        self.global.scale = Vec2::new(
            parent_global.scale.x * self.local.scale.x,
            parent_global.scale.y * self.local.scale.y,
        );
        self.global.anchor = self.local.anchor;

        // 更新全局边界框
        self.update_global_bounds();
    }

    /// 更新全局边界框
    fn update_global_bounds(&mut self) {
        // 根据全局变换计算边界框
        let half_size = Vec2::new(
            self.size.x * self.global.scale.x * 0.5,
            self.size.y * self.global.scale.y * 0.5,
        );

        self.global_bounds = Rect::from_center_size(self.global.position, half_size * 2.0);
    }

    /// 检查点是否在节点内
    pub fn contains_point(&self, point: Vec2) -> bool {
        self.global_bounds.contains(point)
    }

    /// 检查与另一个节点的碰撞
    pub fn intersects(&self, other: &Node2D) -> bool {
        self.global_bounds.intersects(&other.global_bounds)
    }
}

impl Default for Node2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Node2D {
    fn type_name() -> &'static str {
        "Node2D"
    }
}

/// 全局变换 2D（由系统计算）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform2D(pub Transform2D);

impl Default for GlobalTransform2D {
    fn default() -> Self {
        Self(Transform2D::new())
    }
}

impl Component for GlobalTransform2D {
    fn type_name() -> &'static str {
        "GlobalTransform2D"
    }
}

/// 边界框组件
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox2D {
    /// 本地边界框
    pub local: Rect,
    /// 全局边界框（计算后）
    pub global: Rect,
}

impl Default for BoundingBox2D {
    fn default() -> Self {
        Self {
            local: Rect::ZERO,
            global: Rect::ZERO,
        }
    }
}

impl Component for BoundingBox2D {
    fn type_name() -> &'static str {
        "BoundingBox2D"
    }
}
