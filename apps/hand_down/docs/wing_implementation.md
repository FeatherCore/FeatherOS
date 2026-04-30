# Wing Shell 界面实现文档

本文重新整理当前 Wing Shell 的界面布局、运行时结构和后续实现方向。旧版文档中的 `Wing + FHRE` 分离式、纯 3D 壳层描述已经不再作为当前实现基线；历史背景可参考 `docs/wing_fhre_integration.md`，当前统一架构以 `docs/WING_UNIFIED_ARCHITECTURE.md` 和 `apps/wing/rust/src/` 为准。

当前 Wing 的定位是：

```text
NuttX + Rust 上的轻量声明式 ECS Shell/UI runtime
```

目标平台是高性能 MCU、低功耗 MPU、智能手表、MP4、小屏手机和复古掌机一类设备。实现优先级是轻量、无宏、固定容量、声明式 ECS、可软件渲染，并逐步兼容 2D/2.5D/GLES/OpenCL ES 等更强后端。

## 当前实现基线

主要源码位置：

```text
apps/wing/rust/src/
├── runtime.rs          # WingRuntime, frame loop, settings/resource/app integration
├── shell.rs            # Shell mode, input, layout composition, actions
├── shell_notification.rs # Fixed-capacity notification store
├── shell_style.rs      # Shell theme tokens and reusable visual styles
├── ui.rs               # UiFrame/UiSpec declarative UI layer
├── render/             # DrawList, software renderer, texture/font resource
├── platform/nuttx.rs   # NuttX platform bridge, framebuffer/input/resource/task
├── app.rs              # App registry and launch model
├── app_ui.rs           # App-side lightweight UI SDK
└── resource_file.rs    # Runtime filesystem resource inspection
```

构建和运行：

```bash
cd /home/uan-gpd/codes/FeatherOS/nuttx
./wing_build.sh
./nuttx

# NuttX NSH
wing_rust
```

`wing_build.sh` 必须保持可执行无误。当前 sim 构建会把 `apps/wing/resource/fs/` 复制到目标 ROMFS 的 `/etc/wing/resource/`，包括：

```text
/etc/wing/resource/images/shell/*.png
/etc/wing/resource/fonts/simhei.ttf
/etc/wing/resource/fonts/shell_workset.txt
/etc/wing/resource/icons/shell/*.svg
```

## 架构流水线

当前 Shell 不再依赖旧的 FHRE plugin/extractor 结构，而是走 Wing 自己的轻量 ECS/声明式 UI 管线：

```text
Platform input/settings/resource
        │
        ▼
WingRuntime frame loop
        │
        ├─ Phase::Input       shell_input_system
        ├─ Phase::Update      shell_transition_system
        ├─ Phase::Layout      compose_shell_ui_system
        └─ Phase::RenderBuild build_draw_list_system
        │
        ▼
UiFrame / UiSpec
        │
        ▼
World component storage
        │
        ▼
DrawList
        │
        ▼
SoftwareRenderer or future backend
```

`UiFrame` 是当前声明式 UI 的核心中间层。页面每帧用稳定 `UiKey` 声明 UI spec，`World::apply_ui_frame()` 根据 revision 更新实体、布局、可见性和按钮行为。这个方式避免虚拟 DOM、宏、反射和动态分配式 diff。

UI diff 会按变化元素输出多个 damage rect，再由固定容量 `DirtyRegion` 合并和裁剪；这避免小范围变化被过早合成为一个大刷新矩形，更适合 MCU/低功耗 MPU 的局部刷新和低带宽 framebuffer。Runtime 入口会把 dirty rect 裁剪到 framebuffer bounds，dirty 覆盖率达到约 75% 屏幕面积时会退回 full redraw，避免页面切换或大动画阶段产生过度碎片化的刷新列表。每帧提交前还会先尝试把 dirty rect 扩展到 16x16 tile 网格：只有额外重绘面积低于阈值时才采纳，用来适配 MCU/MPU 上常见的 tile buffer、DMA2D/PXP/VG-Lite 局部刷新边界。随后再按固定预算做一次 greedy compact：当多个小 rect 合并带来的额外重绘面积可控时，把 dirty rect 数压到较低目标，减少软件 renderer 的重复 DrawList 扫描和 NuttX framebuffer present 次数。

