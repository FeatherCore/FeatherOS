# FHRE 架构文档

## 概述

FHRE (Feather Hybrid Render Engine) 是一个轻量级的纯 3D 渲染引擎，设计用于嵌入式系统 (no_std)。采用 Bevy 风格的双世界 ECS 架构。

**版本**: 2.8.2  
**目标平台**: 嵌入式系统 (NuttX RTOS)  
**代码规模**: ~19,000 行 (107 文件)  
**依赖**: 0 外部 crate (仅 `alloc`)

### 核心设计理念

**FHRE 是纯 3D 引擎，所有对象都是 3D 的。**

- **2D 只是 3D 的特例**: 所谓的 "2D 对象" 只是恰好与默认幕布处于同一平面 (z=0)
- **统一坐标系**: 所有对象使用 3D 世界坐标 `(x, y, z)`
- **统一变换**: 所有对象使用 `Transform` 组件 (position, rotation, scale)
- **相机投影**: 3D 场景通过相机投影到幕布，形成最终画面

```
┌─────────────────────────────────────────────────────────────────┐
│                        3D Main World                            │
│                                                                  │
│   z=600    Camera ────────────────────┐                        │
│                  \                     │                        │
│                   \  view frustum      │                        │
│                    \                   │                        │
│   z=0    ┌─────────────────────────────▼──────────────────┐    │
│          │              Screen Canvas (幕布)               │    │
│          │                                                   │    │
│          │   Cube (z=0)     Button (z=0)    SoccerBall     │    │
│          │   3D object      3D object       3D object      │    │
│          │   (在幕布平面)    (在幕布平面)    (在幕布平面)    │    │
│          │                                                   │    │
│          └───────────────────────────────────────────────────┘    │
│                                                                  │
│   z=-100  Background objects (behind canvas)                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

"2D" UI = 3D objects with z=0 (same plane as canvas)
"3D" models = 3D objects with any z value
```

**实际意义**:
- Button、Cube、SoccerBall 都是 3D 对象
- Button 的 z=0 使其看起来像 "2D UI"
- 可以通过修改 z 值实现前后层叠效果
- 可以对任何对象应用 3D 变换（旋转、缩放）

### 核心特性

| 特性 | 状态 | 说明 |
|------|------|------|
| 双世界架构 | ✅ | MainWorld + RenderWorld |
| ECS 系统 | ✅ | BTreeMap 存储，支持元组查询 |
| 事件驱动交互 | ✅ | Pointer<E> 事件系统 |
| 动画系统 | ✅ | 多属性动画，支持 Cube/SoccerBall |
| Asset 系统 | ✅ | Handle<Image> + GpuTexture 自动上传 |
| CPU 软件渲染 | ✅ | 无 GPU 依赖 |
| no_std 兼容 | ✅ | 仅使用 alloc crate |

---

## 架构概览

### 双世界架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                           App                                       │
│  ┌─────────────────────────┐    ┌─────────────────────────┐        │
│  │      Main World         │    │     Render World        │        │
│  │  ─────────────────      │    │  ─────────────────      │        │
│  │  • Entity + Component   │    │  • Entity + Component   │        │
│  │  • Transform            │    │  • MainEntity           │        │
│  │  • Cube, SoccerBall     │    │  • ExtractedMesh        │        │
│  │  • AnimationPlayer      │    │  • RenderCommand        │        │
│  │  • Button               │    │  • GpuTextures          │        │
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

### 帧生命周期

```
帧执行顺序 (App::update_and_render):

1. initialize_plugins()     # 仅首帧
2. run_startup_systems()    # 仅首帧: setup_textures → setup
3. Update Time resource     
4. run_systems():           
   ├── PreUpdate: picking_system → button_interaction_system → input_system
   ├── Update: advance_animations → animate_targets → apply_animations → model_switch_system
   └── PostUpdate
5. entity_sync_system()     # 同步实体到渲染世界
6. extractors.run()         # extract_view → extract_3d_components → queue_meshes
7. execute_render()         # 执行渲染
8. Events::update()         # 清理事件缓冲区
```

### 数据流

