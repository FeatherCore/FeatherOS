# Wing Shell 文档

## 概述

Wing 是一个构建在 FHRE (Feather Hybrid Render Engine) 之上的移动/手表风格壳层系统。它遵循 FHRE 的纯 3D 设计理念，所有 UI 元素都是 3D 对象，只是默认位于 z=0 平面附近。

**架构基线**: `docs/ARCHITECTURE.md`

## 核心设计原则

1. **相机 + 幕布系统**: 所有控件都是 3D 对象，`2D` 只是 z=0 平面的视觉结果
2. **呈现窗口 + 输入系统**: 平台层负责 Window/InputBridge，壳层只消费 ECS 资源
3. **核心系统**: 建立在 FHRE 已有系统之上（Query、picking、extractor）
4. **声明式 ECS**: FHRE 自动化帧推进/同步/渲染，壳层手动实现组件/系统

---

## 界面布局

### 屏幕坐标系

```
┌─────────────────────────────────────────┐
│              屏幕尺寸: W × H              │
│                                          │
│   原点 (0, 0) 在屏幕中心                   │
│                                          │
│   x 范围: [-W/2, W/2]                   │
│   y 范围: [-H/2, H/2]                   │
│   z 范围: 用于层次控制（数值越大越靠前）     │
│                                          │
│   Center: (W/2, H/2)                    │
└─────────────────────────────────────────┘
```

### 主屏幕 (Home Surface)

主屏幕是全屏背景层，始终位于最底层。

| 属性 | 值 |
|------|-----|
| 位置 | `(center_x, center_y, 0.01)` |
| 层级 | 最底层 |
| 尺寸 | 全屏 |
| 颜色 | `palette.surface` |

### 通知面板 (Notification Panel)

从屏幕顶部下滑展开的安卓风格通知面板。

#### 通知面板布局顺序