软件渲染管线借鉴 LVGL 的失效区域、scissor clip 和 draw descriptor 思想，但不引入 LVGL 依赖。当前 `DrawCmd` 已经具备 bounds 计算，`SoftwareRenderer` 在每个 dirty clip 内会先跳过不相交的绘制命令，再进入具体像素绘制路径。UI spec 可以携带可选裁剪矩形，构建 DrawList 时下发轻量 `SetClip` 命令，软件后端把它与 dirty clip 相交；通知列表已经使用这个能力把卡片绘制限制在面板内部。`DrawList` 会记住当前 clip 状态，连续相同裁剪区域只输出一次 `SetClip`，离开裁剪区时才恢复默认状态，避免 clipped UI 把固定容量命令队列快速塞满。Fill 类命令支持纯色矩形、抗锯齿圆角、圆形以及垂直圆角渐变，Shell 的面板、通知卡片、快捷开关和应用卡片都通过同一套 `DrawStyle` 下发到底层 draw command。`DrawCmd` 同时会映射到轻量 `DrawTaskKind`，例如 Clear/Clip/Fill/Shadow/Image/Vector/Surface/Text/Effect；`DrawList` 可以根据 `RendererCapabilities` 生成 `RenderPlan`，把本帧任务拆成 native/software fallback/unsupported 三组，并通过无分配 `DrawRouteRuns` 迭代器按 route 和 clip 生成有序 run。`SoftwareRenderer` 会识别 Shell 稳定输出的 `ClearGradient + fullscreen RGB565 wallpaper` 背景前缀，在每个 dirty clip 内一次合成渐变和壁纸；运行时 PNG 已解码到当前窗口尺寸时会走直接像素采样，避免背景层每帧重复 bilinear 采样和两次写 framebuffer。这个快路径是后续 layer cache 的边界，但当前不额外保存全屏缓存 buffer，避免在 MCU 档位上直接翻倍占用内存。`RenderPlan` 同时保存 run summary 和 `RenderLayerSummary`，包括 total/native/fallback/unsupported/clipped runs、最大 run 长度、背景层种类、`RenderBackgroundRoute` 和被背景快路径吞掉的前缀命令数，用来判断后续是否需要合批、图层缓存、硬件 background plane 或减少 route 切换。Runtime 只生成一次 render plan，并同时传给 `RendererBackend::draw()` 与诊断状态；`SoftwareRenderer::draw_routes()` 可以按 `DrawRouteMask` 执行 supported 或 software fallback 子集，且按 run 顺序重放 `SetClip` 状态，避免硬件 native 与软件 fallback 混合执行时破坏透明层和文本的叠放顺序。`BackgroundPlaneRenderer` 则是第一版可替换后端骨架：它声明 `RendererKind::Gpu2d` 和 `background_plane`，但仍用软件 framebuffer 呈现，可通过 `wing_rust --renderer=plane` 验证 `BG PLANE` 路径；它泛型于 `BackgroundPlaneOps`，每帧只在 `NativePlane` route 时提交 `BackgroundPlaneSubmission`，默认 sim ops 会校验 RGB565 纹理并记录提交/拒绝次数。系统页的 `DRAW` 行显示当前主要渲染任务类型，`LAYER` 行显示 `BG FAST/BG PLANE/BG GRAD/BG MISS/BG CAP/NONE`，`DIRTY` 行显示最近一帧 dirty 提交形态，包括 `TILED` 这种已对齐到 tile 网格的提交；`HEALTH` 行在后端缺少 draw task 能力时显示 `DRAW CAP`，背景层能力不足时显示 `BG CAP`，背景资源缺失时显示 `BG MISS`，需要软件回退时显示 `DRAW SW`，run 过碎时显示 `DRAW RUN`，dirty 过碎时显示 `DIRTY FRG`。这为后续继续拆分 draw task、图层缓存和硬件后端分发打基础。

## 坐标和层级

坐标系以屏幕左上角为原点：

```text
(0, 0) ───────────────► x
  │
  │
  ▼ y
```

尺寸来自平台 framebuffer：

```rust
width: u16
height: u16
```

当前 `Layer`：

```rust
Always
Home
Notification
AppSwitcher
ExternalApp
Settings      // legacy shell page, not the current default-app path
SystemInfo    // legacy shell page, not the current default-app path
```

当前 `ShellMode` 与页面层一一对应：

```rust
Home
Notification
AppSwitcher
ExternalApp
Settings      // legacy
SystemInfo    // legacy
```

