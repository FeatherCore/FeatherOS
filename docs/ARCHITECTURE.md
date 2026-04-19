# FHRE (Feather Hybrid Render Engine) v2.4

## 概述

FHRE 是一个轻量级混合渲染引擎，采用声明式双世界架构，灵感来自 Bevy 的 ECS 和渲染图设计，适配嵌入式系统。

### 核心特性

- **声明式双世界架构** (Main World + Render World)
- **ECS (Entity-Component-System)** 设计，对齐 Bevy
- **Observer 机制** - 自动检测 `SyncToRenderWorld` 变化
- **软件渲染后端** - CPU 光栅化，无 GPU 依赖
- **单相机架构** - 唯一 3D 透视相机 + 幕布系统
- **嵌入式友好的无标准库实现** (`no_std`)
- **跨平台窗口抽象** - X11 / Framebuffer / 用户自定义

---

## 1. 架构总览

```
┌─────────────────────────────────────────────────────────────────┐
│                        Application Layer                         │
│                                                                  │
│   用户代码构建 App，使用 WindowRunner 控制主循环                   │
│                                                                  │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │  WindowRunner::run()                                     │  │
│   │    loop {                                                │  │
│   │      collect_input_events()  // 收集平台输入              │  │
│   │      bridge_input()          // 桥接到 FHRE 输入资源      │  │
│   │      app.update_and_render() // 执行一帧                 │  │
│   │      window.present(framebuffer) // 呈现到屏幕            │  │
│   │    }                                                     │  │
│   └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         FHRE Core                                │
│                                                                  │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐    │
│   │  Main World │    │ Extract     │    │  Render World   │    │
│   │  (ECS)      │───▶│ (Sync)      │───▶│  (软件渲染)      │    │
│   │             │    │             │    │                 │    │
│   │ - Entities  │    │ - Extract   │    │ - Views         │    │
│   │ - Components│    │   Systems   │    │ - Commands      │    │
│   │ - Systems   │    │ - Observer  │    │ - Framebuffer   │    │
│   │ - Resources │    │   Mechanism │    │                 │    │
│   └─────────────┘    └─────────────┘    └─────────────────┘    │
│                                                                  │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  Systems Schedule                                        │   │
│   │  Startup → PreUpdate → Update → PostUpdate              │   │
│   └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Platform Layer                              │
│                                                                  │
│   ┌───────────┐  ┌───────────┐  ┌───────────────────────────┐  │
│   │ X11Window │  │ FBWindow  │  │ MyPlatformWindow          │  │
│   │ (sim)     │  │ (nuttx)   │  │ (用户自定义)               │  │
│   │           │  │           │  │                           │  │
│   │ impl Win  │  │ impl Win  │  │ impl Window trait         │  │
│   └───────────┘  └───────────┘  └───────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 单相机架构 + 幕布系统

### 2.1 核心架构

```
3D 主世界（唯一 3D 透视摄像机）
    │
    │ 摄像机采集（MVP 变换）
    │ 摄像机 target 自动追踪幕布位置
    ▼
2D 屏幕幕布（PrimaryScreen，3D 世界中的投影平面）
    │  - 含 Transform3D，可操作（拉近拉远、旋转等）
    │  - 3D 控件：经 MVP 投影到幕布坐标
    │  - 2D UI：直接用幕布像素坐标绘制
    │  - 默认与呈现窗口 1:1 重叠
    ▼
呈现窗口（X11 / FB / 用户定义，不属于 FHRE）
```

### 2.2 核心原则

1. **主世界是纯 3D 的**，只有一个 3D 透视摄像机
2. **2D 屏幕幕布是摄像机投影平面**，所有内容统一绘制到幕布上
3. **幕布是 3D 世界中的对象**，含 Transform3D，可操作
4. **摄像机 target 自动追踪幕布位置**，幕布移动时摄像机跟随
5. **呈现窗口与 FHRE 主世界独立**，由用户定义

### 2.3 默认位置关系

```
Camera:  position = (width/2, height/2, 600)   ← 在幕布正后方 600 单位
         target   = (width/2, height/2, 0)      ← 看向幕布中心
Canvas:  position = (width/2, height/2, 0)      ← 幕布在 z=0 平面
```

### 2.4 幕布操作效果

```rust
// 幕布靠近相机 → 呈现窗口放大
canvas.move_closer(100.0);
// Camera target 自动追踪到 (width/2, height/2, 100)

