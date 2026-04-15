# FHRE 默认资源设计

## 概述

FHRE 提供内置的默认资源，简化应用开发：

1. **PrimaryScreen** - 主屏幕/窗口，全局分辨率定义
2. **Default UI Camera** - 默认 UI 摄像机，正交投影

这些资源在 `App::new()` 时自动创建，用户可以直接使用，也可以自定义覆盖。

## 默认资源架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FHRE 默认资源架构                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  App::new() 自动创建：                                                       │
│  ───────────────────                                                         │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ 1. PrimaryScreen (主屏幕)                                           │   │
│  │    ├── width: 800 (或实际显示分辨率)                                  │   │
│  │    ├── height: 600 (或实际显示分辨率)                                 │   │
│  │    ├── viewport: (0, 0, width, height)                              │   │
│  │    └── pixel_density: 1.0                                           │   │
│  │                                                                     │   │
│  │    这是 FHRE 的全局主窗口，所有渲染默认输出到这里                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ 2. Default UI Camera (默认 UI 摄像机)                                │   │
│  │    Entity: default_ui_camera                                        │   │
│  │    ├── Node { node_type: Camera }                                   │   │
│  │    ├── Transform3D {                                                │   │
│  │    │       position: (width/2, height/2, 100)  // 屏幕中心上方       │   │
│  │    │     }                                                          │   │
│  │    └── Camera3D {                                                   │   │
│  │            orthographic: true,            // 正交投影                │   │
│  │            orthographic_size: height/2,   // 视口范围                │   │
│  │            depth: -100,                   // 最先渲染                │   │
│  │            viewport: (0, 0, 1, 1)         // 全屏                    │   │
│  │        }                                                            │   │
│  │                                                                     │   │
│  │    这是默认的 UI 摄像机，所有 UI 元素默认使用这个摄像机渲染            │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  使用示例：                                                                  │
│  ─────────                                                                  │
│                                                                             │
│  // 纯 UI 应用（使用默认资源）                                               │
│  let mut app = App::new();  // 自动创建主屏幕和默认 UI 摄像机              │
│  app.run();                                                                 │
│                                                                             │
│  // 游戏应用（添加游戏摄像机）                                               │
│  let mut app = App::new();                                                  │
│  let game_cam = app.create_game_camera(                                     │
│      Vec3::new(0.0, 5.0, -10.0),  // 位置                                  │
│      Vec3::new(0.0, 0.0, 0.0),    // 看向原点                              │
│      60.0                          // FOV                                  │
│  );                                                                         │
│  app.run();                                                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## PrimaryScreen (主屏幕)

### 功能

- 定义全局窗口分辨率
- 提供屏幕坐标转换工具
- 支持高 DPI 显示

### API

```rust
use fhre::resources::PrimaryScreen;

// 获取屏幕尺寸
fn get_screen_size(screen: Res<PrimaryScreen>) -> Vec2 {
    screen.size()  // Vec2 { x: 800, y: 600 }
}

// 获取屏幕中心
fn get_screen_center(screen: Res<PrimaryScreen>) -> Vec2 {
    screen.center()  // Vec2 { x: 400, y: 300 }
}

// 坐标转换
fn convert_coordinates(screen: Res<PrimaryScreen>) {
    // 归一化坐标 (0-1) → 屏幕坐标
    let screen_pos = screen.normalized_to_screen(Vec2::new(0.5, 0.5));
    // 结果: Vec2 { x: 400, y: 300 }
    
    // 屏幕坐标 → 归一化坐标
    let normalized = screen.screen_to_normalized(Vec2::new(400, 300));
    // 结果: Vec2 { x: 0.5, y: 0.5 }
}

// 检查点是否在屏幕内
fn check_bounds(screen: Res<PrimaryScreen>, point: Vec2) -> bool {
    screen.contains(point)  // true/false
}
```

### 屏幕坐标系统

```
屏幕坐标 (0,0) 在左上角

(0,0) ──────────────────────→ X
  │
  │    (200, 150)
  │         ●
  │
  │              (600, 450)
  │                   ●
  │
  ↓
  Y

(800, 600)
```

## Default UI Camera (默认 UI 摄像机)

### 功能

- 正交投影，无透视变形
- 屏幕坐标直接对应像素坐标
- 始终存在，不可删除（但可以禁用）

### 工作原理

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     默认 UI 摄像机投影原理                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  摄像机位置: (width/2, height/2, 100)                                       │
│  正交尺寸: height/2                                                         │
│                                                                             │
│  这意味着视图范围是:                                                         │
│  X: -width/2 到 +width/2  →  0 到 width                                     │
│  Y: -height/2 到 +height/2 →  0 到 height                                    │
│                                                                             │
│  所以:                                                                       │
│  世界坐标 (100, 200, 0)  →  屏幕像素 (100, 200)                              │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │   世界坐标系              正交投影              屏幕坐标系            │   │
│  │                                                                     │   │
│  │   (0,0) ──────→ X         ────→              (0,0) ──────→ X        │   │
│  │     │                           无透视变形       │                   │   │
│  │     ↓ Y                                        ↓ Y                  │   │
│  │                                                                     │   │
│  │   ● (100, 200)      ───────────────→      ● (100, 200)              │   │
│  │                                                                     │   │
│  │   z=0 (UI平面)                              屏幕像素位置             │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 使用示例

