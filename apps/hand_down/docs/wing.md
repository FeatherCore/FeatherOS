# Wing Shell Architecture（历史参考）

> 当前新方向是将 FHRE 合入 Wing，形成单 crate 的轻量声明式 ECS UI/渲染系统。
> 新架构基线见 `docs/WING_UNIFIED_ARCHITECTURE.md`。
> 本文保留为旧 `Wing on FHRE` 设计的历史参考，不再作为后续实现的主约束。

`Wing` 是运行在 `FHRE` 之上的移动/手表壳层系统。

当前实现对接基线见 `docs/wing_fhre_integration.md`。本文主要保留 Wing 的设计约束和产品方向；当实现细节与对接关系需要判定时，以 `docs/wing_fhre_integration.md` 为准。

唯一架构基线是 `docs/ARCHITECTURE.md`。Wing 必须严格服从 FHRE 的四个核心设计主题，而不是继续沿着 PC 桌面窗口系统前进。

## 定位

- Wing 不是 PC 桌面窗口管理器
- Wing 不是另一套 GUI runtime
- Wing 不是 `lvgl` 风格对象树的 Rust 复刻
- Wing 是构建在 FHRE 上的移动/手表壳层
- Wing 第一阶段只做空壳层，不做默认应用

一句话定义：`Wing = FHRE 上的 shell，而不是 FHRE 旁边的 GUI 引擎。`

## 必须遵守的四层边界

### 1. 相机 + 幕布系统

Wing 必须直接使用 FHRE 的 `Camera + PrimaryScreen + View` 模型。

**核心原则：Wing 下所有控件，即使是视觉上的 2D 组件，本体都必须是 3D 对象。**

- 所有壳层对象都是 3D 实体
- 所有控件都必须是 3D 对象，包括那些视觉上像 `2D` 的按钮、卡片、面板、文字和手势区
- `2D` 只是对象默认位于幕布平面附近时的视觉结果，而不是另一套对象类型
- 空间表达只允许使用 `Transform`
- 前后层次只允许使用 `Transform.position.z`
- 不能再定义一套独立屏幕空间或 z-index runtime

这条规则必须按 `docs/ARCHITECTURE.md` 的纯 3D 语义理解：

- 视觉上看起来像 `2D` 的控件，本体仍然是 3D 对象
- 默认状态下，它们只是与 `PrimaryScreen` 对应的幕布平面处于同一平面或近平面
- **任何控件都必须允许在 3D 空间中进行位移、缩放、旋转和层叠**
- **例如一个默认贴在幕布上的按钮，可以在 3D 空间内相对于摄像机做远近移动，也可以做旋转、缩放等空间变换**
- **不能因为某个控件视觉上像 2D，就把它实现成脱离相机投影的纯屏幕空间对象**
- Wing 不允许把所谓 `2D` 控件实现成脱离相机与幕布关系的专用屏幕空间 primitive

当前壳层骨架中的 `ShellRoot`、`SurfaceStackRoot`、`CardStackRoot`、`NotificationStackRoot`、`HomeSurface`、`StatusBar`、`QuickSettingsPanel`、`OverlayLayer`、`NotificationLayer`、`NotificationCard`、`AppSurface`、`SurfacePreviewCard`、`BottomBar`、`GestureZone` 都必须作为 3D ECS 实体存在于 `MainWorld`。

### 2. 呈现窗口 + 输入系统

Wing 必须严格区分平台呈现窗口和壳层实体。

- 平台层负责 `Window` trait
- 平台层负责输入设备接入和 `InputBridge`
- FHRE 负责把原始事件桥接成 `ButtonInput<T>`、`MousePosition`、`Pointer<E>`
- Wing 只消费这些 ECS 资源和事件

边界如下：

- `apps/examples/wing/rust/src/platform/`：平台窗口、X11/framebuffer、输入适配
- `apps/wing/rust/src/`：壳层组件、资源、系统、extractor

禁止事项：

- 不能再实现一套 Wing 自己的输入主循环
- 不能在壳层层面维护平台事件缓存树
- 不能把"窗口"概念混淆成平台呈现窗口

