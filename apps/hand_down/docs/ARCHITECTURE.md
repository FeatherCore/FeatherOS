# FHRE 架构文档（历史参考）

> 当前新方向是将 FHRE 合入 Wing，形成单 crate 的轻量声明式 ECS UI/渲染系统。
> 新架构基线见 `docs/WING_UNIFIED_ARCHITECTURE.md`。
> 本文保留为旧 FHRE 独立引擎设计的历史参考，不再作为后续 Wing 实现的主约束。

## 概述

FHRE (Feather Hybrid Render Engine) 是一个轻量级的纯 3D 渲染引擎，设计用于嵌入式系统 (no_std)。采用 Bevy 风格的双世界 ECS 架构。

**版本**: 2.8.3  
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
                                    │ InputPlugin::bridge() (FHRE 提供)
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

// 平台层实现 InputBridge (映射平台按键码)
impl InputBridge for MyInputBridge {
    fn map_mouse_button(&self, button: u32) -> Option<MouseButton> {
        match button {
            1 => Some(MouseButton::Left),
            // ...
        }
    }
}

// 使用 PlatformInputPlugin 组合 InputBridge
let input_plugin = PlatformInputPlugin::new(MyInputBridge);
app.run(&mut window, &input_plugin, 16);
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
    
    // 使用 App::run() 自动化主循环
    let input_plugin = PlatformInputPlugin::new(framebuffer::InputAdapter);
    app.run(&mut window, &input_plugin, 16);
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
| **反射系统** | 无 | 有 (bevy_reflect) | 无 derive 宏，需手动实现 trait |
| **多线程** | 无 | 有 (bevy_tasks) | 单线程简化设计 |

### 必须手动实现的设计

基于 FHRE 与 Bevy 的核心差异，以下功能**必须由应用层手动实现**：

---

#### 1. Window Trait 实现（平台相关）

**原因**: 输出目标和输入设备由硬件平台决定，嵌入式系统没有统一的窗口系统。

**Bevy 自动化方式**:
```rust
// bevy/crates/bevy_winit/src/lib.rs
impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        // 自动创建 winit EventLoop
        let event_loop = EventLoop::<WinitUserEvent>::with_user_event().build();
        
        // 设置 runner，自动管理窗口生命周期
        app.set_runner(|app| winit_runner(app, event_loop));
    }
}

// bevy_winit 自动处理:
// - X11/Wayland/Windows/macOS 窗口创建
// - 事件循环管理
// - 输入事件收集
// - 帧缓冲呈现
```

**FHRE 手动实现**:
```rust
// apps/examples/fhre/rust/src/platform/framebuffer.rs
pub struct Window {
    fb_fd: c_int,      // /dev/fb0 (平台相关)
    input_fd: c_int,   // /dev/input0 (平台相关)
    kbd_fd: c_int,     // /dev/kbd (平台相关)
    fb_ptr: *mut u32,  // framebuffer 内存映射
}

impl WindowTrait for Window {
    fn collect_input_events(&mut self) -> WindowInputEvents {
        // 必须手动: 从平台设备读取
        unsafe {
            read(self.input_fd, &mut sample, size);  // 触摸事件
            read(self.kbd_fd, &mut kbd_event, size); // 键盘事件
        }
    }
    
    fn present(&mut self, framebuffer: &[u32]) {
        // 必须手动: 输出到平台设备
        unsafe {
            core::ptr::copy_nonoverlapping(framebuffer.as_ptr(), self.fb_ptr, size);
        }
    }
}
```

**不同平台的实现差异**:
| 平台 | Framebuffer | 输入设备 | 代码修改点 |
|------|------------|---------|-----------|
| NuttX SIM | `/dev/fb0` (X11) | `/dev/input0` (X11 mouse) | 设备路径、IOCTL 常量 |
| 裸机 STM32 | LCD 寄存器 | GPIO 触摸屏 | 完全重写 Window 实现 |
| ESP32 | SPI LCD | I2C 触摸屏 | 完全重写 Window 实现 |
| Linux 帧缓冲 | `/dev/fb0` | `/dev/input/event0` | 设备路径 |

