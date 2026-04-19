# FHRE 架构文档

## 概述

FHRE (Feather Hybrid Render Engine) 是一个轻量级的纯 3D 渲染引擎，设计用于嵌入式系统 (no_std)。采用 Bevy 风格的双世界 ECS 架构，2D 只是 3D 的特例（z=0 平面上的对象）。

**版本**: 3.0.0  
**目标平台**: 嵌入式系统 (NuttX RTOS)  
**设计理念**: 纯 3D 引擎，ECS 架构，双世界同步，组件自管理交互

## 核心概念

### 纯 3D 引擎

FHRE 是纯 3D 引擎，所有实体都在 3D 空间中：
- 2D 只是 3D 的特例：z=0 平面上的对象
- 通过正交相机投影到屏幕
- 统一使用 `Transform` 作为唯一变换组件
- `Transform3D` 作为别名保持兼容
- NodeType 不再区分 2D/3D（如 Sprite2D → Sprite, Model3D → Model）

### Picking 系统 + 按钮交互

FHRE 使用 Picking 系统检测指针与 UI 元素的交互，Button 组件管理交互状态：

```rust
pub struct Button {
    // 渲染属性
    pub width: f32,
    pub height: f32,
    pub state: ButtonState,  // Normal, Hover, Pressed, Disabled
    
    // 交互状态（由 button_interaction_system 更新）
    pub clicked: bool,  // 本帧是否被点击
}
```

**交互流程**：
1. `picking_system` 检测鼠标位置与 `PickableBounds` 的碰撞
2. `update_hover_map` 更新 `HoverMap` 和 `PreviousHoverMap`
3. `button_interaction_system` 根据悬停状态更新 `Button.state` 和 `Button.clicked`
4. 用户系统检查 `button.clicked` 执行操作

**关键组件**：
- `PickableBounds`: 可点击区域 (width, height)
- `Pickable`: 是否可悬停、是否阻挡下层（必须使用 `Pickable::DEFAULT`，而非 `default()`）
- `HoverMap` / `PreviousHoverMap`: 当前帧/上一帧悬停的实体

## 目录结构

