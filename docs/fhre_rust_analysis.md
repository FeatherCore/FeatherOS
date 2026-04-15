# Feather Hybrid Render Engine (FHRE) Rust 版本 v2.0 说明文档

## 1. 概述

Feather Hybrid Render Engine (FHRE) v2.0 是一个基于声明式双世界架构的轻量级渲染引擎。该架构借鉴了 Bevy 的 ECS 设计，将游戏逻辑和渲染分离到两个独立的世界中，通过 Extract 阶段进行数据同步。

**核心特性：**

- 声明式双世界架构 (Main World + Render World)
- ECS (Entity-Component-System) 设计
- 模块化的系统调度
- 嵌入式友好的无标准库实现
- **NuttX SIM 平台支持** (类似 LVGL 的 framebuffer 对接)
- **文件夹/mod.rs 模块化结构** (mod.rs 仅作为衔接)

***

## 2. 项目结构

### 2.1 目录结构

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
│   │           │   └── object.rs      # RenderObject 实现
│   │           ├── extract/           # Extract 模块
│   │           │   ├── mod.rs         # 衔接：导出 extract
│   │           │   └── extract.rs     # Extract 实现
│   │           ├── schedule/          # Schedule 模块
│   │           │   ├── mod.rs         # 衔接：导出 schedule, label, set
│   │           │   ├── schedule.rs    # Schedule 实现
│   │           │   ├── label.rs       # Label 实现
│   │           │   └── set.rs         # SystemSet 实现
│   │           ├── resources/         # Resources 模块
│   │           │   ├── mod.rs         # 衔接：导出 resources, time, config
│   │           │   ├── resources.rs   # Resources 实现
│   │           │   ├── time.rs        # Time 实现
│   │           │   └── config.rs      # Config 实现
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
│   │
│   └── examples/
│       └── fhre/
│           └── rust/
│               ├── Cargo.toml
│               ├── Makefile
│               └── src/
│                   └── lib.rs         # 双世界架构示例
│
└── nuttx/boards/sim/sim/sim/configs/
    └── fhre/
        ├── defconfig                  # NuttX 配置
        └── README.txt
```

### 2.2 模块化设计原则

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

***

## 3. 双世界架构

### 3.1 架构概览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FHRE 双世界架构 v2.0                               │
│                                                                             │
│  ┌─────────────────────────────┐        ┌─────────────────────────────┐    │
│  │       Main World            │        │       Render World          │    │
│  │      (游戏逻辑世界)          │        │       (渲染世界)             │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Entities            │    │        │  │ RenderObjects       │    │    │
│  │  │ - Transform         │    │        │  │ - ExtractedTransform│    │    │
│  │  │ - Sprite            │    │        │  │ - ExtractedSprite   │    │    │
│  │  │ - Velocity          │    │        │  │ - z_order           │    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Systems             │    │        │  │ Draw Commands       │    │    │
│  │  │ - PreUpdate         │    │        │  │ - Clear             │    │    │
│  │  │ - Update            │    │        │  │ - DrawRect          │    │    │
│  │  │ - PostUpdate        │    │        │  │ - DrawCircle        │    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Resources           │    │        │  │ Framebuffer         │    │    │
│  │  │ - Time              │    │        │  │ - RGBA32 pixels     │    │    │
│  │  │ - WindowConfig      │    │        │  │ - width x height    │    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  └─────────────┬───────────────┘        └─────────────┬───────────────┘    │
│                │                                      │                     │
│                │          ExtractSchedule             │                     │
│                │         (数据提取阶段)                │                     │
│                └──────────────────────────────────────▶                     │
│                                                                             │
│  执行流程:                                                                  │
│  1. Main World 执行游戏逻辑 (Update Systems)                                │
│  2. Extract 阶段: 从 Main World 提取数据到 Render World                      │
│  3. Render World 生成绘制命令并渲染到 Framebuffer                            │
│  4. Present 阶段: 输出到显示设备 (/dev/fb0 via ioctl)                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

***

## 4. NuttX SIM 平台支持

FHRE 实现了与 LVGL 类似的 NuttX SIM 平台 framebuffer 对接机制。

### 4.1 架构对比 (FHRE vs LVGL)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      FHRE vs LVGL 架构对比                                   │
│                                                                             │
│  FHRE v2.0                              LVGL (lv_nuttx_fbdev)               │
│  ─────────                              ─────────────────────               │
│                                                                             │
│  ┌─────────────────┐                    ┌─────────────────┐                 │
│  │   App::update() │                    │ lv_timer_handler│                 │
│  │                 │                    │                 │                 │
│  │ 1. Update Main  │                    │ 1. Update UI    │                 │
│  │    World        │                    │                 │                 │
│  │ 2. Extract      │                    │ 2. flush_cb()   │                 │
│  │ 3. Render to FB │                    │ 3. Render to FB │                 │
│  │ 4. SimDisplay   │                    │ 4. FBIO_UPDATE  │                 │
│  │    .present()   │                    │    ioctl        │                 │
│  └────────┬────────┘                    └────────┬────────┘                 │
│           │                                       │                         │
│           ▼                                       ▼                         │
│  ┌─────────────────┐                    ┌─────────────────┐                 │
│  │  SimDisplay     │                    │  lv_nuttx_fb_t  │                 │
│  │                 │                    │                 │                 │
│  │ - fd: /dev/fb0  │                    │ - fd: /dev/fb0  │                 │
│  │ - ioctl(UPDATE) │                    │ - ioctl(UPDATE) │                 │
│  │ - backbuffer    │                    │ - backbuffer    │                 │
│  └────────┬────────┘                    └────────┬────────┘                 │
│           │                                       │                         │
│           ▼                                       ▼                         │
│  ┌─────────────────┐                    ┌─────────────────┐                 │
│  │  NuttX FB Driver│                    │  NuttX FB Driver│                 │
│  │  (/dev/fb0)     │                    │  (/dev/fb0)     │                 │
│  └────────┬────────┘                    └────────┬────────┘                 │
│           │                                       │                         │
│           ▼                                       ▼                         │
│  ┌─────────────────┐                    ┌─────────────────┐                 │
│  │  X11 Window     │                    │  X11 Window     │                 │
│  │  (sim_x11fb)    │                    │  (sim_x11fb)    │                 │
│  └─────────────────┘                    └─────────────────┘                 │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 SIM 平台显示驱动

**文件**: `platform/sim.rs`

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
    /// Opens /dev/fb0 and gets framebuffer info via ioctl
    pub fn new() -> Option<Self> {
        unsafe {
            // Open framebuffer device
            let fd = libc::open("/dev/fb0", libc::O_RDWR);
            
            // Get video info via ioctl
            let mut vinfo: FbVideoInfo = core::mem::zeroed();
            libc::ioctl(fd, FBIOGET_VIDEOINFO, &mut vinfo);
            
            // Get plane info via ioctl
            let mut pinfo: FbPlaneInfo = core::mem::zeroed();
            libc::ioctl(fd, FBIOGET_PLANEINFO, &mut pinfo);
            
            // ...
        }
    }
    
    /// Present render world to display
    /// Similar to LVGL's flush_cb
    pub fn present(&mut self, render_world: &RenderWorld) {
        // Copy render world framebuffer to backbuffer
        let fb = render_world.get_framebuffer();
        self.backbuffer.copy_from_slice(fb);
        
        // Copy backbuffer to hardware framebuffer
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.backbuffer.as_ptr(),
                self.framebuffer,
                self.backbuffer.len()
            );
        }
        
        // Trigger display update via ioctl
        // Similar to LVGL's FBIO_UPDATE
        self.flush();
    }
    
    /// Flush framebuffer to display
    fn flush(&mut self) {
        unsafe {
            let area = FbArea {
                x: 0, y: 0,
                w: self.width,
                h: self.height,
            };
            // Trigger X11 update via ioctl
            libc::ioctl(self.fd, FBIO_UPDATE, &area);
        }
    }
}
```

