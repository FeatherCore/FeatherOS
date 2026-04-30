# Wing + FHRE 对接架构基线（历史参考）

> 当前新方向是将 FHRE 合入 Wing，形成单 crate 的轻量声明式 ECS UI/渲染系统。
> 新架构基线见 `docs/WING_UNIFIED_ARCHITECTURE.md`。
> 本文保留为旧 `FHRE + Wing` 分离式对接方案的历史参考。

本文用于收束 `Wing` 和 `FHRE` 的当前架构关系，作为后续实现、重构和文档更新的共同基线。

`docs/ARCHITECTURE.md` 仍然是 FHRE 的核心架构基线；本文只描述 Wing 如何建立在 FHRE 之上，以及 `apps/fhre`、`apps/wing`、`apps/examples/fhre`、`apps/examples/wing` 四个目录各自承担什么责任。

## 当前结论

Wing + FHRE 的总体方向已经清楚：`FHRE` 是嵌入式友好的纯 3D ECS 渲染引擎，`Wing` 是运行在 FHRE 之上的移动/手表壳层。

但现有文档仍有几处需要明确收口：

| 问题 | 当前情况 | 本文确定的基线 |
|------|----------|----------------|
| 坐标描述不一致 | `wing_implementation.md` 里曾写原点在屏幕中心，但代码使用屏幕像素坐标 | 当前以左上角 `(0, 0)`、右下角 `(W, H)` 的屏幕像素坐标为准，`center_x = W * 0.5` |
| `2D UI` 语义容易误读 | Wing 最终 queue 的是 `DrawRect/DrawText` 等 2D primitive | Wing 控件本体仍必须是 3D ECS 实体；当前 2D primitive 只是渲染后端的最小输出形式 |
| 输入资源归属不够清楚 | FHRE 提供 `WindowInputEvents/InputPlugin/MousePosition`，Wing 和 demo 自定义 `ButtonInput<T>`、`KeyCode`、`MouseButton` | FHRE 负责输入桥接接口；具体输入枚举和按钮状态资源属于应用/壳层适配层 |
| 动态内容 API 与实体生命周期未完全闭合 | `ShellContent` 已支持 add/remove API，但 Startup 只按初始内容生成实体 | 运行时新增内容必须配套实体同步系统，不能只改资源 |
| 平台窗口和 shell surface 容易混淆 | 都可能被口语称为“窗口” | `Window` 只表示平台呈现窗口；Wing 中只使用 `Surface/AppSurface/Panel/Card` 等壳层概念 |
| 文档版本来源分散 | `docs/ARCHITECTURE.md`、`apps/fhre/ARCHITECTURE.md`、代码版本号存在差异 | 当前以 `docs/ARCHITECTURE.md` 和实际代码为准，旧局部文档只能作为历史参考 |

## 一句话架构

```text
Wing = FHRE 上的一组 shell 组件、资源、系统和 extractor，
       不是 FHRE 旁边的 GUI runtime。
```

FHRE 管“引擎如何推进一帧、如何维护 ECS、如何同步到渲染世界、如何执行渲染”。

Wing 管“移动/手表壳层有哪些实体、状态、手势、布局和主题，以及如何把这些壳层实体提取成 FHRE 可渲染的数据”。

平台示例管“如何在 NuttX SIM 上打开 framebuffer、读取输入设备、映射按键和鼠标按钮、启动 app”。

## 当前构建与运行入口

当前 Wing 桌面通过 NuttX SIM 配置运行：

```bash
cd /home/uan-gpd/codes/FeatherOS/nuttx
./wing_build.sh
./nuttx
```

进入 NSH 后执行：

```text
nsh> wing_rust
```

`wing_build.sh` 当前执行的核心步骤是：

```text
make distclean
./tools/configure.sh sim:wing
make -j
```