// 幕布远离相机 → 呈现窗口缩小
canvas.move_further(100.0);
// Camera target 自动追踪到 (width/2, height/2, -100)
```

### 2.5 幕布坐标系

```
(0,0) ─────────────── (width, 0)
  │                       │
  │    幕布平面           │
  │    原点左上角          │
  │    Y 轴向下            │
  │    单位：像素          │
(0,height) ───────── (width, height)
```

- 3D 对象经 Camera MVP 投影后输出到此坐标系
- 2D UI 的 Transform2D 直接使用此坐标系

### 2.6 渲染流程（以 640×480 为例）

```
1. 初始化
   App::new(640, 480)
     → PrimaryScreen::new(640, 480)
       → transform = Transform3D::from_position(320, 240, 0)  // 幕布中心
     → CameraPlugin::build()
       → canvas_pos = (320, 240, 0)
       → Camera position = (320, 240, 600)  // 幕布正后方 600
       → Camera target   = (320, 240, 0)    // 看向幕布中心

2. 用户 Setup
   Cube:  Transform3D::from_position(320, 240, 0)  // 3D 世界坐标
   Button: Transform2D::from_position(320, 400)    // 幕布像素坐标

3. Extract 阶段
   a. 读取 canvas_pos = PrimaryScreen.position() = (320, 240, 0)
   b. 构建 View:
      view       = look_at_rh((320,240,600), (320,240,0), (0,1,0))
      projection = perspective_rh(45°, 640/480, 0.1, 1000)
      VP         = projection × view
   c. Cube 投影:
      本地顶点 → 旋转 → + Transform3D.position → 世界坐标
      世界坐标 × VP → NDC → world_to_screen() → 幕布像素坐标
      生成 DrawPolygon 命令
   d. Button 绘制:
      Transform2D.position 直接作为 Rect 坐标
      生成 DrawRect 命令

4. 坐标系统一性
   ✅ Cube 经 MVP 后输出 (0,0)~(640,480) 像素坐标 — 和 Button 相同
   ✅ 两者都在同一个幕布坐标系上
```

---

## 3. 模块结构

| 模块 | 职责 | 关键类型 |
|------|------|----------|
| `app` | 应用生命周期管理 | `App`, `FHRE_VERSION` |
| `main_world` | ECS 主世界 | `Entity`, `Component`, `System`, `Query`, `Commands` |
| `render_world` | 渲染世界 | `RenderWorld`, `View`, `RenderCommand`, `RenderComponent` |
| `extract` | 数据提取 | `ExtractComponent`, `Extractors`, `Extract<P>` |
| `sync` | 世界同步 | `SyncToRenderWorld`, `RenderEntity`, `MainEntity` |
| `schedule` | 调度系统 | `Startup`, `Update`, `Schedule`, `ScheduleLabel` |
| `plugin` | 插件系统 | `Plugin` trait, `DefaultPlugins` |
| `resources` | 资源管理 | `Time`, `PrimaryScreen`, `Camera`, `Resource` trait |
| `node` | 节点系统 | `Node`, `Transform2D`, `Transform3D` |
| `animation` | 动画系统 | `AnimationClip`, `AnimationPlayer`, `KeyframeCurve` |
| `input` | 输入系统 | `ButtonInput`, `KeyCode`, `MouseButton` |
| `window` | 窗口抽象 | `Window` trait, `WindowRunner`, `WindowInputAdapter` |
| `pipeline` | 渲染管线 | `SoftwareBackend`, `RenderBatch` |
| `math` | 数学工具 | `Vec2`, `Vec3`, `Mat4`, `Color`, `Rect` |
| `event` | 事件系统 | `Events<T>`, `EventReader`, `EventWriter` |

---

## 4. 系统宏

### 4.1 系统参数数量

FHRE 支持 1-9 个参数的系统定义：

```rust
// 支持 system1 到 system9
pub use main_world::{system1, system2, system3, system4, system5, system6, system7, system8, system9};
```

### 4.2 使用示例

```rust
use fhre::{system3, Res, ResMut, Query};

fn my_system(time: Res<Time>, mut state: ResMut<State>, query: Query<&Transform>) {
    // system logic
}

app.add_systems(Update, system3::<Res<Time>, ResMut<State>, Query<&Transform>, _>(my_system));
```

### 4.3 declare_system! 宏

简化系统注册：

```rust
use fhre::declare_system;