### 3. 核心系统

Wing 只能建立在 FHRE 已有核心系统之上。

- `Query`
- `App` 生命周期
- picking
- `Pointer<E>` 事件
- `entity_sync_system()`
- extractor / queue
- `RenderWorld`
- `RenderCommand`

当前默认主线必须保持为：

```text
Startup
  -> setup_wing_shell
PreUpdate
  -> wing_picking_system
  -> wing_minimal_button_interaction_system
  -> wing_shell_interaction_system
  -> wing_gesture_system
Update
  -> wing_overlay_animation_system
  -> wing_shell_layout_system
  -> wing_shell_stack_layout_system
  -> wing_shell_overlay_layout_system
  -> wing_notification_card_layout_system
  -> wing_shell_overlay_card_layout_system
  -> wing_notification_text_layout_system
entity_sync
extract
  -> extract_view
  -> extract_wing_shell
  -> queue_wing_primitives
render
```

禁止事项：

- 不能恢复 `generate_render_commands()` 式中心对象主线
- 不能让壳层业务绕过 extractor 直接控制最终渲染输出
- 不能再把 PC 风格窗口装饰作为产品默认主语义

### 4. 声明式 ECS：自动化和必须手动的内容

FHRE 已经自动化的内容，Wing 不应重复实现。

FHRE 自动化：

- `App::run()`
- 帧推进
- 事件更新
- `entity_sync_system()`
- extractor 调度
- render 执行
- 平台输入到 ECS 资源的桥接流程

Wing 必须手动实现：

- 壳层组件
- 壳层资源
- 壳层行为系统
- 壳层 extractor
- 平台层 `Window` 与 `InputBridge` 实现

硬性规则：

- 业务状态必须拆到组件和资源里
- 行为必须写成系统
- 渲染态必须通过 extractor 进入 `RenderWorld`
- 不允许重新包装一个长期存在的 `WingRuntime`

## 目录结构

```
apps/wing/rust/src/                   # Wing 库（壳层组件/资源/系统）
├── lib.rs                            # 库入口
├── input.rs                          # 输入类型（KeyCode, MouseButton, ButtonInput）
├── plugin.rs                         # WingShellPlugin
├── types.rs                          # 类型定义（SurfaceId, WidgetId）
├── components/                       # ECS 组件
│   ├── mod.rs
│   ├── shell.rs                     # ShellRoot, HomeSurface, StatusBar, etc.
│   └── widgets.rs                    # WidgetLayoutNode, ButtonWidget
├── resources/                        # ECS 资源
│   ├── mod.rs                        # ShellMetrics
│   ├── animation.rs                  # ShellOverlayAnimation
│   ├── content.rs                    # ShellContent, ShellSurfaceEntry, ShellNotificationEntry
│   ├── gesture.rs                    # GestureState, GesturePhase, SwipeDirection
│   ├── shell.rs                     # ShellState
│   └── theme.rs                     # ThemeState
├── systems/                          # ECS 系统
│   ├── mod.rs
│   ├── animation.rs                  # wing_overlay_animation_system
│   ├── gesture.rs                    # wing_gesture_system
│   ├── shell.rs                     # setup_wing_shell, 交互/布局系统
│   ├── picking.rs                   # wing_picking_system
│   └── pointer.rs                   # wing_minimal_button_interaction_system
├── extract/                          # 渲染提取
│   ├── mod.rs
│   ├── shell.rs                     # extract_wing_shell
│   ├── primitives.rs                # queue_wing_primitives
│   └── view.rs                      # extract_view
└── theme/                            # 主题系统
    └── mod.rs                        # ThemePalette, WingTheme

apps/examples/wing/rust/src/          # Wing Demo（可执行应用）
├── lib.rs                            # wing_rust_main() 入口点
└── platform/                         # 平台实现
    ├── mod.rs
    ├── framebuffer.rs               # Window trait 实现（NuttX SIM）
    ├── runner.rs                     # InputBridge, PlatformInputPlugin
    └── input/                        # 输入类型
        └── mod.rs

nuttx/boards/sim/sim/sim/configs/wing/ # NuttX 配置
├── defconfig                         # 默认配置
└── README.txt                        # 说明文档
```

