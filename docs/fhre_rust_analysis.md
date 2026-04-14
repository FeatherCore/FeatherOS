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
│   │           ├── renderer/          # Renderer 模块
│   │           │   ├── mod.rs         # 衔接：导出 renderer, framebuffer, target
│   │           │   ├── renderer.rs    # Renderer 实现
│   │           │   ├── framebuffer.rs # Framebuffer 实现
│   │           │   └── target.rs      # RenderTarget 实现
│   │           ├── math/              # Math 模块
│   │           │   ├── mod.rs
│   │           │   ├── vec2.rs
│   │           │   ├── vec3.rs
│   │           │   ├── color.rs
│   │           │   └── rect.rs
│   │           └── platform/          # Platform 模块
│   │               ├── mod.rs         # 衔接：导出 sim, nuttx, default
│   │               ├── sim.rs         # NuttX SIM 平台支持
│   │               ├── nuttx.rs       # NuttX 平台支持
│   │               └── default.rs     # 默认平台支持
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
    pub renderer: Renderer,
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

**文件**: `render_world/mod.rs` (衔接) + `world.rs` + `command.rs` + `object.rs`

```rust
// render_world/mod.rs - 仅作为衔接
mod world;
mod command;
mod object;

pub use world::RenderWorld;
pub use command::{RenderCommand, DrawCall};
pub use object::{RenderObject, ExtractedTransform, ExtractedSprite};

// render_world/world.rs - RenderWorld 实现
pub struct RenderWorld {
    width: u32,
    height: u32,
    framebuffer: Vec<u32>,
    objects: Vec<RenderObject>,
    commands: Vec<RenderCommand>,
    clear_color: Color,
}

// render_world/command.rs - 渲染命令
pub enum RenderCommand {
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawLine { start: Vec2, end: Vec2, color: Color, thickness: f32 },
    // ...
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

### 5.4 schedule 模块

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

### 5.5 resources 模块

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
```

### 5.6 renderer 模块

**文件**: `renderer/mod.rs` (衔接) + `renderer.rs` + `framebuffer.rs` + `target.rs`

```rust
// renderer/mod.rs - 仅作为衔接
mod renderer;
mod framebuffer;
mod target;

pub use renderer::Renderer;
pub use framebuffer::Framebuffer;
pub use target::RenderTarget;

// renderer/renderer.rs - Renderer 实现
pub struct Renderer {
    scissor: Option<Rect>,
    stats: RenderStats,
}

// renderer/framebuffer.rs - Framebuffer 实现
pub struct Framebuffer {
    width: u32,
    height: u32,
    data: Vec<u32>,
}

// renderer/target.rs - RenderTarget trait
pub trait RenderTarget {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn clear(&mut self, color: Color);
    fn draw_pixel(&mut self, x: u32, y: u32, color: Color);
    fn data(&self) -> &[u32];
}
```

### 5.7 extract 模块

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

### 5.8 platform/sim 模块

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
pub fn refresh_loop<F>(render_fn: F)
```

***

## 6. 示例代码

### 6.1 使用 SIM 平台显示

```rust
#![no_std]
#![no_main]

use fhre::{
    App, SimDisplay, create_display, FB_DEVICE_PATH,
    main_world::{Transform, Sprite, Velocity},
    math::{Color, Vec2},
};

#[no_mangle]
pub extern "C" fn fhre_rust_main(_argc: i32, _argv: *const *const u8) -> i32 {
    // Create app with SIM display support
    let mut app = App::new();
    
    // Check if SIM display is available
    if app.sim_display.is_some() {
        printf("SIM display initialized: %s\n", FB_DEVICE_PATH);
    }
    
    // Spawn entities
    let entity = app.main_world.spawn();
    app.main_world.insert_component(entity, Transform::from_2d(100.0, 100.0));
    app.main_world.insert_component(entity, Sprite::rect(50.0, 50.0, Color::RED));
    app.main_world.insert_component(entity, Velocity::new(2.0, 1.5));
    
    // Main loop
    for frame in 0..300 {
        // Update game logic
        update_entities(&mut app);
        
        // Render and present
        // Automatically calls SimDisplay::present() which:
        // 1. Copies to backbuffer
        // 2. Copies to hardware framebuffer
        // 3. Calls ioctl(FBIO_UPDATE) to trigger X11 refresh
        app.update();
    }
    
    0
}
```

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

## 9. 模块化设计优势

### 9.1 文件夹/mod.rs 结构的优势

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

### 9.2 与 LVGL 的对比

| 方面 | FHRE v2.0 (模块化) | LVGL (传统) |
|------|-------------------|-------------|
| 文件组织 | 文件夹/mod.rs | 单一大文件 |
| 可读性 | 高 | 中 |
| 可维护性 | 高 | 中 |
| 扩展性 | 高 | 中 |
| 编译速度 | 快 (增量编译) | 较慢 |

***

## 10. 总结

FHRE v2.0 实现了与 LVGL 类似的 NuttX SIM 平台 framebuffer 对接机制：

1. **Main World**: 游戏逻辑、实体组件、系统调度
2. **Render World**: 渲染对象、绘制命令、帧缓冲区
3. **Extract**: 数据同步桥梁
4. **SimDisplay**: SIM 平台显示驱动 (`/dev/fb0` + ioctl)
5. **刷新机制**: 类似 LVGL 的 `FBIO_UPDATE` 触发 X11 刷新
6. **模块化结构**: 文件夹/mod.rs 设计，mod.rs 仅作为衔接

适用于 FeatherOS 嵌入式系统的现代化渲染需求，同时兼容 NuttX SIM 平台的模拟开发环境。
