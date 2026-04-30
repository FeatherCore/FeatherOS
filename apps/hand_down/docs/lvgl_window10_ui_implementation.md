# LVGL Window 10 Mobile UI Implementation Notes

本文档梳理 `/home/uan-gpd/codes/FeatherOS/apps/examples/lvgl_window10` 当前 UI 实现。

当前例程是 `/home/uan-gpd/codes/windows-10-mobile-lvgl` 的 NuttX `sim:lvgl_fb` 迁移版。视觉目标是 Windows 10 Mobile / Lumia 风格，工程目标是先用 LVGL 跑通一个质量较高的手机壳 demo，后续再从中提取适合 wing 的绘制管线、布局组织和交互经验。

## 当前结论

- 这是一个 **LVGL + SquareLine Studio 生成 UI + 少量手写运行时 glue** 的 demo。
- UI 设计尺寸是 **320x480 竖屏**，NuttX `sim:lvgl_fb` 配置也固定为 `320x480`、`16bpp`。
- 所有图片资源目前都是 `ui/images/*.c` 中的 C 数组，直接编译进二进制，没有从 NuttX 文件系统动态加载。
- 字体使用 LVGL 内置 Montserrat 字体族，没有外部矢量字体文件。
- 当前 demo 的 UI 层比 wing shell 原型更像“完整手机系统壳”：开始屏、应用列表、锁屏、通知面板、状态栏、导航栏、设置、新闻、Stars、Thermal IR、通用 App 示例、WiFi 弹窗、系统键盘、Alert。
- 运行时偏好值目前只保存在内存中的 `g_prefs`，不是持久化存储。
- 适合作为 wing 后续“绘制质量、动画、组件结构、手机壳交互”的参考，但不适合作为 wing 的最终代码结构直接照搬。

## 目录结构

```text
apps/examples/lvgl_window10/
├── CMakeLists.txt
├── Kconfig
├── LICENSE.windows-10-mobile-lvgl
├── Make.defs
├── Makefile
├── README.md
├── lvgl_window10.c
└── ui/
    ├── CMakeLists.txt
    ├── components/ui_comp_hook.c
    ├── filelist.txt
    ├── images/*.c
    ├── screens/*.c
    ├── stars.h
    ├── ui.c
    ├── ui.h
    ├── ui_common.c
    ├── ui_common.h
    ├── ui_events.c
    ├── ui_events.h
    ├── ui_helpers.c
    └── ui_helpers.h
```

核心职责如下：

| 文件 | 职责 |
|---|---|
| `lvgl_window10.c` | NuttX 入口、LVGL/NuttX 显示输入初始化、主循环、时间刷新、内存偏好项和硬件回调 stub |
| `Kconfig` | `CONFIG_EXAMPLES_LVGL_WINDOW10`、栈大小、优先级、触摸输入路径 |
| `Makefile` | NuttX make 构建入口，编译主文件、UI 公共文件、屏幕文件、图片 C 数组 |
| `CMakeLists.txt` | CMake/NuttX 构建入口，功能等价于 Makefile |
| `ui/ui.h` | SquareLine 导出的全局对象声明、图片声明、运行时函数声明 |
| `ui/ui.c` | 全局 UI 状态、应用/设置/快速操作表、动画、导航事件、偏好加载、动态组件装配 |
| `ui/ui_common.c` | 手写公共组件工厂：磁贴、列表项、开关、按钮、滑条、下拉框、WiFi 列表项等 |
| `ui/screens/*.c` | SquareLine 生成的各个屏幕的静态对象树 |
| `ui/images/*.c` | SquareLine 转换后的图片描述符和像素数据 |
| `ui/ui_helpers.c` | SquareLine helper：属性设置、动画回调、screen change、flag/state 修改 |
| `ui/ui_events.c` | 弱符号硬件/业务事件 stub，NuttX 迁移后大部分保持空实现 |
| `ui/stars.h` | 静态 GitHub stargazers 名单 |

## 构建与 NuttX 集成

### 构建脚本

`nuttx/lvgl_build.sh` 做四件事：

1. `make distclean`
2. 准备本地 LVGL 源码
3. 配置 `sim:lvgl_fb`
4. `make -j`

默认 LVGL 源码来自：

```sh
LVGL_SRC_REPO=/home/uan-gpd/codes/lvgl
LVGL_REF=v9.2.1
```

脚本会把本地 LVGL worktree 准备到：

```text
apps/graphics/lvgl/lvgl
```

如果本地 ref 不存在，则回退给 NuttX LVGL package 自己处理下载。

### NuttX defconfig

`nuttx/boards/sim/sim/sim/configs/lvgl_fb/defconfig` 当前和 demo 相关的关键配置：

```text
CONFIG_EXAMPLES_LVGL_WINDOW10=y
CONFIG_EXAMPLES_LVGL_WINDOW10_STACKSIZE=65536
CONFIG_INIT_ENTRYPOINT="lvgl_window10_main"
CONFIG_LV_COLOR_DEPTH_16=y
CONFIG_LV_FONT_MONTSERRAT_10=y
CONFIG_LV_FONT_MONTSERRAT_12=y
CONFIG_LV_FONT_MONTSERRAT_18=y
CONFIG_LV_FONT_MONTSERRAT_20=y
CONFIG_LV_FONT_MONTSERRAT_22=y
CONFIG_LV_FONT_MONTSERRAT_48=y
CONFIG_LV_USE_CLIB_MALLOC=y
CONFIG_LV_USE_CLIB_SPRINTF=y
CONFIG_LV_USE_CLIB_STRING=y
CONFIG_LV_USE_LOG=y
CONFIG_LV_USE_NUTTX=y
CONFIG_LV_USE_NUTTX_TOUCHSCREEN=y
CONFIG_SIM_FBWIDTH=320
CONFIG_SIM_FBHEIGHT=480
CONFIG_SIM_FBBPP=16
```

