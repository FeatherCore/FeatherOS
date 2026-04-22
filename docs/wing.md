# Wing Desktop Environment

`Wing` 是 FeatherOS 基于 FHRE 构建的桌面环境。

本文档的目的不是继续描述一套独立 GUI 壳层如何向 FHRE 输出渲染命令，而是严格依据 `docs/ARCHITECTURE.md` 对齐 FHRE 标准，把 Wing 收敛为符合 FHRE 模板的声明式 ECS UI 桌面系统。

本文档描述两件事：

- 当前仓库里已经存在的 Wing 实现基础
- 后续必须收敛到的目标架构：严格沿用 FHRE 的 `MainWorld/RenderWorld` 声明式 ECS 桌面应用模板

本文档同时把修改进度同步记录在文末，作为 Wing 架构迁移的工作台账。

当前仓库中的 `Wing` 已经不是纯概念设计，而是已经具备：

- 可独立编译的 `apps/wing/rust` 桌面壳库
- 可运行的 `apps/examples/wing/rust` 示例入口
- 已接入 NuttX `sim:wing` 配置与构建脚本
- 已打通从输入采集到画面输出的最小运行链路
- 已具备桌面、任务栏、启动器、多窗口和基础控件系统

## 架构立场

Wing 的长期目标不是成为 FHRE 旁边的一套“自维护 GUI 运行时”。

标准的声明式 ECS 是 Wing 的最高优先级原则，也是所有设计取舍的根判断标准。

Wing 的正确定位是：

- 它是强制依赖 FHRE 的桌面应用框架
- 它复用 FHRE 的 `App`、双世界 ECS、输入、事件、提取和软件渲染能力
- 它把桌面、窗口、控件、焦点、布局、交互这些语义组织进 `MainWorld`
- 它通过标准 extract 流程把渲染态送入 `RenderWorld`

更直接地说：Wing 可以理解为“基于 FHRE 实现的桌面系统”，而不是“和 FHRE 并列的一套系统”。

因此，只要 FHRE 已经支持某项能力，Wing 就不应再自行实现一份平行版本。

这包括但不限于：

- `App` 生命周期与主循环
- `MainWorld / RenderWorld` 双世界结构
- `entity_sync_system()`
- `extractors.run()` 与 extract / queue 分相
- `ButtonInput<T>`、`MousePosition`、`Pointer<E>` 输入与事件模型
- `CameraPlugin`、`Camera`、`PrimaryScreen`、`View` 视图体系
- `Assets<Image>`、`Handle<Image>`、`GpuTextures` 资产与纹理上传路径
- `SoftwareBackend` 软件渲染执行

因此，Wing 不应继续扩大下面这种模式的占比：

- 在 `Wing` 单体对象里集中维护桌面状态
- 在对象方法里推进输入、布局和交互
- 最后再统一调用 `generate_render_commands()` 向 FHRE 交付结果

这类模式可以作为当前过渡实现被记录，但不应继续作为长期架构方向。

## 根原则

Wing 的根原则只有一条：先满足 FHRE 标准的声明式 ECS 架构，再讨论桌面功能。

出现设计取舍时，优先级顺序应固定为：

1. 是否符合 FHRE 的声明式 ECS 模型
2. 是否复用了 FHRE 已存在的能力，而不是重复实现
3. 是否符合 FHRE 文档里已经说明的限制、差异和默认行为
4. 最后才是桌面功能本身是否方便表达

如果某个 Wing 方案虽然更直观，但会破坏声明式 ECS、绕过 FHRE 生命周期、复制 FHRE 已有能力，或者忽略 FHRE 已知限制，那么该方案应被视为错误方向。

## FHRE 对齐基线

Wing 的目标设计必须严格以 `docs/ARCHITECTURE.md` 为准，不能自定义第二套运行时标准。

当前需要遵守的 FHRE 基线如下：

1. 架构基线：采用 `App` 下的 `MainWorld + RenderWorld` 双世界结构
2. 帧基线：遵守 `Startup -> PreUpdate -> Update -> PostUpdate -> entity_sync_system() -> extractors.run() -> execute_render()` 的生命周期
3. 输入基线：平台输入先进入 FHRE `InputPlugin::bridge()`，再映射为 `ButtonInput<T>`、`MousePosition`、`Pointer<E>` 等 ECS 资源与事件
4. 提取基线：渲染数据通过 extractor 从 `MainWorld` 复制到 `RenderWorld`，再进入 queue 阶段生成渲染命令
5. 平台基线：platform 层只负责呈现窗口、framebuffer、原始输入采集和运行循环承载
6. 视图基线：必须服从 FHRE 的默认 3D 相机和 `PrimaryScreen` 幕布系统，不能把 Wing 定义成绕开相机/幕布的平面 GUI 引擎
7. 能力边界基线：凡是 FHRE 已提供的能力，Wing 一律复用，不重复实现
8. 限制基线：Wing 的设计必须接受 FHRE 当前的限制与差异，而不是在 Wing 内部偷偷补一套替代机制

`apps/examples/fhre/rust` 已经给出了当前最直接的模板形式：

- 在 `lib.rs` 中创建 `App`
- 通过 `add_plugins()`、`insert_resource()`、`add_systems()` 装配系统
- 通过 `add_extractor()` 声明 extract / queue 链路
- 最终调用 `app.run(&mut window, &input_plugin, ...)`

Wing 的设计必须尽量与这套模板同构，而不是另外定义一条平行主线。

### FHRE 限制对 Wing 的直接约束

`docs/ARCHITECTURE.md` 已经给出了 FHRE 当前的限制和与 Bevy 的差异，Wing 必须接受这些边界。

当前至少要明确下面几点：

- FHRE 当前只有 `MainWorld + RenderWorld`，不是多 `SubApp` 架构；Wing 不应自行抽象出额外并行 App 体系
- Query 能力遵守 FHRE 当前实现，文档已明确 `Option<&T>`、`AnyOf<...>`、`Has<T>` 等不支持；Wing 的 ECS 设计不能默认依赖这些查询形式
- 输入、同步、提取、渲染的自动化责任边界以 FHRE 文档为准；Wing 不应用一层自定义 runtime 覆盖它
- 软件渲染、纹理上传、资源管理等通用能力由 FHRE 提供；Wing 只组织桌面语义，不复制图形引擎能力

