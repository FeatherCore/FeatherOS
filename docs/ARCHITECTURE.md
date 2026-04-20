# FHRE 架构文档

## 概述

FHRE (Feather Hybrid Render Engine) 是一个轻量级的纯 3D 渲染引擎，设计用于嵌入式系统 (no_std)。采用 Bevy 风格的双世界 ECS 架构，2D 只是 3D 的特例（z=0 平面上的对象）。

**版本**: 2.8.0  
**目标平台**: 嵌入式系统 (NuttX RTOS)  
**设计理念**: 纯 3D 引擎，ECS 架构，双世界同步，事件驱动交互，Asset 系统对齐 Bevy，Query 系统对齐 Bevy QueryData/QueryFilter

## 核心概念

### 纯 3D 引擎

FHRE 是纯 3D 引擎，所有实体都在 3D 空间中：
- 2D 只是 3D 的特例：z=0 平面上的对象
- 通过正交相机投影到屏幕
- 统一使用 `Transform` 作为唯一变换组件
- `Transform3D` 作为别名保持兼容
- Node 不包含类型字段，类型分类由应用层定义

### 事件驱动架构 (Event-Driven Architecture)

FHRE 采用事件驱动架构处理用户交互，对齐 Bevy 的 Picking 系统：

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Pointers   │ →  │   Backend   │ →  │    Hover    │ →  │   Events    │
│ ButtonInput │    │ PointerHits │    │  HoverMap   │    │ Pointer<E>  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

**核心事件类型**：
- `Pointer<Over>` / `Pointer<Out>` - 指针进入/离开实体
- `Pointer<Press>` / `Pointer<Release>` - 按钮按下/释放
- `Pointer<Click>` - 点击完成（在实体上按下并释放）
- `Pointer<Move>` - 指针移动
- `Pointer<DragStart>` / `Pointer<Drag>` / `Pointer<DragEnd>` - 拖拽事件

**交互流程**：
1. `picking_system` 检测鼠标位置与 `PickableBounds` 的碰撞，生成 `PointerHits`
2. `update_hover_map` 更新 `HoverMap` 和 `PreviousHoverMap`
3. `pointer_events` 根据 HoverMap 变化和按钮状态生成 `Pointer<E>` 事件
4. `button_interaction_system` 监听 `Pointer<Click>` 等事件更新 `Button.state` 和 `button.clicked`
5. `input_system` 检查 `button.clicked` 执行操作，**处理完后立即重置 `button.clicked = false`**
6. 帧结束时 `Events::update()` 清理事件缓冲区

**关键组件**：
- `PickableBounds`: 可点击区域 (width, height)
- `Pickable`: 是否可悬停、是否阻挡下层（必须使用 `Pickable::DEFAULT`，而非 `default()`）
- `HoverMap` / `PreviousHoverMap`: 当前帧/上一帧悬停的实体
- `PointerPress` / `PointerLocation`: 指针状态资源

## 目录结构

```
apps/fhre/rust/src/                   # FHRE 核心库
├── lib.rs                            # 库入口，全局分配器，panic handler
├── app/
│   ├── mod.rs
│   └── app.rs                        # App 结构，插件注册，update_and_render
├── main_world/                       # 主世界 (游戏逻辑)
│   ├── mod.rs
│   ├── world.rs                      # MainWorld ECS
│   ├── entity.rs                     # Entity ID
│   ├── component.rs                  # Component trait
│   ├── system.rs                     # System trait, declare_system! 宏
│   ├── system_param.rs               # Res, ResMut, Local, SystemParam trait
│   ├── commands.rs                   # Commands, EntityCommands
│   ├── change_detection.rs           # 变更检测
│   ├── tuples.rs                     # all_tuples! 宏
│   ├── query_data.rs                 # QueryData trait (多组件查询)
│   ├── query_filter.rs               # QueryFilter trait (With, Without, Added, Changed)
│   └── filtered_query.rs             # Query<D, F> 统一查询类型
├── render_world/                     # 渲染世界 (渲染数据)
│   ├── mod.rs
│   ├── world.rs                      # RenderWorld ECS
│   ├── command.rs                    # RenderCommand, DrawCall, Vertex
│   ├── phase.rs                      # RenderPhase, PhaseItem
│   ├── view.rs                       # View, ViewTarget, ClearConfig
│   ├── extracted.rs                  # ExtractedMesh, ExtractedView, ExtractedUI
│   └── object.rs                     # RenderObject
├── pipeline/                         # 渲染管线
│   ├── mod.rs
│   ├── renderer.rs                   # Renderer trait
│   ├── batch.rs                      # GpuTaskCollector, HybridScheduler, RenderBatch
│   ├── texture.rs                    # Texture, TextureFormat, Sampler
│   ├── gradient.rs                   # Gradient, GradientStop
│   ├── software/                     # CPU 软件渲染后端
│   │   ├── mod.rs
│   │   └── backend.rs                # SoftwareBackend
│   └── gpu/                          # GPU 渲染后端 (占位)
│       └── mod.rs
├── asset/                            # 资产系统 (对齐 Bevy)
│   ├── mod.rs                        # Asset trait
│   ├── id.rs                         # AssetId, AssetIndex
│   ├── handle.rs                     # Handle<A>, StrongHandle
│   ├── assets.rs                     # Assets<A> 集合
│   ├── event.rs                      # AssetEvent<A>
│   ├── server.rs                     # AssetServer, AssetRegistry
│   ├── render_asset.rs               # RenderAsset trait, RenderAssets<A>
│   └── extract_plugin.rs             # RenderAssetPlugin, ExtractResourcePlugin
├── sync/                             # 双世界同步
│   ├── mod.rs
│   ├── sync_markers.rs               # SyncToRenderWorld, RenderEntity, MainEntity
│   ├── pending_sync.rs               # PendingSyncEntity
│   └── sync_system.rs                # entity_sync_system
├── extract/                          # 提取阶段
│   └── mod.rs                        # ExtractComponent, Extractors, Extract<P>
├── schedule/                         # 调度系统
│   ├── mod.rs
│   ├── schedule.rs                   # Schedule
│   ├── label.rs                      # Startup, Update, PreUpdate, PostUpdate, Last
│   ├── set.rs                        # SystemSet
│   └── condition.rs                  # RunCondition
├── plugin/                           # 插件系统
│   ├── mod.rs
│   ├── plugin.rs                     # Plugin trait
│   ├── plugin_group.rs               # PluginGroup, PluginGroupBuilder
│   ├── default_plugins.rs            # DefaultPlugins
│   └── sync_component_plugin.rs      # SyncComponentPlugin
├── node/                             # 节点系统
│   ├── mod.rs
│   ├── node.rs                       # Node, NodeState, NodeFlags
│   └── node3d.rs                     # Transform, Transform3D, Camera, Light
├── animation/                        # 动画系统
│   ├── mod.rs
│   ├── clip.rs                       # AnimationClip
│   ├── curve.rs                      # KeyframeCurve, Animatable
│   ├── easing.rs                     # Easing 函数
│   ├── player.rs                     # AnimationPlayer, ActiveAnimation
│   ├── property.rs                   # AnimationProperty, AnimationReceiver
│   ├── graph.rs                      # AnimationGraph
│   └── transition.rs                 # AnimationTransitions
├── math/                             # 数学库
│   ├── mod.rs
│   ├── vec2.rs                       # Vec2
│   ├── vec3.rs                       # Vec3, Vec4
│   ├── mat4.rs                       # Mat4
│   ├── rect.rs                       # Rect
│   └── color.rs                      # Color, BlendMode
├── resources/                        # 资源系统
│   ├── mod.rs
│   ├── resources.rs                  # Resource trait, Resources 存储
│   ├── time.rs                       # Time, Timer
│   ├── camera.rs                     # Camera, ProjectionType
│   ├── screen.rs                     # PrimaryScreen
│   └── config.rs                     # RenderConfig, WindowConfig
├── event/                            # 事件系统
│   ├── mod.rs
│   ├── events.rs                     # Events
│   ├── event_reader.rs               # EventReader
│   ├── event_writer.rs               # EventWriter
│   └── system_param.rs               # Event system params
├── camera/                           # 相机插件
│   └── mod.rs                        # CameraPlugin
├── picking/                          # Picking 系统
│   ├── mod.rs                        # 模块导出
│   ├── pickable.rs                   # Pickable 组件
│   ├── bounds.rs                     # PickableBounds 组件
│   ├── hover.rs                      # HoverMap, PreviousHoverMap
│   ├── backend.rs                    # PointerHits, HitData
│   ├── pointer.rs                    # PointerId, PointerButton, PointerPress, PointerLocation, PointerAction, PointerInput
│   ├── events.rs                     # Pointer<E>, Over, Out, Enter, Leave, Press, Release, Click, Move, Drag*
│   ├── system.rs                     # update_hover_map, ui_picking_backend, pointer_events
│   └── plugin.rs                     # PickingPlugin, PointerHitsBuffer
└── window/                           # 窗口抽象 (平台无关)
    └── mod.rs                        # Window trait, MousePosition, 原始事件类型

apps/examples/fhre/rust/src/          # 示例应用
├── lib.rs                            # 主入口，系统注册
├── extract.rs                        # 自定义提取器
├── components/                       # 应用层组件
│   ├── mod.rs
│   ├── button.rs                     # Button 组件 (含交互状态)
│   ├── cube.rs                       # Cube 3D 模型
│   └── soccer_ball.rs                # SoccerBall 3D 模型
└── platform/                         # 平台层实现
    ├── mod.rs
    ├── framebuffer.rs                # Window 实现 (NuttX/X11)
    ├── runner.rs                     # InputBridge, WindowRunner
    └── input/                        # 平台输入类型
        ├── mod.rs
        ├── button_input.rs           # ButtonInput<T>
        ├── keyboard.rs               # KeyCode, Key
        └── mouse.rs                  # MouseButton
```