---

#### 2. InputBridge 实现（平台相关）

**原因**: 不同平台的按键码、鼠标按钮编码不同，需要应用层映射。

**Bevy 自动化方式**:
```rust
// bevy_winit 自动转换 winit 按键码
use winit::event::VirtualKeyCode;

// winit 提供跨平台按键码，Bevy 自动映射
// 无需应用层干预
```

**FHRE 手动实现**:
```rust
// apps/examples/fhre/rust/src/platform/framebuffer.rs
pub struct InputAdapter;

impl InputBridge for InputAdapter {
    fn map_keycode(&self, code: u32) -> Option<KeyCode> {
        // 必须手动: X11 按键码映射
        match code {
            0x0020 => Some(KeyCode::Space),      // X11 Space
            0x0072 => Some(KeyCode::KeyR),       // X11 R
            0xff1b => Some(KeyCode::Escape),     // X11 Escape
            _ => None,
        }
    }
    
    fn map_mouse_button(&self, btn: u32) -> Option<MouseButton> {
        // 必须手动: 鼠标按钮映射
        match btn {
            1 => Some(MouseButton::Left),
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::Right),
            _ => None,
        }
    }
}
```

**不同平台的映射差异**:
| 平台 | 按键码来源 | 映射方式 |
|------|-----------|---------|
| NuttX SIM | X11 keycode | switch-case 映射 |
| 裸机 | GPIO 扫描码 | 硬件相关查表 |
| ESP32 | I2C 触摸值 | 直接映射或查表 |

---

#### 3. 提取器 (Extractors) 实现（应用相关）

**原因**: 提取器需要知道具体的组件类型，无法泛化。

**Bevy 自动化方式**:
```rust
// bevy_render 提供泛型提取器 + derive 宏
#[derive(Component, ExtractComponent)]
struct Position(Vec3);

// 自动生成提取代码
impl ExtractComponent for Position {
    fn extract(&self) -> Self { self.clone() }
}
```

**FHRE 手动实现**:
```rust
// apps/examples/fhre/rust/src/extract.rs
pub fn extract_3d_components(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // 必须手动: 查询具体组件类型
    let cube_query = main_world.query::<(Entity, &Transform, &Cube)>();
    let soccer_query = main_world.query::<(Entity, &Transform, &SoccerBall)>();
    
    // 必须手动: 转换为渲染数据
    for (entity, transform, cube) in cube_query.iter() {
        let render_entity = render_world.get_or_spawn_synced(entity);
        let mesh = ExtractedMesh {
            vertices: cube.get_vertices().to_vec(),
            faces: cube.get_faces().iter().map(|f| f.to_vec()).collect(),
            // ...
        };
        render_world.insert_component(render_entity, mesh);
    }
}
```

**为什么不能自动化**:
1. FHRE 没有反射系统，无法在运行时获取组件类型信息
2. 提取逻辑需要知道组件的具体字段
3. 渲染数据格式由应用决定

---

#### 4. 组件定义（应用相关）

**原因**: 游戏对象由应用定义，引擎无法预知。

**Bevy 自动化方式**:
```rust
// derive 宏自动实现 Component trait
#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

// 自动实现:
// - Component trait
// - Reflect (可选)
// - 注册到类型注册表
```

**FHRE 手动实现**:
```rust
// apps/examples/fhre/rust/src/components/cube.rs
pub struct Cube {
    pub size: f32,
    pub face_colors: [Color; 6],
    pub face_textures: [Option<Handle<Image>>; 6],
    pub rotation: Vec3,
}

// 必须手动实现 Component trait
impl Component for Cube {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_boxed(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

// 必须手动实现 AnimationReceiver (如果需要动画)
impl AnimationReceiver for Cube {
    fn receive_animation(&mut self, property: AnimationProperty, value: f32) {
        match property {
            AnimationProperty::RotationX => self.rotation.x = value,
            AnimationProperty::RotationY => self.rotation.y = value,
            // ...
        }
    }
}
```

