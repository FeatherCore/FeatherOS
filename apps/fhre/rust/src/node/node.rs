//! Node 核心组件
//!
//! Node 是 FHRE 的最小单位，统一了游戏实体和 UI 控件的概念。
//! 采用 ECS 架构，扁平化设计，SOA (Structure of Arrays) 布局，提升缓存效率。
//!
//! # 设计理念
//!
//! 1. **ECS 架构**: 无父子关系，组件扁平存储
//! 2. **SOA 布局**: 数据连续存储，提升缓存命中率
//! 3. **批量处理**: 支持 SIMD 和 GPU 批量处理
//! 4. **统一抽象**: 游戏实体和 UI 控件使用相同的基础组件

use crate::Component;

/// Node 类型枚举
///
/// 定义了 FHRE 支持的所有节点类型，涵盖游戏和 UI 场景。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    // ==================== 游戏类型 ====================
    /// 2D 精灵
    Sprite2D,
    /// 3D 模型
    Model3D,
    /// 粒子系统
    ParticleSystem,
    /// 摄像机
    Camera,
    /// 光源
    Light,
    /// 触发器/碰撞体
    Trigger,
    /// 空节点（用于分组）
    Empty,

    // ==================== UI 类型 ====================
    /// 容器
    Container,
    /// 面板
    Panel,
    /// 按钮
    Button,
    /// 标签/文本
    Label,
    /// 图像
    Image,
    /// 文本输入框
    TextInput,
    /// 滑块
    Slider,
    /// 开关
    Switch,
    /// 进度条
    ProgressBar,
    /// 列表
    List,
    /// 滚动视图
    ScrollView,

    // ==================== 2.5D 混合类型 ====================
    /// 卡片（支持翻转效果）
    Card,
    /// 等角块
    IsoBlock,
    /// 3D UI 元素
    UI3D,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Empty
    }
}

/// Node 状态 - SOA 友好设计
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeState {
    /// 是否可见
    pub visible: bool,
    /// 是否启用交互
    pub interactive: bool,
    /// 是否启用布局
    pub use_layout: bool,
    /// 是否启用 3D 变换
    pub use_3d: bool,
    /// 是否脏（需要重绘）
    pub dirty: bool,
    /// 是否悬停
    pub hovered: bool,
    /// 是否按下
    pub pressed: bool,
    /// 是否聚焦
    pub focused: bool,
    /// 是否禁用
    pub disabled: bool,
    /// 是否选中
    pub checked: bool,
}

impl NodeState {
    /// 创建默认状态（全部启用）
    pub fn new() -> Self {
        Self {
            visible: true,
            interactive: true,
            use_layout: false,
            use_3d: false,
            dirty: true,
            hovered: false,
            pressed: false,
            focused: false,
            disabled: false,
            checked: false,
        }
    }

    /// 创建游戏实体状态
    pub fn game_entity() -> Self {
        Self {
            visible: true,
            interactive: true,
            use_layout: false,
            use_3d: false,
            dirty: true,
            hovered: false,
            pressed: false,
            focused: false,
            disabled: false,
            checked: false,
        }
    }

    /// 创建 UI 控件状态
    pub fn ui_control() -> Self {
        Self {
            visible: true,
            interactive: true,
            use_layout: true,
            use_3d: false,
            dirty: true,
            hovered: false,
            pressed: false,
            focused: false,
            disabled: false,
            checked: false,
        }
    }

    /// 检查是否可以交互
    pub fn can_interact(&self) -> bool {
        self.interactive && !self.disabled && self.visible
    }

    /// 标记为脏（需要重绘）
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// 清除脏标记
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

/// Node 标志位 - SOA 友好设计
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeFlags {
    /// 是否接收输入事件
    pub receive_input: bool,
    /// 是否参与布局计算
    pub participate_layout: bool,
    /// 是否参与渲染
    pub participate_render: bool,
    /// 是否参与碰撞检测
    pub participate_collision: bool,
    /// 是否保持比例
    pub keep_aspect_ratio: bool,
    /// 是否裁剪子节点
    pub clip_children: bool,
    /// 是否忽略父节点的变换
    pub ignore_parent_transform: bool,
    /// 是否启用滚动
    pub scrollable: bool,
}

impl NodeFlags {
    /// 创建默认标志
    pub fn new() -> Self {
        Self {
            receive_input: true,
            participate_layout: true,
            participate_render: true,
            participate_collision: false,
            keep_aspect_ratio: false,
            clip_children: false,
            ignore_parent_transform: false,
            scrollable: false,
        }
    }