### 4.3 IOCTL 命令

| 命令 | 值 | 功能 |
|------|-----|------|
| `FBIOGET_VIDEOINFO` | 0x4600 | 获取视频信息（分辨率、格式等） |
| `FBIOGET_PLANEINFO` | 0x4601 | 获取平面信息（内存地址、stride等） |
| `FBIO_UPDATE` | 0x4602 | 更新显示区域（触发 X11 刷新） |
| `FBIOPAN_DISPLAY` | 0x4603 | 双缓冲切换 |

### 4.4 刷新循环

```rust
/// Sim platform refresh loop
/// Similar to LVGL's display_refr_timer_cb
pub fn refresh_loop<F>(mut render_fn: F)
where
    F: FnMut(),
{
    let frame_duration = Duration::from_millis(16); // ~60 FPS
    
    loop {
        let start = libc::clock();
        
        // Execute render function
        render_fn();
        
        // Calculate sleep time to maintain frame rate
        let elapsed = libc::clock() - start;
        let elapsed_ms = elapsed * 1000 / CLOCKS_PER_SEC;
        
        if elapsed_ms < 16 {
            libc::usleep((16 - elapsed_ms) * 1000);
        }
    }
}
```

***

## 5. 模块详解

### 5.1 app 模块

**文件**: `app/mod.rs` (衔接) + `app/app.rs` (实现) + `app/config.rs` (配置)

```rust
// app/mod.rs - 仅作为衔接
mod app;
mod config;

pub use app::App;
pub use config::AppConfig;

// app/app.rs - 实际实现
pub struct App {
    pub main_world: MainWorld,
    pub render_world: RenderWorld,

    pub schedules: Schedules,
    pub config: AppConfig,
    pub sim_display: Option<SimDisplay>,  // SIM platform display
    running: bool,
}

impl App {
    pub fn new() -> Self
    pub fn update(&mut self)  // Includes SIM display present
    pub fn run(&mut self)
}
```

### 5.2 main_world 模块

**文件**: `main_world/mod.rs` (衔接) + `world.rs` + `entity.rs` + `component.rs` + `system.rs`

```rust
// main_world/mod.rs - 仅作为衔接
mod world;
mod entity;
mod component;
mod system;

pub use world::MainWorld;
pub use entity::Entity;
pub use component::{Component, Transform, Sprite, Velocity};
pub use system::{System, IntoSystem};

// main_world/world.rs - MainWorld 实现
pub struct MainWorld {
    next_entity_id: u64,
    entities: Vec<Entity>,
    components: BTreeMap<TypeId, BTreeMap<u64, Box<dyn Any>>>,
    systems: Vec<Box<dyn System>>,
    resources: Resources,
}

// main_world/entity.rs - Entity 实现
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entity {
    id: u64,
}

// main_world/component.rs - Component 实现
pub trait Component: 'static + Send + Sync {}

pub struct Transform {
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
}

pub struct Sprite {
    pub color: Color,
    pub width: f32,
    pub height: f32,
    pub visible: bool,
}

// main_world/system.rs - System 实现
pub trait System {
    fn run(&mut self, world: &mut MainWorld);
}
```