## 构建与运行

### 构建命令

```bash
cd /home/uan-gpd/codes/FeatherOS/nuttx
./wing_build.sh
```

或手动构建：

```bash
cd /home/uan-gpd/codes/FeatherOS/nuttx
make distclean
./tools/configure.sh sim:wing
make -j4
```

### 运行命令

```bash
./nuttx
nsh> wing_rust
```

### 构建配置

- 配置名称：`sim:wing`
- 目标平台：NuttX SIM (x86_64 Linux)
- 编译方式：Rust native compilation（无交叉编译）
- 输出文件：`nuttx` (ELF 64-bit executable)

## 当前数据流

Wing 现在的正确数据流应当始终保持为：

```text
Platform Window
  -> WindowInputEvents
  -> PlatformInputPlugin
  -> ButtonInput<T> / MousePosition
  -> wing_picking_system
  -> Pointer<E>
  -> shell ECS systems in MainWorld
  -> entity_sync_system()
  -> extractors
  -> RenderWorld
  -> queue_wing_primitives
  -> SoftwareBackend
  -> Presentation Window
```

这个链路里不存在独立的 Wing GUI runtime。

必须进一步强调：这个链路里的壳层控件不是“最后一步临时拼出来的 2D GUI 对象”，而是始终处于 FHRE 相机 + 幕布系统中的 3D 对象。所谓按钮、面板、通知卡片等视觉上接近 `2D` 的控件，只是默认贴近幕布平面，不代表它们可以脱离 3D 世界坐标和相机投影语义独立存在。

## 当前实现状态

### 当前实现状态

### 已落地

**主链路已经成立：**
- ✅ shell 实体在 `MainWorld` 中生成
- ✅ 平台输入通过 `Window + InputBridge` 进入 ECS 资源
- ✅ `wing_picking_system` 生成 `Pointer<E>` 事件
- ✅ shell 状态通过系统更新 `Transform` / `PickableBounds`
- ✅ shell 渲染通过 extractor 进入 `RenderWorld`
- ✅ `queue_wing_primitives` 生成当前阶段的壳层绘制命令

**当前可用的壳层骨架：**
- ✅ `ShellRoot`
- ✅ `HomeSurface`
- ✅ `SurfaceStackRoot`
- ✅ `CardStackRoot`
- ✅ `OverlayLayer`
- ✅ `NotificationPanel`
- ✅ `QuickControlTile`（WiFi、蓝牙、飞行模式等）
- ✅ `BrightnessControl`
- ✅ `NotificationCard`
- ✅ `AppSurface`
- ✅ `SurfacePreviewCard`
- ✅ `SurfaceText`

**当前可用的系统与提取器：**
- ✅ `setup_wing_shell`
- ✅ `wing_picking_system`
- ✅ `wing_minimal_button_interaction_system`
- ✅ `wing_shell_interaction_system`
- ✅ `wing_gesture_system`
- ✅ `wing_overlay_animation_system`
- ✅ `wing_shell_layout_system`
- ✅ `wing_shell_stack_layout_system`
- ✅ `wing_shell_overlay_layout_system`
- ✅ `wing_shell_notification_panel_layout_system`
- ✅ `wing_shell_quick_controls_layout_system`
- ✅ `wing_shell_notification_cards_layout_system`
- ✅ `wing_shell_overlay_card_layout_system`
- ✅ `extract_view`
- ✅ `extract_wing_shell`
- ✅ `queue_wing_primitives`

**当前可用的资源：**
- ✅ `ShellState`
- ✅ `ShellContent` - 支持动态添加/移除 surface 和 notification，包含完整数据模型
- ✅ `ShellMetrics`
- ✅ `ShellOverlayAnimation`
- ✅ `ThemeState`
- ✅ `GestureState`