```
fhre/rust/src/
├── lib.rs                    # 库入口，全局分配器，panic handler
├── app/
│   └── app.rs                # App 结构，插件注册，update_and_render
├── main_world/               # 主世界 (游戏逻辑)
│   ├── mod.rs
│   ├── world.rs              # MainWorld ECS
│   ├── entity.rs             # Entity ID
│   ├── component.rs          # Component trait
│   ├── system.rs             # System trait, declare_system! 宏
│   ├── system_param.rs       # Res, ResMut, Query, Local
│   ├── commands.rs           # Commands, EntityCommands
│   ├── change_detection.rs   # 变更检测 (未使用)
│   ├── filtered_query.rs     # FilteredQuery
│   ├── query_data.rs         # 多组件查询 (未使用)
│   └── query_filter.rs       # With, Without 等
├── render_world/             # 渲染世界 (渲染数据)
│   ├── mod.rs
│   ├── world.rs              # RenderWorld ECS
│   ├── command.rs            # RenderCommand, DrawCall
│   ├── phase.rs              # RenderPhase, PhaseItem (未使用)
│   ├── view.rs               # View, ViewTarget, ClearConfig
│   ├── extracted.rs          # ExtractedMesh, ExtractedView, ExtractedUI
│   └── object.rs             # RenderObject (未使用)
├── pipeline/                 # 渲染管线
│   ├── mod.rs
│   ├── renderer.rs           # Renderer trait
│   ├── batch.rs              # 批处理系统 (未使用)
│   ├── texture.rs            # 纹理系统 (未使用)
│   ├── gradient.rs           # 渐变系统
│   ├── software/             # CPU 软件渲染后端
│   │   ├── mod.rs
│   │   └── backend.rs        # SoftwareBackend
│   └── gpu/                  # GPU 渲染后端 (空占位)
│       └── mod.rs
├── sync/                     # 双世界同步
│   ├── mod.rs
│   ├── sync_markers.rs       # SyncToRenderWorld, RenderEntity, MainEntity
│   ├── pending_sync.rs       # PendingSyncEntity
│   └── sync_system.rs        # entity_sync_system
├── extract/                  # 提取阶段
│   └── mod.rs                # ExtractComponent, Extractors
├── schedule/                 # 调度系统
│   ├── mod.rs
│   ├── schedule.rs           # Schedule
│   ├── label.rs              # Startup, Update, PreUpdate, PostUpdate, Last
│   ├── set.rs                # SystemSet (未使用)
│   └── condition.rs          # RunCondition (未使用)
├── plugin/                   # 插件系统
│   ├── mod.rs
│   ├── plugin.rs             # Plugin trait
│   ├── plugin_group.rs       # PluginGroup
│   ├── default_plugins.rs    # DefaultPlugins
│   └── sync_component_plugin.rs  # SyncComponentPlugin (未使用)
├── node/                     # 节点系统
│   ├── mod.rs
│   ├── node.rs               # Node, NodeType
│   ├── node3d.rs             # Transform, Transform3D
│   ├── style.rs              # Style (未使用)
│   └── layout.rs             # Layout (未使用)
├── animation/                # 动画系统
│   ├── mod.rs
│   ├── clip.rs               # AnimationClip
│   ├── curve.rs              # KeyframeCurve
│   ├── easing.rs             # Easing 函数
│   ├── player.rs             # AnimationPlayer
│   └── property.rs           # AnimationProperty, AnimationReceiver
├── math/                     # 数学库
│   ├── mod.rs
│   ├── vec2.rs               # Vec2
│   ├── vec3.rs               # Vec3
│   ├── mat4.rs               # Mat4
│   ├── rect.rs               # Rect
│   └── color.rs              # Color, BlendMode
├── resources/                # 资源系统
│   ├── mod.rs
│   ├── resources.rs          # Resource trait, Resources 存储
│   ├── time.rs               # Time
│   ├── camera.rs             # Camera
│   ├── screen.rs             # PrimaryScreen
│   └── config.rs             # Config
├── event/                    # 事件系统
│   ├── mod.rs
│   ├── events.rs             # Events
│   ├── event_reader.rs       # EventReader
│   ├── event_writer.rs       # EventWriter
│   └── system_param.rs
├── camera/                   # 相机插件
│   └── mod.rs
├── picking/                  # Picking 系统
│   ├── mod.rs                # 模块导出
│   ├── pickable.rs           # Pickable 组件
│   ├── bounds.rs             # PickableBounds 组件
│   ├── hover.rs              # HoverMap, PreviousHoverMap
│   ├── system.rs             # update_hover_map, ui_picking_backend
│   └── plugin.rs             # PickingPlugin, PointerHitsBuffer
└── window/                   # 窗口抽象 (平台无关)
    └── mod.rs                # Window trait, MousePosition, 原始事件类型

# 平台层 (examples/fhre/rust/src/platform/)
platform/
├── mod.rs
├── framebuffer.rs            # Window 实现 (NuttX/X11)
├── input/                    # 平台输入类型
│   ├── mod.rs
│   ├── button_input.rs       # ButtonInput<T>
│   ├── keyboard.rs           # KeyCode, Key
│   └── mouse.rs              # MouseButton
└── runner.rs                 # InputBridge, WindowRunner

# 应用层组件 (examples/fhre/rust/src/components/)
components/
├── mod.rs
├── button.rs                 # Button 组件 (含交互状态)
├── cube.rs                   # Cube 3D 模型
└── soccer_ball.rs            # SoccerBall 3D 模型
```

## 核心架构

### 1. 双世界架构 (Dual World Architecture)