---

#### 5. 游戏逻辑系统（应用相关）

**原因**: 行为由应用定义，引擎无法预知。

**Bevy 自动化方式**:
```rust
// 系统参数自动注入
fn move_system(
    mut query: Query<(&mut Position, &Velocity)>,
    time: Res<Time>,
) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0 * time.delta_seconds();
    }
}

// 自动注册
app.add_systems(Update, move_system);
```

**FHRE 手动实现**:
```rust
// apps/examples/fhre/rust/src/lib.rs
fn input_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DemoState>,
    mut button_query: Query<&mut Button>,
) {
    // 必须手动: 游戏逻辑
    for (_, button) in button_query.iter_mut() {
        if button.clicked {
            match button.text.as_str() {
                "Prev" => state.current_model = (state.current_model + 1) % 2,
                "Next" => state.current_model = (state.current_model + 1) % 2,
                _ => {}
            }
        }
    }
}

// 必须手动: 声明系统参数
app.add_systems(PreUpdate, declare_system!(input_system; 
    key_input: Res<ButtonInput<KeyCode>>,
    state: ResMut<DemoState>,
    button_query: Query<&mut Button>,
));
```

---

#### 6. 资产加载（平台相关）

**原因**: 存储介质由平台决定。

**Bevy 自动化方式**:
```rust
// bevy_asset 提供异步资产服务器
let handle: Handle<Image> = asset_server.load("textures/player.png");

// 自动:
// - 异步加载
// - 热重载
// - 依赖追踪
```

**FHRE 手动实现**:
```rust
// apps/examples/fhre/rust/src/lib.rs
fn setup_textures(mut images: ResMut<Assets<Image>>, mut events: ResMut<Events>) {
    // 必须手动: 从代码内嵌数据创建纹理
    let texture_data = include_bytes!("texture.raw");
    let image = Image::from_raw(texture_data, width, height);
    let handle = images.add_with_event(image, &mut events);
}
```

**不同平台的资产来源**:
| 平台 | 资产来源 | 加载方式 |
|------|---------|---------|
| NuttX SIM | 编译时内嵌 | `include_bytes!` |
| 裸机 | Flash ROM | 链接脚本 + 指针读取 |
| ESP32 | SPIFFS/LittleFS | 文件系统 API |

---

### 可以自动化的设计（平台无关）

以下功能 FHRE 已在核心库自动化，应用层无需实现：

#### 1. 时间系统
```rust
// apps/fhre/rust/src/app/app.rs
pub fn update_and_render(&mut self) {
    // 自动更新时间
    if let Some(time) = self.main_world.resources_mut().get_mut::<Time>() {
        time.update(DEFAULT_FRAME_TIME);
    }
}
```

#### 2. 实体同步
```rust
// apps/fhre/rust/src/sync/sync_system.rs
pub fn entity_sync_system(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // 自动同步 MainWorld → RenderWorld
}
```

#### 3. 输入事件分发
```rust
// apps/fhre/rust/src/window/mod.rs
pub trait InputPlugin: Plugin {
    fn bridge(&self, app: &mut App, events: &WindowInputEvents);
}

// 应用层只需实现 InputBridge 映射，分发逻辑已自动化
```

#### 4. Picking 系统
```rust
// apps/fhre/rust/src/picking/system.rs
pub fn picking_system(
    mouse_pos: Res<MousePosition>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    // ...
) {
    // 自动检测碰撞、更新 HoverMap、生成事件
}
```

#### 5. 动画系统
```rust
// apps/fhre/rust/src/animation/mod.rs
pub fn apply_animations<T: AnimationReceiver>(
    player_query: Query<&AnimationPlayer>,
    mut target_query: Query<&mut T>,
) {
    // 自动应用动画到组件
}
```

#### 6. 主循环
```rust
// apps/fhre/rust/src/app/app.rs
pub fn run<W: Window, I: InputPlugin>(&mut self, window: &mut W, input_plugin: &I, ...) {
    loop {
        let events = window.collect_input_events();
        input_plugin.bridge(self, &events);
        self.update_and_render();
        window.present(self.framebuffer());
    }
}
```