### 5.3 render_world 模块

**文件**: `render_world/mod.rs` (衔接) + `world.rs` + `command.rs` + `object.rs` + `phase.rs` + `view.rs`

```rust
// render_world/mod.rs - 仅作为衔接
mod world;
mod command;
mod object;
mod phase;
mod view;

pub use world::RenderWorld;
pub use command::{RenderCommand, DrawCall, Vertex, PrimitiveType};
pub use object::{RenderObject, ExtractedTransform, ExtractedSprite};
pub use phase::{
    RenderPhaseType, PhaseItem, RenderPhase, RenderPhases,
    PhaseBatch, BatchBuilder,
};
pub use view::{View, ViewTarget, ClearConfig, ViewBundle};

// render_world/world.rs - RenderWorld 实现
/// Render World - Container for rendering data
///
/// Architecture inspired by Bevy's render world:
/// - Multiple render phases (Background, Opaque2d, Opaque3d, AlphaMask, Transparent, UI)
/// - View management for multiple cameras/viewports
/// - Batch processing for efficient rendering
pub struct RenderWorld {
    width: u32,
    height: u32,
    framebuffer: Vec<u32>,
    objects: Vec<RenderObject>,
    commands: Vec<RenderCommand>,
    clear_color: Color,
    viewport: Rect,
    phases: RenderPhases,           // 渲染阶段系统
    views: Vec<ViewBundle>,         // 视图管理
    current_view: Option<usize>,
    use_phases: bool,
}

// render_world/phase.rs - 渲染阶段系统 (Bevy-inspired)
/// Render phase type - determines rendering order and behavior
pub enum RenderPhaseType {
    Background = 0,     // 背景/清除
    Opaque2d = 1,       // 不透明 2D 物体
    Opaque3d = 2,       // 不透明 3D 物体
    AlphaMask = 3,      // Alpha 测试物体
    Transparent = 4,    // 透明物体（需要排序）
    Ui = 5,             // UI 覆盖层
}

/// Phase item - a single drawable item in a render phase
pub struct PhaseItem {
    pub sort_key: i32,
    pub z_depth: f32,
    pub entity_id: Option<u32>,
    pub draw_command_index: usize,
    pub batchable: bool,
    pub batch_key: u64,
}

/// Render phase - contains items to render in a specific order
pub struct RenderPhase {
    pub phase_type: RenderPhaseType,
    pub items: Vec<PhaseItem>,
    pub sorted: bool,
}

/// Render phase collection - manages all render phases
pub struct RenderPhases {
    phases: [RenderPhase; 6],
}

/// Phase batch - groups items that can be rendered together
pub struct PhaseBatch {
    pub start_index: usize,
    pub count: usize,
    pub batch_key: u64,
    pub phase_type: RenderPhaseType,
}

// render_world/view.rs - 视图管理 (Bevy-inspired)
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

/// View target - describes where to render
pub enum ViewTarget {
    Screen,         // 渲染到屏幕/帧缓冲区
    Texture(u32),   // 渲染到纹理（用于后处理）
}

/// Clear configuration - defines how to clear the view
pub struct ClearConfig {
    pub clear_color: bool,
    pub color: Color,
    pub clear_depth: bool,
    pub depth: f32,
}

/// View bundle - combines view, target, and clear config
pub struct ViewBundle {
    pub view: View,
    pub target: ViewTarget,
    pub clear: ClearConfig,
}

// render_world/command.rs - 渲染命令
pub enum RenderCommand {
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawLine { start: Vec2, end: Vec2, color: Color, thickness: f32 },
    DrawTriangle { p0: Vec2, p1: Vec2, p2: Vec2, color: Color },
    DrawText { position: Vec2, text: &'static str, color: Color, size: f32 },
    SetScissor { rect: Rect },
    DisableScissor,
}

// DrawCall - GPU batching unit
pub struct DrawCall {
    pub primitive: PrimitiveType,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub transform: [f32; 16],
    pub color: Color,
}

pub enum PrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
}

// render_world/object.rs - 渲染对象
pub struct RenderObject {
    pub position: Vec3,
    pub rotation: f32,
    pub scale: Vec2,
    pub color: Color,
    pub size: Vec2,
    pub visible: bool,
    pub z_order: i32,
}
```

#### 5.3.1 渲染阶段系统 (Render Phases)

FHRE 的渲染阶段系统借鉴 Bevy 的设计，将绘制命令组织成多个阶段：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FHRE 渲染阶段系统                                    │
│                                                                             │
│  渲染顺序:                                                                   │
│                                                                             │
│  1. Background    →  清除/背景绘制                                           │
│       ↓                                                                     │
│  2. Opaque2d      →  不透明 2D 物体（精灵、UI 背景）                          │
│       ↓                                                                     │
│  3. Opaque3d      →  不透明 3D 物体（模型）                                   │
│       ↓                                                                     │
│  4. AlphaMask     →  Alpha 测试物体（镂空纹理）                               │
│       ↓                                                                     │
│  5. Transparent   →  透明物体（需要排序，从后往前）                            │
│       ↓                                                                     │
│  6. Ui            →  UI 覆盖层（最上层）                                      │
│                                                                             │
│  特点:                                                                       │
│  - Opaque 阶段: 支持批处理，不排序                                            │
│  - Transparent 阶段: 需要按 Z 深度排序                                        │
│  - UI 阶段: 最后渲染，覆盖所有内容                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

**与 Bevy 的对比：**