```
┌─────────────────────────────────────────────────────────────────────┐
│                           App                                       │
│  ┌─────────────────────────┐    ┌─────────────────────────┐        │
│  │      Main World         │    │     Render World        │        │
│  │  ─────────────────      │    │  ─────────────────      │        │
│  │  • Entity + Component   │    │  • Entity + Component   │        │
│  │  • Transform            │    │  • MainEntity           │        │
│  │  • Cube, SoccerBall     │    │  • ExtractedMesh        │        │
│  │  • AnimationPlayer      │    │  • ExtractedUI          │        │
│  │  • Button (含交互状态)   │    │  • RenderCommand        │        │
│  │  • Game Logic Systems   │    │                         │        │
│  └─────────────────────────┘    └─────────────────────────┘        │
│              │                              ▲                      │
│              │ entity_sync_system           │                      │
│              │ Extractors                   │                      │
│              └──────────────────────────────┘                      │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                    SoftwareBackend                           │  │
│  │              CPU 软件渲染 (fill_rect, fill_polygon, etc.)   │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### 2. 帧生命周期

```
帧执行顺序 (App::update_and_render):

1. initialize_plugins()     # 仅首帧，初始化插件
2. run_startup_systems()    # 仅首帧，执行 Startup 系统
3. Update Time resource     # 更新时间
4. run_systems()            # 执行主世界系统
    ├── PreUpdate            # button_interaction_system, input_system
    ├── apply_commands()     # 应用命令
   ├── Update               # 游戏逻辑、动画
   ├── apply_commands()     # 应用命令
   ├── PostUpdate           # 后处理
   └── apply_commands()     # 应用命令
5. entity_sync_system()     # 同步实体到渲染世界
6. extractors.run()         # 提取组件到渲染世界
7. execute_render()         # 执行渲染
8. clear_commands()         # 清理命令
```

### 3. 实体同步机制

```rust
// Main World 标记
#[derive(Component)]
struct SyncToRenderWorld;  // 标记需要同步到渲染世界

// Main World 组件
#[derive(Component)]
struct RenderEntity(Entity);  // 存储对应的渲染世界实体 ID

// Render World 组件
#[derive(Component)]
struct MainEntity(Entity);  // 存储对应的主世界实体 ID

// 同步流程
Main World: Entity A + SyncToRenderWorld
    │
    │ entity_sync_system (检查 PendingSyncEntity)
    ▼
Render World: Entity B + MainEntity(A)
    │
    │ extract_xxx (自定义提取器)
    ▼
Render World: Entity B + ExtractedMesh / ExtractedUI
    │
    │ queue_xxx (生成命令)
    ▼