## 非目标

为了避免实现方向继续跑偏，下面这些内容应被视为 Wing 的明确非目标：

- 不实现属于 FHRE 职责的 `App` 主循环与帧调度
- 不实现属于 FHRE 职责的第二套 `MainWorld / RenderWorld` 或并行 world 管理器
- 不实现属于 FHRE 职责的实体同步逻辑来替代 `entity_sync_system()`
- 不实现属于 FHRE 职责的通用 extract / queue 基础设施
- 不实现属于 FHRE 职责的输入桥接、统一按钮输入资源、统一指针事件模型
- 不实现属于 FHRE 职责的 `Camera` / `PrimaryScreen` / `View` 视图体系
- 不实现属于 FHRE 职责的通用纹理资源管理、GPU 纹理缓存抽象或软件渲染后端
- 不把 Wing 演化成与 FHRE 对等的 GUI 引擎、渲染框架或平台 runtime

Wing 真正应实现的内容只有桌面语义本身：

- 桌面场景组织
- 窗口生命周期和交互
- 任务栏与启动器语义
- 控件语义、布局语义、焦点语义、文本输入语义
- 把这些桌面语义映射进 FHRE 的组件、资源、系统和 extractor

## 相机与幕布约束

根据 `docs/ARCHITECTURE.md`，FHRE 是纯 3D 引擎，所谓 2D UI 只是处于幕布平面上的 3D 对象特例。

这条规则对 Wing 是强约束，不是可选建议。

Wing 作为桌面系统运行在 FHRE 之上时，必须遵守下面几点：

1. Wing 里的桌面、窗口、按钮、文本区域、本质上都属于 FHRE `MainWorld` 中的 3D 对象或其语义封装
2. 默认桌面 UI 应位于 `PrimaryScreen` 对应的幕布平面上，也就是默认 `z=0` 平面
3. 默认显示结果来自 `Camera + PrimaryScreen -> View` 的投影，而不是来自一套绕开相机的直接 2D blit 逻辑
4. UI 坐标虽然可按幕布像素坐标组织，但它们仍然受 FHRE 的 3D 世界、`Transform`、相机投影和视图提取规则约束
5. 若未来需要桌面层叠、前后景、3D 桌面特效，也应优先通过调整 3D 空间位置、变换和提取规则实现，而不是引入第二套坐标系统

### 默认空间语义

参考 FHRE 文档中的默认行为，Wing 应默认采用下面的空间语义：

```text
Camera position: (cx, cy, 600)
Camera target:   (cx, cy, 0)
PrimaryScreen:   centered at (cx, cy, 0)
Desktop UI:      placed on canvas plane z=0
```

这意味着：

- 桌面背景、任务栏、窗口外框、控件默认都应理解为“位于幕布平面上的桌面对象”
- 窗口层级关系虽然表现得像 2D stacking，但其底层仍应兼容 FHRE 的 3D 空间表达
- `Transform` 仍是统一变换入口，不应为 Wing 再定义一套脱离 FHRE 的位置/缩放主模型

### 对 Wing 设计的直接影响

- Wing 不应假设自己拥有独立于 `Camera` 和 `PrimaryScreen` 的最终屏幕坐标系
- Wing 的 layout / hit-test / focus 行为应建立在 FHRE 提供的幕布像素语义之上，而不是绕开视图系统
- Wing 的 extractor 需要从桌面 ECS 状态中提取“位于幕布平面上的对象”到 `RenderWorld`
- 如果使用文本、图标、窗口装饰，它们也应视作对幕布平面内容的提取和排队，而不是另起一条屏幕叠加通道
- 如果 FHRE 已经有统一的 `Transform`、事件、资源、提取方式，Wing 就不能再定义另一套同职能机制

## 当前状态

### 已完成

- `apps/wing/rust` 已接入 NuttX apps 构建系统
- `apps/examples/wing/rust` 已接入 NuttX examples 构建系统
- `nuttx/boards/sim/sim/sim/configs/wing/defconfig` 已新增
- `nuttx/wing_build.sh` 已可完成 `sim:wing` 的完整构建
- `wing_rust` 已成功注册为内建命令并完成链接
- `./nuttx/wing_build.sh` 已实测构建通过

### 当前可运行能力

- 桌面背景与图标区
- 任务栏
- 启动器
- 多窗口管理
- 窗口聚焦
- 窗口拖动
- 窗口最小化 / 最大化 / 关闭
- 窗口内 Widget 树渲染与事件分发
- 基础文本编辑输入
- 鼠标移动 / 点击 / 拖动 / 滚轮
- 键盘输入到 `TextEditCommand`

### 当前内置示例应用

- `Calculator`
- `Files`
- `Terminal`
- `Settings`
- `Gallery`

这些应用当前仍是 Wing 内部创建的桌面窗口内容，不是独立进程模型。

## 目录结构

```text
apps/wing/
├── Kconfig
├── Make.defs
├── Makefile
├── clang/
│   ├── include/wing/wing.h
│   └── src/wing.c
└── rust/
    ├── Kconfig
    ├── Make.defs
    ├── Makefile
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── desktop/
        │   └── mod.rs
        ├── icon/
        │   └── mod.rs
        ├── launcher/
        │   └── mod.rs
        ├── taskbar/
        │   └── mod.rs
        ├── theme/
        │   └── mod.rs
        ├── wallpaper/
        │   └── mod.rs
        ├── widgets/
        │   └── mod.rs
        └── window/
            └── mod.rs

apps/examples/wing/
├── Kconfig
├── Make.defs
├── Makefile
└── rust/
    ├── Kconfig
    ├── Make.defs
    ├── Makefile
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── platform/
            ├── mod.rs
            ├── framebuffer.rs
            ├── runner.rs
            └── input/
                └── mod.rs
```

## 当前实现概览

下面这些模块说明描述的是当前代码组织方式，不代表目标最终形态。

