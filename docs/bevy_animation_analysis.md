# Bevy 动画系统深度分析

## 1. 概述

Bevy 的动画系统是一个基于 ECS（Entity-Component-System）架构的现代动画引擎，提供了灵活且高性能的动画播放、混合和过渡功能。

### 核心特性

- **基于曲线的动画**：使用 `AnimationCurve`  trait 定义任意属性的动画曲线
- **动画图（Animation Graph）**：支持复杂的动画混合和层级结构
- **动画过渡**：平滑的动画切换和淡入淡出
- **动画重定向**：通过 `AnimationTargetId` 实现骨骼动画的复用
- **动画事件**：在动画播放过程中触发事件
- **蒙版系统**：控制动画影响的骨骼范围

---

## 2. 核心组件架构

### 2.1 动画剪辑 (AnimationClip)

`AnimationClip` 是动画数据的基本单位，包含一组动画曲线和事件。

```rust
#[derive(Asset, Reflect, Clone, Debug, Default)]
pub struct AnimationClip {
    #[reflect(ignore, clone)]
    curves: AnimationCurves,      // 动画曲线映射
    events: AnimationEvents,      // 动画事件
    duration: f32,                // 动画时长（秒）
}

// 动画曲线映射：AnimationTargetId -> Vec<VariableCurve>
pub type AnimationCurves = HashMap<AnimationTargetId, Vec<VariableCurve>, NoOpHash>;
```

**关键特性：**
- 通过 `AnimationTargetId`（UUID）引用动画目标，实现动画重定向
- 支持多个动画目标，每个目标可以有多个动画曲线
- 自动计算动画时长（基于曲线的最大时间范围）

### 2.2 动画目标标识 (AnimationTargetId)

```rust
#[derive(Clone, Copy, PartialEq, Eq, Reflect, Debug, Serialize, Deserialize, Component)]
pub struct AnimationTargetId(pub Uuid);
```

**设计原理：**
- 使用 UUID 作为骨骼/动画目标的唯一标识
- 基于骨骼完整路径名生成（如 `"Arm/Hand/Finger"`）
- 允许动画在不同骨骼结构间复用（只要路径名匹配）

### 2.3 动画播放器 (AnimationPlayer)

```rust
#[derive(Component, Default, Reflect)]
pub struct AnimationPlayer {
    active_animations: HashMap<AnimationNodeIndex, ActiveAnimation>,
}
```

**核心功能：**
- 管理多个同时播放的动画
- 每个动画通过 `AnimationNodeIndex` 在动画图中定位
- 提供播放控制 API（播放、暂停、停止、跳转）

### 2.4 活跃动画 (ActiveAnimation)

```rust
#[derive(Debug, Clone, Copy, Reflect)]
pub struct ActiveAnimation {
    weight: f32,                  // 动画权重（用于混合）
    repeat: RepeatAnimation,      // 重复模式
    speed: f32,                   // 播放速度
    elapsed: f32,                 // 已播放时间
    seek_time: f32,               // 当前时间点
    completions: u32,             // 完成次数
    paused: bool,                 // 是否暂停
}

pub enum RepeatAnimation {
    Never,      // 播放一次
    Count(u32), // 播放指定次数
    Forever,    // 循环播放
}
```

---

## 3. 动画曲线系统

### 3.1 动画曲线 trait

```rust
/// 可动画属性 trait
pub trait Animatable: Reflect + Sized + Send + Sync + 'static {
    /// 插值函数
    fn interpolate(a: &Self, b: &Self, time: f32) -> Self;
    
    /// 混合多个值
    fn blend(inputs: impl Iterator<Item = BlendInput<Self>>) -> Self;
}

/// 混合输入
pub struct BlendInput<T> {
    pub weight: f32,      // 权重
    pub value: T,         // 值
    pub additive: bool,   // 是否加法混合
}
```

### 3.2 内置可动画类型