Render World: RenderCommand 列表
```

### 4. 数据流详解

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              MAIN LOOP                                       │
│  (WindowRunner::run)                                                         │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  1. INPUT BRIDGING                                                          │
│     WindowRunner::bridge_keyboard_input()                                   │
│     WindowRunner::bridge_mouse_input()                                      │
│     → Updates ButtonInput<KeyCode>, ButtonInput<MouseButton>, MousePosition │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  2. App::update_and_render()                                                │
│     ├── initialize_plugins() (first frame only)                             │
│     ├── main_world.run_startup_systems() (first frame only)                 │
│     ├── Update Time resource                                                │
│     └── main_world.run_systems()                                            │
│         ├── PreUpdate: button_interaction_system, input_system              │
│         ├── Update: setup_animation, animation_control_system,              │
│         │           model_switch_system, apply_animations                    │
│         └── PostUpdate: (empty)                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  3. SYNC PHASE                                                              │
│     entity_sync_system(&mut main_world, &mut render_world)                  │
│     → Syncs entities marked with SyncToRenderWorld                          │
│     → Creates RenderEntity in Main World, MainEntity in Render World        │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  4. EXTRACT PHASE                                                           │
│     extractors.run(&main_world, &mut render_world)                          │
│     ├── extract_view: Camera → ViewBundle                                   │
│     ├── extract_3d_components: Cube/SoccerBall → ExtractedMesh              │
│     ├── extract_buttons: Button → ExtractedUI                               │
│     ├── queue_meshes: ExtractedMesh → RenderCommand::DrawPolygon/DrawLine   │
│     └── queue_ui: ExtractedUI → RenderCommand::DrawRect                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  5. RENDER PHASE                                                            │
│     render_world.execute_render()                                           │
│     └── SoftwareBackend::execute_commands(&commands)                        │
│         ├── clear(color)                                                    │
│         ├── fill_polygon()                                                  │
│         ├── draw_line()                                                     │
│         └── fill_rect()                                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  6. PRESENT                                                                 │
│     window.present(app.framebuffer())                                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 模块详解

### 1. Main World (主世界)

**职责**: 游戏逻辑、场景管理、用户系统

**核心类型**:
- `Entity`: 实体 ID (u64)
- `Component`: 组件 trait
- `System`: 系统 trait
- `Query<T>`: 组件查询（单组件）
- `Commands`: 命令缓冲区

**重要限制**:
- `Query<T>` 要求 `T: Component`，不支持元组查询如 `Query<(A, B)>`
- 需要关联多个组件时，使用多个独立的 Query，通过 Entity ID 关联

**系统声明宏**:
```rust
declare_system!(func_name; Param1, Param2, ...)
// 展开为 systemN::<P1, P2, ..., _>(func_name)
```

### 2. Render World (渲染世界)

**职责**: 渲染数据存储、命令生成、帧缓冲输出

**核心类型**:
- `RenderCommand`: 渲染命令枚举
- `ExtractedMesh`: 提取的 3D 网格数据
- `ExtractedUI`: 提取的 UI 数据
- `ViewBundle`: 视图配置

### 3. Pipeline (渲染管线)

**SoftwareBackend**:
- Alpha 混合 (Normal 模式)
- 基本图形绘制 (rect, line, triangle, polygon)
- 渐变填充
- 圆角矩形
- 裁剪区域

**重要**: `blend_lut` 使用堆分配避免栈溢出：
```rust
pub struct SoftwareBackend {
    framebuffer: Vec<u32>,
    blend_lut: Box<[[u8; 256]]>,  // 堆分配，避免栈溢出
    // ...
}
```

### 4. Node System (节点系统)

**设计理念**: Node 是 FHRE 的统一最小单位。

```rust
pub enum NodeType {
    Empty,      // 空节点
    Sprite,     // 精灵
    Model,      // 3D 模型
    Camera,     // 相机
    Light,      // 灯光
    Button,     // UI 按钮
    Text,       // UI 文本
    Panel,      // UI 面板
    Custom(u16),
}
```

**变换组件**:
```rust
// 统一变换组件
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,  // 欧拉角
    pub scale: Vec3,
}

// 别名
pub type Transform3D = Transform;
```

### 5. Animation System (动画系统)

**架构**:
```
AnimationClip (动画数据)
    │
    ▼
AnimationResources (资源存储)
    │
    ▼
AnimationPlayer (实体组件)
    │
    ▼
apply_animations::<T> (应用到组件)
```

**支持的属性**:
```rust
pub enum AnimationProperty {
    TranslationX, TranslationY, TranslationZ,
    RotationX, RotationY, RotationZ,
    ScaleX, ScaleY, ScaleZ,
    ColorR, ColorG, ColorB, ColorA,
    Custom(u32),
}
```

### 6. Button Component (按钮组件)

**设计理念**: Button 组件管理交互状态，Picking 系统检测指针碰撞。

```rust
pub struct Button {
    // 渲染属性
    pub width: f32,
    pub height: f32,
    pub state: ButtonState,
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
    pub text: &'static str,
    
    // 交互状态
    pub clicked: bool,  // 本帧是否被点击
}

pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
    Disabled,
}
```

**实体创建**:
```rust
commands.spawn()
    .insert(Node::ui_control(NodeType::Button))
    .insert(Transform::from_2d(x, y))
    .insert(Button::new(width, height).with_text("Click Me"))
    .insert(PickableBounds::from_size(width, height))
    .insert(Pickable::DEFAULT);  // 重要：使用 DEFAULT，而非 default()
