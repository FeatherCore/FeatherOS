# Wing Unified Architecture

本文是新的 Wing 架构基线，用于取代过去 `FHRE + Wing` 分离式设计。

目标不是继续维护一个独立的 FHRE 引擎，再让 Wing 作为上层 shell 适配它；新的方向是：

```text
Wing = 声明式 ECS UI + Shell + 渲染引擎 + 平台适配
```

也就是说，FHRE 中真正有价值的图形能力、数学类型、输入抽象、软件渲染后端、资源格式和示例效果，应该合入 Wing 内部，成为 Wing 的模块，而不是继续作为一个单独 crate 和单独抽象层存在。

本文中的 ECS 对应前面讨论里的“声明式 ESC”。后续文档统一写作 ECS。

当前落地代码采用更扁平的第一阶段目录：`animation.rs`、`app.rs`、`input.rs`、`math.rs`、`runtime.rs`、`shell.rs`、`ui.rs`、`core/`、`render/`、`platform/`。后续只有当单个模块明显变大，才拆成同名目录下的多个文件，避免在重构早期把目录结构做得比运行时本身还重。

## 1. 重新定位

Wing 的目标平台是 NuttX + Rust，最终运行在高性能 MCU 或低功耗 MPU 上，典型设备包括：

- 智能手表
- MP4 / 随身播放器
- 小屏手机
- 复古分辨率掌机
- 低功耗控制面板

这些平台的共同特点是：

- CPU 和内存都有限
- 很多设备只有 framebuffer 和纯软件渲染
- 有些设备有 2D blitter、DMA2D、PXP、VG-Lite 之类的 2D/2.5D 加速
- 少量设备可能有初级 OpenGL ES 3.0 能力
- 屏幕小，UI 元素数量可控
- shell 的响应速度和功耗比通用性更重要

因此 Wing 不应该追求“Rust 版 Bevy 小引擎”，而应该是一个面向嵌入式 shell 和小屏应用的轻量 UI 运行时。

新的原则：

```text
轻量优先
声明式 ECS 优先
单 crate 优先
显式流程优先
低分配优先
可预测性能优先
```

## 2. 当前 FHRE + Wing 的主要问题

### 2.1 抽象边界过重

旧设计中：

```text
Wing -> FHRE App -> MainWorld -> RenderWorld -> Extractor -> RenderCommand -> Backend
```

这个流程来自 Bevy 的游戏引擎架构。它适合大型通用引擎，但对 Wing 的目标平台偏重。

具体问题：

- Wing 和 FHRE 是两个 crate，接口层多，概念重复。
- FHRE 自己有 App、Plugin、DefaultPlugins、MainWorld、RenderWorld、Extractors。
- Wing 又有 shell components、resources、systems、extractors。
- 为了让 Wing 画一个界面，需要经过完整的双世界同步和提取流程。
- RenderWorld 本身也是一套 ECS，很多数据只是过帧临时渲染数据，却被长期 ECS 化。

新的设计里，Wing 只保留一个主 ECS 世界，渲染阶段生成短生命周期 DrawList，不再维护第二个 RenderWorld ECS。

### 2.2 Bevy 风格系统不适合无宏轻量目标

旧 FHRE 里存在：

- `declare_system!`
- `system1` 到 `system10`
- tuple query 宏
- `Plugin` / `PluginGroup`
- `DefaultPlugins`
- `SystemParam`
- `Res<T>` / `ResMut<T>` 的函数参数注入

这些东西带来了较好的 Bevy 观感，但问题是：

- 实现复杂。
- 宏和泛型膨胀增加代码体积。
- 系统注册不够直观。
- 为了模拟 Bevy 参数注入，内部有较多 unsafe 借用绕过。
- 对 NuttX + MCU 目标来说收益不够大。

新的 Wing 不再追求函数签名自动注入，而是采用显式系统函数：

```rust
pub type SystemFn = fn(&mut WingRuntime);

fn gesture_system(rt: &mut WingRuntime) {
    let input = rt.resources.get::<InputState>();
    let shell = rt.resources.get_mut::<ShellState>();
    // 通过 world.query 显式访问组件
}
```

系统仍然是声明式 ECS：状态在 components/resources 中，行为在 system 中，绘制由组件推导而来。只是系统参数获取变得显式、无宏、可控。

### 2.3 BTreeMap + Box<dyn Any> 不适合作为热路径

旧 ECS 组件存储大致是：

```text
TypeId -> BTreeMap<EntityId, Box<dyn Any>>
```

这很容易实现，但热路径性能较弱：

- 每个组件是独立 Box，缓存局部性差。
- 查询时需要 downcast。
- BTreeMap 访问成本高。
- 多组件查询会在 entity 列表和多个 map 间反复查找。

小屏 UI 的组件类型和实体数量有限，应该使用更简单、更紧凑的存储。

新的 Wing 第一阶段推荐使用 sparse-set 组件存储：

```text
Storage<T>
  dense_entities: Vec<Entity>
  dense_values:   Vec<T>
  sparse:         Vec<Option<u16/u32>>
```

特点：

- 单组件遍历是连续内存。
- 插入、删除、查找接近 O(1)。
- 不需要 archetype 迁移。
- 对 UI/shell 这种实体组合较稳定的场景足够快。
- 比完整 hecs archetype 更容易在 no_std + alloc 下控制代码体积。

hecs 的价值在于实体 generation、archetype column storage、query 思路；不建议直接搬整套 hecs，也不建议引入它的宏和依赖。

planck_ecs 的价值在于 system/dispatcher 与 component storage 分离、低 unsafe 思路；但本地版本依赖 std，不适合作为直接移植对象。

`/home/uan-gpd/codes/freecs` 本地仓库不是 Rust ECS 库，而是 Nuclide/FTEQW 体系的游戏逻辑项目，不作为 Wing ECS 直接参考。

### 2.4 “纯 3D 引擎”语义不适合 Wing 默认 UI

旧 `docs/ARCHITECTURE.md` 强调 FHRE 是纯 3D 引擎，2D 是 3D 的特例。

这个表达对 demo 中的足球、立方体有意义，但对 Wing 默认 shell 不合理：

- Wing 的主场景是小屏 UI。
- 绝大部分元素是 2D/2.5D shell 对象。
- UI 的主要坐标应该是屏幕逻辑坐标，而不是被迫套入 3D 相机/幕布模型。
- 3D 效果应该是局部效果，例如卡片预览足球、立方体、翻转、景深。

新的设计：

```text
Wing 默认是 2D UI 坐标系统。
2.5D/3D 是局部 Effect 或 SurfaceTransform。
```

默认 UI 坐标：

- 原点在左上角。
- x 向右，y 向下。
- 单位为逻辑像素。
- z/layer 只用于排序和局部变换。

需要足球、立方体、旋转卡片时，使用 `EffectNode` 或 `MeshNode` 进入 2.5D/3D 子管线，而不是让所有 UI 都变成 3D 对象。

## 3. 新总架构

```text
┌──────────────────────────────────────────────────────────────┐
│ wing_rust NuttX application                                  │
│ - C ABI entry                                                │
│ - allocator / panic handler                                  │
│ - platform init                                              │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│ WingRuntime                                                   │
│ - World                                                       │
│ - Resources                                                   │
│ - Schedule                                                    │
│ - Renderer                                                    │
│ - Platform                                                    │
│ - FrameArena / DrawList                                      │
└──────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼────────────────┐
              ▼               ▼                ▼
┌───────────────────┐ ┌────────────────┐ ┌───────────────────┐
│ ECS Core           │ │ Shell/UI        │ │ Render Core        │
│ - Entity           │ │ - Home          │ │ - DrawCmd          │
│ - Component store  │ │ - Notifications │ │ - TextureStore     │
│ - Resources        │ │ - App switcher  │ │ - Atlas            │
│ - Events           │ │ - Settings      │ │ - Font/Glyph       │
│ - Schedule         │ │ - Gestures      │ │ - Software backend │
└───────────────────┘ └────────────────┘ └───────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│ Platform                                                      │
│ - framebuffer present                                         │
│ - touch/key input                                             │
│ - timer                                                       │
│ - NuttX task/app launch                                       │
└──────────────────────────────────────────────────────────────┘
```

帧流程：

```text
poll platform input
  -> update InputState / EventQueue
  -> run PreUpdate systems
  -> run Update systems
  -> run Layout systems
  -> run Animation systems
  -> hit-test / focus update
  -> build DrawList from ECS components
  -> renderer executes DrawList
  -> platform presents dirty region
  -> clear transient frame data
```

核心变化：

- 不再有 FHRE crate。
- 不再有 `MainWorld -> RenderWorld` 双 ECS。
- 不再有 extractor 作为长期架构边界。
- 不再有 `PluginGroup` 和默认插件。
- 渲染数据是每帧短生命周期 `DrawList`。
- ECS 世界只保存业务和 UI 状态。
- 后端只消费 `DrawCmd`。

## 4. 目录规划

合并后的主目录：

```text
apps/wing/
├── Kconfig
├── Makefile
├── resource/
│   ├── fs/                       # copied to /etc/wing/resource/
│   │   ├── fonts/simhei.ttf
│   │   ├── images/shell/*.png
│   │   └── icons/shell/*.svg
│   ├── icons/bootstrap/*.svg     # built-in SVG fallback sources
│   ├── generated/                # generated RGB565 fallback + resource partition
│   ├── tools/
│   └── manifest.txt
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── entry.rs
│       ├── core/
│       │   ├── entity.rs
│       │   ├── world.rs
│       │   ├── storage.rs
│       │   ├── resource.rs
│       │   ├── event.rs
│       │   ├── schedule.rs
│       │   └── time.rs
│       ├── math/
│       │   ├── vec2.rs
│       │   ├── vec3.rs
│       │   ├── rect.rs
│       │   ├── color.rs
│       │   └── transform.rs
│       ├── ui/
│       │   ├── components.rs
│       │   ├── layout.rs
│       │   ├── style.rs
│       │   ├── hit_test.rs
│       │   ├── gesture.rs
│       │   ├── animation.rs
│       │   └── theme.rs
│       ├── render/
│       │   ├── draw_cmd.rs
│       │   ├── draw_list.rs
│       │   ├── renderer.rs
│       │   ├── texture.rs
│       │   ├── atlas.rs
│       │   ├── font.rs
│       │   ├── effects.rs
│       │   ├── soft/
│       │   │   ├── mod.rs
│       │   │   ├── blit.rs
│       │   │   ├── fill.rs
│       │   │   ├── blend.rs
│       │   │   └── raster.rs
│       │   ├── gpu2d/
│       │   │   └── mod.rs
│       │   └── gles/
│       │       └── mod.rs
│       ├── shell/
│       │   ├── mod.rs
│       │   ├── home.rs
│       │   ├── launcher.rs
│       │   ├── notification.rs
│       │   ├── app_switcher.rs
│       │   ├── settings.rs
│       │   ├── app_manager.rs
│       │   └── systems.rs
│       ├── platform/
│       │   ├── mod.rs
│       │   ├── window.rs
│       │   ├── framebuffer.rs
│       │   ├── input.rs
│       │   ├── timer.rs
│       │   └── task.rs
│       └── demos/
│           ├── soccer.rs
│           └── cube.rs
```

迁移完成后：

- `apps/fhre` 不再作为构建依赖。
- `apps/examples/fhre` 保留为历史 demo 或迁移到 `apps/wing/rust/src/demos`。
- `apps/examples/wing` 的平台入口并入 `apps/wing`，避免库和示例之间继续绕一层。

## 5. ECS 设计

### 5.1 Entity

```rust
pub struct Entity {
    index: u32,
    generation: u16,
}
```

要求：

- index 可复用。
- generation 防止旧 handle 误命中。
- MCU 小规模配置可把 index 改为 u16。
- 实体删除进入 freelist。

### 5.2 Component storage

第一阶段使用 sparse-set：

```rust
pub struct Storage<T> {
    dense_entities: Vec<Entity>,
    dense_values: Vec<T>,
    sparse: Vec<Option<u32>>,
}
```

操作：

- `insert(entity, component)`
- `remove(entity)`
- `get(entity)`
- `get_mut(entity)`
- `iter()`
- `iter_mut()`

多组件查询选择最小 storage 驱动：

```rust
for (entity, transform, visual) in world.query2::<Transform, Visual>() {
    // build draw command
}
```

为了无宏，第一阶段只手写常用 query：

- `query1<T>()`
- `query1_mut<T>()`
- `query2<A, B>()`
- `query2_mut<A, B>()`
- `query3<A, B, C>()`
- `query3_mut<A, B, C>()`

复杂系统可以拆分为多个简单系统，或者显式用 entity handle 访问组件。

### 5.3 Resources

资源不是组件，不参与 entity query。

```rust
pub struct Resources {
    slots: Vec<ResourceSlot>,
}
```

资源数量很少，可以用 `TypeId` 排序 Vec，而不是 BTreeMap。

常用资源：

- `Time`
- `InputState`
- `GestureState`
- `ShellState`
- `ThemeState`
- `TextureStore`
- `FontStore`
- `AppRegistry`
- `AppManager`
- `DamageState`
- `RuntimeDiagnostics`
- `FrameStats`

系统显式取资源：

```rust
let time = rt.resources.get::<Time>();
let shell = rt.resources.get_mut::<ShellState>();
```

### 5.4 Schedule

无插件、无宏、无自动参数注入。

```rust
pub type SystemFn = fn(&mut WingRuntime);

pub enum Phase {
    Startup,
    Input,
    PreUpdate,
    Update,
    Layout,
    Animation,
    PostUpdate,
    RenderBuild,
}
```

注册方式：

```rust
rt.schedule.add(Phase::Startup, shell_setup_system);
rt.schedule.add(Phase::Input, input_state_system);
rt.schedule.add(Phase::Update, settings_system);
rt.schedule.add(Phase::Layout, shell_layout_system);
rt.schedule.add(Phase::RenderBuild, build_draw_list_system);
```

系统排序由代码显式决定，不做复杂依赖图。小型 shell 不需要 Bevy 级别调度器。

当前 `Schedule` 保存的是固定容量系统函数表，每个 phase 默认最多 `SYSTEMS_PER_PHASE_CAPACITY = 16` 个系统。运行某个 phase 时，Runtime 先读取该 phase 的系统数量，再按 index 取出 `fn(&mut Runtime)` 指针并立即调用，不为本帧创建临时 `Vec<SystemFn>`，初始化注册阶段也不会因为系统表增长触发堆扩容。

