//! Node 模块 - FHRE 的最小单位系统
//!
//! Node 是 FHRE 的统一最小单位，既可以作为游戏引擎的实体，也可以作为 UI 控件。
//! 采用 ECS 架构，扁平化设计，SOA (Structure of Arrays) 布局，提升缓存效率。
//!
//! # 设计理念
//!
//! 1. **ECS 架构**: 无父子关系，组件扁平存储，符合 Bevy 双世界设计
//! 2. **SOA 布局**: 数据连续存储，提升缓存命中率，支持 SIMD 批量处理
//! 3. **统一抽象**: Node 既可以表示 2D 游戏精灵，也可以表示 UI 按钮
//! 4. **2.5D 支持**: 默认 2D 模式，需要时可启用 3D 变换
//!
//! # 使用示例
//!
//! ```rust
//! // 创建 2D 游戏精灵
//! commands.spawn((
//!     Node::game_entity(NodeType::Sprite2D),
//!     Node2D::from_position(100.0, 200.0),
//!     Sprite::from_image(image_handle),
//! ));
//!
//! // 创建 UI 按钮
//! commands.spawn((
//!     Node::ui_control(NodeType::Button),
//!     Node2D::from_position(50.0, 50.0),
//!     Style::ui_default(),
//! ));
//!
//! // 创建 3D 模型
//! commands.spawn((
//!     Node::game_entity(NodeType::Model3D),
//!     Node3D::from_position(0.0, 0.0, 10.0),
//!     Model::from_mesh(mesh_handle),
//! ));
//! ```

pub mod node;
pub mod node2d;
pub mod node3d;
pub mod style;
pub mod layout;

pub use node::{Node, NodeType, NodeState, NodeFlags, NodeStateBatch, NodeTransformBatch};
pub use node2d::{Node2D, Transform2D, GlobalTransform2D, BoundingBox2D};
pub use node3d::{Node3D, Transform3D, GlobalTransform3D, Camera3D, Light3D, LightType};
pub use style::{Style, Dimension, Rect, Border, Shadow, TextStyle, TextAlignment, Background, Gradient, ImageBackground, ImageScaling, ImageRepeat};
pub use layout::{Layout, LayoutType, LayoutResult, FlexDirection, JustifyContent, AlignItems, FlexWrap, ListDirection};