app.add_systems(Update, declare_system!(my_system; Res<Time>, ResMut<State>, Query<&Transform>));
```

---

## 5. 双世界架构

### 5.1 架构对比

| 组件 | Bevy | FHRE |
|------|------|------|
| **Main World** | 完整 ECS | 完整 ECS |
| **Render World** | 完整 ECS (SubApp) | 完整 ECS |
| **实体同步** | `SyncToRenderWorld` 标记 + Observer | `SyncToRenderWorld` 标记 + Observer |
| **实体映射** | `RenderEntity`/`MainEntity` | `RenderEntity`/`MainEntity` |
| **组件提取** | `ExtractComponent` trait | `ExtractComponent` trait |
| **提取调度** | `ExtractSchedule` (SubApp Schedule) | `Extractors` + `App::add_extractor()` |
| **渲染后端** | wgpu (GPU) | SoftwareBackend (CPU) |

### 5.2 提取流程

```
┌─────────────────────────────────────────────────────────────────┐
│                        Main World Update                         │
│                                                                  │
│   Startup → PreUpdate → Update → PostUpdate                     │
│                                                                  │
│   系统运行，修改组件数据                                          │
│   Observer 机制自动检测 SyncToRenderWorld 变化：                  │
│   - insert_component<SyncToRenderWorld> → Added 记录            │
│   - remove_component<SyncToRenderWorld> → Removed 记录          │
│   - despawn 带 RenderEntity 的实体 → Removed 记录               │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Extract Phase                             │
│                                                                  │
│   1. entity_sync_system()                                       │
│      - 处理 PendingSyncEntity 队列                              │
│      - Added: 在 Render World 创建对应实体                       │
│      - Removed: 在 Render World 删除对应实体                     │
│      - 建立 MainEntity ↔ RenderEntity 映射                       │
│                                                                  │
│   2. extractors.run()                                           │
│      - 运行用户注册的提取函数                                     │
│      - 从 Main World 读取数据，写入 Render World                 │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Render Phase                              │
│                                                                  │
│   render_world.execute_render()                                  │
│   - 执行渲染命令                                                  │
│   - 输出到 framebuffer                                           │
└─────────────────────────────────────────────────────────────────┘
```

### 5.3 同步标记组件

```rust
/// 标记实体需要同步到 Render World
pub struct SyncToRenderWorld;

/// Main World 实体持有，指向对应的 Render World 实体
pub struct RenderEntity(pub Entity);

/// Render World 实体持有，指向对应的 Main World 实体
pub struct MainEntity(pub Entity);
```

---

## 6. ECS 核心

### 6.1 系统参数

| 参数 | 说明 | Bevy 对齐 |
|------|------|-----------|
| `Res<T>` | 只读资源 | ✅ |
| `ResMut<T>` | 可变资源 | ✅ |
| `Query<T, F>` | 组件查询 | ✅ |
| `Commands` | 延迟命令 | ✅ |
| `Local<T>` | 系统本地状态 | ✅ |
| `Entity` | 实体 ID | ✅ |

### 6.2 查询过滤器

| 过滤器 | 说明 | Bevy 对齐 |
|--------|------|-----------|
| `With<T>` | 要求组件存在 | ✅ |
| `Without<T>` | 要求组件不存在 | ✅ |
| `Added<T>` | 组件刚添加 | ✅ |
| `Changed<T>` | 组件刚修改 | ✅ |

### 6.3 调度系统

```
Startup → PreUpdate → Update → PostUpdate
    │          │          │          │
    │          │          │          └── Commands apply
    │          │          └── Commands apply
    │          └── Commands apply
    └── Commands apply (一次性)
```

---

## 7. Commands 延迟执行

### 7.1 问题背景

`Commands` 是延迟执行的：在系统调用 `commands.spawn()` 后，实体要到当前 Schedule 结束时才会真正创建。这意味着同一帧内后续系统的 `Query` 无法查询到新创建的实体。

### 7.2 示例场景

```rust
fn model_switch_system(
    mut commands: Commands,
    mut players: Query<&mut AnimationPlayer>,
) {
    commands.spawn()
        .insert(AnimationPlayer::new())
        .insert(SyncToRenderWorld);
    
    // 此时 players.iter_mut() 查询不到新实体！
    // 因为 Commands 还未 apply
}
```

### 7.3 解决方案

在创建实体时就完成初始化，而不是依赖后续系统：

```rust
fn model_switch_system(
    mut commands: Commands,
    clips: Res<AnimationResources>,
) {
    let mut player = AnimationPlayer::new();
    player.play_with_target(clip_handle, target_id);
    
    commands.spawn()
        .insert(player)
        .insert(SyncToRenderWorld);
}
```

### 7.4 最佳实践

1. **创建时初始化**：在 spawn 时就设置好所有初始状态
2. **避免帧内查询新实体**：不要在同一帧内用 Query 查询刚创建的实体
3. **使用 Added<T>**：下一帧用 `Added<T>` 过滤器处理新实体

---

## 8. 动画系统

### 8.1 核心组件

| 类型 | 说明 |
|------|------|
| `AnimationClip` | 动画剪辑数据（关键帧曲线集合） |
| `AnimationClipHandle` | 动画剪辑引用句柄 |
| `AnimationPlayer` | 动画播放器组件，挂载到实体上 |
| `AnimationResources` | 动画资源存储（全局资源） |
| `KeyframeCurve` | 关键帧曲线 |

### 8.2 AnimationPlayer API

```rust
// 创建播放器
let mut player = AnimationPlayer::new();