### 5.5 Events

事件采用双缓冲 Vec：

```rust
pub struct Events<T> {
    current: Vec<T>,
    previous: Vec<T>,
}
```

帧末 swap/clear。常用事件：

- `PointerEvent`
- `GestureEvent`
- `ShellCommand`
- `AppEvent`
- `ThemeEvent`

输入和手势事件不能直接绘制，只能修改 ECS 状态或资源。

### 5.6 Runtime diagnostics

Wing 的诊断路径保持为普通资源和固定字段统计，不使用日志字符串或动态指标表作为热路径。

当前实现分两层：

- `RuntimeDiagnostics` 记录 shell 主运行时的帧状态和容量溢出。
- `AppUiFrameStats` 记录外部 Wing surface 应用的声明式 AppUi 帧状态。

主运行时每帧提交一个 `RuntimeFrameSnapshot`：

```rust
pub struct RuntimeFrameSnapshot {
    pub rendered: bool,
    pub dirty: bool,
    pub input: bool,
    pub surface_events: u8,
    pub settings_events: u8,
    pub task_exits: u8,
}
```

这些字段会被累计为 `RuntimeFrameDiagnostics`，同时保留最近一帧的 `RuntimeFrameFlags`。System 页面只显示静态标签，例如 `DRAW`、`IDLE`、`INPUT`、`SURFACE`、`SETTING`、`TASK`，避免为了诊断文字引入运行时字符串分配。

AppUi 应用侧则通过 `AppUiLoop::frame_stats()` 获取累计结果，包括渲染帧、空闲帧、输入帧、命令帧、dirty rect 数、溢出次数、最近 sleep 时间和最近一帧的 `AppUiFrameBudget`。预算快照记录 spec/event/command/dirty rect 的使用量、固定容量和溢出标记，让外部 app 能在不引入动态指标表的情况下看到帧级容量压力。应用若要把这份信息画到自己的 surface，可以实现 `AppUiApp::observe_frame()` 或 `AppUiScheduledApp::observe_scheduled_frame()`，在 render 后保存 `AppUiFrameResult`，下一帧通过 `app_ui_budget_label()`、`app_ui_budget_pressure()` 或 `app_ui_frame_result_label()` 显示静态标签。这样 settings app 可以在无输入时进入 idle，动画 demo 仍可保持 active，而诊断层可以看到两类应用的帧行为。

AppUi RGB565 renderer 使用固定容量多 dirty rect。当前 `APP_UI_DIRTY_RECT_CAPACITY` 为 4：少量控件变化时逐 rect clip 渲染和提交；dirty 列表溢出、当前 frame spec 溢出或上一帧 spec 溢出时退化为单个 union/full rect。这条路径不引入 heap 分配，适合 MCU/低功耗 MPU 上的小屏局部刷新。

### 5.7 AppUi typed signals

外部 Wing surface 应用使用 `AppUiScheduledApp` 时，输入手势不会直接修改应用状态，而是先映射成命令队列：

```text
AppUiGesture -> AppUiSignal -> AppUiSchedule systems
```

当前 signal 是一个轻量结构：

```rust
pub struct AppUiSignalId(pub u16);

pub struct AppUiSignal {
    id: AppUiSignalId,
    value: i32,
}
```

设计约束：

- `AppUiSignalId` 和 `AppUiKey` 分离，避免把控件 ID 和业务命令 ID 混用。
- payload 固定为 `i32`，覆盖 stepper delta、segment index、list selection、boolean toggle 等小屏 UI 常见命令。
- command queue 使用固定容量 `FixedList`，无动态事件表、无字符串分发、无宏生成。
- 系统通过 `signal.is(CMD_*)` 和 `signal.value()` 显式处理命令，保持可读和可预测。

这层不是完整反射式事件系统，而是 MCU 友好的声明式命令层：应用先声明 view，再把 gesture 映射为少量 typed signal，最后由 schedule 中的系统函数更新状态。

### 5.8 AppUi staged schedule

`AppUiSchedule<S>` 仍然保留为最小单阶段 schedule。对于需要区分输入命令、帧准备和帧 tick 的应用，当前实现提供 `AppUiStagedSchedule<Input, Tick, Prepare>`：

```rust
pub struct AppUiStagedSchedule<Input, Tick = (), Prepare = ()> {
    input: Input,
    tick: Tick,
    prepare: Prepare,
}
```

执行规则很简单：

- `Gesture` 和 `Signal` 进入 input stage。
- `Prepare` 进入 prepare stage，在应用 update 之后、声明 view 之前运行。
- `Tick` 进入 tick stage，在一帧完成后运行。
- 三个 stage 都是普通 `AppUiSystem`，可以是单个函数、tuple 系统组或另一个 schedule。
- `AppUiStagedSchedule` 自己也实现 `AppUiSystem`，因此能直接作为 `AppUiScheduledApp::Schedule` 使用。

这不是 Bevy 风格的依赖图调度器，而是面向小屏应用的静态阶段拆分。settings app 用 input stage 处理设置变化、tick stage 衰减 toast；demo app 用 input stage 处理交互、tick stage 处理轻量动画倒计时。prepare stage 用于布局前派生状态、动画预计算或 render-prep；默认是 `()`，不需要的应用没有额外代码。

外部应用的推荐骨架见 `docs/wing_app_ui_sdk.md`。应用侧优先从 `app_ui_sdk::prelude::*` 导入稳定 facade；NuttX builtin 入口优先调用 `run_app_ui_rgb565_from_argv()`，把 surface open、RGB565 runner 和 exit code 转换收敛到 SDK。`app_ui.rs` 继续承载当前 crate 内部实现，后续拆第三方 app crate 时只需要收敛 facade 的导出边界。

## 6. 声明式 UI 模型

Wing UI 的核心组件：

```rust
pub struct Node {
    parent: Option<Entity>,
    first_child: Option<Entity>,
    next_sibling: Option<Entity>,
}

pub struct Layout {
    x: i16,
    y: i16,
    w: u16,
    h: u16,
}

pub struct Transform {
    tx: f32,
    ty: f32,
    z: i16,
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
}

pub enum Visual {
    Rect(RectStyle),
    Image(ImageStyle),
    Text(TextStyle),
    Mesh(MeshStyle),
    Effect(EffectStyle),
}

pub struct HitBox {
    rect: Rect,
    enabled: bool,
}

pub struct Visibility {
    visible: bool,
    opacity: u8,
}
```

声明式含义：

- Home、通知面板、设置页、预览卡片都是实体。
- 实体通过组件描述“是什么”和“当前状态”。
- 布局系统根据组件和资源计算位置。
- 渲染构建系统根据组件生成 DrawCmd。
- 用户代码不直接调用 framebuffer 绘制。

示例：

```rust
let settings = world.spawn();
world.insert(settings, Node::root());
world.insert(settings, Layout::fullscreen());
world.insert(settings, Surface::settings());
world.insert(settings, Visibility::hidden());

let row = world.spawn();
world.insert(row, Node::child_of(settings));
world.insert(row, Layout::row(0));
world.insert(row, Visual::Rect(theme.panel_row));
world.insert(row, HitBox::enabled());
world.insert(row, SettingsAction::CycleTheme);
```

## 7. Shell 状态机

Shell 不再依靠“页面上还能不能响应手势”的临时判断，而是使用明确状态机。

```rust
pub enum ShellMode {
    Home,
    NotificationPanel { return_to: SurfaceRef },
    AppSwitcher { return_to: SurfaceRef },
    SystemApp { app: BuiltinAppId },
    ExternalApp { app: AppId },
}
```

手势规则放到状态机中：

| 当前状态 | 手势 | 行为 |
|---|---|---|
| Home | 下滑 | 进入通知页 |
| Home | 上滑 | 进入卡片预览 |
| NotificationPanel | 上滑 | 返回 return_to |
| NotificationPanel | 下滑 | 更新通知页拖动进度 |
| AppSwitcher | 下滑 | 返回 return_to |
| AppSwitcher | 上滑 | 更新预览页拖动进度 |
| SystemApp | 点击返回 | 回 Home |
| ExternalApp | 系统返回手势/按钮 | 回 Home 或 AppSwitcher |

这样可以避免之前的问题：进入设置页后主界面仍然响应，或者通知页与预览页同时抢占显示。

## 8. 设置与系统内置应用

设置属于系统内置应用，建议第一阶段作为 Wing 进程内的 builtin surface。

原因：

- 设置直接修改 `ThemeState`、`PreviewEffect`、亮度、无线开关等 shell 资源。
- 进程内切换没有 IPC 成本。
- 可以和通知页、卡片预览共享同一套 UI 风格和返回按钮。
- 对 MCU/低功耗平台更省内存。

外部应用则应该是独立 NuttX task。

应用分两类：

```text
Builtin app:
  - 设置
  - 系统信息
  - 主题选择
  - 简单文件/媒体入口
  - 与 Wing 同线程、同 ECS 世界

External app:
  - 第三方软件
  - 独立 NuttX task
  - 可通过 nsh 命令直接运行
  - 由 Wing launcher 调用 task_spawn 或等价平台接口启动
```

第一阶段应用联动接口：

```rust
pub struct AppManifest {
    id: AppId,
    name: &'static str,
    icon: ImageId,
    launch: LaunchKind,
}

pub enum LaunchKind {
    Builtin(BuiltinAppId),
    PlatformTask(PlatformTask),
}
```

后续 external app surface 可以通过共享 framebuffer、消息队列或 stream texture 接入。

## 9. 渲染架构

### 9.1 DrawList 取代 RenderWorld

新的渲染数据是短生命周期：

```rust
pub struct DrawList {
    commands: FixedList<DrawCmd, DRAW_CMD_CAPACITY>,
}
```

`build_draw_list_system` 从 ECS 组件读取 UI 状态：

```text
World components/resources
  -> layout/visibility/theme
  -> stable z ordered scan
  -> DrawCmd
  -> Renderer
```

当前实现不会在 `RenderBuild` 阶段为排序临时创建 `Vec<(z, entity)>`。可见节点按 `(z, visual_dense_index)` 做稳定有序扫描，同 z 时保持组件存储中的原始顺序。这样小屏 UI 的实体数量较少时，可以用一点 O(n^2) 扫描换掉帧内堆分配和排序缓冲，行为更适合 MCU/低功耗 MPU。

`UiFrame` 和 `DrawList` 都使用固定容量 `FixedList<T, N>` 作为帧内清单。当前容量分别是 `UI_SPEC_CAPACITY = 128` 和 `DRAW_CMD_CAPACITY = 128`。这不是完整通用 Vec，而是一个只面向 `Copy` 小对象的帧缓冲：`clear()` 只重置长度，`push()` 顺序写入，读取时暴露已初始化 slice。容量不足会设置 overflow 标记，并由 `RuntimeDiagnostics` 在帧循环中统一采样；默认路径不再因为普通页面刷新触发堆扩容。

不再执行：

```text
MainWorld -> SyncToRenderWorld -> RenderWorld ECS -> ExtractedUI -> Queue
```

### 9.2 DrawCmd

```rust
pub enum DrawCmd {
    Clear { color: Color },
    FillRect { rect: RectI, color: Color },
    FillRoundRect { rect: RectI, radius: u8, color: Color },
    Image { rect: RectI, image: ImageId, tint: Color },
    Text { pos: PointI, text: TextId, style: TextStyle },
    Line { from: PointI, to: PointI, color: Color },
    Mesh2d { mesh: MeshId, transform: Transform2d, material: MaterialId },
    Effect { rect: RectI, effect: EffectId, params: EffectParams },
    PushClip { rect: RectI },
    PopClip,
}
```

Shell 默认只生成 2D draw commands。足球当前仍走基础 2D 圆形拼接；立方体这类效果走 `Visual::Effect -> DrawCmd::Effect`，以后需要真实 3D 时再由 GPU/GLES 后端扩展成 `Mesh3d` 子路径。

### 9.3 Renderer trait

```rust
pub enum RendererKind {
    Software,
    Gpu2d,
    Gles,
    Custom,
}

pub struct RendererCapabilities {
    cpu_framebuffer: bool,
    dirty_rects: bool,
    alpha_blend: bool,
    rounded_rect: bool,
    circle: bool,
    image_rgb565: bool,
    image_a8: bool,
    background_layer: bool,
    background_plane: bool,
    mesh2d: bool,
    mesh3d: bool,
    effects: bool,
}

pub trait RendererBackend {
    fn kind(&self) -> RendererKind;
    fn capabilities(&self) -> RendererCapabilities;
    fn render_plan(&self, list: &DrawList) -> RenderPlan;
    fn draw(
        &mut self,
        list: &DrawList,
        plan: RenderPlan,
        textures: &TextureStore,
        fonts: &FontStore,
        dirty: DirtyRegion,
    );
    fn framebuffer(&self) -> &[u32];
}
```

`WingRuntime<P, R>` 现在同时泛型于平台和渲染后端，其中 `R: RendererBackend`，默认后端是 `SoftwareRenderer`。这不是 `Box<dyn Renderer>`，也没有虚函数表持有权问题；系统函数在编译期绑定到具体 runtime 类型，仍然符合 NuttX + Rust 的轻量目标。硬件后端只需要消费同一份 `DrawList`、`RenderPlan`、`TextureStore`、`FontStore` 和 `DirtyRegion`，不反向理解 Shell、应用或 UI 状态。`DrawList::route_runs()` 会用无分配迭代器按 route 和当前 clip 产生有序 run；`RenderPlan` 会记录 run summary，便于发现 route 切换过碎的帧。`SoftwareRenderer::draw_routes()` 可以按 `DrawRouteMask` 执行 supported 或 software fallback 子集，并重放 `SetClip` 状态。以后 GPU2D/GLES 后端可以按 run 顺序执行 native draw task，再把 fallback route 交给同一套软件执行器，避免打乱透明层、文字和 surface 的叠放关系。