| 特性 | Bevy | FHRE |
|------|------|------|
| 阶段类型 | Binned + Sorted | 简化版 Binned + Sorted |
| 批处理 | GPU 预处理 | CPU 批处理（嵌入式限制） |
| 多视图 | 完整支持 | 简化支持（单视图为主） |
| 排序 | GPU 计算着色器 | CPU 排序 |

#### 5.3.2 视图管理系统 (Views)

FHRE 的视图系统简化自 Bevy，支持多视图渲染：

```rust
// 创建主视图（2D 正交投影）
let main_view = ViewBundle::new_screen(800.0, 600.0)
    .with_clear_color(Color::BLACK);

// 创建 3D 透视视图
let view_3d = ViewBundle {
    view: View::new_3d(800.0, 600.0, 60.0),  // 60° FOV
    target: ViewTarget::Screen,
    clear: ClearConfig::color(Color::DARK_GRAY),
};

// 添加到 RenderWorld
render_world.add_view(main_view);
render_world.add_view(view_3d);

// 渲染所有视图的所有阶段
render_world.render_phases();
```

**视图管理 API：**

```rust
impl RenderWorld {
    /// Add a view
    pub fn add_view(&mut self, view: ViewBundle) -> usize
    
    /// Get/set current view
    pub fn set_current_view(&mut self, index: Option<usize>)
    pub fn current_view(&self) -> Option<&ViewBundle>
    
    /// Create default screen view
    pub fn create_default_view(&mut self) -> usize
    
    /// Render all phases for all views
    pub fn render_phases(&mut self)
}
```


### 5.6 schedule 模块

**文件**: `schedule/mod.rs` (衔接) + `schedule.rs` + `label.rs` + `set.rs`

```rust
// schedule/mod.rs - 仅作为衔接
mod schedule;
mod label;
mod set;

pub use schedule::{Schedule, Schedules};
pub use label::{ScheduleLabel, Label, labels};
pub use set::SystemSet;

// schedule/schedule.rs - Schedule 实现
pub struct Schedule {
    name: &'static str,
    systems: Vec<Box<dyn FnMut()>>,
    sets: Vec<SystemSet>,
}

pub struct Schedules {
    schedules: BTreeMap<ScheduleLabel, Schedule>,
    order: Vec<ScheduleLabel>,
}

// schedule/label.rs - Label 实现
pub enum ScheduleLabel {
    PreUpdate,
    Update,
    PostUpdate,
    Extract,
    Render,
    Custom(&'static str),
}

// schedule/set.rs - SystemSet 实现
pub struct SystemSet {
    name: &'static str,
    systems: Vec<SystemLabel>,
    run_after: Vec<&'static str>,
    run_before: Vec<&'static str>,
}
```

### 5.7 resources 模块

**文件**: `resources/mod.rs` (衔接) + `resources.rs` + `time.rs` + `config.rs`

```rust
// resources/mod.rs - 仅作为衔接
mod resources;
mod time;
mod config;

pub use resources::{Resources, Resource, Res, ResMut};
pub use time::{Time, Timer};
pub use config::{RenderConfig, WindowConfig};

// resources/resources.rs - Resources 实现
pub trait Resource: 'static + Send + Sync {}

pub struct Resources {
    storage: BTreeMap<TypeId, Box<dyn Any>>,
}

// resources/time.rs - Time 实现
pub struct Time {
    elapsed: f32,
    delta: f32,
    frame: u64,
    scale: f32,
    paused: bool,
    fps: f32,
}

// resources/config.rs - Config 实现
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub target_fps: u32,
    pub vsync: bool,
    pub clear_color: Color,
}
pub trait RenderTarget {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn clear(&mut self, color: Color);
    fn draw_pixel(&mut self, x: u32, y: u32, color: Color);
    fn data(&self) -> &[u32];
}
```

### 5.8 extract 模块

**文件**: `extract/mod.rs` (衔接) + `extract.rs`

```rust
// extract/mod.rs - 仅作为衔接
mod extract;

pub use extract::{Extract, ExtractSchedule, ExtractResource, ExtractFn};

// extract/extract.rs - Extract 实现
pub trait Extract {
    fn extract(&self, main_world: &MainWorld, render_world: &mut RenderWorld);
}

pub struct ExtractSchedule {
    extractors: Vec<Box<dyn Fn(&MainWorld, &mut RenderWorld)>>,
}

pub fn extract_sprites(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // Extract Transform + Sprite -> RenderObject
}
```

### 5.9 platform/sim 模块

**文件**: `platform/mod.rs` (衔接) + `sim.rs` + `nuttx.rs` + `default.rs`

```rust
// platform/mod.rs - 仅作为衔接
#[cfg(feature = "sim")]
pub mod sim;
#[cfg(feature = "nuttx")]
pub mod nuttx;
pub mod default;

// platform/sim.rs - SIM 平台实现
pub struct SimDisplay {
    fd: i32,
    width: u32,
    height: u32,
    framebuffer: *mut u32,
    backbuffer: Vec<u32>,
}

impl SimDisplay {
    pub fn new() -> Option<Self>
    pub fn present(&mut self, render_world: &RenderWorld)
    pub fn update_area(&mut self, x: u32, y: u32, width: u32, height: u32)
}

pub fn create_display() -> Option<SimDisplay>
pub fn refresh_loop<F>(render_fn: F)  // Legacy function, use App::run() instead
```

### 5.10 Integrated Refresh Loop (New in v2.0)

**Similar to LVGL's `lv_nuttx_run()`**, FHRE v2.0 provides integrated refresh loop management:

```rust
impl App {
    /// Run with integrated refresh loop (60 FPS with CPU yield)
    pub fn run(&mut self)
    
    /// Run with custom callback for game logic
    pub fn run_with_callback<F>(&mut self, callback: F)
    where F: FnMut()
}
```

**Usage:**

```rust
// Simple usage - integrated loop
let mut app = App::new();
app.run();  // Blocks until exit, manages 60 FPS internally

// With custom game logic callback
let mut app = App::new();
app.run_with_callback(|| {
    // Custom update logic here
    update_entities(&mut app);
});
```

**Benefits:**
- Single entry point like LVGL's `lv_nuttx_run()`
- Automatic frame rate control (60 FPS)
- CPU yield via `usleep()` for cooperative multitasking
- No need for manual loop management in applications

***

## 6. 示例代码

### 6.1 使用 SIM 平台显示 (New API v2.0)

```rust
#![no_std]
#![no_main]

use fhre::{
    App, FHRE_VERSION,
    main_world::{Transform, Sprite, Velocity},
    math::{Color, Vec2},
};

/// Custom application with game logic
struct DemoApp {
    app: App,
    frame_count: u32,
}

impl DemoApp {
    fn new() -> Self {
        let mut app = App::new();
        
        // Spawn entities
        let entity = app.main_world.spawn();
        app.main_world.insert_component(entity, Transform::from_2d(100.0, 100.0));
        app.main_world.insert_component(entity, Sprite::rect(50.0, 50.0, Color::RED));
        app.main_world.insert_component(entity, Velocity::new(2.0, 1.5));
        
        Self { app, frame_count: 0 }
    }
    
    /// Custom game logic - called before each frame
    fn update(&mut self) {
        self.frame_count += 1;
        
        // Update entity positions, handle input, etc.
        for (entity, velocity) in self.app.main_world.query::<Velocity>() {
            if let Some(transform) = self.app.main_world.get_component_mut::<Transform>(entity) {
                transform.position.x += velocity.linear.x;
                transform.position.y += velocity.linear.y;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn fhre_rust_main(_argc: i32, _argv: *const *const u8) -> i32 {
    printf("FHRE Rust Demo - Version: %s\n", FHRE_VERSION);
    
    // Create demo application
    let mut demo = DemoApp::new();
    
    // Use integrated refresh loop (similar to LVGL's lv_nuttx_run())
    // The loop is managed internally by FHRE with 60 FPS frame rate control
    demo.app.run_with_callback(|| {
        // Custom game logic - called before each frame
        demo.update();
    });
    
    0
}
```

**Key Changes in v2.0:**
- `app.run()` - Simple integrated loop, no manual frame management
- `app.run_with_callback()` - Loop with custom game logic callback
- Automatic 60 FPS frame rate control with CPU yield
- No need for manual `usleep()` calls in application code

***

## 7. 编译和运行

### 7.1 NuttX 配置

**defconfig**:
```conf
# FHRE 配置
CONFIG_FHRE_RUST=y
CONFIG_EXAMPLES_FHRE_RUST=y

# Framebuffer 配置 (类似 LVGL)
CONFIG_VIDEO_FB=y
CONFIG_SIM_X11FB=y
CONFIG_SIM_FBWIDTH=640
CONFIG_SIM_FBHEIGHT=480

# 入口点
CONFIG_INIT_ENTRYPOINT="nsh_main"
```

### 7.2 编译步骤

```bash
cd /home/uan/develop/FeatherOS-code/FeatherOS/nuttx
make distclean
./tools/configure.sh sim:fhre
make -j
```

### 7.3 运行

```bash
./nuttx
nsh> fhre_rust_demo
```

***

## 8. 架构对比

### 8.1 FHRE vs LVGL (NuttX SIM 平台)

| 特性 | FHRE v2.0 | LVGL |
|------|-----------|------|
| 架构 | ECS 双世界 | 传统回调 |
| Framebuffer 访问 | `/dev/fb0` + ioctl | `/dev/fb0` + ioctl |
| 刷新机制 | `FBIO_UPDATE` ioctl | `FBIO_UPDATE` ioctl |
| 双缓冲 | 支持 | 支持 |
| 刷新率控制 | 60 FPS 循环 | 定时器回调 |
| 目标平台 | 嵌入式 | 嵌入式/桌面 |
| 模块结构 | 文件夹/mod.rs | 单文件 |

### 8.2 关键相似点

1. **都使用 `/dev/fb0` 设备文件**
2. **都使用 `ioctl(fd, FBIO_UPDATE, ...)` 触发刷新**
3. **都使用双缓冲机制**
4. **都支持 NuttX SIM 平台的 X11 framebuffer**

***

## 9. 控件管理系统设计

### 9.1 架构对比分析

#### LVGL vs Bevy vs FHRE

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

### 9.2 FHRE 控件系统核心设计

#### 9.2.1 最小单位：Node

FHRE 的最小单位是 **Node**，采用 ECS 架构，扁平化设计，SOA (Structure of Arrays) 布局：

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

**重要设计决策：**

1. **无父子关系**: 不同于 LVGL 的对象树，FHRE 采用 Bevy 的 ECS 扁平结构
2. **SOA 布局**: 数据按类型连续存储，提升缓存命中率，支持批量处理
3. **组件组合**: 通过组件组合实现功能，而非继承
4. **统一抽象**: Node 既可以表示 2D 游戏精灵，也可以表示 UI 按钮

#### 9.2.2 Main World 与 Node 系统的配合