### NuttX 应用入口

`Makefile` 中：

```make
PROGNAME  = lvgl_window10
MAINSRC   = lvgl_window10.c
```

NuttX 会生成内置入口 `lvgl_window10_main`，因此 defconfig 中 `CONFIG_INIT_ENTRYPOINT="lvgl_window10_main"` 可以让 sim 启动后直接进入 demo，而不是进入 NSH 再手动运行应用。

### 编译源文件

当前 Makefile 编译：

```make
CSRCS += ui/ui.c
CSRCS += ui/ui_common.c
CSRCS += ui/ui_events.c
CSRCS += ui/ui_helpers.c
CSRCS += ui/components/ui_comp_hook.c
CSRCS += $(wildcard ui/screens/*.c)
CSRCS += $(wildcard ui/images/*.c)
```

也就是说：

- 所有屏幕都会编译。
- 所有图片 C 数组都会编译。
- 没有从 `windows-10-mobile-lvgl` 原工程动态引用源码。
- 没有 `upstream/` 目录参与编译。

## 运行入口与主循环

`lvgl_window10.c` 的主流程：

1. 如果需要，执行 `boardctl(BOARDIOC_INIT, 0)`。
2. 调用 `lv_init()`。
3. 调用 `lv_nuttx_dsc_init(&info)`。
4. 如果配置 LCD，设置 `info.fb_path = "/dev/lcd0"`。
5. 如果配置触摸屏，设置 `info.input_path = CONFIG_EXAMPLES_LVGL_WINDOW10_INPUT_DEVPATH`，默认 `/dev/input0`。
6. 调用 `lv_nuttx_init(&info, &result)` 创建 LVGL display 和 input device。
7. 初始化内存偏好项默认值：

```text
bg_type = 1
bg_img = 5
lock_img = 2
start_opa = 200
nav_opa = 200
```

8. 调用 `ui_init()` 创建 UI。
9. 调用 `lvgl_window10_update_time()` 刷新状态栏和锁屏时间。
10. 进入循环：

```c
while (1)
{
  lvgl_window10_update_time();
  idle = lv_timer_handler();
  idle = idle ? idle : 1;
  usleep(idle * 1000);
}
```

如果启用 `CONFIG_LV_USE_NUTTX_LIBUV`，则改用 `lv_nuttx_uv_init()` 和 `uv_run()` 驱动事件循环。

## 偏好项系统

当前实现了一个极小的内存 key-value 存储：

```c
#define PREF_SLOTS 24
#define PREF_KEY_LEN 24

struct pref_slot_s
{
  char key[PREF_KEY_LEN];
  uint32_t value;
  bool used;
};
```

公开接口：

```c
void setPrefInt(const char *key, uint32_t value);
uint32_t getPrefInt(const char *key, uint32_t def);
void setPrefBool(const char *key, bool value);
bool getPrefBool(const char *key, bool def);
```

重要限制：

- 只支持 `uint32_t` 和 bool。
- 最多 24 个 key。
- 只存在内存中，重启丢失。
- 没有文件系统持久化。
- 没有 namespace，也没有类型检查。

当前使用的偏好 key：

| key | 用途 |
|---|---|
| `bg_type` | 背景类型，`0=None`，`1=Picture` |
| `bg_img` | 桌面背景图片索引 |
| `lock_img` | 锁屏背景图片索引 |
| `start_opa` | 开始屏磁贴透明度 |
| `nav_opa` | 底部导航栏透明度 |
| `nav_tint` | 底部导航栏是否使用主题色 |
| `theme_color` | 当前 accent color |
| `brightness` | 亮度滑条值 |
| `timeout` | 屏幕超时下拉框选项 |
| `wifi_state` | WiFi 开关状态 |

## UI 初始化流程

`ui_init()` 位于 `ui/ui.c`，执行顺序：

1. 初始化 LVGL 默认主题：

```c
lv_theme_default_init(dispp,
                      lv_palette_main(LV_PALETTE_BLUE),
                      lv_palette_main(LV_PALETTE_RED),
                      true,
                      LV_FONT_DEFAULT);
```

2. 创建所有屏幕：

```c
ui_homeScreen_screen_init();
ui_settingsScreen_screen_init();
ui_appScreen_screen_init();
ui_newsScreen_screen_init();
ui_starsScreen_screen_init();
ui_thermalScreen_screen_init();
```

3. 调用 `init_custom()` 动态补齐 SquareLine 没有静态展开的内容。
4. 创建隐藏的 `ui____initial_actions0` 初始动作屏。
5. 先加载 `ui____initial_actions0`，触发初始动作事件。
6. 再加载 `ui_homeScreen`。

`ui_event____initial_actions0()` 在 `LV_EVENT_SCREEN_LOAD_START` 做：

- 把 `ui_lockScreenPanel` 的 Y 位置设为 `0`，因此首次进入时显示锁屏覆盖层。
- 为所有 live tile 启动循环动画。
- 为 Cortana 圆形图标启动 pulse 动画。

## 运行时数据表

### 应用表 `apps[]`

`apps[]` 用于 All Apps 列表和应用启动。