软件后端现在还有一个很窄的背景层快路径：当 DrawList 前缀是 Shell 标准的 `ClearGradient + fullscreen RGB565 wallpaper` 时，renderer 会在 dirty clip 内一次性合成渐变和壁纸，并跳过这两个前缀命令。运行时 PNG 壁纸通常已经缩放到当前 framebuffer 尺寸，此时背景采样走直接 RGB565 读取；fallback 资源尺寸不匹配时仍保留 bilinear 采样。这个路径不是完整 layer cache，不额外保存一份全屏 framebuffer，只是把稳定背景层抽成可识别的渲染前缀，为后续可选 tile cache 或硬件背景 plane 留出边界。`RenderPlan.layers` 会记录背景层种类、`RenderBackgroundRoute` 和被快路径吞掉的前缀命令数，System 页用 `LAYER` 行显示 `BG FAST/BG PLANE/BG GRAD/BG MISS/BG CAP/NONE`，让后续硬件后端可以复用同一份诊断语义。

`BackgroundPlaneRenderer` 是第一版可替换后端骨架：它声明 `RendererKind::Gpu2d` 和 `background_plane` 能力，但仍使用 `SoftwareRenderer` 输出 framebuffer，目的是在 NuttX sim 上验证 `RenderBackgroundRoute::NativePlane`、System 诊断和 runtime 泛型后端切换，而不绑定真实 2D/GLES 驱动。它通过静态泛型的 `BackgroundPlaneOps` 提交 `BackgroundPlaneSubmission`，默认 `SimBackgroundPlaneOps` 只校验 RGB565 背景纹理并记录提交，不进入动态派发，也不要求平台驱动存在。默认 `wing_rust` 仍走软件后端；运行 `wing_rust --renderer=plane` 或 `wing_rust --renderer=gpu2d-plane` 可以进入这个背景 plane profile。

`RendererCapabilities` 是小型 `Copy` 能力表，不是插件注册表。默认 `SoftwareRenderer` 声明支持 framebuffer、dirty rect、alpha blend、基础 2D shape、RGB565 图片和 A8 mask，但不声明 mesh/effect 能力。以后 GPU2D、GLES 或 OpenCL ES 后端可以打开对应能力；Shell 和 Preview 只按能力选择效果或降级，不直接依赖具体后端类型。

### 9.4 Software renderer

软件后端重点优化：

- RGB565 背景直接 blit。
- A8 图标 tint blit。
- 预乘 alpha 格式减少混合计算。
- dirty rect 只重绘变化区域。
- 小屏设备可使用 tile dirty，例如 16x16 或 32x32。
- 避免每帧 clone 大纹理。
- 避免 per-pixel 浮点采样。
- 默认 nearest，只有缩放预览图时才使用 linear。
- 文本使用 glyph atlas，不在帧内动态 rasterize。

### 9.5 GPU/2D 加速扩展

同一份 DrawList 可以映射到不同后端：

| 后端 | 适用硬件 | 能力 |
|---|---|---|
| Software | 纯 framebuffer | 所有基础 UI |
| Gpu2d | DMA2D/PXP/VG-Lite | fill/blit/alpha/scale/rotate |
| Gles | OpenGL ES 3.0 | 2.5D/3D effect、shader、复杂变换 |

设计要求：

- Shell 不知道具体后端。
- Shell 只声明 Visual/Effect。
- Renderer 自己决定软件或硬件路径。
- 不支持的 effect 必须有软件降级或静态 fallback。

## 10. 资源系统

旧 FHRE 的 AssetPlugin/RenderAssetPlugin 对 Wing 偏重。新的资源系统应该更显式。

```rust
pub struct TextureStore {
    textures: FixedList<Texture, TEXTURE_STORE_CAPACITY>,
}

pub struct Texture {
    id: ImageId,
    width: u16,
    height: u16,
    format: PixelFormat,
    data: &'static [u8],
}
```

当前轻量实现中 `TextureStore` 默认 `TEXTURE_STORE_CAPACITY = 32`，只保存静态切片数据，适合 ROMFS、`include_bytes!` 或后续 flash 分区映射出来的只读资源。动态 `Owned(Vec<u8>)`、外部 GPU texture handle、异步 asset loading 不进入第一版核心路径；需要时应作为可选后端扩展，而不是默认资源模型。

资源格式优先级：

- 背景：RGB565
- 图标/遮罩：A8
- 需要透明彩色图：RGBA4444 或 premultiplied RGBA8888
- 字体：A8 glyph atlas

主题资源：

```rust
pub struct Theme {
    id: ThemeId,
    wallpaper: ImageId,
    palette: Palette,
    icon_set: IconSetId,
}
```

资源加载原则：

- 内置资源优先 `include_bytes!`。
- NuttX 文件系统资源后续再接。
- 启动阶段显式注册。
- 帧内不解析 PNG。
- 帧内不创建大纹理。

## 11. 卡片预览效果

卡片预览不应该硬编码为一种渲染路径。它应该是 shell 状态 + effect component。

```rust
pub enum PreviewEffect {
    Cards,
    Soccer,
    Cube,
}
```

每个打开的 surface 对应一个 preview face/card：

```rust
pub struct PreviewItem {
    surface: SurfaceRef,
    title: TextId,
    icon: ImageId,
    snapshot: Option<ImageId>,
}
```

效果层：

```rust
pub struct PreviewEffectNode {
    effect: PreviewEffect,
    progress: u8,
    selected: Option<SurfaceRef>,
}
```

行为：

- `Cards`：普通卡片栈，基础默认效果。
- `Soccer`：每个 surface 是足球上的一个面，点击 face 进入对应 surface。
- `Cube`：surface 分布到立方体面或环形卡片。

注意：效果只改变 preview 的表现，不改变 shell surface 的生命周期。

当前第一版代码已经把固定 demo preview 改成了 `AppRegistry` 驱动的 preview item：每个 manifest 会生成一个可点击的 card/face，点击后进入统一 `ButtonAction::LaunchApp(AppId)`，再由 `AppManager` 产生 builtin surface 或外部 command launch request。`Cards`、`Soccer` 和 capability gated `Cube` 复用同一批 item，只改变布局和视觉形态。

Preview effect 选择必须经过 renderer capability：

```text
ShellState.preview_effect
  -> resolve_preview_effect(RendererCapabilities)
  -> compose Cards / Soccer / Cube
```

软件后端当前声明支持基础 2D、圆形、alpha blend 和轻量 effect，因此可以显示 `Cards`、2D `Soccer` 和软件 2.5D `Cube` fallback。`compose_cube_preview()` 生成 `Visual::Effect { PreviewCube }` 和透明 face hitbox，`build_draw_list_system()` 转成 `DrawCmd::Effect`；`SoftwareRenderer` 用固定点/整数扫描线填充几个 cube face、线框和阴影，不引入矩阵库、mesh buffer 或帧内分配。真正的 mesh3d cube 仍留给后续 GPU2D/GLES/Custom 后端通过同一个 `EffectKind::PreviewCube` 替换实现。

## 12. 输入和手势

平台输入只做硬件事件读取：

```text
/dev/input0, /dev/kbd
  -> RawInputEvent
  -> InputState
  -> PointerEvent
  -> GestureEvent
  -> ShellState
```

Shell 手势必须由状态机决定是否生效。

输入组件：

```rust
pub struct HitBox;
pub struct Button;
pub struct GestureRegion;
pub struct Focusable;
```

资源：

```rust
pub struct InputState {
    pointer: PointerState,
    keys: KeyState,
}

pub struct GestureState {
    active: Option<Gesture>,
    start: PointI,
    current: PointI,
    velocity: Vec2,
}
```

命中测试：

- 按 layer/z 从前到后。
- 不可见实体不参与。
- 当前 `ShellMode` 禁止响应的层不参与。
- overlay 优先级由 `ShellMode` 决定，不由绘制顺序临时碰撞。

## 13. 对外应用模型

Wing 作为 shell，需要能从图标启动独立 NuttX 应用。

推荐分层：

```text
Launcher icon
  -> AppManager::launch(app_id)
  -> AppLaunchRequest::PlatformTask
  -> Platform::launch_task(...)
  -> AppHandle
  -> Surface placeholder / external surface
```

第一阶段：

- 图标点击可以通过 `PlatformTask { command, stdio, priority, stack_size, surface }` 启动 NuttX builtin command。
- Platform 暴露 task capability，Wing 在进入平台启动前可以明确判断哪些 policy 当前可用。
- Wing 记录 app 处于 running/focused/background。
- app 画面先用 placeholder surface 表示。

第二阶段：

- 外部 app 通过 `SurfaceRequest` 申请 Wing-managed surface，Wing 返回 `SurfaceDescriptor { SurfaceId, stride, buffer_count }`。
- surface transport 第一类是共享 framebuffer + 轻量消息队列，后续可扩展为 stream texture。
- Wing 把它作为 `ExternalSurfaceTexture` 显示。
- 输入由 Wing 转发给 focused app。

第三阶段：

- 支持 app lifecycle：pause/resume/close。
- 支持后台任务状态。
- 支持通知消息。

## 14. 迁移路线

### 阶段 0：冻结旧 FHRE 设计

- `apps/fhre` 作为历史参考，不再新增功能。
- `docs/ARCHITECTURE.md` 和 `docs/wing_fhre_integration.md` 标记为旧架构。
- 新实现以本文为准。

### 阶段 1：Wing 内建立统一 runtime 骨架

在 `apps/wing/rust/src` 内新增：

- `core`
- `render`
- `platform`
- `shell`
- `ui`

先保留旧代码能构建，再逐步迁移。

### 阶段 2：移入 FHRE 中必要模块

从 FHRE 迁移并瘦身：

- math: Color, Rect, Vec2, Vec3, Transform
- pipeline: Texture, SoftwareBackend 中有效绘制函数
- window: Window/InputEvent trait
- picking 中有价值的 hit-test 逻辑

不要迁移：

- App/PluginGroup
- MainWorld/RenderWorld 双世界
- ExtractComponentPlugin
- RenderAssetPlugin
- Bevy-style SystemParam
- 宏式系统注册

### 阶段 3：替换系统调度

把：

```text
App + Plugin + declare_system!
```

替换为：

```text
WingRuntime + Schedule + fn(&mut WingRuntime)
```

Wing shell 系统改为显式访问资源和 query。

### 阶段 4：替换 RenderWorld/extractor

把：

```text
extract_wing_shell -> ExtractedShellImage/Text/UI -> queue_wing_primitives
```

替换为：

```text
build_draw_list_system
```

渲染数据只在一帧内存在。

### 阶段 5：替换 ECS 存储

把 `BTreeMap<TypeId, BTreeMap<EntityId, Box<dyn Any>>>` 替换为 sparse-set storage。

先支持 Wing 内置组件，再开放给应用。

### 阶段 6：资源和渲染优化

- 建立 TextureStore。
- 建立 Icon/Glyph atlas。
- 背景 RGB565 直通。
- 图标 A8 tint。
- dirty rect。
- 后续增加 GPU2D/GLES 后端。

### 阶段 7：应用联动

- `AppManifest`
- `AppRegistry`
- `AppManager`
- builtin settings surface
- external task launch
- external surface placeholder

## 15. 兼容策略

迁移期间不建议一次性删除 FHRE。推荐方式：

```text
旧 FHRE/Wing 可构建
  -> Wing 新 runtime 骨架并存
  -> 单个模块替换
  -> wing_build.sh 始终通过
  -> 新 runtime 成为默认入口
  -> 删除旧依赖
```

每个阶段都必须保证：

```bash
cd /home/uan-gpd/codes/FeatherOS/nuttx
./wing_build.sh
```

构建通过。

## 16. 第一版落地目标

第一版 unified Wing 不追求全部重写完成，而是要先建立正确骨架：

- 一个 `WingRuntime`
- 一个 ECS World
- 一个显式 schedule
- 一个 DrawList
- 一个默认 `SoftwareRenderer` 后端
- 一个平台 Window
- Home + Settings + Notification + AppSwitcher
- PreviewEffect 支持 Cards 和 Soccer，并接入 capability gated Cube effect 入口
- 设置页可以切换主题和预览效果
- Launcher 图标可以启动 builtin app，并预留 external app launch

当这个骨架跑通后，再逐步把现有 UI 效果、主题资源和足球/立方体 demo 迁移进来。

## 17. 最终判断标准

新的 Wing 架构是否正确，不看它像不像 Bevy，而看下面这些问题：

- 能否在 NuttX 上稳定构建和运行？
- 是否只有一个主 ECS 世界？
- 是否没有宏式系统注册？
- 是否没有长期 RenderWorld ECS？
- 帧内是否避免大分配、大 clone、大 downcast 热路径？
- 软件渲染是否有 RGB565/A8 快路径？
- Shell 状态机是否明确？
- 手势是否由当前 ShellMode 决定？
- 设置是否是低成本 builtin surface？
- 外部应用是否可以独立 task 运行并被 Wing launcher 管理？
- 后续是否能接 2D/GLES 后端而不改 shell 逻辑？

如果这些成立，Wing 才是适合 FeatherOS 的小屏 UI 系统，而不是一套被 Bevy 形状拖重的通用引擎。

## 18. 第二轮重构后的实现基线

本节描述当前重构后的新基线，优先级高于前文中仍偏“目标形态”的描述。

### 18.1 对参考项目的取舍

`/home/uan-gpd/codes/bevy` 值得参考的是：

- `App` 明确拥有 runner 和 update 生命周期。
- Schedule 把系统放到明确阶段里。
- ECS 状态和系统行为分离。
- 渲染由数据驱动，而不是业务代码直接画 framebuffer。

Wing 不复制 Bevy 的内容：

- 不要 `derive(Component)`、`derive(Resource)`、`IntoSystem` 等宏。
- 不要复杂 `SystemParam` 自动注入。
- 不要插件组和默认插件树。
- 不要长期 `RenderWorld`。
- 不要为了通用游戏引擎能力牺牲 MCU/低功耗平台体积。

`/home/uan-gpd/codes/hecs` 值得参考的是：

- `Entity { index, generation }` 防止旧 handle 误命中。
- 组件数据要紧凑连续。
- ECS 可以只是库，不必变成完整框架。
- 系统可以是普通函数和普通循环。

Wing 第一版没有采用 hecs 的 archetype：

- 小屏 UI 实体数量有限。
- 组件组合相对稳定。
- sparse-set 更容易控制代码体积和 no_std 行为。
- 多组件 query 可以先手写热点路径，不急着泛化。

`planck_ecs` 的参考点是低 unsafe、dispatcher 与 storage 分离；但 Wing 目前继续使用更小的显式 `Schedule<WingRuntime>`。本地 `freecs` 不是 Rust ECS 库，不作为 Wing ECS 主参考。

### 18.2 当前运行时边界