## 示例应用结构 (apps/examples/fhre)

### 目录结构

```
apps/examples/fhre/rust/src/
├── lib.rs                    # 主入口
│   ├── App 初始化
│   ├── 资源注册 (DemoState, ButtonInput, MousePosition)
│   ├── 系统注册 (Startup, PreUpdate, Update)
│   └── 提取器注册 (extract_view, extract_3d_components, etc.)
│
├── extract.rs                # 自定义提取器
│   ├── extract_view()        # 提取相机视图
│   ├── extract_3d_components() # 提取 Cube, SoccerBall
│   ├── extract_buttons()     # 提取 Button 组件
│   ├── queue_meshes()        # 生成 3D 网格渲染命令
│   └── queue_ui()            # 生成 UI 渲染命令
│
├── components/               # 应用层组件
│   ├── button.rs             # Button 组件
│   │   ├── 外观: width, height, colors, text
│   │   ├── 状态: Normal, Hover, Pressed
│   │   └── 交互: clicked 标志
│   ├── cube.rs               # Cube 3D 模型
│   │   ├── size, color
│   │   └── face_textures (可选)
│   └── soccer_ball.rs        # SoccerBall 3D 模型
│       └── radius, color
│
└── platform/                 # 平台层实现
    ├── framebuffer.rs        # Window trait 实现
    │   ├── NuttX: /dev/fb0
    │   └── X11: Xlib (开发测试)
    ├── runner.rs             # WindowRunner
    │   ├── main loop
    │   ├── input bridging
    │   └── frame presentation
    └── input/                # 输入类型
        ├── button_input.rs   # ButtonInput<T>
        ├── keyboard.rs       # KeyCode
        └── mouse.rs          # MouseButton
```

### 系统流程

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frame Lifecycle                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Startup (仅首帧)                                             │
│     └── setup(): 创建 Cube, Button, Camera 实体                  │
│                                                                  │
│  2. PreUpdate (每帧)                                             │
│     ├── picking_system(): 更新 HoverMap                         │
│     ├── button_interaction_system(): 更新 Button 状态            │
│     └── input_system(): 处理按钮点击                             │
│                                                                  │
│  3. Update (每帧)                                                │
│     ├── update(): 旋转 Cube                                      │
│     └── animation_control_system(): 动画控制                     │
│                                                                  │
│  4. PostUpdate (每帧)                                            │
│     └── apply_animations(): 应用动画到 Transform                 │
│                                                                  │
│  5. Extract (每帧)                                               │
│     ├── entity_sync_system(): 同步实体                           │
│     ├── extract_view(): 提取相机                                 │
│     ├── extract_3d_components(): 提取 3D 组件                    │
│     ├── extract_buttons(): 提取 Button                           │
│     ├── queue_meshes(): 生成网格渲染命令                         │
│     └── queue_ui(): 生成 UI 渲染命令                             │
│                                                                  │
│  6. Render (每帧)                                                │
│     └── SoftwareBackend::execute(): CPU 软件渲染                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 关键系统实现

#### 1. setup() - Startup 系统

```rust
fn setup(mut commands: Commands, screen: Res<PrimaryScreen>) {
    // 创建 3D 模型
    commands.spawn()
        .insert(Node::model_3d())
        .insert(Transform::from_position(Vec3::new(0.0, 0.0, -3.0)))
        .insert(Cube::new(1.0).with_color(Color::RED))
        .insert(SyncToRenderWorld);
    
    // 创建按钮
    commands.spawn()
        .insert(Node::ui_control())
        .insert(Transform::from_2d(100.0, 50.0))
        .insert(Button::new(80.0, 30.0).with_text("Click"))
        .insert(PickableBounds::from_size(80.0, 30.0))
        .insert(Pickable::DEFAULT);
    
    // 创建相机
    commands.spawn()
        .insert(Node::camera())
        .insert(Transform::from_position(Vec3::new(0.0, 0.0, 5.0)))
        .insert(Camera::default());
}
```

#### 2. input_system() - 处理按钮点击

```rust
fn input_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DemoState>,
    button_query: Query<&Button>,
) {
    for (entity, button) in button_query.iter() {
        if button.clicked {
            match button.text.as_str() {
                "Prev" => state.prev_model(),
                "Pause" => state.toggle_pause(),
                "Next" => state.next_model(),
                _ => {}
            }
        }
    }
    
    // 键盘快捷键
    if key_input.just_pressed(KeyCode::Space) {
        state.toggle_pause();
    }
}
```

#### 3. extract_3d_components() - 提取 3D 组件

```rust
fn extract_3d_components(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    // 提取 Cube
    for (entity, cube) in main_world.query::<Cube>() {
        let transform = main_world.get_component::<Transform>(entity);
        if let Some(transform) = transform {
            let render_entity = render_world.get_or_spawn_synced(entity);
            render_world.insert_component(render_entity, ExtractedMesh::from_cube(&cube, &transform));
        }
    }
    
    // 提取 SoccerBall
    for (entity, ball) in main_world.query::<SoccerBall>() {
        // ...
    }
}
```

### 平台层实现

#### Window trait

```rust
pub trait Window {
    /// 收集原始输入事件
    fn collect_input_events(&mut self) -> WindowInputEvents;
    
    /// 获取帧缓冲区
    fn framebuffer(&mut self) -> &mut [u32];
    
    /// 呈现帧
    fn present(&mut self);
}
```

#### WindowRunner

```rust
pub struct WindowRunner<'a> {
    app: &'a mut App,
    window: &'a mut dyn Window,
}

impl<'a> WindowRunner<'a> {
    pub fn run(&mut self) {
        loop {
            // 1. 收集输入事件
            let events = self.window.collect_input_events();
            self.bridge_input(events);
            
            // 2. 运行应用逻辑
            self.app.update_and_render();
            
            // 3. 呈现帧
            self.window.present();
        }
    }
    
    fn bridge_input(&mut self, events: WindowInputEvents) {
        // 转换为 ECS 资源
        // ButtonInput<KeyCode>, ButtonInput<MouseButton>, MousePosition
    }
}
```

## 核心架构

### 1. 双世界架构 (Dual World Architecture)

```
┌─────────────────────────────────────────────────────────────────────┐
│                           App                                       │
│  ┌─────────────────────────┐    ┌─────────────────────────┐        │
│  │      Main World         │    │     Render World        │        │
│  │  ─────────────────      │    │  ─────────────────      │        │
│  │  • Entity + Component   │    │  • Entity + Component   │        │
│  │  • Transform            │    │  • MainEntity           │        │
│  │  • Cube, SoccerBall     │    │  • ExtractedMesh        │        │
│  │  • AnimationPlayer      │    │  • ExtractedUI          │        │
│  │  • Button (含交互状态)   │    │  • RenderCommand        │        │
│  │  • Game Logic Systems   │    │                         │        │
│  └─────────────────────────┘    └─────────────────────────┘        │
│              │                              ▲                      │
│              │ entity_sync_system           │                      │
│              │ Extractors                   │                      │
│              └──────────────────────────────┘                      │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │                    SoftwareBackend                           │  │
│  │              CPU 软件渲染 (fill_rect, fill_polygon, etc.)   │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### 2. 帧生命周期

```
帧执行顺序 (App::update_and_render):

1. initialize_plugins()     # 仅首帧，初始化插件
2. run_startup_systems()    # 仅首帧，执行 Startup 系统
3. Update Time resource     # 更新时间
4. run_systems()            # 执行主世界系统
   ├── PreUpdate            # picking_system → pointer_events → button_interaction_system
   ├── apply_commands()     # 应用命令
   ├── Update               # 游戏逻辑、动画
   ├── apply_commands()     # 应用命令
   ├── PostUpdate           # 后处理
   └── apply_commands()     # 应用命令
5. entity_sync_system()     # 同步实体到渲染世界
6. extractors.run()         # 提取组件到渲染世界
7. execute_render()         # 执行渲染
8. clear_commands()         # 清理命令
```

### 3. 实体同步机制

```rust
// Main World 标记
#[derive(Component)]
struct SyncToRenderWorld;  // 标记需要同步到渲染世界

// Main World 组件
#[derive(Component)]
struct RenderEntity(Entity);  // 存储对应的渲染世界实体 ID

// Render World 组件
#[derive(Component)]
struct MainEntity(Entity);  // 存储对应的主世界实体 ID

// 同步流程
Main World: Entity A + SyncToRenderWorld
    │
    │ entity_sync_system (检查 PendingSyncEntity)
    ▼
Render World: Entity B + MainEntity(A)
    │
    │ extract_xxx (自定义提取器)
    ▼
Render World: Entity B + ExtractedMesh / ExtractedUI
    │
    │ queue_xxx (生成命令)
    ▼