只有当前 `ShellMode` 对应的 layer 参与命中和绘制，`Always` 用于状态栏，但 `ExternalApp` 前台应用态会隐藏状态栏，让应用 surface 占用完整 framebuffer。这一点很重要：通知页和应用切换器不会同时抢输入，也不会残留在可点击路径里。

## Home 页面

当前 Home 是一个轻量入口页：

```text
┌────────────────────────────┐
│ WING                 12:00 │
│                            │
│       ┌──────┐ ┌──────┐    │
│       │ SET  │ │ TERM │    │
│       └──────┘ └──────┘    │
│       ┌──────┐ ┌──────┐    │
│       │ NOTE │ │ VIEW │    │
│       └──────┘ └──────┘    │
│                            │
│ SWIPE DOWN: NOTIFY UP: ... │
└────────────────────────────┘
```

当前布局参数来自 `compose_home()`：

| 元素 | 当前值 |
|------|--------|
| App grid | 2 x 2 |
| Cell | `96 x 96` |
| Gap | `28` |
| Grid top | `height / 2 - 88` |
| Buttons | Settings, Terminal, Notification, Preview |
| Icons | `VectorIcon`，运行时从 Bootstrap SVG 解析并缓存 A8 mask |

Home 背景优先使用文件系统中的原始 PNG 壁纸，运行时解码并缩放到窗口大小；如果 PNG 资源不可用，则使用构建期生成的 RGB565 payload fallback。

当前 Aurora 主题的视觉目标参考：

```text
apps/wing_old/preview/theme_aurora_preview.png
```

实现上优先靠近这个方向：壁纸作为第一视觉层，大字号时间信息居中，浅色玻璃卡片浮在背景上，蓝色作为可交互强调色，底部使用圆形入口/返回控件。当前软件 renderer 还没有真实 blur 和 shadow，先用透明圆角层、暗色垫片和层叠卡片模拟深度。

## 通知页面

通知页面对应：

```rust
ShellMode::Notification
Layer::Notification
compose_notification()
```

进入方式：

| 来源 | 操作 |
|------|------|
| Home | 下滑 |
| Home | 点击 `NOTE` |
| 其他页面 | 可通过 `ButtonAction::OpenNotification` 进入 |

退出方式：

| 当前页 | 操作 |
|--------|------|
| Notification | 上滑返回 `return_to` |
| 任意页 | Home/Escape 返回 Home |

### 当前实现

当前实现已经从空状态骨架升级为接近 Aurora preview 的“控制中心 + 通知卡片”：

```text
┌────────────────────────────┐
│ WING                 12:00 │
│  ┌──────────────────────┐  │
│  │ NOTIFICATIONS      ◎  │  │
│  │ LIGHT ━━━━━━━━━       │  │
│  │ [WIFI] [BT] [AIR]     │  │
│  │ [DND]  [LITE] [SYNC]  │  │
│  │ [MESSAGE] [MAIL]      │  │
│  │ [WEATHER] [SYSTEM]    │  │
│  │ SWIPE UP TO RETURN    │  │
│  └──────────────────────┘  │
└────────────────────────────┘
```

当前 panel rect：

```rust
Rect::new(18, 70, width - 36, height - 110)
```

当前内容已经从硬编码卡片升级为固定容量 `NotificationStore`，页面只消费 store 中的 `NotificationEntry`：

| 区块 | 当前内容 |
|------|----------|
| Brightness | 显示 `ShellState::brightness` 的进度条 |
| Quick controls | WiFi、BT、Air、DND、Lite、Sync |
| Notification cards | `NotificationStore` 中的 Message、Mail、Weather、System、App lifecycle |
| Icon source | `/etc/wing/resource/icons/shell/*.svg` -> runtime `SvgStore` parse/cache，缺失时回退内置 Shell Bootstrap SVG，不依赖 PNG/RGB565/A8 图标资源 |

### 旧设计可保留的方向

旧文档中的通知页设计仍然值得参考，但需要按当前轻量 runtime 重写：

```text
┌────────────────────────────┐
│ WING                 12:00 │
│ ┌────────────────────────┐ │
│ │ NOTIFICATIONS          │ │
│ │ ━━━━━━━━━━━━━━━        │ │  Brightness
│ │ [WiFi] [BT] [DND]      │ │  Quick settings
│ │ [Air]  [Lite] [Save]   │ │
│ │ ┌──────────────────┐   │ │
│ │ │ App notification │   │ │
│ │ └──────────────────┘   │ │
│ │ ┌──────────────────┐   │ │
│ │ │ System event     │   │ │
│ │ └──────────────────┘   │ │
│ └────────────────────────┘ │
└────────────────────────────┘
```