当前代码基线：

```text
wing_rust_main
  -> NuttXPlatform::new()
  -> WingRuntime::new(platform)
  -> install_shell()
  -> runtime.run()
```

`WingRuntime` 拥有：

- `World`：唯一主 ECS 世界。
- `ShellState`：当前 shell 状态机。
- `InputState`：平台输入归一化结果。
- `UiFrame`：本帧声明式 UI 清单。
- `DrawList`：本帧渲染命令。
- `RendererBackend`：当前渲染后端，默认是 `SoftwareRenderer`。
- `AppRegistry/AppManager`：应用注册与启动请求状态。
- `Schedule`：显式阶段系统。
- `Platform`：framebuffer、输入、timer、后续 task launch。

这里暂时没有 `Box<dyn Any>` 资源表。第一版采用具体字段是有意的：资源数量很少，具体字段更省、更直观、更适合 NuttX。后续如果需要应用扩展资源，再引入一个小型 `ResourceSlots`，而不是一开始就恢复 Bevy 式动态资源系统。

### 18.3 声明式 UI 的当前定义

当前声明式 UI 不再是手写 `spawn_*` 一次性创建静态页面，而是：

```text
ShellState/InputState
  -> compose_shell_ui_system()
  -> UiFrame { UiSpec... }
  -> World::apply_ui_frame()
  -> ECS components
  -> build_draw_list_system()
  -> DrawList
  -> Renderer
```

`UiFrame` 是一帧的 UI 声明清单。每个 `UiSpec` 带稳定 `UiKey`：

```rust
pub struct UiSpec {
    pub key: UiKey,
    pub layer: Layer,
    pub rect: Rect,
    pub z: i16,
    pub alpha: u8,
    pub visual: Visual,
    pub action: Option<ButtonAction>,
}
```

`World::apply_ui_frame()` 根据 key 做三件事：

- key 已存在：更新 `Layout/Visual/Visibility/Layer/Button` 等组件。
- key 不存在：创建实体并插入组件。
- 本帧没有声明的旧 UI 实体：通过 `UiMark` 清理。

旧 UI 实体清理不创建临时 stale 列表。`World` 每次扫描出一个 revision 不匹配的实体，先把它的旧 visual bounds 合入 dirty，再立即 `despawn`，直到没有过期实体。这样页面切换仍然保持简单 diff，但不会在 Layout 阶段为清理旧节点分配 `Vec<Entity>`。

这样页面切换后，旧主界面不会继续留在点击/绘制路径里；通知页、预览页、设置页由当前 `ShellMode` 唯一决定。它保留 ECS 的数据驱动结构，但没有虚拟 DOM、宏、反射和复杂 diff。

### 18.4 应用模型的当前定义

系统内置应用和外部应用分开：

```rust
pub enum LaunchKind {
    Builtin(BuiltinAppId),
    PlatformTask(PlatformTask),
}

pub struct CommandSpec {
    program: CommandToken,
    args: &'static [CommandToken],
}

pub struct PlatformTask {
    command: CommandSpec,
    stdio: TaskStdio,
    priority: TaskPriority,
    stack_size: TaskStackSize,
    surface: TaskSurface,
}

pub struct CommandToken {
    text: &'static str,
    cstr: &'static [u8],
}

pub enum TaskStdio {
    Inherit,
    Null,
}

pub enum TaskPriority {
    BuiltinDefault,
    Value(i32),
}

pub enum TaskStackSize {
    BuiltinDefault,
    Bytes(u32),
}

pub enum TaskSurface {
    Detached,
    WingManaged(SurfaceRequest),
}

pub struct SurfaceRequest {
    owner: AppId,
    size: Size,
    format: SurfacePixelFormat,
    buffering: SurfaceBuffering,
    transport: SurfaceTransport,
    input: SurfaceInputPolicy,
}

pub struct PlatformTaskCapabilities {
    max_args: u8,
    inherit_stdio: bool,
    null_stdio: bool,
    builtin_default_priority: bool,
    custom_priority: bool,
    builtin_default_stack: bool,
    custom_stack: bool,
    detached_surface: bool,
    wing_managed_surface: bool,
}

pub enum TaskLaunchError {
    InvalidProgram,
    InvalidArgument,
    TooManyArguments,
    UnsupportedStdio,
    UnsupportedPriority,
    UnsupportedStackSize,
    UnsupportedSurface,
    SpawnFailed(i32),
}
```

当前 `AppRegistry` 内置：

- Settings：`wing_settings`，Wing-managed surface，由 NuttX task runner 启动为独立 task。
- System：builtin system info surface。
- Terminal：预留外部 command。

当前实现中 `AppRegistry` 不是无界 `Vec`，而是固定容量 manifest 表，默认 `APP_REGISTRY_CAPACITY = 32`。`register()` 遇到相同 `AppId` 会原地覆盖，遇到新应用会追加到固定表；容量满时只记录 overflow，不触发堆扩容。这样系统内置应用和少量外部应用可以保持确定的内存上限，后续如果设备形态需要更多应用，应优先调整容量或切到静态分区表，而不是让 launcher 在运行期无限增长。

当前 `AppManager` 维护 focused app、last launch request 和 pending launch request，但不直接启动 NuttX task。`WingRuntime` 在 `Update` 后统一消费请求并调用平台层。这样 shell 逻辑保持稳定，不把 NuttX 线程管理、surface 管理和页面 UI 混到一起。

```text
Launcher/ButtonAction
  -> AppManager::launch(app_id)
  -> pending AppLaunchRequest
  -> WingRuntime::handle_app_launch_requests()
  -> Platform::launch_task(PlatformTask, Option<SurfaceDescriptor>)
  -> NuttX task runner shim
```

当前实现已经支持固定容量 command args：`COMMAND_ARG_CAPACITY = 4`。`CommandToken` 同时保存 UI/调试可读的 `text` 和 NuttX C ABI 可直接使用的静态 `cstr`，平台层只把这些静态 token 组装成固定数组 `argv`，不做字符串拼接、不分配堆内存、不走 shell 解析。NuttX 平台实现通过 `wing_task_spawn()` C shim 启动，这会让目标 builtin 作为独立 task 运行。

第一版 `PlatformTask` 已显式建模 stdio、priority、stack size 和 surface policy，并加入 `PlatformTaskCapabilities::validate()` 与 typed `TaskLaunchError`。NuttX 实现已经有一个很薄的 C ABI shim：Rust 侧传入稳定的 `wing_task_launch_config`，C 侧用 NuttX 的 `builtin_isavail()`、`builtin_for_index()`、`posix_spawn_file_actions_*()`、`posix_spawnattr_*()` 和 `task_spawn()` 完成实际启动。这样 Rust 不需要复制 `posix_spawnattr_t` 等平台 ABI。

当前 NuttX 后端声明的是 `NUTTX_TASK_RUNNER` 能力，支持 `TaskStdio::Inherit/Null`、builtin default 或自定义 priority、builtin default 或自定义 stack size、detached surface。dirty queue 可用时，后端还会打开 `PlatformTaskCapabilities.wing_managed_surface`，允许 `TaskSurface::WingManaged(_)` 走 Wing-owned surface。

`WingRuntime` 现在暴露 `task_capabilities()`，系统页会显示当前 task runner 级别。默认 NuttX 后端显示为 `CUSTOM`，表示可以启动 builtin task，并且可以覆盖 stdio、priority 和 stack size；当 `/wing_surface_dirty` 可用时，还可以接入 Wing-managed surface。

#### 18.4.1 Wing-managed surface 合约

外部应用的画面不能直接变成 renderer 私有对象。Wing 需要先有一个稳定、可验证、固定容量的 surface 合约：

```rust
pub enum SurfacePixelFormat {
    Rgb565,
    Argb8888,
}

pub enum SurfaceBuffering {
    Single,
    Double,
}

pub enum SurfaceTransport {
    SharedMemory,
    StreamTexture,
}

pub enum SurfaceInputPolicy {
    None,
    Pointer,
    PointerKeyboard,
}

pub struct SurfaceDescriptor {
    id: SurfaceId,
    request: SurfaceRequest,
    stride_bytes: u16,
    buffer_count: u8,
    transport: SurfaceTransportDescriptor,
}

pub struct SurfaceTransportDescriptor {
    token: u32,
    frame: SurfaceHandle,
    dirty: SurfaceHandle,
    input: SurfaceHandle,
}

pub struct SurfaceCapabilities {
    max_surfaces: u8,
    max_width: u16,
    max_height: u16,
    rgb565: bool,
    argb8888: bool,
    single_buffer: bool,
    double_buffer: bool,
    shared_memory: bool,
    stream_texture: bool,
    pointer_input: bool,
    keyboard_input: bool,
}
```

当前代码已经加入 `SurfaceTable`，默认容量是 `SURFACE_CAPACITY = 8`。`SurfaceId` 由 slot + generation 组成，避免外部应用关闭后旧 handle 继续误用。`SurfaceTable::create()` 会用 `SurfaceCapabilities::validate()` 检查尺寸、像素格式、buffering、transport 和输入策略，并返回包含 stride、buffer count 和 transport handle 的 `SurfaceDescriptor`。dirty 信息先以 `Option<Rect>` 记录，后续可升级成 tile dirty，但不提前把复杂 tile 系统压进普通 UI。

`SurfaceTransportDescriptor` 当前生成三个轻量 handle：

- `frame`：后续用于查找 surface framebuffer 或 stream texture。
- `dirty`：后续用于外部应用提交 dirty rect 或帧序号。
- `input`：后续用于 Wing 向 focused app 转发 pointer/key 事件。

`token` 是一个 31-bit 正数，用于把启动参数和 SurfaceTable 中的 generation 绑定起来，避免旧进程拿旧 surface id 写回新 surface。

当前 `SurfaceTable` 已经是第一版 registry，而不仅是 descriptor 列表：

```text
SurfaceHandle + token
  -> SurfaceTable::resolve_handle()
  -> SurfaceEndpoint { id, kind, descriptor }

dirty handle + token + Rect
  -> SurfaceTable::submit_dirty()
  -> validate token and handle kind
  -> clamp rect to surface bounds
  -> mark surface dirty
```

`WingRuntime::submit_surface_dirty(handle, token, rect)` 是后续 IPC 轮询系统的入口。它会先让 `SurfaceTable` 校验 handle/token 并记录 surface-local dirty，再保守地 `mark_all_dirty()`。等外部 surface 有稳定的屏幕布局后，可以把这一步优化成 surface bounds 到 screen rect 的精确 dirty，而不改变外部应用协议。

`Platform` 现在同时暴露：

```rust
fn poll_surface_event(&mut self) -> Option<SurfaceEvent>;
fn task_capabilities(&self) -> PlatformTaskCapabilities;
fn surface_capabilities(&self) -> SurfaceCapabilities;
```

NuttX 后端当前已经打开第一条 surface IPC 通道：`task_runner.c` 创建非阻塞 POSIX message queue `/wing_surface_dirty`，消息体是固定 16 字节的 `handle/token/x/y/w/h`。`NuttXPlatform::poll_surface_event()` 每帧最多被 Runtime drain 8 条消息，转成 `SurfaceEvent::Dirty` 后进入 `WingRuntime::submit_surface_dirty()`。

同一层还建立了最小软件 framebuffer registry。`wing_task_spawn()` 在启动 `TaskSurface::WingManaged` 时，根据 descriptor 中的 width/height/stride/buffers 为 `frame` handle 分配固定上限的软件 buffer，并用 `frame handle + token` 注册。Wing 重启时会 `wing_surface_frame_reset()` 清掉旧 slot；启动失败时会撤销本次注册。

外部应用侧的最小 C ABI 放在 `apps/wing/rust/include/wing/wing_surface.h`：

```c
struct wing_surface_frame_info {
    uint32_t handle;
    uint32_t token;
    void *pixels;
    size_t bytes;
    int width;
    int height;
    int stride;
    int format;
    int buffers;
};

int wing_surface_frame_resolve(uint32_t handle, uint32_t token,
                               struct wing_surface_frame_info *info);
int wing_surface_frame_unregister(uint32_t handle, uint32_t token);
int wing_surface_dirty_sender_open(void);
int wing_surface_dirty_send(int queue, uint32_t handle, uint32_t token,
                            int16_t x, int16_t y, uint16_t w, uint16_t h);
int wing_surface_dirty_close(int queue);
```

应用启动后解析 `--wing-surface=...`，保存其中的 `frame/dirty handle` 和 `token`。初始化阶段调用 `wing_surface_frame_resolve()` 得到可写像素指针；绘制完成后调用 `wing_surface_dirty_send()` 提交 dirty。队列满时返回负 errno，应用可以丢弃本次 dirty，因为下一帧或下一次全量 dirty 仍然能恢复画面。

这条 C ABI 是平台传输层，不是 Rust 声明式 UI 的一部分。UI 和 Shell 不接触裸指针；Rust 侧应用逻辑仍然只看 descriptor、handle、token、dirty event 和 draw command。NuttX 后端会在 dirty queue 可用时报告 `SurfaceCapabilities::SOFTWARE_SHARED_RGB565`，并打开 `PlatformTaskCapabilities.wing_managed_surface`；如果队列不可用，则自动降级为 `SurfaceCapabilities::NONE`。

Renderer 合成外部应用 surface 的路径：

```text
focused SurfaceId
  -> SurfaceTable::get(id)
  -> descriptor.transport.frame + token
  -> Platform::resolve_surface_frame()
  -> SurfaceFrame { pixels, bytes, width, height, stride, format }
  -> DrawCmd::Surface { rect, frame, alpha }
  -> SoftwareRenderer::draw_surface()
```

这个边界使裸指针只存在于平台解析和 renderer blit 的低层，不进入 ESC 状态树。软件后端当前支持 RGB565 和 ARGB8888 surface blit；RGB565 是默认路径。后续 GPU2D/GLES 后端可以用同一个 `DrawCmd::Surface` 转成 DMA blit、texture draw 或 stream texture sampling。

启动路径已经为 surface descriptor 留出位置：

```text
AppLaunchRequest::PlatformTask(task)
  -> Runtime validates PlatformTaskCapabilities
  -> if TaskSurface::WingManaged(request):
       SurfaceTable::create(surface_capabilities, request)
       -> SurfaceDescriptor { SurfaceId, stride, buffer_count }
  -> Platform::launch_task(task, Option<SurfaceDescriptor>)
  -> NuttX wing_task_spawn()
  -> append "--wing-surface=id:w:h:stride:format:buffers:transport:input:token:frame:dirty:input_handle"
```