**当前布局模型：**
- ✅ `CardStackLayout` - 根据 `ShellContent.surfaces.len()` 动态计算卡片尺寸和间距
- ✅ `NotificationStackLayout` - 根据 `ShellContent.notifications.len()` 动态计算通知卡片布局

**主题与平台层：**
- ✅ `ThemePalette` / `WingTheme`
- ✅ `shell_palette()`
- ✅ 预设主题：`aurora()`、`dusk()`
- ✅ NuttX SIM framebuffer `Window` 实现
- ✅ `InputBridge` 与 `PlatformInputPlugin`

### 架构约束

以下部分仍未完全达到 `docs/ARCHITECT.md` 要求的纯 3D 语义，必须明确记录：

- 当前控件虽然已经作为 3D ECS 实体存在于 `MainWorld`，但最终渲染结果仍偏向最小 2D primitive 输出
- 视觉上像 `2D` 的按钮、卡片、面板，当前实现还没有完整落实为"默认贴在幕布平面上的 3D 对象"这一约束
- 当前主线已经接入 `Camera + PrimaryScreen + View`，但壳层控件的最终呈现语义仍需要继续向统一的相机投影语义收口
- `Transform.position.z` 现在已经承担层次表达，但后续仍要确保它不仅是排序辅助值，而是真正可参与控件空间行为的 3D 深度

### 当前限制

- 现在是最小 shell 骨架，不是完整系统壳层
- `ShellState` 已开始收口为更明确的壳层模式状态，但整体仍是最小状态模型
- 布局已实现动态计算：`CardStackLayout` 和 `NotificationStackLayout` 根据 `ShellContent` 数量自适应
- **手势识别系统**：手势检测覆盖整个屏幕，支持下滑打开通知面板
- **长按手势已实现**：在屏幕顶部区域长按可切换主题（Aurora ↔ Dusk）
- Overlay 已接入基础过渡动画，NotificationPanel / AppSwitcher 不再瞬时切换
- **Overlay 动画已扩展透明度细节**：卡片和遮罩层透明度随动画进度平滑过渡
- **ShellContent 已支持动态更新 API**：`add_surface()`、`remove_surface()`、`add_notification()`、`remove_notification()` 等方法
- **ThemeAnimation 主题过渡动画**：`ThemeAnimation` 资源支持平滑的主题切换过渡
- **ThemeState 扩展**：`pending_variant`、`is_transitioning` 支持主题切换动画状态追踪
- **安卓风格通知面板**：下滑显示快捷控制（WiFi、蓝牙、亮度等）+ 应用通知列表
- **全屏手势**：顶部状态栏和底部导航栏已移除，不占用主屏幕显示区域
- `WingShellPlugin` 已负责主线资源、系统和 extractor 的默认装配

### 当前交互

当前默认交互保持克制：

**滑动手势（全屏）：**
- 在屏幕任意位置向下滑动 → 打开 `NotificationPanel`（安卓风格通知面板）
- 在屏幕任意位置上滑动 → 打开 `AppSwitcher`（应用预览卡片）
- 在 `NotificationPanel` 打开时向上滑动 → 关闭
- 在 `AppSwitcher` 打开时向下滑动 → 关闭
- 滑动阈值：50 像素位移 或 200 像素/秒速度
- 快速滑动（速度 > 200 像素/秒）即使位移较小也能触发

**长按手势：**
- 在屏幕顶部区域长按（> 0.5 秒且移动 < 15 像素）→ 切换主题（Aurora ↔ Dusk）

**点击交互：**
- StatusBar 和 BottomBar 不再显示（不占用屏幕空间）
- 点击 `OverlayLayer`（遮罩层）关闭 panel
- 点击通知面板关闭

**注意：**
- 顶部状态栏和底部导航栏已移除，不占用主屏幕显示区域
- 手势检测覆盖整个屏幕，无需特定区域触发

## 数据模型

### ShellSurfaceEntry

应用 Surface 数据模型：
- `id`: SurfaceId - 唯一标识
- `label`: &'static str - 显示名称
- `preview_title`: &'static str - 预览卡片标题
- `state`: SurfaceState - 运行状态（Running/Paused/Background/Closed）
- `icon_hint`: &'static str - 图标提示
- `last_active_time`: u64 - 最后活跃时间戳