因此，`apps/examples/wing` 是被 NuttX 应用入口 `wing_rust` 拉起的示例装配层；它再创建 FHRE `App`、加载 `WingShellPlugin`，最后调用 `app.run(...)` 进入 FHRE 主循环。

## 目录责任

### `apps/fhre`

FHRE 核心库。它应保持平台无关、应用无关、壳层无关。

主要任务：

- 提供 `App` 主循环和 `update_and_render()`。
- 提供 `MainWorld`、`RenderWorld`、`Entity`、`Component`、`Query`、`Commands`、`Resource`。
- 提供 `Startup/PreUpdate/Update/PostUpdate/Last` 调度阶段。
- 提供 `Plugin` / `PluginGroup`。
- 提供 `Window` trait 和 `InputPlugin` trait，但不绑定具体平台。
- 提供 `MousePosition` 等基础输入资源。
- 提供 picking 基础设施：`Pickable`、`PickableBounds`、`HoverMap`、`Pointer<E>`。
- 提供相机和幕布资源：`Camera`、`PrimaryScreen`、`View`。
- 提供双世界同步：`SyncToRenderWorld`、`entity_sync_system()`。
- 提供 extractor 调度：`add_extractor()`、`Extractors`。
- 提供渲染世界与渲染命令：`RenderWorld`、`ExtractedUI`、`RenderCommand`、`RenderPhase`。
- 提供软件渲染后端。
- 提供基础资产和动画系统。

FHRE 不应该承担：

- NuttX/X11/Linux/裸机的具体窗口实现。
- `/dev/fb0`、`/dev/input0`、`/dev/kbd` 等设备读取。
- 平台按键码映射。
- Wing 的壳层状态、手势、主题、通知、应用卡片。
- 任何 PC 桌面窗口管理语义。

### `apps/examples/fhre`

FHRE 的示例应用，不是引擎核心。

主要任务：

- 展示如何用 FHRE 写一个 3D demo。
- 定义 demo 自己的组件：`Cube`、`SoccerBall`、`Button`。
- 定义 demo 自己的资源：`DemoState`、纹理 handle、动画状态等。
- 注册 demo 自己的 startup/update/pre-update 系统。
- 实现 demo 自己的 extractor：把 `Cube/SoccerBall/Button` 转成渲染数据。
- 提供 NuttX SIM 平台层：`Window` 实现、`InputBridge`、`PlatformInputPlugin`。

这个目录可以作为 FHRE 使用范例，但不能把 demo 中的组件或平台代码上升为 FHRE core 的设计要求。

### `apps/wing`

Wing 壳层库。它依赖 FHRE，但不应该实现自己的 runtime。

主要任务：

- 定义 shell 领域组件：
  - `ShellRoot`
  - `HomeSurface`
  - `AppSurface`
  - `SurfaceStackRoot`
  - `CardStackRoot`
  - `OverlayLayer`
  - `NotificationPanel`
  - `QuickControlTile`
  - `BrightnessControl`
  - `NotificationCard`
  - `SurfacePreviewCard`
  - `SurfaceText`
  - `ButtonWidget`
- 定义 shell 领域资源：
  - `ShellState`
  - `ShellContent`
  - `ShellMetrics`
  - `GestureState`
  - `ShellOverlayAnimation`
  - `ThemeState`
  - `ThemeAnimation`
- 定义 shell 行为系统：
  - shell 初始化
  - picking 封装
  - pointer/button 状态更新
  - overlay 交互
  - 手势识别
  - overlay 动画
  - 布局系统
- 定义 shell extractor：
  - `extract_view`
  - `extract_wing_shell`
  - `queue_wing_primitives`
- 提供 `WingShellPlugin`，把默认资源、系统、extractor 组装进 FHRE `App`。

Wing 不应该承担：

- 自己的主循环。
- 自己的窗口系统。
- 自己的平台事件缓存树。
- 绕过 FHRE extractor 的中心式绘制管线。
- `WindowManager/Desktop/Taskbar/Launcher` 这类 PC 桌面默认语义。