| code | name | icon |
|---|---|---|
| `0xA900` | Edge | `ui_img_edge_ic_png` |
| `0xAE00` | Files | `ui_img_file_ic_png` |
| `0xA400` | Groove Music | `ui_img_groove_ic_png` |
| `0xA200` | Messaging | `ui_img_message_ic_png` |
| `0xA500` | News | `ui_img_news_ic_png` |
| `0xA800` | Outlook Mail | `ui_img_outlook_ic_png` |
| `0xA300` | People | `ui_img_people_ic_png` |
| `0xA100` | Phone | `ui_img_phone_ic_png` |
| `0xA600` | Photos | `ui_img_photos_ic_png` |
| `0xA700` | Settings | `ui_img_wp_settings_png` |
| `0xAD00` | Stars | `ui_img_stars_ic_png` |
| `0xAA00` | Store | `ui_img_microsoft_ic_png` |
| `0xAC00` | Thermal IR | `ui_img_camera_ic_png` |
| `0xAB00` | Tips | `ui_img_tips_ic_png` |

### 开始屏磁贴 `tiles[]`

`tiles[]` 用于动态生成 Windows Phone 风格开始屏磁贴。

| app | wide | live image | live direction |
|---|---:|---|---|
| Phone | false | none | none |
| People | true | none | none |
| Outlook Mail | true | none | none |
| Messaging | false | none | none |
| Settings | false | none | none |
| Groove Music | false | none | none |
| Tips | false | `ui_img_embedded_tile_png` | horizontal |
| News | true | `ui_img_news_tile_png` | vertical |
| Thermal IR | false | none | none |
| Photos | false | `ui_img_photo_tile_png` | vertical |
| Stars | false | none | none |
| Edge | false | none | none |
| Store | true | none | none |
| Files | false | none | none |

### 设置项 `settings[]`

| code | title | desc | icon |
|---|---|---|---|
| `0xA701` | System | Display, notifications, battery | `ui_img_wp_system_png` |
| `0xA702` | Devices | Bluetooth, printers, mouse | `ui_img_wp_devices_png` |
| `0xA703` | Network | WiFi, cellular, hotspot | `ui_img_wp_network_png` |
| `0xA704` | Personalization | Background, lockscreen, colors | `ui_img_wp_personalization_png` |
| `0xA705` | Apps | Uninstall, defaults | `ui_img_wp_apps_png` |
| `0xA706` | Accounts | Email, sync, family | `ui_img_wp_account_png` |
| `0xA707` | Time & Language | Speech, region, date | `ui_img_wp_time_png` |
| `0xA708` | Privacy | Location, camera, microphone | `ui_img_wp_privacy_png` |
| `0xA709` | About | Serial, version, reset | `ui_img_wp_about_png` |

当前真正有内容的设置分支：

- `System`
- `Network`
- `Personalization`

其他设置项点击后会显示 alert：

```text
Settings error
This setting is not available at the moment
```

### 快速操作 `actions[]`

用于通知面板 quick actions。

| code | name | icon | checkable |
|---|---|---|---|
| `0x00C1` | Airplane | `ui_img_airplane_ic_png` | true |
| `0x00C2` | Cellular | `ui_img_cellular_ic_png` | true |
| `0x00C3` | WiFi | `ui_img_wifi_ic_png` | true |
| `0x00C4` | Bluetooth | `ui_img_bluetooth_ic_png` | true |
| `0x00C5` | Brightness | `ui_img_brightness_ic_png` | false |
| `0x00C6` | Battery | `ui_img_battery_ic_png` | true |
| `0x00C7` | Hotspot | `ui_img_hotspot_ic_png` | true |
| `0x00C8` | Settings | `ui_img_settings_ic_png` | false |
| `0x00C9` | VPN | `ui_img_vpn_ic_png` | true |
| `0x00CA` | Location | `ui_img_location_ic_png` | true |

当前有实际行为的 quick actions：

- `Bluetooth`：切换状态栏蓝牙图标显示。
- `Settings`：关闭通知面板并进入设置屏。
- `WiFi` 逻辑被注释掉，没有修改状态栏。

### Accent colors

`ui_common.c` 中定义 20 个 Windows Phone accent color：

```text
Lime, Green, Emerald, Teal, Cyan, Cobalt, Indigo, Violet, Pink, Magenta,
Crimson, Red, Orange, Amber, Yellow, Brown, Olive, Steel, Mauve, Taupe
```

点击颜色块触发 `theme_change()`，更新：

- LVGL 默认主题 primary color。
- 开始屏磁贴背景色。
- 注册过的图片 tint。
- 注册过的面板背景色。
- 注册过的文本输入边框色。
- Cortana 文本、输入框、圆形图标颜色。
- 如果启用 `nav_tint`，更新底部导航栏背景色。

## 屏幕结构

### Home Screen

文件：`ui/screens/ui_homeScreen.c`

根对象：

```text
ui_homeScreen
```

基础属性：

- 320x480。
- 背景色黑色。
- 默认背景图 `ui_img_img5_png`。
- 不可滚动。

主要子结构：

```text
ui_homeScreen
├── ui_homePanel
│   ├── ui_startPanel
│   └── ui_appsListPanel
├── ui_cortanaPanel
├── ui_navPanelListener
├── ui_navPanel
├── ui_lockScreenPanel
├── ui_notificationPanel
├── ui_statusPanel
├── ui_alertPanel
├── wifi_dialog_panel
└── ui_systemKeyboard
```

#### `ui_homePanel`