```rust
use fhre::{
    App,
    node::{Node, NodeType, Transform3D, Style},
    math::{Vec2, Color},
};

fn setup_ui(mut commands: Commands, screen: Res<PrimaryScreen>) {
    let center = screen.center();
    
    // 创建按钮 - 使用屏幕坐标直接定位
    commands.spawn((
        Node::ui_control(NodeType::Button),
        Transform3D::from_position(100.0, 200.0, 0.0),  // 屏幕 (100, 200)
        Style {
            width: Dimension::Pixel(120.0),
            height: Dimension::Pixel(40.0),
            background_color: Color::BLUE,
            ..default()
        },
    ));
    
    // 创建居中的面板
    commands.spawn((
        Node::ui_control(NodeType::Panel),
        Transform3D::from_position(center.x - 150.0, center.y - 100.0, 0.0),
        Style {
            width: Dimension::Pixel(300.0),
            height: Dimension::Pixel(200.0),
            background_color: Color::DARK_GRAY,
            ..default()
        },
    ));
}

fn main() {
    let mut app = App::new();
    
    // 默认 UI 摄像机已经创建好了
    // 直接添加 UI 元素即可
    app.add_startup_system(setup_ui);
    
    app.run();
}
```

## 自定义摄像机

### 创建游戏摄像机

```rust
// 创建 3D 游戏摄像机
let game_camera = app.create_game_camera(
    Vec3::new(0.0, 5.0, -10.0),  // 位置：在原点上方偏后
    Vec3::new(0.0, 0.0, 0.0),    // 看向：原点
    60.0                          // FOV：60度
);

// 现在有两个摄像机：
// 1. 默认 UI 摄像机 (depth: -100, 正交) - 渲染 UI
// 2. 游戏摄像机 (depth: 0, 透视) - 渲染 3D 场景
```

### 多摄像机场景

```rust
fn setup_cameras(mut app: &mut App) {
    // 1. 默认 UI 摄像机（已自动创建）
    //    - 渲染全屏 UI
    //    - 正交投影
    
    // 2. 游戏主摄像机
    let main_cam = app.create_game_camera(
        Vec3::new(0.0, 10.0, -20.0),
        Vec3::ZERO,
        60.0
    );
    
    // 3. 小地图摄像机（俯视）
    let minimap_cam = app.main_world.spawn();
    app.main_world.insert_component(minimap_cam, Node::ui_control(NodeType::Camera));
    app.main_world.insert_component(minimap_cam, 
        Transform3D::from_position(0.0, 100.0, 0.0)
            .with_rotation(-90.0_f32.to_radians(), 0.0, 0.0)
    );
    app.main_world.insert_component(minimap_cam, Camera3D {
        orthographic: true,
        orthographic_size: 50.0,
        viewport: Rect::new(0.7, 0.7, 0.3, 0.3),  // 右上角
        depth: 10,
        ..default()
    });
}
```

## 渲染顺序

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         多摄像机渲染顺序                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  depth 值越小，越早渲染（在底层）                                             │
│                                                                             │
│  默认 UI 摄像机:    depth = -100    (最先渲染，清除屏幕)                      │
│       ↓                                                                     │
│  游戏摄像机:        depth = 0       (渲染 3D 场景到屏幕)                      │
│       ↓                                                                     │
│  小地图摄像机:      depth = 10      (叠加渲染小地图)                          │
│       ↓                                                                     │
│  特效/UI 覆盖层:    depth = 100     (最后渲染，叠加在最上层)                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 高级用法

### 获取默认资源

```rust
// 获取主屏幕
fn use_screen(screen: Res<PrimaryScreen>) {
    println!("Screen size: {}x{}", screen.width, screen.height);
}

// 获取默认 UI 摄像机
fn use_ui_camera(camera: Res<DefaultUiCamera>) {
    println!("UI Camera entity: {:?}", camera.entity);
}

// 修改默认摄像机属性
fn modify_ui_camera(
    mut cameras: Query<&mut Camera3D>,
    default_cam: Res<DefaultUiCamera>,
) {
    if let Ok(mut cam) = cameras.get_mut(default_cam.entity) {
        cam.background_color = Color::DARK_GRAY;
    }
}
```

### 禁用默认 UI 摄像机

```rust
// 如果你想完全自定义，可以禁用默认 UI 摄像机
fn disable_default_ui_camera(
    mut commands: Commands,
    default_cam: Res<DefaultUiCamera>,
) {
    // 禁用渲染（不是删除）
    commands.entity(default_cam.entity)
        .insert(Disabled);  // 添加 Disabled 标记组件
}
```

## 总结

| 资源 | 类型 | 用途 | 可覆盖 |
|------|------|------|--------|
| **PrimaryScreen** | Resource | 全局窗口分辨率 | 否（运行时确定） |
| **Default UI Camera** | Entity + Components | 默认 UI 渲染 | 可以禁用/忽略 |

**设计原则：**
1. **开箱即用** - `App::new()` 后立即可渲染 UI
2. **可扩展** - 可以添加更多摄像机
3. **不强制** - 可以禁用默认资源，完全自定义
4. **一致性** - 所有 UI 使用相同的坐标系统

---

**文档版本**: v1.0  
**日期**: 2026-04-15  
**作者**: FeatherOS Team