### `apps/examples/wing`

Wing 的示例可执行应用。它负责把 FHRE core、Wing shell 和具体平台实现组装起来。

主要任务：

- 创建平台 `Window`。
- 从平台窗口获取屏幕尺寸。
- 创建 `App::new(width, height)`。
- 添加 `DefaultPlugins`。
- 添加 `WingShellPlugin`。
- 插入 Wing 示例需要的输入资源：
  - `ButtonInput<KeyCode>`
  - `ButtonInput<MouseButton>`
  - `MousePosition`
  - `ThemeState`
  - `ShellMetrics`
- 创建 `PlatformInputPlugin`。
- 调用 `app.run(&mut window, &input_plugin, FRAME_DELAY_MS)`。

这个目录是“平台装配层”，不是 Wing 库本身。

## 分层架构

```text
┌────────────────────────────────────────────────────────────────────┐
│ apps/examples/wing                                                  │
│ Platform app composition                                            │
│ - NuttX SIM framebuffer Window                                      │
│ - InputBridge / PlatformInputPlugin                                │
│ - App::new + DefaultPlugins + WingShellPlugin                       │
└────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────────┐
│ apps/wing                                                          │
│ Wing shell layer                                                    │
│ - Shell components/resources/systems                                │
│ - Gesture, overlay, layout, theme                                   │
│ - extract_wing_shell + queue_wing_primitives                        │
└────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────────┐
│ apps/fhre                                                          │
│ FHRE core                                                           │
│ - App, ECS, schedule, plugins                                       │
│ - Window/InputPlugin traits                                         │
│ - picking/events                                                    │
│ - Camera + PrimaryScreen + View                                     │
│ - MainWorld -> RenderWorld sync/extract/render                      │
│ - SoftwareBackend                                                   │
└────────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────────┐
│ Platform devices                                                    │
│ - /dev/fb0                                                          │
│ - /dev/input0                                                       │
│ - /dev/kbd                                                          │
│ - X11 simulation / board-specific display and input                 │
└────────────────────────────────────────────────────────────────────┘
```

## 帧执行链路

Wing 运行时的一帧应该按下面的链路理解：

```text
Platform Window
  -> collect_input_events()
  -> WindowInputEvents
  -> PlatformInputPlugin::bridge()
  -> ButtonInput<T> / MousePosition

FHRE App::update_and_render()
  -> initialize_plugins()
  -> Startup: setup_wing_shell
  -> Time update
  -> PreUpdate:
       wing_picking_system
       wing_minimal_button_interaction_system
       wing_shell_interaction_system
       wing_gesture_system
  -> Update:
       wing_overlay_animation_system
       wing_shell_layout_system
       wing_shell_stack_layout_system
       wing_shell_overlay_layout_system
       wing_shell_notification_panel_layout_system
       wing_shell_quick_controls_layout_system
       wing_shell_notification_cards_layout_system
       wing_shell_overlay_card_layout_system
  -> entity_sync_system()
  -> extractors:
       extract_view
       extract_wing_shell
       queue_wing_primitives
  -> RenderWorld::execute_render()

Platform Window
  -> present(framebuffer)
```

注意：`queue_wing_primitives` 虽然名字里有 `queue`，但它仍作为 FHRE extractor 阶段的一部分运行，负责把已经提取到 `RenderWorld` 的 shell 渲染组件排入 render phase。

## 输入对接

输入分三层：

### 平台层

位置：`apps/examples/wing/rust/src/platform/`

职责：

- 从平台设备读取原始输入。
- 生成 FHRE 的 `WindowInputEvents`：
  - `MouseButtonEvent`
  - `MouseMotionEvent`
  - `MouseWheelEvent`
  - `KeyboardEvent`
- 实现 `InputBridge`，把平台码映射成 Wing 使用的 `KeyCode` / `MouseButton`。

### FHRE 接口层

位置：`apps/fhre/rust/src/window/mod.rs`