- 占满 320x480。
- 横向 flex。
- 横向滚动。
- `LV_SCROLL_SNAP_CENTER`。
- 无滚动条。
- 包含开始屏和应用列表两个页面。

这相当于 Windows Phone 的 Start Screen / All Apps 横向切换。

#### `ui_startPanel`

- 320x480。
- `LV_FLEX_FLOW_ROW_WRAP`。
- 纵向滚动。
- padding top 25、bottom 45、左右 5。
- 所有磁贴由 `init_custom()` 中循环 `tiles[]` 动态生成。
- `ui_allAppsPanel` 最初由 SquareLine 创建，之后 `load_prefs()` 调整到磁贴列表末尾。

磁贴由 `cm_start_tile()` 创建：

- 窄磁贴：`100x76`
- 宽磁贴：`205x76`
- 背景使用主题色。
- 图片作为 `style_bg_img_src`。
- label 放在左下角。
- 如果存在 live image，会创建子 `lv_img` 并注册到 `live[]`。

#### `ui_appsListPanel`

- 320x480。
- 纵向 flex。
- 纵向滚动。
- 黑色半透明背景。
- padding top/bottom 70。
- 所有应用列表项由 `init_custom()` 循环 `apps[]` 调用 `cm_app_list()` 动态生成。

列表项由 `cm_app_list()` 创建：

- 行高 `50`。
- 左侧 `50x50` icon panel。
- icon panel 背景色初始 `0x1390EA`。
- 右侧应用名使用 `lv_font_montserrat_18`。
- 点击整行调用 `launch_app()`。

#### `ui_cortanaPanel`

- 320x480。
- 初始 Y = 480，位于屏幕底部外。
- 搜索按钮触发 `openPanelUp_Animation()` 从底部上滑。
- 下滑手势触发 `closePanelDown_Animation()` 关闭。
- 内部包括：
  - `ui_TextArea3`：输入框，placeholder `How can i help you?`
  - `ui_Label6`：`Hi there, I'm Cortana`
  - `ui_cortanaIcon`：圆形对象，pulse 动画改变宽高

#### `ui_navPanel`

- 底部导航栏。
- 320x40。
- 黑色半透明背景。
- 三个按钮：

| 对象 | 图标 | 行为 |
|---|---|---|
| `ui_navBack` | `ui_img_wp_back_png` | 返回、回到开始屏、打开锁屏，或调用 `backHandler` |
| `ui_navHome` | `ui_img_windows_logo_png` | 回到 home，如果已在 home 则滚到 startPanel |
| `ui_navSearch` | `ui_img_wp_search_png` | 打开/关闭 Cortana 面板 |

`ui_navPanelListener` 是同尺寸透明监听层，目前没有单独行为。

#### `ui_lockScreenPanel`

- 320x480。
- 初始 Y = -480。
- 在初始动作中被移动到 Y = 0，因此 demo 初始显示锁屏。
- 背景默认 `ui_img_img2_png`。
- 有两个 label：
  - `ui_Label41`：时间，默认 `12:00`，运行时刷新。
  - `ui_Label39`：日期，运行时刷新。
- 上滑手势关闭锁屏。
- 在 home 的 startPanel 可见时点击 back，也会重新打开锁屏。

#### `ui_notificationPanel`

- 320x480。
- 初始 Y = -480。
- 状态栏右侧点击打开。
- 上滑手势关闭。
- 包含 quick action grid：
  - `ui_quickActionPanel`：320x103，row wrap。
  - `ui_Panel30`：底部 10px drag handle，显示 `____`。

Quick action button 由 `cm_quick_action()` 动态创建：

- `58x40`
- flex column
- 上方 icon，下方 `lv_font_montserrat_10` label
- checkable 项默认 checked

#### `ui_statusPanel`

- 顶部状态栏。
- 320x20。
- 左侧：
  - cellular bar：用 `lv_bar`，背景图片为 cellular icon。
  - WiFi icon：默认 hidden。
  - Bluetooth icon：默认 hidden。
- 右侧：
  - battery icon。
  - time label，运行时刷新。

#### `ui_alertPanel`

- 全屏半透明黑色遮罩。
- 默认 hidden。
- `show_alert(title, text)` 设置文本并显示。
- 中间 `ui_alertDialogPanel`：
  - 250 宽。
  - 黑色背景。
  - 主题色边框。
  - title、text、Close 按钮。

#### WiFi dialog

对象：`wifi_dialog_panel`

- 280 宽。
- 顶部 Y = 100。
- 默认 hidden。
- 包含：
  - `wifi_dialog_text`
  - `wifi_dialog_textarea`
  - `Show Password` checkbox
  - Cancel / Connect 按钮

输入框 focus 时显示 `ui_systemKeyboard`，defocus 时隐藏。

#### `ui_systemKeyboard`

- LVGL keyboard。
- 320x150。
- 底部对齐，Y = -40。
- 默认 hidden。
- 由 `ui_event_textarea()` 绑定到当前 text area。

### Settings Screen

文件：`ui/screens/ui_settingsScreen.c`

根对象：

```text
ui_settingsScreen
```

布局：

```text
ui_settingsScreen
├── ui_settingsTitlePanel
├── ui_settingsMainPanel
├── ui_personalizationPanel
├── ui_networkPanel
└── ui_systemPanel
```

#### Title panel

- 320x70。
- row flex。
- 左侧 icon / back / title。
- `ui_settingsBack` 默认 hidden，只在进入二级设置页时显示。
- `ui_settingsAppIcon` 默认设置图标。
- `ui_settingsAppTitle` 默认 `Settings`。

