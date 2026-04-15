# FHRE 默认摄像机设计

## 核心问题

对于 UI 界面，是否应该有一个默认的摄像机组件来完成 2D 窗口的视角问题？然后后端渲染世界基于这个默认的摄像机组件来实现投影？

**答案：是的，这是一个正确且必要的设计。**

## 设计思路

### 为什么需要默认摄像机？

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         默认摄像机的必要性                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  场景 1：纯 UI 应用                                                          │
│  ───────────────                                                            │
│  用户只创建了一些按钮和标签：                                                 │
│  ```rust                                                                     │
│  commands.spawn((Button, Transform3D::from_position(100, 50, 0)));          │
│  commands.spawn((Label, Transform3D::from_position(200, 100, 0)));          │
│  ```                                                                         │
│                                                                             │
│  问题：没有摄像机！怎么渲染？                                                │
│  答案：App 自动创建一个默认的 2D 正交摄像机                                  │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  场景 2：游戏开发                                                            │
│  ─────────────                                                              │
│  用户创建了游戏摄像机：                                                       │
│  ```rust                                                                     │
│  commands.spawn((                                                            │
│      Camera3D::new().with_fov(60.0),                                        │
│      Transform3D::from_position(0, 10, -20),                                │
│  ));                                                                         │
│  ```                                                                         │
│                                                                             │
│  问题：UI 元素（血条、小地图）在哪渲染？                                     │
│  答案：App 自动创建一个 UI 摄像机，专门渲染 UI 层                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 默认摄像机架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         默认摄像机架构                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  App::new() 时自动创建：                                                     │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Main World (自动初始化)                           │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  默认 2D UI 摄像机 (Entity 0)                                │   │   │
│  │  │  ─────────────────────────                                   │   │   │
│  │  │  Node { node_type: Camera }                                  │   │   │
│  │  │  Transform3D {                                               │   │   │
│  │  │      position: (width/2, height/2, 100),  // 屏幕中心上方    │   │   │
│  │  │      rotation: (0, 0, 0),                                    │   │   │
│  │  │  }                                                           │   │   │
│  │  │  Camera3D {                                                  │   │   │
│  │  │      orthographic: true,       // 正交投影                    │   │   │
│  │  │      orthographic_size: height/2,  // 视口范围                │   │   │
│  │  │      viewport: (0, 0, 1, 1),   // 全屏                        │   │   │
│  │  │      depth: -100,              // 最先渲染                    │   │   │
│  │  │      culling_mask: 0xFFFFFFFF, // 渲染所有层                  │   │   │
│  │  │  }                                                           │   │   │
│  │  │  DefaultUiCamera,              // 标记为默认摄像机            │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│                              ↓ Extract 阶段                                 │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Render World                                     │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  ViewBundle (从默认摄像机生成)                               │   │   │
│  │  │  ───────────────────────────                                 │   │   │
│  │  │  view: View {                                                │   │   │
│  │  │      projection: ortho(0, width, height, 0, -1000, 1000),    │   │   │
│  │  │      view: identity,         // 无变换                        │   │   │
│  │  │      viewport: (0, 0, width, height),                        │   │   │
│  │  │      ...                                                     │   │   │
│  │  │  }                                                           │   │   │
│  │  │  target: Screen                                              │   │   │
│  │  │  clear: ClearConfig::color(Color::BLACK)                     │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│                              ↓ Render 阶段                                  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Pipeline                                         │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  SoftwareBackend (CPU 软件渲染)                              │   │   │
│  │  │  - 执行 RenderCommand                                        │   │   │
│  │  │  - 输出到 framebuffer                                        │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 实现方案

### 1. 默认摄像机资源

```rust
// app/app.rs

/// Resource to store the default UI camera entity
#[derive(Clone, Copy, Debug)]
pub struct DefaultUiCamera {
    pub entity: Entity,
}

/// Resource to store the default game camera entity (if any)
#[derive(Clone, Copy, Debug)]
pub struct DefaultGameCamera {
    pub entity: Entity,
}

impl crate::resources::Resource for DefaultUiCamera {}
impl crate::resources::Resource for DefaultGameCamera {}
```