从 FHRE 标准角度看，当前实现最大的问题不是功能缺失，而是桌面状态、输入处理、布局推进和渲染准备仍主要围绕 `Wing` 中心对象展开，没有像 FHRE demo 那样把行为直接拆进 ECS 的资源、组件、系统和 extractor。

## 模块说明

### `apps/wing/rust/src/lib.rs`

`Wing` 当前仍是桌面壳总控对象，统一持有：

- `WindowManager`
- `Desktop`
- `Taskbar`
- `AppLauncher`
- `WingTheme`

当前核心接口包括：

- `Wing::new(width, height)`
- `Wing::init()`
- `Wing::resize(width, height)`
- `Wing::update(delta_time)`
- `Wing::handle_mouse_move(position)`
- `Wing::handle_click(position)`
- `Wing::handle_mouse_release(position)`
- `Wing::handle_drag(delta)`
- `Wing::handle_scroll(position, delta)`
- `Wing::handle_text_edit(command)`
- `Wing::generate_render_commands()`

### `window/`

负责窗口系统与窗口管理：

- `Window`
- `WindowManager`
- `WindowState`
- `WindowWidgetEvent`

当前已经实现：

- 创建应用窗口
- 焦点切换
- Z 顺序管理
- 标题栏拖动
- 最小化 / 最大化 / 关闭
- 窗口内部 Widget 命中测试与事件收集

### `widgets/`

负责控件种类、树结构、布局、交互和渲染命令生成。

这部分能力未来应逐步从“树对象递归调度”收敛到“可被 ECS 系统消费和提取的数据模型”。

当前关键类型：

- `WidgetKind`
- `WidgetNode`
- `WidgetTreeNode`
- `WidgetLayout`
- `WidgetEvent`
- `WidgetAction`
- `TextEditCommand`

当前已经具备：

- 树形 Widget 结构
- `Free / Vertical / Horizontal / Grid` 布局
- 递归渲染
- 递归 hit-test
- 悬停 / 按压 / 点击 / 选择变化等事件
- 文本编辑基础命令
- 不同示例应用对应的默认 Widget 树

### `desktop/`

负责桌面图标区、图标布局、图标点击与桌面层渲染。

### `launcher/`

负责应用启动器与应用清单。

当前默认应用定义在 `launcher/mod.rs` 中，启动器仍是桌面壳内部数据结构，不依赖独立进程装配。

### `taskbar/`

负责任务栏布局、启动器按钮、窗口按钮同步和交互。

### `theme/`

负责 `ThemePalette` 与 `WingTheme`。

当前内建主题：

- `WingTheme::aurora()`
- `WingTheme::dusk()`

### `wallpaper/` 和 `icon/`

负责桌面背景与图标的表现层数据和渲染命令输出。

## 当前运行链路

当前 Wing 示例的实际运行路径为：

```text
NuttX sim:wing
  -> wing_rust builtin
  -> apps/examples/wing/rust/src/lib.rs
  -> PlatformInputPlugin
  -> WingDesktopPlugin
  -> WingRuntime
  -> wing_input_system()
  -> wing_update_system()
  -> Wing
  -> generate_render_commands()
  -> apps/wing/rust/src/extract/
  -> FHRE RenderCommand
  -> SoftwareBackend
  -> Framebuffer / X11
```

其中当前职责划分为：

- `apps/examples/wing/rust/src/lib.rs` 负责启动入口、平台资源注入与 `app.run(...)`
- `platform/` 负责 framebuffer、输入和运行时桥接
- `WingDesktopPlugin` 负责注册过渡 runtime、systems 和 extractor
- `apps/wing/rust/src/extract/` 负责把 Wing 内部生成的过渡渲染命令桥接到 FHRE 渲染链路

这条链路说明的是“当前实现”，不是后续推荐继续扩展的长期形态。

从架构角度看，当前链路的关键问题不是它不能运行，而是它把桌面逻辑的主导权放在 `Wing` 对象和桥接层上，而不是 FHRE 的 ECS 调度模型上。

严格按 FHRE 标准看，当前这条链路只是“可运行的过渡链路”，还不是“标准 Wing 链路”。

## 构建与运行

### 推荐方式

在 `nuttx/` 目录下：

```bash
./wing_build.sh
```

该脚本会执行：

1. `make distclean`
2. `./tools/configure.sh sim:wing`
3. `make -j`

构建成功后运行：

```bash
./nuttx
```

在 NSH 中执行：

```bash
wing_rust
```

### FHRE 对照构建

如果要单独构建 FHRE demo：

```bash
./fhre_build.sh
```

## NuttX 集成现状

当前已完成以下接线：

- `apps/Kconfig` 已包含 `apps/wing/Kconfig`
- `apps/wing/rust` 已提供 `Kconfig / Make.defs / Makefile`
- `apps/examples/wing/rust` 已提供 `Kconfig / Make.defs / Makefile`
- `apps/examples/wing/Make.defs` 已正确包含 `rust/Make.defs`
- `sim:wing` 已启用：
  - `CONFIG_FHRE_RUST=y`
  - `CONFIG_WING_RUST=y`
  - `CONFIG_EXAMPLES_WING_RUST=y`

## 目标架构调整

Wing 后续不应继续沿着“自管 2D GUI，再额外吐给 FHRE `RenderCommand`”的方向扩展。

目标应改为：把 Wing 明确收敛成一个严格沿用 FHRE 模板的声明式 ECS 桌面应用。

这是本文档最重要的结论。

### 参考模板

目标模板直接参考：

- `docs/ARCHITECTURE.md`
- `apps/examples/fhre/rust/src/lib.rs`
- `apps/examples/fhre/rust/src/extract.rs`

也就是说，Wing 应当尽量采用和 FHRE 示例一致的结构：

- `MainWorld` 里表达桌面、窗口、任务栏、启动器、控件树、焦点、输入状态等桌面语义
- `RenderWorld` 里只保留提取后的渲染态与绘制命令
- 输入通过 FHRE 的资源与事件系统进入 `MainWorld`
- 桌面 UI 由 ECS 组件和系统声明，不再由 `Wing` 单体对象手工维护一套独立 GUI 树后统一导出渲染命令

