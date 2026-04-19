//! Node 模块 - FHRE 的最小单位系统
//!
//! Node 是 FHRE 的统一最小单位，既可以作为游戏引擎的实体，也可以作为 UI 控件。
//! 采用 ECS 架构，扁平化设计，SOA (Structure of Arrays) 布局，提升缓存效率。
//!
//! # 设计理念
//!
//! 1. **纯 3D 引擎**: 所有实体都在 3D 空间中，2D 只是 z=0 平面的特例
//! 2. **ECS 架构**: 无父子关系，组件扁平存储，符合 Bevy 双世界设计
//! 3. **SOA 布局**: 数据连续存储，提升缓存命中率，支持 SIMD 批量处理
//! 4. **统一抽象**: Node 既可以表示游戏精灵，也可以表示 UI 按钮
//!
//! # 使用示例
//!
//! ```rust
//! // 创建游戏精灵 (2D = 3D 的 z=0 特例)
//! commands.spawn((
//!     Node::game_entity(NodeType::Sprite),
//!     Transform::from_2d(100.0, 200.0),
//!     Sprite::from_image(image_handle),
//! ));
//!
//! // 创建 UI 按钮
//! commands.spawn((
//!     Node::ui_control(NodeType::Button),
//!     Transform::from_2d(50.0, 50.0),
//!     Style::ui_default(),
//! ));
//!
//! // 创建 3D 模型
//! commands.spawn((
//!     Node::game_entity(NodeType::Model),
//!     Transform::from_position(0.0, 0.0, 10.0),
//!     Model::from_mesh(mesh_handle),
//! ));
//! ```

pub mod node;
pub mod node3d;
pub mod style;
pub mod layout;

pub use node::{Node, NodeType, NodeState, NodeFlags, NodeStateBatch, NodeTransformBatch};
pub use node3d::{
    Transform, Transform3D, GlobalTransform,
    Node3D, Camera, Camera3D, Light, Light3D, LightType, BoundingBox,
};
pub use style::{Style, Dimension, Rect, Border, Shadow, TextStyle, TextAlignment, Background, Gradient, ImageBackground, ImageScaling, ImageRepeat};
pub use layout::{Layout, LayoutType, LayoutResult, FlexDirection, JustifyContent, AlignItems, FlexWrap, ListDirection};