---

### 架构对比：App 结构

**Bevy App 结构** (复杂):
```rust
// bevy/crates/bevy_app/src/app.rs
pub struct App {
    pub(crate) sub_apps: SubApps,  // 支持多个 SubApp
    pub(crate) runner: RunnerFn,   // 可替换的 runner
}

// SubApps 包含 MainWorld + 自定义 SubApp (如 RenderApp)
pub struct SubApps {
    pub main: SubApp,
    pub sub_apps: HashMap<InternedAppLabel, SubApp>,
}

// Bevy 的 run() 使用 runner 模式
pub fn run(&mut self) -> AppExit {
    let runner = core::mem::replace(&mut self.runner, Box::new(run_once));
    (runner)(app)  // runner 负责事件循环
}

// WinitPlugin 设置 runner
impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        app.set_runner(|app| winit_runner(app, event_loop));
    }
}
```

**FHRE App 结构** (简化):
```rust
// apps/fhre/rust/src/app/app.rs
pub struct App {
    pub main_world: MainWorld,
    pub render_world: RenderWorld,
    plugins: Vec<Box<dyn Plugin>>,
    extractors: Extractors,
}

// FHRE 的 run() 直接内嵌主循环
pub fn run<W: Window, I: InputPlugin>(
    &mut self,
    window: &mut W,        // 外部传入
    input_plugin: &I,      // 外部传入
    frame_delay_ms: u32,
) {
    loop {
        let events = window.collect_input_events();
        if !window.is_running() { break; }
        
        input_plugin.bridge(self, &events);
        self.update_and_render();
        window.present(self.framebuffer());
        usleep(frame_delay_ms * 1000);
    }
}
```

**关键差异**:
| 特性 | Bevy | FHRE |
|------|------|------|
| SubApp 支持 | ✅ 多个 SubApp | ❌ 仅 MainWorld + RenderWorld |
| Runner 模式 | ✅ 可替换 | ❌ 固定循环 |
| 窗口管理 | 内置 winit | 外部传入 Window trait |
| 事件循环 | winit EventLoop | 裸机 while loop |

### 架构对比：窗口系统

**Bevy Window** (完整窗口管理):
```rust
// bevy/crates/bevy_window/src/window.rs
#[derive(Component)]
pub struct Window {
    pub present_mode: PresentMode,
    pub mode: WindowMode,
    pub position: WindowPosition,
    pub resolution: WindowResolution,
    pub title: String,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub focused: bool,
    // ... 30+ 字段
}

// bevy_winit 负责创建和管理窗口
impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        let event_loop = EventLoop::<WinitUserEvent>::with_user_event().build();
        app.set_runner(|app| winit_runner(app, event_loop));
    }
}
```

**FHRE Window** (trait 抽象):
```rust
// apps/fhre/rust/src/window/mod.rs
pub trait Window {
    fn is_running(&self) -> bool;
    fn collect_input_events(&mut self) -> WindowInputEvents;
    fn present(&mut self, framebuffer: &[u32]);
    fn dimensions(&self) -> (u32, u32);
}

// 应用层实现 (平台相关)
// apps/examples/fhre/rust/src/platform/framebuffer.rs
pub struct Window {
    width: u32,
    height: u32,
    fb_fd: c_int,      // /dev/fb0
    fb_ptr: *mut u32,  // framebuffer 内存
    input_fd: c_int,   // /dev/input0
    kbd_fd: c_int,     // /dev/kbd
}

impl WindowTrait for Window {
    fn collect_input_events(&mut self) -> WindowInputEvents {
        // 从 /dev/input0 读取触摸事件
        // 从 /dev/kbd 读取键盘事件
    }
    
    fn present(&mut self, framebuffer: &[u32]) {
        // 复制到 framebuffer
        core::ptr::copy_nonoverlapping(framebuffer.as_ptr(), self.fb_ptr, size);
    }
}
```