```

**完整交互流程 (X11 → 按钮选中)**:

```
┌─────────────────────────────────────────────────────────────────────┐
│ 1. X11 触摸事件                                                      │
│    /dev/input0 → read() → TouchEvent { x, y, pressure }             │
│    pressure > 0 → PRESS, pressure == 0 → RELEASE                    │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 2. Window::collect_input_events()                                    │
│    framebuffer.rs 读取触摸事件，转换为：                              │
│    - MouseMotionEvent { x, y }                                       │
│    - MouseButtonEvent { button: 1, pressed, x, y }                   │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 3. WindowRunner::bridge_*()                                          │
│    bridge_mouse_input():                                             │
│      ButtonInput<MouseButton>.press(MouseButton::Left)              │
│    bridge_mouse_position():                                          │
│      MousePosition { x, y }                                          │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 4. picking_system (PreUpdate)                                        │
│    输入: MousePosition, ButtonInput<MouseButton>                     │
│         Query<Transform>, Query<PickableBounds>, Query<Pickable>     │
│                                                                      │
│    a. 收集所有 (Entity, Transform, PickableBounds, Pickable)         │
│    b. ui_picking_backend():                                          │
│       for each (entity, transform, bounds):                         │
│         if bounds.contains_point(transform.position, mouse_pos):     │
│           hits.push(PointerHits { entity, ... })                    │
│    c. update_hover_map():                                            │
│       swap(hover_map, prev_hover_map)                                │
│       for hit in sorted_hits:                                        │
│         if pickable.is_hoverable:  // Pickable::DEFAULT = true       │
│           hover_map.insert(pointer_id, (entity, hit))               │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 5. button_interaction_system (PreUpdate)                             │
│    输入: ButtonInput<MouseButton>, HoverMap, PreviousHoverMap        │
│         Query<Button>                                                │
│                                                                      │
│    a. 清除所有 button.clicked = false                                │
│    b. if just_released(MouseButton::Left):                          │
│         if let Some((entity, _)) = prev_hover_map.get(&0):          │
│           if button.state == Pressed:                               │
│             button.clicked = true   ← 点击完成！                     │
│             button.state = Hover                                    │
│    c. if let Some((entity, _)) = hover_map.get(&0):                 │
│         if pressed(MouseButton::Left):                              │
│           button.state = Pressed                                    │
│         else:                                                        │
│           button.state = Hover                                      │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 6. input_system (Update)                                             │
│    for (_, button) in button_query.iter():                          │
│      if button.clicked:                                              │
│        match button.text:                                            │
│          "Prev" => 切换模型                                          │
│          "Pause" => 暂停/恢复                                        │
│          "Next" => 切换模型                                          │
└─────────────────────────────────────────────────────────────────────┘
```

**关键数据流**:

| 阶段 | 数据示例 |
|------|----------|
| X11 | TouchEvent { x: 483, y: 396, pressure: 42 } |
| Window | MouseButtonEvent { button: 1, pressed: true, x: 483, y: 396 } |
| InputBridge | MousePosition { x: 483, y: 396 }, ButtonInput.pressed(Left) |
| Picking | hits: [Entity 3], hover_map: { 0 → (Entity 3, ...) } |
| Button | button.state = Pressed → clicked = true |

**交互系统代码**:
```rust
fn button_interaction_system(
    mouse_input: Res<ButtonInput<MouseButton>>,
    hover_map: Res<HoverMap>,
    prev_hover_map: Res<PreviousHoverMap>,
    mut button_query: Query<Button>,
) {
    // 1. 清除 clicked
    for (_, button) in button_query.iter_mut() {
        button.clicked = false;
    }
    
    // 2. 检测点击完成
    if mouse_input.just_released(MouseButton::Left) {
        if let Some((entity, _)) = prev_hover_map.get(&POINTER_ID) {
            if let Some((_, button)) = button_query.get_pair_mut(*entity) {
                if button.state == ButtonState::Pressed {
                    button.clicked = true;
                    button.state = ButtonState::Hover;
                }
            }
        }
    }
    
    // 3. 更新状态
    if let Some((entity, _)) = hover_map.get(&POINTER_ID) {
        if let Some((_, button)) = button_query.get_pair_mut(*entity) {
            if mouse_input.pressed(MouseButton::Left) {
                button.state = ButtonState::Pressed;
            } else {
                button.state = ButtonState::Hover;
            }
        }
    }
}
```

**使用示例**:
```rust
fn input_system(button_query: Query<Button>) {
    for (_, button) in button_query.iter() {
        if button.clicked {
            match button.text {
                "Prev" => { /* 处理 */ }
                "Pause" => { /* 处理 */ }
                "Next" => { /* 处理 */ }
                _ => {}
            }
        }
    }
}
```

### 7. Input System (输入系统)

**架构原则**: FHRE 核心不知道具体的输入设备，输入类型由平台层定义。

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Application Layer                            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  User Systems:                                               │   │
│  │  - button_interaction_system: 更新 Button 状态              │   │
│  │  - input_system: 检查 button.clicked 执行操作               │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                              ▲
                              │ ECS Resources
┌─────────────────────────────────────────────────────────────────────┐
│                          Platform Layer                              │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Input Types: ButtonInput<T>, KeyCode, MouseButton          │   │
│  │  InputBridge: map_keycode(), map_mouse_button()             │   │
│  │  WindowRunner: bridge_keyboard_input(), bridge_mouse_input()│   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Window Implementation: collect_input_events()              │   │
│  │  - NuttX/X11: /dev/input0, /dev/kbd                         │   │
│  │  - Other platforms: custom implementation                    │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                              ▲
                              │ Window trait
┌─────────────────────────────────────────────────────────────────────┐
│                           FHRE Core                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Window trait: collect_input_events(), present()            │   │
│  │  MousePosition: x, y (resource)                              │   │
│  │  WindowInputEvents: raw event types                          │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

**为什么这样设计？**
1. 不同平台有不同的输入设备（键盘、触摸屏、游戏手柄）
2. KeyCode、MouseButton 等类型是平台相关的
3. FHRE 核心只关心抽象的"指针位置"和"按钮状态"
4. 平台层负责将原始事件转换为 ECS 资源

**使用示例**:
```rust
// 平台层定义输入类型
pub enum KeyCode { Space, Enter, ... }
pub enum MouseButton { Left, Right, Middle, ... }