#### Main panel

- 320x410。
- Y = 70。
- 纵向 flex。
- 动态内容：
  - 搜索 text area：placeholder `Find a setting`
  - `settings[]` 列表项

设置列表项由 `cm_settings_list()` 创建：

- 行高 `50`。
- 左侧 `32x32` 图标。
- 图标默认 recolor 为主题蓝。
- title 使用 `lv_font_montserrat_18`。
- desc 使用 `lv_font_montserrat_12`。
- 点击调用 `launch_settings()`。

#### System panel

由 `init_custom()` 动态填充：

```text
Display
Brightness
brightness slider: 1..255
Timeout
dropdown:
  5 Seconds
  10 Seconds
  20 Seconds
  30 Seconds
  Always On
```

Brightness 调用 `onBrightnessChange(value)`，当前 NuttX 入口中只写入内存 pref。

Timeout 调用 `onTimeoutChange(selected)`，当前 NuttX 入口中只写入内存 pref。

#### Personalization panel

由 `init_custom()` 动态填充：

```text
Accent Color
20 个颜色块

Tile Opacity
slider 0..255

Background
dropdown: None / Picture
background thumbnail grid

NavBar Opacity
slider 20..255

Navbar accent color
switch

Lockscreen
lockscreen thumbnail grid
```

行为：

- Accent color 更新主题色和已注册对象。
- Tile opacity 更新开始屏磁贴背景透明度。
- Background dropdown 为 `None` 时去掉 home 背景图，为 `Picture` 时使用 `bg_img` 索引。
- Background thumbnail 点击更新 `ui_homeScreen` 背景图。
- NavBar opacity 更新底部导航栏背景透明度。
- Navbar accent color 打开后让导航栏使用主题色。
- Lockscreen thumbnail 点击更新 `ui_lockScreenPanel` 背景图。

#### Network panel

由 `init_custom()` 动态填充：

```text
WiFi switch
Refresh button
wifi_status text
ui_wifiList
```

WiFi list item 由 `add_wifi_list()` 创建：

- 行宽 280。
- 左侧 WiFi icon。
- 中间 SSID label，宽 197，长文本滚动。
- 右侧 padlock icon，根据 `wifi.secure` 决定是否 hidden。
- 点击 open WiFi dialog。

当前 NuttX 迁移中，`onRefreshWifi()`、`onOpenNetworks()`、`onConnectWifi()` 都是 weak 空实现，因此 WiFi UI 只有壳，没有真实扫描和连接。

### Generic App Screen

文件：`ui/screens/ui_appScreen.c`

用途：通用示例 app，目前由 Tips tile 使用。

结构：

```text
ui_appScreen
├── ui_appTitlePanel
│   ├── ui_appTitleIcon
│   └── ui_appTitleName
└── ui_appMainPanel
    ├── sample text
    ├── switch + label
    ├── button
    ├── dropdown
    ├── label
    ├── slider
    ├── dynamic bar
    └── dynamic slider
```

静态内容：

- title 默认 `App Name`。
- icon 默认 settings。
- `ui_Label23`：`Sample text widget for general information in the app`
- switch label 默认 `Off`
- button label `Button`
- dropdown 由 SquareLine 创建。
- `ui_event_genSwitch()` 切换 `On` / `Off` 文本。
- `ui_event_app_load()` 调用 `onLoadTestApp()`，当前为空实现。

动态补充：

- `init_custom()` 在 `ui_appMainPanel` 里增加一个 progress bar 和 slider。
- slider 改变时 `slider_val()` 更新 bar label。

### News Screen

文件：`ui/screens/ui_newsScreen.c`

结构：

```text
ui_newsScreen
├── ui_appTitlePanel1
└── ui_settingsMainPanel1
```

标题：

- icon：`ui_img_news_ic_png`
- title：`News`

内容：

- 静态新闻列表。
- 每条新闻由一个 panel、一个标题 label、一块 image crop 区域和一段摘要 label 组成。
- 所有新闻图片都使用 `ui_img_news_image_png`，通过不同 Y offset 截取同一张长图的不同区域。

当前静态标题包括：

```text
Windows 10 Mobile UI Designed with SquareLine Studio and LVGL
LVGL Unveils Vibrant New Website and Logo
Yusuf Dikec's Casual Look Wins Silver at Paris Olympics
Chronos App Surpasses 50K Downloads on Google Play
ESP32 C3 Mini LVGL UI Project Now Supports Installable Watchfaces
'Apple Explained' YouTube Channel Discusses Windows Phone Failure
```

### Stars Screen

文件：`ui/screens/ui_starsScreen.c`

用途：展示 GitHub stargazers。

结构：

```text
ui_starsScreen
├── ui_starsTitlePanel
└── ui_starsMainPanel
    ├── ui_starsInfo
    ├── ui_Label7
    └── ui_stargazersPanel
```

数据来自 `ui/stars.h`：

- `users[]` 是静态用户名数组。
- icon 当前全部为 `NULL`。
- `init_custom()` 计算 `users[]` 数量并更新 `ui_Label7` 为 `Total Stars: N`。
- 每个用户通过 `cm_user_list()` 渲染为一个小 label chip。

`stars.h` 注释说明原工程可能由 `stars.py` 拉取仓库 stargazers 生成；当前迁移版是静态数据。

### Thermal IR Screen

文件：`ui/screens/ui_thermalScreen.c`

结构：