**设计决策**:
- **Bevy**: 窗口是引擎核心功能，由 `bevy_winit` 自动管理
- **FHRE**: 窗口是平台相关，由应用层实现 `Window` trait

### 架构对比：输入系统

**Bevy 输入系统** (事件驱动):
```rust
// bevy_winit 通过事件循环接收输入
fn winit_runner(app: App, event_loop: EventLoop) -> AppExit {
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::MouseInput { button, state, .. } => {
                        // 发送到 ECS Events
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        // 发送到 ECS Events
                    }
                }
            }
        }
    });
}

// bevy_input 提供 ButtonInput<T> 资源
app.insert_resource(ButtonInput::<KeyCode>::default());
app.insert_resource(ButtonInput::<MouseButton>::default());
```

**FHRE 输入系统** (轮询 + 桥接):
```rust
// FHRE 核心: InputPlugin trait
pub trait InputPlugin: Plugin {
    fn bridge(&self, app: &mut App, events: &WindowInputEvents);
}

// 应用层: InputBridge 映射平台按键码
pub trait InputBridge {
    fn map_keycode(&self, platform_keycode: u32) -> Option<KeyCode>;
    fn map_mouse_button(&self, platform_button: u32) -> Option<MouseButton>;
}

// 应用层: PlatformInputPlugin 实现
impl<B: InputBridge> InputPlugin for PlatformInputPlugin<B> {
    fn bridge(&self, app: &mut App, events: &WindowInputEvents) {
        // 1. 映射键盘事件 → ButtonInput<KeyCode>
        for event in &events.keyboard_events {
            if let Some(kc) = self.bridge.map_keycode(event.keycode) {
                if event.pressed { key_input.press(kc); }
                else { key_input.release(kc); }
            }
        }
        // 2. 映射鼠标事件 → ButtonInput<MouseButton>
        // 3. 更新 MousePosition
    }
}

// 使用
let input_plugin = PlatformInputPlugin::new(InputAdapter);
app.run(&mut window, &input_plugin, 16);
```

**设计决策**:
| 层次 | Bevy | FHRE |
|------|------|------|
| 事件来源 | winit EventLoop | Window::collect_input_events() |
| 事件分发 | 自动 (winit → ECS Events) | 手动 (InputPlugin::bridge) |
| 按键映射 | winit 内置 | 应用层 InputBridge |
| 坐标系统 | 自动转换 | 直接使用屏幕坐标 |

### 架构对比：渲染系统

**Bevy 渲染** (GPU + SubApp):
```rust
// bevy_render 作为 SubApp
let render_app = SubApp::new();
app.insert_sub_app(RenderApp, render_app);

// 渲染阶段
pub enum RenderSystems {
    ExtractCommands,    // 提取
    PrepareAssets,      // 准备资源
    Queue,              // 队列
    PhaseSort,          // 排序
    Render,             // 渲染
    Cleanup,            // 清理
}

// GPU 渲染
render_pass.set_pipeline(pipeline);
render_pass.draw_indexed(indices);
```

**FHRE 渲染** (CPU 软件):
```rust
// apps/fhre/rust/src/render_world/world.rs
pub struct RenderWorld {
    framebuffer: Vec<u32>,  // CPU 帧缓冲
    commands: Vec<RenderCommand>,
}

pub enum RenderCommand {
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawPolygon { vertices: Vec<Vec2>, color: Color },
    DrawPolygonTextured { vertices, uvs, texture_id, color },
}

// CPU 软件渲染
impl SoftwareBackend {
    pub fn fill_polygon(&mut self, vertices: &[Vec2], color: Color) {
        // 扫描线填充算法
    }
}
```

**设计决策**:
- **Bevy**: GPU 渲染，需要 wgpu，支持多后端
- **FHRE**: CPU 软件渲染，无 GPU 依赖，适合嵌入式

### 设计决策总结