    /// 游戏实体默认标志
    pub fn game_entity() -> Self {
        Self {
            receive_input: true,
            participate_layout: false,
            participate_render: true,
            participate_collision: true,
            keep_aspect_ratio: false,
            clip_children: false,
            ignore_parent_transform: false,
            scrollable: false,
        }
    }

    /// UI 控件默认标志
    pub fn ui_control() -> Self {
        Self {
            receive_input: true,
            participate_layout: true,
            participate_render: true,
            participate_collision: false,
            keep_aspect_ratio: false,
            clip_children: false,
            ignore_parent_transform: false,
            scrollable: false,
        }
    }

    /// 容器默认标志
    pub fn container() -> Self {
        Self {
            receive_input: false,
            participate_layout: true,
            participate_render: true,
            participate_collision: false,
            keep_aspect_ratio: false,
            clip_children: true,
            ignore_parent_transform: false,
            scrollable: true,
        }
    }
}

/// Node 组件 - FHRE 的最小单位
///
/// 这是 FHRE 的核心组件，所有可渲染、可交互的对象都应该包含此组件。
/// 采用 ECS 架构，无父子关系，数据扁平存储。
#[derive(Debug, Clone)]
pub struct Node {
    /// 节点类型
    pub node_type: NodeType,
    /// 节点状态
    pub state: NodeState,
    /// 节点标志
    pub flags: NodeFlags,
    /// 节点层级（用于渲染排序）
    pub z_order: i32,
    /// 透明度 (0.0 - 1.0)
    pub opacity: f32,
}

impl Node {
    /// 创建新的 Node
    pub fn new(node_type: NodeType) -> Self {
        Self {
            node_type,
            state: NodeState::new(),
            flags: NodeFlags::new(),
            z_order: 0,
            opacity: 1.0,
        }
    }

    /// 创建游戏实体类型的 Node
    pub fn game_entity(node_type: NodeType) -> Self {
        Self {
            node_type,
            state: NodeState::game_entity(),
            flags: NodeFlags::game_entity(),
            z_order: 0,
            opacity: 1.0,
        }
    }

    /// 创建 UI 控件类型的 Node
    pub fn ui_control(node_type: NodeType) -> Self {
        Self {
            node_type,
            state: NodeState::ui_control(),
            flags: NodeFlags::ui_control(),
            z_order: 0,
            opacity: 1.0,
        }
    }

    /// 创建容器类型的 Node
    pub fn container() -> Self {
        Self {
            node_type: NodeType::Container,
            state: NodeState::ui_control(),
            flags: NodeFlags::container(),
            z_order: 0,
            opacity: 1.0,
        }
    }

    /// 设置 z_order
    pub fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    /// 设置透明度
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// 设置是否可见
    pub fn set_visible(&mut self, visible: bool) {
        if self.state.visible != visible {
            self.state.visible = visible;
            self.state.mark_dirty();
        }
    }

    /// 设置是否启用交互
    pub fn set_interactive(&mut self, interactive: bool) {
        self.state.interactive = interactive;
    }

    /// 设置是否禁用
    pub fn set_disabled(&mut self, disabled: bool) {
        self.state.disabled = disabled;
    }

    /// 检查是否是游戏类型
    pub fn is_game_type(&self) -> bool {
        matches!(
            self.node_type,
            NodeType::Sprite2D
                | NodeType::Model3D
                | NodeType::ParticleSystem
                | NodeType::Camera
                | NodeType::Light
                | NodeType::Trigger
                | NodeType::Empty
        )
    }

    /// 检查是否是 UI 类型
    pub fn is_ui_type(&self) -> bool {
        matches!(
            self.node_type,
            NodeType::Container
                | NodeType::Panel
                | NodeType::Button
                | NodeType::Label
                | NodeType::Image
                | NodeType::TextInput
                | NodeType::Slider
                | NodeType::Switch
                | NodeType::ProgressBar
                | NodeType::List
                | NodeType::ScrollView
        )
    }