职责：

- 定义平台无关 `Window` trait：
  - `is_running()`
  - `collect_input_events()`
  - `present()`
  - `dimensions()`
- 定义平台无关 `InputPlugin` trait：
  - `bridge(&self, app, events)`
- 提供 `MousePosition` resource。

FHRE 不规定具体 `KeyCode` 和 `MouseButton` 枚举，因为不同应用可以定义自己的输入语义。

### Wing 输入资源层

位置：`apps/wing/rust/src/input.rs`

职责：

- 定义 Wing 当前需要的 `KeyCode`、`MouseButton`、`MouseWheel`。
- 定义 `ButtonInput<T>`。
- 作为 ECS resource 被 Wing 系统读取。

当前 Wing 的 `wing_picking_system` 读取：

```text
MousePosition + ButtonInput<MouseButton>
```

然后复用 FHRE picking 基础能力：

```text
ui_picking_backend()
update_hover_map()
pointer_events()
```

最终生成：

```text
Pointer<Over>
Pointer<Out>
Pointer<Press>
Pointer<Click>
...
```

## 坐标与空间语义

当前代码基线使用屏幕像素坐标：

```text
(0, 0)             -> 屏幕左上角
(W, H)             -> 屏幕右下角
(W * 0.5, H * 0.5) -> 屏幕中心
```

Wing 的 `ShellMetrics::center_x()` / `center_y()` 就是按这个模型计算。

`MousePosition` 直接保存平台输入坐标，picking 也直接用这个坐标做 hit test。因此，当前不应再写“原点在屏幕中心、范围是 `[-W/2, W/2]`”这样的描述。

同时，Wing 控件仍然必须遵守 FHRE 的纯 3D 语义：

- 每个 shell 控件是 `MainWorld` 中的 ECS 实体。
- 每个可见控件通过 `Transform.position` 表示空间位置。
- `x/y` 当前按幕布像素坐标使用。
- `z` 表示 3D 深度和层次。
- 视觉上像 2D 的元素，只是默认贴近幕布平面。

当前 Wing 的渲染输出仍然主要是 `ExtractedUI -> DrawRect` 和 `ExtractedShellText -> DrawText`，这是软件渲染后端的最小表达，不代表 Wing 可以退化成独立 2D GUI runtime。

后续如果要把 shell 控件进一步做成完整 3D primitive，应沿下面路径演进：

```text
Shell component
  -> Transform / size / state
  -> extractor 生成 3D-aware render data
  -> RenderWorld phase
  -> SoftwareBackend 或未来 GPU backend
```

而不是新增一套屏幕空间 GUI 对象树。

## 渲染对接

渲染分四步：

### 1. MainWorld 中维护 shell 语义

Wing 系统只修改 ECS 世界：

- `ShellState`
- `ShellContent`
- `ThemeState`
- `Transform`
- `PickableBounds`
- shell 组件上的 `visible/open/active` 字段

### 2. `entity_sync_system()` 同步实体关系

FHRE 自动把需要同步的实体关系维护到 `RenderWorld`。

Wing 不应手动维护一套长期存在的 render entity map。

### 3. `extract_wing_shell()` 提取渲染态

Wing extractor 读取 MainWorld：

- `Transform`
- `WidgetLayoutNode`
- shell marker/state components
- `ShellState`
- `ShellContent`
- `ThemeState`
- `ShellOverlayAnimation`

然后写入 RenderWorld：

- `ExtractedUI`
- `ExtractedShellText`

不可见实体必须移除旧的 render component，避免 stale rendering。

### 4. `queue_wing_primitives()` 排入渲染阶段

`queue_wing_primitives()` 从 RenderWorld 读取：

- `ExtractedUI`
- `ExtractedShellText`

生成：

- `RenderCommand::DrawRect`
- `RenderCommand::draw_text(...)`

并通过 `RenderPhaseType::Ui` 排序。

