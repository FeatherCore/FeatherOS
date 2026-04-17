# FHRE 完整架构文档

## 目录

1. [概述](#1-概述)
2. [双世界架构](#2-双世界架构)
3. [默认资源系统](#3-默认资源系统)
4. [摄像机与视图系统](#4-摄像机与视图系统)
5. [输入系统](#5-输入系统)
6. [渲染管线](#6-渲染管线)
7. [模块详解](#7-模块详解)
8. [控件系统](#8-控件系统)
9. [NuttX SIM 平台支持](#9-nuttx-sim-平台支持)
10. [使用示例](#10-使用示例)

---

## 1. 概述

Feather Hybrid Render Engine (FHRE) v2.0 是一个基于声明式双世界架构的轻量级渲染引擎，专为嵌入式系统设计。

### 核心特性

- **声明式双世界架构** (Main World + Render World)
- **ECS (Entity-Component-System)** 设计
- **默认资源系统** (主屏幕 + 默认UI摄像机)
- **模块化系统调度**
- **嵌入式友好的无标准库实现**
- **NuttX SIM 平台支持** (类似 LVGL 的 framebuffer 对接)
- **文件夹/mod.rs 模块化结构**

### 设计哲学

```
开箱即用，可扩展覆盖
─────────────────────
App::new() 自动提供：
- PrimaryScreen (主屏幕)
- Default UI Camera (默认UI摄像机)

用户可以：
- 直接使用默认资源
- 创建额外资源
- 覆盖默认资源
```

---

## 2. 双世界架构

### 2.1 架构概览

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           FHRE 双世界架构 v2.0                               │
│                                                                             │
│  ┌─────────────────────────────┐        ┌─────────────────────────────┐    │
│  │       Main World            │        │       Render World          │    │
│  │      (游戏逻辑世界)          │        │       (渲染世界)             │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Entities (3D)       │    │        │  │ RenderCommands      │    │    │
│  │  │ - Transform3D       │    │        │  │ - DrawTriangle      │    │    │
│  │  │ - BookPage          │    │        │  │ - DrawRect          │    │    │
│  │  │ - Sprite            │    │        │  │ - (2D screen coords)│    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Systems             │    │        │  │ Pipeline            │    │    │
│  │  │ - PreUpdate         │    │        │  │ - SoftwareBackend   │    │    │
│  │  │ - Update            │    │        │  │ - Execute Commands  │    │    │
│  │  │ - PostUpdate        │    │        │  │ - Output Framebuffer│    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  │                             │        │                             │    │
│  │  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │    │
│  │  │ Resources           │    │        │  │ Framebuffer         │    │    │
│  │  │ - Time              │    │        │  │ - RGBA32 pixels     │    │    │
│  │  │ - PrimaryScreen     │    │        │  │ - width x height    │    │    │
│  │  │ - DefaultUiCamera   │    │        │  │                     │    │    │
│  │  └─────────────────────┘    │        │  └─────────────────────┘    │    │
│  └─────────────┬───────────────┘        └─────────────┬───────────────┘    │
│                │                                      │                     │
│                │          Extract Phase               │                     │
│                │    (3D World → 2D Screen Projection) │                     │
│                └──────────────────────────────────────▶                     │
│                                                                             │
│  核心设计原则:                                                              │
│  1. Main World: 声明"是什么" (3D 位置、旋转角度)                            │
│  2. Extract:    计算"怎么变" (3D变换 → 投影 → 2D屏幕坐标)                   │
│  3. Render:     执行"怎么画" (绘制三角形到 framebuffer)                     │
│                                                                             │
│  注: FHRE 的最小单位(Node)默认具有 3D 属性，2D 是 3D 的特例 (Z=0)           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 执行流程

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Startup    │───▶│    Update    │───▶│   Extract    │───▶│    Render    │
│   (一次)     │    │  (每帧执行)   │    │  (数据同步)   │    │  (生成画面)   │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
                           │                   │                   │
                           ▼                   ▼                   ▼
                    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
                    │ System 执行   │    │ Camera3D →   │    │ 渲染到屏幕    │
                    │ 游戏逻辑更新  │    │ View 转换    │    │ Present      │
                    └──────────────┘    └──────────────┘    └──────────────┘
```

---

## 3. 默认资源系统

### 3.1 核心设计

FHRE 提供内置的默认资源，简化应用开发：

1. **PrimaryScreen** - 主屏幕/窗口，全局分辨率定义
2. **Default UI Camera** - 默认 UI 摄像机，正交投影

这些资源在 `App::new()` 时自动创建。

### 3.2 默认资源架构

```
App::new() 自动创建：
┌─────────────────────────────────────────────────────────────────────┐
│ 1. PrimaryScreen (主屏幕)                                           │
│    ├── width: 800 (或实际显示分辨率)                                  │
│    ├── height: 600 (或实际显示分辨率)                                 │
│    ├── viewport: (0, 0, width, height)                              │
│    └── pixel_density: 1.0                                           │
│                                                                     │
│    这是 FHRE 的全局主窗口，所有渲染默认输出到这里                      │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ 2. Default UI Camera (默认 UI 摄像机)                                │
│    Entity: default_ui_camera                                        │
│    ├── Node { node_type: Camera }                                   │
│    ├── Transform3D {                                                │
│    │       position: (width/2, height/2, 100)  // 屏幕中心上方       │
│    │     }                                                          │
│    └── Camera3D {                                                   │
│            orthographic: true,            // 正交投影                │
│            orthographic_size: height/2,   // 视口范围                │
│            depth: -100,                   // 最先渲染                │
│            viewport: (0, 0, 1, 1)         // 全屏                    │
│        }                                                            │
│                                                                     │
│    这是默认的 UI 摄像机，所有 UI 元素默认使用这个摄像机渲染            │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.3 PrimaryScreen (主屏幕)

```rust
/// Primary Screen Resource
///
/// This is the default render target for FHRE.
/// It defines the main window/screen dimensions and properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimaryScreen {
    pub width: u32,              // Screen width in pixels
    pub height: u32,             // Screen height in pixels
    pub viewport: Rect,          // Full screen viewport
    pub pixel_density: f32,      // For high-DPI displays
    pub fullscreen: bool,
}

impl PrimaryScreen {
    pub fn new(width: u32, height: u32) -> Self
    pub fn dimensions(&self) -> (u32, u32)
    pub fn center(&self) -> Vec2
    pub fn aspect_ratio(&self) -> f32
    pub fn contains(&self, point: Vec2) -> bool
    pub fn normalized_to_screen(&self, normalized: Vec2) -> Vec2
}
```

### 3.4 Default UI Camera

```rust
/// Resource to store the default UI camera entity
#[derive(Clone, Copy, Debug)]
pub struct DefaultUiCamera {
    pub entity: Entity,
}

impl crate::resources::Resource for DefaultUiCamera {}
```

**创建逻辑：**

```rust
fn setup_default_ui_camera(&mut self, width: u32, height: u32) {
    let camera_entity = self.main_world.spawn();
    
    // 创建 UI camera node
    self.main_world.insert_component(
        camera_entity,
        Node::ui_control(NodeType::Camera)
    );
    
    // 位置: 屏幕中心上方
    self.main_world.insert_component(
        camera_entity,
        Transform3D::from_position(
            width as f32 / 2.0,   // Center X
            height as f32 / 2.0,  // Center Y
            100.0                 // Above the screen
        )
    );
    
    // 正交摄像机
    self.main_world.insert_component(
        camera_entity,
        Camera3D {
            orthographic: true,
            orthographic_size: height as f32 / 2.0,
            depth: -100,  // Render first
            viewport: Rect::new(0.0, 0.0, 1.0, 1.0),  // Fullscreen
            ..default()
        }
    );
    
    // 存储为默认 UI 摄像机资源
    self.main_world.resources_mut().insert(DefaultUiCamera {
        entity: camera_entity,
    });
}
```

### 3.5 使用示例

```rust
// 纯 UI 应用（使用默认资源）
let mut app = App::new();  // 自动创建主屏幕和默认 UI 摄像机
app.run();

// 游戏应用（添加游戏摄像机）
let mut app = App::new();
let game_cam = app.create_game_camera(
    Vec3::new(0.0, 5.0, -10.0),  // 位置
    Vec3::new(0.0, 0.0, 0.0),    // 看向原点
    60.0                          // FOV
);
app.run();
```

---

## 4. 摄像机与视图系统

### 4.1 两种摄像机概念

FHRE 中有两个层次的"摄像机"概念：

**概念 1: Main World 的 Camera3D 组件**
- 这是传统意义上的 3D 游戏摄像机
- 位于 3D 场景中的某个位置
- 决定"从什么角度观察 3D 世界"
- 将 3D 世界投影到 2D 屏幕

**概念 2: Render World 的 View（视图）**
- 这是渲染时的技术实现
- 由 Camera3D + Transform3D 计算得出
- 包含投影矩阵、视图矩阵等数学工具
- 每帧从 Camera3D 提取生成

### 4.2 核心区别

| 特性 | Camera3D (Main World) | View (Render World) |
|------|----------------------|---------------------|
| **本质** | 3D 场景中的实体 | 渲染用的数学工具 |
| **存在形式** | ECS 组件 | 运行时结构体 |
| **生命周期** | 持续存在 | 每帧重建 |
| **可动画** | ✅ 可以被动画系统控制 | ❌ 只读数据 |
| **用途** | 游戏逻辑控制视角 | GPU 渲染计算 |

### 4.3 工作流程

```
3D 场景 (Main World)          Extract          2D 屏幕 (Render World)
───────────────────                           ─────────────────────

摄像机在 (0, 2, -5)                              
看向原点 (0, 0, 0)              →               立方体投影到 (400, 300)
FOV 60°                                       球体投影到 (600, 250)
                                                
立方体在 (0, 0, 0)                              
球体在 (3, 0, 2)                               
```

### 4.4 视图系统 (View System)

```rust
/// View configuration - defines a render target
pub struct View {
    pub viewport: Rect,
    pub projection: Mat4,
    pub view: Mat4,
    pub view_projection: Mat4,
    pub camera_position: Vec3,
    pub near: f32,
    pub far: f32,
    pub orthographic: bool,
}

impl View {
    /// Create a new 2D orthographic view
    pub fn new_2d(width: f32, height: f32) -> Self
    
    /// Create a new 3D perspective view
    pub fn new_3d(width: f32, height: f32, fov_degrees: f32) -> Self
    
    /// Project world position to screen coordinates
    pub fn world_to_screen(&self, world_pos: Vec3) -> Option<(f32, f32)>
}

/// View bundle - combines view, target, and clear config
pub struct ViewBundle {
    pub view: View,
    pub target: ViewTarget,
    pub clear: ClearConfig,
}
```

### 4.5 摄像机控制与动画

摄像机作为普通 Node 实体，可以被动画系统完全控制：

```rust
// 第一人称视角（FPS）
fn fps_camera_control(
    mut camera_query: Query<(&mut Transform3D, &Camera3D), With<PlayerCamera>>,
    input: Res<Input>,
    time: Res<Time>,
) {
    for (mut transform, _camera) in camera_query.iter_mut() {
        // 鼠标控制视角旋转
        transform.rotation.y += input.mouse_delta.x * SENSITIVITY;
        transform.rotation.x -= input.mouse_delta.y * SENSITIVITY;
        
        // 限制垂直视角
        transform.rotation.x = transform.rotation.x.clamp(-PI/2.0, PI/2.0);
        
        // WASD 移动
        let forward = transform.forward();
        if input.key_pressed(Key::W) {
            transform.position += forward * MOVE_SPEED * time.delta();
        }
    }
}

// 第三人称视角（跟随角色）
fn third_person_camera(
    mut camera_query: Query<&mut Transform3D, With<FollowCamera>>,
    target_query: Query<&Transform3D, With<Player>>,
    time: Res<Time>,
) {
    let target = target_query.single();
    
    for mut camera_transform in camera_query.iter_mut() {
        // 计算目标位置（角色后方 + 上方）
        let offset = Vec3::new(0.0, 2.0, -5.0);
        let target_pos = target.position + target.rotation * offset;
        
        // 平滑跟随
        camera_transform.position = camera_transform.position.lerp(
            target_pos,
            5.0 * time.delta()
        );
        
        // 看向角色
        camera_transform.look_at(target.position);
    }
}
```

### 4.6 多视图渲染

```rust
// 场景：游戏主视角 + 小地图 + 后视镜（赛车游戏）
fn setup_racing_cameras(mut commands: Commands) {
    // 主驾驶视角
    commands.spawn((
        Node::game_entity(NodeType::Camera),
        Transform3D::from_position(0.0, 1.2, 0.5), // 驾驶座位置
        Camera3D::new()
            .with_fov(70.0)
            .with_viewport(Rect::new(0.0, 0.0, 1.0, 1.0)) // 全屏
            .with_depth(0),
        DriverCamera,
    ));
    
    // 小地图（俯视）
    commands.spawn((
        Node::ui_control(NodeType::Camera),
        Transform3D::from_position(0.0, 100.0, 0.0) // 高空俯视
            .with_rotation(-90.0_f32.to_radians(), 0.0, 0.0),
        Camera3D::new()
            .orthographic(50.0) // 正交投影
            .with_viewport(Rect::new(0.7, 0.7, 0.28, 0.28)) // 右上角
            .with_depth(1)
            .with_clear_config(ClearConfig::none()),
        MinimapCamera,
    ));
}
```

---

## 5. 输入系统

### 5.1 设计哲学

FHRE 的输入系统融合了 **Bevy 的 ECS 事件驱动架构** 和 **LVGL 的输入设备抽象**，采用混合设计模式：

```
底层硬件 (evdev / nuttx input)
    ↓
输入驱动 (读取原始事件)
    ↓
事件分发 (Message 系统)
    ↓
输入系统 (System)
    ↓
资源 (Resource)
    ↓
游戏/UI 逻辑查询
```

### 5.2 架构对比

| 特性 | Bevy | LVGL | FHRE |
|------|------|------|------|
| 架构模式 | ECS (Resource/Message) | 回调驱动 + 定时器 | **ECS + 事件驱动** |
| 状态管理 | HashSet 资源 | 结构体字段 | **ButtonInput\<T\> 资源** |
| 事件类型 | Message | lv_indev_data_t | **InputEvent** |
| 帧处理 | 每帧清空 just_* | 定时器周期性读取 | **PreUpdate 阶段处理** |
| 设备类型 | 分离资源 | 统一 lv_indev_t | **泛型 ButtonInput\<T\>** |
| 坐标系统 | 窗口相对 | 屏幕绝对 | **PrimaryScreen 归一化** |

### 5.3 核心数据结构

#### 5.3.1 输入事件

```rust
/// 输入事件类型
pub enum InputEvent {
    /// 鼠标按键事件
    MouseButton {
        button: MouseButton,  // Left, Right, Middle
        state: ButtonState,   // Pressed, Released
        position: Vec2,       // 屏幕坐标
    },
    /// 鼠标移动事件
    MouseMotion {
        delta: Vec2,          // 相对位移
        position: Vec2,       // 绝对位置
    },
    /// 鼠标滚轮事件
    MouseWheel {
        delta: Vec2,          // 滚动量
        unit: ScrollUnit,     // Line, Pixel
    },
    /// 键盘按键事件
    Keyboard {
        key_code: KeyCode,    // 物理键码
        state: ButtonState,   // Pressed, Released
    },
    /// 触摸事件
    Touch {
        id: u64,              // 触摸 ID
        phase: TouchPhase,    // Started, Moved, Ended, Canceled
        position: Vec2,       // 触摸位置
        force: Option<f32>,   // 按压力度 (0.0 - 1.0)
    },
}

/// 按钮状态
pub enum ButtonState {
    Pressed,
    Released,
}

/// 触摸阶段
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Canceled,
}
```

#### 5.3.2 输入资源

```rust
/// 通用按钮输入资源
pub struct ButtonInput<T: Clone + Eq + Hash> {
    pressed: HashSet<T>,       // 当前按下的按钮
    just_pressed: HashSet<T>,  // 当前帧刚按下的按钮
    just_released: HashSet<T>, // 当前帧刚释放的按钮
}

impl<T: Clone + Eq + Hash> ButtonInput<T> {
    /// 注册按下
    pub fn press(&mut self, input: T) {
        if self.pressed.insert(input.clone()) {
            self.just_pressed.insert(input);
        }
    }
    
    /// 注册释放
    pub fn release(&mut self, input: T) {
        if self.pressed.remove(&input) {
            self.just_released.insert(input);
        }
    }
    
    /// 检查是否按下
    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }
    
    /// 检查当前帧是否刚按下
    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }
    
    /// 检查当前帧是否刚释放
    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }
    
    /// 清空 just_pressed/just_released (每帧调用)
    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

/// 触摸状态资源
pub struct Touches {
    pressed: HashMap<u64, Touch>,
    just_pressed: HashMap<u64, Touch>,
    just_released: HashMap<u64, Touch>,
}

/// 单个触摸信息
pub struct Touch {
    id: u64,
    start_position: Vec2,
    previous_position: Vec2,
    position: Vec2,
}

impl Touch {
    /// 当前位置 - 上次位置 (移动量)
    pub fn delta(&self) -> Vec2 {
        self.position - self.previous_position
    }
    
    /// 当前位置 - 起始位置 (总位移)
    pub fn distance(&self) -> Vec2 {
        self.position - self.start_position
    }
}
```

#### 5.3.3 鼠标/键盘按键枚举

```rust
/// 鼠标按键
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

/// 键盘键码 (物理位置)
pub enum KeyCode {
    KeyA, KeyB, KeyC,    // 字母
    Digit0, Digit1,      // 数字
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight, // 方向键
    Space, Enter, Escape, Tab,                 // 功能键
    F1, F2, ... F24,                           // F 键
    ShiftLeft, ShiftRight,                     // Shift
    ControlLeft, ControlRight,                 // Ctrl
    AltLeft, AltRight,                         // Alt
    // ... 更多
}
```

### 5.4 输入处理流程

#### 5.4.1 帧处理时序

```
Frame N:
┌──────────────────────────────────────────────────────────────┐
│ 1. 平台层收集输入事件                                         │
│    └→ evdev_read() / nuttx_input()                           │
│    └→ 转换为 InputEvent 并加入消息队列                        │
│                                                              │
│ 2. PreUpdate 阶段 - 输入处理系统执行                          │
│    ├── input_clear_system()                                  │
│    │   └→ ButtonInput<T>::clear()                            │
│    │                                                          │
│    ├── mouse_input_system()                                  │
│    │   └→ 处理 MouseButton, MouseMotion, MouseWheel          │
│    │   └→ 更新 ButtonInput<MouseButton>                      │
│    │                                                          │
│    ├── keyboard_input_system()                               │
│    │   └→ 处理 Keyboard                                      │
│    │   └→ 更新 ButtonInput<KeyCode>                          │
│    │                                                          │
│    └── touch_input_system()                                  │
│        └→ 处理 Touch                                        │
│        └→ 更新 Touches                                       │
│                                                              │
│ 3. Update 阶段 - 用户系统执行                                 │
│    └→ 查询 Res<ButtonInput<T>> / Res<Touches>               │
│                                                              │
│ 4. PostUpdate 阶段                                           │
│    └→ (可选) 清空临时输入数据                                │
└──────────────────────────────────────────────────────────────┘
```

#### 5.4.2 事件处理系统

```rust
/// 清空 just_pressed/just_released (每帧调用)
pub fn input_clear_system(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    mouse.bypass_change_detection().clear();
    keys.bypass_change_detection().clear();
}

/// 鼠标输入处理系统
pub fn mouse_input_system(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut mouse_motion_events: MessageReader<InputEvent>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
) {
    for event in mouse_motion_events.read() {
        match event {
            InputEvent::MouseButton { button, state, position } => {
                match state {
                    ButtonState::Pressed => mouse.press(*button),
                    ButtonState::Released => mouse.release(*button),
                }
            }
            InputEvent::MouseWheel { delta, .. } => {
                scroll.delta += delta;
            }
            _ => {}
        }
    }
}

/// 键盘输入处理系统
pub fn keyboard_input_system(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut keyboard_events: MessageReader<InputEvent>,
) {
    for event in keyboard_events.read() {
        if let InputEvent::Keyboard { key_code, state } = event {
            match state {
                ButtonState::Pressed => keys.press(*key_code),
                ButtonState::Released => keys.release(*key_code),
            }
        }
    }
}

/// 触摸输入处理系统
pub fn touch_input_system(
    mut touches: ResMut<Touches>,
    mut touch_events: MessageReader<InputEvent>,
) {
    // 更新上一帧的位置
    for touch in touches.pressed.values_mut() {
        touch.previous_position = touch.position;
    }
    
    for event in touch_events.read() {
        if let InputEvent::Touch { id, phase, position, force } = event {
            match phase {
                TouchPhase::Started => {
                    let touch = Touch {
                        id: *id,
                        start_position: *position,
                        previous_position: *position,
                        position: *position,
                    };
                    touches.pressed.insert(*id, touch);
                    touches.just_pressed.insert(*id, touch);
                }
                TouchPhase::Moved => {
                    if let Some(mut touch) = touches.pressed.get(id).cloned() {
                        touch.position = *position;
                        touches.pressed.insert(*id, touch);
                    }
                }
                TouchPhase::Ended => {
                    if let Some((_, touch)) = touches.pressed.remove_entry(id) {
                        touches.just_released.insert(*id, touch);
                    }
                }
                TouchPhase::Canceled => {
                    touches.pressed.remove(id);
                }
            }
        }
    }
}
```

### 5.8 SIM 环境下输入系统对接参考 (LVGL SDL 驱动)

FHRE 输入系统的设计参考了 LVGL 在 NuttX SIM 环境下的 SDL 驱动对接方式。

#### 5.8.1 LVGL SIM 输入架构

LVGL 在 SIM 环境中使用 SDL 作为底层窗口和输入系统：

```
SDL 事件循环 (SDL_PollEvent)
    ↓
lv_sdl_window.c: sdl_event_handler() (每 5ms 定时器调用)
    ↓ 分发事件到各驱动
    ├── lv_sdl_mouse.c: lv_sdl_mouse_handler()
    └── lv_sdl_keyboard.c: lv_sdl_keyboard_handler()
    ↓ 更新驱动内部状态 → 调用 lv_indev_read() 触发 LVGL 输入系统
lv_indev.c: 事件处理 → UI 更新
```

#### 5.8.2 SDL 事件循环

```c
// lv_sdl_window.c
static lv_timer_t * event_handler_timer;

lv_display_t * lv_sdl_window_create(int32_t hor_res, int32_t ver_res)
{
    if(!inited) {
        SDL_Init(SDL_INIT_VIDEO);
        SDL_StartTextInput();
        event_handler_timer = lv_timer_create(sdl_event_handler, 5, NULL);
        lv_tick_set_cb(SDL_GetTicks);
        inited = true;
    }
    // ...
}

static void sdl_event_handler(lv_timer_t * t)
{
    SDL_Event event;
    while(SDL_PollEvent(&event)) {
        lv_sdl_mouse_handler(&event);
        lv_sdl_keyboard_handler(&event);
        
        if(event.type == SDL_QUIT) {
            SDL_Quit();
            lv_deinit();
            exit(0);
        }
    }
}
```

#### 5.8.3 鼠标驱动对接

LVGL 使用驱动私有数据 + 读取回调的方式：

```c
typedef struct {
    int16_t last_x;
    int16_t last_y;
    bool left_button_down;
} lv_sdl_mouse_t;

lv_indev_t * lv_sdl_mouse_create(void)
{
    lv_sdl_mouse_t * dsc = lv_malloc_zeroed(sizeof(lv_sdl_mouse_t));
    lv_indev_t * indev = lv_indev_create();
    
    lv_indev_set_type(indev, LV_INDEV_TYPE_POINTER);
    lv_indev_set_read_cb(indev, sdl_mouse_read);
    lv_indev_set_driver_data(indev, dsc);
    lv_indev_set_mode(indev, LV_INDEV_MODE_EVENT);  // 事件驱动模式
    
    return indev;
}

static void sdl_mouse_read(lv_indev_t * indev, lv_indev_data_t * data)
{
    lv_sdl_mouse_t * dsc = lv_indev_get_driver_data(indev);
    data->point.x = dsc->last_x;
    data->point.y = dsc->last_y;
    data->state = dsc->left_button_down ? LV_INDEV_STATE_PRESSED 
                                        : LV_INDEV_STATE_RELEASED;
}

void lv_sdl_mouse_handler(SDL_Event * event)
{
    // ... 找到对应的输入设备
    switch(event->type) {
        case SDL_MOUSEBUTTONDOWN:
            dsc->left_button_down = true;
            dsc->last_x = event->motion.x / zoom;
            dsc->last_y = event->motion.y / zoom;
            break;
        case SDL_MOUSEBUTTONUP:
            dsc->left_button_down = false;
            break;
        case SDL_MOUSEMOTION:
            dsc->last_x = event->motion.x / zoom;
            dsc->last_y = event->motion.y / zoom;
            break;
    }
    lv_indev_read(indev);  // 触发 LVGL 输入处理
}
```

#### 5.8.4 FHRE 参考实现

FHRE 的输入系统采用 ECS 架构，但借鉴了 LVGL 的以下设计：

| LVGL 设计 | FHRE 对应实现 | 说明 |
|----------|--------------|------|
| `lv_sdl_mouse_t` 驱动私有数据 | `ButtonInput<MouseButton>` Resource | ECS Resource 替代驱动私有数据 |
| `sdl_mouse_read` 回调 | `input_process_system` | ECS System 替代读取回调 |
| `lv_indev_read(indev)` 触发处理 | `App::update()` 中的输入阶段 | 帧更新时处理 |
| 事件驱动模式 `LV_INDEV_MODE_EVENT` | `App.input_events` 队列 | 事件队列替代事件模式 |
| `lv_sdl_window_create()` 创建输入设备 | `init_input_system()` | 资源初始化 |
| 按键缓冲区 + dummy_read | `just_pressed/just_released` | 帧边界状态管理 |

FHRE SIM 环境输入对接流程图：

```
main()
    ↓
App::new()
    ├── init_input_system()     ← 创建输入资源
    │   ├── MouseInput
    │   ├── KeyboardInput
    │   └── Touches
    └── ...

主循环:
    run_with_callback(|app| {
        // 平台层收集输入事件 (类似 SDL_PollEvent)
        #[cfg(feature = "sim")]
        {
            if let Some(driver) = &mut app.input_driver {
                let events = driver.read_events();
                app.push_input_events(&events);
            }
        }
    })
    ↓
App::update()
    ├── input_clear_system()     ← 清空 just_pressed/just_released
    ├── input_process_system()   ← 处理 input_events，更新资源
    ├── main_world.run_systems() ← 用户系统执行
    ├── extract_renderable_components()
    ├── render_world.execute_render()
    └── sim_display.present()
```

FHRE 平台输入驱动 (类似 `lv_sdl_mouse_handler`):

```rust
// driver.rs - 平台输入驱动
pub struct PlatformInputDriver {
    fd: c_int,
    current_x: f32,
    current_y: f32,
}

impl PlatformInputDriver {
    pub fn new(path: &str) -> Option<Self> { /* ... */ }
    
    pub fn read_events(&mut self) -> Vec<InputEvent> {
        let mut events = Vec::new();
        loop {
            match self.read_single_event() {
                Some(event) => events.push(event),
                None => break,
            }
        }
        events
    }
    
    fn convert_raw_event(&mut self, raw: &RawInputEvent) -> Option<InputEvent> {
        match raw.type_ {
            EV_KEY => { /* 键盘/按钮 */ }
            EV_REL => { /* 鼠标移动 */ }
            EV_ABS => { /* 触摸/绝对坐标 */ }
            _ => None,
        }
    }
}
```

FHRE 应用集成 (类似 `sdl_event_handler`):

```rust
// app.rs - App 生命周期集成
pub struct App {
    pub main_world: MainWorld,
    pub input_events: Vec<InputEvent>,
    // ...
}

impl App {
    pub fn push_input_event(&mut self, event: InputEvent) {
        self.input_events.push(event);
    }
    
    pub fn update(&mut self) {
        // 1. 清空 just_* 状态
        input_clear_system(self.main_world.resources_mut());
        
        // 2. 处理输入事件
        input_process_system(self.main_world.resources_mut(), &self.input_events);
        
        // 3. 清空事件队列
        self.input_events.clear();
        
        // 4. 继续其他阶段...
        self.main_world.run_systems();
    }
}
```

### 5.9 与 LVGL/Bevy 的完整对比

| 特性 | Bevy | LVGL | FHRE |
|------|------|------|------|
| 架构模式 | ECS (Resource/Message) | 回调驱动 + 定时器 | **ECS + 事件队列** |
| 状态管理 | HashSet 资源 | 结构体字段 | **ButtonInput\<T\> 资源** |
| 事件类型 | Message | lv_indev_data_t | **InputEvent** |
| 帧处理 | 每帧清空 just_* | 定时器周期性读取 | **App.update() 中处理** |
| 驱动私有数据 | 不适用 | `lv_indev_set_driver_data()` | **Resource 替代** |
| 事件触发 | 自动 | `lv_indev_read()` 手动调用 | **App.push_input_event()** |
| 坐标系统 | 窗口相对 | 屏幕绝对 | **PrimaryScreen 归一化** |
| SIM 集成 | winit | SDL | **evdev / SDL** |

---

### 5.10 平台输入驱动

#### 5.5.1 Linux evdev 驱动

```rust
/// evdev 输入驱动
pub struct EvdevDriver {
    fd: i32,                    // /dev/input/event* 文件描述符
    capabilities: Vec<u16>,     // 设备能力
}

impl EvdevDriver {
    /// 读取输入事件
    pub fn read(&mut self) -> Option<InputEvent> {
        let mut event = input_event { type: 0, code: 0, value: 0 };
        
        if read(self.fd, &mut event as *mut _ as *mut _, size_of_val(&event)) > 0 {
            match event.type {
                EV_KEY => {
                    let state = if event.value == 1 { 
                        ButtonState::Pressed 
                    } else { 
                        ButtonState::Released 
                    };
                    // 转换为 InputEvent::MouseButton 或 InputEvent::Keyboard
                }
                EV_REL => {
                    // 相对移动 (鼠标)
                    InputEvent::MouseMotion { ... }
                }
                EV_ABS => {
                    // 绝对坐标 (触摸)
                    InputEvent::Touch { ... }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
```

#### 5.5.2 NuttX 输入驱动

```rust
/// NuttX 输入驱动
pub struct NuttxInputDriver {
    fd: i32,                    // 输入设备文件描述符
}

impl NuttxInputDriver {
    /// 读取输入事件
    pub fn read(&mut self) -> Option<InputEvent> {
        let mut buf = [0u8; 16];
        
        if read(self.fd, buf.as_mut_ptr() as *mut _, buf.len()) > 0 {
            // 解析 NuttX input_event 格式
            // 转换为 InputEvent
        } else {
            None
        }
    }
}
```

### 5.6 常用 API

#### 5.6.1 鼠标输入

```rust
fn mouse_system(
    mouse: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
) {
    // 检查按键状态
    if mouse.pressed(MouseButton::Left) { }
    if mouse.just_pressed(MouseButton::Right) { }
    if mouse.just_released(MouseButton::Middle) { }
    
    // 获取滚轮滚动
    let scroll_delta = scroll.delta;
}
```

#### 5.6.2 键盘输入

```rust
fn keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
) {
    // WASD 移动
    if keys.pressed(KeyCode::KeyW) { }
    if keys.pressed(KeyCode::KeyA) { }
    if keys.just_pressed(KeyCode::Escape) { } // 暂停
    
    // 组合键
    if keys.pressed(KeyCode::ControlLeft) 
        && keys.just_pressed(KeyCode::KeyC) 
    {
        println!("Copy!");
    }
}
```

#### 5.6.3 触摸输入

```rust
fn touch_system(
    touches: Res<Touches>,
) {
    // 获取所有活动触摸
    for touch in touches.pressed.values() {
        let pos = touch.position;
        let delta = touch.delta();      // 移动量
        let distance = touch.distance(); // 总位移
    }
    
    // 检查特定触摸
    if touches.just_pressed(id) { }
    if let Some(touch) = touches.just_released.get(&id) { }
}
```

### 5.7 输入系统与 UI 集成

FHRE 的输入系统与 UI 控件系统紧密集成：

```
InputEvent (位置)
    ↓
Hit Testing (指针命中测试)
    ↓
查找被点击的 Node/控件
    ↓
发送 UI 事件 (Click, Hover, Drag)
    ↓
控件响应事件
```

```rust
/// 命中测试系统
fn hit_test_system(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut ui_events: EventWriter<UiEvent>,
    nodes: Query<(Entity, &Node, &Transform3D)>,
) {
    // 获取鼠标/触摸位置
    let pointer_pos = get_pointer_position(&mouse, &touches);
    
    if let Some(pos) = pointer_pos {
        // 查找命中的节点
        for (entity, node, transform) in nodes.iter() {
            if is_point_in_rect(pos, transform.position, node.size) {
                // 发送 UI 事件
                ui_events.send(UiEvent::Hover { entity, position: pos });
            }
        }
    }
}
```

---

## 6. 渲染管线

### 6.1 架构分层

FHRE 的渲染管线采用三层架构，实现职责分离：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         渲染管线架构                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Layer 1: RenderWorld                                               │   │
│  │  - 渲染数据管理（命令队列、视图、阶段）                                │   │
│  │  - 封装渲染后端，提供统一接口                                         │   │
│  │  - 对外隐藏具体渲染实现                                               │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Layer 2: Pipeline                                                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐ │   │
│  │  │  Software   │  │    GPU      │  │      Hybrid                 │ │   │
│  │  │  Backend    │  │  Backend    │  │   (未来)                    │ │   │
│  │  │  (CPU)      │  │  (OpenGL)   │  │                             │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────┘ │   │
│  │                                                                     │   │
│  │  - 渲染执行层，支持多后端切换                                         │   │
│  │  - 批处理、命令转换、资源管理                                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Layer 3: Framebuffer Output                                        │   │
│  │  - RGBA32 像素缓冲区                                                 │   │
│  │  - 输出到显示设备 (/dev/fb0)                                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.2 渲染阶段系统

FHRE 的渲染阶段系统借鉴 Bevy 的设计：

```
渲染顺序:

1. Background    →  清除/背景绘制
     ↓
2. Opaque2d      →  不透明 2D 物体（精灵、UI 背景）
     ↓
3. Opaque3d      →  不透明 3D 物体（模型）
     ↓
4. AlphaMask     →  Alpha 测试物体（镂空纹理）
     ↓
5. Transparent   →  透明物体（需要排序，从后往前）
     ↓
6. Ui            →  UI 覆盖层（最上层）
```

### 6.3 渲染阶段类型

```rust
pub enum RenderPhaseType {
    Background = 0,     // 背景/清除
    Opaque2d = 1,       // 不透明 2D 物体
    Opaque3d = 2,       // 不透明 3D 物体
    AlphaMask = 3,      // Alpha 测试物体
    Transparent = 4,    // 透明物体（需要排序）
    Ui = 5,             // UI 覆盖层
}

pub struct PhaseItem {
    pub sort_key: i32,
    pub z_depth: f32,
    pub entity_id: Option<u32>,
    pub draw_command_index: usize,
    pub batchable: bool,
    pub batch_key: u64,
}

pub struct RenderPhase {
    pub phase_type: RenderPhaseType,
    pub items: Vec<PhaseItem>,
    pub sorted: bool,
}
```

### 6.4 Pipeline 模块详解

#### 6.4.1 模块结构

```
pipeline/
├── mod.rs           # 模块导出
├── backend.rs       # SoftwareBackend (CPU 软件渲染)
├── renderer.rs      # Renderer trait (抽象接口)
├── batch.rs         # 批处理逻辑
└── backend.rs       # 软件渲染后端（三角形光栅化）
```

#### 6.4.2 3D 渲染与背面剔除

FHRE 支持简单的 3D 渲染，使用画家算法（Painter's Algorithm）进行深度排序：

```
3D 渲染流程:

1. Transform (变换)
   - 本地坐标 → 世界坐标 (Transform3D)
   - 世界坐标 → 视图坐标 (View Matrix)
   - 视图坐标 → 裁剪坐标 (Projection Matrix)
   - 裁剪坐标 → 屏幕坐标 (Viewport Transform)

2. Backface Culling (背面剔除)
   - 在屏幕空间计算叉积判断面的朝向
   - cross_z < 0: 面朝向相机（可见）
   - cross_z >= 0: 面背向相机（剔除）
   - 注意：屏幕坐标系 Y 轴向下，叉积符号与标准坐标系相反

3. Painter's Algorithm (画家算法)
   - 按 view-space Z 排序（远的先画）
   - **重要**：右手坐标系 view space 中，相机看向 -Z 方向
   - **Z 值越小（越负）表示离相机越远，Z 值越大（越接近 0）表示离相机越近**
   - 画家算法：先画远的（Z 值小的/更负的），后画近的（Z 值大的/接近 0 的）
   - 排序使用升序：`faces.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap())`

4. Triangle Rasterization (三角形光栅化)
   - 使用叉积法判断点是否在三角形内
   - 支持边界框裁剪和视口裁剪
```

**背面剔除实现：**

```rust
// 在屏幕空间计算叉积
let edge1 = v1 - v0;
let edge2 = v3 - v0;
let cross_z = edge1.x * edge2.y - edge1.y * edge2.x;

// 屏幕坐标系 Y 轴向下，所以 cross_z < 0 表示面朝向相机
if cross_z < 0.0 {
    // 面朝向相机，保留
}
```

**深度排序实现：**

```rust
// 计算面的平均 view-space Z
let avg_view_z = (view_z[v0] + view_z[v1] + view_z[v2] + view_z[v3]) / 4.0;

// 按 Z 升序排列（Z 小的在前，即远的先画）
// 右手坐标系 view space：相机看向 -Z，Z 越小（越负）表示越远
// 画家算法：先画远的（Z 值小的/更负的），后画近的（Z 值大的/接近 0 的）
visible_faces.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
```

#### 6.4.3 SoftwareBackend (CPU 软件渲染)

```rust
/// Software rendering backend - simulates a GPU pipeline on CPU
pub struct SoftwareBackend {
    framebuffer: Vec<u32>,
    width: u32,
    height: u32,
    viewport: Rect,
}

impl SoftwareBackend {
    /// Execute a single render command
    pub fn execute_command(&mut self, command: &RenderCommand) {
        match command {
            RenderCommand::Clear { color } => self.clear(*color),
            RenderCommand::DrawRect { rect, color } => self.fill_rect(*rect, *color),
            RenderCommand::DrawLine { start, end, color, thickness } => 
                self.draw_line(*start, *end, *color, *thickness),
            RenderCommand::DrawTriangle { p0, p1, p2, color } => 
                self.fill_triangle(*p0, *p1, *p2, *color),
            // ...
        }
    }

    /// Execute multiple commands in batch
    pub fn execute_commands(&mut self, commands: &[RenderCommand]) {
        for command in commands {
            self.execute_command(command);
        }
    }
}
```

#### 6.4.4 Renderer Trait (未来扩展)

```rust
/// Renderer trait - implemented by all rendering backends
pub trait Renderer {
    fn new(width: u32, height: u32) -> Self where Self: Sized;
    fn execute_commands(&mut self, commands: &[RenderCommand]);
    fn framebuffer(&self) -> &[u32];
    fn resize(&mut self, width: u32, height: u32);
    fn set_viewport(&mut self, rect: Rect);
    fn reset(&mut self);
}

pub enum RendererType {
    Software,   // Pure software CPU rendering
    Gpu,        // GPU accelerated rendering
    Hybrid,     // Hybrid: GPU for batches, CPU for small tasks
}
```

### 6.5 RenderWorld 与 Pipeline 集成

```rust
pub struct RenderWorld {
    // ... 其他字段 ...
    /// Rendering backend (encapsulated)
    backend: SoftwareBackend,  // Could be GPU backend in future
}

impl RenderWorld {
    /// Execute all pending render commands
    pub fn execute_render(&mut self) {
        self.backend.execute_commands(&self.commands);
    }

    /// Get framebuffer reference
    pub fn framebuffer(&self) -> &[u32] {
        self.backend.framebuffer()
    }
}
```

### 6.6 与 Bevy 的对比

| 特性 | Bevy | FHRE |
|------|------|------|
| 阶段类型 | Binned + Sorted | 简化版 Binned + Sorted |
| 批处理 | GPU 预处理 | CPU 批处理（嵌入式限制） |
| 多视图 | 完整支持 | 简化支持（单视图为主） |
| 排序 | GPU 计算着色器 | CPU 排序 |
| 后端架构 | 单一 GPU 后端 | 可切换 Software/GPU/Hybrid |

---

## 7. 模块详解

### 7.1 项目结构

```
FeatherOS/
├── apps/
│   ├── fhre/                          # FHRE 核心库 v2.0
│   │   └── rust/
│   │       ├── Cargo.toml
│   │       ├── Makefile
│   │       └── src/
│   │           ├── lib.rs             # 库入口，模块声明
│   │           ├── app/               # App 模块
│   │           │   ├── mod.rs         # 衔接：导出 app, config
│   │           │   ├── app.rs         # App 实现
│   │           │   └── config.rs      # App 配置
│   │           ├── main_world/        # Main World 模块
│   │           │   ├── mod.rs         # 衔接：导出 world, entity, component, system, system_param
│   │           │   ├── world.rs       # MainWorld 实现
│   │           │   ├── entity.rs      # Entity 实现
│   │           │   ├── component.rs   # Component 实现
│   │           │   ├── system.rs      # System 实现 (声明式 ECS)
│   │           │   └── system_param.rs # SystemParam 实现 (Res, ResMut, Query)
│   │           ├── render_world/      # Render World 模块
│   │           │   ├── mod.rs         # 衔接：导出 world, command, object
│   │           │   ├── world.rs       # RenderWorld 实现
│   │           │   ├── command.rs     # RenderCommand 实现
│   │           │   ├── object.rs      # RenderObject 实现
│   │           │   ├── phase.rs       # RenderPhase 实现
│   │           │   └── view.rs        # View 实现
│   │           ├── extract/           # Extract 模块
│   │           │   ├── mod.rs         # 衔接：导出 extract
│   │           │   └── extract.rs     # Extract 实现
│   │           ├── schedule/          # Schedule 模块
│   │           │   ├── mod.rs         # 衔接：导出 schedule, label, set
│   │           │   ├── schedule.rs    # Schedule 实现
│   │           │   ├── label.rs       # Label 实现
│   │           │   └── set.rs         # SystemSet 实现
│   │           ├── resources/         # Resources 模块
│   │           │   ├── mod.rs         # 衔接：导出 resources, time, config, screen
│   │           │   ├── resources.rs   # Resources 实现
│   │           │   ├── time.rs        # Time 实现
│   │           │   ├── config.rs      # Config 实现
│   │           │   └── screen.rs      # PrimaryScreen 实现
│   │           ├── pipeline/          # Pipeline 模块（渲染管线）
│   │           │   ├── mod.rs         # 衔接：导出 pipeline 子模块
│   │           │   └── batch.rs       # 批处理实现
│   │           ├── math/              # Math 模块
│   │           │   ├── mod.rs
│   │           │   ├── vec2.rs
│   │           │   ├── vec3.rs
│   │           │   ├── color.rs
│   │           │   └── rect.rs
│   │           ├── animation/         # Animation 模块（动画系统）
│   │           │   ├── mod.rs         # 衔接：导出 animation 子模块
│   │           │   ├── clip.rs        # AnimationClip 动画剪辑
│   │           │   ├── curve.rs       # 动画曲线和插值
│   │           │   ├── easing.rs      # 缓动函数（30+种）
│   │           │   ├── player.rs      # AnimationPlayer 组件
│   │           │   ├── graph.rs       # 动画图（混合）
│   │           │   ├── transition.rs  # 动画过渡
│   │           │   └── property.rs    # 动画属性
│   │           ├── platform/          # Platform 模块
│   │           │   ├── mod.rs         # 衔接：导出 sim, nuttx, default, framebuffer
│   │           │   ├── sim.rs         # NuttX SIM 平台支持
│   │           │   ├── nuttx.rs       # NuttX 平台支持
│   │           │   ├── default.rs     # 默认平台支持
│   │           │   └── framebuffer.rs # Framebuffer 抽象实现
│   │           └── lib.rs             # 库入口
```

### 7.2 模块化设计原则

**文件夹/mod.rs 结构：**

```rust
// mod.rs 仅作为衔接，导出同级目录下的子模块
// 不包含实际功能实现

// 示例：main_world/mod.rs
mod world;
mod entity;
mod component;
mod system;

pub use world::MainWorld;
pub use entity::Entity;
pub use component::{Component, Transform, Sprite, Velocity};
pub use system::{System, IntoSystem};
```

**设计原则：**

1. **mod.rs 只负责衔接**：声明子模块并导出公共接口
2. **功能实现在同级 .rs 文件**：如 `world.rs`, `entity.rs` 等
3. **清晰的模块边界**：每个模块有明确的职责
4. **易于扩展**：添加新功能只需创建新文件并在 mod.rs 中导出

---

## 8. 控件系统

### 8.1 架构对比

| 特性 | LVGL | Bevy | FHRE (设计目标) |
|------|------|------|-----------------|
| **架构** | 对象树 (Object Tree) | ECS 双世界 | **ECS 双世界** |
| **最小单位** | `lv_obj_t` (基础对象) | Entity + Components | **Node + Components** |
| **关系管理** | 父子指针 | 组件关联 | **组件关联 (无父子)** |
| **数据布局** | AOS (Array of Structs) | SOA (Structure of Arrays) | **SOA (缓存友好)** |
| **属性存储** | 对象内部字段 | 组件分离 | **组件分离** |
| **渲染方式** | CPU/GPU 混合 | GPU 批处理 | **GPU 批处理 (2D/3D 统一)** |
| **布局系统** | 内置布局 (Flex/Grid) | 无内置 (需自定义) | **内置 2D/3D 混合布局** |
| **事件系统** | 事件冒泡 | ECS 事件 | **ECS 事件** |
| **动画系统** | 属性动画 | 纹理图集/程序化 | **属性动画 + 程序化** |
| **3D 支持** | 无 | 完整 3D | **2.5D (简化 3D)** |

### 8.2 最小单位：Node

FHRE 的最小单位是 **Node**，采用 ECS 架构，扁平化设计：

```rust
/// Node 组件 - FHRE 的最小单位
/// 
/// 设计原则：
/// 1. **ECS 架构**: 无父子关系，组件扁平存储
/// 2. **SOA 布局**: 数据连续存储，提升缓存命中率
/// 3. **批量处理**: 支持 SIMD 和 GPU 批量处理
/// 4. **统一抽象**: 游戏实体和 UI 控件使用相同的基础组件
#[derive(Component, Debug, Clone)]
pub struct Node {
    /// 节点类型
    pub node_type: NodeType,
    /// 节点状态
    pub state: NodeState,
    /// 节点标志
    pub flags: NodeFlags,
    /// 节点层级（用于渲染排序）
    pub z_order: i32,
    /// 透明度 (0.0 - 1.0)
    pub opacity: f32,
}
```

### 8.3 控件类型层次

```
Control (基础控件)
├── Container (容器)
│   ├── Panel (面板)
│   ├── Window (窗口)
│   └── ScrollView (滚动视图)
├── Input (输入控件)
│   ├── Button (按钮)
│   ├── TextInput (文本输入)
│   ├── Slider (滑块)
│   ├── Switch (开关)
│   └── Dropdown (下拉框)
├── Display (显示控件)
│   ├── Label (标签)
│   ├── Image (图像)
│   ├── ProgressBar (进度条)
│   └── Chart (图表)
├── 2.5D Controls (2.5D 控件)
│   ├── Card (卡片 - 支持翻转)
│   ├── IsoBlock (等角块)
│   └── FlipView (翻转视图)
└── 3D Controls (3D 控件)
    ├── Model3d (3D 模型)
    ├── ParticleWidget (粒子控件)
    └── View3d (3D 视图容器)
```

### 8.4 控件创建示例

```rust
// 纯 2D 控件（默认）
commands.spawn((
    Node::ui_control(NodeType::Button),
    Transform3D::from_position(100.0, 200.0, 0.0),  // z=0 在 UI 平面
    Style {
        width: Dimension::Pixel(120.0),
        height: Dimension::Pixel(40.0),
        background_color: Color::BLUE,
        ..default()
    },
));

// 2.5D 控件（卡片翻转效果）
commands.spawn((
    Node::ui_control(NodeType::Card),
    Transform3D::from_position(200.0, 200.0, 10.0)  // z=10 浮起
        .with_rotation(0.0, 0.2, 0.0),  // Y轴旋转
    Style {
        width: Dimension::Pixel(150.0),
        height: Dimension::Pixel(200.0),
        background_color: Color::WHITE,
        shadow_color: Color::BLACK.with_alpha(0.3),
        shadow_offset: Vec2::new(0.0, 4.0),
        shadow_blur: 8.0,
        ..default()
    },
    AnimationPlayer::default(),
));
```

---

## 8. NuttX SIM 平台支持

### 8.1 架构对比 (FHRE vs LVGL)

```
FHRE v2.0                              LVGL (lv_nuttx_fbdev)
─────────                              ─────────────────────

┌─────────────────┐                    ┌─────────────────┐
│   App::update() │                    │ lv_timer_handler│
│                 │                    │                 │
│ 1. Update Main  │                    │ 1. Update UI    │
│    World        │                    │                 │
│ 2. Extract      │                    │ 2. flush_cb()   │
│ 3. Render to FB │                    │ 3. Render to FB │
│ 4. SimDisplay   │                    │ 4. FBIO_UPDATE  │
│    .present()   │                    │    ioctl        │
└────────┬────────┘                    └────────┬────────┘
         │                                       │
         ▼                                       ▼
┌─────────────────┐                    ┌─────────────────┐
│  SimDisplay     │                    │  lv_nuttx_fb_t  │
│                 │                    │                 │
│ - fd: /dev/fb0  │                    │ - fd: /dev/fb0  │
│ - ioctl(UPDATE) │                    │ - ioctl(UPDATE) │
│ - backbuffer    │                    │ - backbuffer    │
└────────┬────────┘                    └────────┬────────┘
         │                                       │
         ▼                                       ▼
┌─────────────────┐                    ┌─────────────────┐
│  NuttX FB Driver│                    │  NuttX FB Driver│
│  (/dev/fb0)     │                    │  (/dev/fb0)     │
└────────┬────────┘                    └────────┬────────┘
         │                                       │
         ▼                                       ▼
┌─────────────────┐                    ┌─────────────────┐
│  X11 Window     │                    │  X11 Window     │
│  (sim_x11fb)    │                    │  (sim_x11fb)    │
└─────────────────┘                    └─────────────────┘
```

### 8.2 SIM 平台显示驱动

```rust
/// Sim platform display driver
pub struct SimDisplay {
    fd: i32,                    // /dev/fb0 file descriptor
    width: u32,
    height: u32,
    bpp: u8,
    stride: u32,
    framebuffer: *mut u32,      // Hardware framebuffer (mmap)
    backbuffer: Vec<u32>,       // Software backbuffer
}

impl SimDisplay {
    /// Create a new sim display
    pub fn new() -> Option<Self>
    
    /// Present render world to display
    pub fn present(&mut self, render_world: &RenderWorld)
    
    /// Flush framebuffer to display
    fn flush(&mut self)
}
```

### 8.3 编译和运行

```bash
cd /home/uan/develop/FeatherOS-code/FeatherOS/nuttx
make distclean
tools/configure.sh sim:fhre
make -j

./nuttx
nsh> fhre_rust_demo
```

---

## 9. 使用示例

### 9.1 纯 UI 应用

```rust
use fhre::{
    App,
    node::{Node, NodeType, Transform3D, Style},
    math::{Vec2, Color},
    resources::PrimaryScreen,
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

### 9.2 游戏应用

```rust
use fhre::{
    App,
    node::{Node, NodeType, Transform3D},
    math::{Vec3, Color},
};

fn main() {
    let mut app = App::new();
    
    // 创建游戏摄像机
    let game_cam = app.create_game_camera(
        Vec3::new(0.0, 5.0, -10.0),  // 位置
        Vec3::new(0.0, 0.0, 0.0),    // 看向原点
        60.0                          // FOV
    );
    
    // 添加游戏实体
    app.add_startup_system(setup_game);
    
    app.run();
}

fn setup_game(mut commands: Commands) {
    // 创建玩家
    commands.spawn((
        Node::game_entity(NodeType::Sprite2D),
        Transform3D::from_position(0.0, 0.0, 0.0),
    ));
}
```

---

## 10. 总结

FHRE v2.0 是一个完整的嵌入式渲染引擎，具有以下特点：

1. **双世界架构** - Main World 处理逻辑，Render World 处理渲染
2. **默认资源系统** - PrimaryScreen + Default UI Camera，开箱即用
3. **ECS 设计** - Node 作为最小单位，SOA 布局，缓存友好
4. **摄像机系统** - 支持 FPS、TPS、轨道视角等多种控制方式
5. **渲染管线** - 阶段化渲染，支持批处理
6. **NuttX SIM 支持** - 类似 LVGL 的 framebuffer 对接机制

**设计原则：**
- 开箱即用 - `App::new()` 后立即可渲染
- 可扩展 - 可以添加更多摄像机和资源
- 不强制 - 可以禁用默认资源，完全自定义
- 一致性 - 所有 UI 使用相同的坐标系统

---

**文档版本**: v2.0  
**日期**: 2026-04-15  
**作者**: FeatherOS Team