// 平台层实现 InputBridge
pub struct InputAdapter;
impl InputBridge for InputAdapter {
    fn map_keycode(&self, code: u32) -> Option<KeyCode> { ... }
    fn map_mouse_button(&self, btn: u32) -> Option<MouseButton> { ... }
}

// 应用层使用
fn input_system(button_query: Query<Button>) {
    for (_, button) in button_query.iter() {
        if button.clicked {
            // 处理点击
        }
    }
}
```

### 8. Picking System (拾取系统)

**架构**: Picking 系统检测指针与 UI 元素的碰撞，更新悬停状态。

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Picking System Flow                          │
├─────────────────────────────────────────────────────────────────────┤
│  1. picking_system (PreUpdate)                                      │
│     ├── 收集 (Entity, Transform, PickableBounds, Pickable)          │
│     ├── ui_picking_backend() → hits                                 │
│     └── update_hover_map() → HoverMap, PreviousHoverMap             │
│                                                                      │
│  2. button_interaction_system (PreUpdate)                           │
│     ├── 检测 just_released + prev_hover_map                         │
│     └── 更新 Button.state, Button.clicked                           │
└─────────────────────────────────────────────────────────────────────┘
```

**核心组件**:
```rust
// 可点击区域
pub struct PickableBounds {
    pub width: f32,
    pub height: f32,
}

// 拾取行为配置
pub struct Pickable {
    pub should_block_lower: bool,  // 是否阻挡下层元素
    pub is_hoverable: bool,        // 是否可悬停
}

impl Pickable {
    pub const DEFAULT: Self = Self { should_block_lower: true, is_hoverable: true };
    pub const IGNORE: Self = Self { should_block_lower: false, is_hoverable: false };
}
```

**重要陷阱**: `Pickable::default()` 的 `is_hoverable=false`（bool 默认值），导致 hits 被丢弃。必须使用 `Pickable::DEFAULT`。

**悬停状态管理**:
```rust
pub struct HoverMap(pub BTreeMap<PointerId, (Entity, HitData)>);
pub struct PreviousHoverMap(pub BTreeMap<PointerId, (Entity, HitData)>);

// update_hover_map 流程:
// 1. 交换 hover_map 和 prev_hover_map
// 2. 按 order 排序 hits
// 3. 如果 pickable.is_hoverable=true，插入 hover_map
// 4. 如果 pickable.should_block_lower=true，停止处理
```