## Wing 领域模型

### Shell 状态

`ShellState` 是当前壳层模式的核心资源：

```text
active_surface: Option<SurfaceId>
overlay_mode:
  - None
  - NotificationPanel
  - AppSwitcher
```

它描述当前 shell 在哪个主模式，而不是描述平台窗口。

### Shell 内容

`ShellContent` 是 shell 可展示数据的资源：

- `surfaces`
- `notifications`
- `quick_controls`

它已经有动态 API：

- `add_surface()`
- `remove_surface()`
- `add_notification()`
- `remove_notification()`
- `clear_notifications()`
- quick control toggle/set 方法

但当前代码中，surface/card/notification 实体主要在 `setup_wing_shell` 的 Startup 阶段按初始内容生成。因此：

```text
只修改 ShellContent 资源 != 自动生成或删除对应实体
```

如果要支持真正运行时动态内容，需要新增系统：

```text
ShellContent diff
  -> spawn missing entities
  -> despawn removed entities
  -> update component fields
  -> layout system updates Transform/PickableBounds
  -> extractor renders new state
```

这个系统应在 Wing 层实现，不应放入 FHRE core。

### Shell 布局

`ShellMetrics` 是当前布局度量资源：

- 屏幕宽高。
- 中心点。
- 通知面板高度。
- 快捷控制 tile 自适应布局。
- 通知卡片尺寸和间距。
- 应用 surface 尺寸。

布局系统负责把状态和内容转换为：

- `Transform.position.x/y/z`
- `PickableBounds.width/height`
- `WidgetLayoutNode.width/height`
- component visibility/open/active 字段

### 手势

`GestureState` 保存当前 pointer 手势过程。

`wing_gesture_system` 读取：

- `MousePosition`
- `ButtonInput<MouseButton>`
- `Time`
- `ShellState`
- `ThemeState`

输出：

- 下滑打开通知面板。
- 上滑打开应用切换器。
- 在 overlay 打开时反向滑动关闭。
- 顶部区域长按切换主题。

手势系统只改变 shell resources，不直接绘制。

### 动画

`ShellOverlayAnimation` 保存 overlay 进度：

- `notification_panel_progress`
- `app_switcher_progress`
- `overlay_alpha`
- `card_alpha`

`ThemeAnimation` 保存主题过渡进度。

动画系统只推进资源；布局和 extractor 再消费这些资源。

## 插件装配

`WingShellPlugin` 是 Wing 和 FHRE 的主要对接点。

它负责：

- 确保 Wing resources 存在。
- 注册 Startup 系统。
- 注册 PreUpdate 系统。
- 注册 Update 系统。
- 注册 extractors。

它不负责：

- 创建平台窗口。
- 读取平台输入。
- 调用主循环。
- 选择 NuttX 设备路径。

示例 app 的装配关系应保持为：

```rust
let (width, height) = window.dimensions();
let mut app = App::new(width, height);

app.add_plugins(DefaultPlugins)
   .add_plugin(WingShellPlugin)
   .insert_resource(ButtonInput::<KeyCode>::default())
   .insert_resource(ButtonInput::<MouseButton>::default())
   .insert_resource(MousePosition::default())
   .insert_resource(ThemeState::default())
   .insert_resource(ShellMetrics::new(Vec2::new(width as f32, height as f32)));

let input_plugin = PlatformInputPlugin::new(InputAdapter);
app.run(&mut window, &input_plugin, 16);
```

## 与 FHRE 示例应用的关系

`apps/examples/fhre` 和 `apps/examples/wing` 的关系是并列的：

```text
apps/examples/fhre -> 展示 FHRE 作为 3D/demo engine 怎么用
apps/examples/wing -> 展示 Wing shell 怎么在 FHRE 上运行
```

二者都可以有自己的：

- platform `Window` 实现。
- `InputBridge`。
- `ButtonInput<T>`。
- 应用组件。
- extractor。

差别是：