### 2. App 初始化时创建默认摄像机

```rust
// app/app.rs

impl App {
    pub fn new() -> Self {
        let config = AppConfig::default();
        let (width, height) = (config.width, config.height);

        // Try to create SIM display (for NuttX SIM platform)
        #[cfg(feature = "sim")]
        let sim_display = SimDisplay::new();
        
        #[cfg(not(feature = "sim"))]
        let sim_display: Option<()> = None;
        
        // If SIM display is available, use its dimensions
        #[cfg(feature = "sim")]
        let (width, height) = if let Some(ref display) = sim_display {
            display.dimensions()
        } else {
            (width, height)
        };

        let mut app = Self {
            main_world: MainWorld::new(),
            render_world: RenderWorld::new(width, height),
            schedules: Schedules::new(),
            config,
            sim_display,
            running: false,
        };

        // Setup default resources
        app.setup_primary_screen(width, height);
        app.setup_default_ui_camera(width, height);
        
        // Initialize default schedules
        app.init_schedules();

        app
    }
    
    /// Setup the default UI camera
    ///
    /// Creates an orthographic camera for UI rendering.
    /// This camera is always available and renders UI elements
    /// in screen coordinates (pixel-perfect).
    fn setup_default_ui_camera(&mut self, width: u32, height: u32) {
        let camera_entity = self.main_world.spawn();
        
        // Create UI camera node
        self.main_world.insert_component(
            camera_entity,
            Node::ui_control(NodeType::Camera)
        );
        
        // Position camera to look at the screen center from above
        // In orthographic mode, position Z doesn't affect scale,
        // only determines what gets culled by near/far planes
        self.main_world.insert_component(
            camera_entity,
            Transform3D::from_position(
                width as f32 / 2.0,  // Center X
                height as f32 / 2.0, // Center Y
                100.0                // Above the screen
            )
        );
        
        // Orthographic camera for UI
        // orthographic_size = height / 2 means the view spans from -height/2 to +height/2
        // which maps to screen coordinates when camera is at center
        self.main_world.insert_component(
            camera_entity,
            Camera3D {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
                background_color: crate::math::Color::BLACK,
                orthographic: true,
                orthographic_size: height as f32 / 2.0,
                viewport: crate::math::Rect::new(0.0, 0.0, 1.0, 1.0),
                culling_mask: 0xFFFFFFFF,
                depth: -100,  // Render first (lowest depth)
            }
        );
        
        // Store as default UI camera resource
        self.main_world.resources_mut().insert(DefaultUiCamera {
            entity: camera_entity,
        });
    }
}
```

### 3. 渲染流程与 Pipeline 集成

```rust
// app/app.rs

impl App {
    /// Run one update frame
    pub fn update(&mut self) {
        // 1. Update time
        if let Some(time) = self.main_world.resources_mut().get_mut::<Time>() {
            time.update(1.0 / 60.0); // Default to 60 FPS
        }

        // 2. Run Main World systems
        self.main_world.run_systems();

        // 3. Extract phase - sync Main World to Render World
        //    This includes extracting camera data to ViewBundle
        extract_system(&self.main_world, &mut self.render_world);

        // 4. Render phase - execute render commands through RenderWorld
        //    RenderWorld manages the backend internally (Software/GPU/Hybrid)
        self.render_world.execute_render();

        // 5. Present to SIM display if available (NuttX SIM platform)
        #[cfg(feature = "sim")]
        {
            if let Some(ref mut display) = self.sim_display {
                display.present(&self.render_world);
            }
        }
    }
    
    /// Get framebuffer data for output
    pub fn get_framebuffer(&self) -> &[u32] {
        self.render_world.framebuffer()
    }
}
```

### 4. RenderWorld 与 Pipeline 架构