| 功能 | Bevy | FHRE | 原因 |
|------|------|------|------|
| 窗口创建 | 自动 (winit) | 手动 (Window trait) | 嵌入式无统一窗口系统 |
| 事件循环 | 自动 (EventLoop) | 手动 (while loop) | 嵌入式无 OS 事件系统 |
| 按键映射 | 自动 (winit) | 手动 (InputBridge) | 不同平台按键码不同 |
| 提取器 | 泛型 (derive) | 手动 | 无反射系统 |
| 组件定义 | derive | 手动 | 无反射系统 |
| 时间系统 | 自动 | 自动 | 平台无关 |
| 实体同步 | 自动 | 自动 | 平台无关 |
| Picking | 自动 | 自动 | 平台无关 |
| 动画 | 自动 | 自动 | 平台无关 |

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

| 功能 | 当前状态 | 实现位置 |
|------|----------|----------|
| **时间系统** | ✅ 已自动化 | `App::update_and_render()` |
| **实体同步** | ✅ 已自动化 | `entity_sync_system()` |
| **提取调度** | ✅ 已自动化 | `App::update_and_render()` |
| **事件更新** | ✅ 已自动化 | `App::run()` |
| **输入事件分发** | ✅ 已自动化 | `InputPlugin` trait + `PlatformInputPlugin` |
| **主循环** | ✅ 已自动化 | `App::run()` |

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
│                    FHRE 库 (已自动化)                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  InputPlugin (v2.8.3 已实现)                                     │   │
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

**v2.8.3 已实现 `App::run()`**:
```rust
// FHRE 库提供:
impl App {
    pub fn run<W: Window, I: InputPlugin>(
        &mut self,
        window: &mut W,
        input_plugin: &I,
        frame_delay_ms: u32,
    ) {
        loop {
            let events = window.collect_input_events();  // 平台层采集
            if !window.is_running() { break; }
            
            input_plugin.bridge(self, &events);          // FHRE 分发
            self.update_and_render();                     // FHRE 核心循环
            window.present(self.framebuffer());           // 平台层输出
            events.update();                              // 清理事件
            usleep(frame_delay_ms * 1000);                // 帧延迟
        }
    }
}

// 应用层只需:
let input_plugin = PlatformInputPlugin::new(InputAdapter);
app.run(&mut window, &input_plugin, 16);
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

#### 平台适配层 (platform/)

| 文件 | 职责 | 代码量 | 必须手动的原因 |
|------|------|--------|----------------|
| `framebuffer.rs` | Window trait 实现 | ~360 行 | 设备路径、IOCTL 调用是平台相关的 |
| `runner.rs` | InputBridge + PlatformInputPlugin | ~120 行 | 按键码映射是平台相关的 |
| `input/mod.rs` | KeyCode, MouseButton 定义 | ~100 行 | 输入类型由平台决定 |

**framebuffer.rs 关键实现**:
```rust
// 必须手动: 平台相关的设备路径
const FB_PATH: &[u8] = b"/dev/fb0\0";
const INPUT_PATH: &[u8] = b"/dev/input0\0";
const KBD_PATH: &[u8] = b"/dev/kbd\0";

// 必须手动: 平台相关的 IOCTL 常量
const FBIOGET_VIDEOINFO: c_int = 0x2801;
const FBIOGET_PLANEINFO: c_int = 0x2802;