建议拆成三块：

| 区块 | 作用 | 当前状态 |
|------|------|----------|
| Brightness row | 显示/调整 `ShellState::brightness` | 已显示，暂未接输入 |
| Quick settings | 网络、勿扰、灯光、同步等快捷入口 | 已接 `ButtonAction` 和 `ShellState.quick` |
| Notification cards | 应用/系统消息列表 | 已接 `NotificationStore`，真实消息源继续补齐 |

### 推荐数据模型

通知列表应保持固定容量，避免动态分配和复杂生命周期：

```rust
pub struct NotificationEntry {
    pub id: u16,
    pub icon: NotificationIcon,
    pub title: &'static str,
    pub body: &'static str,
    pub time: &'static str,
    pub priority: NotificationPriority,
}

pub enum NotificationPriority {
    Normal,
    High,
}

pub struct NotificationStore {
    entries: FixedList<NotificationEntry, NOTIFICATION_CAPACITY>,
    revision: u32,
}
```

通知卡片布局建议：

| 参数 | 建议 |
|------|------|
| side inset | `24..44`，随宽度 clamp |
| card height | `56..88`，随屏幕高度 clamp |
| gap | `8..14` |
| max visible | 小屏 3 条，竖屏 4 条，横屏 2-3 条 |
| 超出处理 | 先显示 `+N`，后续再做滚动 |

### 通知页实现计划

短期可以按这个顺序推进：

1. 将 brightness row 接入输入，支持触摸/旋钮调节。
2. 接入真实系统/应用通知来源，替换当前默认 seed 数据。
3. 将 quick controls 状态持久化到 settings 或 platform capability。
4. 后续加入滚动、通知清除动作和 `+N` 折叠提示。

## 应用切换器 / 卡片预览

应用切换器对应：

```rust
ShellMode::AppSwitcher
Layer::AppSwitcher
compose_app_switcher()
```

进入方式：

| 来源 | 操作 |
|------|------|
| Home | 上滑 |
| Home | 点击 `VIEW` |
| Detached app launch | 回到 AppSwitcher 显示 launch banner |

退出方式：

| 当前页 | 操作 |
|--------|------|
| AppSwitcher | 下滑返回 `return_to` |
| 任意页 | Home/Escape 返回 Home |

### 当前实现

当前 AppSwitcher 不是旧的静态 surface demo，而是从 `AppRegistry` 读取应用列表：

```rust
Settings
System
Terminal
Surface
```

点击预览项会触发：

```rust
ButtonAction::LaunchApp(app.id)
```

`launch_app()` 根据 `LaunchKind` 决定进入：

| LaunchKind | 结果 |
|------------|------|
| `Builtin(...)` | 兼容旧 Shell 内部页面 |
| `PlatformTask(Detached)` | NuttX task 启动后回到 AppSwitcher |
| `PlatformTask(WingManaged)` | 进入 ExternalApp surface 页面；Settings/System/Terminal/Surface 默认走这条路径 |

### Cards 模式

`PreviewEffect::Cards` 是当前最稳妥的默认应用切换器样式。

当前布局来自 `compose_card_preview()`，默认使用 2 x 2 应用卡片：

```text
┌────────────────────────────┐
│ WING                 12:00 │
│ ┌────────────────────────┐ │
│ │ PREVIEW                │ │
│ │                        │ │
│ │  ┌─────────┐ ┌────────┐│ │
│ │  │Settings │ │System  ││ │
│ │  └─────────┘ └────────┘│ │
│ │  ┌─────────┐ ┌────────┐│ │
│ │  │Terminal │ │Surface ││ │
│ │  └─────────┘ └────────┘│ │
│ │   phone camera music... │ │
│ │ CARDS                  │ │
│ │ SWIPE DOWN TO RETURN   │ │
│ └────────────────────────┘ │
└────────────────────────────┘
```

当前参数：

| 参数 | 当前值 |
|------|--------|
| card count | `apps.len().min(4)` |
| layout | 2 x 2 |
| side inset | `44` |
| card width | `(width - 88 - gap) / 2` |
| card height | `72` compact / `116` normal |
| content | vector icon, app name, short status subtitle |
| icon rail | Phone, Camera, Music, Check, Close, Wing |