```
┌─────────────────────────────────────────┐
│  ┌─────────────────────────────────┐    │
│  │       NotificationPanel         │    │
│  │                                  │    │
│  │  ████████████████░░░░░░░░░░░░  │    │  1. Brightness Bar (最上)
│  │                                  │    │
│  │  ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐│    │
│  │  │  │ │  │ │  │ │  │ │  │ │  ││    │  2. QuickControls (居中)
│  │  └──┘ └──┘ └──┘ └──┘ └──┘ └──┘│    │     水平排布居中
│  │  ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐ ┌──┐│    │
│  │  │  │ │  │ │  │ │  │ │  │ │  ││    │
│  │  └──┘ └──┘ └──┘ └──┘ └──┘ └──┘│    │
│  │                                  │    │
│  │  ┌───────────────────────────┐    │    │
│  │  │      NotificationCard 1     │    │    │  3. Notification Cards
│  │  └───────────────────────────┘    │    │
│  │  ┌───────────────────────────┐    │    │
│  │  │      NotificationCard 2     │    │    │
│  │  └───────────────────────────┘    │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

#### 亮度条 (Brightness Bar)

位于通知面板最顶部，全宽（扣除侧边距）。

| 属性 | 值 |
|------|-----|
| 位置 | `top_padding + brightness_height * 0.5` |
| 高度 | `屏幕高度 × 0.02` |
| 颜色 | `palette.surface_alt` (200 alpha) |

#### QuickControls 自适应布局

QuickControls 使用 2 行网格布局，位于亮度条下方，列数根据屏幕宽度自适应，目标是**均匀填满可用宽度并居中**。

**布局算法**：

1. 计算理想列数：`ideal_columns = available_width / 64` (目标 tile 大小)
2. 四舍五入取整：`columns = round(ideal_columns)`
3. 反算 tile 大小以填满宽度：`tile_size = width / (columns + (columns-1) × 0.1)`
4. 如果 tile 太小（<48px），减少列数
5. 如果 tile 太大（>80px），增加列数
6. 网格水平居中：`grid_start_x = center_x - grid_width / 2`

**示例计算**：

| 屏幕宽度 | 理想列数 | 四舍五入 | 最终列数 | Tile 大小 |
|---------|---------|---------|---------|----------|
| 320px | 5.0 | 5 | 5 | 53px |
| 480px | 7.5 | 8 | 6 (限制) | 75px |
| 640px | 10.0 | 10 | 6 (限制) | 96px → 6列 → 98px |

**布局参数**：
- Tile 大小范围: 48-80px（良好触摸目标）
- Tile 间距: `tile_size × 0.1` (10%)
- 列数范围: 3-6 列
- 行数: 固定 2 行
- 顶部内边距: `屏幕高度 × 0.02`
- 区块间距: `屏幕高度 × 0.02`

#### 亮度条 (Brightness Bar)

位于 QuickControls 下方，全宽（扣除侧边距）。

| 属性 | 值 |
|------|-----|
| 高度 | `屏幕高度 × 0.02` |
| 颜色 | `palette.surface_alt` (200 alpha) |

#### 通知卡片 (Notification Cards)

位于亮度条下方，垂直堆叠排列。

**布局计算**：
```
start_y = quick_controls_height + brightness_height + spacing
card_y[i] = start_y + i × (card_height + spacing)
```

| 属性 | 值 |
|------|-----|
| 宽度 | `屏幕宽度 - 侧边距 × 2` |
| 高度 | `屏幕高度 × 0.15` |
| 间距 | `屏幕高度 × 0.03` |
| 优先级颜色 | High: `palette.accent`, Normal/Low: `palette.surface` |

#### NotificationPanel 层级

```
z = 0.050  → Panel 容器
z = 0.051  → QuickControlTile, BrightnessControl
z = 0.052  → NotificationCard (索引递增 +0.001)
z = 0.053  → NotificationCardTitle/Content
```

---

### 应用切换器 (App Switcher)

从屏幕底部上滑展开的应用预览卡片栈。

```
┌─────────────────────────────────────────┐
│                                          │
│                                          │
│     ┌─────────────────────────────┐      │
│     │      SurfaceCard 0          │      │  ← 卡片底部
│     │      (较大, 前景)           │      │
│     └─────────────────────────────┘      │
│       ┌─────────────────────────┐      │
│       │      SurfaceCard 1        │      │
│       └─────────────────────────┘      │
│         ┌───────────────────────┐      │
│         │      SurfaceCard 2      │      │
│         └───────────────────────┘      │
│                                          │
│                                          │
└─────────────────────────────────────────┘
```

#### SurfacePreviewCard 堆叠布局

卡片使用堆叠效果，后面卡片的尺寸和位置逐渐缩小/偏移。

| 属性 | 计算公式 |
|------|---------|
| 卡片宽度 | `屏幕宽度 × 0.55 - 索引 × 宽度 × 0.05` |
| 卡片高度 | `可用高度/卡片数`，限制在 `13%-27%` 屏幕高度 |
| 垂直偏移 | `索引 × 卡片高度 × 0.25` |
| 水平宽度递减 | `索引 × 宽度 × 0.05` |

#### AppSwitcher 层级

```
z = 0.046  → SurfacePreviewCard (索引递增 +0.001)
z = 0.047  → SurfaceCardTitle/Subtitle
```

---

### 遮罩层 (Overlay Layer)

覆盖整个屏幕的半透明遮罩，用于增强面板弹出时的视觉层次感。

| 属性 | 值 |
|------|-----|
| 颜色 | `RGB(10, 16, 32, 160 alpha)` |
| 透明度动画 | `0.6 × max(notification_progress, app_progress)` |

---

## 手势交互

### 滑动手势

| 手势 | 触发条件 | 动作 |
|------|---------|------|
| 下滑（任意位置） | `delta_y > 50px` 或 `velocity_y > 200` | 打开通知面板 |
| 上滑（任意位置） | `delta_y < -50px` 或 `velocity_y < -200` | 打开应用切换器 |
| 上滑（通知面板打开时） | 同上 | 关闭通知面板 |
| 下滑（应用切换器打开时） | 同上 | 关闭应用切换器 |

### 长按手势

| 手势 | 触发条件 | 动作 |
|------|---------|------|
| 长按（屏幕顶部区域） | `hold_time > 0.5s` 且 `移动 < 15px` | 切换主题 (Aurora ↔ Dusk) |

---

## 主题系统

### 预设主题

#### Aurora（亮色）

```
background: RGB(20, 28, 46)     # 深蓝色背景
surface: RGB(248, 250, 252)      # 白色卡片
surface_alt: RGB(236, 242, 248)  # 浅灰卡片
text: RGB(58, 68, 88)           # 深灰文字
accent: RGB(88, 142, 255)       # 蓝色强调
```

#### Dusk（暗色）

```
background: RGB(18, 18, 26)      # 深紫黑背景
surface: RGB(236, 232, 244)      # 浅紫白卡片
surface_alt: RGB(214, 210, 226)  # 浅紫灰卡片
text: RGB(56, 48, 72)           # 深紫文字
accent: RGB(180, 136, 255)       # 紫色强调
```

### 主题切换动画

长按屏幕顶部区域可切换主题，过渡动画持续约 0.33 秒。

---

## 数据模型

### ShellSurfaceEntry（应用表面）

```rust
pub struct ShellSurfaceEntry {
    pub id: SurfaceId,              // 唯一标识
    pub label: &'static str,         // 显示名称
    pub preview_title: &'static str, // 预览卡片标题
    pub state: SurfaceState,        // Running/Paused/Background/Closed
    pub icon_hint: &'static str,    // 图标提示
    pub last_active_time: u64,      // 最后活跃时间戳
}
```

### ShellNotificationEntry（通知条目）

```rust
pub struct ShellNotificationEntry {
    pub id: u32,                        // 唯一标识
    pub title: &'static str,             // 标题
    pub summary: &'static str,           // 摘要内容
    pub app_name: &'static str,          // 来源应用
    pub priority: NotificationPriority,  // Low/Normal/High
    pub category: NotificationCategory,  // System/Message/Email/Social/...
    pub timestamp: u64,                  // 时间戳
}
```

### QuickControlState（快捷控制状态）

```rust
pub struct QuickControlState {
    pub wifi_enabled: bool,      // WiFi 开关
    pub wifi_connected: bool,    // WiFi 连接状态
    pub bluetooth_enabled: bool,  // 蓝牙开关
    pub bluetooth_connected: bool,// 蓝牙连接状态
    pub airplane_mode: bool,      // 飞行模式
    pub flashlight_on: bool,     // 手电筒
    pub dnd_mode: bool,          // 勿扰模式
    pub auto_rotate: bool,       // 自动旋转
    pub battery_saver: bool,     // 省电模式
    pub brightness: f32,          // 亮度 (0.0-1.0)
}
```

---

## 目录结构

```
apps/wing/rust/src/                   # Wing 库
├── lib.rs                            # 库入口
├── plugin.rs                         # WingShellPlugin
├── types.rs                          # 类型定义
├── input.rs                          # 输入类型
├── components/                       # ECS 组件
│   ├── shell.rs                     # ShellRoot, HomeSurface, NotificationPanel, etc.
│   └── widgets.rs                    # ButtonWidget, WidgetLayoutNode
├── resources/                        # ECS 资源
│   ├── mod.rs                       # ShellMetrics, QuickControlsLayout
│   ├── shell.rs                     # ShellState, ShellOverlayMode
│   ├── content.rs                   # ShellContent, notifications, surfaces
│   ├── gesture.rs                   # GestureState, GesturePhase
│   ├── animation.rs                 # ShellOverlayAnimation, ThemeAnimation
│   └── theme.rs                     # ThemeState
├── systems/                          # ECS 系统
│   ├── shell.rs                     # 布局系统
│   ├── gesture.rs                   # 手势识别
│   ├── picking.rs                   # 命中检测
│   ├── pointer.rs                   # 指针交互
│   └── animation.rs                 # 动画系统
├── extract/                          # 渲染提取
│   ├── shell.rs                    # extract_wing_shell
│   ├── primitives.rs               # queue_wing_primitives
│   └── view.rs                     # extract_view
└── theme/                            # 主题
    └── mod.rs                       # ThemePalette, WingTheme