这里的 `--wing-surface=...` 只是启动期的轻量 descriptor 传递，不携带裸指针。外部应用必须通过 `wing_surface_frame_resolve(frame, token, &info)` 换取本平台的实际 transport。当前 transport 是软件共享 buffer；后续 GPU2D/GLES 后端可以保持同一个 descriptor，把 `frame` handle 映射为 DMA buffer、texture id 或 stream texture slot。

第一版端到端验证应用是 `apps/wing/surface_demo`。它是一个独立 NuttX builtin task，程序名为 `wing_surface_demo`，用于验证 C ABI 和 NSH 直接启动路径。这个 demo 做四件事：

```text
parse "--wing-surface=..."
  -> wing_surface_frame_resolve(frame, token)
  -> draw RGB565 pixels into Wing-owned buffer
  -> wing_surface_dirty_send(dirty, token, full_rect)
```

它不是 UI 框架的一部分，而是外部应用该如何和 Wing 对接的最小样例。Rust 侧已经在 `apps/wing/rust/src/app_sdk.rs` 建立第一版应用 SDK，把这套 C ABI 包成一个很小的 `WingSurface` 类型；随后 `apps/wing/rust/src/rust_surface_demo.rs` 用这个 SDK 实现了 Rust 版 managed-surface demo，程序名为 `wing_surface_rust_demo`，Wing 注册表中的 `Surface` 图标默认启动这个 Rust demo。

这个边界必须保持：

- UI 只声明图标和按钮。
- Shell 只产生 `ButtonAction`。
- AppManager 只做应用状态与启动请求。
- Platform 负责 NuttX 任务启动细节和平台 surface 能力声明。
- SurfaceTable 负责 Wing-managed surface 生命周期、dirty 和 handle generation。
- Renderer 不知道应用如何启动，只知道如何合成一个已解析的 `SurfaceFrame`。

#### 18.4.2 Managed app 生命周期

Wing-managed task 不是普通 detached command。它占用两个固定容量资源：

```text
AppManager.active_task
SurfaceTable slot
NuttX frame registry slot
```

因此进入和退出必须成对出现。当前路径是：

```text
LaunchApp(Surface)
  -> AppManager::launch()
  -> Runtime::prepare_task_surface()
  -> SurfaceTable::create()
  -> Platform::launch_task()
  -> AppManager::complete_launch(pid, SurfaceDescriptor)
  -> ShellMode::ExternalApp

Back/Home/launch another app
  -> Runtime::close_focused_app()
  -> Platform::close_task(pid, SurfaceDescriptor)
  -> NuttX wing_task_close(pid, frame, token)
  -> task_delete(pid)
  -> wing_surface_frame_unregister(frame, token)
  -> SurfaceTable::destroy(id)

app exits by itself
  -> Platform::poll_task_exit()
  -> NuttX waitpid(-1, WNOHANG)
  -> AppManager::handle_task_exit(pid)
  -> Runtime releases frame registry and SurfaceTable slot
  -> Shell returns to Home if it was showing ExternalApp
```

`AppManager` 只保存一条 `ActiveAppTask`，第一阶段不做多窗口，也不让多个 managed app 同时常驻。这样能保持 MCU 目标上的资源上界清晰：一个外部应用页面最多占一个 surface slot、一个 frame registry slot 和一个 task。detached task 仍然不进入这套生命周期，避免把 NSH/调试命令误杀。

这还不是完整任务管理器，但已经覆盖两条资源释放路径：用户主动 Back/Home，以及外部应用自行退出。后续如果要支持多个常驻 app，再把 `ActiveAppTask` 从单槽扩展成固定容量表；当前阶段保持单 managed app，避免 MCU 目标上任务和 framebuffer 资源失控。

#### 18.4.3 Rust app SDK

Rust 应用不应该直接处理 `sscanf`、裸 dirty queue 和 C struct。应用侧最小入口是：

```rust
let mut surface = unsafe {
    WingSurface::open_from_argv(argc, argv)?
};

let pixels = unsafe {
    surface.pixels_rgb565_mut()?
};
draw(pixels, surface.width(), surface.height(), surface.stride_bytes());
surface.submit_full()?;
```

这个 SDK 的边界：

- 无宏：没有 derive macro、proc macro，也不依赖 Bevy 风格反射。
- 无分配：解析 argv、保存 descriptor、resolve frame 都走固定字段。
- 无 runtime 泄漏：应用只看到 `WingSurfaceDescriptor`、像素 slice 和 dirty rect。
- 无 ECS 依赖：第三方应用可以自己写循环，也可以后续接入一个极小的 app-local ESC。
- ABI 唯一：`SurfaceFrameInfoAbi` 由 `surface.rs` 统一定义，平台后端和 SDK 共用同一个 C layout。

SDK 不是新的 FHRE 库，也不是第二个 UI 框架。它只是 Wing 的应用接入门面：把 NuttX task 启动期传来的 `--wing-surface=...` 变成一个可绘制的 Rust surface。声明式 ESC 仍然属于应用内部和 Wing Shell 内部，不跨进程共享 ECS World。

当前 Rust demo 的构建方式是故意保守的：它和 `wing_rust_main` 位于同一个 Rust staticlib 中，由 NuttX builtin registry 注册成另一个入口 `wing_surface_rust_demo_main`。运行时仍然是独立 NuttX task；链接层不额外引入第二个 Rust runtime、panic handler 或 allocator。后续如果要让第三方应用作为完全独立 Rust crate 构建，需要先把 `app_sdk` 提取成一个不含 allocator/panic/runtime 的小 crate，避免多个 Rust staticlib 在 flat NuttX 链接里重复定义运行时符号。

#### 18.4.4 App-local ESC

`apps/wing/rust/src/app_ui.rs` 是第一版应用内声明式 UI 层。它不是 Shell ECS，也不跨任务共享 World，而是在单个外部 app 内部完成：

```text
app composes AppUiFrame
  -> AppUiRuntime compares with previous frame by AppUiKey
  -> compute one clipped dirty rect
  -> clear dirty rect to app background
  -> redraw intersecting specs by z order into RGB565 surface
  -> WingSurface::submit_dirty(rect)
```

第一版支持 `Rect`、`RoundRect`、`Circle` 和小型 5x7 `Text`。所有 spec 都是 `Copy`，存储在 `FixedList<AppUiSpec, N>`，没有堆分配，没有宏，没有反射。应用代码只负责每帧声明：

```rust
let frame = ui.begin();
frame.round_rect(key, rect, z, radius, color);
frame.text(key, x, y, z, "APP LOCAL ESC", color, 2);
ui.render_rgb565(&mut surface)?;
```

这个层级的意义是把“应用 UI 的声明”和“RGB565 软件绘制细节”隔开。对于 MCU 目标，它仍然只是固定容量数组 + 简单 diff + clipped redraw；对于后续 GPU2D/GLES 后端，`AppUiSpec` 可以自然映射为 fill/blit/text 命令，而不要求应用重写逻辑。

#### 18.4.5 App-local input

Wing-managed surface 现在有一条和 dirty queue 对称的输入通路：

```text
NuttX touch/key input
  -> Wing Runtime InputState
  -> ShellMode::ExternalApp 时命中 focused surface rect
  -> global pointer 坐标映射为 surface local 坐标
  -> /wing_surface_input 非阻塞 mqueue
  -> external Rust app WingSurface::poll_input()
  -> AppUiRuntime::handle_input()
  -> app-local AppUiKey click
```

这条通路仍然遵守“Wing 管 surface、应用管自己 UI”的边界。Wing 只负责：

- 判断当前是否在 `ExternalApp` 页面。
- 只把主指针事件转发给 focused managed surface。
- 把屏幕坐标按当前 surface 绘制区域映射到 app framebuffer 坐标。
- 发送 `Move/Down/Up/Cancel` 和 primary button bit。

外部 app 不拿 Shell ECS World，也不拿全局输入状态；它只通过 `WingSurface::poll_input()` 读取自己的 input handle/token 对应事件。app-local ESC 在 `AppUiSpec` 上增加了 `clickable` 标记，应用可以声明：

```rust
let frame = ui.begin();
frame.button_round_rect(key, rect, z, radius, color);
frame.text(label_key, x, y, z, "INPUT: COOL", Color::WHITE, 1);

while let Some(event) = surface.poll_input()? {
    if ui.handle_input(event) == Some(key) {
        toggle_state();
    }
}
```

当前 app-local ESC 已从 pointer click 扩展到轻量手势：

```rust
match ui.handle_gesture(event) {
    Some(AppUiGesture::Click { key, .. }) => {}
    Some(AppUiGesture::Drag { key, delta, .. }) => {}
    Some(AppUiGesture::Swipe { key, direction, .. }) => {}
    _ => {}
}
```

旧的 `handle_input()` 仍保留，作为只关心 click 的兼容 API。第一阶段仍不做键盘文本输入、不做多点触控、不做跨 surface focus。这样是有意收窄：智能手表/MP4/小屏设备上的第一优先级是稳定的点击、拖动和返回逻辑，复杂输入法和多窗口 focus 后面再加。

在手势事件之上，app-local ESC 已开始提供声明式控件 helper。当前已有 slider、toggle、drag handle、scroll list、segmented control、radio list select、top bar、toast、dialog、switch row、status row 和 numeric stepper：

```rust
frame.top_bar(
    page_key,
    top_bar_rect,
    6,
    "SETTINGS",
    "SYSTEM",
    Some(back_key),
    AppUiTopBarStyle::default(),
);
frame.segmented(
    mode_key,
    mode_rect,
    5,
    &segments,
    selected_mode,
    AppUiSegmentStyle::default(),
);
frame.slider(
    level_key,
    Rect::new(42, height as i32 - 178, width - 84, 30),
    6,
    level,
    track,
    fill,
    knob,
);
frame.toggle(mode_key, mode_rect, 6, enabled, off_track, on_track, knob);
frame.scroll_list(
    list_key,
    list_rect,
    5,
    &items,
    first_visible_item,
    34,
    AppUiListStyle::default(),
);
frame.radio_list(
    choice_key,
    choice_rect,
    5,
    &items,
    selected_item_key,
    first_visible_item,
    34,
    AppUiRadioStyle::default(),
);
frame.status_row(
    status_key,
    status_rect,
    6,
    "RENDER LOAD",
    "192",
    render_load,
    AppUiRowStyle::default(),
);
frame.switch_row(
    warm_key,
    warm_rect,
    6,
    "WARM MODE",
    "DISPLAY",
    warm_enabled,
    AppUiRowStyle::default(),
);
frame.stepper(
    level_key,
    level_rect,
    6,
    "LEVEL",
    "192",
    AppUiRowStyle::default(),
);
frame.toast(toast_key, screen_bounds, 40, "SAVED", AppUiToastStyle::default());
frame.dialog(
    dialog_key,
    screen_bounds,
    50,
    "TASK",
    "NUTTX TASK THREAD",
    &buttons,
    AppUiDialogStyle::default(),
);

while let Some(event) = surface.poll_input()? {
    match ui.handle_gesture(event) {
        Some(AppUiGesture::Click { key, .. }) if key == back_key => {
            close_page();
        }
        Some(AppUiGesture::Click { key, .. }) if key == segments[0].key => {
            selected_mode = 0;
        }
        Some(AppUiGesture::Drag { key, point, .. }) if key == level_key => {
            level = slider_value_from_point(level_rect, point);
        }
        Some(AppUiGesture::Drag { key, point, .. }) if key == scroll_list_handle_key(list_key) => {
            first_visible_item = scroll_list_first_from_point(
                list_rect,
                items.len(),
                34,
                point,
            );
        }
        Some(AppUiGesture::Click { key, .. }) if key == warm_key => {
            warm_enabled = !warm_enabled;
        }
        Some(AppUiGesture::Click { key, .. }) if key == stepper_decrement_key(level_key) => {
            level = level.saturating_sub(1);
        }
        Some(AppUiGesture::Click { key, .. }) if key == stepper_increment_key(level_key) => {
            level = level.saturating_add(1);
        }
        _ => {}
    }
}
```

这些 helper 本身只是向 `AppUiFrame` 写入固定数量的 specs：可见部分加一个纯命中区。纯命中区使用 `AppVisual::None`，不会让软件渲染器去扫透明矩形。控件不持有状态，应用只保存 `level`、`enabled`、`first_visible_item` 这类小状态。这样控件保持无宏、无 trait object、无运行时 widget tree 分配，同时应用代码也不用重复写基础命中区绘制逻辑。

`scroll_list()` 当前是第一版 MCU 友好的列表：按固定行高显示可见项，不做像素级 clip tree；列表项使用 `AppUiListItem { key, title, subtitle }`，滚动条 thumb 使用 `drag_handle()`，应用可以通过 `scroll_list_handle_key(list_key)` 区分拖柄事件。这个模型的限制是有意的：先让系统设置、应用列表、诊断页这类小屏列表稳定工作，再讨论复杂文本排版和任意裁剪。

`segmented()` 和 `radio_list()` 面向系统设置里的“模式选择”和“列表选择”。它们不返回状态、不修改状态，只声明当前 selected 的视觉；点击事件仍然以 `AppUiKey` 回到应用侧，由应用决定 `selected_mode` 或 `selected_item_key`。这条边界非常关键：Wing 只提供可复用的绘制/命中 primitive，不把应用状态藏进控件对象里。

`top_bar()` 和 `back_button()` 给系统内置应用统一标题栏和返回入口，但返回行为仍由应用决定。`toast()` 是非模态提示，只写视觉 specs，不吃输入。`dialog()` 是轻量 modal：scrim 使用 dialog 自身的 key 吃掉底层输入，按钮使用更高 z 的 key 优先命中。这样不需要独立 modal manager，也不会让 app-local runtime 引入焦点树。

`switch_row()`、`status_row()` 和 `stepper()` 是给系统设置页补上的第二批基础控件。它们仍然不保存状态：`switch_row()` 只声明当前开关视觉和整行命中区，`status_row()` 只声明状态文本和进度条，`stepper()` 只声明减号/数值/加号三个区域。应用通过 `stepper_decrement_key(parent)` 和 `stepper_increment_key(parent)` 区分两个按钮，再自己修改数值。这样设置页可以拥有手机/手表风格的列表行，但 runtime 不需要 widget tree、回调闭包、trait object 或宏生成代码。

