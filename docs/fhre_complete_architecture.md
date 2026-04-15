# FHRE 完整架构文档

## 目录

1. [概述](#1-概述)
2. [双世界架构](#2-双世界架构)
3. [默认资源系统](#3-默认资源系统)
4. [摄像机与视图系统](#4-摄像机与视图系统)
5. [渲染管线](#5-渲染管线)
6. [模块详解](#6-模块详解)
7. [控件系统](#7-控件系统)
8. [NuttX SIM 平台支持](#8-nuttx-sim-平台支持)
9. [使用示例](#9-使用示例)

---

## 1. 概述

Feather Hybrid Render Engine (FHRE) v2.0 是一个基于声明式双世界架构的轻量级渲染引擎，专为嵌入式系统设计。

### 核心特性

- **声明式双世界架构** (Main World + Render World)
- **ECS (Entity-Component-System)** 设计
- **默认资源系统** (主屏幕 + 默认UI摄像机)
- **模块化系统调度**
- **嵌入式友好的无标准库实现**
- **NuttX SIM 平台支持** (类似 LVGL 的 framebuffer 对接)
- **文件夹/mod.rs 模块化结构**

### 设计哲学

```
开箱即用，可扩展覆盖
─────────────────────
App::new() 自动提供：
- PrimaryScreen (主屏幕)
- Default UI Camera (默认UI摄像机)

用户可以：
- 直接使用默认资源
- 创建额外资源
- 覆盖默认资源
```

---

## 2. 双世界架构

### 2.1 架构概览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FHRE 双世界架构 v2.0                               │
│                                                                             │
│  ┌─────────────────────────────┐        ┌─────────────────────────────┐    │
│  │       Main World            │        │       Render World          │    │
│  │      (游戏逻辑世界)          │        │       (渲染世界)             │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Entities (3D)       │    │        │  │ RenderCommands      │    │    │
│  │  │ - Transform3D       │    │        │  │ - DrawTriangle      │    │    │
│  │  │ - BookPage          │    │        │  │ - DrawRect          │    │    │
│  │  │ - Sprite            │    │        │  │ - (2D screen coords)│    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Systems             │    │        │  │ Pipeline            │    │    │
│  │  │ - PreUpdate         │    │        │  │ - SoftwareBackend   │    │    │
│  │  │ - Update            │    │        │  │ - Execute Commands  │    │    │
│  │  │ - PostUpdate        │    │        │  │ - Output Framebuffer│    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Resources           │    │        │  │ Framebuffer         │    │    │
│  │  │ - Time              │    │        │  │ - RGBA32 pixels     │    │    │
│  │  │ - PrimaryScreen     │    │        │  │ - width x height    │    │    │
│  │  │ - DefaultUiCamera   │    │        │  │                     │    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  └─────────────┬───────────────┘        └─────────────┬───────────────┘    │
│                │                                      │                     │
│                │          Extract Phase               │                     │
│                │    (3D World → 2D Screen Projection) │                     │
│                └──────────────────────────────────────▶                     │
│                                                                             │
│  核心设计原则:                                                              │
│  1. Main World: 声明"是什么" (3D 位置、旋转角度)                            │
│  2. Extract:    计算"怎么变" (3D变换 → 投影 → 2D屏幕坐标)                   │
│  3. Render:     执行"怎么画" (绘制三角形到 framebuffer)                     │
│                                                                             │
│  注: FHRE 的最小单位(Node)默认具有 3D 属性，2D 是 3D 的特例 (Z=0)           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 执行流程

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Startup    │───▶│    Update    │───▶│   Extract    │───▶│    Render    │
│   (一次)     │    │  (每帧执行)   │    │  (数据同步)   │    │  (生成画面)   │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
                           │                   │                   │
                           ▼                   ▼                   ▼
                    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
                    │ System 执行   │    │ Camera3D →   │    │ 渲染到屏幕    │
                    │ 游戏逻辑更新  │    │ View 转换    │    │ Present      │
                    └──────────────┘    └──────────────┘    └──────────────┘
```

---

## 3. 默认资源系统

### 3.1 核心设计

FHRE 提供内置的默认资源，简化应用开发：

1. **PrimaryScreen** - 主屏幕/窗口，全局分辨率定义
2. **Default UI Camera** - 默认 UI 摄像机，正交投影

这些资源在 `App::new()` 时自动创建。

### 3.2 默认资源架构

```
App::new() 自动创建：
┌─────────────────────────────────────────────────────────────────────┐
│ 1. PrimaryScreen (主屏幕)                                           │
│    ├── width: 800 (或实际显示分辨率)                                  │
│    ├── height: 600 (或实际显示分辨率)                                 │
│    ├── viewport: (0, 0, width, height)                              │
│    └── pixel_density: 1.0                                           │
│                                                                     │
│    这是 FHRE 的全局主窗口，所有渲染默认输出到这里                      │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ 2. Default UI Camera (默认 UI 摄像机)                                │
│    Entity: default_ui_camera                                        │
│    ├── Node { node_type: Camera }                                   │
│    ├── Transform3D {                                                │
│    │       position: (width/2, height/2, 100)  // 屏幕中心上方       │
│    │     }                                                          │
│    └── Camera3D {                                                   │
│            orthographic: true,            // 正交投影                │
│            orthographic_size: height/2,   // 视口范围                │
│            depth: -100,                   // 最先渲染                │
│            viewport: (0, 0, 1, 1)         // 全屏                    │
│        }                                                            │
│                                                                     │
│    这是默认的 UI 摄像机，所有 UI 元素默认使用这个摄像机渲染            │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.3 PrimaryScreen (主屏幕)

```rust
/// Primary Screen Resource
///
/// This is the default render target for FHRE.
/// It defines the main window/screen dimensions and properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimaryScreen {
    pub width: u32,              // Screen width in pixels
    pub height: u32,             // Screen height in pixels
    pub viewport: Rect,          // Full screen viewport
    pub pixel_density: f32,      // For high-DPI displays
    pub fullscreen: bool,
}

impl PrimaryScreen {
    pub fn new(width: u32, height: u32) -> Self
    pub fn dimensions(&self) -> (u32, u32)
    pub fn center(&self) -> Vec2
    pub fn aspect_ratio(&self) -> f32
    pub fn contains(&self, point: Vec2) -> bool
    pub fn normalized_to_screen(&self, normalized: Vec2) -> Vec2
}
```

### 3.4 Default UI Camera

```rust
/// Resource to store the default UI camera entity
#[derive(Clone, Copy, Debug)]
pub struct DefaultUiCamera {
    pub entity: Entity,
}

impl crate::resources::Resource for DefaultUiCamera {}
```

**创建逻辑：**

```rust
fn setup_default_ui_camera(&mut self, width: u32, height: u32) {
    let camera_entity = self.main_world.spawn();
    
    // 创建 UI camera node
    self.main_world.insert_component(
        camera_entity,
        Node::ui_control(NodeType::Camera)
    );
    
    // 位置: 屏幕中心上方
    self.main_world.insert_component(
        camera_entity,
        Transform3D::from_position(
            width as f32 / 2.0,   // Center X
            height as f32 / 2.0,  // Center Y
            100.0                 // Above the screen
        )
    );
    
    // 正交摄像机
    self.main_world.insert_component(
        camera_entity,
        Camera3D {
            orthographic: true,
            orthographic_size: height as f32 / 2.0,
            depth: -100,  // Render first
            viewport: Rect::new(0.0, 0.0, 1.0, 1.0),  // Fullscreen
            ..default()
        }
    );
    
    // 存储为默认 UI 摄像机资源
    self.main_world.resources_mut().insert(DefaultUiCamera {
        entity: camera_entity,
    });
}
```

### 3.5 使用示例

```rust
// 纯 UI 应用（使用默认资源）
let mut app = App::new();  // 自动创建主屏幕和默认 UI 摄像机
app.run();

// 游戏应用（添加游戏摄像机）
let mut app = App::new();
let game_cam = app.create_game_camera(
    Vec3::new(0.0, 5.0, -10.0),  // 位置
    Vec3::new(0.0, 0.0, 0.0),    // 看向原点
    60.0                          // FOV
);
app.run();
```

---

## 4. 摄像机与视图系统

### 4.1 两种摄像机概念

FHRE 中有两个层次的"摄像机"概念：

**概念 1: Main World 的 Camera3D 组件**
- 这是传统意义上的 3D 游戏摄像机
- 位于 3D 场景中的某个位置
- 决定"从什么角度观察 3D 世界"
- 将 3D 世界投影到 2D 屏幕

**概念 2: Render World 的 View（视图）**
- 这是渲染时的技术实现
- 由 Camera3D + Transform3D 计算得出
- 包含投影矩阵、视图矩阵等数学工具
- 每帧从 Camera3D 提取生成

### 4.2 核心区别

| 特性 | Camera3D (Main World) | View (Render World) |
|------|----------------------|---------------------|
| **本质** | 3D 场景中的实体 | 渲染用的数学工具 |
| **存在形式** | ECS 组件 | 运行时结构体 |
| **生命周期** | 持续存在 | 每帧重建 |
| **可动画** | ✅ 可以被动画系统控制 | ❌ 只读数据 |
| **用途** | 游戏逻辑控制视角 | GPU 渲染计算 |

### 4.3 工作流程

```
3D 场景 (Main World)          Extract          2D 屏幕 (Render World)
───────────────────                           ─────────────────────

摄像机在 (0, 2, -5)                              
看向原点 (0, 0, 0)              →               立方体投影到 (400, 300)
FOV 60°                                       球体投影到 (600, 250)
                                                
立方体在 (0, 0, 0)                              
球体在 (3, 0, 2)                               
```

### 4.4 视图系统 (View System)

```rust
/// View configuration - defines a render target
pub struct View {
    pub viewport: Rect,
    pub projection: Mat4,
    pub view: Mat4,
    pub view_projection: Mat4,
    pub camera_position: Vec3,
    pub near: f32,
    pub far: f32,
    pub orthographic: bool,
}

impl View {
    /// Create a new 2D orthographic view
    pub fn new_2d(width: f32, height: f32) -> Self
    
    /// Create a new 3D perspective view
    pub fn new_3d(width: f32, height: f32, fov_degrees: f32) -> Self
    
    /// Project world position to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec3) -> Option<(f32, f32)>
}

/// View bundle - combines view, target, and clear config
pub struct ViewBundle {
    pub view: View,
    pub target: ViewTarget,
    pub clear: ClearConfig,
}
```

### 4.5 摄像机控制与动画

摄像机作为普通 Node 实体，可以被动画系统完全控制：

```rust
// 第一人称视角（FPS）
fn fps_camera_control(
    mut camera_query: Query<(&mut Transform3D, &Camera3D), With<PlayerCamera>>,
    input: Res<Input>,
    time: Res<Time>,
) {
    for (mut transform, _camera) in camera_query.iter_mut() {
        // 鼠标控制视角旋转
        transform.rotation.y += input.mouse_delta.x * SENSITIVITY;
        transform.rotation.x -= input.mouse_delta.y * SENSITIVITY;
        
        // 限制垂直视角
        transform.rotation.x = transform.rotation.x.clamp(-PI/2.0, PI/2.0);
        
        // WASD 移动
        let forward = transform.forward();
        if input.key_pressed(Key::W) {
            transform.position += forward * MOVE_SPEED * time.delta();
        }
    }
}

// 第三人称视角（跟随角色）
fn third_person_camera(
    mut camera_query: Query<&mut Transform3D, With<FollowCamera>>,
    target_query: Query<&Transform3D, With<Player>>,
    time: Res<Time>,
) {
    let target = target_query.single();
    
    for mut camera_transform in camera_query.iter_mut() {
        // 计算目标位置（角色后方 + 上方）
        let offset = Vec3::new(0.0, 2.0, -5.0);
        let target_pos = target.position + target.rotation * offset;
        
        // 平滑跟随
        camera_transform.position = camera_transform.position.lerp(
            target_pos,
            5.0 * time.delta()
        );
        
        // 看向角色
        camera_transform.look_at(target.position);
    }
}
```

### 4.6 多视图渲染

```rust
// 场景：游戏主视角 + 小地图 + 后视镜（赛车游戏）
fn setup_racing_cameras(mut commands: Commands) {
    // 主驾驶视角
    commands.spawn((
        Node::game_entity(NodeType::Camera),
        Transform3D::from_position(0.0, 1.2, 0.5), // 驾驶座位置
        Camera3D::new()
            .with_fov(70.0)
            .with_viewport(Rect::new(0.0, 0.0, 1.0, 1.0)) // 全屏
            .with_depth(0),
        DriverCamera,
    ));
    
    // 小地图（俯视）
    commands.spawn((
        Node::ui_control(NodeType::Camera),
        Transform3D::from_position(0.0, 100.0, 0.0) // 高空俯视
            .with_rotation(-90.0_f32.to_radians(), 0.0, 0.0),
        Camera3D::new()
            .orthographic(50.0) // 正交投影
            .with_viewport(Rect::new(0.7, 0.7, 0.28, 0.28)) // 右上角
            .with_depth(1)
            .with_clear_config(ClearConfig::none()),
        MinimapCamera,
    ));
}
```

---

## 5. 渲染管线

### 5.1 架构分层

FHRE 的渲染管线采用三层架构，实现职责分离：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         渲染管线架构                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Layer 1: RenderWorld                                               │   │
│  │  - 渲染数据管理（命令队列、视图、阶段）                                │   │
│  │  - 封装渲染后端，提供统一接口                                         │   │
│  │  - 对外隐藏具体渲染实现                                               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Layer 2: Pipeline                                                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐ │   │
│  │  │  Software   │  │    GPU      │  │      Hybrid                 │ │   │
│  │  │  Backend    │  │  Backend    │  │   (未来)                    │ │   │
│  │  │  (CPU)      │  │  (OpenGL)   │  │                             │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────┘ │   │
│  │                                                                     │   │
│  │  - 渲染执行层，支持多后端切换                                         │   │
│  │  - 批处理、命令转换、资源管理                                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Layer 3: Framebuffer Output                                        │   │
│  │  - RGBA32 像素缓冲区                                                 │   │
│  │  - 输出到显示设备 (/dev/fb0)                                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 渲染阶段系统

FHRE 的渲染阶段系统借鉴 Bevy 的设计：

```
渲染顺序:

1. Background    →  清除/背景绘制
     ↓
2. Opaque2d      →  不透明 2D 物体（精灵、UI 背景）
     ↓
3. Opaque3d      →  不透明 3D 物体（模型）
     ↓
4. AlphaMask     →  Alpha 测试物体（镂空纹理）
     ↓
5. Transparent   →  透明物体（需要排序，从后往前）
     ↓
6. Ui            →  UI 覆盖层（最上层）
```

### 5.3 渲染阶段类型

```rust
pub enum RenderPhaseType {
    Background = 0,     // 背景/清除
    Opaque2d = 1,       // 不透明 2D 物体
    Opaque3d = 2,       // 不透明 3D 物体
    AlphaMask = 3,      // Alpha 测试物体
    Transparent = 4,    // 透明物体（需要排序）
    Ui = 5,             // UI 覆盖层
}

pub struct PhaseItem {
    pub sort_key: i32,
    pub z_depth: f32,
    pub entity_id: Option<u32>,
    pub draw_command_index: usize,
    pub batchable: bool,
    pub batch_key: u64,
}

pub struct RenderPhase {
    pub phase_type: RenderPhaseType,
    pub items: Vec<PhaseItem>,
    pub sorted: bool,
}
```

### 5.4 Pipeline 模块详解

#### 5.4.1 模块结构

```
pipeline/
├── mod.rs           # 模块导出
├── backend.rs       # SoftwareBackend (CPU 软件渲染)
├── renderer.rs      # Renderer trait (抽象接口)
└── batch.rs         # 批处理逻辑
```

#### 5.4.2 SoftwareBackend (CPU 软件渲染)

```rust
/// Software rendering backend - simulates a GPU pipeline on CPU
pub struct SoftwareBackend {
    framebuffer: Vec<u32>,
    width: u32,
    height: u32,
    viewport: Rect,
}

impl SoftwareBackend {
    /// Execute a single render command
    pub fn execute_command(&mut self, command: &RenderCommand) {
        match command {
            RenderCommand::Clear { color } => self.clear(*color),
            RenderCommand::DrawRect { rect, color } => self.fill_rect(*rect, *color),
            RenderCommand::DrawLine { start, end, color, thickness } => 
                self.draw_line(*start, *end, *color, *thickness),
            RenderCommand::DrawTriangle { p0, p1, p2, color } => 
                self.fill_triangle(*p0, *p1, *p2, *color),
            // ...
        }
    }

    /// Execute multiple commands in batch
    pub fn execute_commands(&mut self, commands: &[RenderCommand]) {
        for command in commands {
            self.execute_command(command);
        }
    }
}
```

#### 5.4.3 Renderer Trait (未来扩展)

```rust
/// Renderer trait - implemented by all rendering backends
pub trait Renderer {
    fn new(width: u32, height: u32) -> Self where Self: Sized;
    fn execute_commands(&mut self, commands: &[RenderCommand]);
    fn framebuffer(&self) -> &[u32];
    fn resize(&mut self, width: u32, height: u32);
    fn set_viewport(&mut self, rect: Rect);
    fn reset(&mut self);
}

pub enum RendererType {
    Software,   // Pure software CPU rendering
    Gpu,        // GPU accelerated rendering
    Hybrid,     // Hybrid: GPU for batches, CPU for small tasks
}
```

### 5.5 RenderWorld 与 Pipeline 集成

```rust
pub struct RenderWorld {
    // ... 其他字段 ...
    /// Rendering backend (encapsulated)
    backend: SoftwareBackend,  // Could be GPU backend in future
}

impl RenderWorld {
    /// Execute all pending render commands
    pub fn execute_render(&mut self) {
        self.backend.execute_commands(&self.commands);
    }

    /// Get framebuffer reference
    pub fn framebuffer(&self) -> &[u32] {
        self.backend.framebuffer()
    }
}
```

### 5.6 与 Bevy 的对比

| 特性 | Bevy | FHRE |
|------|------|------|
| 阶段类型 | Binned + Sorted | 简化版 Binned + Sorted |
| 批处理 | GPU 预处理 | CPU 批处理（嵌入式限制） |
| 多视图 | 完整支持 | 简化支持（单视图为主） |
| 排序 | GPU 计算着色器 | CPU 排序 |
| 后端架构 | 单一 GPU 后端 | 可切换 Software/GPU/Hybrid |

---

## 6. 模块详解

### 6.1 项目结构

```
FeatherOS/
├── apps/
│   ├── fhre/                          # FHRE 核心库 v2.0
│   │   └── rust/
│   │       ├── Cargo.toml
│   │       ├── Makefile
│   │       └── src/
│   │           ├── lib.rs             # 库入口，模块声明
│   │           ├── app/               # App 模块
│   │           │   ├── mod.rs         # 衔接：导出 app, config
│   │           │   ├── app.rs         # App 实现
│   │           │   └── config.rs      # App 配置
│   │           ├── main_world/        # Main World 模块
│   │           │   ├── mod.rs         # 衔接：导出 world, entity, component, system
│   │           │   ├── world.rs       # MainWorld 实现
│   │           │   ├── entity.rs      # Entity 实现
│   │           │   ├── component.rs   # Component 实现
│   │           │   └── system.rs      # System 实现
│   │           ├── render_world/      # Render World 模块
│   │           │   ├── mod.rs         # 衔接：导出 world, command, object
│   │           │   ├── world.rs       # RenderWorld 实现
│   │           │   ├── command.rs     # RenderCommand 实现
│   │           │   ├── object.rs      # RenderObject 实现
│   │           │   ├── phase.rs       # RenderPhase 实现
│   │           │   └── view.rs        # View 实现
│   │           ├── extract/           # Extract 模块
│   │           │   ├── mod.rs         # 衔接：导出 extract
│   │           │   └── extract.rs     # Extract 实现
│   │           ├── schedule/          # Schedule 模块
│   │           │   ├── mod.rs         # 衔接：导出 schedule, label, set
│   │           │   ├── schedule.rs    # Schedule 实现
│   │           │   ├── label.rs       # Label 实现
│   │           │   └── set.rs         # SystemSet 实现
│   │           ├── resources/         # Resources 模块
│   │           │   ├── mod.rs         # 衔接：导出 resources, time, config, screen
│   │           │   ├── resources.rs   # Resources 实现
│   │           │   ├── time.rs        # Time 实现
│   │           │   ├── config.rs      # Config 实现
│   │           │   └── screen.rs      # PrimaryScreen 实现
│   │           ├── pipeline/          # Pipeline 模块（渲染管线）
│   │           │   ├── mod.rs         # 衔接：导出 pipeline 子模块
│   │           │   └── batch.rs       # 批处理实现
│   │           ├── math/              # Math 模块
│   │           │   ├── mod.rs
│   │           │   ├── vec2.rs
│   │           │   ├── vec3.rs
│   │           │   ├── color.rs
│   │           │   └── rect.rs
│   │           ├── animation/         # Animation 模块（动画系统）
│   │           │   ├── mod.rs         # 衔接：导出 animation 子模块
│   │           │   ├── clip.rs        # AnimationClip 动画剪辑
│   │           │   ├── curve.rs       # 动画曲线和插值
│   │           │   ├── easing.rs      # 缓动函数（30+种）
│   │           │   ├── player.rs      # AnimationPlayer 组件
│   │           │   ├── graph.rs       # 动画图（混合）
│   │           │   ├── transition.rs  # 动画过渡
│   │           │   └── property.rs    # 动画属性
│   │           ├── platform/          # Platform 模块
│   │           │   ├── mod.rs         # 衔接：导出 sim, nuttx, default, framebuffer
│   │           │   ├── sim.rs         # NuttX SIM 平台支持
│   │           │   ├── nuttx.rs       # NuttX 平台支持
│   │           │   ├── default.rs     # 默认平台支持
│   │           │   └── framebuffer.rs # Framebuffer 抽象实现
│   │           └── lib.rs             # 库入口
```

### 6.2 模块化设计原则

**文件夹/mod.rs 结构：**

```rust
// mod.rs 仅作为衔接，导出同级目录下的子模块
// 不包含实际功能实现

// 示例：main_world/mod.rs
mod world;
mod entity;
mod component;
mod system;

pub use world::MainWorld;
pub use entity::Entity;
pub use component::{Component, Transform, Sprite, Velocity};
pub use system::{System, IntoSystem};
```

**设计原则：**

1. **mod.rs 只负责衔接**：声明子模块并导出公共接口
2. **功能实现在同级 .rs 文件**：如 `world.rs`, `entity.rs` 等
3. **清晰的模块边界**：每个模块有明确的职责
4. **易于扩展**：添加新功能只需创建新文件并在 mod.rs 中导出

---

## 7. 控件系统

### 7.1 架构对比

| 特性 | LVGL | Bevy | FHRE (设计目标) |
|------|------|------|-----------------|
| **架构** | 对象树 (Object Tree) | ECS 双世界 | **ECS 双世界** |
| **最小单位** | `lv_obj_t` (基础对象) | Entity + Components | **Node + Components** |
| **关系管理** | 父子指针 | 组件关联 | **组件关联 (无父子)** |
| **数据布局** | AOS (Array of Structs) | SOA (Structure of Arrays) | **SOA (缓存友好)** |
| **属性存储** | 对象内部字段 | 组件分离 | **组件分离** |
| **渲染方式** | CPU/GPU 混合 | GPU 批处理 | **GPU 批处理 (2D/3D 统一)** |
| **布局系统** | 内置布局 (Flex/Grid) | 无内置 (需自定义) | **内置 2D/3D 混合布局** |
| **事件系统** | 事件冒泡 | ECS 事件 | **ECS 事件** |
| **动画系统** | 属性动画 | 纹理图集/程序化 | **属性动画 + 程序化** |
| **3D 支持** | 无 | 完整 3D | **2.5D (简化 3D)** |

### 7.2 最小单位：Node

FHRE 的最小单位是 **Node**，采用 ECS 架构，扁平化设计：

```rust
/// Node 组件 - FHRE 的最小单位
/// 
/// 设计原则：
/// 1. **ECS 架构**: 无父子关系，组件扁平存储
/// 2. **SOA 布局**: 数据连续存储，提升缓存命中率
/// 3. **批量处理**: 支持 SIMD 和 GPU 批量处理
/// 4. **统一抽象**: 游戏实体和 UI 控件使用相同的基础组件
#[derive(Component, Debug, Clone)]
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
```

### 7.3 控件类型层次

```
Control (基础控件)
├── Container (容器)
│   ├── Panel (面板)
│   ├── Window (窗口)
│   └── ScrollView (滚动视图)
├── Input (输入控件)
│   ├── Button (按钮)
│   ├── TextInput (文本输入)
│   ├── Slider (滑块)
│   ├── Switch (开关)
│   └── Dropdown (下拉框)
├── Display (显示控件)
│   ├── Label (标签)
│   ├── Image (图像)
│   ├── ProgressBar (进度条)
│   └── Chart (图表)
├── 2.5D Controls (2.5D 控件)
│   ├── Card (卡片 - 支持翻转)
│   ├── IsoBlock (等角块)
│   └── FlipView (翻转视图)
└── 3D Controls (3D 控件)
    ├── Model3d (3D 模型)
    ├── ParticleWidget (粒子控件)
    └── View3d (3D 视图容器)
```

### 7.4 控件创建示例

```rust
// 纯 2D 控件（默认）
commands.spawn((
    Node::ui_control(NodeType::Button),
    Transform3D::from_position(100.0, 200.0, 0.0),  // z=0 在 UI 平面
    Style {
        width: Dimension::Pixel(120.0),
        height: Dimension::Pixel(40.0),
        background_color: Color::BLUE,
        ..default()
    },
));

// 2.5D 控件（卡片翻转效果）
commands.spawn((
    Node::ui_control(NodeType::Card),
    Transform3D::from_position(200.0, 200.0, 10.0)  // z=10 浮起
        .with_rotation(0.0, 0.2, 0.0),  // Y轴旋转
    Style {
        width: Dimension::Pixel(150.0),
        height: Dimension::Pixel(200.0),
        background_color: Color::WHITE,
        shadow_color: Color::BLACK.with_alpha(0.3),
        shadow_offset: Vec2::new(0.0, 4.0),
        shadow_blur: 8.0,
        ..default()
    },
    AnimationPlayer::default(),
));
```

---

## 8. NuttX SIM 平台支持

### 8.1 架构对比 (FHRE vs LVGL)

```
FHRE v2.0                              LVGL (lv_nuttx_fbdev)
─────────                              ─────────────────────

┌─────────────────┐                    ┌─────────────────┐
│   App::update() │                    │ lv_timer_handler│
│                 │                    │                 │
│ 1. Update Main  │                    │ 1. Update UI    │
│    World        │                    │                 │
│ 2. Extract      │                    │ 2. flush_cb()   │
│ 3. Render to FB │                    │ 3. Render to FB │
│ 4. SimDisplay   │                    │ 4. FBIO_UPDATE  │
│    .present()   │                    │    ioctl        │
└────────┬────────┘                    └────────┬────────┘
         │                                       │
         ▼                                       ▼
┌─────────────────┐                    ┌─────────────────┐
│  SimDisplay     │                    │  lv_nuttx_fb_t  │
│                 │                    │                 │
│ - fd: /dev/fb0  │                    │ - fd: /dev/fb0  │
│ - ioctl(UPDATE) │                    │ - ioctl(UPDATE) │
│ - backbuffer    │                    │ - backbuffer    │
└────────┬────────┘                    └────────┬────────┘
         │                                       │
         ▼                                       ▼
┌─────────────────┐                    ┌─────────────────┐
│  NuttX FB Driver│                    │  NuttX FB Driver│
│  (/dev/fb0)     │                    │  (/dev/fb0)     │
└────────┬────────┘                    └────────┬────────┘
         │                                       │
         ▼                                       ▼
┌─────────────────┐                    ┌─────────────────┐
│  X11 Window     │                    │  X11 Window     │
│  (sim_x11fb)    │                    │  (sim_x11fb)    │
└─────────────────┘                    └─────────────────┘
```

### 8.2 SIM 平台显示驱动

```rust
/// Sim platform display driver
pub struct SimDisplay {
    fd: i32,                    // /dev/fb0 file descriptor
    width: u32,
    height: u32,
    bpp: u8,
    stride: u32,
    framebuffer: *mut u32,      // Hardware framebuffer (mmap)
    backbuffer: Vec<u32>,       // Software backbuffer
}

impl SimDisplay {
    /// Create a new sim display
    pub fn new() -> Option<Self>
    
    /// Present render world to display
    pub fn present(&mut self, render_world: &RenderWorld)
    
    /// Flush framebuffer to display
    fn flush(&mut self)
}
```

### 8.3 编译和运行

```bash
cd /home/uan/develop/FeatherOS-code/FeatherOS/nuttx
make distclean
./tools/configure.sh sim:fhre
make -j

./nuttx
nsh> fhre_rust_demo
```

---

## 9. 使用示例

### 9.1 纯 UI 应用

```rust
use fhre::{
    App,
    node::{Node, NodeType, Transform3D, Style},
    math::{Vec2, Color},
    resources::PrimaryScreen,
};

fn setup_ui(mut commands: Commands, screen: Res<PrimaryScreen>) {
    let center = screen.center();
    
    // 创建按钮 - 使用屏幕坐标直接定位
    commands.spawn((
        Node::ui_control(NodeType::Button),
        Transform3D::from_position(100.0, 200.0, 0.0),  // 屏幕 (100, 200)
        Style {
            width: Dimension::Pixel(120.0),
            height: Dimension::Pixel(40.0),
            background_color: Color::BLUE,
            ..default()
        },
    ));
    
    // 创建居中的面板
    commands.spawn((
        Node::ui_control(NodeType::Panel),
        Transform3D::from_position(center.x - 150.0, center.y - 100.0, 0.0),
        Style {
            width: Dimension::Pixel(300.0),
            height: Dimension::Pixel(200.0),
            background_color: Color::DARK_GRAY,
            ..default()
        },
    ));
}

fn main() {
    let mut app = App::new();
    
    // 默认 UI 摄像机已经创建好了
    // 直接添加 UI 元素即可
    app.add_startup_system(setup_ui);
    
    app.run();
}
```

### 9.2 游戏应用

```rust
use fhre::{
    App,
    node::{Node, NodeType, Transform3D},
    math::{Vec3, Color},
};

fn main() {
    let mut app = App::new();
    
    // 创建游戏摄像机
    let game_cam = app.create_game_camera(
        Vec3::new(0.0, 5.0, -10.0),  // 位置
        Vec3::new(0.0, 0.0, 0.0),    // 看向原点
        60.0                          // FOV
    );
    
    // 添加游戏实体
    app.add_startup_system(setup_game);
    
    app.run();
}

fn setup_game(mut commands: Commands) {
    // 创建玩家
    commands.spawn((
        Node::game_entity(NodeType::Sprite2D),
        Transform3D::from_position(0.0, 0.0, 0.0),
    ));
}
```

---

## 10. 总结

FHRE v2.0 是一个完整的嵌入式渲染引擎，具有以下特点：

1. **双世界架构** - Main World 处理逻辑，Render World 处理渲染
2. **默认资源系统** - PrimaryScreen + Default UI Camera，开箱即用
3. **ECS 设计** - Node 作为最小单位，SOA 布局，缓存友好
4. **摄像机系统** - 支持 FPS、TPS、轨道视角等多种控制方式
5. **渲染管线** - 阶段化渲染，支持批处理
6. **NuttX SIM 支持** - 类似 LVGL 的 framebuffer 对接机制

**设计原则：**
- 开箱即用 - `App::new()` 后立即可渲染
- 可扩展 - 可以添加更多摄像机和资源
- 不强制 - 可以禁用默认资源，完全自定义
- 一致性 - 所有 UI 使用相同的坐标系统

---

**文档版本**: v2.0  
**日期**: 2026-04-15  
**作者**: FeatherOS Team