Render World: RenderCommand 列表
```

### 4. 数据流详解

#### 4.1 完整帧流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              MAIN LOOP                                       │
│  (WindowRunner::run)                                                         │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  1. INPUT BRIDGING                                                          │
│     WindowRunner::bridge_keyboard_input()                                   │
│     WindowRunner::bridge_mouse_input()                                      │
│     → Updates ButtonInput<KeyCode>, ButtonInput<MouseButton>, MousePosition │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  2. App::update_and_render()                                                │
│     ├── initialize_plugins() (first frame only)                             │
│     ├── main_world.run_startup_systems() (first frame only)                 │
│     ├── Update Time resource                                                │
│     └── main_world.run_systems()                                            │
│         ├── PreUpdate: picking_system → pointer_events →                    │
│         │           button_interaction_system, input_system                 │
│         ├── Update: setup_animation, animation_control_system,              │
│         │           model_switch_system, apply_animations                    │
│         └── PostUpdate: (empty)                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  3. SYNC PHASE                                                              │
│     entity_sync_system(&mut main_world, &mut render_world)                  │
│     → Syncs entities marked with SyncToRenderWorld                          │
│     → Creates RenderEntity in Main World, MainEntity in Render World        │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  4. EXTRACT PHASE                                                           │
│     extractors.run(&main_world, &mut render_world)                          │
│     ├── extract_view: Camera → ViewBundle                                   │
│     ├── extract_3d_components: Cube/SoccerBall → ExtractedMesh              │
│     ├── extract_buttons: Button → ExtractedUI                               │
│     ├── queue_meshes: ExtractedMesh → RenderCommand::DrawPolygon/DrawLine   │
│     └── queue_ui: ExtractedUI → RenderCommand::DrawRect                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  5. RENDER PHASE                                                            │
│     render_world.execute_render()                                           │
│     └── SoftwareBackend::execute_commands(&commands)                        │
│         ├── clear(color)                                                    │
│         ├── fill_polygon()                                                  │
│         ├── draw_line()                                                     │
│         └── fill_rect()                                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  6. PRESENT                                                                 │
│     window.present(app.framebuffer())                                       │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 4.2 输入事件响应流程 (Event-Driven)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        INPUT EVENT FLOW (事件驱动)                           │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ 1. 平台输入采集                                                              │
│    Window::collect_input_events()                                           │
│    ├── X11/NuttX: 读取 /dev/input0, /dev/kbd                                │
│    ├── 转换为 MouseMotionEvent { x, y }                                      │
│    └── 转换为 MouseButtonEvent { button, pressed, x, y }                    │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 2. 输入桥接 (WindowRunner)                                                  │
│    bridge_mouse_input():                                                    │
│      ButtonInput<MouseButton>.press(MouseButton::Left)                      │
│    bridge_mouse_position():                                                 │
│      MousePosition { x, y }                                                 │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 3. picking_system (PreUpdate)                                               │
│    输入: MousePosition, ButtonInput<MouseButton>                            │
│         Query<Transform>, Query<PickableBounds>, Query<Pickable>            │
│                                                                             │
│    a. 更新 PointerPress, PointerLocation 资源                               │
│    b. 收集所有 (Entity, Transform, PickableBounds, Pickable)                │
│    c. ui_picking_backend():                                                 │
│       for each (entity, transform, bounds):                                 │
│         if bounds.contains_point(transform.position, mouse_pos):            │
│           hits.push(PointerHits { pointer, entity, hit, order })            │
│    d. update_hover_map():                                                   │
│       swap(hover_map, prev_hover_map)                                       │
│       for hit in sorted_hits:                                               │
│         if pickable.is_hoverable:                                           │
│           hover_map.insert(pointer_id, (entity, hit))                       │
│    e. pointer_events():                                                     │
│       根据 HoverMap 变化和 PointerPress 状态生成事件:                        │
│       ├── Hover 变化 → Pointer<Over>, Pointer<Out>                          │
│       ├── Press 变化 → Pointer<Press>, Pointer<Release>                     │
│       └── Release on same entity → Pointer<Click>                           │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 4. button_interaction_system (PreUpdate)                                    │
│    输入: Events, Query<Button>                                              │
│                                                                             │
│    监听事件:                                                                 │
│    ├── Pointer<Click> → button.clicked = true, state = Hover                │
│    ├── Pointer<Over> → button.state = Hover                                 │
│    ├── Pointer<Out> → button.state = Normal                                 │
│    └── Pointer<Press> → button.state = Pressed                              │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 5. input_system (PreUpdate)                                                 │
│    for (_, button) in button_query.iter():                                  │
│      if button.clicked:                                                     │
│        match button.text:                                                   │
│          "Prev" => 切换模型                                                  │
│          "Pause" => 暂停/恢复                                                │
│          "Next" => 切换模型                                                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 4.3 组件到渲染流程 (Component → Render Pipeline)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    COMPONENT TO RENDER PIPELINE                              │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│ 1. MAIN WORLD - 组件存储                                                    │
│                                                                             │
│    Entity A:                                                                │
│    ├── Node::ui_control()                                                   │
│    ├── Transform { position: Vec3, rotation: Vec3, scale: Vec3 }            │
│    ├── Button { width, height, state, normal_color, hover_color, ... }      │
│    ├── PickableBounds { width, height }                                     │
│    ├── Pickable::DEFAULT                                                    │
│    └── SyncToRenderWorld  ← 标记需要同步到渲染世界                           │
│                                                                             │
│    Entity B:                                                                │
│    ├── Node::game_entity()                                                  │
│    ├── Transform3D { position, rotation, scale }                            │
│    ├── Cube { size, face_colors, rotation, wireframe }                      │
│    ├── AnimationPlayer { active_animations }                                │
│    └── SyncToRenderWorld                                                    │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       │ entity_sync_system
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 2. ENTITY SYNC - 创建渲染世界实体                                           │
│                                                                             │
│    Main World Entity A ←→ Render World Entity A'                            │
│         RenderEntity(A')              MainEntity(A)                         │
│                                                                             │
│    Main World Entity B ←→ Render World Entity B'                            │
│         RenderEntity(B')              MainEntity(B)                         │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       │ extractors.run()
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 3. EXTRACT PHASE - 提取组件数据到渲染世界                                    │
│                                                                             │
│    extract_view():                                                          │
│      Camera → ViewBundle { view, projection, viewport }                     │
│                                                                             │
│    extract_3d_components():                                                 │
│      for (entity, cube, transform) in main_world.query():                   │
│        render_world.insert(entity, ExtractedMesh {                          │
│          vertices: cube.get_vertices(),                                     │
│          colors: cube.face_colors,                                          │
│          transform: transform.to_matrix(),                                  │
│        })                                                                   │
│                                                                             │
│    extract_buttons():                                                       │
│      for (entity, button, transform) in main_world.query():                 │
│        render_world.insert(entity, ExtractedUI {                            │
│          rect: Rect::from_center_size(transform.position, button.size),     │
│          color: button.current_color(),                                     │
│          corner_radius: button.corner_radius,                               │
│        })                                                                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       │ queue_meshes(), queue_ui()
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 4. QUEUE PHASE - 生成渲染命令                                               │
│                                                                             │
│    queue_meshes():                                                          │
│      for (entity, mesh) in render_world.query():                            │
│        commands.push(RenderCommand::DrawPolygon {                           │
│          vertices: mesh.transformed_vertices(),                             │
│          color: mesh.colors,                                                │
│        })                                                                   │
│        if mesh.wireframe:                                                   │
│          commands.push(RenderCommand::DrawLine { ... })                     │
│                                                                             │
│    queue_ui():                                                              │
│      for (entity, ui) in render_world.query():                              │
│        commands.push(RenderCommand::DrawRectRounded {                       │
│          rect: ui.rect,                                                     │
│          color: ui.color,                                                   │
│          radius: ui.corner_radius,                                          │
│        })                                                                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       │ execute_render()
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 5. RENDER PHASE - 执行渲染                                                  │
│                                                                             │
│    SoftwareBackend::execute_commands(&commands):                            │
│      for cmd in commands:                                                   │
│        match cmd:                                                           │
│          Clear { color } => framebuffer.fill(color)                         │
│          DrawPolygon { vertices, color } => fill_polygon(...)               │
│          DrawLine { start, end, color } => draw_line(...)                   │
│          DrawRectRounded { rect, color, radius } => fill_rect_rounded(...)  │
│                                                                             │
│    输出: framebuffer: Vec<u32> (ARGB8888)                                   │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       │ window.present()
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ 6. PRESENT - 输出到屏幕                                                     │
│                                                                             │
│    X11: XPutImage(display, window, gc, image, ...)                          │
│    NuttX: write(fb_fd, framebuffer, size)                                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 4.4 组件数据流总结

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                         COMPONENT DATA FLOW                                   │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Main World                    Render World                    Output        │
│  ──────────                    ────────────                    ──────        │
│                                                                              │
│  Transform ──────────────┐                                                   │
│  Cube ───────────────────┼──→ ExtractedMesh ──→ DrawPolygon ──→ Framebuffer  │
│  SoccerBall ─────────────┘                                                   │
│                                                                              │
│  Transform ──────────────┐                                                   │
│  Button ─────────────────┼──→ ExtractedUI ────→ DrawRect ────→ Framebuffer   │
│  (state affects color) ───┘                                                  │
│                                                                              │
│  AnimationPlayer ────────→ apply_animations ──→ 更新 Cube/SoccerBall 属性    │
│                                                                              │
│  Button.state ───────────→ button.current_color() ──→ ExtractedUI.color      │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

## 模块详解

### 1. Asset System (资产系统)

**职责**: 资产管理，对齐 Bevy 的 bevy_asset crate

**架构**:
```
Main World                          Render World
-----------                         ------------
Assets<Image>  ──ExtractSchedule──> ExtractedAssets<GpuImage>
              (RenderAssetPlugin)
                                    ──PrepareAssets──> RenderAssets<GpuImage>
```

**核心类型**:
- `Asset`: 资产类型标记 trait
- `AssetId<A>`: 资产唯一标识符 (Index 或 Uuid)
- `Handle<A>`: 引用计数的资产句柄 (Strong/Weak/Uuid)
- `Assets<A>`: 资产集合存储 (Resource)
- `AssetEvent<A>`: 资产生命周期事件 (Added/Modified/Removed/Unused)
- `RenderAsset`: GPU 资产抽象 trait
- `RenderAssets<A>`: Render World 资产存储
- `RenderAssetPlugin<A>`: 自动提取和准备资产的插件

**使用示例**:
```rust
// 定义资产类型
#[derive(Clone)]
struct Image {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

// 定义 GPU 表示
struct GpuTexture { ... }

impl RenderAsset for GpuTexture {
    type SourceAsset = Image;
    