```
Main World                    Render World                    Output
──────────                    ────────────                    ──────

Transform ──────────────┐
Cube ───────────────────┼──→ ExtractedMesh ──→ DrawPolygon ──→ Framebuffer
SoccerBall ─────────────┘

Transform ──────────────┐
Button ─────────────────┼──→ ExtractedUI ────→ DrawRect ────→ Framebuffer

AnimationPlayer ────────→ apply_animations ──→ 更新 Cube/SoccerBall 属性

Assets<Image> ──────────→ ExtractedAssets ──→ GpuTextures ──→ DrawPolygonTextured
```

### 相机 + 幕布系统

FHRE 采用单相机架构，相机捕获 3D 场景并投影到幕布 (Screen Canvas)：

```
Camera (3D) ──looks at──▶ Screen Canvas (幕布, 3D plane)
     │                          │
     │ position: (cx, cy, 600)  │ position: (cx, cy, 0)
     │ target: (cx, cy, 0)      │ size: width × height
     │ FOV: 45°                 │
     │                          │ 1:1 default mapping
     └──────────────────────────┼──────────────────────▶ Presentation Window
                                │                         (X11/Framebuffer)
                                ▼
                           Rendered Frame
```

**核心概念**:
- **Camera**: 3D 透视相机，默认位于幕布后方 600 单位
- **PrimaryScreen**: 幕布资源，定义投影平面位置和大小
- **View**: RenderWorld 中的视图配置，由 Camera + PrimaryScreen 提取生成
- **默认行为**: 幕布位于 z=0，相机看向幕布中心，实现 1:1 像素映射

**配置方式**:
```rust
// 默认配置 (CameraPlugin 自动注册)
app.add_plugins(DefaultPlugins);  // 包含 CameraPlugin

// 自定义相机
app.insert_resource(Camera::perspective_3d(
    Vec3::new(320.0, 240.0, 500.0),  // position
    Vec3::new(320.0, 240.0, 0.0),    // target
    60.0,                              // FOV
));

// 自定义幕布位置 (缩放效果)
let mut screen = PrimaryScreen::new(640, 480);
screen.move_closer(100.0);  // 幕布靠近相机 → 放大效果
app.insert_resource(screen);
```

**坐标系统**:
- 3D 对象使用世界坐标 `(x, y, z)`
- 相机投影后转换为幕布像素坐标
- UI 元素直接使用幕布像素坐标 `(x, y)`

### 呈现窗口 + 输入系统

呈现窗口 (Presentation Window) 是 FHRE 与用户交互的接口，负责：
1. 显示渲染结果 (输出)
2. 收集用户输入 (输入)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Presentation Window (X11/NuttX)                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  /dev/fb0     → Framebuffer (ARGB8888) → 显示渲染结果               │   │
│  │  /dev/input0  → Touch/Mouse events → 原始坐标 (x, y) + pressure     │   │
│  │  /dev/kbd     → Keyboard events → keycode + pressed                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ Window::collect_input_events()
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         WindowInputEvents (原始事件)                         │
│  - MouseButtonEvent { button, pressed, x, y }                               │
│  - MouseMotionEvent { x, y, delta_x, delta_y }                              │
│  - KeyboardEvent { keycode, pressed }                                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ WindowRunner::bridge_*()
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ECS Resources (FHRE Core)                            │
│  - ButtonInput<MouseButton> → press(MouseButton::Left)                      │
│  - ButtonInput<KeyCode>     → press(KeyCode::Space)                         │
│  - MousePosition            → { x, y } (屏幕像素坐标)                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ picking_system (PreUpdate)
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Picking System (命中检测)                            │
│                                                                              │
│  1. 收集所有可点击实体: (Entity, Transform, PickableBounds, Pickable)        │
│  2. 碰撞检测: bounds.contains_point(transform.position, mouse_pos)          │
│  3. 生成 PointerHits                                                        │
│  4. 更新 HoverMap                                                           │
│  5. 生成 Pointer<E> 事件:                                                   │
│     - Pointer<Over>  → 鼠标进入实体                                          │
│     - Pointer<Out>   → 鼠标离开实体                                          │
│     - Pointer<Press> → 鼠标按下                                              │
│     - Pointer<Click> → 点击完成 (同一实体上按下并释放)                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ button_interaction_system
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Button 状态更新                                       │
│                                                                              │
│  Pointer<Over>  → button.state = Hover                                      │
│  Pointer<Out>   → button.state = Normal                                     │
│  Pointer<Press> → button.state = Pressed                                    │
│  Pointer<Click> → button.clicked = true                                     │
└─────────────────────────────────────────────────────────────────────────────┘
```

**关键流程**:
1. **原始输入**: `/dev/input0` 读取触摸/鼠标事件，坐标为屏幕像素坐标
2. **坐标传递**: `MousePosition` 直接使用屏幕坐标，无需转换
3. **命中检测**: `PickableBounds` 使用相同的屏幕坐标系统进行碰撞检测
4. **事件驱动**: 通过 `Pointer<E>` 事件解耦输入和组件状态

**平台适配**:
```rust
// 平台层实现 Window trait
impl Window for MyWindow {
    fn collect_input_events(&mut self) -> WindowInputEvents {
        // 读取平台输入设备，返回原始事件
    }
    