Bevy 为以下类型实现了 `Animatable`：

| 类型 | 插值方式 | 混合方式 |
|------|----------|----------|
| `f32`/`f64` | 线性插值 | 加权平均/加法 |
| `Vec2`/`Vec3`/`Vec4` | 线性插值 | 加权平均/加法 |
| `Quat` | 球面线性插值(SLERP) | 加权混合 |
| `Transform` | 分量插值 | 分量混合 |
| 颜色类型 | 线性插值 | 加权混合 |
| `bool` | 步进插值 | 最大权重值 |

### 3.3 动画曲线适配器

```rust
/// 将 Curve<T> 转换为 AnimationCurve
pub struct AnimatableCurve<P: AnimatableProperty, C: Curve<P::Property>> {
    property: P,
    curve: C,
}

/// 属性动画字段宏
let curve = AnimatableCurve::new(
    animated_field!(Transform::translation),
    wobble_curve
);
```

---

## 4. 动画图 (Animation Graph)

### 4.1 图结构

动画图是一个有向无环图（DAG），定义动画如何混合：

```rust
#[derive(Asset, Reflect, Clone, Debug)]
pub struct AnimationGraph {
    pub graph: AnimationDiGraph,           // petgraph 图结构
    pub root: NodeIndex,                   // 根节点索引
    pub mask_groups: HashMap<AnimationTargetId, AnimationMask>,
}

pub type AnimationDiGraph = DiGraph<AnimationGraphNode, (), u32>;
```

### 4.2 节点类型

```rust
#[derive(Clone, Default, Reflect, Debug)]
pub enum AnimationNodeType {
    /// 剪辑节点 - 播放动画剪辑（叶子节点）
    Clip(Handle<AnimationClip>),
    
    /// 混合节点 - 归一化混合子节点
    #[default]
    Blend,
    
    /// 加法混合节点 - 非归一化加法混合
    Add,
}

pub struct AnimationGraphNode {
    pub node_type: AnimationNodeType,
    pub mask: AnimationMask,      // 蒙版位域
    pub weight: f32,              // 节点权重
}
```

### 4.3 图评估流程

动画图采用**后序遍历**（Post-order）评估：

```
示例图结构：
         ┌─────┐
         │Root │
         └──┬──┘
            │
    ┌───────┼───────┐
    │       │       │
    ▼       ▼       ▼
┌─────┐ ┌─────┐ ┌─────┐
│Idle │ │Blend│ │Walk │
└─────┘ └──┬──┘ └─────┘
           │
      ┌────┴────┐
      │         │
      ▼         ▼
  ┌─────┐   ┌─────┐
  │ Run │   │Walk │
  └─────┘   └─────┘

评估顺序（后序）：Run → Walk → Blend → Idle → Root
```

**评估算法：**
1. 使用栈结构存储中间结果
2. 叶子节点（Clip）直接压栈
3. 混合节点（Blend/Add）弹出子节点结果，混合后压栈
4. 根节点结果即为最终动画姿势

### 4.4 蒙版系统

```rust
pub type AnimationMask = u64;  // 64位蒙版位域

/// 蒙版组映射：AnimationTargetId -> 蒙版位域
pub mask_groups: HashMap<AnimationTargetId, AnimationMask>
```

**使用场景：**
- 角色手持物品时屏蔽手部动画
- 上半身/下半身分离动画（如跑步时射击）
- 局部动画叠加

---

## 5. 动画过渡系统

### 5.1 过渡组件

```rust
#[derive(Component, Default, Reflect)]
pub struct AnimationTransitions {
    main_animation: Option<AnimationNodeIndex>,
    transitions: Vec<AnimationTransition>,
}

pub struct AnimationTransition {
    current_weight: f32,          // 当前权重（1.0 → 0.0）
    weight_decline_per_sec: f32,  // 权重衰减速度
    animation: AnimationNodeIndex,// 正在淡出的动画
}
```