这个 Cards 模式已经可以作为“应用切换器”的基础路径；下一步重点不是重画卡片，而是把 WingManaged surface 的最近帧缩略图接进卡片内容。

### Soccer / Cube 模式

当前还有两个可选预览效果：

| 模式 | 触发条件 | 说明 |
|------|----------|------|
| Soccer | basic 2D + circle | 用圆形节点做 2.5D 预览布局 |
| Cube | basic 2D + cube effect capability | 使用 `DrawCmd::Effect(PreviewCube)` |

效果选择通过 `PreviewEffect` 和 renderer capability 决定：

```rust
Cards  -> basic 2D
Soccer -> basic 2D + circle
Cube   -> basic 2D + supports_cube_preview()
```

Settings 页的 `PREVIEW` 行可以切换 Cards/Soccer/Cube。如果当前后端不支持某个效果，会回退到支持的模式。

### 应用切换器是否可以参考旧卡片预览

可以参考，但不要照搬。

旧设计里的“卡片预览栈”非常适合小屏设备，因为它：

- 信息密度高。
- 单手滑动和点击目标清晰。
- 只需要基础 2D 绘制即可稳定运行。
- 可以自然承载外部 app surface 的缩略图。

但当前 Wing 应该把它实现为“应用注册表 + surface 快照 + fixed-list UI”的组合，而不是静态 demo 卡片：

```text
AppRegistry
  -> AppManifest
  -> preview item
  -> optional SurfaceFrame thumbnail
  -> LaunchApp(AppId)
```

后续真实应用切换器建议：

1. Cards 模式作为默认稳定路径。
2. 卡片内容优先显示 app icon/name/status。
3. 对 WingManaged surface，卡片中嵌入最近一帧 surface 缩略图。
4. 对 Detached task，显示启动状态或最近错误。
5. Soccer/Cube 作为可选 showcase，不作为基础交互依赖。

## 外部应用页面

`ShellMode::ExternalApp` 用于显示 WingManaged surface。

典型路径：

```text
AppSwitcher card click
  -> launch_app()
  -> PlatformTask(WingManaged)
  -> ShellMode::ExternalApp
  -> DrawCmd::Surface
```

当前 Settings / System / Terminal / Surface demo 已经能通过 task runner 使用 Wing-managed surface。Shell 在启动独立 app 时会按 `external_surface_rect(width, height)` 创建全屏 surface，并在 capability 上限内直接使用最终显示区域尺寸，避免把 `320x360` 小画布整体放大造成字体和 SVG 图标模糊。ExternalApp 页面是前台应用态：应用 surface 绘制在完整屏幕上，Shell 不叠加默认退出按钮；退出依赖应用自己的返回键、系统手势或 Home/Escape。

默认 Wing-managed app 的运行配置统一为 `priority=110`、`stack=64KiB`。这里不沿用最小 NuttX task stack：Rust App UI 会在一帧内组合 SVG、字体、固定容量命令队列和 RGB565 surface，4KiB 栈在 sim 和真实板上都容易表现为退出卡死或随机不能操作。

应用主动退出时，`WingSurface` 会在关闭 dirty/input queue 前发送一个轻量 close 事件。Shell 收到后立即释放 active task、销毁 surface，并把 `ExternalApp` 或指向 `ExternalApp` 的 `return_to` 修正回 Home；这样即使 `waitpid(WNOHANG)` 没有及时回报，默认应用的返回键也不会把 Shell 留在空的前台应用层。

前台应用态的系统手势由 Shell 优先处理，再把普通 pointer 事件分发给应用：

| 输入 | 动作 |
|------|------|
| 下滑 | 打开 Notification，`return_to = ExternalApp` |
| 上滑 | 打开 AppSwitcher，`return_to = ExternalApp` |
| 左滑/右滑 | 顶层应用返回 Home，并关闭 focused task |
| 普通点击/拖动 | 分发给当前 WingManaged surface |

## 默认应用

Settings 是独立 WingManaged 默认应用，当前承载几类配置：

| Row | 作用 |
|-----|------|
| BRIGHTNESS / LEVEL | 亮度状态和步进调节 |
| HAPTIC / MOTION | 触感和低动画开关 |
| THEME | Aurora / Dusk |
| PREVIEW | Cards / Soccer / Cube |

System 也已经迁到独立 WingManaged 默认应用，用同一套 App UI / SVG / TTF 路径绘制。当前 System app 展示稳定的系统摘要，避免直接耦合 Shell 内部诊断结构：