```text
ui_thermalScreen
├── ui_thermalTitlePanel
└── ui_thermalPanel
    ├── ui_tempTextPanel
    │   ├── ui_lowTemp
    │   ├── ui_averageTemp
    │   └── ui_highTemp
    └── ui_gridTempPanel
```

标题：

- icon：`ui_img_camera_ic_png`
- title：`Thermal IR`

内容：

- `ui_lowTemp` 默认 `Low\n--`
- `ui_averageTemp` 默认 `Average\n--`
- `ui_highTemp` 默认 `High\n--`
- `ui_gridTempPanel` 是 300x300。
- `init_custom()` 循环创建 64 个 `cm_ir_tile()`，对应 8x8 热成像网格占位。

启动行为：

- `launch_app(0xAC00)` 进入 Thermal IR。
- 如果 `thermal_status == false`，三个温度 label 显示：

```text
Sensor
not
found
```

- 如果 `thermal_status == true`，设置 `thermal_active = true`。
- 屏幕 unload 时 `ui_event_screen_unload(0xAC00)` 设置 `thermal_active = false`。

当前 NuttX 迁移版没有真实 AMG88xx 传感器驱动接入。

## 组件工厂

`ui_common.c` 是这个 demo 最值得参考的一层。它把 SquareLine 静态对象树之外的重复 UI 变成 C 函数。

| 函数 | 作用 |
|---|---|
| `cm_create_text()` | 创建 280 宽普通文本 |
| `cm_create_switch()` | 创建 switch + label 行 |
| `cm_create_button()` | 创建无圆角按钮 |
| `cm_create_title()` | 创建 280 宽标题 |
| `cm_create_text_area()` | 创建 text area 并绑定 keyboard focus 事件 |
| `cm_create_slider()` | 创建 slider |
| `cm_create_panel_space()` | 创建透明 spacer |
| `cm_create_bar()` | 创建 label + bar 组合 |
| `cm_set_bar()` | 更新 `cm_create_bar()` 生成的 bar |
| `cm_create_dropdown()` | 创建 dropdown |
| `cm_ir_tile()` | 创建 35x35 热成像网格块 |
| `cm_user_list()` | 创建 stargazer chip |
| `cm_quick_action()` | 创建通知面板 quick action button |
| `cm_start_tile()` | 创建开始屏磁贴 |
| `cm_app_list()` | 创建 All Apps 行 |
| `cm_settings_list()` | 创建 Settings 行 |
| `cm_accent_color()` | 创建主题色方块 |
| `cm_image_select()` | 创建背景图缩略选择块 |
| `cm_create_app_title()` | 创建 app 标题栏，目前没有在主流程中使用 |
| `set_parent()` | 把全局 overlay 移到当前 active screen 下 |
| `add_wifi_list()` | 添加一个 WiFi 网络列表项 |

这一层对 wing 很有参考价值：它不是宏，也不是复杂 DSL，而是用普通 C 函数把 UI 片段构造出来。后续如果 wing 使用 Rust，可以把这一层演化为轻量 builder 或 ECS command。

## 动画系统

动画都在 `ui/ui.c` 中手写/生成。

| 函数 | 效果 |
|---|---|
| `closeNotificationPanel_Animation()` | Y 从 `0` 到 `-480`，关闭通知/锁屏类上方面板 |
| `openNotificationPanel_Animation()` | Y 从 `-480` 到 `0`，打开通知/锁屏类上方面板 |
| `liveTileVertical6_Animation()` | live tile 图片纵向循环滚动，6 段 |
| `liveTileHorizontal5_Animation()` | live tile 图片横向循环滚动，5 段 |
| `cortanaPulse_Animation()` | Cortana 圆形对象宽高在 `50` 与 `30` 之间循环 |
| `closePanelDown_Animation()` | Y 从 `0` 到 `480`，同时透明度从 `255` 到 `0` |
| `openPanelUp_Animation()` | Y 从 `480` 到 `0`，同时透明度从 `0` 到 `255` |

动画依赖 `ui_helpers.c` 中的回调：

- `_ui_anim_callback_set_x`
- `_ui_anim_callback_set_y`
- `_ui_anim_callback_set_width`
- `_ui_anim_callback_set_height`
- `_ui_anim_callback_set_opacity`
- `_ui_anim_callback_free_user_data`

live tile 注册流程：

1. `cm_start_tile()` 遇到 `tile.live != NULL` 时创建 live image。
2. 调用 `register_live()` 保存到 `live[]`。
3. `ui_event____initial_actions0()` 遍历 `live[]`。
4. 按 `CM_LIVE_VERTICAL` 或 `CM_LIVE_HORIZONTAL` 启动对应无限循环动画。

## 导航与事件

### 应用启动

入口：`launch_app(lv_event_t *e)`

根据 app code 分派：

| code | 行为 |
|---|---|
| `0xAC00` | 打开 Thermal IR |
| `0xA700` | 打开 Settings |
| `0xAD00` | 打开 Stars |
| `0xA500` | 打开 News |
| `0xAB00` | 打开 Generic App，并把 title/icon 设置为 Tips |
| 其他 | 弹出 `App error` |

屏幕切换使用：

```c
_ui_screen_change(&screen, LV_SCR_LOAD_ANIM_FADE_ON, 500, 0, &screen_init);
```

### 设置导航

入口：`launch_settings(lv_event_t *e)`

每次进入设置分支前先隐藏：

```text
ui_settingsMainPanel
ui_personalizationPanel
ui_systemPanel
ui_networkPanel
```

然后按 code 显示对应 panel：