FHRE 的 Main World 是核心 ECS 世界，Node 系统作为组件层与其配合：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Main World + Node System                            │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Main World (ECS Core)                        │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │   │
│  │  │  Entity     │  │  Component  │  │  System     │  │ Resources │  │   │
│  │  │  (ID)       │  │  (Data)     │  │  (Logic)    │  │ (Global)  │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  Component Storage (SOA Layout)                             │   │   │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │   │   │
│  │  │  │ Node[]  │ │Node2D[] │ │Node3D[] │ │Style[]  │ ...       │   │   │
│  │  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘           │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Node System (Components)                     │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │   │
│  │  │ Node        │  │ Node2D      │  │ Node3D      │  │ Style     │  │   │
│  │  │ (Core)      │  │ (Transform) │  │ (Transform) │  │ (Visual)  │  │   │
│  │  │             │  │             │  │             │  │           │  │   │
│  │  │ - node_type │  │ - position  │  │ - position  │  │ - width   │  │   │
│  │  │ - state     │  │ - rotation  │  │ - rotation  │  │ - height  │  │   │
│  │  │ - flags     │  │ - scale     │  │ - scale     │  │ - color   │  │   │
│  │  │ - z_order   │  │ - anchor    │  │ - anchor    │  │ - border  │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │   │
│  │  │ Layout      │  │ Sprite      │  │ Text        │                 │   │
│  │  │ (Arrange)   │  │ (Render)    │  │ (Render)    │                 │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        System Execution                             │   │
│  │                                                                     │   │
│  │  1. Input System  →  2. Layout System  →  3. Transform System      │   │
│  │       ↓                    ↓                    ↓                  │   │
│  │  4. Animation System → 5. Extract System → 6. Render World         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

**配合方式：**

1. **实体创建**: Main World 创建 Entity，附加 Node 组件
2. **组件存储**: Main World 使用 SOA 布局存储组件数据
3. **系统查询**: Systems 通过 Query 批量处理 Node 组件
4. **数据流**: Node 组件 → Extract → Render World

**与 Bevy 的对比：**

| 方面 | Bevy | FHRE |
|------|------|------|
| **World 结构** | `App` 包含多个 `World` | `App` 包含 `MainWorld` + `RenderWorld` |
| **组件存储** | `World` 使用 Archetype | `MainWorld` 使用 SOA (TypeId → Entity → Component) |
| **System 调度** | `Schedule` + `Stage` | `Schedule` + `Label` |
| **Entity 创建** | `commands.spawn()` | `main_world.spawn()` |
| **组件添加** | `.insert(Component)` | `.insert_component(entity, component)` |
| **Query** | `Query<&Component>` | `main_world.query::<Component>()` |
| **批量处理** | 自动批处理 | `NodeStateBatch` + `NodeTransformBatch` |

**代码示例：**

```rust
// 创建 Main World
let mut main_world = MainWorld::new();

// 创建游戏实体
let entity1 = main_world.spawn();
main_world.insert_component(entity1, Node::game_entity(NodeType::Sprite2D));
main_world.insert_component(entity1, Node2D::from_position(100.0, 200.0));
main_world.insert_component(entity1, Sprite::from_image(image_handle));

// 创建 UI 控件
let entity2 = main_world.spawn();
main_world.insert_component(entity2, Node::ui_control(NodeType::Button));
main_world.insert_component(entity2, Node2D::from_position(50.0, 50.0));
main_world.insert_component(entity2, Style::ui_default());

// 批量查询和处理
for (entity, node) in main_world.query::<Node>() {
    if node.state.visible {
        // 处理可见节点
    }
}

// SOA 批量处理
let mut batch = NodeStateBatch::new();
for (_, node) in main_world.query::<Node>() {
    batch.add(&node.state);
}
// 批量操作
batch.mark_all_dirty();
```

/// 控件类型枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlType {
    // 基础控件 (2D 为主)
    Container,      // 容器
    Panel,          // 面板
    Button,         // 按钮
    Label,          // 标签
    Image,          // 图像
    TextInput,      // 文本输入
    
    // 2.5D 控件 (支持简单 3D 效果)
    Card,           // 卡片 (支持翻转)
    IsoBlock,       // 等角块
    Sprite3d,       // 3D 精灵
    
    // 3D 控件 (完整 3D 属性)
    Model3d,        // 3D 模型
    ParticleSystem, // 粒子系统
}