```rust
// render_world/world.rs

pub struct RenderWorld {
    width: u32,
    height: u32,
    objects: Vec<RenderObject>,
    commands: Vec<RenderCommand>,
    clear_color: Color,
    viewport: Rect,
    phases: RenderPhases,
    views: Vec<ViewBundle>,
    current_view: Option<usize>,
    use_phases: bool,
    // Rendering backend (encapsulated)
    backend: SoftwareBackend,  // Could be GPU backend in future
}

impl RenderWorld {
    /// Create a new Render World with specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            objects: Vec::new(),
            commands: Vec::new(),
            clear_color: Color::BLACK,
            viewport: Rect::new(0.0, 0.0, width as f32, height as f32),
            phases: RenderPhases::new(),
            views: Vec::new(),
            current_view: None,
            use_phases: true,
            backend: SoftwareBackend::new(width, height),
        }
    }

    /// Get framebuffer reference
    pub fn framebuffer(&self) -> &[u32] {
        self.backend.framebuffer()
    }

    /// Execute all pending render commands
    ///
    /// This is the main entry point for rendering execution.
    /// It submits all queued commands to the rendering backend.
    pub fn execute_render(&mut self) {
        // Execute all commands through the backend
        self.backend.execute_commands(&self.commands);
    }
    
    /// Render all phases for all views
    pub fn render_all(&mut self) {
        // If no views, create default view
        if self.views.is_empty() {
            self.create_default_view();
            self.current_view = Some(0);
        }

        // Clear the backend framebuffer
        self.backend.reset();

        // Render each view
        for view_idx in 0..self.views.len() {
            self.render_view(view_idx);
        }
    }
}
```

### 5. Pipeline 模块架构

```
pipeline/
├── mod.rs           # 模块导出
├── backend.rs       # SoftwareBackend (CPU 软件渲染)
├── renderer.rs      # Renderer trait (未来 GPU 支持)
└── batch.rs         # 批处理逻辑

// backend.rs - 软件渲染实现
pub struct SoftwareBackend {
    framebuffer: Vec<u32>,
    width: u32,
    height: u32,
    viewport: Rect,
}

impl SoftwareBackend {
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
}
```

## 架构优势

### 1. 职责分离

| 模块 | 职责 |
|------|------|
| **MainWorld** | 游戏逻辑、摄像机实体管理 |
| **RenderWorld** | 渲染数据管理、后端封装 |
| **Pipeline** | 渲染执行（Software/GPU） |
| **App** | 协调各模块，不直接操作后端 |

### 2. 扩展性

- 未来支持 GPU 时，只需在 RenderWorld 中切换 backend 类型
- App 代码无需修改，保持向后兼容
- 通过 `Renderer` trait 可以支持多种后端

### 3. 默认摄像机的好处

1. **开箱即用**：用户无需手动创建摄像机即可渲染 UI
2. **屏幕坐标系**：默认摄像机使用正交投影，方便 UI 定位
3. **自动管理**：App 自动处理摄像机的生命周期
4. **可覆盖**：用户可以创建自己的摄像机替代默认摄像机

## 使用示例

```rust
// 纯 UI 应用 - 无需创建摄像机
fn main() {
    let mut app = App::new();  // 自动创建默认 UI 摄像机
    
    app.add_startup_system(setup_ui);
    app.run();
}

fn setup_ui(mut commands: Commands) {
    // 直接在屏幕坐标系中创建 UI
    commands.spawn((
        Node::ui_control(NodeType::Button),
        Transform3D::from_position(100.0, 200.0, 0.0),
    ));
}

// 游戏应用 - 添加游戏摄像机
fn main() {
    let mut app = App::new();
    
    // 创建游戏摄像机（UI 摄像机仍然存在）
    let game_cam = app.create_game_camera(
        Vec3::new(0.0, 5.0, -10.0),
        Vec3::new(0.0, 0.0, 0.0),
        60.0
    );
    
    app.run();
}
```

---

**文档版本**: v2.0  
**更新日期**: 2026-04-15  
**说明**: 本文档已更新以反映 Pipeline 架构重构，RenderWorld 现在封装了 SoftwareBackend，App 不再直接操作渲染后端。