| code | panel |
|---|---|
| `0xA700` | main |
| `0xA701` | system |
| `0xA703` | network |
| `0xA704` | personalization |

进入二级设置页时显示 `ui_settingsBack`，并替换 title icon/title。

### 底部导航栏

`ui_event_nav_home()`：

- 如果 Cortana 打开，先关闭 Cortana。
- 如果当前已经是 home，滚动到 `ui_startPanel`。
- 否则立即回到 home。

`ui_event_nav_back()`：

- 如果 Cortana 打开，关闭 Cortana。
- 如果当前是 home 且不在 startPanel，滚动回 startPanel。
- 如果当前是 home 且已经在 startPanel，打开锁屏 panel。
- 如果当前是 settings 且不在 settings main，回到 settings main。
- 如果注册了 `backHandler`，调用 `backHandler()`。
- 否则回到 home。

`ui_event_nav_search()`：

- toggle Cortana panel。

注意：这套导航是 Windows Phone 风格的底部 nav bar，不是 wing 前面讨论的手势式系统导航。

### 手势

当前只有两个明确手势：

| 对象 | 手势 | 行为 |
|---|---|---|
| `ui_cortanaPanel` | 下滑 `LV_DIR_BOTTOM` | 关闭 Cortana |
| `ui_lockScreenPanel` | 上滑 `LV_DIR_TOP` | 关闭锁屏 |
| `ui_notificationPanel` | 上滑 `LV_DIR_TOP` | 关闭通知面板 |

通知面板不是从顶部下滑打开，而是点击状态栏右侧打开。

### Overlay 迁移到当前屏

所有屏幕加载后都会执行 `ui_event_screen_load()`，调用 `set_parent(target)`。

`set_parent()` 会把以下全局 overlay 重新挂到当前屏幕：

```text
ui_cortanaPanel
wifi_dialog_panel
ui_systemKeyboard
ui_navPanel
ui_lockScreenPanel
ui_notificationPanel
ui_statusPanel
ui_alertPanel
```

意义：

- 底部导航、状态栏、通知面板、锁屏、键盘、alert 不只是 home 的子对象。
- 切换到 settings/news/app/stars/thermal 后，这些系统层 overlay 仍能显示在当前屏幕上。

这是一个很关键的 shell 思路：系统 overlay 是跨 app 的，只是当前实现通过 `lv_obj_set_parent()` 搬运对象实现。

## 硬件/业务回调

`ui_events.c` 中提供 weak 空实现：

```c
__attribute__((weak)) void onRefreshWifi(void){}
__attribute__((weak)) void onOpenNetworks(void){}
__attribute__((weak)) void onConnectWifi(void){}
__attribute__((weak)) void onLoadTestApp(void){}
__attribute__((weak)) void onWifiStateChange(int state){}
```

`lvgl_window10.c` 中提供实际存在但仍然很轻的实现：

```c
void onBrightnessChange(int32_t value)
void onTimeoutChange(int16_t selected)
void onWifiStateChange(int state)
```

这些函数当前只写 `g_prefs`，没有真正控制背光、睡眠、WiFi 设备。

后续如果要让 demo 更真实，可以接入：

- framebuffer backlight 或模拟亮度。
- NuttX netmgr / WiFi scan。
- 输入设备震动或 haptic。
- 真实传感器数据。

## 资源实现

### 图片资源

`ui/images/*.c` 当前共有 57 个图片 C 文件。

特点：

- 图片以 `const uint8_t xxx_data[]` 存储。
- 每个图片暴露 `const lv_img_dsc_t` 或少量 `const lv_image_dsc_t`。
- `ui.h` 中通过 `LV_IMG_DECLARE(...)` 声明。
- 运行时直接用 `lv_img_set_src()` 或 `lv_obj_set_style_bg_img_src()`。
- 没有 PNG/JPEG 解码路径。
- 没有从 NuttX 文件系统加载。

资源类型：

| 类型 | 示例 |
|---|---|
| 背景图 | `ui_img_img0_png` 到 `ui_img_img9_png` |
| 应用图标 | phone、people、outlook、message、settings、groove、tips、news、camera、photos、weather、edge、microsoft、file |
| 系统图标 | back、windows logo、search、airplane、cellular、wifi、bluetooth、brightness、battery、hotspot、VPN、location |
| 设置图标 | system、about、account、apps、devices、network、personalization、privacy、time |
| live tile 图 | embedded tile、news tile、photo tile |
| 新闻图 | `ui_img_news_image_png` |
| WiFi dialog 图 | wifi icon、padlock |

背景选择集：

```c
#ifdef MIN_BG_IMG
const lv_img_dsc_t *ui_imgset_img[5] = {
  &ui_img_img1_png,
  &ui_img_img2_png,
  &ui_img_img5_png,
  &ui_img_img6_png,
  &ui_img_img8_png
};
#else
const lv_img_dsc_t *ui_imgset_img[10] = {
  &ui_img_img0_png,
  &ui_img_img1_png,
  &ui_img_img2_png,
  &ui_img_img3_png,
  &ui_img_img4_png,
  &ui_img_img5_png,
  &ui_img_img6_png,
  &ui_img_img7_png,
  &ui_img_img8_png,
  &ui_img_img9_png
};
#endif
```

当前 NuttX 配置没有定义 `MIN_BG_IMG`，因此默认包含 10 张背景。

背景图尺寸从 descriptors 看是 `320x569`，超过 320x480 屏幕高度。它们作为背景图绘制时不是运行时 PNG 解码，而是 LVGL 使用已编译的像素数组。