apps/examples/wing/rust/src/          # 示例应用
├── lib.rs                            # 应用入口
└── platform/                         # 平台实现
    ├── framebuffer.rs               # Window 实现
    ├── runner.rs                   # InputBridge
    └── input/                      # 输入类型
```

---

## 系统执行顺序

```
Startup
  └─ setup_wing_shell              # 创建所有壳层实体

PreUpdate
  ├─ wing_picking_system            # 命中检测
  ├─ wing_minimal_button_interaction_system  # 按钮交互
  ├─ wing_shell_interaction_system # 面板交互
  └─ wing_gesture_system           # 手势识别

Update
  ├─ wing_overlay_animation_system # 面板动画
  ├─ wing_shell_layout_system     # 基础布局
  ├─ wing_shell_stack_layout_system       # 堆叠布局
  ├─ wing_shell_overlay_layout_system     # 遮罩布局
  ├─ wing_shell_notification_panel_layout_system
  ├─ wing_shell_quick_controls_layout_system
  ├─ wing_shell_notification_cards_layout_system
  └─ wing_shell_overlay_card_layout_system

entity_sync
extract
  ├─ extract_view
  ├─ extract_wing_shell
  └─ queue_wing_primitives

render
```

---

## ShellMetrics 布局常量

```rust
// 屏幕尺寸
width()          = screen_size.x
height()         = screen_size.y
center_x()       = screen_size.x × 0.5
center_y()       = screen_size.y × 0.5