### 5.2 过渡机制

**"贪婪层"系统：**
1. 最新过渡获得所需权重
2. 剩余权重分配给旧过渡
3. 当前播放动画获得最后剩余的权重
4. 确保权重总和始终归一化

```rust
pub fn advance_transitions(
    mut query: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
    time: Res<Time>,
) {
    for (mut transitions, mut player) in query.iter_mut() {
        let mut remaining_weight = 1.0;
        
        // 从最新到最旧处理过渡
        for transition in transitions.transitions.iter_mut().rev() {
            transition.current_weight -= 
                transition.weight_decline_per_sec * time.delta_secs();
            transition.current_weight = transition.current_weight.max(0.0);
            
            if let Some(animation) = player.animation_mut(transition.animation) {
                animation.weight = transition.current_weight * remaining_weight;
                remaining_weight -= animation.weight;
            }
        }
        
        // 主动画获得剩余权重
        if let Some(main) = transitions.main_animation {
            if let Some(animation) = player.animation_mut(main) {
                animation.weight = remaining_weight;
            }
        }
    }
}
```

---

## 6. 动画事件系统

### 6.1 事件类型

```rust
pub struct TimedAnimationEvent {
    time: f32,
    event: AnimationEventData,
}

pub struct AnimationEventData {
    trigger: AnimationEventFn,  // Arc<dyn Fn(...)>
}

pub enum AnimationEventTarget {
    Root,                       // 触发到 AnimationPlayer 实体
    Node(AnimationTargetId),    // 触发到指定目标实体
}
```

### 6.2 事件触发

```rust
impl AnimationClip {
    /// 添加事件到根实体
    pub fn add_event(&mut self, time: f32, event: impl AnimationEvent);
    
    /// 添加事件到指定目标
    pub fn add_event_to_target(
        &mut self, 
        target_id: AnimationTargetId, 
        time: f32, 
        event: impl AnimationEvent
    );
    
    /// 添加自定义事件函数
    pub fn add_event_fn(
        &mut self, 
        time: f32, 
        func: impl Fn(&mut Commands, Entity, f32, f32) + Send + Sync + 'static
    );
}
```

---

## 7. 系统执行流程

### 7.1 动画系统注册

```rust
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<AnimationClip>()
            .init_asset::<AnimationGraph>()
            .register_type::<AnimationPlayer>()
            .add_systems(
                PostUpdate,
                (
                    advance_transitions,           // 更新过渡
                    animate_targets,               // 评估动画
                    expire_completed_transitions,  // 清理已完成过渡
                )
                    .chain()
                    .in_set(AnimationSystems::Animation),
            );
    }
}
```

### 7.2 动画评估流程

```
┌─────────────────────────────────────────────────────────────────┐
│                     动画评估流程                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. advance_transitions                                         │
│     ├── 更新过渡权重                                            │
│     └── 分配动画权重                                            │
│                          ↓                                      │
│  2. animate_targets                                             │
│     ├── 遍历所有 AnimationPlayer                                │
│     ├── 对每个活跃动画：                                         │
│     │   ├── 更新 seek_time                                      │
│     │   ├── 触发动画事件                                        │
│     │   └── 评估动画图                                          │
│     │       ├── 后序遍历图节点                                   │
│     │       ├── Clip节点：采样曲线                              │
│     │       └── Blend/Add节点：混合子节点结果                    │
│     └── 应用最终值到组件                                         │
│                          ↓                                      │
│  3. expire_completed_transitions                                │
│     └── 移除已完成的过渡                                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. 使用示例

### 8.1 基础动画播放

```rust
// 播放单个动画
fn play_animation(
    mut players: Query<&mut AnimationPlayer>,
    animations: Res<Assets<AnimationClip>>,
) {
    for mut player in &mut players {
        player.play(animations.get_handle("walk"));
    }
}
```

### 8.2 使用动画图

```rust
// 创建动画图
let mut graph = AnimationGraph::new();
let idle = graph.add_clip(idle_clip, 1.0, graph.root);
let walk = graph.add_clip(walk_clip, 1.0, graph.root);
let run = graph.add_clip(run_clip, 1.0, graph.root);