### 字体资源

当前使用 LVGL 内置 Montserrat：

- `lv_font_montserrat_10`
- `lv_font_montserrat_12`
- `lv_font_montserrat_14`
- `lv_font_montserrat_18`
- `lv_font_montserrat_20`
- `lv_font_montserrat_22`
- `lv_font_montserrat_48`

注意：

- defconfig 显式启用了 10、12、18、20、22、48。
- 源码中也使用了 `lv_font_montserrat_14`，当前 LVGL 默认配置如果没有启用 14 可能存在潜在配置依赖。实际编译通过说明当前 LVGL/NuttX 配置中 14 可用，或由 LVGL 默认启用。
- 没有外部 TTF。
- 没有中文字体。

### 颜色和风格

整体风格是 Windows Phone 平铺 UI：

- 大量无圆角矩形。
- 黑色背景。
- 主题蓝 accent。
- 图片 recolor/tint 用于统一色彩。
- `bg_opa` 广泛用于半透明 panel。
- 滚动条基本关闭。
- 底部 nav bar 半透明。

## 需要注意的问题

### 1. 资源较重

当前 `apps/examples/lvgl_window10` 目录约 23MB，其中主要是图片 C 数组和生成代码。因为图片直接编译进二进制，它适合作为 demo，但不适合高性能 MCU 的最终资源策略。

对 wing 的启发：

- 背景类资源应该谨慎。
- 图标应尽量转为矢量、A8 mask 或小尺寸 atlas。
- 文件系统资源和编译进 ROM 的资源需要明确分层。

### 2. UI 代码强依赖全局变量

SquareLine 生成风格是大量全局 `lv_obj_t *`：

```text
ui_homeScreen
ui_settingsScreen
ui_navPanel
ui_alertPanel
...
```

优点：

- 易于迁移。
- 易于调试。
- 事件回调能直接操作对象。

缺点：

- 不适合复杂应用隔离。
- 不适合声明式状态同步。
- 对内存生命周期和 screen delete 不够清晰。

### 3. 系统 overlay 通过 reparent 实现

`set_parent()` 的思路很好：系统层 overlay 随当前屏幕移动。

但实现上直接 `lv_obj_set_parent()` 有风险：

- z-order 需要小心维护。
- 不同屏幕切换时 overlay 的位置/状态可能残留。
- 后续多应用场景更适合 shell root/layer 统一管理。

wing 可以参考思想，但实现应更像：

```text
Root
├── AppLayer
├── SystemOverlayLayer
├── NotificationLayer
├── KeyboardLayer
└── ModalLayer
```

### 4. 事件模型仍是命令式

事件基本是：

```text
LVGL event -> callback -> 直接操作对象
```

这对 demo 足够，但不是声明式 ECS UI。后续 wing 可以把这些操作抽象为：

```text
Event -> Intent -> State Update -> Diff -> Render Commands
```

### 5. 当前 demo 的硬件功能大多是壳

以下功能未真正接入：

- WiFi scan/connect。
- Brightness/backlight。
- Timeout/sleep。
- Thermal sensor。
- Haptic。
- API 新闻加载。
- Stars 在线刷新。

它是 UI demo，不是完整 OS shell。

## 对 wing 的参考价值

最值得参考的部分：

1. **屏幕和 overlay 分层思路**
   - app screen 和系统 overlay 分离。
   - 状态栏、通知面板、导航栏、键盘、alert 跨屏存在。

2. **组件工厂层**
   - `cm_start_tile()`、`cm_app_list()`、`cm_settings_list()` 很像轻量 declarative UI 的底层积木。
   - 没有宏，普通函数即可组织 UI。

3. **主题传播**
   - 通过注册 start tile、image tint、panel、textarea，实现主题色统一更新。
   - 这可以升级为 wing 的资源/样式系统。

4. **live tile 动画**
   - 简单但有效。
   - 动画目标是对象属性：x/y/width/height/opacity。
   - 对应 wing 可以抽象为 property animation command。

5. **系统交互模型**
   - lockscreen、notification panel、nav bar、keyboard、modal 都是 shell 需要的核心部件。

6. **LVGL 绘制质量经验**
   - 使用 16bpp。
   - 大量缓存好的 bitmap。
   - 控件风格保持简单。
   - 避免复杂裁剪和复杂文字排版。

## 建议的后续整理方向

如果继续完善这个 LVGL demo：

1. 保持它作为 `lvgl_window10` 示例，不要继续往 wing 里混合。
2. 把 `ui_events.c` 的 weak hook 对接 NuttX 模拟能力。
3. 加一个资源统计文档或脚本，统计图片数量、尺寸、总大小。
4. 可选定义 `MIN_BG_IMG`，减少背景资源。
5. 检查 `lv_font_montserrat_14` 配置依赖，避免换配置后编译失败。
6. 逐步把静态图片背景替换为更轻的渐变/矢量/程序绘制实验。

如果用于 wing 重新设计：

1. 不直接复制 SquareLine 的全局对象模型。
2. 参考 `cm_*` 组件工厂，设计 Rust 无宏 builder/command。
3. 把 `set_parent()` 的思想变成 shell layer tree。
4. 把 `theme_change()` 的注册表思想变成 ECS style resource。
5. 把 `launch_app()`、`launch_settings()` 改成状态机或 route stack。
6. 把图片 C 数组策略改为可裁剪的资源系统：小图标内置，大背景放文件系统，必要时支持解码缓存。