- FHRE demo 的应用语义是 `Cube/SoccerBall/Button`。
- Wing demo 的应用语义是 `Shell/Surface/Notification/QuickControl`。

不要把 `apps/examples/fhre` 的 demo 组件迁入 FHRE core，也不要把 `apps/examples/wing` 的平台代码迁入 Wing core。

## 责任矩阵

| 能力 | FHRE core | Wing library | examples/fhre | examples/wing |
|------|-----------|--------------|---------------|---------------|
| ECS world/entity/component/query | 负责 | 使用 | 使用 | 使用 |
| App 主循环 | 负责 | 不负责 | 使用 | 使用 |
| Schedule label | 负责 | 注册系统 | 注册系统 | 不直接定义 |
| Window trait | 定义 | 不实现 | 实现平台版本 | 实现平台版本 |
| InputPlugin trait | 定义 | 不实现平台版本 | 实现 demo 平台桥接 | 实现 Wing 平台桥接 |
| KeyCode/MouseButton | 不固定 | 定义 Wing 版本 | 定义 demo 版本 | 使用 Wing 版本 |
| MousePosition | 提供 | 使用 | 使用 | 插入资源 |
| Picking backend | 提供 | 封装使用 | 直接/封装使用 | 间接使用 |
| Pointer events | 提供 | 消费 | 消费 | 间接使用 |
| Camera/PrimaryScreen/View | 提供 | extract view 使用 | 使用 | 通过插件使用 |
| ShellState/ShellContent | 不知道 | 负责 | 不使用 | 组装 |
| Shell layout/gesture/theme | 不知道 | 负责 | 不使用 | 组装 |
| ExtractedUI/RenderCommand | 提供 | 写入/queue | 写入/queue | 间接使用 |
| NuttX `/dev/fb0` | 不知道 | 不知道 | 负责 | 负责 |
| Demo model objects | 不知道 | 不知道 | 负责 | 不使用 |

## 禁止回退的设计线

后续修改 Wing 时，不应引入：

- `WingRuntime` 这类长期存在的独立 runtime。
- 独立 GUI 对象树。
- 独立 z-index 系统。
- 壳层私有主循环。
- 壳层私有输入事件缓存树。
- 绕过 extractor 的 `generate_render_commands()` 中心绘制主线。
- PC 桌面窗口管理器默认语义。

如果确实需要新增能力，应优先判断它属于哪一层：

```text
平台设备/事件读取        -> apps/examples/wing/platform
输入映射和 app 组装      -> apps/examples/wing
壳层状态/布局/手势/主题   -> apps/wing
ECS/调度/同步/渲染基础    -> apps/fhre
```

## 推荐的下一步整理

1. 继续把 `docs/wing.md` 作为设计约束文档维护，把本文作为实现对接文档维护。
2. 清理 `docs/ARCHITECTURE.md`、`apps/fhre/ARCHITECTURE.md`、代码 `FHRE_VERSION` 之间的版本差异。
3. 若要支持真正动态 surface/notification，新增 Wing 内容实体同步系统，而不是修改 FHRE core。
4. 继续推进 Wing 控件从“3D ECS 实体 + 2D primitive 输出”向更完整的 3D-aware render data 收口。

## 判断标准

一个新的 Wing 功能如果满足下面条件，就符合当前架构：

- 壳层状态存在于 resource 或 component 中。
- 行为由 system 推进。
- 空间位置由 `Transform` 表达。
- 点击命中由 FHRE picking 链路产生 `Pointer<E>`。
- 可见渲染态通过 extractor 进入 `RenderWorld`。
- 最终绘制由 `RenderCommand` 和 FHRE backend 执行。
- 平台差异只存在于 `apps/examples/wing/platform` 或类似平台适配层。

反过来，如果一个功能需要私有主循环、私有对象树、私有 z-index、私有最终绘制出口，它就不符合 Wing + FHRE 的当前架构基线。