换句话说，Wing 未来应更像“一个桌面 ECS 应用模板”，而不是“一个内部先跑完 GUI，再把结果塞给 FHRE 的适配器”。

### 目标装配方式

参考 `apps/examples/fhre/rust/src/lib.rs`，标准 Wing 入口最终应收敛成如下形态：

```rust
let mut app = App::new(width, height);

app.add_plugins(DefaultPlugins)
    .add_plugin(WingDesktopPlugin)
    .insert_resource(ButtonInput::<KeyCode>::default())
    .insert_resource(ButtonInput::<MouseButton>::default())
    .insert_resource(MousePosition::default())
    .insert_resource(WingDesktopState::default())
    .insert_resource(FocusState::default())
    .insert_resource(WindowManagerState::default())
    .add_systems(Startup, declare_system!(setup_wing_desktop; Commands, Res<PrimaryScreen>))
    .add_systems(PreUpdate, declare_system!(wing_pointer_picking_system; Res<MousePosition>, Res<ButtonInput<MouseButton>>, Query<&Transform>, Query<&PickableBounds>, Query<&Pickable>, ResMut<HoverMap>, ResMut<PreviousHoverMap>, ResMut<PointerPress>, ResMut<PointerLocation>, ResMut<Events>))
    .add_systems(PreUpdate, declare_system!(wing_focus_system; Res<Events>, Query<&mut WindowFocus>, Query<&mut WidgetFocus>))
    .add_systems(PreUpdate, declare_system!(wing_input_dispatch_system; Res<ButtonInput<KeyCode>>, Res<Events>, ResMut<TextInputState>, Query<&mut TextField>))
    .add_systems(Update, declare_system!(wing_window_management_system; Commands, ResMut<WindowManagerState>, Query<&mut WindowState>))
    .add_systems(Update, declare_system!(wing_layout_system; Res<PrimaryScreen>, Query<&mut LayoutNode>, Query<&mut Transform>))
    .add_systems(Update, declare_system!(wing_widget_state_system; Res<Events>, Query<&mut Button>, Query<&mut ScrollArea>));

app.add_extractor(extract::extract_view)
   .add_extractor(extract::extract_wing_desktop)
   .add_extractor(extract::extract_wing_windows)
   .add_extractor(extract::extract_wing_widgets)
   .add_extractor(extract::queue_wing_primitives)
   .add_extractor(extract::queue_wing_text)
   .add_extractor(extract::queue_wing_icons);
```

上面不是要求一字不差照抄，而是要求 Wing 的主干结构必须与 FHRE demo 同型：

- `App` 是入口
- plugin / resource / system / extractor 是装配单元
- `MainWorld` 承载状态与行为
- `RenderWorld` 承载提取后的渲染态

### 目标数据流

```text
NuttX sim:wing
  -> wing_rust builtin
  -> apps/examples/wing/rust/src/lib.rs
  -> App::new(width, height)
  -> add_plugins(DefaultPlugins + WingDesktopPlugin)
  -> insert_resource(...)
  -> add_systems(Startup / PreUpdate / Update / PostUpdate)
  -> MainWorld
     - Desktop entities
     - Window entities
     - Widget entities
     - Layout / Focus / Selection / TextInput resources
     - entities placed on the PrimaryScreen canvas plane by default
     - systems: setup, input, focus, layout, window management, widget state
  -> entity_sync_system()
  -> extractors.run()
     - extract_view
     - extract_wing_desktop
     - extract_wing_windows
     - extract_wing_widgets
     - queue_wing_primitives
     - queue_wing_text
     - queue_wing_icons
  -> RenderWorld
     - extracted desktop quads
     - extracted text / icons / decorations
     - render commands / phase items
  -> SoftwareBackend
  -> Framebuffer / X11
```

### 目标输入链路

严格按 FHRE 输入模型，Wing 应采用如下输入路径：

```text
Presentation Window / NuttX input devices
  -> WindowInputEvents
  -> InputPlugin::bridge()
  -> ButtonInput<MouseButton>
  -> ButtonInput<KeyCode>
  -> MousePosition
  -> picking / pointer event systems
  -> focus / click / drag / resize / text edit systems
```

这意味着：

- Wing 不应维护一套绕开 FHRE 输入桥接的主输入模型
- 命中检测优先基于 `Transform + PickableBounds + Pickable`
- 点击、悬停、焦点切换、按钮态变化都应尽量建立在 `Pointer<E>` 事件之上

### 目标桌面 ECS 分层

为了和 FHRE demo 的组件提取模式一致，Wing 建议按下面方式分层：

- 组件：`DesktopRoot`、`DesktopIcon`、`WindowFrame`、`WindowChrome`、`WidgetNode`、`TextField`、`Button`、`ScrollArea`、`TaskbarItem`、`LauncherEntry`
- 资源：`WingDesktopState`、`WindowManagerState`、`FocusState`、`SelectionState`、`TextInputState`、`ThemeState`
- 系统：`setup_wing_desktop`、`wing_pointer_picking_system`、`wing_focus_system`、`wing_drag_system`、`wing_window_management_system`、`wing_layout_system`、`wing_widget_state_system`
- extractor：`extract_wing_desktop`、`extract_wing_windows`、`extract_wing_widgets`
- queue：`queue_wing_primitives`、`queue_wing_text`、`queue_wing_icons`

### 第一版 ECS 清单

下面这份清单不是最终 API，而是第一版落地时建议采用的职责拆分。

#### 组件

- `DesktopRoot`: 标记桌面根实体
- `DesktopWallpaper`: 桌面背景表现
- `DesktopIcon`: 桌面图标语义
- `LauncherPanel`: 启动器面板语义
- `Taskbar`: 任务栏根实体
- `TaskbarItem`: 任务栏中的窗口或应用入口
- `WindowFrame`: 窗口实体主体
- `WindowChrome`: 标题栏、控制按钮等窗口边框语义
- `WindowContentRoot`: 窗口内容根节点
- `WindowState`: 最小化、最大化、活动态、可拖拽等状态组件
- `WindowFocus`: 窗口焦点状态
- `WidgetNode`: 通用控件节点标记
- `WidgetLayoutNode`: 布局节点语义
- `Button`: 按钮控件语义
- `TextField`: 文本输入控件语义
- `ScrollArea`: 滚动区域语义
- `Label`: 纯文本显示语义
- `IconGlyph`: 图标显示语义
- `PickableBounds`: 命中检测边界
- `Pickable`: 可交互标记，遵守 FHRE `Pickable::DEFAULT` 语义
- `Transform`: 统一 3D 位置、旋转、缩放入口