### 9. Plugin System (插件系统)

```rust
pub trait Plugin {
    fn build(&self, app: &mut App);
}

pub trait PluginGroup {
    fn build(&self, app: &mut App);
}

// 默认插件组
pub struct DefaultPlugins;
impl PluginGroup for DefaultPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugin(EventPlugin)
           .add_plugin(AnimationPlugin)
           .add_plugin(CameraPlugin);
    }
}
```

## 渲染命令

```rust
pub enum RenderCommand {
    // 基础绘制
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawLine { start: Vec2, end: Vec2, color: Color, thickness: f32 },
    DrawTriangle { p0: Vec2, p1: Vec2, p2: Vec2, color: Color },
    DrawPolygon { vertices: Vec<Vec2>, color: Color },
    
    // 高级绘制
    DrawRectGradient { rect: Rect, gradient: Gradient },
    DrawRectRounded { rect: Rect, color: Color, radius: f32 },
    DrawRectRoundedGradient { rect: Rect, gradient: Gradient, radius: f32 },
    
    // 纹理绘制
    DrawImage { rect: Rect, texture_id: u32, region: TextureRegion, color: Color },
    DrawImageTransformed { position: Vec2, size: Vec2, ..., rotation: f32, color: Color },
    
    // 文本 (占位)
    DrawText { position: Vec2, text: &'static str, color: Color, size: f32 },
    
    // 裁剪
    SetScissor { rect: Rect },
    DisableScissor,
    PushMask,
    PopMask,
}
```

## 使用示例

### 创建应用

```rust
let mut app = App::new(width, height);

// 添加插件
app.add_plugins(DefaultPlugins);

// 注册资源
app.insert_resource(DemoState::new())
   .insert_resource(ButtonInput::<KeyCode>::default())
   .insert_resource(ButtonInput::<MouseButton>::default())
   .insert_resource(MousePosition::default());

// 注册系统
app.add_systems(Startup, declare_system!(setup; Commands, Res<PrimaryScreen>))
   .add_systems(PreUpdate, declare_system!(picking_system; Res<MousePosition>, Res<ButtonInput<MouseButton>>, Query<Transform>, Query<PickableBounds>, Query<Pickable>, ResMut<HoverMap>, ResMut<PreviousHoverMap>))
   .add_systems(PreUpdate, declare_system!(button_interaction_system; Res<ButtonInput<MouseButton>>, Res<HoverMap>, Res<PreviousHoverMap>, Query<Button>))
   .add_systems(Update, declare_system!(update; Query<&mut Transform>));

// 添加提取器
app.add_extractor(extract_meshes)
   .add_extractor(extract_ui);

// 运行
WindowRunner::new(&mut app, &mut window, &input_adapter).run();
```

### 创建实体

```rust
fn spawn_button(mut commands: Commands) {
    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform::from_2d(100.0, 200.0))
        .insert(Button::new(100.0, 40.0)
            .with_text("Click Me")
            .with_colors(normal, hover, pressed))
        .insert(PickableBounds::from_size(100.0, 40.0))
        .insert(Pickable::DEFAULT);  // 重要：使用 DEFAULT
}
```

### 动画

```rust
// 创建动画剪辑
let mut clip = AnimationClip::with_duration(6.0);
clip.add_curve_to_target(
    target_id,
    AnimationProperty::RotationY,
    KeyframeCurve::new(vec![
        Keyframe::new(0.0, 0.0, Easing::Linear),
        Keyframe::new(6.0, 360.0, Easing::Linear),
    ]),
);

// 注册并播放
let handle = anim_resources.insert_clip(clip);
player.play_with_target(handle, target_id);
```

## 重要限制与注意事项

### 1. Query 不支持元组查询

```rust
// ❌ 错误：FHRE Query 不支持元组
fn system(query: Query<(Transform, Pickable)>) { ... }

// ✅ 正确：使用多个独立 Query，通过 Entity 关联
fn system(
    transform_query: Query<Transform>,
    pickable_query: Query<Pickable>,
) {
    for (entity, transform) in transform_query.iter() {
        if let Some(pickable) = pickable_query.get(entity) {
            // ...
        }
    }
}
```