`apps/wing/rust/src/settings_app.rs` 是第一版真正的系统设置应用。它不是 Shell 内部页面，而是注册成 NuttX builtin `wing_settings`，由 `AppRegistry` 的 Settings 图标通过 `PlatformTask + TaskSurface::WingManaged` 启动。应用独立解析 `--wing-surface=...`，创建自己的 `AppUiRuntime<128>`，用 app-local ESC 声明 top bar、brightness status row、numeric stepper、switch rows 和 segmented controls。点击应用内 back button 时，`wing_settings_main()` 正常返回；Runtime 通过 `poll_task_exit()` 回收 surface 并回到 Home。这验证了“系统内置软件也可以是独立 task，同时仍使用 Wing 声明式 UI 门面”的基本路径。

Settings app 与 Shell 的同步走一个很窄的 settings service：

```text
wing_settings app
  -> WingSettingsClient::send(SettingsKey, value)
  -> /wing_settings nonblocking mqueue
  -> Platform::poll_settings_event()
  -> WingRuntime::apply_settings_event()
  -> ShellState { theme, preview_effect, brightness, haptic_enabled, reduce_motion }
```

消息只有 `u16 key + u32 value`，没有动态 payload、没有字符串、没有回调闭包。`wing_settings` 只提交配置意图，不拿 Shell ECS World。Runtime 只在帧开始 drain 固定数量消息，并把合法 key 映射到 `ShellState`。当前已同步 theme、preview effect、brightness、haptic flag 和 reduce motion；brightness 会影响 Shell 壁纸 tint，preview effect 会影响卡片预览页，reduce motion 会关闭 Shell 页面切换动画。

Settings app 的启动态走 launch-time snapshot，而不是再开一个查询通道：

```text
WingRuntime::settings_snapshot()
  -> Platform::launch_task(..., Option<SettingsSnapshot>)
  -> wing_task_launch_config { settings_* }
  -> task runner appends --wing-settings=theme:preview:brightness:haptic:reduce_motion
  -> settings app parse_settings_snapshot()
```

这让 `wing_settings` 打开时能对齐当前 Shell 配置，同时不需要 app 访问 Shell World，也不需要启动后的同步握手。当前 snapshot 只覆盖小配置；后续如果要更多设置，应继续保持固定字段或版本化小结构，避免把 JSON/字符串 KV 放进 MCU 默认路径。

Settings persistence 现在走一个 16 字节固定二进制块，而不是文件格式先行：

```text
SettingsSnapshot
  -> SettingsBlock { magic, version, flags, theme, preview, brightness, checksum }
  -> Platform::save_settings_snapshot()

WingRuntime::new_with_renderer()
  -> Platform::load_settings_snapshot()
  -> ShellState
```

`SettingsBlock` 使用固定 magic、版本号和 FNV 风格校验，字段仍然是小整数和 bit flag。Runtime 启动时先问平台恢复 snapshot；没有后端时回到默认值。Runtime 每帧在 `Input/Update` 之后比较当前 snapshot 和上次保存 snapshot，只在 theme、preview effect、brightness、haptic、reduce motion 这些小配置变化时调用 `Platform::save_settings_snapshot()`。这样 Shell 内部按钮和独立 `wing_settings` 应用都会进入同一个保存路径。

当前 NuttX 后端已经接入轻量文件后端：启动时优先读取 `/data/wing_settings.bin`，适合 sim 的 hostfs 或真实板级数据分区；失败后尝试 `/tmp/wing_settings.bin`，再失败才使用 `SettingsMemoryStore`。保存时按同样顺序写入 16 字节 block，使用 `O_WRONLY | O_CREAT | O_TRUNC`，写满后 best-effort `fsync()`。`SettingsStorageSource` 会记录最终来源为 `DataFile`、`TempFile` 或 `Memory`，System 页面显示 `SETTINGS` 行为 `DATA/TMP/RAM`。这里没有 JSON、路径表、动态 KV 或堆分配；如果后续平台提供 MTD 分区或板级 NVS，只需要替换 `Platform` 的 load/save 实现，Runtime 和 app SDK 不需要改变。

### 18.5 后续代码切分目标

当前为了快速建立骨架，`shell.rs` 仍然集中了一些页面声明。下一步应该拆成：

```text
shell/
  mod.rs
  state.rs
  systems.rs
  home.rs
  notification.rs
  app_switcher.rs
  settings.rs
  launcher.rs
```

拆分规则：

- 页面模块只负责 `compose_xxx(frame, state, metrics)`。
- `systems.rs` 负责输入状态机、命中分发和 draw list 构建。
- `launcher.rs` 负责 AppRegistry 显示、AppManager 请求和平台启动。
- 页面模块不直接访问 framebuffer。
- 页面模块不直接 spawn/despawn entity，只写 `UiFrame`。

### 18.6 下一阶段优先级

下一阶段不要先做复杂视觉效果，优先做系统能力。

已落地：