#### 资源

- `WingDesktopState`: 桌面级共享状态
- `WindowManagerState`: 窗口注册、层级、活动窗口、拖动事务
- `LauncherState`: 启动器开合和当前条目状态
- `TaskbarState`: 任务栏展示状态
- `FocusState`: 当前焦点窗口和控件
- `SelectionState`: 文本或列表选择状态
- `TextInputState`: 文本编辑上下文
- `LayoutInvalidation`: 需要重算布局的标记资源
- `ThemeState`: 主题与调色板
- `DesktopMetrics`: 桌面布局参数与边距

#### 系统

- `setup_wing_desktop`: 启动时生成桌面根实体、任务栏、默认窗口
- `setup_wing_assets`: 注册桌面需要的图标、纹理、字体相关资源句柄
- `wing_pointer_picking_system`: 基于 FHRE 输入资源和命中边界生成指针相关事件
- `wing_focus_system`: 基于 `Pointer<E>` 和键盘输入推进焦点状态
- `wing_drag_system`: 处理窗口拖动与拖拽事务
- `wing_window_management_system`: 处理打开、关闭、最小化、最大化、激活、层级
- `wing_launcher_system`: 处理启动器开合和应用启动语义
- `wing_taskbar_sync_system`: 同步窗口集合到任务栏项
- `wing_layout_system`: 处理桌面、窗口和控件布局
- `wing_text_input_system`: 处理文本输入和编辑命令
- `wing_widget_state_system`: 推进按钮、滚动区、输入框等控件状态
- `wing_theme_apply_system`: 把主题状态映射到表现组件或提取态

#### extractor / queue

- `extract_view`: 直接沿用 FHRE 模板视图提取
- `extract_wing_desktop`: 提取桌面背景、图标、任务栏背景等
- `extract_wing_windows`: 提取窗口边框、标题栏、阴影、内容区域
- `extract_wing_widgets`: 提取按钮、文本、图标、输入框、滚动条等控件表现
- `queue_wing_primitives`: 输出矩形、边框、填充、多边形等基础图元
- `queue_wing_text`: 输出文本绘制命令或文本 phase item
- `queue_wing_icons`: 输出图标和装饰元素

#### 建议模块落点

- `apps/wing/rust/src/components/`: 桌面 ECS 组件定义
- `apps/wing/rust/src/resources/`: 共享资源定义
- `apps/wing/rust/src/systems/`: 当前过渡系统目录，已拆出 `input.rs` 与 `update.rs`
- `apps/wing/rust/src/extract/`: 当前过渡 extractor 目录，现含 `shell.rs`，后续继续拆成 `extract_wing_*` / `queue_wing_*`
- `apps/examples/wing/rust/src/lib.rs`: 示例入口与 App 装配

### 目标目录结构

为了让 `apps/wing/rust` 和 `apps/examples/wing/rust` 真正落到 FHRE 风格，建议目录逐步收敛成下面这种形态：

```text
apps/wing/rust/src/
├── lib.rs                    # WingDesktopPlugin 导出与模块装配
├── extract/
│   ├── mod.rs                # 当前过渡 extractor 导出
│   └── shell.rs              # 当前过渡 queue_wing_shell
├── input.rs                  # 过渡输入资源类型
├── systems/
│   ├── mod.rs                # 当前过渡系统导出
│   ├── input.rs              # 当前过渡 input system
│   └── update.rs             # 当前过渡 update system
├── components/
│   ├── mod.rs
│   ├── desktop.rs            # DesktopRoot, DesktopWallpaper, DesktopIcon
│   ├── window.rs             # WindowFrame, WindowChrome, WindowState, WindowFocus
│   ├── taskbar.rs            # Taskbar, TaskbarItem
│   ├── launcher.rs           # LauncherPanel, LauncherEntry
│   └── widgets.rs            # WidgetNode, Button, TextField, ScrollArea, Label
├── resources/
│   ├── mod.rs
│   ├── runtime.rs            # 当前过渡 WingRuntime
│   ├── desktop.rs            # WingDesktopState, DesktopMetrics
│   ├── window.rs             # WindowManagerState, drag transaction state
│   ├── input.rs              # FocusState, SelectionState, TextInputState
│   └── theme.rs              # ThemeState
└── plugin.rs                 # add_systems / resources / plugin assembly

apps/examples/wing/rust/src/
├── lib.rs                    # App::new + plugin/resource assembly
└── platform/
    ├── mod.rs
    ├── framebuffer.rs        # framebuffer/window adapter
    ├── runner.rs             # create_window / window wrapper
    └── input/
        └── mod.rs
```

### 文件职责约束

为了防止目录形态看起来像 ECS，实际又偷偷回到中心对象架构，下面这些职责边界应固定下来：

- `apps/wing/rust/src/lib.rs` 只负责导出 plugin、extract、input 和公共模块，不再继续扩大运行时装配职责
- `components/` 只放组件定义和紧邻组件的轻量语义类型，不放主业务调度
- `resources/` 只放共享资源和事务状态，不放渲染桥接逻辑
- `systems/` 当前负责过渡桌面行为推进，后续继续收敛成目标中的 `startup / input / window / layout / launcher / taskbar / theme`
- `plugin.rs` 负责统一注册资源和系统顺序，是 Wing crate 内部唯一的装配入口
- `apps/examples/wing/rust/src/lib.rs` 负责示例程序的 FHRE `App` 装配和平台资源注入，不再实现独立 GUI runtime
- `apps/wing/rust/src/extract/` 只负责从状态提取渲染态和排队，不负责桌面业务逻辑

### 设计约束