// 添加混合节点
let blend = graph.add_blend(graph.root);
graph.add_clip(walk_clip, 0.5, blend);
graph.add_clip(run_clip, 0.5, blend);

// 应用到实体
commands.entity(player_entity)
    .insert(AnimationGraphHandle(graphs.add(graph)))
    .insert(AnimationPlayer::default());
```

### 8.3 动画过渡

```rust
// 使用 AnimationTransitions 组件
fn transition_to_animation(
    mut query: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
) {
    for (mut transitions, mut player) in &mut query {
        // 播放新动画，自动淡出旧动画
        transitions.play(
            &mut player,
            animation_node_index,
            Duration::from_millis(300),  // 300ms 过渡时间
        );
    }
}
```

### 8.4 自定义动画曲线

```rust
use bevy_animation::{animated_field, animation_curves::*};
use bevy_math::curve::{Curve, Interval, FunctionCurve};

// 创建自定义曲线
let wobble_curve = FunctionCurve::new(
    Interval::UNIT,
    |t| vec3(t.cos() * 0.1, 0.0, 0.0),
);

// 创建动画曲线
let animation = AnimatableCurve::new(
    animated_field!(Transform::translation),
    wobble_curve,
);

// 添加到动画剪辑
let mut clip = AnimationClip::default();
clip.add_curve_to_target(target_id, animation);
```

---

## 9. 架构设计亮点

### 9.1 数据驱动设计

- **AnimationClip 作为 Asset**：支持异步加载和缓存
- **动画图资源化**：`.animgraph.ron` 文件定义复杂混合逻辑
- **UUID 标识系统**：实现动画重定向和复用

### 9.2 ECS 集成

- **组件化设计**：`AnimationPlayer`、`AnimationTransitions`、`AnimatedBy` 等组件
- **系统链**：`advance_transitions` → `animate_targets` → `expire_completed_transitions`
- **事件系统**：动画事件通过 ECS 事件系统触发

### 9.3 性能优化

- **ThreadedAnimationGraph**：预计算图遍历顺序，避免运行时排序
- **AnimationCurveEvaluator 缓存**：复用曲线评估器，减少内存分配
- **后序遍历评估**：使用栈结构实现高效图评估

### 9.4 扩展性

- **Animatable trait**：支持任意类型的动画
- **AnimationCurve trait**：支持自定义曲线类型
- **AnimatableProperty**：支持动画任意实体属性

---

## 10. 与其他动画系统对比

| 特性 | Bevy Animation | Unity Mecanim | Unreal Animation |
|------|----------------|---------------|------------------|
| 架构 | ECS-based | Component-based | Blueprint-based |
| 混合图 | DAG | 状态机 + 混合树 | 状态机 + 混合空间 |
| 蒙版 | 位域蒙版 | Avatar Mask | Bone Filter |
| 事件 | 函数回调 | Animation Event | Anim Notifies |
| 重定向 | UUID 路径匹配 | Avatar | Retargeting |
| 运行时创建 | 完全支持 | 有限支持 | 有限支持 |

---

## 11. 总结

Bevy 的动画系统采用现代化的 ECS 架构，提供了：

1. **灵活的动画曲线系统**：支持任意属性的动画
2. **强大的动画图**：DAG 结构支持复杂混合逻辑
3. **平滑的过渡机制**："贪婪层"系统确保过渡自然
4. **高效的事件系统**：支持精确的动画事件触发
5. **完善的蒙版系统**：支持局部动画控制

该系统的设计充分考虑了性能、扩展性和易用性，适合从简单 2D 动画到复杂 3D 角色动画的各种场景。
