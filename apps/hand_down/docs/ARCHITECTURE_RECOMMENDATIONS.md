# FHRE 推荐做法实现指南

本文档详细说明 ARCHITECTURE.md 中推荐的五种做法如何在 FHRE 中实现。

## 目录

1. [平台适配：实现 Window trait](#1-平台适配实现-window-trait)
2. [输入桥接：实现 InputBridge trait](#2-输入桥接实现-inputbridge-trait)
3. [提取器：复制模板，修改组件类型](#3-提取器复制模板修改组件类型)
4. [组件：实现 AnimationReceiver 映射属性](#4-组件实现-animationreceiver-映射属性)
5. [游戏逻辑：使用 FHRE 核心函数组装](#5-游戏逻辑使用-fhre-核心函数组装)

---

## 1. 平台适配：实现 Window trait

### 概述

`Window` trait 是 FHRE 与平台交互的核心接口。平台层必须实现此 trait 来：
- 收集原始输入事件（触摸、鼠标、键盘）
- 输出渲染结果到屏幕
- 提供窗口尺寸

### Window trait 定义

```rust
// apps/fhre/rust/src/window/mod.rs
pub trait Window {
    /// 检查窗口是否仍在运行
    fn is_running(&self) -> bool;

    /// 从窗口收集输入事件
    fn collect_input_events(&mut self) -> WindowInputEvents;

    /// 将帧缓冲呈现到窗口
    fn present(&mut self, framebuffer: &[u32]);

    /// 获取窗口尺寸
    fn dimensions(&self) -> (u32, u32);
}
```

### 实现示例（NuttX 平台）

```rust
// apps/examples/fhre/rust/src/platform/framebuffer.rs

use fhre::window::{Window as WindowTrait, WindowInputEvents, 
                   MouseButtonEvent, MouseMotionEvent, KeyboardEvent};

pub struct Window {
    width: u32,
    height: u32,
    fb_fd: c_int,           // 帧缓冲文件描述符
    fb_ptr: *mut u32,       // 帧缓冲内存指针
    input_fd: c_int,        // 输入设备文件描述符
    kbd_fd: c_int,          // 键盘设备文件描述符
    touch_pressed: bool,    // 触摸状态跟踪
    last_touch_x: i32,      // 上次触摸 X 坐标
    last_touch_y: i32,      // 上次触摸 Y 坐标
}

impl Window {
    pub fn new() -> Option<Self> {
        unsafe {
            // 打开帧缓冲设备
            let fb_fd = open(b"/dev/fb0\0".as_ptr(), O_RDWR);
            
            // 通过 ioctl 获取视频信息
            let mut vinfo: VideoInfo = core::mem::zeroed();
            ioctl(fb_fd, FBIOGET_VIDEOINFO, &mut vinfo);
            
            // 获取帧缓冲内存指针
            let mut pinfo: PlaneInfo = core::mem::zeroed();
            ioctl(fb_fd, FBIOGET_PLANEINFO, &mut pinfo);
            
            // 打开输入设备（非阻塞模式）
            let input_fd = open(b"/dev/input0\0".as_ptr(), O_RDWR | O_NONBLOCK);
            let kbd_fd = open(b"/dev/kbd\0".as_ptr(), O_RDWR | O_NONBLOCK);
            
            Some(Self { /* ... */ })
        }
    }
}

impl WindowTrait for Window {
    fn is_running(&self) -> bool {
        true  // NuttX 模拟器始终运行
    }

    fn collect_input_events(&mut self) -> WindowInputEvents {
        let mut events = WindowInputEvents::default();
        self.read_touch_events(&mut events);   // 从 /dev/input0 读取
        self.read_keyboard_events(&mut events); // 从 /dev/kbd 读取
        events
    }

    fn present(&mut self, framebuffer: &[u32]) {
        // 直接复制到帧缓冲内存
        unsafe {
            core::ptr::copy_nonoverlapping(
                framebuffer.as_ptr(),
                self.fb_ptr,
                (self.width * self.height) as usize,
            );
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
```

### 关键点

1. **设备路径**：
   - `/dev/fb0` - 帧缓冲设备
   - `/dev/input0` - 触摸/鼠标输入
   - `/dev/kbd` - 键盘输入

2. **非阻塞模式**：输入设备必须使用 `O_NONBLOCK` 打开，避免阻塞渲染循环

3. **事件转换**：将平台原始事件转换为 `WindowInputEvents`

---

## 2. 输入桥接：实现 InputBridge trait

### 概述

`InputBridge` 将平台特定的按键/按钮码映射到 FHRE 标准类型。不同平台有不同的键码定义，需要适配。

### InputBridge trait 定义

```rust
// apps/examples/fhre/rust/src/platform/runner.rs
pub trait InputBridge {
    /// 将平台键码映射到 KeyCode
    fn map_keycode(&self, platform_keycode: u32) -> Option<KeyCode>;
    
    /// 将平台鼠标按钮映射到 MouseButton
    fn map_mouse_button(&self, platform_button: u32) -> Option<MouseButton>;
}
```

### 实现示例（X11 键码映射）

```rust
// apps/examples/fhre/rust/src/platform/framebuffer.rs

// X11 键码常量
const X11_KEY_SPACE: u32 = 0x0020;
const X11_KEY_R: u32 = 0x0072;
const X11_KEY_ESCAPE: u32 = 0xff1b;

pub struct InputAdapter;

impl InputBridge for InputAdapter {
    fn map_keycode(&self, code: u32) -> Option<KeyCode> {
        match code {
            X11_KEY_SPACE  => Some(KeyCode::Space),
            X11_KEY_R      => Some(KeyCode::KeyR),
            X11_KEY_ESCAPE => Some(KeyCode::Escape),
            _ => None,
        }
    }
    
    fn map_mouse_button(&self, btn: u32) -> Option<MouseButton> {
        match btn {
            1 => Some(MouseButton::Left),
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::Right),
            _ => None,
        }
    }
}
```

### 使用 PlatformInputPlugin

```rust
// apps/examples/fhre/rust/src/lib.rs

// 创建输入插件
let input_plugin = PlatformInputPlugin::new(framebuffer::InputAdapter);

// 在主循环中使用
app.run(&mut window, &input_plugin, 16);  // 16ms 帧延迟
```

### PlatformInputPlugin 内部实现

```rust
// apps/examples/fhre/rust/src/platform/runner.rs

pub struct PlatformInputPlugin<B: InputBridge> {
    bridge: B,
}

impl<B: InputBridge + 'static> InputPlugin for PlatformInputPlugin<B> {
    fn bridge(&self, app: &mut App, events: &WindowInputEvents) {
        // 桥接键盘输入
        self.bridge_keyboard_input(app, &events.keyboard_events);
        // 桥接鼠标按钮
        self.bridge_mouse_input(app, &events.mouse_button_events);
        // 桥接鼠标位置
        self.bridge_mouse_position(app, &events.mouse_button_events, &events.mouse_motion_events);
    }
}

impl<B: InputBridge> PlatformInputPlugin<B> {
    fn bridge_keyboard_input(&self, app: &mut App, events: &[KeyboardEvent]) {
        if let Some(key_input) = app.main_world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
            key_input.clear();
            for event in events {
                if let Some(kc) = self.bridge.map_keycode(event.keycode) {
                    if event.pressed {
                        key_input.press(kc);
                    } else {
                        key_input.release(kc);
                    }
                }
            }
        }
    }
}
```

### 数据流

```
┌─────────────────────────────────────────────────────────────┐
│  Platform Layer                                             │
│  /dev/input0 → TouchSample → MouseButtonEvent              │
│  /dev/kbd    → KeyboardEventNuttX → KeyboardEvent          │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼ WindowInputEvents
┌─────────────────────────────────────────────────────────────┐
│  InputBridge                                                │
│  map_keycode(0x0020) → KeyCode::Space                       │
│  map_mouse_button(1) → MouseButton::Left                    │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼ ECS Resources
┌─────────────────────────────────────────────────────────────┐
│  ButtonInput<KeyCode>     → press(KeyCode::Space)          │
│  ButtonInput<MouseButton> → press(MouseButton::Left)       │
│  MousePosition            → { x, y }                        │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. 提取器：复制模板，修改组件类型

### 概述

提取器（Extractor）负责将数据从 Main World 复制到 Render World。这是双世界架构的核心机制。

### 提取器模板

```rust
// apps/examples/fhre/rust/src/extract.rs

/// 提取视图（Camera → View）
pub fn extract_view(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let screen = main_world.resources().get::<PrimaryScreen>();
    let camera = main_world.resources().get::<Camera>();
    
    // 构建 ViewBundle
    let view_bundle = camera_to_view_bundle(&camera, canvas_pos, width, height);
    
    // 添加到 Render World
    let view_idx = render_world.add_view(view_bundle);
    render_world.set_current_view(Some(view_idx));
}

/// 提取 3D 组件（Cube, SoccerBall → ExtractedMesh）
pub fn extract_3d_components(main_world: &MainWorld, render_world: &mut RenderWorld) {
    for (entity, transform) in main_world.query::<Transform>() {
        // 提取 Cube
        if let Some(cube) = main_world.get_component::<Cube>(entity) {
            let render_entity = render_world.get_or_spawn_synced(entity);
            
            let mesh = ExtractedMesh {
                vertices: cube.get_vertices().to_vec(),
                faces: cube.get_faces().iter().map(|f| f.to_vec()).collect(),
                face_colors: cube.face_colors.to_vec(),
                face_textures: cube.face_textures.to_vec(),
                position: transform.position,
                rotation: cube.rotation,
                wireframe: cube.wireframe,
                wireframe_color: cube.wireframe_color,
            };
            
            render_world.insert_component(render_entity, mesh);
            continue;
        }
        
        // 提取 SoccerBall（类似模式）
        if let Some(soccer_ball) = main_world.get_component::<SoccerBall>(entity) {
            // ... 同样的模式
        }
    }
}

/// 提取 UI 组件（Button → ExtractedUI）
pub fn extract_buttons(main_world: &MainWorld, render_world: &mut RenderWorld) {
    for (entity, transform) in main_world.query::<Transform>() {
        if let Some(button) = main_world.get_component::<Button>(entity) {
            let render_entity = render_world.get_or_spawn_synced(entity);
            
            let ui = ExtractedUI {
                position: Vec2::new(transform.position.x, transform.position.y),
                width: button.width,
                height: button.height,
                color: button.current_color(),
            };
            
            render_world.insert_component(render_entity, ui);
        }
    }
}
```

### 队列阶段（生成渲染命令）

```rust
/// 队列网格渲染命令
pub fn queue_meshes(_main_world: &MainWorld, render_world: &mut RenderWorld) {
    let view = render_world.current_view()?.view.clone();
    let meshes: Vec<ExtractedMesh> = render_world.query::<ExtractedMesh>().collect();
    
    for mesh in &meshes {
        // 生成 PhaseItem（自动按深度排序）
        let phase_items = generate_mesh_phase_items(mesh, &view, gpu_textures);
        
        for (phase_type, item) in phase_items {
            render_world.add_phase_item(phase_type, item);
        }
    }
}

/// 队列 UI 渲染命令
pub fn queue_ui(_main_world: &MainWorld, render_world: &mut RenderWorld) {
    for (entity, ui) in render_world.query::<ExtractedUI>() {
        let rect = Rect::from_center_size(ui.position, Vec2::new(ui.width, ui.height));
        let command = RenderCommand::DrawRect { rect, color: ui.color };
        
        // UI 元素使用 Ui 渲染阶段
        let item = PhaseItem::ui(command, entity.id() as i32);
        render_world.add_phase_item(RenderPhaseType::Ui, item);
    }
}
```

### 添加新组件的提取器模板

要为新组件 `MyComponent` 添加提取器：

```rust
/// 1. 定义提取后的数据结构
#[derive(Clone)]
pub struct ExtractedMyComponent {
    pub position: Vec3,
    pub custom_data: f32,
    // ...
}

/// 2. 实现提取函数
pub fn extract_my_components(main_world: &MainWorld, render_world: &mut RenderWorld) {
    for (entity, transform) in main_world.query::<Transform>() {
        if let Some(my_comp) = main_world.get_component::<MyComponent>(entity) {
            let render_entity = render_world.get_or_spawn_synced(entity);
            
            let extracted = ExtractedMyComponent {
                position: transform.position,
                custom_data: my_comp.some_field,
            };
            
            render_world.insert_component(render_entity, extracted);
        }
    }
}

/// 3. 实现队列函数
pub fn queue_my_components(_main_world: &MainWorld, render_world: &mut RenderWorld) {
    for (entity, extracted) in render_world.query::<ExtractedMyComponent>() {
        let command = RenderCommand::DrawRect { /* ... */ };
        let item = PhaseItem::ui(command, entity.id() as i32);
        render_world.add_phase_item(RenderPhaseType::Ui, item);
    }
}

/// 4. 在 main 中注册
app.add_extractor(extract::extract_my_components)
   .add_extractor(extract::queue_my_components);
```

---

## 4. 组件：实现 AnimationReceiver 映射属性

### 概述

`AnimationReceiver` trait 定义了组件如何接收动画值。动画系统采样 `AnimationClip` 后，通过此 trait 将值应用到组件属性。

### AnimationReceiver trait 定义

```rust
// apps/fhre/rust/src/animation/property.rs

pub trait AnimationReceiver {
    /// 将动画属性值应用到组件
    fn apply_animation(&mut self, property: AnimationProperty, value: f32);
}
```

### 可动画属性

```rust
pub enum AnimationProperty {
    // 位移
    TranslationX, TranslationY, TranslationZ,
    // 旋转（欧拉角，度）
    RotationX, RotationY, RotationZ,
    // 缩放
    ScaleX, ScaleY, ScaleZ,
    // 颜色
    ColorR, ColorG, ColorB, ColorA,
    // 精灵尺寸
    SpriteWidth, SpriteHeight,
    // 自定义
    Custom(u32),
}
```

### 实现示例（Cube）

```rust
// apps/examples/fhre/rust/src/components/cube.rs

impl AnimationReceiver for Cube {
    fn apply_animation(&mut self, property: AnimationProperty, value: f32) {
        match property {
            // 旋转属性 → Cube.rotation
            AnimationProperty::RotationX => { self.rotation.x = value; }
            AnimationProperty::RotationY => { self.rotation.y = value; }
            AnimationProperty::RotationZ => { self.rotation.z = value; }
            
            // 缩放属性 → Cube.size
            AnimationProperty::ScaleX | 
            AnimationProperty::ScaleY | 
            AnimationProperty::ScaleZ => {
                self.size = value;
            }
            
            // 忽略不支持的属性
            _ => {}
        }
    }
}
```

### 实现示例（SoccerBall）

```rust
// apps/examples/fhre/rust/src/components/soccer_ball.rs

impl AnimationReceiver for SoccerBall {
    fn apply_animation(&mut self, property: AnimationProperty, value: f32) {
        match property {
            AnimationProperty::RotationX => { self.rotation.x = value; }
            AnimationProperty::RotationY => { self.rotation.y = value; }
            AnimationProperty::RotationZ => { self.rotation.z = value; }
            AnimationProperty::ScaleX | 
            AnimationProperty::ScaleY | 
            AnimationProperty::ScaleZ => {
                self.size = value;
            }
            _ => {}
        }
    }
}
```

### 动画系统工作流程

```
┌─────────────────────────────────────────────────────────────┐
│  AnimationClip                                              │
│  - duration: 6.0s                                           │
│  - curves: {                                                │
│      RotationY: [0° → 360°],                                │
│      RotationX: [30° → -30° → 30°],                         │
│      ScaleX: [120 → 160 → 120],                             │
│    }                                                        │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼ AnimationPlayer::tick(dt)
┌─────────────────────────────────────────────────────────────┐
│  采样当前时间的动画值                                        │
│  t = 3.0s → { RotationY: 180°, RotationX: -30°, ... }       │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼ apply_animations::<Cube>
┌─────────────────────────────────────────────────────────────┐
│  Cube::apply_animation(RotationY, 180.0)                    │
│  Cube::apply_animation(RotationX, -30.0)                    │
│  Cube::apply_animation(ScaleX, 160.0)                       │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  Cube { rotation: { x: -30, y: 180, z: 45 }, size: 160 }    │
└─────────────────────────────────────────────────────────────┘
```

### 注册 apply_animations 系统

```rust
// apps/examples/fhre/rust/src/lib.rs

app.add_systems(Update, fhre::declare_system!(
    apply_animations::<Cube>; 
    Query<&AnimationPlayer>, Query<&mut Cube>
));

app.add_systems(Update, fhre::declare_system!(
    apply_animations::<SoccerBall>; 
    Query<&AnimationPlayer>, Query<&mut SoccerBall>
));
```

---

## 5. 游戏逻辑：使用 FHRE 核心函数组装

### 概述

游戏逻辑系统使用 FHRE 提供的核心函数和 ECS 查询来组装功能。遵循 Bevy 风格的系统声明。

### 系统注册模板

```rust
// apps/examples/fhre/rust/src/lib.rs

#[no_mangle]
pub extern "C" fn fhre_rust_main() -> i32 {
    let mut app = App::new(width, height);
    
    app.add_plugins(DefaultPlugins)                    // 核心插件
        .add_plugin(TextureAssetPlugin)                // 纹理插件
        .insert_resource(DemoState::new())             // 自定义资源
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(MousePosition::default())
        
        // Startup 系统（仅首帧执行）
        .add_systems(Startup, declare_system!(setup_textures; ...))
        .add_systems(Startup, declare_system!(setup; ...))
        
        // PreUpdate 系统（在 Update 之前）
        .add_systems(PreUpdate, declare_system!(picking_system; ...))
        .add_systems(PreUpdate, declare_system!(button_interaction_system; ...))
        .add_systems(PreUpdate, declare_system!(input_system; ...))
        
        // Update 系统（主逻辑）
        .add_systems(Update, declare_system!(setup_animation; ...))
        .add_systems(Update, declare_system!(animation_control_system; ...))
        .add_systems(Update, declare_system!(model_switch_system; ...))
        .add_systems(Update, declare_system!(apply_animations::<Cube>; ...))
        .add_systems(Update, declare_system!(apply_animations::<SoccerBall>; ...));
    
    // 提取器
    app.add_extractor(extract::extract_view)
       .add_extractor(extract::extract_3d_components)
       .add_extractor(extract::extract_buttons)
       .add_extractor(extract::queue_meshes)
       .add_extractor(extract::queue_ui);
    
    // 运行主循环
    let input_plugin = PlatformInputPlugin::new(framebuffer::InputAdapter);
    app.run(&mut window, &input_plugin, 16);
    
    0
}
```

### 系统执行顺序

```
帧执行顺序 (App::update_and_render):

1. initialize_plugins()         # 仅首帧
2. run_startup_systems()        # 仅首帧
   ├── setup_textures           # 创建纹理资源
   └── setup                    # 创建初始实体
3. Update Time resource
4. run_systems():
   ├── PreUpdate:
   │   ├── picking_system       # 命中检测
   │   ├── button_interaction   # 按钮状态更新
   │   └── input_system         # 输入处理
   ├── Update:
   │   ├── setup_animation      # 初始化动画
   │   ├── animation_control    # 播放/暂停
   │   ├── model_switch         # 模型切换
   │   └── apply_animations     # 应用动画值
   └── PostUpdate
5. entity_sync_system()         # 同步实体到渲染世界
6. extractors.run()             # 提取阶段
   ├── extract_view
   ├── extract_3d_components
   ├── extract_buttons
   ├── queue_meshes
   └── queue_ui
7. execute_render()             # 执行渲染命令
8. Events::update()             # 清理事件缓冲区
```

### 典型系统实现

#### Setup 系统

```rust
fn setup(mut commands: Commands, screen: Res<PrimaryScreen>, cube_textures: Res<CubeTextures>) {
    let (width, height) = screen.dimensions();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    
    // 创建 3D 模型
    commands.spawn()
        .insert(Node::game_entity())
        .insert(Transform3D::from_position(center_x, center_y, 0.0))
        .insert(Cube::new(120.0)
            .with_face_colors(colors::CUBE_FACES)
            .with_wireframe(true, Color::WHITE))
        .insert(AnimationPlayer::new())
        .insert(SyncToRenderWorld);
    
    // 创建按钮
    commands.spawn()
        .insert(Node::ui_control())
        .insert(Transform::from_2d(x, y))
        .insert(Button::new(100.0, 40.0).with_text("Click"))
        .insert(PickableBounds::from_size(100.0, 40.0))
        .insert(Pickable::DEFAULT);  // 重要：使用 DEFAULT
}
```

#### Picking 系统

```rust
fn picking_system(
    mouse_pos: Res<MousePosition>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    transform_query: Query<&Transform>,
    bounds_query: Query<&PickableBounds>,
    pickable_query: Query<&Pickable>,
    mut hover_map: ResMut<HoverMap>,
    mut prev_hover_map: ResMut<PreviousHoverMap>,
    mut events: ResMut<Events>,
) {
    // 收集可点击实体
    let mut pickables = Vec::new();
    for (entity, transform) in transform_query.iter() {
        if let Some((_, bounds)) = bounds_query.get_pair(entity) {
            let pickable = pickable_query.get(entity).copied();
            pickables.push((entity, *transform, *bounds, pickable));
        }
    }
    
    // 执行命中检测
    let hits = ui_picking_backend(&pointers, &pickables);
    
    // 更新悬停状态
    update_hover_map(hits, &pickables, &mut hover_map, &mut prev_hover_map);
    
    // 生成指针事件
    pointer_events(&mut events, &hover_map, &prev_hover_map, ...);
}
```

#### 输入处理系统

```rust
fn input_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DemoState>,
    mut button_query: Query<&mut Button>,
) {
    // 处理按钮点击
    for (_, button) in button_query.iter_mut() {
        if button.clicked {
            button.clicked = false;
            match button.text {
                "Prev" => { /* 切换到上一个模型 */ }
                "Pause" => { state.is_rotating = !state.is_rotating; }
                "Next" => { /* 切换到下一个模型 */ }
                _ => {}
            }
        }
    }
    
    // 处理键盘快捷键
    if key_input.just_pressed(KeyCode::Space) {
        state.is_rotating = !state.is_rotating;
    }
}
```

### 核心函数参考

| 函数 | 用途 | 所在模块 |
|------|------|----------|
| `ui_picking_backend` | UI 命中检测 | `picking::backend` |
| `update_hover_map` | 更新悬停状态 | `picking::hover` |
| `pointer_events` | 生成指针事件 | `picking::events` |
| `apply_animations::<T>` | 应用动画到组件 | `animation` |
| `entity_sync_system` | 同步实体到渲染世界 | `sync` |

---

## 总结

| 推荐做法 | 实现位置 | 关键 trait/函数 |
|----------|----------|----------------|
| 平台适配 | `platform/framebuffer.rs` | `Window` trait |
| 输入桥接 | `platform/runner.rs` | `InputBridge` trait, `PlatformInputPlugin` |
| 提取器 | `extract.rs` | `Extract`, `queue_*` 函数 |
| 组件动画 | `components/*.rs` | `AnimationReceiver` trait |
| 游戏逻辑 | `lib.rs` | `declare_system!`, ECS 查询 |

所有推荐做法已在示例应用中完整实现，可作为新项目的参考模板。