    fn present(&mut self, framebuffer: &[u32]) {
        // 将帧缓冲输出到屏幕
    }
}

// 平台层实现 InputBridge
impl InputBridge for MyInputBridge {
    fn map_mouse_button(&self, button: u32) -> Option<MouseButton> {
        match button {
            1 => Some(MouseButton::Left),
            // ...
        }
    }
}
```

---

## 目录结构

```
apps/fhre/rust/src/                   # FHRE 核心库 (~15,500 行)
├── lib.rs                            # 库入口，全局分配器
├── app/                              # 应用框架 (~350 行)
│   ├── app.rs                        # App 结构，update_and_render
│   └── config.rs                     
├── main_world/                       # 主世界 ECS (~2,200 行)
│   ├── world.rs                      # MainWorld
│   ├── entity.rs                     # Entity ID
│   ├── component.rs                  # Component trait
│   ├── system.rs                     # System trait, declare_system!
│   ├── system_param.rs               # Res, ResMut, Query
│   ├── commands.rs                   # Commands, EntityCommands
│   ├── query_data.rs                 # QueryData trait (元组查询)
│   ├── query_filter.rs               # QueryFilter trait
│   ├── filtered_query.rs             # Query<D, F>
│   ├── change_detection.rs           # Mut<T>, Ref<T>
│   └── tuples.rs                     # all_tuples! 宏
├── render_world/                     # 渲染世界 (~1,500 行)
│   ├── world.rs                      # RenderWorld
│   ├── command.rs                    # RenderCommand
│   ├── phase.rs                      # RenderPhase, PhaseItem
│   ├── view.rs                       # View, ViewTarget
│   └── extracted.rs                  # ExtractedMesh, ExtractedUI
├── pipeline/                         # 渲染管线 (~1,400 行)
│   ├── renderer.rs                   # Renderer trait
│   ├── texture.rs                    # Texture, TextureFormat
│   └── software/backend.rs           # SoftwareBackend (CPU)
├── asset/                            # 资产系统 (~800 行)
│   ├── assets.rs                     # Assets<A>
│   ├── handle.rs                     # Handle<A>
│   ├── image.rs                      # Image, GpuTexture
│   └── extract_plugin.rs             # RenderAssetPlugin
├── animation/                        # 动画系统 (~1,500 行)
│   ├── clip.rs                       # AnimationClip
│   ├── player.rs                     # AnimationPlayer
│   ├── property.rs                   # AnimationProperty, AnimationReceiver
│   └── curve.rs                      # KeyframeCurve
├── picking/                          # Picking 系统 (~350 行)
│   ├── pickable.rs                   # Pickable, PickableBounds
│   ├── events.rs                     # Pointer<E>
│   └── system.rs                     # picking_system, pointer_events
├── sync/                             # 双世界同步 (~250 行)
├── extract/                          # 提取系统 (~480 行)
├── event/                            # 事件系统 (~450 行)
├── schedule/                         # 调度系统 (~400 行)
├── plugin/                           # 插件系统 (~300 行)
├── node/                             # 节点系统 (~1,200 行)
├── math/                             # 数学库 (~500 行)
├── resources/                        # 资源系统 (~500 行)
├── camera/                           # 相机插件 (~60 行)
└── window/                           # 窗口抽象 (~95 行)