| Row | 作用 |
|-----|------|
| RUNTIME | NuttX Rust |
| DISPLAY | 当前 WingManaged RGB565 surface 方向 |
| PIPELINE | 软件绘制为基础，后续可接 2D/GPU plane |
| RESOURCES | 文件系统 PNG/SVG + 编译期 app 私有 SVG |
| FONT | 默认 SimHei TTF |
| THEME / PREVIEW | Shell 传入的设置快照 |
| INPUT | 触感/低动画状态 |

Terminal 也已经从 detached `nsh` 入口改为独立 WingManaged 默认应用。当前 sim/wing 配置启用 NuttX `PSEUDOTERM`，Terminal 通过轻量 C bridge 打开 `/dev/ptmx`、创建 slave tty，并把 `nsh_consolemain` spawn 到该 tty 上；Rust app 侧只保留固定行列的无分配文本缓冲，按帧读取 PTY 输出，无法显示的字符会退化为占位字符。页面提供一个小屏 token 命令编辑器：常用命令可一键发送，短命令可通过 command/path token、`SP/BK/CLR/ENT` 拼接后发给 NSH。若目标板没有启用 PTY/NSH library，页面会显示 `NO PTY` 并保持可退出、可切换：

| Row | 作用 |
|-----|------|
| SESSION | `PTY RUN` / `NO PTY` / `PTY ERR` |
| SHELL | `NSH LIVE` / `NSH OFF` / `NSH FAIL` |
| SURFACE | RGB565 WingManaged surface |
| BACKEND | `PSEUDOTERM` / `CONFIG OFF` / `OPEN FAIL` |
| HELP / LS / PS / FREE | 触摸命令按钮，向 NSH 写入常用调试命令 |
| command/path token | `ls/cat/pwd/clear` 与 `/etc/wing/resource` 等 token 拼接短命令 |
| SP / BK / CLR / ENT | 空格、退格、清空、发送当前输入缓冲 |

Surface demo 是默认应用里的渲染/交互实验场，仍然使用 App UI SDK 而不是 Shell 内部绘制路径。它会读取 Shell 传入的设置快照：Aurora/Dusk 会改变背景与 accent，Brightness 作为默认强度，PreviewEffect 决定初始 demo tab，ReduceMotion 会降低刷新率并固定动态元素，便于在低功耗目标上验证“同一套声明式 UI 在不同节能策略下的表现”。

这些页面是当前 Shell 迭代时最重要的验证入口。

## 资源和字体

当前 Shell 资源分两类：

| 类别 | 路径 | 运行时 |
|------|------|--------|
| Shell PNG 背景 | `apps/wing/resource/fs/images/shell/*.png` | `/etc/wing/resource/images/shell/*.png`，横屏 `640x480`，竖屏 `480x640` |
| 默认字体 | `apps/wing/resource/fs/fonts/simhei.ttf` | `/etc/wing/resource/fonts/simhei.ttf` |
| Shell glyph 工作集 | `apps/wing/resource/fs/fonts/shell_workset.txt` | `/etc/wing/resource/fonts/shell_workset.txt` |
| Shell 图标 | `apps/wing/resource/fs/icons/shell/*.svg` -> `VectorIcon` | Wing 优先从 `/etc/wing/resource/icons/shell/*.svg` 读取，按尺寸缓存 A8 mask；缺失时回退到内置 Bootstrap SVG |
| 默认应用图标 | `apps/wing/resource/apps/<app>/icons/*.svg` -> `AppUiIcon` / `AppVisual::VectorIcon` | 独立 app 的私有 SVG 在构建时嵌入自身 App UI SVG 表，例如 `apps/settings/icons/*.svg`；不放入 Shell runtime filesystem，不引入 PNG 图标或 LVGL 依赖 |
| fallback payload | `apps/wing/resource/generated/*.bin` | `/etc/wing/wing_*.bin`，背景 fallback 默认长边 320 像素 |

字体策略：