### ShellNotificationEntry

通知数据模型：
- `id`: u32 - 唯一标识
- `title`: &'static str - 标题
- `summary`: &'static str - 摘要
- `app_name`: &'static str - 应用名称
- `priority`: NotificationPriority - 优先级（Low/Normal/High）
- `category`: NotificationCategory - 分类（System/Message/Email/Social/Alarm/Reminder/Other）
- `timestamp`: u64 - 时间戳

### QuickControlState

快捷控制状态：
- `wifi_enabled`: bool - WiFi 开关
- `wifi_connected`: bool - WiFi 连接状态
- `bluetooth_enabled`: bool - 蓝牙开关
- `bluetooth_connected`: bool - 蓝牙连接状态
- `airplane_mode`: bool - 飞行模式
- `flashlight_on`: bool - 手电筒
- `dnd_mode`: bool - 勿扰模式
- `auto_rotate`: bool - 自动旋转
- `battery_saver`: bool - 省电模式
- `brightness`: f32 - 亮度（0.0-1.0）

### ShellContent 动态更新 API

```rust
// 添加/移除 Surface
content.add_surface(surface_entry);
content.remove_surface(surface_id);

// 添加/移除通知
content.add_notification(notification_entry);
content.remove_notification(notification_id);
content.clear_notifications();

// 查询
content.get_surface(surface_id);
content.surface_count();
content.notification_count();

// 快捷控制
content.toggle_wifi();
content.toggle_bluetooth();
content.toggle_airplane_mode();
content.toggle_flashlight();
content.set_brightness(0.8);
```

## 当前非目标

当前仍然明确不做：

- 默认应用
- PC 桌面窗口控件体系
- 独立输入运行时
- 独立 GUI 对象树
- 绕过 extractor 的渲染主线
- 通知中心的复杂数据模型和生命周期调度

## 迁移原则

旧的 `desktop/window/taskbar/launcher` 过渡代码已经从 `apps/wing/rust/src/` 主线中移除。后续不再恢复这些 PC 桌面语义模块。

处理原则：

- 不把旧 PC 窗口语义继续扩展成默认主线
- 不重新引入旧模块或旧命名
- 新能力只沿 shell/surface 语义增加

## 下一步

下一阶段应继续沿 shell 主线推进：

1. ~~继续把 `SurfacePreviewCard` 从最小卡片栈扩展成更完整的应用预览栈~~ ✅ 已完成
2. ~~继续把 `NotificationCard + NotificationText` 从默认 demo 内容扩展成更完整的通知数据模型和卡片栈~~ ✅ 已完成
3. ~~补齐手势速度阈值判断，让滑动识别不只依赖位移阈值~~ ✅ 已完成
4. ~~添加长按手势交互~~ ✅ 已完成
5. ~~完善主题切换机制~~ ✅ 已完成
6. ~~让 `ShellContent` 支持动态更新（运行时添加/移除 surface 和 notification）~~ ✅ 已完成
7. ~~把 overlay 动画从位置/尺寸过渡扩展到透明度和层级细节~~ ✅ 已完成

## 结论

Wing 必须彻底放弃 PC 桌面窗口控件方向，严格回到 `docs/ARCHITECTURE.md` 的四个核心设计理念之下：

1. 相机 + 幕布系统
2. 呈现窗口 + 输入系统
3. 核心系统
4. 声明式 ECS 的自动化边界与手动边界

尤其要坚持一条不可回退的约束：Wing 下所有控件，无论视觉上多像 `2D`，本体都必须是 3D 对象。它们只是默认与幕布处于同一平面，因此看起来像传统平面 UI；但任何控件都必须允许在 3D 空间中相对摄像机做前后移动、层叠、变换和其他空间操作。

当前默认主线已经是 shell 骨架。后续所有新增能力都必须围绕移动/手表壳层继续推进，而不是回到 PC 桌面窗口模型。