apps/examples/fhre/rust/src/          # 示例应用 (~2,770 行)
├── lib.rs                            # 主入口，系统注册
├── extract.rs                        # 自定义提取器
├── components/                       # 应用层组件
│   ├── button.rs                     # Button 组件
│   ├── cube.rs                       # Cube 3D 模型
│   └── soccer_ball.rs                # SoccerBall 3D 模型
└── platform/                         # 平台层实现
    ├── framebuffer.rs                # Window (X11/NuttX)
    ├── runner.rs                     # WindowRunner
    └── input/                        # 输入类型
```

---

## 核心系统

### 1. Query 系统

支持 Bevy 风格的元组查询：

```rust
// 单组件
fn system1(query: Query<&Transform>) {
    for (entity, transform) in query.iter() { }
}

// 多组件元组 (最多 15 个)
fn system2(query: Query<(Entity, &Transform, &Velocity)>) {
    for (entity, (transform, velocity)) in query.iter() { }
}

// 可变访问
fn system3(query: Query<(&mut Transform, &Velocity)>) {
    for (entity, (transform, velocity)) in query.iter_mut() { }
}

// 带过滤器
fn system4(query: Query<&Transform, With<Velocity>>) { }

// 变更检测
fn system5(query: Query<Mut<Transform>>) {
    for (entity, transform) in query.iter() {
        if transform.is_added() { /* 刚添加 */ }
        if transform.is_changed() { /* 已修改 */ }
        transform.position.x += 1.0; // 自动标记变更
    }
}
```

**支持**: `&T`, `&mut T`, `Entity`, `Mut<T>`, `Ref<T>`, 元组, `With<T>`, `Without<T>`, `Added<T>`, `Changed<T>`

**不支持**: `Option<&T>`, `AnyOf<...>`, `Has<T>` (需要反射系统)

### 2. Asset 系统

```rust
// 创建纹理
fn setup_textures(mut images: ResMut<Assets<Image>>, mut events: ResMut<Events>) {
    let handle = images.add_with_event(Image { ... }, &mut events);
}

// 在组件中使用
commands.spawn()
    .insert(Cube::new(120.0)
        .with_face_textures_handles([
            cube_textures.front.clone(),
            cube_textures.back.clone(),
        ]));

// 自动流程
Assets::add_with_event() → AssetEvent → ExtractAssets → PrepareAssets → GpuTextures
```

### 3. 动画系统

```rust
// 创建动画剪辑
let mut clip = AnimationClip::with_duration(6.0);
clip.add_curve_to_target(target_id, AnimationProperty::RotationY, 
    KeyframeCurve::new(vec![
        Keyframe::new(0.0, 0.0, Easing::Linear),
        Keyframe::new(6.0, 360.0, Easing::Linear),
    ]));

// 注册并播放
let handle = anim_resources.insert_clip(clip);
player.play_with_target(handle, target_id);

// 应用到组件 (自动)
apply_animations::<Cube>: AnimationPlayer → Cube.rotation
```

### 4. Picking 系统

```
输入事件流:
Window::collect_input_events()
    → ButtonInput<MouseButton>, MousePosition
    → picking_system: 生成 PointerHits
    → update_hover_map: HoverMap
    → pointer_events: Pointer<Over>, Pointer<Click>, etc.
    → button_interaction_system: 更新 Button.state
    → input_system: 执行操作