- 默认字体是 `simhei.ttf`。
- Shell `DrawCmd::Text` 与 App UI RGB565 surface 都通过 `FontStore` 栅格化。
- 所有文本优先从字库获取。
- 解析不出来的字符绘制缩放后的占位符。
- `FontStore` 带可配置上限 glyph cache，Shell 软件渲染器会直接复用缓存字形，避免每帧重复解析和栅格化常用文本。
- glyph cache 默认 `BASE` 档 96 个 cell；`WING_GLYPH_CACHE_PROFILE=tiny` 为 32，`large` 为 192，也可以用 `WING_GLYPH_CACHE_CAPACITY` 指定 8..256 范围内的自定义容量。
- Runtime 加载默认字体后会优先读取 `/etc/wing/resource/fonts/shell_workset.txt`，预热 Shell 首屏、通知页和系统页的常用 glyph 工作集；文件缺失或格式不匹配时回退到内置工作集。预热只在 cache 有空间时插入，不触发 LRU 淘汰。
- `FontCacheSummary` 会统计命中率、填充率、eviction 和缓存 glyph bitmap 字节数。
- 运行时 diagnostics 仍记录 glyph cache 档位、命中率、内存压力和 eviction，后续可以通过独立 System app 的 SDK 通道展示为 `GLYPH/GHIT` 行。
- SVG 图标走同样的 cache 诊断语义：`WING_SVG_CACHE_PROFILE=tiny` 为 16 个 raster mask，默认 `BASE` 为 40，`large` 为 64；`WING_SVG_CACHE_CAPACITY` 可在 4..96 范围内覆盖。Shell 通过 `/etc/wing/resource/icons/shell/*.svg` 加载文件系统图标并回退到内置 Shell Bootstrap SVG；默认 App UI surface 通过编译期私有 SVG 表初始化自己的 `SvgStore`。两条路径都按目标尺寸 rasterize 到 A8 mask，后续 System app 可以通过稳定 SDK 通道显示 Shell 图标来源和 SVG cache 状态。

## 输入和动作

当前全局交互：

| 输入 | 当前模式 | 动作 |
|------|----------|------|
| 下滑 | Home | 打开 Notification |
| 上滑 | Home | 打开 AppSwitcher |
| 上滑 | Notification | 返回 `return_to` |
| 下滑 | AppSwitcher | 返回 `return_to` |
| 左滑/右滑 | Notification/AppSwitcher | 返回 `return_to` |
| 下滑/上滑 | ExternalApp | 打开 Notification/AppSwitcher |
| 左滑/右滑 | ExternalApp | 关闭当前 app 并返回 Home |
| Escape/Home | 任意 | 关闭当前 app 并返回 Home |
| 鼠标释放/触摸释放 | 命中 button | 执行 `ButtonAction` |

当前 `ButtonAction`：

```rust
OpenSettings
OpenNotification
OpenAppSwitcher
LaunchTerminal
LaunchApp(AppId)
BackHome
ToggleTheme
TogglePreviewEffect
ToggleWifi
ToggleBluetooth
ToggleAirplane
ToggleDnd
ToggleLight
ToggleSync
```

## 动画

当前页面切换使用 `SlideTransition`：

| 进入页面 | 动画方向 |
|----------|----------|
| Notification | FromTop |
| AppSwitcher | FromBottom |
| Settings/System/ExternalApp | FromRight |
| Return Home | FromLeft |

`ShellState::reduce_motion` 为 true 时关闭进入动画。

后续通知页和卡片预览的拖动进度可以继续复用这套动画状态，但不要让多个页面同时进入可点击层。

## 当前限制

当前还不是完整系统壳层，限制如下：

- 通知页已有固定容量 `NotificationStore`，但真实系统/应用消息来源还没有完整接入。
- Quick settings 已接本地状态和动作，但还没有 platform capability 或持久化来源。
- AppSwitcher 已经基于 `AppRegistry`，卡片还没有 surface 缩略图。
- Cards 布局已有 compact/normal 两档，小屏细节还可以继续调。
- 字体已有按内存档位配置、启动预热和诊断的 glyph cache，但还没有离线 glyph atlas 或按语言集裁剪的字体包。
- 资源已有 PNG/TTF 运行时读取，JPEG 仍以 header 识别为主。

## 推荐下一步

优先级建议：

1. 接入真实系统/应用通知来源，并支持清除或折叠。
2. 将 quick controls 状态持久化到 settings 或 platform capability。
3. 给 WingManaged surface 添加最近帧缩略图，作为 AppSwitcher 卡片内容。
4. 增加离线 glyph atlas 或字体子集生成，把常用 Shell 字符从完整 `simhei.ttf` 中拆成更小的设备档。
5. 再考虑 Soccer/Cube 的细节增强，把它们作为可选视觉效果，而不是基础交互前提。