- Wing 是 FHRE 上层桌面应用，不是独立于 FHRE 的第二套 UI 引擎
- 声明式 ECS 是第一原则，任何便利性的面向对象封装都不能凌驾于它之上
- 桌面语义应以组件、资源、系统来表达，而不是集中在 `Wing::update()` 和 `Wing::generate_render_commands()` 里
- 渲染提取应从 ECS 状态生成，而不是先维护一套自定义 GUI 中间表示再转译
- 示例应用也应优先表达为 ECS scene / app setup，而不是硬编码默认 Widget 树拼装
- 生命周期必须服从 FHRE `Startup / PreUpdate / Update / PostUpdate / entity_sync / extract / render` 顺序
- 输入桥接必须复用 FHRE `ButtonInput<T>`、`MousePosition`、`Pointer<E>` 机制
- extractor 设计必须参考 `apps/examples/fhre/rust/src/extract.rs` 的 extract/queue 分相模式
- 桌面对象默认位于 `PrimaryScreen` 对应的幕布平面，受 `Camera + PrimaryScreen -> View` 规则约束
- FHRE 已支持的能力不得在 Wing 中重复实现
- FHRE 已说明的限制和差异必须直接体现在 Wing 设计上，不能靠 Wing 私自规避

### 必须避免的方向

- 不再新增以 `Wing` 为中心的总控状态聚合逻辑
- 不再继续把更多交互行为固化到对象方法分发链里
- 不再把 `generate_render_commands()` 发展成事实上的主渲染前端
- 不再把 `extract.rs` 当作长期承载 GUI 语义翻译的核心层
- 不再新增绕开 FHRE `App` 生命周期的自定义桌面主循环
- 不再新增绕开 FHRE 输入资源与指针事件系统的并行输入通道
- 不再为同步、提取、输入、相机、资源管理、软件渲染实现 Wing 自己的替代版本
- 不再假设可使用 FHRE 文档已明确不支持的 ECS 查询或运行时特性作为 Wing 基础

### 建议的目标边界

- `MainWorld` 负责桌面语义、窗口生命周期、布局、焦点、交互状态
- `RenderWorld` 负责提取后的绘制态、材质/纹理句柄、文本/图标/装饰的渲染数据
- platform 层只负责输入采集、framebuffer 对接和运行循环承载
- Wing crate 负责定义桌面领域组件、资源、系统和插件装配，而不是维护一个中心化桌面对象
- `apps/examples/wing/rust` 应像 `apps/examples/fhre/rust` 一样作为标准示例入口，而不是额外包一层独立 GUI runtime
- 当前过渡实现中，示例入口已收敛为平台壳，运行时装配转由 `WingDesktopPlugin` 承担
- FHRE 负责图形引擎能力，Wing 只负责桌面应用语义

### `apps/examples/wing/rust` 目标入口骨架

`apps/examples/wing/rust` 最终应尽量和 `apps/examples/fhre/rust` 同型，入口建议收敛成下面这种结构：

```rust
pub extern "C" fn wing_rust_main(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut window = match platform::runner::create_window() {
        Some(window) => window,
        None => return 0,
    };

    let (width, height) = window.dimensions();
    let mut app = App::new(width, height);

    app.add_plugins(DefaultPlugins)
        .add_plugin(WingDesktopPlugin)
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(MousePosition::default())
        .insert_resource(WingDesktopState::default())
        .insert_resource(WindowManagerState::default())
        .insert_resource(FocusState::default())
        .insert_resource(ThemeState::default())
        .add_systems(Startup, declare_system!(setup_wing_assets; ResMut<Assets<Image>>, ResMut<Events>))
        .add_systems(Startup, declare_system!(setup_wing_desktop; Commands, Res<PrimaryScreen>))
        .add_systems(PreUpdate, declare_system!(wing_pointer_picking_system; Res<MousePosition>, Res<ButtonInput<MouseButton>>, Query<&Transform>, Query<&PickableBounds>, Query<&Pickable>, ResMut<HoverMap>, ResMut<PreviousHoverMap>, ResMut<PointerPress>, ResMut<PointerLocation>, ResMut<Events>))
        .add_systems(PreUpdate, declare_system!(wing_focus_system; Res<Events>, Query<&mut WindowFocus>, Query<&mut Button>, Query<&mut TextField>))
        .add_systems(PreUpdate, declare_system!(wing_text_input_system; Res<ButtonInput<KeyCode>>, Res<Events>, ResMut<TextInputState>, Query<&mut TextField>))
        .add_systems(Update, declare_system!(wing_window_management_system; Commands, ResMut<WindowManagerState>, Query<&mut WindowState>))
        .add_systems(Update, declare_system!(wing_launcher_system; Res<Events>, ResMut<LauncherState>))
        .add_systems(Update, declare_system!(wing_taskbar_sync_system; Res<WindowManagerState>, Query<&mut TaskbarItem>))
        .add_systems(Update, declare_system!(wing_layout_system; Res<PrimaryScreen>, ResMut<LayoutInvalidation>, Query<&mut WidgetLayoutNode>, Query<&mut Transform>))
        .add_systems(Update, declare_system!(wing_widget_state_system; Res<Events>, Query<&mut Button>, Query<&mut ScrollArea>, Query<&mut TextField>));

    app.add_extractor(extract::extract_view)
        .add_extractor(extract::extract_wing_desktop)
        .add_extractor(extract::extract_wing_windows)
        .add_extractor(extract::extract_wing_widgets)
        .add_extractor(extract::queue_wing_primitives)
        .add_extractor(extract::queue_wing_text)
        .add_extractor(extract::queue_wing_icons);

    let input_plugin = platform::PlatformInputPlugin::new(platform::framebuffer::InputAdapter);
    app.run(&mut window, &input_plugin, timing::FRAME_DELAY_MS);
    0
}
```

这个骨架表达的重点不是具体名字，而是以下结构必须稳定：

- 入口直接创建 FHRE `App`
- 通过 plugin / resource / systems / extractors 完成装配
- 不插入独立于 FHRE 的 `WingRuntime` 中心对象
- 不插入额外主循环
- 平台层只负责窗口和输入适配