/// 控件状态
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ControlState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub disabled: bool,
    pub checked: bool,
}
```

#### 9.2.2 双世界架构下的控件管理

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FHRE 控件双世界架构                                │
│                                                                             │
│  ┌─────────────────────────────┐        ┌─────────────────────────────┐    │
│  │       Main World            │        │       Render World          │    │
│  │      (控件逻辑世界)          │        │       (渲染世界)             │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Control Entities    │    │        │  │ RenderObjects       │    │    │
│  │  │ - Control           │    │        │  │ - Mesh/Quad         │    │    │
│  │  │ - Transform2D       │    │ Extract│  │ - Material          │    │    │
│  │  │ - Transform3D (opt) │    │───────▶│  │ - Depth             │    │    │
│  │  │ - Style             │    │        │  │ - Visibility        │    │    │
│  │  │ - Layout            │    │        │  └─────────────────────┘    │    │
│  │  │ - Interaction       │    │        │                             │    │
│  │  └─────────────────────┘    │        │  ┌─────────────────────┐    │    │
│  │                             │        │  │ Draw Commands       │    │    │
│  │  ┌─────────────────────┐    │        │  │ - DrawRect          │    │    │
│  │  │ Layout System       │    │        │  │ - DrawText          │    │    │
│  │  │ - Flex/Grid         │    │        │  │ - DrawMesh          │    │    │
│  │  │ - Auto 2D/3D        │    │        │  │ - DrawImage         │    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Interaction System  │    │        │  │ Framebuffer         │    │    │
│  │  │ - Event handling    │    │        │  │ - 2D/3D unified     │    │    │
│  │  │ - Focus management  │    │        │  │ - Z-sorting         │    │    │
│  │  │ - Animation trigger │    │        │  │ - Batch rendering   │    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  └─────────────────────────────┘        └─────────────────────────────┘    │
│                                                                             │
│  关键设计：                                                                  │
│  1. 所有控件默认是 "2D 模式"（use_3d_transform = false）                      │
│  2. 2D 模式时，Transform3D 使用默认值（与窗口平面重合）                        │
│  3. 设置 use_3d_transform = true 后，可以自定义 3D 属性                        │
│  4. Extract 阶段根据 use_3d_transform 决定如何生成 RenderObject               │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 9.2.3 组件设计

**基础组件（所有控件都有）：**

```rust
/// 2D 变换组件 - 必须
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
    pub position: Vec2,     // 2D 位置
    pub rotation: f32,      // 2D 旋转（弧度）
    pub scale: Vec2,        // 2D 缩放
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

/// 3D 变换组件 - 可选，默认与 2D 窗口平面重合
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Transform3D {
    pub position: Vec3,     // 3D 位置（默认 z = 0）
    pub rotation: Quat,     // 3D 旋转（默认单位四元数）
    pub scale: Vec3,        // 3D 缩放（默认 1,1,1）
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,           // 原点
            rotation: Quat::IDENTITY,       // 无旋转
            scale: Vec3::ONE,               // 原始大小
        }
    }
}

/// 样式组件 - 类似 LVGL 的样式系统
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Style {
    // 尺寸
    pub width: Dimension,
    pub height: Dimension,
    
    // 边距和间距
    pub margin: Rect<f32>,
    pub padding: Rect<f32>,
    
    // 背景和边框
    pub background_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub border_radius: f32,
    
    // 文字样式
    pub text_color: Color,
    pub font_size: f32,
    pub font_family: Option<String>,
    
    // 阴影（2.5D 效果）
    pub shadow_color: Color,
    pub shadow_offset: Vec2,
    pub shadow_blur: f32,
    
    // 变换效果
    pub opacity: f32,
    pub visible: bool,
}