// 播放动画（一次性）
player.play(clip_handle);

// 循环播放
player.play_repeat(clip_handle);

// 播放并设置目标（推荐）
player.play_with_target(clip_handle, target_id);

// 控制播放
player.pause_all();
player.resume_all();
player.stop_all();

// 设置播放速度
player.animation_mut(0).unwrap().speed = 2.0;
```

### 8.3 play_with_target() 方法

`play_with_target()` 是一个便捷方法，同时设置动画剪辑、目标 ID 和循环模式：

```rust
pub fn play_with_target(&mut self, clip_handle: AnimationClipHandle, target: AnimationTargetId) -> &mut ActiveAnimation {
    let anim = self.play(clip_handle);
    anim.target_id = target;
    anim.repeat = RepeatAnimation::Forever;
    anim
}
```

这个方法解决了 Commands 延迟执行问题：在创建 AnimationPlayer 时就设置好动画，而不是在后续系统中查询并设置。

---

## 9. 渲染后端

### 9.1 软件渲染架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    SoftwareBackend                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  framebuffer: Vec<u32>  // ARGB8888 像素数组                     │
│  width: u32, height: u32                                        │
│  viewport: Rect                                                  │
│                                                                  │
│  核心算法:                                                       │
│  - fill_rect: 逐像素填充矩形                                     │
│  - fill_triangle: 包围盒 + 重心坐标判断                          │
│  - draw_line: Bresenham 直线算法                                 │
│  - fill_polygon: 扫描线填充算法                                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 9.2 渲染命令

```rust
pub enum RenderCommand {
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawLine { start: Vec2, end: Vec2, color: Color, thickness: f32 },
    DrawTriangle { p0: Vec2, p1: Vec2, p2: Vec2, color: Color },
    DrawPolygon { vertices: Vec<Vec2>, color: Color },
    DrawText { position: Vec2, text: &'static str, color: Color, size: f32 },
    SetScissor { rect: Rect },
    DisableScissor,
}
```

### 9.3 渲染阶段

```
Background    → 清除/背景绘制
     ↓
Opaque2d      → 不透明 2D 物体
     ↓
Opaque3d      → 不透明 3D 物体（画家算法排序）
     ↓
AlphaMask     → Alpha 测试物体
     ↓
Transparent   → 透明物体（从后往前排序）
     ↓
Ui            → UI 覆盖层
```

---

## 10. 输入系统

### 10.1 架构设计

```
底层硬件 (evdev / nuttx input)
    ↓
输入驱动 (读取原始事件)
    ↓
事件分发 (WindowInputEvents)
    ↓
WindowRunner::bridge_input()
    ↓
ButtonInput<T> Resources
    ↓
游戏/UI 逻辑查询
```

### 10.2 使用示例

```rust
fn keyboard_system(keys: Res<ButtonInput<KeyCode>>) {
    if keys.pressed(KeyCode::KeyW) { /* 移动 */ }
    if keys.just_pressed(KeyCode::Space) { /* 跳跃 */ }
}

fn mouse_system(mouse: Res<ButtonInput<MouseButton>>) {
    if mouse.pressed(MouseButton::Left) { /* 拖拽 */ }
}
```

---

## 11. 核心 API

### 11.1 应用构建

```rust
use fhre::prelude::*;

let mut app = App::new(640, 480);

app.add_plugins(DefaultPlugins)
   .insert_resource(MyState::new())
   .add_systems(Startup, setup)
   .add_systems(Update, my_system)
   .add_extractor(extract_view)
   .add_extractor(extract_my_components);
```

### 11.2 系统定义

```rust
fn setup(mut commands: Commands) {
    commands.spawn()
        .insert(Transform2D::from_xy(100.0, 50.0))
        .insert(Button::new(80, 30))
        .insert(SyncToRenderWorld);
}

fn my_system(query: Query<&mut Transform2D, With<Button>>, time: Res<Time>) {
    for (entity, mut transform) in query.iter_mut() {
        transform.position.x += 1.0;
    }
}
```

### 11.3 提取器

```rust
fn extract_buttons(main_world: &MainWorld, render_world: &mut RenderWorld) {
    for (entity, transform) in main_world.query::<Transform2D>() {
        if let Some(button) = main_world.get_component::<Button>(entity) {
            let render_entity = render_world.get_or_spawn_synced(entity);
            
            let ui = ExtractedUI {
                position: Vec2::new(transform.position.x, transform.position.y),
                width: button.width,
                height: button.height,
                color: button.current_color(),
            };
            
            render_world.insert_component(render_entity, ui);
        }
    }
}

app.add_extractor(extract_buttons);
```

### 11.4 主循环

```rust
// 使用 WindowRunner（推荐）
let mut window = X11Window::new(640, 480, "App")?;
let input_adapter = InputAdapter;

WindowRunner::new(&mut app, &mut window, &input_adapter)
    .with_frame_delay_ms(16)
    .run();

// 或手动控制
loop {
    app.update_and_render();
    window.present(app.framebuffer());
}
```

---

## 12. 平台适配

### 12.1 实现 Window trait

```rust
pub struct MyWindow { /* ... */ }

impl Window for MyWindow {
    fn is_running(&self) -> bool { true }
    fn collect_input_events(&mut self) -> WindowInputEvents { /* ... */ }
    fn present(&mut self, framebuffer: &[u32]) { /* ... */ }
    fn dimensions(&self) -> (u32, u32) { (self.width, self.height) }
}
```

### 12.2 实现 WindowInputAdapter trait

```rust
pub struct MyInputAdapter;

impl WindowInputAdapter for MyInputAdapter {
    fn map_keycode(&self, code: u32) -> Option<KeyCode> { /* ... */ }
    fn map_mouse_button(&self, btn: u32) -> Option<MouseButton> { /* ... */ }
}
```

---

## 13. 与 Bevy 的差异

| 特性 | Bevy | FHRE | 原因 |
|------|------|------|------|
| **过程宏** | 完整支持 | 受限 | `no_std` 环境 |
| **Archetype 存储** | ✅ | ❌ | 简化为 BTreeMap |
| **并行系统** | ✅ | ❌ | 嵌入式单线程 |
| **GPU 渲染** | ✅ | ❌ | 软件渲染 |
| **Render World** | 完整 ECS (SubApp) | 完整 ECS | 对齐设计 |
| **ExtractSchedule** | SubApp Schedule | `App::add_extractor()` | 单线程简化 |
| **渲染后端** | wgpu (GPU) | SoftwareBackend (CPU) | 嵌入式无 GPU |

---

## 14. 示例项目结构

```
examples/fhre/rust/
├── Cargo.toml              # Rust 包配置
├── Makefile                # NuttX 构建脚本
├── src/
│   ├── lib.rs              # 主入口文件
│   ├── extract.rs          # 自定义提取器
│   ├── components/
│   │   ├── mod.rs          # 组件模块导出
│   │   ├── button.rs       # 按钮组件
│   │   ├── cube.rs         # 立方体组件
│   │   └── soccer_ball.rs  # 足球组件
│   └── platform/
│       ├── mod.rs          # 平台模块导出
│       ├── x11.rs          # X11 模拟器平台
│       └── framebuffer.rs  # 嵌入式 Framebuffer 平台
```

---

## 15. 版本历史

| 版本 | 变更 |
|------|------|
| **v2.4.0** | 扩展系统参数：`system8`/`system9` 支持最多 9 个参数；`AnimationPlayer::play_with_target()` 便捷方法；修复 Commands 延迟执行导致的动画初始化问题 |
| **v2.3.0** | Observer 机制：自动检测 `SyncToRenderWorld` 变化；统一 ScheduleLabel 定义；废弃 `Transform` 使用 `Transform2D`/`Transform3D`；简化 `Extractors`；修复 XImage 内存泄漏 |
| **v2.2.0** | 重构提取系统：`App::add_extractor()` 替代 `ExtractSchedule` Resource；移除 UI 组件到示例 |
| **v2.1.0** | 删除未使用组件；修复潜在 panic；清理硬编码 |
| **v2.0.0** | 单相机架构重构；移除 `App::run()`, `AppRunner`；重构主循环控制；Bevy ECS 对齐 |
| **v1.0.0** | 初始版本 |