    /// 检查是否是 2.5D 混合类型
    pub fn is_mixed_type(&self) -> bool {
        matches!(
            self.node_type,
            NodeType::Card | NodeType::IsoBlock | NodeType::UI3D
        )
    }

    /// 检查是否需要 3D 变换
    pub fn needs_3d_transform(&self) -> bool {
        self.state.use_3d
            || matches!(
                self.node_type,
                NodeType::Model3D | NodeType::ParticleSystem | NodeType::UI3D
            )
    }

    /// 检查是否需要布局计算
    pub fn needs_layout(&self) -> bool {
        self.state.use_layout || self.flags.participate_layout
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new(NodeType::Empty)
    }
}

impl Component for Node {
    fn type_name() -> &'static str {
        "Node"
    }
}

impl Component for NodeState {
    fn type_name() -> &'static str {
        "NodeState"
    }
}

impl Component for NodeFlags {
    fn type_name() -> &'static str {
        "NodeFlags"
    }
}

/// Node 批量状态组件 - 用于 SOA 批量处理
///
/// 将 Node 的状态数据分离，便于批量处理和缓存优化
#[derive(Debug, Clone, Default)]
pub struct NodeStateBatch {
    /// 可见性数组
    pub visible: alloc::vec::Vec<bool>,
    /// 交互性数组
    pub interactive: alloc::vec::Vec<bool>,
    /// 脏标记数组
    pub dirty: alloc::vec::Vec<bool>,
    /// 悬停状态数组
    pub hovered: alloc::vec::Vec<bool>,
    /// 按下状态数组
    pub pressed: alloc::vec::Vec<bool>,
}

impl NodeStateBatch {
    /// 创建新的批量状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加节点状态
    pub fn add(&mut self, state: &NodeState) {
        self.visible.push(state.visible);
        self.interactive.push(state.interactive);
        self.dirty.push(state.dirty);
        self.hovered.push(state.hovered);
        self.pressed.push(state.pressed);
    }

    /// 获取数量
    pub fn count(&self) -> usize {
        self.visible.len()
    }

    /// 批量标记为脏
    pub fn mark_all_dirty(&mut self) {
        for dirty in &mut self.dirty {
            *dirty = true;
        }
    }

    /// 批量清除脏标记
    pub fn clear_all_dirty(&mut self) {
        for dirty in &mut self.dirty {
            *dirty = false;
        }
    }
}

/// Node 批量变换组件 - 用于 SOA 批量处理
#[derive(Debug, Clone, Default)]
pub struct NodeTransformBatch {
    /// 位置数组 (x, y, z)
    pub positions: alloc::vec::Vec<(f32, f32, f32)>,
    /// 旋转数组 (x, y, z)
    pub rotations: alloc::vec::Vec<(f32, f32, f32)>,
    /// 缩放数组 (x, y, z)
    pub scales: alloc::vec::Vec<(f32, f32, f32)>,
}

impl NodeTransformBatch {
    /// 创建新的批量变换
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加 2D 变换
    pub fn add_2d(&mut self, x: f32, y: f32, rotation: f32, scale_x: f32, scale_y: f32) {
        self.positions.push((x, y, 0.0));
        self.rotations.push((0.0, 0.0, rotation));
        self.scales.push((scale_x, scale_y, 1.0));
    }

    /// 添加 3D 变换
    pub fn add_3d(
        &mut self,
        x: f32,
        y: f32,
        z: f32,
        rot_x: f32,
        rot_y: f32,
        rot_z: f32,
        scale_x: f32,
        scale_y: f32,
        scale_z: f32,
    ) {
        self.positions.push((x, y, z));
        self.rotations.push((rot_x, rot_y, rot_z));
        self.scales.push((scale_x, scale_y, scale_z));
    }

    /// 获取数量
    pub fn count(&self) -> usize {
        self.positions.len()
    }

    /// 批量平移
    pub fn translate_all(&mut self, dx: f32, dy: f32, dz: f32) {
        for pos in &mut self.positions {
            pos.0 += dx;
            pos.1 += dy;
            pos.2 += dz;
        }
    }

    /// 批量缩放
    pub fn scale_all(&mut self, factor: f32) {
        for scale in &mut self.scales {
            scale.0 *= factor;
            scale.1 *= factor;
            scale.2 *= factor;
        }
    }
}