/// 尺寸类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Auto,           // 自动（根据内容）
    Pixel(f32),     // 固定像素
    Percent(f32),   // 百分比
    Fill,           // 填充剩余空间
}
```

**布局组件：**

```rust
/// 布局组件
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Layout {
    pub layout_type: LayoutType,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub gap: Vec2,
    pub wrap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutType {
    None,       // 无布局（绝对定位）
    Flex,       // Flexbox 布局
    Grid,       // Grid 布局
    Stack,      // 堆叠布局
}
```

**交互组件：**

```rust
/// 交互组件
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Interaction {
    pub enabled: bool,
    pub draggable: bool,
    pub clickable: bool,
    pub hoverable: bool,
    pub focusable: bool,
}

/// 事件处理器组件
#[derive(Component)]
pub struct EventHandlers {
    pub on_click: Option<Box<dyn Fn(&mut Control, &EventContext)>>,
    pub on_hover: Option<Box<dyn Fn(&mut Control, &EventContext)>>,
    pub on_drag: Option<Box<dyn Fn(&mut Control, &DragEvent)>>,
}
```

#### 9.2.4 控件创建示例

**纯 2D 控件（默认）：**

```rust
// 创建一个 2D 按钮 - 只需指定 2D 属性
commands.spawn((
    Control {
        control_type: ControlType::Button,
        state: ControlState::default(),
        use_3d_transform: false,  // 默认 false，可省略
    },
    Transform2D {
        position: Vec2::new(100.0, 100.0),
        rotation: 0.0,
        scale: Vec2::ONE,
    },
    // Transform3D 使用默认值（与窗口平面重合）
    Style {
        width: Dimension::Pixel(120.0),
        height: Dimension::Pixel(40.0),
        background_color: Color::BLUE,
        ..default()
    },
    Interaction::default(),
));
```

**2.5D 控件（卡片翻转效果）：**

```rust
// 创建一个可翻转的卡片
commands.spawn((
    Control {
        control_type: ControlType::Card,
        state: ControlState::default(),
        use_3d_transform: true,  // 启用 3D 变换
    },
    Transform2D {
        position: Vec2::new(200.0, 200.0),
        ..default()
    },
    Transform3D {
        position: Vec3::new(0.0, 0.0, 10.0),  // 稍微浮起
        rotation: Quat::from_rotation_x(0.1), // 轻微倾斜
        ..default()
    },
    Style {
        width: Dimension::Pixel(150.0),
        height: Dimension::Pixel(200.0),
        background_color: Color::WHITE,
        shadow_color: Color::BLACK.with_alpha(0.3),
        shadow_offset: Vec2::new(0.0, 4.0),
        shadow_blur: 8.0,
        ..default()
    },
    // 卡片翻转动画
    AnimationPlayer::default(),
));
```

**3D 控件：**

```rust
// 创建一个 3D 模型控件
commands.spawn((
    Control {
        control_type: ControlType::Model3d,
        state: ControlState::default(),
        use_3d_transform: true,
    },
    Transform2D {
        // 2D 位置决定投影到屏幕的位置
        position: Vec2::new(300.0, 300.0),
        ..default()
    },
    Transform3D {
        // 完整的 3D 变换
        position: Vec3::new(0.0, 0.0, 50.0),
        rotation: Quat::from_euler(EulerRot::YXZ, 0.5, 0.3, 0.0),
        scale: Vec3::splat(2.0),
    },
    Model3d {
        mesh: mesh_handle,
        material: material_handle,
    },
));
```

#### 9.2.5 布局系统

FHRE 的布局系统自动处理 2D 和 3D 控件：

```rust
/// 布局系统
pub fn layout_system(
    mut controls: Query<(&Control, &mut Transform2D, Option<&mut Transform3D>, &Layout, &Children)>,
    parents: Query<&GlobalTransform2D>,
) {
    for (control, mut transform_2d, transform_3d_opt, layout, children) in controls.iter_mut() {
        if control.use_3d_transform {
            // 3D 模式：使用 3D 布局计算
            if let Some(mut transform_3d) = transform_3d_opt {
                calculate_3d_layout(&mut transform_3d, layout, children);
            }
        } else {
            // 2D 模式：使用 2D 布局计算
            calculate_2d_layout(&mut transform_2d, layout, children);
        }
    }
}
```

#### 9.2.6 Extract 阶段处理

```rust
/// Extract 阶段：将 Main World 的控件转换为 Render World 的渲染对象
pub fn extract_controls(
    mut render_world: ResMut<RenderWorld>,
    controls: Query<(&Control, &Transform2D, Option<&Transform3D>, &Style, &GlobalTransform2D)>,
) {
    for (control, transform_2d, transform_3d_opt, style, global_transform) in controls.iter() {
        let render_object = if control.use_3d_transform {
            // 3D 控件：使用 Transform3D
            let transform_3d = transform_3d_opt.unwrap_or_default();
            RenderObject::Mesh3d {
                transform: calculate_final_transform(global_transform, transform_2d, &transform_3d),
                mesh: generate_mesh(style),
                material: generate_material(style),
            }
        } else {
            // 2D 控件：使用默认的 Transform3D（与窗口平面重合）
            RenderObject::Quad2D {
                position: global_transform.translation(),
                size: calculate_size(style),
                color: style.background_color,
                border_radius: style.border_radius,
            }
        };
        
        render_world.add_object(render_object);
    }
}
```

### 9.3 与 LVGL 和 Bevy 的对比总结

| 设计决策 | LVGL 方式 | Bevy 方式 | FHRE 方式 | 理由 |
|---------|-----------|-----------|-----------|------|
| **最小单位** | `lv_obj_t` 对象 | Entity + 多个 Components | `Control` 组件 + 相关组件 | 保持 ECS 灵活性，但提供更高层抽象 |
| **2D/3D 切换** | 不支持 | 完全分离 | `use_3d_transform` 标志 | 简化使用，默认 2D，需要时启用 3D |
| **属性描述** | 对象字段 | 多个组件 | 2D 必须 + 3D 可选 | 2D 控件简洁，3D 控件完整 |
| **布局系统** | 内置 Flex/Grid | 无内置 | 内置 2D/3D 混合布局 | 嵌入式 UI 需要内置布局 |
| **事件系统** | 事件冒泡 | ECS 事件 | ECS 事件 + 可选冒泡 | 保留 ECS 性能，提供 LVGL 易用性 |
| **样式系统** | 类似 CSS 的样式 | 程序化 | 类似 LVGL 的样式组件 | 嵌入式开发熟悉 |
| **渲染优化** | CPU/GPU 混合 | GPU 批处理 | GPU 批处理 (2D/3D 统一) | 现代 GPU 优化 |

### 9.4 控件类型层次

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

***

## 10. 模块化设计优势

### 10.1 文件夹/mod.rs 结构的优势

```
main_world/
├── mod.rs          # 仅作为衔接：导出子模块
├── world.rs        # MainWorld 实现
├── entity.rs       # Entity 实现
├── component.rs    # Component 实现
└── system.rs       # System 实现
```

**优势：**

1. **清晰的职责分离**：每个文件专注于单一功能
2. **易于导航**：通过 mod.rs 快速了解模块结构
3. **便于维护**：修改一个功能不会影响其他文件
4. **易于测试**：每个文件可以独立测试
5. **可扩展性**：添加新功能只需创建新文件

### 10.2 与 LVGL 的对比

| 方面 | FHRE v2.0 (模块化) | LVGL (传统) |
|------|-------------------|-------------|
| 文件组织 | 文件夹/mod.rs | 单一大文件 |
| 可读性 | 高 | 中 |
| 可维护性 | 高 | 中 |
| 扩展性 | 高 | 中 |
| 编译速度 | 快 (增量编译) | 较慢 |

***

## 11. 总结

FHRE v2.0 实现了与 LVGL 类似的 NuttX SIM 平台 framebuffer 对接机制：

1. **Main World**: 游戏逻辑、实体组件、系统调度
2. **Render World**: 渲染对象、绘制命令、帧缓冲区
3. **Extract**: 数据同步桥梁
4. **SimDisplay**: SIM 平台显示驱动 (`/dev/fb0` + ioctl)
5. **刷新机制**: 类似 LVGL 的 `FBIO_UPDATE` 触发 X11 刷新
6. **模块化结构**: 文件夹/mod.rs 设计，mod.rs 仅作为衔接

适用于 FeatherOS 嵌入式系统的现代化渲染需求，同时兼容 NuttX SIM 平台的模拟开发环境。