```

**关键组件**:
- `PickableBounds`: 可点击区域
- `Pickable::DEFAULT`: 必须使用 DEFAULT 而非 default()
- `HoverMap` / `PreviousHoverMap`: 悬停状态

### 5. 渲染命令

```rust
pub enum RenderCommand {
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawLine { start: Vec2, end: Vec2, color: Color, thickness: f32 },
    DrawPolygon { vertices: Vec<Vec2>, color: Color },
    DrawPolygonTextured { vertices, uvs, texture_id, color },
    DrawRectRounded { rect, color, radius },
    DrawRectGradient { rect, gradient },
    // ...
}
```

---

## 示例应用

### 系统注册

```rust
fn fhre_rust_main() -> i32 {
    let mut app = App::new(width, height);
    
    app.add_plugins(DefaultPlugins)
        .add_plugin(TextureAssetPlugin)
        .insert_resource(DemoState::new())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(MousePosition::default())
        .add_systems(Startup, declare_system!(setup_textures; ...))
        .add_systems(Startup, declare_system!(setup; ...))
        .add_systems(PreUpdate, declare_system!(picking_system; ...))
        .add_systems(PreUpdate, declare_system!(button_interaction_system; ...))
        .add_systems(PreUpdate, declare_system!(input_system; ...))
        .add_systems(Update, declare_system!(setup_animation; ...))
        .add_systems(Update, declare_system!(animation_control_system; ...))
        .add_systems(Update, declare_system!(model_switch_system; ...))
        .add_systems(Update, declare_system!(apply_animations::<Cube>; ...))
        .add_systems(Update, declare_system!(apply_animations::<SoccerBall>; ...))
        .add_extractor(extract::extract_view)
        .add_extractor(extract::extract_3d_components)
        .add_extractor(extract::queue_meshes)
        .add_extractor(extract::queue_ui);
    
    WindowRunner::new(&mut app, &mut window, &input_adapter).run();
}
```

### 实体创建

```rust
// 3D 模型
commands.spawn()
    .insert(Node::game_entity())
    .insert(Transform3D::from_position(x, y, 0.0))
    .insert(Cube::new(120.0)
        .with_face_colors(colors)
        .with_face_textures_handles(textures)
        .with_wireframe(true, Color::WHITE))
    .insert(AnimationPlayer::new())
    .insert(SyncToRenderWorld);

// 按钮
commands.spawn()
    .insert(Node::ui_control())
    .insert(Transform::from_2d(x, y))
    .insert(Button::new(80.0, 30.0).with_text("Click"))
    .insert(PickableBounds::from_size(80.0, 30.0))
    .insert(Pickable::DEFAULT);  // 重要：使用 DEFAULT