// 必须手动: 平台相关的按键码映射
impl InputBridge for InputAdapter {
    fn map_keycode(&self, code: u32) -> Option<KeyCode> {
        match code {
            0x0020 => Some(KeyCode::Space),   // X11 Space
            0xff1b => Some(KeyCode::Escape),  // X11 Escape
            _ => None,
        }
    }
}
```

#### 应用逻辑层 (components/, extract.rs, lib.rs)

| 文件 | 职责 | 代码量 | 必须手动的原因 |
|------|------|--------|----------------|
| `components/cube.rs` | Cube 组件 + AnimationReceiver | ~200 行 | 游戏对象由应用定义 |
| `components/soccer_ball.rs` | SoccerBall 组件 | ~250 行 | 游戏对象由应用定义 |
| `components/button.rs` | Button 组件 | ~100 行 | UI 组件由应用定义 |
| `extract.rs` | 提取器实现 | ~200 行 | 需知道具体组件类型 |
| `lib.rs` | 系统注册、游戏逻辑 | ~900 行 | 行为由应用定义 |

**extract.rs 关键实现**:
```rust
// 必须手动: 查询具体组件类型
pub fn extract_3d_components(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let cube_query = main_world.query::<(Entity, &Transform, &Cube)>();
    let soccer_query = main_world.query::<(Entity, &Transform, &SoccerBall)>();
    
    // 必须手动: 转换为渲染数据
    for (entity, transform, cube) in cube_query.iter() {
        render_world.insert_component(entity, ExtractedMesh::from_cube(cube, transform));
    }
}

// 必须手动: 队列渲染命令
pub fn queue_meshes(render_world: &mut RenderWorld) {
    let meshes = render_world.query::<&ExtractedMesh>();
    for mesh in meshes.iter() {
        render_world.add_command(RenderCommand::DrawPolygon { ... });
    }
}
```

**lib.rs 关键实现**:
```rust
// 必须手动: 系统注册
app.add_systems(Startup, declare_system!(setup_textures; ...))
   .add_systems(Startup, declare_system!(setup; ...))
   .add_systems(PreUpdate, declare_system!(picking_system; ...))
   .add_systems(Update, declare_system!(input_system; ...));

// 必须手动: 游戏逻辑
fn input_system(key_input: Res<ButtonInput<KeyCode>>, mut state: ResMut<DemoState>) {
    if key_input.just_pressed(KeyCode::Space) {
        state.is_rotating = !state.is_rotating;
    }
}
```

### FHRE 库自动化 (平台无关)

| 功能 | 实现位置 | 代码量 |
|------|----------|--------|
| 时间系统 | `App::update_and_render()` | ~10 行 |
| 实体同步 | `sync/entity_sync_system()` | ~50 行 |
| 提取调度 | `App::update_and_render()` | ~20 行 |
| 输入分发 | `InputPlugin::bridge()` | ~60 行 |
| Picking | `picking/system.rs` | ~150 行 |
| 动画 | `animation/mod.rs` | ~100 行 |
| 主循环 | `App::run()` | ~30 行 |

### 代码量分布

```
示例应用总代码量: ~2,770 行

平台适配层 (必须手动):
├── framebuffer.rs    ~360 行 (13%)
├── runner.rs         ~120 行 (4%)
└── input/            ~100 行 (4%)
                      ~580 行 (21%)

应用逻辑层 (必须手动):
├── components/       ~550 行 (20%)
├── extract.rs        ~200 行 (7%)
└── lib.rs            ~900 行 (32%)
                      ~1650 行 (59%)

FHRE 库自动化:
├── 时间系统          ~10 行
├── 实体同步          ~50 行
├── 输入分发          ~60 行
├── Picking          ~150 行
├── 动画             ~100 行
└── 主循环            ~30 行
                      ~400 行 (14%)  ← 应用层无需编写
```

### 开发流程

1. **复制模板**: 从 `apps/examples/fhre_template` 复制
2. **修改平台适配**: 
   - 修改 `framebuffer.rs` 中的设备路径和 IOCTL
   - 修改 `runner.rs` 中的按键映射
3. **定义组件**: 在 `components/` 中定义游戏对象
4. **实现提取器**: 在 `extract.rs` 中提取渲染数据
5. **编写游戏逻辑**: 在 `lib.rs` 中注册系统和编写逻辑

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

### v2.8.3 (当前)
- **InputPlugin trait**: FHRE 核心提供 `InputPlugin` trait，平台层实现 `PlatformInputPlugin`
- **App::run()**: 自动化主循环，包含输入桥接、更新、渲染、呈现、帧延迟
- **简化应用层代码**: `WindowRunner` → `app.run(&mut window, &input_plugin, frame_delay_ms)`

### v2.8.2
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