## 最小迁移阶段

为了避免一次性重写，Wing 的代码迁移建议分阶段进行。

### Phase 1: 装配入口收敛

目标：先让 `apps/examples/wing/rust/src/lib.rs` 的入口结构对齐 FHRE demo。

这一阶段应完成：

- 保留现有可运行能力，但让入口改为 `App::new(...)` 主导
- 引入 `WingDesktopPlugin` 作为统一装配点
- 把 `ButtonInput<T>`、`MousePosition`、主题和桌面共享状态改为正式资源插入
- 保留旧实现时，也只能把旧逻辑作为过渡系统挂进 FHRE 生命周期，不能继续保留独立主循环

完成标志：

- `apps/examples/wing/rust/src/lib.rs` 不再以 `WingRuntime` 为主入口
- `app.run(&mut window, &input_plugin, ...)` 成为唯一主循环入口

### Phase 2: 中心对象拆解

目标：把当前 `Wing`、`WindowManager`、`WidgetTreeNode` 中的中心化状态下沉为组件和资源。

这一阶段应完成：

- 把窗口集合、活动窗口、拖动事务迁入 `WindowManagerState`
- 把焦点、选择、文本编辑上下文迁入资源
- 把窗口、任务栏、启动器、桌面图标改造成 ECS 实体和组件组合
- 把对象方法驱动的行为改写为系统驱动

完成标志：

- `Wing::update()` 不再是桌面行为主入口
- `Wing::handle_*()` 这类接口只剩过渡兼容价值，或被完全移除

### Phase 3: 提取链路收敛

目标：让渲染链路直接从 ECS 状态进入 `RenderWorld`，消减 `generate_render_commands()` 的中心职责。

这一阶段应完成：

- 新增 `extract_wing_desktop`
- 新增 `extract_wing_windows`
- 新增 `extract_wing_widgets`
- 新增 `queue_wing_primitives`、`queue_wing_text`、`queue_wing_icons`
- 逐步废弃“先生成自定义 GUI 渲染命令，再桥接到 FHRE”的路径

完成标志：

- `generate_render_commands()` 不再是渲染前必经步骤
- `apps/wing/rust/src/extract/` 过渡拆分为和 FHRE demo 同型的 `extract_wing_*` / `queue_wing_*` 主提取入口

### Phase 4: 旧架构清理

目标：删除已经失去主干职责的旧运行时抽象。

这一阶段应完成：

- 删除或彻底边缘化 `WingRuntime`
- 删除或彻底边缘化 `Wing` 总控对象
- 清理不再需要的桥接层和重复状态缓存
- 让目录结构和运行时职责与文档中的目标结构一致

完成标志：

- 桌面系统主干完全由 plugin / resource / system / extractor 组成
- 文档和代码结构不再出现“双主线”并存

## 与 FHRE 的关系

依据 `docs/ARCHITECTURE.md`，FHRE 是底层引擎与执行框架，Wing 应处在其上层：

- FHRE 负责 `App`、双世界 ECS、资源、提取、软件后端与 framebuffer 输出
- Wing 负责桌面壳语义、窗口系统、Widget 系统与桌面交互模型

需要特别注意的是：Wing 不应把桌面逻辑写进 FHRE 核心，但也不应在 FHRE 之外再长期维护一套平行的 GUI 调度框架。

更合适的边界是：

- FHRE 提供通用 ECS、输入、提取、渲染能力
- Wing 作为 FHRE 之上的桌面应用模板，使用这些能力组织自己的桌面系统
- Wing 特有的数据结构如果保留，也应逐步收敛成 ECS 组件/资源的薄封装，而不是主导运行时的中心对象模型

## 当前限制

- 仍是单进程桌面壳，不是应用进程隔离模型
- 应用窗口内容仍以内建示例 Widget 树为主
- 核心运行时仍偏向 `Wing` 单体对象驱动，而不是 FHRE 风格的声明式 ECS 驱动
- 当前仍存在 GUI 语义层与 FHRE 提取层之间的桥接式分层，长期会放大维护成本
- 没有文件系统浏览、终端执行、设置持久化等真实后端能力
- 没有完整的输入法、文本选择、剪贴板、窗口缩放边框等高级桌面能力
- Clang 版本 `apps/wing/clang` 仍基本是 stub

## 迁移进度

以下进度用于同步当前修改状态。

### 已完成

- 已明确 Wing 目标必须严格遵循 `docs/ARCHITECTURE.md`
- 已明确目标模板直接参考 `apps/examples/fhre/rust`
- 已在本文档中把 Wing 的目标运行链路改写为 FHRE 标准的 `App -> MainWorld -> entity_sync -> extractors -> RenderWorld -> SoftwareBackend`
- 已明确输入必须经由 FHRE `InputPlugin::bridge()`、`ButtonInput<T>`、`MousePosition`、`Pointer<E>`
- 已明确 Wing 不再以 `Wing` 中心对象和 `generate_render_commands()` 作为长期主线
- 已明确声明式 ECS 是 Wing 的最高优先级原则
- 已明确凡是 FHRE 已支持的能力，Wing 不重复实现
- 已明确 Wing 必须接受 FHRE 当前限制与差异
- 已补充 Wing 的明确非目标清单
- 已补充第一版 ECS 组件 / 资源 / 系统 / extractor 清单
- 已补充 `apps/examples/wing/rust` 对齐 FHRE demo 的目标入口骨架
- 已补充 `apps/wing/rust` / `apps/examples/wing/rust` 的目标目录结构与文件职责
- 已补充最小迁移阶段 `Phase 1 -> Phase 4` 计划
- 已在 `apps/wing/rust/src/plugin.rs` 新增 `WingDesktopPlugin` 与过渡 `WingRuntime`
- 已让 `apps/examples/wing/rust/src/lib.rs` 通过 `add_plugin(WingDesktopPlugin)` 组装运行时资源
- 已把过渡输入资源类型提升到 `apps/wing/rust/src/input.rs`
- 已让示例平台输入层通过 `wing` crate 复用 `ButtonInput`、`KeyCode`、`MouseButton`、`MouseWheel`
- 已在 `apps/wing/rust/src/systems/` 建立过渡 `wing_input_system`、`wing_update_system`
- 已将输入处理进一步拆分为 `wing_pointer_input_system`、`wing_shell_shortcut_system`、`wing_text_input_system`
- 已将运行时状态同步拆分为 `wing_shell_state_sync_system`、`wing_window_state_sync_system`
- 已在 `apps/wing/rust/src/resources/` 建立过渡资源模块，并把 `WingRuntime` 迁入 `resources/runtime.rs`
- 已在 `apps/wing/rust/src/extract/` 建立过渡 extractor 目录，并在 `extract/shell.rs` 中承载 `queue_wing_shell`
- 已让 `queue_wing_shell` 接入 `ThemeState`、`DesktopMetrics` 作为过渡渲染提取输入
- 已在 `apps/wing/rust/src/components/` 建立过渡组件目录
- 已在 `apps/wing/rust/src/components/` 落地首批过渡组件：`DesktopRoot`、`DesktopWallpaper`、`TaskbarRoot`、`TaskbarItem`、`LauncherPanel`、`LauncherEntry`、`WindowFrame`、`WindowFocus`、`WidgetNodeComponent`、`ButtonWidget`、`TextFieldWidget`、`ScrollAreaWidget`
- 已让 `WingDesktopPlugin` 统一注册 `WingRuntime`、过渡 systems 和 extractor
- 已将 `apps/examples/wing/rust/src/lib.rs` 收敛为平台窗口创建、基础输入资源注入和 `app.run(...)`
- 已删除示例侧废弃的 `apps/examples/wing/rust/src/extract.rs`

