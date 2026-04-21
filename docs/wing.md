# Wing Desktop Environment

Wing 是 FeatherOS 基于 FHRE 构建的桌面环境与 2D GUI 壳层。

当前仓库中的 `wing` 已经不再只是概念性骨架，而是具备了：

- 可编译的桌面壳核心
- 可运行的 FHRE demo 入口
- 统一的窗口系统
- LVGL 风格的 2D 组件种类体系
- Widget 树与基础布局
- Widget 事件、动作与主题系统
- 最小可用的文本编辑与下拉选择能力
- 一个真正可交互的 Calculator demo

## 当前状态

### 已完成

- `apps/wing/rust` 可编译
- `apps/examples/wing/rust` 可编译
- 已接入 FHRE `App + Window + InputPlugin` 运行链路
- 已实现最小桌面系统：
  - 壁纸
  - 桌面图标
  - 任务栏
  - 启动器
  - 多窗口
  - 窗口聚焦、拖动、最小化、最大化、关闭

### 当前示例应用

- Calculator
- Files
- Terminal
- Settings
- Gallery

这些应用目前不是独立进程，而是 Wing 内部的桌面窗口示例内容。

## 目录结构

```text
apps/wing/
├── rust/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs        # Wing 入口与总控
│       ├── window.rs     # 窗口系统与窗口管理器
│       ├── widgets.rs    # Widget 类型、树、事件、动作、文本编辑
│       ├── theme.rs      # ThemePalette / WingTheme
│       ├── desktop.rs    # 桌面与图标区
│       ├── taskbar.rs    # 任务栏
│       ├── launcher.rs   # 启动器
│       ├── wallpaper.rs  # 壁纸
│       └── icon.rs       # 桌面图标
└── clang/
    ├── include/wing/wing.h
    └── src/wing.c        # 仍主要是 stub

apps/examples/wing/
└── rust/
    └── src/
        ├── lib.rs        # wing_rust_main()
        ├── extract.rs    # 把 Wing RenderCommand 送入 FHRE
        └── platform/     # framebuffer/input/runner
```

## 架构分层

```text
User Interaction
    ↓
Platform Window (/dev/fb0, /dev/input0, /dev/kbd)
    ↓
examples/wing PlatformInputPlugin
    ↓
WingRuntime (FHRE resource)
    ↓
Wing
  ├── Desktop
  ├── Taskbar
  ├── Launcher
  └── WindowManager
         └── Window
              └── WidgetTreeNode
                   └── WidgetNode
    ↓
RenderCommand
    ↓
FHRE RenderWorld
    ↓
SoftwareBackend
    ↓
Framebuffer
```

## Wing 核心能力

### 1. 桌面壳

`Wing` 统一管理：

- `Desktop`
- `Taskbar`
- `AppLauncher`
- `WindowManager`
- `WingTheme`

关键入口：

- `Wing::new(width, height)`
- `Wing::init()`
- `Wing::update(delta_time)`
- `Wing::handle_mouse_move(position)`
- `Wing::handle_click(position)`
- `Wing::handle_mouse_release(position)`
- `Wing::handle_drag(delta)`
- `Wing::handle_text_edit(command)`
- `Wing::set_theme(theme)`
- `Wing::generate_render_commands()`

位置：`apps/wing/rust/src/lib.rs`

### 2. 窗口系统

`WindowManager` 当前支持：

- 创建窗口
- 聚焦窗口
- 关闭窗口
- 最小化/恢复
- 最大化/恢复
- 标题栏拖动
- Z 顺序管理
- 窗口内 Widget 事件收集

位置：`apps/wing/rust/src/window.rs`

### 3. Widget 种类体系

参考 `lvgl/src/widgets`，当前 `WidgetKind` 已包含：

- `Panel`
- `Label`
- `Button`
- `Image`
- `CheckBox`
- `Switch`
- `Slider`
- `Bar`
- `Arc`
- `Spinner`
- `TextArea`
- `List`
- `Menu`
- `TabView`
- `Table`
- `Chart`
- `Canvas`
- `Keyboard`
- `Dropdown`
- `Roller`
- `SpinBox`
- `Calendar`
- `MsgBox`
- `TileView`
- `ButtonMatrix`
- `Led`
- `Line`

位置：`apps/wing/rust/src/widgets.rs`

### 4. Widget Tree 与布局

当前不是平铺控件列表，而是树形结构：

- `WidgetTreeNode`
- `WidgetLayout::Free`
- `WidgetLayout::Vertical`
- `WidgetLayout::Horizontal`
- `WidgetLayout::Grid`

支持：

- 递归渲染
- 递归 hit-test
- 递归查找 Widget
- 容器 relayout

## 事件系统

### Widget 事件

当前已有：

- `HoverEnter`
- `HoverLeave`
- `Press`
- `Release`
- `Click`
- `ToggleChanged`
- `ValueChanged`
- `SelectionChanged`

关键类型：

- `WidgetEvent`
- `WidgetResponse`
- `WindowWidgetEvent`