    fn prepare_asset(source: &Image, render_world: &mut RenderWorld) -> Option<Self> {
        Some(GpuTexture::from_image(source))
    }
}

// 注册插件
app.add_plugin(RenderAssetPlugin::<GpuTexture>::default());

// 使用资产
fn setup(mut images: ResMut<Assets<Image>>) {
    let handle = images.add(Image { ... });
}
```

**与 Bevy 对比**:
| 特性 | Bevy | FHRE |
|------|------|------|
| 资产类型 | `Asset` derive | `Asset` trait |
| 句柄 | `Handle<A>` (Strong/Weak) | `Handle<A>` (Strong/Weak/Uuid) |
| 存储 | `Assets<A>` | `Assets<A>` |
| 事件 | `AssetEvent<A>` | `AssetEvent<A>` |
| GPU 资产 | `RenderAsset` trait | `RenderAsset` trait |
| 自动提取 | `RenderAssetPlugin` | `RenderAssetPlugin` |
| 异步加载 | ✅ | ❌ |
| 热重载 | ✅ | ❌ |

### 2. Main World (主世界)

**职责**: 游戏逻辑、场景管理、用户系统

**核心类型**:
- `Entity`: 实体 ID (u64)
- `Component`: 组件 trait
- `System`: 系统 trait
- `Query<T>`: 组件查询（单组件）
- `MultiCompQuery<D>`: 多组件查询（支持元组）
- `Commands`: 命令缓冲区

**变更检测**:
```rust
// Mut<T> - 可变访问，自动标记变更
fn my_system(query: MultiCompQuery<Mut<'static, Transform>>) {
    for (_, transform) in query.items() {
        if transform.is_added() { /* 刚添加 */ }
        if transform.is_changed() { /* 已修改 */ }
        transform.position.x += 1.0; // 自动标记变更
    }
}

// Ref<T> - 只读访问，可检查变更状态
fn read_only_system(query: MultiCompQuery<Ref<'static, Transform>>) {
    for (_, transform) in query.items() {
        if transform.is_changed() { /* 检测到变更 */ }
        // transform.position.x += 1.0; // 编译错误：只读
    }
}
```

**重要限制**:
- `Query<T>` 要求 `T: Component`，不支持元组查询如 `Query<(A, B)>`
- 使用 `MultiCompQuery<(A, B)>` 进行多组件查询
- 需要关联多个组件时，使用多个独立的 Query，通过 Entity ID 关联

**系统声明宏**:
```rust
declare_system!(func_name; Param1, Param2, ...)
// 展开为 systemN::<P1, P2, ..., _>(func_name)
```

### 2. Render World (渲染世界)

**职责**: 渲染数据存储、命令生成、帧缓冲输出

**核心类型**:
- `RenderCommand`: 渲染命令枚举
- `DrawCall`: 批处理绘制调用
- `Vertex`: 顶点数据
- `ExtractedMesh`: 提取的 3D 网格数据
- `ExtractedUI`: 提取的 UI 数据
- `ViewBundle`: 视图配置

### 3. Pipeline (渲染管线)

**核心类型**:
- `Renderer`: 渲染后端 trait
- `SoftwareBackend`: CPU 软件渲染器
- `HybridScheduler`: GPU/CPU 任务调度器
- `RenderBatch`: 批处理数据
- `Texture`, `Gradient`: 纹理和渐变系统

**SoftwareBackend 功能**:
- Alpha 混合 (Normal 模式)
- 基本图形绘制 (rect, line, triangle, polygon)
- 渐变填充
- 圆角矩形
- 裁剪区域

**重要**: `blend_lut` 使用堆分配避免栈溢出：
```rust
pub struct SoftwareBackend {
    framebuffer: Vec<u32>,
    blend_lut: Box<[[u8; 256]]>,  // 堆分配，避免栈溢出
    // ...
}
```

### 4. Extract (提取系统)

**职责**: 从 Main World 同步数据到 Render World

**核心类型**:
- `Extract<P>`: 访问 MainWorld 数据的包装器
- `ExtractComponent`: 可提取组件的 trait
- `Extractors`: 提取函数集合

**提取流程**:
```rust
pub fn run_extraction(
    main_world: &mut MainWorld,
    render_world: &mut RenderWorld,
    extract_systems: &[Box<dyn Fn(&MainWorld, &mut RenderWorld)>],
) {
    entity_sync_system(main_world, render_world);
    render_world.clear_views();
    for system in extract_systems {
        system(main_world, render_world);
    }
}
```

### 5. Node System (节点系统)

**设计理念**: Node 是 FHRE 的统一最小单位，作为基础组件不包含类型分类。类型分类由应用层根据需要自行定义。

```rust
pub struct Node {
    pub state: NodeState,
    pub flags: NodeFlags,
}

impl Node {
    pub fn game_entity() -> Self { ... }
    pub fn ui_control() -> Self { ... }
}
```

**注意**: FHRE 是纯图形引擎，不预设实体类型。应用层可根据需要定义自己的类型枚举。

**变换组件**:
```rust
// 统一变换组件
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,  // 欧拉角
    pub scale: Vec3,
}

// 别名
pub type Transform3D = Transform;
```

### 6. Animation System (动画系统)

**架构**:
```
AnimationClip (动画数据)
    │
    ▼
AnimationResources (资源存储)
    │
    ▼
AnimationPlayer (实体组件)
    │
    ▼
apply_animations::<T> (应用到组件)
```

**支持的属性**:
```rust
pub enum AnimationProperty {
    TranslationX, TranslationY, TranslationZ,
    RotationX, RotationY, RotationZ,
    ScaleX, ScaleY, ScaleZ,
    ColorR, ColorG, ColorB, ColorA,
    Custom(u32),
}
```

**AnimationReceiver trait**:
```rust
pub trait AnimationReceiver: Component {
    fn apply_animation(&mut self, property: AnimationProperty, value: f32);
}
```

### 7. Button Component (按钮组件)

**设计理念**: Button 组件管理交互状态，通过事件驱动更新状态。

```rust
pub struct Button {
    // 渲染属性
    pub width: f32,
    pub height: f32,
    pub state: ButtonState,
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
    pub text: &'static str,
    
    // 交互状态
    pub clicked: bool,  // 本帧是否被点击
}

pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
    Disabled,
}
```

**实体创建**:
```rust
commands.spawn()
    .insert(Node::ui_control())
    .insert(Transform::from_2d(x, y))
    .insert(Button::new(width, height).with_text("Click Me"))
    .insert(PickableBounds::from_size(width, height))
    .insert(Pickable::DEFAULT);  // 重要：使用 DEFAULT，而非 default()
```

**事件驱动交互流程**:

```
┌─────────────────────────────────────────────────────────────────────┐
│ 1. 平台触摸事件                                                      │
│    /dev/input0 → read() → TouchEvent { x, y, pressure }             │
│    pressure > 0 → PRESS, pressure == 0 → RELEASE                    │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 2. Window::collect_input_events()                                    │
│    framebuffer.rs 读取触摸事件，转换为：                              │
│    - MouseMotionEvent { x, y }                                       │
│    - MouseButtonEvent { button: 1, pressed, x, y }                   │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 3. WindowRunner::bridge_*()                                          │
│    bridge_mouse_input():                                             │
│      ButtonInput<MouseButton>.press(MouseButton::Left)              │
│    bridge_mouse_position():                                          │
│      MousePosition { x, y }                                          │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 4. picking_system (PreUpdate)                                        │
│    a. 更新 PointerPress, PointerLocation 资源                        │
│    b. 收集所有 (Entity, Transform, PickableBounds, Pickable)         │
│    c. ui_picking_backend(): 生成 PointerHits                         │
│    d. update_hover_map(): 更新 HoverMap, PreviousHoverMap            │
│    e. pointer_events(): 生成 Pointer<E> 事件                         │
│       - Hover 变化 → Pointer<Over>, Pointer<Out>                     │
│       - Press → Pointer<Press>                                       │
│       - Release → Pointer<Release>, Pointer<Click>                   │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 5. button_interaction_system (PreUpdate)                             │
│    监听 Events 中的 Pointer<E> 事件:                                  │
│                                                                      │
│    if Pointer<Click> event:                                          │
│      button.clicked = true   ← 点击完成！                            │
│      button.state = Hover                                            │
│                                                                      │
│    if Pointer<Over> event:                                           │
│      button.state = Hover                                            │
│                                                                      │
│    if Pointer<Out> event:                                            │
│      button.state = Normal                                           │
│                                                                      │
│    if Pointer<Press> event:                                          │
│      button.state = Pressed                                          │
└─────────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 6. input_system (PreUpdate)                                          │
│    for (_, button) in button_query.iter():                          │
│      if button.clicked:                                              │
│        match button.text:                                            │
│          "Prev" => 切换模型                                          │
│          "Pause" => 暂停/恢复                                        │
│          "Next" => 切换模型                                          │
└─────────────────────────────────────────────────────────────────────┘
```

### 8. Input System (输入系统)

**架构原则**: FHRE 核心不知道具体的输入设备，输入类型由平台层定义。

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Application Layer                            │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  User Systems:                                               │   │
│  │  - button_interaction_system: 更新 Button 状态              │   │
│  │  - input_system: 检查 button.clicked 执行操作               │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                              ▲
                              │ ECS Resources
┌─────────────────────────────────────────────────────────────────────┐
│                          Platform Layer                              │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Input Types: ButtonInput<T>, KeyCode, MouseButton          │   │
│  │  InputBridge: map_keycode(), map_mouse_button()             │   │
│  │  WindowRunner: bridge_keyboard_input(), bridge_mouse_input()│   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Window Implementation: collect_input_events()              │   │
│  │  - NuttX/X11: /dev/input0, /dev/kbd                         │   │
│  │  - Other platforms: custom implementation                    │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                              ▲
                              │ Window trait
┌─────────────────────────────────────────────────────────────────────┐
│                           FHRE Core                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  Window trait: collect_input_events(), present()            │   │
│  │  MousePosition: x, y (resource)                              │   │
│  │  WindowInputEvents: raw event types                          │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

**为什么这样设计？**
1. 不同平台有不同的输入设备（键盘、触摸屏、游戏手柄）
2. KeyCode、MouseButton 等类型是平台相关的
3. FHRE 核心只关心抽象的"指针位置"和"按钮状态"
4. 平台层负责将原始事件转换为 ECS 资源

### 9. Picking System (拾取系统)

**架构**: 事件驱动架构，对齐 Bevy 的 Picking 系统。

```
┌─────────────────────────────────────────────────────────────────────┐
│                    PICKING SYSTEM ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌────────┐│
│  │  Pointers   │ →  │   Backend   │ →  │    Hover    │ →  │ Events ││
│  │ ButtonInput │    │ PointerHits │    │  HoverMap   │    │Pointer ││
│  └─────────────┘    └─────────────┘    └─────────────┘    └────────┘│
│                                                                      │
│  1. picking_system (PreUpdate)                                      │
│     ├── 更新 PointerPress, PointerLocation 资源                      │
│     ├── 收集 (Entity, Transform, PickableBounds, Pickable)          │
│     ├── ui_picking_backend() → PointerHits                          │
│     └── update_hover_map() → HoverMap, PreviousHoverMap             │
│                                                                      │
│  2. pointer_events (PreUpdate)                                      │
│     ├── 检测 HoverMap 变化 → Pointer<Over>, Pointer<Out>            │
│     ├── 检测 Press 状态 → Pointer<Press>, Pointer<Release>          │
│     └── 检测 Click 条件 → Pointer<Click>                            │
│                                                                      │
│  3. button_interaction_system (PreUpdate)                           │
│     └── 监听 Pointer<E> 事件更新 Button 状态                         │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**核心类型**:

```rust
// 指针标识
pub enum PointerId {
    Mouse,
    Touch(u64),
    Custom(u64),
}

// 指针状态资源
pub struct PointerPress {
    pub primary: bool,
    pub secondary: bool,
    pub middle: bool,
}

pub struct PointerLocation {
    pub position: Vec2,
}

// 命中数据
pub struct HitData {
    pub position: Vec2,
    pub depth: f32,
}

pub struct PointerHits {
    pub pointer: PointerId,
    pub entity: Entity,
    pub hit: HitData,
    pub order: f32,
}

// 指针事件
pub struct Pointer<E> {
    pub entity: Entity,
    pub pointer_id: PointerId,
    pub pointer_location: PointerLocation,
    pub event: E,
}

// 事件类型
pub struct Over { pub hit: HitData }
pub struct Out { pub hit: HitData }
pub struct Press { pub hit: HitData, pub button: PointerButton }
pub struct Release { pub hit: HitData, pub button: PointerButton }
pub struct Click { pub hit: HitData, pub button: PointerButton }
```

**可点击组件**:
```rust
// 可点击区域
pub struct PickableBounds {
    pub width: f32,
    pub height: f32,
}

// 拾取行为配置
pub struct Pickable {
    pub should_block_lower: bool,  // 是否阻挡下层元素
    pub is_hoverable: bool,        // 是否可悬停
}

impl Pickable {
    pub const DEFAULT: Self = Self { should_block_lower: true, is_hoverable: true };
    pub const IGNORE: Self = Self { should_block_lower: false, is_hoverable: false };
}
```

**重要陷阱**: `Pickable::default()` 的 `is_hoverable=false`（bool 默认值），导致 hits 被丢弃。必须使用 `Pickable::DEFAULT`。

**悬停状态管理**:
```rust
pub struct HoverMap(pub BTreeMap<PointerId, (Entity, HitData)>);
pub struct PreviousHoverMap(pub BTreeMap<PointerId, (Entity, HitData)>);

// update_hover_map 流程:
// 1. 交换 hover_map 和 prev_hover_map
// 2. 按 order 排序 hits
// 3. 如果 pickable.is_hoverable=true，插入 hover_map
// 4. 如果 pickable.should_block_lower=true，停止处理
```

**pointer_events 生成逻辑**:
```rust
// 伪代码
fn pointer_events(...) {
    // Hover 变化检测
    if prev_hit != current_hit {
        if prev_hit.is_some() { send Pointer<Out> }
        if current_hit.is_some() { send Pointer<Over> }
    }
    
    // Press/Release/Click 检测
    if current_hit.is_some() {
        if just_pressed { send Pointer<Press> }
        if just_released { 
            send Pointer<Release>
            send Pointer<Click>  // 在同一实体上按下并释放
        }
    }
}
```

### 10. Plugin System (插件系统)

```rust
pub trait Plugin {
    fn build(&self, app: &mut App);
}

pub trait PluginGroup {
    fn build(self) -> PluginGroupBuilder;
}

// 默认插件组
pub struct DefaultPlugins;
impl PluginGroup for DefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start()
            .add(EventPlugin)
            .add(AnimationPlugin)
            .add(CameraPlugin)
            .add(PickingPlugin)
    }
}
```

## 渲染命令

```rust
pub enum RenderCommand {
    // 基础绘制
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawLine { start: Vec2, end: Vec2, color: Color, thickness: f32 },
    DrawTriangle { p0: Vec2, p1: Vec2, p2: Vec2, color: Color },
    DrawPolygon { vertices: Vec<Vec2>, color: Color },
    
    // 纹理绘制 (v2.6.0 新增)
    DrawPolygonTextured { vertices: Vec<Vec2>, uvs: Vec<Vec2>, texture_id: u32, color: Color },
    
    // 高级绘制
    DrawRectGradient { rect: Rect, gradient: Gradient },
    DrawRectRounded { rect: Rect, color: Color, radius: f32 },
    DrawRectRoundedGradient { rect: Rect, gradient: Gradient, radius: f32 },
    
    // 纹理绘制
    DrawImage { rect: Rect, texture_id: u32, region: TextureRegion, color: Color },
    DrawImageTransformed { position: Vec2, size: Vec2, ..., rotation: f32, color: Color },
    
    // 文本 (占位)
    DrawText { position: Vec2, text: &'static str, color: Color, size: f32 },
    
    // 裁剪
    SetScissor { rect: Rect },
    DisableScissor,
    PushMask,
    PopMask,
}
```

### 纹理映射 (v2.6.0)

FHRE 支持多边形纹理映射，通过 `DrawPolygonTextured` 命令实现：

**数据流**:
```
Cube { face_textures: [Option<u32>; 6] }
    │
    │ extract_3d_components
    ▼
ExtractedMesh { face_textures: Vec<Option<u32>> }
    │
    │ generate_mesh_commands
    ▼
RenderCommand::DrawPolygonTextured { vertices, uvs, texture_id, color }
    │
    │ SoftwareBackend::fill_polygon_textured
    ▼
Framebuffer (采样纹理像素)
```

**UV 坐标生成**:
```rust
// 四边形 UV
let uvs = vec![
    Vec2::new(0.0, 0.0),  // 左上
    Vec2::new(1.0, 0.0),  // 右上
    Vec2::new(1.0, 1.0),  // 右下
    Vec2::new(0.0, 1.0),  // 左下
];
```

**纹理上传**:
```rust
// 在 App 初始化时上传纹理
let texture = Texture::from_rgba32(width, height, data);
app.render_world.upload_texture(texture_id, texture);

// 在组件中引用
Cube::new(size)
    .with_face_textures([tex_front, tex_back, tex_top, tex_bottom, tex_left, tex_right])
```

## 使用示例

### 创建应用

```rust
let mut app = App::new(width, height);

// 添加插件
app.add_plugins(DefaultPlugins);

// 注册资源
app.insert_resource(DemoState::new())
   .insert_resource(ButtonInput::<KeyCode>::default())
   .insert_resource(ButtonInput::<MouseButton>::default())
   .insert_resource(MousePosition::default());

// 注册系统
app.add_systems(Startup, declare_system!(setup; Commands, Res<PrimaryScreen>))
   .add_systems(PreUpdate, declare_system!(picking_system; Res<MousePosition>, Res<ButtonInput<MouseButton>>, Query<Transform>, Query<PickableBounds>, Query<Pickable>, ResMut<HoverMap>, ResMut<PreviousHoverMap>))
   .add_systems(PreUpdate, declare_system!(button_interaction_system; Res<ButtonInput<MouseButton>>, Res<HoverMap>, Res<PreviousHoverMap>, Query<Button>))
   .add_systems(Update, declare_system!(update; Query<&mut Transform>));

// 添加提取器
app.add_extractor(extract_meshes)
   .add_extractor(extract_ui);

// 运行
WindowRunner::new(&mut app, &mut window, &input_adapter).run();
```

### 创建实体

```rust
fn spawn_button(mut commands: Commands) {
    commands.spawn()
        .insert(Node::ui_control())
        .insert(Transform::from_2d(100.0, 200.0))
        .insert(Button::new(100.0, 40.0)
            .with_text("Click Me")
            .with_colors(normal, hover, pressed))
        .insert(PickableBounds::from_size(100.0, 40.0))
        .insert(Pickable::DEFAULT);  // 重要：使用 DEFAULT
}
```

### 动画

```rust
// 创建动画剪辑
let mut clip = AnimationClip::with_duration(6.0);
clip.add_curve_to_target(
    target_id,
    AnimationProperty::RotationY,
    KeyframeCurve::new(vec![
        Keyframe::new(0.0, 0.0, Easing::Linear),
        Keyframe::new(6.0, 360.0, Easing::Linear),
    ]),
);

// 注册并播放
let handle = anim_resources.insert_clip(clip);
player.play_with_target(handle, target_id);
```

## 重要限制与注意事项

### 1. Query 系统已支持元组查询 (v2.8.0)

FHRE 现已支持 Bevy 风格的元组查询，通过 `QueryData` trait 实现：

