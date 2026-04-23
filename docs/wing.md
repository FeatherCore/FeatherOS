# Wing Shell Architecture

`Wing` 是运行在 `FHRE` 之上的移动/手表壳层系统。

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

- 所有壳层对象都是 3D 实体
- `2D` 只是对象位于幕布平面附近时的视觉结果
- 空间表达只允许使用 `Transform`
- 前后层次只允许使用 `Transform.position.z`
- 不能再定义一套独立屏幕空间或 z-index runtime

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
Update
  -> wing_shell_layout_system
  -> wing_shell_stack_layout_system
  -> wing_shell_overlay_layout_system
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
│   ├── mod.rs                        # DesktopMetrics
│   ├── shell.rs                     # ShellState
│   ├── theme.rs                     # ThemeState
│   └── window.rs                    # DragTransaction, WindowManagerState
├── systems/                          # ECS 系统
│   ├── mod.rs
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
        ├── mod.rs
        ├── button_input.rs
        ├── keyboard.rs
        └── mouse.rs

nuttx/boards/sim/sim/sim/configs/wing/ # NuttX 配置
├── defconfig                         # 默认配置
└── README.txt                        # 说明文档
```

## 构建与运行

### 构建命令

```bash
cd /home/uan-wsl2/codes/FeatherOS/nuttx
./wing_build.sh
```

或手动构建：

```bash
cd /home/uan-wsl2/codes/FeatherOS/nuttx
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

## 当前实现状态

### 已完成

**核心架构：**
- ✅ 相机 + 幕布系统（z=0 平面）
- ✅ 呈现窗口 + 输入系统（NuttX SIM framebuffer）
- ✅ 核心系统（Query, App 生命周期, picking, Pointer<E>）
- ✅ 声明式 ECS（自动化边界与手动边界）

**壳层组件：**
- ✅ `ShellRoot` - 壳层根节点
- ✅ `HomeSurface` - 主屏幕
- ✅ `SurfaceStackRoot` - Surface 堆栈
- ✅ `CardStackRoot` - 卡片堆栈
- ✅ `NotificationStackRoot` - 通知堆栈
- ✅ `StatusBar` - 状态栏
- ✅ `QuickSettingsPanel` - 快速设置面板
- ✅ `OverlayLayer` - 覆盖层
- ✅ `NotificationLayer` - 通知层
- ✅ `NotificationCard` - 通知卡片
- ✅ `NotificationText` - 通知文本
- ✅ `AppSurface` - 应用 Surface
- ✅ `SurfacePreviewCard` - Surface 预览卡片
- ✅ `BottomBar` - 底部栏
- ✅ `GestureZone` - 手势区域
- ✅ `SurfaceText` - Surface 文本

**系统：**
- ✅ `setup_wing_shell` - 初始化壳层 UI
- ✅ `wing_picking_system` - 指针拾取
- ✅ `wing_minimal_button_interaction_system` - 按钮交互
- ✅ `wing_shell_interaction_system` - 壳层交互
- ✅ `wing_shell_layout_system` - 壳层布局
- ✅ `wing_shell_stack_layout_system` - 堆栈布局
- ✅ `wing_shell_overlay_layout_system` - 覆盖层布局
- ✅ `wing_notification_text_layout_system` - 通知文本布局

**提取器：**
- ✅ `extract_view` - 视图提取
- ✅ `extract_wing_shell` - 壳层数据提取
- ✅ `queue_wing_primitives` - 渲染命令队列

**主题系统：**
- ✅ `ThemePalette` - 主题调色板
- ✅ `WingTheme` - 完整主题
- ✅ `shell_palette()` - 默认调色板
- ✅ 预设主题：`aurora()`, `dusk()`

**平台层：**
- ✅ `Window` trait 实现（NuttX SIM X11 framebuffer）
- ✅ `InputBridge` trait 实现（X11 按键码映射）
- ✅ `PlatformInputPlugin` - 输入桥接插件

### 当前交互

当前默认交互保持克制：

- 点击 `StatusBar` 切换 `QuickSettingsPanel`
- `QuickSettingsPanel` 打开时显示 `OverlayLayer`
- `QuickSettingsPanel` 打开时同时显示 `NotificationCard + NotificationText` 骨架
- 点击 `OverlayLayer` 关闭 panel
- 点击 `GestureZone` 回到 `HomeSurface`
- 点击 `BottomBar` 打开或关闭 `SurfacePreviewCard` 卡片栈
- 点击任意 `SurfacePreviewCard` 激活对应 `surface`

## 当前非目标

当前仍然明确不做：

- 默认应用
- PC 桌面窗口控件体系
- 独立输入运行时
- 独立 GUI 对象树
- 绕过 extractor 的渲染主线
- 通知中心的复杂数据模型和生命周期调度

## 迁移原则

仓库里仍然存在部分旧的 `desktop/window/taskbar/launcher` 代码。这些内容当前只应被视为迁移残留，不能再代表 Wing 的产品方向。

处理原则：

- 不把旧 PC 窗口语义继续扩展成默认主线
- 不把旧模块再接回示例入口
- 后续逐步清理或降级为内部过渡代码
- 新能力只沿 shell/surface 语义增加

## 下一步

下一阶段应继续沿 shell 主线推进：

1. 清理 `widgets.rs` 中仍未使用的占位组件
2. 继续把 `SurfacePreviewCard` 从最小卡片栈扩展成更完整的应用预览栈
3. 继续把 `NotificationCard + NotificationText` 从最小通知列表扩展成更完整的通知卡片栈
4. 添加更多手势交互（滑动、长按等）
5. 完善主题切换机制

## 结论

Wing 必须彻底放弃 PC 桌面窗口控件方向，严格回到 `docs/ARCHITECTURE.md` 的四个核心设计理念之下：

1. 相机 + 幕布系统
2. 呈现窗口 + 输入系统
3. 核心系统
4. 声明式 ECS 的自动化边界与手动边界

当前默认主线已经是 shell 骨架。后续所有新增能力都必须围绕移动/手表壳层继续推进，而不是回到 PC 桌面窗口模型。
