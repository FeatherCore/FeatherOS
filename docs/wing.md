# Wing Desktop Environment

Wing 是 FeatherOS 的桌面环境，基于 FHRE (Feather Hybrid Rendering Engine) 构建。

## 概述

Wing 提供了完整的桌面体验，包括：

- **窗口管理**：创建、移动、调整大小、关闭、聚焦窗口
- **桌面环境**：壁纸、图标网格、右键菜单
- **任务栏**：开始按钮、窗口切换器、系统托盘
- **应用启动器**：应用程序菜单、搜索功能
- **多任务支持**：多窗口管理、窗口层级

## 架构

```
┌─────────────────────────────────────────┐
│         User Applications               │
│    (Calculator, Editor, Browser, etc.)  │
├─────────────────────────────────────────┤
│         Wing Desktop Shell              │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐ │
│  │  Window │ │ Desktop │ │ Taskbar  │ │
│  │ Manager │ │  + Icons│ │+ Launcher│ │
│  └────┬────┘ └────┬────┘ └────┬─────┘ │
│       └───────────┴───────────┘       │
├─────────────────────────────────────────┤
│         FHRE (Graphics Library)         │
│    - 2D/3D Rendering                    │
│    - UI Components                      │
│    - Event Handling                     │
├─────────────────────────────────────────┤
│         NuttX (Operating System)        │
│    - Process Management                 │
│    - File System                        │
│    - Device Drivers                     │
└─────────────────────────────────────────┘
```

## 目录结构

```
apps/wing/
├── rust/                    # Rust 实现
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # 主入口
│       ├── window.rs        # 窗口管理
│       ├── desktop.rs       # 桌面环境
│       ├── taskbar.rs       # 任务栏
│       ├── launcher.rs      # 应用启动器
│       ├── wallpaper.rs     # 壁纸
│       └── icon.rs          # 桌面图标
│
└── clang/                   # C 实现
    ├── include/wing/
    │   └── wing.h           # C API 头文件
    ├── src/
    │   └── wing.c           # C 实现
    └── examples/            # 示例程序
```

## Rust API

### 基本使用

```rust
use wing::{Wing, AppInfo};
use fhre::math::{Vec2, Color};

// 创建 Wing 桌面环境
let mut desktop = Wing::new(800.0, 600.0);
desktop.init();

// 启动应用程序
let app = AppInfo::new(
    "Calculator",
    "Simple calculator",
    Vec2::new(300.0, 400.0),
    Vec2::new(100.0, 100.0),
    Color::rgb(100, 200, 100),
);
let window_id = desktop.launch_app(&app);

// 每帧更新
desktop.update(delta_time);
```

### 核心组件

#### WindowManager

管理所有应用程序窗口：

```rust
// 创建窗口
let window_id = desktop.window_manager.create_window(
    "My App",
    Vec2::new(400.0, 300.0),
    Vec2::new(100.0, 100.0),
);

// 移动窗口
desktop.window_manager.move_window(window_id, Vec2::new(10.0, 10.0));

// 聚焦窗口
desktop.window_manager.focus_window(window_id);

// 关闭窗口
desktop.window_manager.close_window(window_id);
```

#### Desktop

桌面环境管理：

```rust
// 获取图标
desktop.get_icon_at(position);

// 生成渲染命令
let commands = desktop.generate_render_commands();
```

#### Taskbar

任务栏管理：

```rust
// 添加应用到任务栏
desktop.taskbar.add_app(app_info, window_id);

// 从任务栏移除
desktop.taskbar.remove_window(window_id);
```

## C API

### 基本使用

```c
#include <wing/wing.h>

// 创建 Wing 上下文
wing_context_t* ctx = wing_create(800.0f, 600.0f);
wing_init(ctx);

// 创建窗口
wing_window_id_t window = wing_window_create(
    ctx,
    "My Window",
    wing_vec2(400.0f, 300.0f),
    wing_vec2(100.0f, 100.0f),
    NULL,
    NULL
);

// 主循环
while (running) {
    wing_update(ctx, delta_time);
}

// 清理
wing_destroy(ctx);
```

### 窗口回调

```c
void on_window_close(wing_window_id_t window_id, void* user_data) {
    printf("Window %d closed\n", window_id);
}

wing_window_callbacks_t callbacks = {
    .on_close = on_window_close,
    // ... other callbacks
};

wing_window_id_t window = wing_window_create(
    ctx, "Title", size, pos, &callbacks, user_data
);
```

## 与 FHRE 的关系

Wing 构建在 FHRE 之上：

| 层级 | 功能 | 实现 |
|------|------|------|
| Wing | 桌面环境、窗口管理 | Rust / C |
| FHRE | 图形渲染、UI组件 | Rust |
| NuttX | 操作系统服务 | C |

## 应用开发

### 创建 Wing 应用 (Rust)

```rust
use wing::{AppInfo, Wing};

fn main() {
    let mut desktop = Wing::new(800.0, 600.0);
    desktop.init();
    
    // 注册应用
    let my_app = AppInfo::new(
        "My App",
        "Description",
        Vec2::new(400.0, 300.0),
        Vec2::new(100.0, 100.0),
        Color::rgb(100, 150, 200),
    );
    
    // 添加到启动器
    desktop.launcher.add_app(my_app);
    
    // 运行
    loop {
        desktop.update(1.0 / 60.0);
    }
}
```

### 创建 Wing 应用 (C)

```c
#include <wing/wing.h>

int main() {
    wing_context_t* ctx = wing_create(800.0f, 600.0f);
    wing_init(ctx);
    
    // 定义应用
    wing_app_info_t my_app = {
        .name = "My App",
        .description = "Description",
        .default_size = wing_vec2(400.0f, 300.0f),
        .initial_position = wing_vec2(100.0f, 100.0f),
        .icon_color = wing_color_rgb(100, 150, 200)
    };
    
    // 启动
    wing_window_id_t window = wing_launch_app(ctx, &my_app);
    
    // 主循环
    while (running) {
        wing_update(ctx, 1.0f / 60.0f);
    }
    
    wing_destroy(ctx);
    return 0;
}
```

## 配置

### 桌面配置

```rust
use wing::DesktopConfig;

let config = DesktopConfig {
    background_color: Color::rgb(30, 30, 30),
    icon_spacing: 80.0,
    icon_size: 64.0,
};
```

### 任务栏配置

```rust
use wing::TaskbarConfig;

let config = TaskbarConfig {
    height: 48.0,
    background_color: Color::rgb(40, 40, 40),
    button_color: Color::rgb(60, 60, 60),
    position_bottom: true,
};
```

## 路线图

- [x] 基础框架设计
- [x] Rust 实现
- [x] C API 设计
- [ ] C 完整实现
- [ ] 窗口装饰（标题栏、边框）
- [ ] 窗口动画
- [ ] 多桌面支持
- [ ] 系统托盘
- [ ] 设置面板
- [ ] 文件管理器集成
- [ ] 主题系统

## 许可证

MIT OR Apache-2.0