1. `DirtyRegion`：当前 runtime 已有 dirty region，并由 software renderer 按 dirty rect 绘制。
2. `PreviewItem`：卡片预览已经从固定 demo 改成 `AppRegistry` surface/app 列表。
3. `PreviewItem` 点击进入：每个预览项绑定 `LaunchApp(AppId)`，统一进入 builtin surface 或外部 command launch request。
4. `LaunchKind::PlatformTask`：外部 command 已升级为 `PlatformTask`，显式携带 command、stdio、priority、stack size 和 surface policy。
5. `ResourceManifest`：内置资源已经从 `render/texture.rs` 移到独立资源管线，启动时统一注册进 `TextureStore`。
6. `PreviewEffect::Cube` 最小入口：`Visual::Effect -> DrawCmd::Effect` 已落地，Shell 按 `RendererCapabilities::supports_cube_preview()` 决定是否开放。
7. `PlatformTaskCapabilities/TaskLaunchError`：平台 task 能力和启动错误已经类型化，系统页可以显示当前 task runner 级别。
8. NuttX task runner shim：`apps/wing/rust/task_runner.c` 已接入构建，负责 stdio、priority、stack size 和 builtin task spawn。
9. Wing-managed surface contract：`src/surface.rs` 已定义 `SurfaceRequest`、`SurfaceDescriptor`、`SurfaceCapabilities` 和固定容量 `SurfaceTable`，Runtime/Platform/Diagnostics/System page 已接入能力边界。
10. Surface descriptor launch path：`Platform::launch_task()` 已接收 `Option<SurfaceDescriptor>`，NuttX C shim 会把 descriptor 追加成一个紧凑的 `--wing-surface=...` argv 参数。
11. Surface transport handles：`SurfaceTransportDescriptor` 已生成 frame/dirty/input 三个 handle 和 token，先完成启动期 ABI，后续再映射到具体 NuttX transport。
12. Surface handle registry：`SurfaceTable` 已支持 `resolve_handle()` 和 `submit_dirty()`，可以校验 token、区分 frame/dirty/input handle，并把外部 dirty rect 夹到 surface 范围内。
13. External surface dirty IPC pump：NuttX C shim 已建立 `/wing_surface_dirty` 非阻塞消息队列，Runtime 每帧固定上限 drain dirty event 并提交给 SurfaceTable；外部应用侧也有最小 sender ABI。
14. External surface frame registry：NuttX C shim 已建立 `frame handle + token -> software buffer` 的固定容量 registry，并提供 public header `wing/wing_surface.h` 给外部应用 resolve。
15. External surface compositor：`DrawCmd::Surface`、`Platform::resolve_surface_frame()` 和软件 renderer RGB565/ARGB8888 blit 已落地；`TaskSurface::WingManaged` 应用进入外部 surface 页面，detached task 仍回到预览页。
16. Managed surface demo app：`apps/wing/surface_demo` 已加入构建和应用注册表，能作为独立 NuttX task 解析 `--wing-surface`、resolve RGB565 frame、绘制动态画面并提交 dirty。
17. Managed app lifecycle：`AppManager` 保存 active task，Shell Back/Home 或启动另一个 app 时会调用 `Runtime::close_focused_app()`，NuttX 后端执行 `wing_task_close(pid, frame, token)` 并释放 frame registry，Runtime 同步销毁 `SurfaceTable` slot。
18. Task exit event：`Platform::poll_task_exit()` 已接入 Runtime 帧循环，NuttX 后端用 `waitpid(-1, WNOHANG)` 发现子任务退出，`AppManager::handle_task_exit()` 匹配 active managed app 并释放 surface 资源。
19. Rust app SDK：`src/app_sdk.rs` 已提供无宏、无分配的 `WingSurface`，应用可以解析 `--wing-surface`、resolve framebuffer、获取 RGB565 slice 并提交 dirty rect。
20. Rust external app demo：`src/rust_surface_demo.rs` 已实现 `wing_surface_rust_demo_main`，并通过 `CONFIG_WING_RUST_SURFACE_DEMO_PROGNAME` 注册为 NuttX builtin；Wing 的 `Surface` 图标默认启动 Rust 版 demo。
21. App-local ESC：`src/app_ui.rs` 已提供固定容量 `AppUiFrame/AppUiRuntime`，Rust demo 已从逐像素绘制改成声明式 compose + dirty diff + RGB565 clipped redraw。
22. App-local input：NuttX C shim 已建立 `/wing_surface_input` 非阻塞队列，Runtime 会把 ExternalApp 页面内的 pointer 事件映射到 focused surface local 坐标；`WingSurface::poll_input()` 和 `AppUiRuntime::handle_input()` 已支持应用声明按钮并接收 click。
23. Binary resource packer bootstrap：`apps/wing/rust/build.rs` 已生成 `OUT_DIR/wing_assets.bin`，Runtime 通过 `render/resource.rs` 的固定 record parser 注册内置 RGB565/A8 资源，不再 include `builtin_assets.rs` 源数组 manifest。
24. Manifest-driven resource source：`apps/wing/resource/manifest.txt` 已成为第一版外部资源源文件，`build.rs` 从 manifest 读取稳定 `ImageId`、尺寸、格式和生成器/bitmap 数据，再打包成同一个 `wing_assets.bin`；运行时边界没有变化。
25. App-local gesture：`AppUiRuntime::handle_gesture()` 已提供 Press/Click/Drag/Swipe/Release/Cancel 事件，旧 `handle_input()` 保持 click-only 兼容；Rust surface demo 已用按钮左右滑演示轻量 swipe。
26. App-local slider widget：`AppUiFrame::slider()` 已提供第一版声明式拖动控件，内部生成固定 specs 和纯命中区；Rust surface demo 已用 slider 控制 demo 的 level 值。
27. App-local toggle widget：`AppUiFrame::toggle()` 已提供第一版声明式开关控件，内部生成轨道、旋钮和纯命中区；Rust surface demo 已用 toggle 替代冷暖模式按钮。
28. App-local drag handle widget：`AppUiFrame::drag_handle()` 已提供可拖动把手 helper，内部生成圆角把手、grip 线和放大的纯命中区。
29. App-local scroll list widget：`AppUiFrame::scroll_list()` 已提供固定行高、固定 specs、无分配的列表 helper；Rust surface demo 已接入列表项点击、上/下滑滚动和滚动条拖柄。
30. App-local segmented control：`AppUiFrame::segmented()` 已提供固定段数、无状态的分段选择控件；Rust surface demo 已用它切换 SYS/INPUT/FX 模式。
31. App-local radio list select：`AppUiFrame::radio_list()` 已提供列表选择控件，复用滚动 metrics 和拖柄 helper；Rust surface demo 已用它显示当前选中的系统项。
32. App-local top bar/back button：`AppUiFrame::top_bar()` 和 `back_button()` 已提供系统内置应用统一标题栏入口；Rust surface demo 已接入返回按钮。
33. App-local toast：`AppUiFrame::toast()` 已提供非模态提示 helper；Rust surface demo 已用它反馈选择/返回动作。
34. App-local dialog：`AppUiFrame::dialog()` 已提供轻量 modal helper，scrim 吃掉底层输入、按钮高 z 优先命中；Rust surface demo 已用 TASK 项弹出对话框。
35. App-local switch row：`AppUiFrame::switch_row()` 已提供系统设置风格的整行开关控件，视觉开关与命中区分离；Rust surface demo 已用它替代裸 toggle。
36. App-local status row：`AppUiFrame::status_row()` 已提供标题、右侧状态值和进度条组合，用于负载、电量、存储等只读状态展示；Rust surface demo 已用它展示 render load。
37. App-local numeric stepper：`AppUiFrame::stepper()` 已提供减号/数值/加号控件，并通过 `stepper_decrement_key()`、`stepper_increment_key()` 暴露无状态事件边界；Rust surface demo 已用它调整 level。
38. Wing Settings app：`src/settings_app.rs` 已实现 `wing_settings_main`，并通过 `CONFIG_WING_RUST_SETTINGS_PROGNAME` 注册为 NuttX builtin；`AppRegistry` 的 Settings 图标现在启动独立 Wing-managed surface task，应用内 back 退出后 Runtime 回收 surface 并回到 Home。
39. Settings service：`src/settings.rs` 定义固定 key/value 协议，NuttX C shim 建立 `/wing_settings` 非阻塞 mqueue，`WingSettingsClient` 可从独立 app 发送配置更新；Runtime 每帧 drain 固定数量消息并同步到 ShellState。
40. Settings state bootstrap：Runtime 启动 Wing-managed app 时会把当前 `SettingsSnapshot` 填入 launch config，task runner 追加 `--wing-settings=...`，`wing_settings` 启动后用 `parse_settings_snapshot()` 初始化本地 UI 状态。
41. Settings persistence boundary：`SettingsBlock` 定义 16 字节固定二进制配置块，`Platform` 暴露 `load_settings_snapshot/save_settings_snapshot`，Runtime 启动时恢复配置、运行中按 snapshot diff 保存；NuttX 后端保留 `SettingsMemoryStore` 作为失败回退。
42. Runtime frame diagnostics：`RuntimeDiagnostics` 已记录主 shell 帧状态、dirty/input/surface/settings/task 事件和容量溢出，System 页面显示静态诊断标签。
43. AppUi frame hint + stats：`AppUiFrameHint`、`AppUiFrameResult` 和 `AppUiFrameStats` 已接入 `AppUiLoop`，settings app 可以在无活动时 idle，demo app 可以保持 active 动画。
44. AppUi typed signals：`AppUiSignalId/AppUiSignal` 已把 UI key 和业务命令 ID 分离，settings app 和 Rust surface demo 均已迁移到固定 ID + `i32` payload 的 typed signal。
45. AppUi staged schedule：`AppUiStagedSchedule<Input, Tick, Prepare>` 已提供 input/prepare/tick 三段静态调度，`Prepare` 在 update 之后、view 之前运行，`Tick` 在帧后运行；默认 stage 为 `()`，不引入宏和动态调度图。
46. AppUi RGB565 runner：`run_app_ui_rgb565()` 已收敛外部 app 的 RGB565 格式检查、`AppUiRuntime/AppUiLoop` 创建、sleep 和错误返回路径；settings app 与 Rust surface demo 已迁移，应用入口只负责打开 surface 和构造本地状态。
47. AppUi multi dirty rect：`AppUiRuntime::render_rgb565_result()` 已使用固定容量 dirty region，局部变化时提交多个小 dirty rect；超过 `APP_UI_DIRTY_RECT_CAPACITY` 或 frame spec 溢出时退化为 union/full rect，并通过 `AppUiRenderResult/AppUiFrameResult/AppUiFrameStats` 记录 dirty rect 数和溢出状态。
48. AppUi SDK facade：`src/app_ui_sdk.rs` 已把外部 app 需要的 app、command、input、layout、render、runner、schedule、surface、widget 和 settings API 分组 re-export；settings app 与 Rust surface demo 已迁移到 `app_ui_sdk::prelude::*`，减少对 `app_ui.rs` 内部布局的直接依赖。
49. AppUi NuttX argv runner：`run_app_ui_rgb565_from_argv()` 已把 NuttX builtin 的 `argc/argv` surface open、RGB565 loop 执行和 `WingSurfaceError::exit_code()` 转换收敛到 SDK；settings app 与 Rust surface demo 的入口只保留 app 状态初始化。
50. AppUi app template：`src/app_ui_template.rs` 已提供可编译检查的最小外部 app 骨架，使用 `app_ui_sdk::prelude::*`、`run_app_ui_rgb565_from_argv()`、typed signal 和 staged schedule；当前不导出 `#[no_mangle]` builtin 入口，避免在系统里新增未使用命令。
51. AppUi frame budget：`AppUiFrameBudget` 已接入 `AppUiFrameResult` 和 `AppUiFrameStats::last_budget()`，记录 spec、event、command、dirty rect 的使用量、固定容量和溢出状态；`AppUiFrameFlags` 也能标出当前/上一帧 spec 溢出，便于外部 app 做轻量诊断。
52. AppUi budget observe hook：`AppUiApp::observe_frame()` 与 `AppUiScheduledApp::observe_scheduled_frame()` 已接入 `AppUiLoop`，Rust surface demo 现在把上一帧 `AppUiFrameBudget` 显示为 `APP BUDGET` 静态诊断行，验证外部 app 能无分配地观察自身容量压力。
53. Binary file asset source：`apps/wing/rust/build.rs` 的 manifest parser 已支持 `asset <id> rgb565/a8 <w> <h> file <relative>`，可直接打包离线工具生成的 `.rgb565/.a8` 原始资源；构建时校验路径和字节数，运行时 `render/resource.rs` 也会校验 blob record 长度。
54. Build-time atlas slice source：manifest 已支持 `asset <id> rgb565/a8 <w> <h> atlas <relative> <atlas_w> <atlas_h> <x> <y>`，构建期校验 atlas 字节数并裁切成紧密 texture record；运行时 renderer 不需要 atlas stride/origin，保持普通 `Texture` 热路径。
55. Atlas slice table source：manifest 已支持 `atlas <format> <relative> <atlas_w> <atlas_h>` block，内部用多行 `slice <id> <w> <h> <x> <y>` 批量声明切片；packer 只读取和校验 atlas 一次，每个 slice 仍输出普通紧密 texture record，为后续二进制 slice table 做语义铺垫。
56. Binary manifest/payload split：`build.rs` 现在除兼容的 `wing_assets.bin` 外，还生成 `wing_manifest.bin` + `wing_payload.bin`；`TextureStore::with_builtin_textures()` 已优先走 split parser，资源索引和像素 payload 分离，后续替换为 ROMFS/flash 资源分区时不需要改 Shell/UI/Renderer。
57. Resource partition boundary：`ResourcePartition { manifest, payload }` 已成为运行时资源边界，`Platform::resource_partition()` 默认返回内置 pair，`WingRuntime` 初始化时通过 platform 注入到 `TextureStore`；后续 NuttX 板级代码可以改为 ROMFS/hostfs/flash 只读切片，Shell/UI/Renderer 不需要知道资源来源。
58. NuttX file resource backend：`NuttXPlatform::resource_partition()` 现在会先尝试从 `/etc/wing/wing_manifest.bin` 与 `/etc/wing/wing_payload.bin` 读取资源分区，使用固定静态缓冲区，不做堆分配；文件不存在、超过容量或 parser 校验失败时，`TextureStore` 自动回退内置 `include_bytes!` 资源。
59. Resource packaging into sim ROMFS：Cargo packer 除 `OUT_DIR` 外会把 `wing_manifest.bin` 与 `wing_payload.bin` 写入 `apps/wing/resource/generated/`；Wing NuttX Makefile 在 native sim 构建时复制到 `boards/sim/sim/sim/src/etc/wing/`，板级 ROMFS `RCRAWS` 已包含这两个文件，因此运行时能从 `/etc/wing/` 读取真实板级资源分区。
60. Resource source diagnostics：`ResourcePartition` 现在标记 `Builtin/External` 来源，`TextureStore` 记录最终资源来源为 `Builtin`、`External` 或 `BuiltinFallback`；System 页面显示 `RESOURCE` 状态，方便确认当前 NuttX sim 是否真的走 ROMFS 资源包。
61. NuttX settings file backend：`NuttXPlatform::load_settings_snapshot/save_settings_snapshot` 现在读写固定 16 字节 `SettingsBlock`，优先 `/data/wing_settings.bin`，再落到 `/tmp/wing_settings.bin`，最后回退 `SettingsMemoryStore`；同时修正 NuttX `O_RDONLY` 常量并让固定缓冲读取函数接受刚好等于容量的文件。
62. Settings backend diagnostics：`SettingsStorageSource` 已成为平台可读的 `Copy` 状态，`NuttXPlatform` 在 load/save 时记录 `DataFile/TempFile/Memory`，`WingRuntime::settings_storage_source()` 暴露给 Shell，System 页面用 `SETTINGS` 行显示 `DATA/TMP/RAM`。
63. Software Cube preview fallback：`SoftwareRenderer` 现在声明 `effects` 能力并处理 `EffectKind::PreviewCube`，用无分配的整数 scanline quad fill、line stroke 和透明阴影绘制 2.5D cube；设置页可以在纯软件后端切换到 `CUBE`，后续硬件后端仍可替换为真正 mesh3d。
64. AppUi compact diagnostics pack：AppUi SDK 现在提供 `app_ui_budget_label()`、`app_ui_budget_pressure()`、`app_ui_budget_summary()`、`app_ui_frame_flags_label()` 和 `app_ui_frame_result_label()`，外部 app 可以复用 `OK/SPEC FULL/DIRTY MULTI` 等静态映射；Rust surface demo 已迁移，诊断显示不需要动态格式化或分配。
65. Resource tool bootstrap：`apps/wing/resource/tools/wing_resource_tool.rs` 已提供第一版 host-side 原始资源工具，可从 recipe 生成 `.rgb565`、`.a8`、row-major `.atlas` 和 manifest fragment；工具保持在 NuttX/no_std Cargo 目标之外，运行时仍只消费 `wing_manifest.bin + wing_payload.bin`。
66. Glyph table resource frontend：`wing_resource_tool` 现在支持 `glyph_table`，可以把 recipe 内联的位图字形直接打包成 A8 atlas 和 manifest fragment；这给数字、小字体和 icon-font 表留出了无 PNG 依赖的第一条离线路径。
67. BDF bitmap font frontend：`wing_resource_tool` 现在支持 `bdf_font`，可解析最小 BDF bitmap font，将 `ENCODING` 映射为 `id_base + ENCODING` 的稳定 `ImageId`，并输出固定 cell 的 A8 atlas 和 manifest fragment。
68. Shell-only resource workspace：`apps/wing/resource/` 现在收敛为 Shell 绘制资源目录，不再保留设计文件 cache、case manifest 或 draw-program 运行时入口；`fs/` 会原样复制到目标 `/etc/wing/resource/`，其中 `fs/icons/shell/*.svg` 是真正的运行时 SVG 图标资源，`icons/bootstrap/` 保存内置 fallback SVG 源，`manifest.txt` 则从原始 PNG 壁纸生成当前显示尺寸的 RGB565 背景 payload，`generated/` 保存打包产物。
69. Runtime filesystem resources：当前 sim ROMFS 会包含 `/etc/wing/resource/images/shell/*.png`、`/etc/wing/resource/fonts/simhei.ttf` 和 `/etc/wing/resource/icons/shell/*.svg`。`NuttXPlatform` 已能从这些路径读取 header 并识别 PNG/JPEG/TrueType/OpenType/TTC/SVG，Runtime/System 页面用 `FILES`、`BG`、`FONT`、`ICON` 行显示原始资源是否存在以及当前绘制路径；`simhei.ttf` 现在会作为默认运行时字体加载，Shell `DrawCmd::Text` 与 App UI RGB565 surface 都经由同一套 font store 栅格化，无法映射或暂不支持解析的字符绘制为按字号缩放的占位符，后续 JPEG 像素解码和持久 glyph/SVG cache 接在这层。
70. Runtime PNG desktop path：`manifest.txt` 使用 `wallpaper` 条目声明稳定 `ImageId` 1-4，Cargo build script 仍会生成 `generated/shell/*.rgb565` 作为确定性 fallback；Shell 启动时优先从 `/etc/wing/resource/images/shell/*.png` 读取原始背景文件，用内置 PNG/deflate 解码器缩放到当前窗口大小，再注册成运行时背景纹理。
71. Runtime SVG vector icons：Shell 全部图标已从 `/home/uan-gpd/codes/icons/icons` 选取 Bootstrap Icons SVG 源，语义化重命名后复制到 `apps/wing/resource/fs/icons/shell/`；Wing 启动时优先从 `/etc/wing/resource/icons/shell/*.svg` 注册 `SvgStore`，缺失或解析失败时回退到 `icons/bootstrap/*.svg` 内置 bytes。`SvgStore` 运行时解析 path/curve/arc，并按目标尺寸缓存 A8 mask；System 页 `SVG` 行显示 mask cache 的 tier 与 warm/hot/full/evict 状态。运行时不携带 PNG/RGB565/A8 图标资源，也不把 SVG 图标塞进 `TextureStore`。
72. Stable Shell icon semantics：`VectorIcon` 仍然作为 Shell/UI 的语义枚举，Settings/System/Terminal/Surface/Notification 等入口通过 `ThemedIcon -> VectorIcon` 映射到 Bootstrap SVG；AppRegistry 不再依赖图标 `ImageId`，避免把图标资源和 wallpaper texture id 混在一起。
73. Background layer diagnostics：`RenderPlan` 增加 `RenderLayerSummary`，记录 Shell 背景快路径识别结果、`RenderBackgroundRoute` 和跳过的前缀命令数；`RendererCapabilities` 增加 `background_layer/background_plane`，`SoftwareRenderer::render_plan()` 会结合 viewport 与 `TextureStore` 判断 `ClearGradient + fullscreen RGB565 wallpaper`，System 页面新增 `LAYER` 行，资源缺失时 `HEALTH` 显示 `BG MISS`，后端缺少背景层能力时显示 `BG CAP`。
74. BackgroundPlaneRenderer skeleton：新增 `BackgroundPlaneRenderer`，默认用软件 framebuffer 呈现，但声明 `RendererKind::Gpu2d` 与 `background_plane`，可通过 `wing_rust --renderer=plane` 启动；这让 `BG PLANE` 路径在没有真实 GPU 驱动时也能验证，后续真实硬件后端只需替换 draw/present 侧实现。
75. BackgroundPlaneOps contract：`BackgroundPlaneRenderer<O>` 现在泛型于 `BackgroundPlaneOps`，每帧只在 `RenderBackgroundRoute::NativePlane` 时提交 `BackgroundPlaneSubmission`；默认 `SimBackgroundPlaneOps` 验证 RGB565 纹理并记录提交/拒绝次数，为后续 NuttX 板级 GPU2D/GLES plane ops 留出零动态分发入口。

继续优先：

1. GPU/GLES Cube backend：为 GPU2D/GLES/Custom 后端实现真正 mesh3d 或硬件加速 cube；软件 fallback 保留为低端平台路径。
2. Shell resource refinement：继续围绕 `fs/images/shell/`、`fs/icons/shell/`、`icons/bootstrap/`、`generated/shell/` 和 manifest 生成器收敛真正被 Shell 使用的资源；背景继续生成稳定 RGB565 fallback，图标继续保持 SVG 源 + runtime parse/cache，不再恢复 bitmap icon payload。
3. Settings MTD/NVS backend：在真实板级 flash 分区明确后，把 `SettingsStorageSource` 扩展到 MTD/NVS；Runtime 和 AppUi SDK 仍保持同一份 snapshot/save 边界。

这条顺序能保证 Wing 仍然轻：先把数据边界变清楚，再逐步提高视觉质量和渲染性能。

### 18.7 当前资源管线

当前代码已经建立第一版资源管线：

```text
TextureStore
  -> Texture { ImageId, width, height, PixelFormat, static data }
  -> Visual::Image { image, tint }
  -> DrawCmd::Image { rect, image, tint }
  -> SoftwareRenderer::draw_image()
```

第一阶段只支持两种格式：

- `Rgb565`：用于壁纸、静态背景、低成本照片类资源。
- `A8`：用于图标和 glyph mask，通过 tint 变成不同主题颜色。

这样做的原因：

- RGB565 对 MCU/低功耗 MPU 更友好，内存和带宽都是 RGBA8888 的一半。
- A8 图标可以共享一份 mask，主题色由 tint 决定。
- 帧内不解析 PNG，不分配大纹理，不做动态格式转换。
- 同一份 `DrawCmd::Image` 后续可以映射到软件 blit、DMA2D/PXP/VG-Lite blit，或者 GLES texture draw。