```

---

## 与 Bevy 对比

### 核心差异

| 维度 | FHRE | Bevy | 影响 |
|------|------|------|------|
| **运行环境** | `no_std` (裸机/嵌入式) | `std` (桌面/移动/Web) | 无标准库，无系统调用 |
| **外部依赖** | 0 (仅 `alloc`) | ~100+ crates | 完全自包含 |
| **呈现窗口** | 外部传入 (平台层实现) | 内置 (winit) | 输出/输入与引擎解耦 |
| **事件循环** | 无 (裸机循环) | 有 (winit event loop) | 嵌入式通常没有 OS 事件系统 |
| **资产加载** | 同步 (Flash/ROM) | 异步 (文件系统) | 嵌入式无文件系统 |

### 呈现窗口架构

FHRE 的核心设计原则：**渲染引擎不关心输出目标和输入来源**。

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FHRE Core (no_std)                            │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  MainWorld → RenderWorld → Framebuffer (Vec<u32>)              │   │
│  │                                                                   │   │
│  │  不包含:                                                          │   │
│  │  - 窗口管理                                                       │   │
│  │  - 输入设备驱动                                                   │   │
│  │  - 事件循环                                                       │   │
│  │  - 文件系统                                                       │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                    │                                    ▲
                    │ Framebuffer                        │ WindowInputEvents
                    ▼                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                      Platform Layer (应用层实现)                         │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
│  │ X11 + /dev/input │  │ NuttX + /dev/fb0 │  │ 裸机 + LCD驱动   │      │
│  │ (桌面模拟器)      │  │ (嵌入式 RTOS)    │  │ (裸机系统)       │      │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘      │
│                                                                         │
│  实现 Window trait:                                                      │
│  - present(framebuffer) → 输出到屏幕                                    │
│  - collect_input_events() → 从设备读取输入                              │
│  - dimensions() → 返回屏幕尺寸                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**设计意义**:
- FHRE 可在任何 `no_std` 环境运行，无需修改核心代码
- 平台层只需实现 `Window` trait，适配不同硬件
- 输入/输出完全解耦，便于测试和移植

### 规模对比

| 指标 | FHRE | Bevy | 比例 |
|------|------|------|------|
| 代码行数 | ~19,000 | ~468,000 | 1:25 |
| 文件数 | 107 | 1192 | 1:11 |
| Crate 数 | 1 | 58 | - |
| 外部依赖 | 0 | ~100+ | - |

### 功能对比

| 特性 | FHRE | Bevy |
|------|------|------|
| 目标平台 | 嵌入式 (no_std) | 桌面/移动/Web |
| ECS 存储 | BTreeMap | Archetype |
| Query 元组 | ✅ 0-15 元素 | ✅ derive |
| Query Option | ❌ | ✅ |
| 双世界 | ✅ | ✅ |
| 渲染后端 | CPU 软件 | GPU (wgpu) |
| 调度系统 | 5 阶段 | 12+ 阶段 |
| 动画系统 | ✅ 基础 | ✅ 完整 |
| Picking | ✅ UI | ✅ 多后端 |
| Asset 系统 | ✅ 同步 | ✅ 异步+热重载 |
| 反射 | ❌ | ✅ |
| 多线程 | ❌ | ✅ |
| 窗口管理 | ❌ (外部传入) | ✅ (winit) |
| 事件循环 | ❌ (裸机循环) | ✅ (winit) |

### Bevy Crate 映射

| FHRE 模块 | Bevy Crate | 备注 |
|-----------|------------|------|
| main_world/ | bevy_ecs | 简化版 ECS |
| render_world/ | bevy_render | 无 GPU 抽象 |
| app/ | bevy_app | 无 SubApp |
| animation/ | bevy_animation | 基础动画 |
| picking/ | bevy_picking | 仅 UI |
| asset/ | bevy_asset | 同步加载 |
| node/ | bevy_transform | 简化版 |
| math/ | bevy_math (glam) | 自实现 |
| - | bevy_window | ❌ 外部传入 |
| - | bevy_winit | ❌ 平台层实现 |
| - | bevy_input | ⚠️ 部分实现 |

### 自动化边界

基于核心差异，确定哪些功能应该在 FHRE 库自动化：

#### 必须保留在应用层 (平台相关)

| 功能 | 原因 | 应用层职责 |
|------|------|-----------|
| **Window trait 实现** | 输出目标由平台决定 | 实现 `present()`, `collect_input_events()`, `dimensions()` |
| **Runner 逻辑** | 事件循环/裸机循环由平台决定 | 调用 `app.update_and_render()`，处理平台事件 |
| **输入事件采集** | 输入设备由平台决定 | 从 `/dev/input0`、触摸屏、键盘读取原始事件 |
| **资产加载** | 存储介质由平台决定 | 从 Flash/ROM/文件系统加载资产数据 |
| **提取器** | 需知道具体组件类型 | `extract_3d_components()`, `queue_meshes()` 等 |
| **组件定义** | 应用定义的游戏对象 | Cube, Button, AnimationReceiver 等 |
| **游戏逻辑** | 应用定义的行为 | setup, input_system, model_switch_system 等 |

#### 应该移入 FHRE 库 (平台无关)

| 功能 | 当前状态 | 应该自动化 | 实现位置 |
|------|----------|-----------|----------|
| **时间系统** | 手动 `time.update()` | `TimePlugin` 在 `First` 阶段自动更新 | `app/app.rs` |
| **实体同步** | 手动调用 `entity_sync_system()` | Observer 模式自动响应变化 | `sync/sync_system.rs` |
| **提取调度** | 手动调用 `extractors.run()` | `ExtractSchedule` 自动运行 | `app/app.rs` |
| **事件更新** | 手动 `events.update()` | 自动在 `PostUpdate` 后清理 | `event/mod.rs` |
| **输入事件分发** | 手动 `bridge_*()` | `InputPlugin` 自动分发到 ECS Resources | 新增 `input/plugin.rs` |

#### 可选改进 (中优先级)

| 功能 | 描述 | 实现位置 |
|------|------|----------|
| **`RenderSystems` 枚举** | 渲染阶段自动排序 | `render_world/` |
| **`MainScheduleOrder`** | 调度顺序管理 | `schedule/` |

### 输入系统分层

输入系统是典型的分层案例，展示哪些可以在 FHRE 自动化：

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    应用层 (必须手动)                                     │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Window::collect_input_events()                                  │   │
│  │  - 从 /dev/input0 读取触摸事件                                    │   │
│  │  - 从 /dev/kbd 读取键盘事件                                       │   │
│  │  - 返回 WindowInputEvents { mouse_button_events, ... }          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                    │
                    │ WindowInputEvents
                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    FHRE 库 (可自动化)                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  InputPlugin (建议添加)                                          │   │
│  │  - InputBridge::map_mouse_button() → MouseButton                │   │
│  │  - InputBridge::map_keycode() → KeyCode                         │   │
│  │  - 自动分发到 ButtonInput<MouseButton>, ButtonInput<KeyCode>    │   │
│  │  - 自动更新 MousePosition                                        │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                    │
                    │ ECS Resources
                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    FHRE 库 (已自动化)                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  PickingPlugin                                                   │   │
│  │  - picking_system() → PointerHits                                │   │
│  │  - pointer_events() → Pointer<Over>, Pointer<Click>, ...        │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 核心循环改进

**当前应用层代码** (`App::update_and_render`):
```rust
pub fn update_and_render(&mut self) {
    // 1. 手动更新时间 ← 应该自动化
    if let Some(time) = self.main_world.resources_mut().get_mut::<Time>() {
        time.update(DEFAULT_FRAME_TIME);
    }
    
    // 2. 手动运行调度 ← 应该自动化
    self.main_world.run_systems();
    
    // 3. 手动更新事件 ← 应该自动化
    if let Some(events) = self.main_world.resources_mut().get_mut::<Events>() {
        events.update();
    }
    
    // 4. 手动同步实体 ← 应该自动化
    entity_sync_system(&mut self.main_world, &mut self.render_world);
    
    // 5. 手动提取 ← 应该自动化
    self.extractors.run(&self.main_world, &mut self.render_world);
    
    // 6. 手动渲染 ← 应该自动化
    self.render_world.execute_render();
}
```

**目标 FHRE 库自动化**:
```rust
// FHRE 库内部:
impl App {
    pub fn update_and_render(&mut self) {
        // 1. TimePlugin 自动更新时间 (First 阶段)
        // 2. MainScheduleOrder 自动运行调度
        // 3. Observer 自动同步实体
        // 4. ExtractSchedule 自动提取
        // 5. RenderSystems 自动渲染
        // 6. Events 自动清理 (Last 阶段)
    }
}