### 2. 栈溢出风险

在 no_std 环境下避免大数组栈分配：
```rust
// ❌ 危险：64KB 栈数组可能导致栈溢出
let blend_lut: [[u8; 256]; 256] = [[0; 256]; 256];

// ✅ 安全：使用堆分配
let blend_lut: Box<[[u8; 256]]> = /* ... */;
```

### 3. Pickable::default() 陷阱

```rust
// ❌ 错误：default() 的 is_hoverable=false，导致 hits 被丢弃
.insert(Pickable::default())

// ✅ 正确：使用 DEFAULT 常量
.insert(Pickable::DEFAULT)
```

## 未使用模块

以下模块已实现但在当前 demo 中未使用，可根据需要启用或移除：

| 模块 | 文件 | 说明 |
|------|------|------|
| node/style.rs | Style | CSS 样式系统 |
| node/layout.rs | Layout | Flexbox 布局 |
| animation/graph.rs | AnimationGraph | 动画混合图 |
| animation/transition.rs | AnimationTransitions | 动画过渡 |
| pipeline/gpu/ | - | GPU 后端占位 |
| pipeline/batch.rs | RenderBatch | 批处理 |
| pipeline/texture.rs | Texture | 纹理系统 |
| schedule/condition.rs | RunCondition | 运行条件 |
| schedule/set.rs | SystemSet | 系统集 |
| main_world/query_data.rs | QueryData | 多组件查询数据 |

## 与 Bevy 对比

| 特性 | FHRE | Bevy |
|------|------|------|
| 目标平台 | 嵌入式 (no_std) | 桌面/移动端 |
| ECS | 简化 ECS | 完整 ECS |
| Query 元组 | ❌ 仅单组件 | ✅ 支持元组 |
| 双世界 | ✅ | ✅ |
| 渲染后端 | CPU 软件 | GPU (wgpu) |
| 调度系统 | 简化 Schedule | 完整 Schedule |
| 插件系统 | ✅ | ✅ |
| 动画系统 | ✅ | ✅ |
| Picking 系统 | ✅ | ✅ |
| UI 系统 | Node 统一 | bevy_ui |
| 着色器 | ❌ | WGSL |
| 多线程 | ❌ | ✅ |

## 文件统计

| 模块 | 文件数 | 代码行数 | 说明 |
|------|--------|----------|------|
| main_world | 9 | ~2000 | ECS 核心 |
| render_world | 7 | ~1200 | 渲染数据 |
| pipeline | 8 | ~1500 | 渲染管线 |
| animation | 8 | ~1200 | 动画系统 |
| node | 5 | ~600 | 节点系统 |
| math | 6 | ~800 | 数学库 |
| schedule | 5 | ~400 | 调度系统 |
| plugin | 5 | ~300 | 插件系统 |
| sync | 3 | ~200 | 双世界同步 |
| event | 5 | ~400 | 事件系统 |
| resources | 5 | ~300 | 资源系统 |
| app | 1 | ~200 | 应用管理 |
| window | 1 | ~100 | 窗口抽象 |
| camera | 1 | ~100 | 相机插件 |
| picking | 5 | ~300 | Picking 系统 |
| **FHRE 核心** | **66** | **~8600** | |
| platform/input | 4 | ~200 | 平台输入类型 |
| platform/runner | 1 | ~150 | 输入桥接 |
| platform/framebuffer | 1 | ~400 | 窗口实现 |
| **平台层** | **6** | **~750** | |
| components/button | 1 | ~150 | Button 组件 |
| **应用层** | **1** | **~150** | |
| **总计** | **73** | **~9500** | |

## 未来规划

### 短期
- [ ] 字体渲染系统
- [ ] 纹理图集 (TextureAtlas)
- [ ] 抗锯齿 (AA)

### 中期
- [ ] GPU 后端 (OpenGL ES)
- [ ] 着色器系统
- [ ] 后处理效果

### 长期
- [ ] Vulkan 后端
- [ ] 多线程渲染
- [ ] 粒子系统