当前内置资源已经通过 manifest-driven build-time binary packer 进入 `TextureStore`：

- Aurora/Dusk 竖屏与横屏四个 RGB565 壁纸 fallback。
- Shell 图标不再进入 `TextureStore`，而是由 `SvgStore` 优先从 `/etc/wing/resource/icons/shell/*.svg` 解析为矢量几何，并按尺寸缓存成 A8 mask；内置 Bootstrap SVG 只作为 fallback。

`apps/wing/resource/manifest.txt` 是第一版外部资源源文件：

```text
asset 1 rgb565 64 96 gradient 0c172e 105a62 4eb09a
asset 2 rgb565 64 96 file wallpaper_dusk.rgb565
asset 16 a8 16 16 bitmap
0000001111000000
...
end
asset 20 a8 16 16 file icon_extra.a8
asset 21 a8 16 16 atlas icons.a8.atlas 128 64 32 16
atlas a8 icons.a8.atlas 128 64
slice 22 16 16 0 0
slice 23 16 16 16 0
end
```

`file` 源已经可以直接引用离线工具生成的原始二进制资源。路径必须相对 `apps/wing/resource`，不允许绝对路径或 `..`，并会通过 `cargo:rerun-if-changed` 参与增量构建。packer 会在构建时验证字节数：RGB565 必须是 `width * height * 2`，A8 必须是 `width * height`。

`atlas` 源使用同样的原始像素格式，但额外声明 atlas 尺寸和切片坐标。单条 `asset ... atlas ...` 适合少量验证资源；`atlas ... slice ... end` block 适合大量 icon/glyph 切片，packer 会读取并校验 atlas 一次，再把每个 slice 裁成紧密 texture record 写入 `wing_assets.bin`。这让离线工具可以先产出 `.atlas`，而当前 runtime/renderer 仍然不需要处理 stride、UV 或 source rect。

`apps/wing/rust/build.rs` 读取 manifest 后生成 `OUT_DIR/wing_manifest.bin` 与 `OUT_DIR/wing_payload.bin`，同时把稳定产物复制到 `apps/wing/resource/generated/`，并保留旧的 `wing_assets.bin` 合包用于兼容：

```text
wing_manifest.bin
  header: magic "WRM1", count, record_len, payload_len, manifest_len
  records[count]: id, width, height, format, data_offset, data_len

wing_payload.bin
  data: concatenated rgb565/a8 payloads referenced by manifest records

wing_assets.bin
  compatibility combined blob: "WRS1" header + records + payload
```

`render/resource.rs` 用 `include_bytes!` 引入默认 manifest/payload pair，并把它包装成 `ResourcePartition`。`Platform::resource_partition()` 是运行时资源来源边界，当前默认返回内置 pair；`NuttXPlatform` 会先尝试从 `/etc/wing/wing_manifest.bin` 和 `/etc/wing/wing_payload.bin` 读取外部资源分区，使用固定静态缓冲区，文件缺失或解析失败时回退内置资源。native sim 构建时，Wing Makefile 会把 `apps/wing/resource/generated/wing_manifest.bin` 与 `wing_payload.bin` 复制到板级 `src/etc/wing/`，sim ROMFS 再把它们暴露为 `/etc/wing/` 文件。`WingRuntime` 初始化时把该 partition 注入 `TextureStore`，再由固定 record parser 逐项注册纹理；`TextureStore::resource_source()` 可以诊断最终使用的是 external、builtin 还是 fallback。旧的 combined blob parser 仍保留为兼容入口。运行时 parser 会校验 record 数据长度与格式/尺寸匹配。这里没有运行时路径字符串、没有帧内解码、没有堆分配；renderer 仍只消费 `ImageId`、尺寸、格式和静态数据切片。

`manifest.txt` 里的 gradient/bitmap 仍是可读、可 diff 的 bootstrap 源；`file/atlas` 源则是面向真实美术产物的第一步。后续 `/apps/wing/resource` 还应该继续收敛为 compact manifest 和 slice table：

当前已有第一版 std-only host 工具 `apps/wing/resource/tools/wing_resource_tool.rs`，用于生成 build.rs 可直接消费的 raw source 文件：

```text
rgb565_gradient raw/wall.rgb565 64 96 0c172e 105a62 4eb09a
a8_bitmap raw/icon.a8 16 16
atlas_row a8 raw/icons.a8.atlas raw/icons.manifest 16 16 8
glyph_table raw/glyphs.a8.atlas raw/glyphs.manifest 5 7 16
bdf_font tiny_digits.bdf raw/digits.a8.atlas raw/digits.manifest 5 7 16 400
```

它可以生成 `.rgb565`、`.a8`、row-major `.atlas`、glyph/BDF A8 atlas 和 manifest fragment，但刻意不进入 NuttX/no_std Cargo 目标。后续 PNG 转换器也应该把输出落到同一类 raw 文件或 recipe，再交给当前 build-time binary packer。

```text
resource/
  manifest.bin
  icons.a8.atlas + slice table
  glyphs.a8.atlas + slice table
```

启动时由同一个 binary resource parser 注册到 `TextureStore`。Shell、UI 和 Renderer 不需要知道资源来自 build.rs、ROMFS、hostfs 还是后续 flash 分区；调试路径只读取 `TextureStore` 的来源枚举，不参与帧内资源解析。

### 18.8 当前脏区提交模型

当前代码已经加入节点级 `DirtyRegion`，但它还不是最终的 tile 级局部重绘系统。现在的目标是避免静止界面每 16ms 都重新 layout、build draw list、整屏提交 framebuffer，并让普通 UI 节点变化只污染必要区域。

当前链路：

```text
Input/Update
  -> PresentedState { shell, app }
  -> decide whether layout is needed
  -> UiFrame declarative specs
  -> World::apply_ui_frame() returns old/new visual bounds
  -> DirtyRegion::include_rect() fixed small rect list
  -> RenderBuild
  -> RendererBackend::draw(draw_list, render_plan, textures, fonts, dirty)
  -> Platform::present(framebuffer, dirty)
```

当前判定粒度分成两层：

- `ShellState` / `AppRenderState` 改变：触发 Layout，但不默认全屏 dirty。
- `ThemeKind` 改变：背景和壁纸会变化，直接标记全屏 dirty。
- `World::apply_ui_frame()` 比较同一个 `UiKey` 的旧组件和新声明，组件变化时合并旧 visual bounds 和新 visual bounds。
- 本帧没有继续声明的旧 UI 实体会被逐个 sweep，sweep 前把旧 visual bounds 合入 dirty，不创建临时 stale `Vec`。
- 状态未改变且没有系统主动标记 dirty：跳过 Layout、RenderBuild、draw 和 present。

`DirtyRegion` 当前是固定容量的小矩形列表，不在帧内分配堆内存。默认容量是 8 个 rect：相交或贴边的 rect 会先合并；容量满时，会选择合并后面积增长最小的已有 rect。每帧进入 renderer/present 前会先尝试 16x16 tile 对齐，只有扩展后的覆盖面积仍在阈值内才采纳；这让 NuttX sim 的逐行 present、后续 MCU tile buffer，以及 DMA2D/PXP/VG-Lite 这类按块刷新的后端能消费更规整的 dirty 边界。之后再做一次固定预算 compact：当两个 dirty rect 合并增加的重绘面积低于阈值时，优先合并最便宜的一对，把提交数量压到目标 rect 数附近。这样多个分散的小 UI 变化不会立刻退化成整屏 dirty，也不会让软件 renderer 在许多小 clip 上重复扫描 DrawList。

`UiFrame`、`DrawList`、`Schedule`、`AppRegistry` 和 `TextureStore` 当前都是固定容量表，不在普通页面刷新、初始化系统注册、应用 manifest 注册或内置纹理注册中扩容。`RenderBuild` 也避免为 draw order 创建帧内临时列表。它每次寻找下一个最小 `(z, dense_index)` 的可见实体，找到后立刻把该实体转成 `DrawCmd`。系统调度同样不在每个 phase 复制系统列表，而是按 index 读取函数指针并执行。这个策略的前提是 Wing 的目标界面实体数量、系统数量、应用数量和资源数量可控；如果后续复杂应用或 3D effect 让可见实体数量显著增加，再引入固定容量 order buffer 或 arena，而不是回到无界临时分配。

`RuntimeDiagnostics` 当前会采样这些固定容量表的 overflow 标记，并用一个小 bitmask 记录当前帧来源和历史 latched 来源。它也记录最近一帧 `DirtyRegionSummary`，包括 dirty rect 数、覆盖率、bounds 覆盖率、是否 full redraw，以及是否已对齐到当前 runtime 的 tile 网格；系统页用 `DIRTY` 行显示 `IDLE/FULL/FRAG/TILED/BROAD/MULTI/TIGHT`，`HEALTH` 行会在 dirty 过碎时显示 `DIRTY FRG`。字体和 SVG 图标 cache 也暴露为静态标签：`GLYPH/GHIT` 用于字形 cache，`SVG` 用于图标 mask cache，仍然不分配字符串、不打印日志、不影响渲染后端；后续设置页、调试串口或测试用例可以直接读取 `diagnostics.capacity()`、`diagnostics.frames()` 和对应 cache summary 判断容量与重绘压力。

`RendererBackend::draw()` 和 `Platform::present()` 都已经接收 `DirtyRegion`。默认软件渲染器会逐个 dirty rect 作为内部 clip，只绘制和脏区相交的命令与像素；NuttX 平台也会逐个 dirty rect，根据 framebuffer stride 做逐行拷贝，而不是无条件复制整块显存。即使当前上层多数变化仍然先走全屏 dirty，这个接口边界已经为后续 tile dirty 留好位置。

Runtime 还提供 `renderer_kind()` 和 `renderer_capabilities()`。这两个接口只返回 `Copy` 值，用于后续设置页、预览效果和测试判断当前后端能力；它们不改变 ESC 数据流，也不让页面直接调用 renderer 私有 API。

文字节点需要特别处理：`UiFrame::text()` 的 layout rect 只是锚点，真实绘制范围由 `Visual::bounds()` 根据文本长度和 scale 计算。这样文字变化不会只污染 `1x1` 像素。

当前限制也必须明确：

- 动画、计时器、异步应用 surface 后续必须在状态变化时调用 `mark_all_dirty()`，或进一步调用 rect 级 dirty。
- `DirtyRegion` 仍然是少量 rect，不会记录大量细碎区域；复杂动画或粒子式 UI 后续需要 tile bitset。
- 透明混合仍然由软件 renderer 逐像素处理，GPU2D/GLES 后端需要复用同一 dirty 输入重新实现 blit 策略。

下一步优化顺序：

1. 保留现在的全屏 dirty 作为保守兜底。
2. 把动画系统接入 rect 级 dirty，移动节点时只污染 old/new bounds。
3. 增加 tile bitset 后端，用于复杂动画或大量小控件同时变化。
4. GPU2D/GLES 后端复用同一个 dirty 输入，只改变 present/blit 策略。

这样 Wing 的渲染模型仍然保持轻量：声明式 UI 只描述状态和节点，ECS 系统负责生成稳定实体和绘制命令，平台层只接收最终的 framebuffer 与脏区，不反向依赖 shell、应用或资源系统。

### 18.9 当前动画模型

当前代码加入了第一版轻量页面过渡动画。它不是独立动画引擎，也不是命令式绘制回调，而是普通 ESC 状态：

```text
SlideTransition { direction, track: FrameTrack }
  -> Update phase advances frame
  -> Layout phase composes normal UiFrame
  -> UiFrame::translate_layer(current_layer, dx, dy)
  -> UiFrame::fade_layer(current_layer, alpha)
  -> World::apply_ui_frame() compares old/new visual bounds
  -> DirtyRegion records changed rects
```

动画类型位于 `src/animation.rs`，Shell 只是它的一个使用方。这个边界很重要：页面、卡片、应用窗口、通知弹层以后都应该复用同一类小型动画状态，而不是在各自模块里重复写私有计时器。

这个模型的重点是：动画仍然表现为声明式 UI 节点的位置变化。Renderer 不知道“动画”这个概念，只知道本帧 draw list 和 dirty rects。这样后续换成软件渲染、DMA2D/PXP/VG-Lite 或 GLES 后端时，动画系统不需要重写。

当前只实现页面进入方向：

- Notification：从上方进入。
- AppSwitcher：从下方进入。
- Settings：从右侧进入。
- 返回 Home：按来源方向进入。

动画持续时间是固定帧数 `DEFAULT_TRANSITION_FRAMES`，底层由 `FrameTrack { frame, duration }` 表达。`FrameTrack` 只输出整数位移和 0..255 alpha，不依赖堆分配、动态调度器或浮点插值。后续如果接入真实时间源，也应该保持同样边界：Update 只更新状态，Layout 只声明节点，Renderer 只处理 draw list 和 dirty。

后续动画扩展规则：

1. 优先使用整数插值和定点数，避免 MCU 平台上的浮点成本。
2. 移动、缩放、淡入淡出都应该变成组件状态，然后由声明式 UI 生成新 bounds。
3. 动画系统需要主动推进状态；没有状态变化时 Runtime 必须继续跳过帧。
4. 大量节点动画再引入 tile bitset，不把 tile 系统提前塞进普通静态 UI 路径。

当前实现只保留最小的 `SlideTransition`：

- `SlideDirection` 描述进入方向，不绑定 Shell 页面枚举。
- `FrameTrack` 描述整数帧进度，是后续 offset、alpha、scale 轨道的共同基础。
- `advance()` 在 Update 阶段推进整数帧。
- `offset(width, height)` 返回整型位移，Layout 阶段把位移应用到对应 layer。
- `alpha()` 返回 0..255 的定点透明度，Layout 阶段把它写入 `UiSpec::alpha`。
- `duration == 0` 或 `SlideDirection::None` 时不产生动画。

`UiSpec` 已经携带声明式 alpha，`World::apply_ui_frame()` 会把它转换成 `Visibility { visible, alpha }` 组件。渲染阶段继续只消费组件和 draw list：透明度是 UI 状态的一部分，不是 renderer 私有的临时特效。

这意味着动画模块没有宏、没有 trait object、没有堆分配，也不需要 renderer 反向理解 UI 状态。动画只改变声明式 UI 输入，脏区系统通过 old/new bounds 和 alpha 变化决定本帧需要重绘的区域。