### 进行中

- 把当前 `apps/wing/rust` 的中心对象状态拆分为 ECS 组件 / 资源 / 系统边界
- 把当前窗口、任务栏、启动器、Widget 树语义映射为 FHRE 风格的 `MainWorld` 数据结构
- 规划和 `apps/examples/fhre/rust/src/extract.rs` 对齐的 Wing extractor / queue 分层
- 继续把原先集中在单个系统里的输入与状态写回拆成更小的系统边界
- 把文档中的第一版 ECS 清单逐步映射到实际代码目录和类型命名
- 把目标目录结构逐步映射到真实文件与模块边界
- 把 `queue_wing_shell` 过渡提取改造成 FHRE 风格的 `extract_wing_*` / `queue_wing_*` 链路
- 当前渲染提取仍依赖 `WingRuntime.wing.generate_render_commands()`，尚未直接从 ECS 状态生成 RenderWorld 数据
- 把 `components/` 从当前 `desktop / taskbar / launcher / window / widgets` 继续扩展为更接近目标结构的完整组件集合
- 把当前 `systems/` 继续扩展为更接近目标结构的 `startup / input / window / layout / launcher / taskbar / theme`
- 把 `resources/` 从当前过渡 `runtime.rs` 继续扩展为 `desktop / window / input / theme`

### 下一步

- 把第一版 ECS 清单转换成实际代码模块和类型定义
- 把 `apps/examples/wing/rust` 的入口继续收敛为更接近 FHRE demo 的装配形式
- 在代码层逐步消减 `Wing::update()` 和 `Wing::generate_render_commands()` 的主导作用
- 把 `queue_wing_shell` 从当前运行时代理改造成真正的 `extract_wing_*` / `queue_wing_*` 多阶段链路
- 把窗口焦点、拖动、文本输入的状态写回继续从 `WingRuntime` 桥接迁向 ECS 原生资源与组件
- 继续扩展 `apps/wing/rust/src/components/`，补齐 launcher/widgets 等目标组件边界
- 把当前 `systems/` 和 `extract/` 继续拆成与目标目录一致的多文件模块

## 迁移原则

迁移时应遵守下面几条原则：

1. 先迁移状态归属，再迁移渲染表达；不要先做表面 API 改名
2. 先把输入、焦点、窗口管理这些核心行为改成系统驱动，再处理复杂控件细节
3. 保留可以帮助过渡的领域模型，但要明确它们不能继续成为运行时主入口
4. 每次迁移都应保证 `sim:wing` 仍可运行，避免一次性重写
5. 新代码优先写成组件、资源、系统、plugin 装配，不再扩张中心对象接口

## 建议的下一步开发顺序

建议继续按下面顺序推进，把当前实现收敛到目标架构，而不是继续扩散现有桥接层：

1. 先定义 Wing 对应的 ECS 组件、资源、系统边界，替代 `Wing` 总控对象里的中心化状态
2. 把窗口管理、桌面图标、任务栏、启动器、焦点与文本输入逐步迁入 `MainWorld`
3. 把现有 Widget 树表示收敛为 ECS 友好的 scene / node / layout 数据模型
4. 让渲染提取直接从 ECS 状态生成 `RenderWorld` 数据，而不是继续扩大 `generate_render_commands()` 的职责
5. 把示例应用从“默认 Widget 树”提升为更清晰的 app setup / app scene 结构
6. 在 ECS 架构稳定后，再补滚动区域、列表、下拉框、按钮矩阵等控件细节
7. 最后再评估应用生命周期、进程模型、持久化、文件浏览、终端后端等系统能力

## 当前最务实的开发重点

如果目标是继续把 Wing 变成“可持续开发的桌面壳”，当前最应该优先做的是：

- 明确哪些现有类型应保留为领域模型，哪些必须下沉为 ECS 组件 / 资源
- 先把输入、焦点、窗口激活、拖动、最小化等行为改成系统驱动，而不是对象方法驱动
- 明确 `WindowManager`、`WidgetTreeNode`、`AppLauncher` 与 ECS world 之间的过渡关系，避免再形成新的中心对象
- 在 `apps/examples/wing/rust` 中补最小运行验证说明，确保后续架构迁移始终有可回归入口

这一步完成后，再继续推进更复杂的桌面特性，风险会更低。

## 一句话结论

Wing 应从“自管 2D GUI 后桥接到 FHRE”的实现，收敛为“严格运行在 FHRE 双世界 ECS 模板之上的声明式桌面应用”。