```rust
// ✅ 单组件查询
fn system1(query: Query<&Transform>) {
    for (entity, transform) in query.iter() {
        // ...
    }
}

// ✅ 多组件查询 (元组)
fn system2(query: Query<(&Transform, &Velocity)>) {
    for (entity, (transform, velocity)) in query.iter() {
        // ...
    }
}

// ✅ 可变访问
fn system3(query: Query<(&mut Transform, &Velocity)>) {
    for (entity, (transform, velocity)) in query.iter_mut() {
        transform.position.x += velocity.x;
    }
}

// ✅ 带 Entity
fn system4(query: Query<(Entity, &Transform, &mut Velocity)>) {
    for (entity, (e, transform, velocity)) in query.iter() {
        // e 是 Entity ID
    }
}

// ✅ 变更检测
fn system5(query: Query<Mut<Transform>>) {
    for (entity, transform) in query.iter() {
        if transform.is_added() { /* 刚添加 */ }
        if transform.is_changed() { /* 已修改 */ }
        transform.position.x += 1.0; // 自动标记变更
    }
}
```

**QueryData trait 实现**:
- `&T` - 只读组件引用
- `&mut T` - 可变组件引用
- `Entity` - 实体 ID
- `Mut<T>` - 带变更检测的可变引用
- `Ref<T>` - 带变更检测的只读引用
- `(D1, D2, ...)` - 元组组合 (最多 15 个元素)

**QueryFilter 支持**:
- `With<T>` - 包含组件 T
- `Without<T>` - 不包含组件 T
- `Added<T>` - 刚添加的组件
- `Changed<T>` - 已修改的组件
- `(F1, F2, ...)` - 过滤器组合

### 2. 栈溢出风险

在 no_std 环境下避免大数组栈分配：
```rust
// ❌ 危险：64KB 栈数组可能导致栈溢出
let blend_lut: [[u8; 256]; 256] = [[0; 256]; 256];

// ✅ 安全：使用堆分配
let blend_lut: Box<[[u8; 256]]> = /* ... */;
```

### 3. Pickable::default() 陷阱

```rust
// ❌ 错误：default() 的 is_hoverable=false，导致 hits 被丢弃
.insert(Pickable::default())

// ✅ 正确：使用 DEFAULT 常量
.insert(Pickable::DEFAULT)
```

## 未使用模块

以下模块已实现但在当前 demo 中未使用，可根据需要启用或移除：

| 模块 | 文件 | 说明 |
|------|------|------|
| animation/graph.rs | AnimationGraph | 动画混合图 |
| animation/transition.rs | AnimationTransitions | 动画过渡 |
| pipeline/gpu/ | - | GPU 后端占位 |
| pipeline/batch.rs | HybridScheduler | GPU/CPU 混合调度 |
| schedule/condition.rs | RunCondition | 运行条件 |
| schedule/set.rs | SystemSet | 系统集 |

**已启用的模块**:
| 模块 | 文件 | 说明 |
|------|------|------|
| main_world/tuples.rs | all_tuples! | 元组宏，生成 0-15 元素实现 |
| main_world/query_data.rs | QueryData | 多组件查询 trait |
| main_world/query_filter.rs | QueryFilter | 查询过滤器 trait |
| main_world/filtered_query.rs | Query | 统一查询类型 |
| asset/server.rs | AssetServer | 统一资产管理接口 |
| asset/extract_plugin.rs | ExtractResourcePlugin | 资源自动提取 |

## Query 系统详解

### 架构概览

FHRE 的 Query 系统对齐 Bevy 的 `QueryData`/`QueryFilter` 架构：

```
┌─────────────────────────────────────────────────────────────────┐
│                        Query System                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Query<D: QueryData, F: QueryFilter = ()>                        │
│       │                                                           │
│       ├── QueryData (数据访问)                                    │
│       │   ├── &T          → 只读组件引用                          │
│       │   ├── &mut T      → 可变组件引用                          │
│       │   ├── Entity      → 实体 ID                               │
│       │   ├── Mut<T>      → 带变更检测的可变引用                  │
│       │   ├── Ref<T>      → 带变更检测的只读引用                  │
│       │   └── (D1, D2,..) → 元组组合 (最多 15 个)                │
│       │                                                           │
│       └── QueryFilter (过滤条件)                                  │
│           ├── With<T>     → 包含组件 T                            │
│           ├── Without<T>  → 不包含组件 T                          │
│           ├── Added<T>    → 刚添加的组件                          │
│           ├── Changed<T>  → 已修改的组件                          │
│           └── (F1, F2,..) → 过滤器组合 (AND 语义)                │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### all_tuples! 宏

用于生成元组实现，类似 Bevy 的 `variadics_please::all_tuples!`：

```rust
// main_world/tuples.rs
#[macro_export]
macro_rules! all_tuples {
    ($macro:ident, $start:tt, $end:tt, $T:ident) => {
        $macro!();           // ()
        $macro!(T0);         // (T0,)
        $macro!(T0, T1);     // (T0, T1)
        // ... 最多 15 个
        $macro!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
    };
}

// 使用示例
macro_rules! impl_tuple_query_data {
    () => { /* 空元组实现 */ };
    ($($T:ident),*) => {
        impl<$($T: QueryData),*> QueryData for ($($T,)*) {
            type Item<'w> = ($($T::Item<'w>,)*);
            // ...
        }
    };
}

crate::all_tuples!(impl_tuple_query_data, 0, 15, T);
```

### QueryData trait

```rust
pub trait QueryData {
    type Item<'w>;
    
    unsafe fn fetch<'w>(world: &'w mut MainWorld, entity: Entity) -> Option<Self::Item<'w>>;
    fn matches(world: &MainWorld, entity: Entity) -> bool;
}

// 实现
impl<T: Component> QueryData for &T { ... }
impl<T: Component> QueryData for &mut T { ... }
impl QueryData for Entity { ... }
impl<T: Component> QueryData for Mut<'static, T> { ... }
impl<T: Component> QueryData for Ref<'static, T> { ... }
// 元组实现由 all_tuples! 宏生成
```

### QueryFilter trait

```rust
pub trait QueryFilter {
    fn matches(entity: Entity, world: &MainWorld) -> bool;
}

// 实现
impl<T: Component> QueryFilter for With<T> { ... }
impl<T: Component> QueryFilter for Without<T> { ... }
impl<T: Component> QueryFilter for Added<T> { ... }
impl<T: Component> QueryFilter for Changed<T> { ... }
// 元组实现由 all_tuples! 宏生成 (AND 语义)
```

### Query 使用示例

```rust
// 单组件
fn system1(query: Query<&Transform>) {
    for (entity, transform) in query.iter() { }
}

// 多组件
fn system2(query: Query<(&Transform, &Velocity)>) {
    for (entity, (transform, velocity)) in query.iter() { }
}

// 可变访问
fn system3(query: Query<(&mut Transform, &Velocity)>) {
    for (entity, (transform, velocity)) in query.iter_mut() { }
}

// 带 Entity
fn system4(query: Query<(Entity, &Transform)>) {
    for (entity, (e, transform)) in query.iter() { }
}

// 变更检测
fn system5(query: Query<Mut<Transform>>) {
    for (entity, transform) in query.iter() {
        if transform.is_changed() { }
    }
}

// 过滤器
fn system6(query: Query<&Transform, With<Velocity>>) {
    // 只查询有 Velocity 的实体
}

fn system7(query: Query<&Transform, (With<Velocity>, Without<Static>)>) {
    // 有 Velocity 且没有 Static 的实体
}