// 侧边距
side_inset()     = screen_size.x × 0.05  (5%)

// QuickControls
quick_controls_height()  = 自适应计算 (约 170-200px)
tile_size()              = 48-80px (自适应)

// 亮度条
brightness_height() = screen_size.y × 0.02  (2%)

// 通知卡片
notification_card_height()   = screen_size.y × 0.15  (15%)
notification_card_spacing()  = screen_size.y × 0.03  (3%)

// 文本
text_size()        = screen_size.y × 0.025  (2.5%)
text_size_small()  = text_size × 0.85

// 面板
notification_panel_height() = screen_size.y × 0.65  (65%)
```

---

## 当前实现状态

### ✅ 已完成

- Shell 实体在 MainWorld 中生成
- 平台输入通过 Window + InputBridge 进入 ECS 资源
- wing_picking_system 生成 Pointer<E> 事件
- Shell 状态通过系统更新 Transform/PickableBounds
- Shell 渲染通过 extractor 进入 RenderWorld
- queue_wing_primitives 生成壳层绘制命令

### 壳层骨架

- ✅ ShellRoot
- ✅ HomeSurface
- ✅ NotificationPanel (安卓风格)
- ✅ QuickControls (WiFi, Bluetooth, Airplane, Flashlight, DND, Auto-rotate)
- ✅ BrightnessControl
- ✅ NotificationCard (支持优先级)
- ✅ SurfacePreviewCard 堆叠布局
- ✅ OverlayLayer (遮罩)
- ✅ 手势系统 (滑动手势 + 长按)
- ✅ 主题系统 (Aurora ↔ Dusk)

### 动画系统

- ✅ 面板滑动动画 (ease_out_cubic)
- ✅ 遮罩透明度动画
- ✅ 主题切换过渡动画
- ✅ 模式切换时立即完成前一个动画

### ⚠️ 当前限制

- 最简 shell 骨架，非完整系统壳层
- 没有实际应用支持
- 没有通知数据动态更新
- 没有滚动支持（通知超出时）

---

## 运行命令

```bash
cd /home/uan-wsl2/codes/FeatherOS/nuttx
./wing_build.sh

# 运行
./nuttx
nsh> wing_rust
```

---

## 设计决策

### 为什么用 3D 对象表示 2D UI？

遵循 FHRE 的纯 3D 设计理念，所有 UI 元素都是 3D 对象。这允许：
- 任意 3D 变换（旋转、缩放、位移）
- z-depth 控制层次
- 未来支持 3D 空间 UI

### QuickControls 为什么用自适应布局？

移动/手表设备屏幕尺寸差异大，自适应布局确保：
- Tile 大小始终在良好触摸范围内 (48-80px)
- 列数根据宽度调整 (3-5 列)
- 固定 2 行适合大多数用例

### 为什么动画系统要立即完成前一个动画？

避免两个面板同时渲染的视觉问题。当从通知面板切换到应用切换器时，前一个面板会立即消失，新面板平滑进入。