// 应用层只需:
pub fn run(mut app: App, mut window: impl Window) {
    while window.is_running() {
        let events = window.collect_input_events();  // 平台层采集 (必须手动)
        app.bridge_input_events(events);             // FHRE 分发 (建议自动化)
        app.update_and_render();                     // FHRE 核心循环 (应该自动化)
        window.present(app.framebuffer());           // 平台层输出 (必须手动)
    }
}
```

### Bevy 自动化机制参考

| 机制 | Bevy 实现 | FHRE 可行性 | 备注 |
|------|-----------|-------------|------|
| **Observer 模式** | `app.add_observer(...)` | ✅ 可实现 | 无需外部依赖 |
| **SubApp 架构** | `app.insert_sub_app(...)` | ⚠️ 简化版 | 无多线程，可简化 |
| **ExtractSchedule** | `render_app.set_extract(...)` | ✅ 可实现 | 无需外部依赖 |
| **RenderSystems** | 枚举 + 自动排序 | ✅ 可实现 | 无需外部依赖 |
| **winit 事件循环** | `event_loop.run(...)` | ❌ 平台相关 | 嵌入式无事件循环 |
| **异步资产加载** | `AssetServer::load_async()` | ❌ 无异步运行时 | 嵌入式同步加载 |

---

## 示例应用手动实现清单

以下是从 `/apps/examples/fhre/rust` 模板 demo 中需要手动实现的部分：

### 必须手动 (平台相关或应用相关)

| 类别 | 文件/函数 | 职责 | 原因 |
|------|-----------|------|------|
| **平台适配** | `framebuffer.rs` | 实现 `Window` trait | 输出目标由平台决定 |
| **平台适配** | `runner.rs` | 主循环 + 输入桥接 | 事件循环由平台决定 |
| **提取器** | `extract_3d_components()` | Cube/SoccerBall → ExtractedMesh | 需知道具体组件类型 |
| **提取器** | `queue_meshes()` | ExtractedMesh → RenderCommand | 需知道渲染逻辑 |
| **组件** | `Cube`, `Button` | 游戏对象定义 | 应用定义的游戏对象 |
| **游戏逻辑** | `setup()`, `input_system()` | 场景创建、交互逻辑 | 应用定义的行为 |

### 建议自动化 (平台无关)

| 类别 | 当前状态 | 建议 |
|------|----------|------|
| **输入桥接** | 手动 `bridge_*()` | FHRE 提供 `InputPlugin` |
| **核心循环** | 手动 `update_and_render()` | FHRE 自动化时间、调度、同步、提取、渲染 |

### 责任边界

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    应用层责任 (必须手动)                                 │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  平台适配: Window trait, Runner                                  │   │
│  │  组件定义: Cube, Button, ...                                     │   │
│  │  游戏逻辑: setup, input_system, ...                              │   │
│  │  提取器: extract_3d_components, queue_meshes, ...                │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                    │
                    │ 明确的接口
                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    FHRE 库责任 (应该自动化)                              │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  核心循环: update_and_render (时间、调度、同步、提取、渲染)        │   │
│  │  输入分发: InputPlugin (WindowInputEvents → ECS Resources)       │   │
│  │  Picking: picking_system, pointer_events                         │   │
│  │  动画: apply_animations                                          │   │
│  │  资产: Assets, AssetEvent, GpuTexture                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

**推荐做法**:
- 平台适配: 根据目标平台实现 `Window` trait
- 提取器: 复制模板，修改组件类型
- 组件: 实现 `AnimationReceiver` 映射属性
- 游戏逻辑: 使用 FHRE 提供的核心函数组装
- 输入桥接: 等待 FHRE 提供 `InputPlugin`，或手动实现

---

## 重要限制

### 1. 内存对齐 (v2.8.1 已修复)

**问题**: `Cube` 包含 `[Option<Handle<Image>>; 6]`，需要 16 字节对齐，但 `malloc()` 只保证 8 字节。

**解决**: `CAllocator` 使用 `aligned_alloc()` 处理大于 8 字节对齐。

```rust
unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    if layout.align() <= 8 {
        malloc(layout.size()) as *mut u8
    } else {
        aligned_alloc(layout.align(), layout.size()) as *mut u8
    }
}
```

### 2. Pickable::default() 陷阱

```rust
// ❌ 错误：is_hoverable=false
.insert(Pickable::default())