// 变更检测过滤器
fn system8(query: Query<&Transform, Changed<Transform>>) {
    // 只查询已修改的 Transform
}
```

### 与 Bevy Query 对比

| 特性 | FHRE | Bevy |
|------|------|------|
| 基本语法 | `Query<&T>` | `Query<&T>` |
| 元组查询 | `Query<(&A, &B)>` | `Query<(&A, &B)>` |
| Entity | `Query<(Entity, &T)>` | `Query<(Entity, &T)>` |
| 可选组件 | ❌ | `Query<Option<&T>>` |
| AnyOf | ❌ | `Query<AnyOf<(&A, &B)>>` |
| Has<T> | ❌ | `Query<Has<T>>` |
| 过滤器 | `Query<&T, With<U>>` | `Query<&T, With<U>>` |
| Or 过滤器 | ❌ | `Query<&T, Or<(With<A>, With<B>)>>` |
| 变更检测 | `Mut<T>`/`Ref<T>` | `&mut T` 自动 |
| QueryState | ✅ | ✅ |
| ParIter | ❌ | ✅ |
| iter_combinations | ❌ | ✅ |

## 与 Bevy 对比

### 版本信息

| 项目 | 版本 | 代码规模 |
|------|------|----------|
| FHRE | 2.8.0 | ~18,000 行 (95 文件) |
| Bevy | 0.19.0-dev | ~468,000 行 (57 crates) |

### 架构对比

| 特性 | FHRE | Bevy |
|------|------|------|
| 目标平台 | 嵌入式 (no_std) | 桌面/移动端/Web |
| ECS | 简化 ECS (BTreeMap) | 完整 ECS (Archetype) |
| Query 元组 | ✅ Query<(&A, &mut B)> | ✅ 支持元组 |
| QueryData trait | ✅ 手动实现 | ✅ derive macro |
| QueryFilter | ✅ With/Without/Added/Changed | ✅ 完整过滤器 |
| 变更检测 | ✅ Mut<T>/Ref<T> | ✅ 完整变更检测 |
| 双世界 | ✅ MainWorld + RenderWorld | ✅ MainWorld + RenderWorld |
| 渲染后端 | CPU 软件渲染 | GPU (wgpu: Vulkan/Metal/DX12/WebGPU) |
| 调度系统 | 简化 Schedule (5 阶段) | 完整 Schedule (12+ 阶段) |
| 插件系统 | ✅ Plugin + PluginGroup | ✅ Plugin + PluginGroup |
| 动画系统 | ✅ 基础动画 | ✅ 完整动画 (骨骼/变形/混合) |
| Picking 系统 | ✅ UI Picking | ✅ 多后端 Picking (Mesh/Sprite/UI) |
| UI 系统 | Node 统一 | bevy_ui + bevy_ui_widgets |
| 着色器 | ❌ | WGSL/SPIR-V/GLSL |
| 多线程 | ❌ | ✅ async_executor |
| 资产系统 | ✅ Assets<T> + RenderAsset | ✅ bevy_asset (异步加载/热重载) |
| 纹理映射 | ✅ 多边形纹理 | ✅ 完整纹理系统 |
| 反射 | ❌ | ✅ bevy_reflect |
| 场景序列化 | ❌ | ✅ bevy_scene (glTF/RON) |
| 音频 | ❌ | ✅ bevy_audio |
| 输入 | ⚠️ 基础 (ButtonInput) | ✅ bevy_input (键盘/鼠标/手柄/触摸) |
| 窗口 | ❌ 外部提供 | ✅ bevy_winit |
| 数学库 | 自实现 (Vec2/Vec3/Mat4) | glam + bevy_math |
| 依赖 | 0 外部 crate | ~100+ crates |

### Bevy Crate 映射

FHRE 模块与 Bevy crate 的对应关系：

| FHRE 模块 | Bevy Crate | 说明 |
|-----------|------------|------|
| main_world/ | bevy_ecs | ECS 核心 |
| main_world/tuples.rs | bevy_utils/variadics_please | all_tuples! 宏 |
| main_world/query_data.rs | bevy_ecs::query::QueryData | QueryData trait |
| main_world/query_filter.rs | bevy_ecs::query::QueryFilter | QueryFilter trait |
| main_world/filtered_query.rs | bevy_ecs::system::Query | Query 类型 |
| render_world/ | bevy_render | 渲染世界 |
| app/ | bevy_app | 应用框架 |
| plugin/ | bevy_app (Plugin) | 插件系统 |
| schedule/ | bevy_ecs::schedule | 调度系统 |
| resources/ | bevy_ecs::resource | 资源系统 |
| event/ | bevy_ecs::event | 事件系统 |
| animation/ | bevy_animation | 动画系统 |
| picking/ | bevy_picking | 拾取系统 |
| node/ | bevy_transform + bevy_hierarchy | 变换层级 |
| math/ | bevy_math (glam) | 数学类型 |
| pipeline/ | bevy_render | 渲染管线 |
| camera/ | bevy_camera | 相机系统 |
| window/ | bevy_window | 窗口抽象 |
| asset/ | bevy_asset | 资产系统 |
| - | bevy_reflect | 反射系统 (FHRE 无) |
| - | bevy_scene | 场景序列化 (FHRE 无) |
| - | bevy_input | 输入系统 (FHRE 简化) |
| - | bevy_ui | UI 系统 (FHRE 简化) |
| - | bevy_sprite | 2D 精灵 (FHRE 无) |
| - | bevy_pbr | PBR 渲染 (FHRE 无) |
| - | bevy_gltf | glTF 加载 (FHRE 无) |
| - | bevy_audio | 音频 (FHRE 无) |

### 关键差异详解

#### 1. ECS 存储

**Bevy**: Archetype-based storage
- 组件按实体组合分组存储
- 缓存友好的迭代
- 复杂的元数据管理
- 支持 `#[component(storage = "SparseSet")]`

**FHRE**: BTreeMap storage
- `BTreeMap<TypeId, BTreeMap<EntityId, Component>>`
- 简单直接
- 适合小规模实体 (<1000)
- 无 Archetype 开销

#### 2. Query 系统

**Bevy**: 完整 Query 系统
```rust
// QueryData derive macro
#[derive(QueryData)]
struct MyQuery<'w> {
    entity: Entity,
    transform: &'w Transform,
    velocity: Option<&'w Velocity>,
}

// WorldQuery trait
fn system(query: Query<MyQuery>) { ... }
```

**FHRE**: 简化 Query 系统
```rust
// 手动使用元组
fn system(query: Query<(Entity, &Transform, &Velocity)>) { ... }

// 或使用 QueryData trait 手动实现
impl QueryData for &Transform { ... }
impl QueryData for &mut Transform { ... }
```

**核心差异**:
| 特性 | FHRE | Bevy |
|------|------|------|
| 元组查询 | ✅ 手动 | ✅ derive |
| WorldQuery | ❌ | ✅ 自定义查询类型 |
| Option<T> | ❌ | ✅ 可选组件 |
| AnyOf<...> | ❌ | ✅ 任一组件 |
| Has<T> | ❌ | ✅ 组件存在检测 |
| Query 组合 | ❌ | ✅ iter_combinations |

#### 3. 调度系统

**Bevy**: 完整 Schedule
```
First → PreStartup → Startup → PostStartup
     → PreUpdate → Update → PostUpdate
     → FixedPreUpdate → FixedUpdate → FixedPostUpdate
     → Last
```

**FHRE**: 简化 Schedule
```
Startup → PreUpdate → Update → PostUpdate → Last
```

#### 3. 渲染后端

**Bevy**: wgpu (GPU)
- Vulkan, Metal, DirectX 12, WebGPU
- WGSL 着色器
- 计算着色器支持
- 后处理效果

**FHRE**: SoftwareBackend (CPU)
- 帧缓冲区直接操作
- 基本图形绘制 (rect, line, polygon)
- Alpha 混合
- 无着色器

#### 4. no_std 支持

**Bevy**: 需要 std
- 依赖 async_executor
- 依赖 bevy_reflect
- 依赖大量 std-only crates

**FHRE**: no_std 兼容
- 仅使用 alloc
- 无外部依赖
- 适合嵌入式系统 (NuttX RTOS)

#### 5. 资源系统

**Bevy**: bevy_asset
- 异步加载
- 热重载
- 资源处理器
- 多种格式支持

**FHRE**: 简化资源
- 直接嵌入代码
- 无异步加载
- 无热重载

## 文件统计

### FHRE 代码统计

| 模块 | 文件数 | 代码行数 | 说明 |
|------|--------|----------|------|
| main_world | 11 | ~2,200 | ECS 核心 (含 QueryData/QueryFilter/tuples) |
| render_world | 7 | ~1,500 | 渲染数据 |
| pipeline | 7 | ~1,400 | 渲染管线 |
| animation | 8 | ~1,500 | 动画系统 |
| node | 5 | ~1,200 | 节点系统 |
| math | 6 | ~500 | 数学库 |
| schedule | 5 | ~400 | 调度系统 |
| plugin | 5 | ~300 | 插件系统 |
| sync | 4 | ~250 | 双世界同步 |
| extract | 1 | ~180 | 提取系统 |
| event | 5 | ~450 | 事件系统 |
| resources | 5 | ~500 | 资源系统 |
| app | 2 | ~300 | 应用管理 |
| window | 1 | ~95 | 窗口抽象 |
| camera | 1 | ~62 | 相机插件 |
| picking | 8 | ~350 | Picking 系统 (事件驱动) |
| asset | 7 | ~800 | 资产系统 (对齐 Bevy) |
| **FHRE 核心** | **83** | **~11,800** | |
| platform/input | 4 | ~200 | 平台输入类型 |
| platform/runner | 1 | ~150 | 输入桥接 |
| platform/framebuffer | 1 | ~400 | 窗口实现 |
| **平台层** | **6** | **~750** | |
| components | 4 | ~700 | Button, Cube, SoccerBall |
| extract.rs | 1 | ~330 | 自定义提取器 |
| **应用层** | **5** | **~1,030** | |
| **FHRE 总计** | **95** | **~18,000** | |

### Bevy 代码统计

| Crate | 说明 |
|-------|------|
| bevy_ecs | ECS 核心 (Archetype, Query, Schedule) |
| bevy_app | 应用框架 (App, Plugin) |
| bevy_render | 渲染系统 (wgpu, 着色器) |
| bevy_asset | 资源加载 (异步, 热重载) |
| bevy_reflect | 反射系统 |
| bevy_scene | 场景序列化 |
| bevy_animation | 动画系统 |
| bevy_picking | 拾取系统 |
| bevy_transform | 变换层级 |
| bevy_camera | 相机系统 |
| bevy_input | 输入系统 |
| bevy_window | 窗口抽象 |
| bevy_ui | UI 系统 |
| bevy_sprite | 2D 精灵 |
| bevy_pbr | PBR 渲染 |
| bevy_gltf | glTF 加载 |
| bevy_audio | 音频系统 |
| bevy_math | 数学类型 (glam) |
| bevy_color | 颜色类型 |
| bevy_text | 文本渲染 |
| bevy_time | 时间系统 |
| bevy_state | 状态机 |
| bevy_log | 日志系统 |
| bevy_diagnostic | 诊断工具 |
| bevy_tasks | 任务调度 |
| bevy_utils | 工具库 |
| ... | (共 57 crates) |
| **Bevy 总计** | **~468,000 行 (57 crates)** |

### 规模对比

| 指标 | FHRE | Bevy | 比例 |
|------|------|------|------|
| 代码行数 | ~17,000 | ~468,000 | 1:27 |
| 文件/Crate 数 | 92 | 57 crates | - |
| 外部依赖 | 0 | ~100+ | - |
| 目标平台 | 嵌入式 | 桌面/移动/Web | - |

## 版本历史

### v2.8.0 (当前)
- **统一 Query 系统** - 对齐 Bevy 的 QueryData/QueryFilter 架构
  - `Query<D, F>` 支持单组件和多组件元组查询
  - `QueryData` trait: `&T`, `&mut T`, `Entity`, `Mut<T>`, `Ref<T>`, 元组
  - `QueryFilter` trait: `With<T>`, `Without<T>`, `Added<T>`, `Changed<T>`, 元组
  - `all_tuples!` 宏生成 0-15 元素的元组实现
- **变更检测集成**
  - `Mut<T>` 可变访问，自动标记变更
  - `Ref<T>` 只读访问，可检查变更状态
  - `is_added()` / `is_changed()` 方法