### Widget Action

当前已有：

- `ToggleLauncher`
- `CycleTheme`
- `ApplySelectedTheme`
- `ApplyButtonMatrixSelection`
- `CloseWindow`
- `MinimizeWindow`
- `MaximizeWindow`
- `LaunchAppNamed(name)`

`Wing::update()` 会 drain `WindowManager` 中收集的 widget 事件，并执行动作。

## 主题系统

### Theme 类型

- `ThemePalette`
- `WingTheme`

内建主题：

- `WingTheme::aurora()`
- `WingTheme::dusk()`

当前 `set_theme()` 已会更新：

- Desktop
- Wallpaper
- DesktopIcon
- Taskbar
- Launcher
- WindowManager
- Window
- WidgetTreeNode / WidgetNode

位置：

- `apps/wing/rust/src/theme.rs`
- `apps/wing/rust/src/lib.rs`

## 文本渲染与 TextArea

### FHRE 文本渲染现状

FHRE 当前已支持动态文本 `String`：

- `RenderCommand::DrawText { text: String, ... }`
- `RenderCommand::draw_text(position, text, color, size)` 接受 `AsRef<str>`

软件后端已从文本占位矩形升级为最小 bitmap 字符绘制：

- 内建 5x7 ASCII glyph
- 简单按 size 缩放
- 支持 Wing 现阶段常见字符显示

位置：

- `apps/fhre/rust/src/render_world/command.rs`
- `apps/fhre/rust/src/pipeline/software/backend.rs`

### TextArea 当前能力

`TextArea` 已支持：

- 动态文本显示
- 点击聚焦
- 焦点描边
- 光标显示
- 自动换行
- 鼠标点击定位光标
- `Backspace`
- `Delete`
- `Enter` 换行
- `ArrowLeft` / `ArrowRight`
- `ArrowUp` / `ArrowDown`
- `Home` / `End`
- 自动滚动到当前光标行

当前内部状态：

- `text: String`
- `cursor: usize`
- `scroll_row: usize`

## Dropdown 当前能力

`Dropdown` 已支持：

- 真实 option 列表
- 展开/收起
- 当前项显示
- popup 区域命中
- 点击项选择
- `SelectionChanged` 事件
- 可绑定 `ApplySelectedTheme`

Settings 窗口里已用该控件切换：

- `Aurora`
- `Dusk`
- `System`

## ButtonMatrix 与 Calculator

### ButtonMatrix 当前能力

- 单元格级 hit-test
- 单元格选中态
- `SelectionChanged`
- 可绑定动作

### Calculator demo 当前能力

目前 Calculator 已具备最小四则运算状态机：

- 数字输入
- `+ - * /`
- `=`
- `C`

当前按钮矩阵布局：

```text
7 8 9 /
4 5 6 *
1 2 3 -
C 0 = +
```

当前数值显示通过 `SpinBox` 完成。

## examples/wing 运行方式

入口：

- `wing_rust_main()`

位置：

- `apps/examples/wing/rust/src/lib.rs`

主要运行链路：

- `framebuffer::Window`
- `PlatformInputPlugin`
- `WingRuntime`
- `app.add_extractor(extract::queue_wing_shell)`

在 NuttX 配置中启用：

- `EXAMPLES_WING_RUST`

运行名默认：

- `wing_rust`

## 当前限制

### 仍未完成

- `clang` 侧 `wing` 仍主要是 stub
- 真实应用进程/窗口内容隔离尚未做
- 文本选区、拖选、剪贴板、IME 未做
- 滚动条可视化未做
- 鼠标滚轮输入未做
- List / Dropdown 键盘导航还不完整
- ButtonMatrix 还未做更复杂键盘导航与表达式编辑
- Calculator 仍是最小 demo，不是完整计算器

### FHRE 文本限制

- 当前是最小 bitmap ASCII 字体
- 不支持完整 Unicode
- 小写会近似映射为大写显示
- 没有高级排版、kerning、字体资源系统

## 推荐下一步

优先级建议：

1. 鼠标滚轮输入与滚动条可视化
2. TextArea 选区/拖选
3. List / Dropdown 键盘导航
4. Calculator 完整表达式输入与浮点显示
5. 更完整的字体/文本系统
6. 真实应用模型与桌面服务化

## 构建验证

当前已验证通过：

- `apps/fhre/rust`: `cargo check --target x86_64-unknown-linux-gnu`
- `apps/wing/rust`: `cargo check --target x86_64-unknown-linux-gnu`
- `apps/examples/wing/rust`: `cargo check --target x86_64-unknown-linux-gnu`

## 结论

Wing 当前已经不是单纯的“桌面壳雏形”，而是一个正在成形的：

- 桌面 shell
- 2D widget 系统
- 主题系统
- 文本输入系统
- 小型应用承载框架

它现在最接近的阶段是：

**可运行的嵌入式桌面 GUI 原型 + 持续扩展中的组件系统**。