// ✅ 正确
.insert(Pickable::DEFAULT)
```

### 3. 栈溢出风险

```rust
// ❌ 危险：64KB 栈数组
let blend_lut: [[u8; 256]; 256] = [[0; 256]; 256];

// ✅ 安全：堆分配
let blend_lut: Box<[[u8; 256]]> = /* ... */;
```

---

## 版本历史

### v2.8.2 (当前)
- **架构文档重构**: 明确 FHRE 与 Bevy 的核心差异 (no_std、零依赖、外部窗口)
- **责任边界划分**: 区分平台相关 (应用层) vs 平台无关 (FHRE 库)
- **输入系统分层**: 明确哪些可自动化 (InputPlugin) vs 必须手动 (Window trait)
- **核心循环自动化建议**: 时间、调度、同步、提取、渲染应该在 FHRE 库自动完成

### v2.8.1
- **内存对齐修复**: `CAllocator` 使用 `aligned_alloc()`
- **模型切换动画修复**: SoccerBall 正确使用 `player` 变量
- **调试打印清理**

### v2.8.0
- **统一 Query 系统**: QueryData/QueryFilter 架构
- **变更检测**: `Mut<T>`, `Ref<T>`
- **自动纹理上传**: TextureAssetPlugin

### v2.7.0
- RenderPhase + PhaseItem 自动排序
- ExtractComponent trait
- AssetServer 简化版

### v2.6.0
- Asset 系统 (Handle, Assets, AssetEvent)
- 纹理映射功能

---

## 未来规划

### 短期
- [ ] 字体渲染系统
- [ ] 纹理图集 (TextureAtlas)
- [ ] 抗锯齿 (AA)

### 中期
- [ ] GPU 后端 (OpenGL ES)
- [ ] 着色器系统
- [ ] `Option<&T>` 查询支持

### 长期
- [ ] Vulkan 后端
- [ ] 多线程渲染
- [ ] Query derive macro

---

## 参考

- Bevy 源码: `/home/uan/develop/FeatherOS-code/third/bevy`
- Bevy 版本: 0.19.0-dev
- Bevy 仓库: https://github.com/bevyengine/bevy