- **系统参数简化**
  - `declare_system!` 宏自动推断参数数量
  - 支持 1-10 个系统参数

### v2.7.0
- RenderPhase + PhaseItem 自动排序渲染命令
  - `PhaseItem` 包含 `RenderCommand` 和排序键
  - `RenderPhases` 按类型存储和排序 PhaseItem
  - `RenderWorld::add_phase_item()` 添加渲染项
  - `RenderWorld::execute_render()` 自动排序并执行
- ExtractComponent trait 自动提取组件
  - 简化 trait，只需实现 `extract_component()`
  - `ExtractComponentPlugin<C>` 自动注册提取器
  - `ExtractComponentWithTransform` 支持带 Transform 的组件
- 变更检测集成到 Query 系统
  - `Mut<T>` 可变访问，自动标记变更
  - `Ref<T>` 只读访问，可检查变更状态
  - `QueryData` trait 支持 `Mut<T>` 和 `Ref<T>`
  - `is_added()` / `is_changed()` 方法
- AssetServer 简化版
  - `AssetServer` 统一资产管理接口
  - `AssetRegistry` 按类型存储资产
  - `AssetPlugin<A>` 自动初始化资产类型
  - `AppAssetExt::init_asset::<A>()` 扩展方法

### v2.6.0
- 新增 Asset 系统 (对齐 Bevy bevy_asset)
  - `Asset` trait, `AssetId<A>`, `Handle<A>`, `Assets<A>`
  - `AssetEvent<A>` 资产生命周期事件
  - `RenderAsset` trait, `RenderAssets<A>` GPU 资产抽象
  - `RenderAssetPlugin<A>` 自动提取和准备资产
  - `ExtractResourcePlugin<R>` 自动提取资源
- 新增纹理映射功能
  - `DrawPolygonTextured` 渲染命令
  - `fill_polygon_textured()` 软件渲染实现
  - `ExtractedMesh.face_textures` 字段
  - `Cube.with_face_textures()` 方法
- RenderWorld 新增资源管理 API
  - `init_resource()`, `insert_resource()`, `get_resource()`, `get_resource_mut()`
- Events 新增 `read()` 方法返回 `EventReader<T>`

### v2.5.0
- 事件驱动 Picking 系统
- Button 组件交互状态管理
- Pointer<E> 事件类型

### v2.4.0
- 动画系统
- 双世界同步机制

## FHRE vs Bevy 自动化差距分析

### 已自动化 ✅

| 功能 | FHRE | Bevy | 说明 |
|------|------|------|------|
| 组件同步 | `SyncComponentPlugin<T>` | `ExtractComponentPlugin<T>` | 自动提取组件 |
| 资源提取 | `ExtractResourcePlugin<R>` | `ExtractResourcePlugin<R>` | 自动提取资源 |
| 资产生命周期 | `Assets<T>` + `AssetEvent<T>` | `Assets<T>` + `AssetEvent<T>` | 自动管理 |
| GPU 资产准备 | `RenderAssetPlugin<T>` | `RenderAssetPlugin<T>` | 自动 Extract + Prepare |
| 事件缓冲 | `Events` 双缓冲 | `Events` 双缓冲 | 自动管理 |
| 动画应用 | `apply_animations<T>` | `apply_animations<T>` | 自动应用到组件 |
| Picking 事件 | `pointer_events()` | `pointer_events()` | 自动生成事件 |
| 渲染阶段排序 | `RenderPhase<T>` + `PhaseItem` | `RenderPhase<T>` 自动排序 | 自动排序渲染命令 |
| 组件提取逻辑 | `ExtractComponent` trait | `ExtractComponent` trait | 声明式提取 |
| 变更检测 | `Mut<T>`, `Ref<T>` 集成 Query | `Mut<T>`, `Ref<T>` 完整 | Query 中使用变更检测 |
| 资产加载 | `AssetServer` + `AssetPlugin<T>` | `AssetServer` 异步加载 | 统一资产接口 |
| 元组查询 | `Query<(&A, &B)>` | `Query<(&A, &B)>` | all_tuples! 宏自动生成 |
| QueryFilter | `With<T>`, `Changed<T>` 等 | 完整过滤器 | all_tuples! 宏自动生成 |

### 仍需手动 ❌

| 功能 | FHRE (手动) | Bevy (自动) | 说明 |
|------|-------------|-------------|------|
| 提取器注册 | `app.add_extractor(fn)` | 系统自动调度 | 需手动注册每个提取函数 |
| 视图提取 | 手写 `extract_view()` | `ExtractedViews` 自动 | 需手动提取 Camera |
| 纹理上传 | `render_world.upload_texture()` | `RenderAssetPlugin<GpuImage>` | 需手动上传到 RenderWorld |
| 热重载 | 无 | 文件监视自动重载 | 开发时无热重载 |

### 代码对比示例

#### 1. 提取器注册

**FHRE (手动)**:
```rust
app.add_extractor(extract_view)
   .add_extractor(extract_3d_components)
   .add_extractor(extract_buttons)
   .add_extractor(queue_meshes)
   .add_extractor(queue_ui);
```

**Bevy (自动)**:
```rust
app.add_plugins(RenderAssetPlugin::<GpuImage>::default())
   .add_plugins(ExtractComponentPlugin::<Transform>::default());
```

#### 2. 渲染命令生成 (已自动化 ✅)

**FHRE v2.7.0 (自动)**:
```rust
// 使用 RenderPhase + PhaseItem 自动排序
let item = PhaseItem::opaque_3d(command, sort_key);
render_world.add_phase_item(RenderPhaseType::Opaque3d, item);

// execute_render() 自动排序并执行
render_world.execute_render();
```

**Bevy (自动)**:
```rust
// RenderPhase 自动收集、排序、执行
pub struct Transparent3d {
    pub entity: Entity,
    pub draw_function: DrawFunctionId,
    pub pipeline: CachedRenderPipelineId,
    pub distance: f32,
}

// 自动按 distance 排序
phase.sort_by_key(|item| item.distance);
```

#### 3. 组件提取 (已自动化 ✅)

**FHRE v2.7.0 (自动)**:
```rust
// 实现 ExtractComponent trait
impl ExtractComponent for MyComponent {
    type Out = ExtractedMyComponent;
    
    fn extract_component(&self) -> Option<Self::Out> {
        Some(ExtractedMyComponent { ... })
    }
}

// 注册插件自动提取
app.add_plugin(ExtractComponentPlugin::<MyComponent>::default());
```

**Bevy (自动)**:
```rust
// 类似的 ExtractComponent trait
#[derive(ExtractComponent)]
struct MyComponent;
```

#### 4. 变更检测 (已自动化 ✅)

**FHRE v2.8.0 (自动)**:
```rust
fn my_system(query: Query<Mut<Transform>>) {
    for (entity, transform) in query.iter() {
        if transform.is_added() { /* 刚添加 */ }
        if transform.is_changed() { /* 已修改 */ }
        transform.position.x += 1.0; // 自动标记变更
    }
}
```

**Bevy (自动)**:
```rust
fn my_system(query: Query<&mut Transform>) {
    for mut transform in query {
        if transform.is_added() { /* 刚添加 */ }
        if transform.is_changed() { /* 已修改 */ }
        transform.translation.x += 1.0;
    }
}
```

#### 5. 资产加载 (已自动化 ✅)

**FHRE v2.7.0 (自动)**:
```rust
// 使用 AssetServer 统一接口
let handle: Handle<Image> = asset_server.add(Image { ... });

// 或使用 AssetPlugin 自动初始化
app.init_asset::<Image>();
```

**Bevy (自动)**:
```rust
// AssetServer 异步加载
let handle: Handle<Image> = asset_server.load("textures/cube.png");
```

### 架构流程对比

**Bevy 完整流程**:
```
AssetServer.load() → Assets<T> → AssetEvent → Extract → Prepare → RenderPhase → Draw
```

**FHRE 当前流程**:
```
手动创建 → 手动 upload_texture() → 手动 extract_*() → 手动 queue_*() → 手动排序 → Draw
```

### 改进优先级建议

| 优先级 | 功能 | 工作量 | 收益 | 状态 |
|--------|------|--------|------|------|
| 高 | RenderPhase + PhaseItem | 中 | 自动排序渲染命令 | ✅ 已完成 |
| 高 | ExtractComponent 自动提取 | 低 | 减少样板代码 | ✅ 已完成 |
| 高 | 变更检测集成 | 中 | 优化提取性能 | ✅ 已完成 |
| 高 | Query 元组支持 | 中 | 对齐 Bevy Query | ✅ 已完成 |
| 中 | AssetServer 简化版 | 高 | 统一资产加载 | ✅ 已完成 |
| 低 | 热重载 | 高 | 开发体验提升 | 待实现 |

## 未来规划

### 短期
- [x] RenderPhase + PhaseItem 自动排序
- [x] ExtractComponent trait 自动提取
- [x] 变更检测集成到 Query
- [x] AssetServer 简化版
- [x] Query 元组支持 (QueryData/QueryFilter)
- [ ] 字体渲染系统
- [ ] 纹理图集 (TextureAtlas)
- [ ] 抗锯齿 (AA)

### 中期
- [ ] GPU 后端 (OpenGL ES)
- [ ] 着色器系统
- [ ] 后处理效果
- [ ] 热重载支持
- [ ] Option<T> 可选组件查询
- [ ] AnyOf<...> 查询支持

### 长期
- [ ] Vulkan 后端
- [ ] 多线程渲染
- [ ] 粒子系统
- [ ] Query derive macro

## 参考

- Bevy 源码: `/home/uan-wsl2/codes/bevy`
  - bevy_ecs: ECS 核心
  - bevy_render: 渲染系统
  - bevy_app: 应用框架
- Bevy 版本: 0.19.0-dev (Rust 1.92.0)
- Bevy 仓库: https://github.com/bevyengine/bevy
