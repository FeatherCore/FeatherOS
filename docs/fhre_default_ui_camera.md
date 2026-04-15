# FHRE 默认 UI 摄像机设计

## 核心概念

是的，UI 应该有一个**默认的 2D 摄像机**！这个设计遵循以下原则：

1. **默认存在**：App 初始化时自动创建默认 UI 摄像机
2. **正交投影**：使用正交投影，确保 UI 元素没有透视变形
3. **屏幕坐标对齐**：UI 坐标直接对应屏幕像素坐标
4. **可覆盖**：用户可以创建自己的摄像机来替代默认摄像机

## 设计架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         默认 UI 摄像机架构                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  App::new() 初始化时：                                                       │
│  ───────────────────                                                         │
│                                                                             │
│  1. 创建默认 UI 摄像机实体                                                   │
│     ┌─────────────────────────────────────────────────────────────────┐    │
│     │ Entity: DefaultUICamera                                         │    │
│     │ ├── Node { node_type: Camera, ... }                             │    │
│     │ ├── Transform3D { position: (0, 0, 100), ... }  // z=100 俯视    │    │
│     │ └── Camera3D {                                                  │    │
│     │     orthographic: true,                                         │    │
│     │     orthographic_size: screen_height / 2,                       │    │
│     │     viewport: (0, 0, 1, 1),  // 全屏                            │    │
│     │     depth: -100,  // 最底层                                      │    │
│     │     background_color: Color::BLACK,                             │    │
│     │ }                                                               │    │
│     └─────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  2. 用户添加 UI 元素：                                                       │
│     ┌─────────────────────────────────────────────────────────────────┐    │
│     │ Entity: Button                                                  │    │
│     │ ├── Node { node_type: Button, ... }                             │    │
│     │ ├── Transform3D { position: (100, 200, 0), ... }  // z=0         │    │
│     │ └── Style { width: 120, height: 40, ... }                       │    │
│     └─────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  3. Extract 阶段：                                                           │
│     - 找到 DefaultUICamera → 创建 View (正交投影)                            │
│     - 找到 Button → 创建 PhaseItem (UI 阶段)                                 │
│     - 使用 View.world_to_screen() 将 (100, 200, 0) → 屏幕坐标 (100, 200)      │
│                                                                             │
│  4. 渲染：                                                                   │
│     - 使用正交投影矩阵                                                       │
│     - 100x200 的按钮直接渲染在屏幕 (100, 200) 位置                            │
│     - 没有透视变形                                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 代码实现

### 1. App 初始化默认摄像机

```rust
// app/app.rs
impl App {
    pub fn new() -> Self {
        // ... 现有初始化代码 ...
        
        let mut app = Self {
            main_world: MainWorld::new(),
            render_world: RenderWorld::new(width, height),
            schedules: Schedules::new(),
            config,
            sim_display,
            running: false,
        };

        // 初始化默认 UI 摄像机
        app.setup_default_ui_camera(width, height);
        
        // 初始化默认 schedules
        app.init_schedules();

        app
    }
    
    /// 设置默认 UI 摄像机
    fn setup_default_ui_camera(&mut self, width: u32, height: u32) {
        // 计算正交投影尺寸
        // 我们希望 (0, 0) 在左上角，(width, height) 在右下角
        let ortho_size = height as f32 / 2.0;
        
        let camera_entity = self.main_world.spawn();
        
        // 设置摄像机位置（俯视整个屏幕）
        self.main_world.insert_component(
            camera_entity,
            Transform3D::from_position(width as f32 / 2.0, height as f32 / 2.0, 100.0)
        );
        
        // 设置摄像机组件
        self.main_world.insert_component(
            camera_entity,
            Camera3D {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
                background_color: Color::BLACK,
                orthographic: true,
                orthographic_size: ortho_size,
                culling_mask: 0xFFFFFFFF,  // 渲染所有层
                depth: -100,  // 最底层，最先渲染
                viewport: Rect::new(0.0, 0.0, 1.0, 1.0),  // 全屏
            }
        );
        
        // 标记为默认 UI 摄像机
        self.main_world.insert_component(
            camera_entity,
            DefaultUiCamera
        );
        
        // 存储默认摄像机实体 ID
        self.main_world.resources_mut().insert(DefaultCameraResource {
            entity: camera_entity,
        });
    }
}

/// 默认 UI 摄像机标记组件
#[derive(Component)]
pub struct DefaultUiCamera;

/// 默认摄像机资源
#[derive(Resource)]
pub struct DefaultCameraResource {
    pub entity: Entity,
}
```

### 2. 正交投影矩阵计算

```rust
// render_world/view.rs
impl View {
    /// 创建 2D UI 视图（正交投影，屏幕坐标）
    pub fn new_ui_view(screen_width: f32, screen_height: f32) -> Self {
        let viewport = Rect::new(0.0, 0.0, screen_width, screen_height);
        
        // 正交投影矩阵
        // 左、右、下、上、近、远
        let projection = Mat4::orthographic_rh(
            0.0,           // 左
            screen_width,  // 右
            screen_height, // 下（Y轴向下）
            0.0,           // 上
            -1000.0,       // 近
            1000.0,        // 远
        );
        
        // 视图矩阵（单位矩阵，因为摄像机已经在正确位置）
        let view = Mat4::IDENTITY;
        
        Self {
            viewport,
            projection,
            view,
            view_projection: projection * view,
            camera_position: Vec3::new(screen_width / 2.0, screen_height / 2.0, 100.0),
            near: -1000.0,
            far: 1000.0,
            orthographic: true,
        }
    }
    
    /// 将世界坐标转换为屏幕坐标（正交投影下直接对应）
    pub fn world_to_screen_ortho(&self, world_pos: Vec3) -> Vec2 {
        // 正交投影下，世界坐标直接对应屏幕坐标（考虑视口偏移）
        Vec2::new(
            world_pos.x - self.camera_position.x + self.viewport.width / 2.0,
            world_pos.y - self.camera_position.y + self.viewport.height / 2.0,
        )
    }
}
```

### 3. Extract 阶段处理默认摄像机

```rust
// extract/camera.rs
pub fn extract_cameras(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
    screen_width: f32,
    screen_height: f32,
) {
    // 查询所有摄像机
    let cameras = main_world.query::<(Node, Transform3D, Camera3D)>();
    
    // 按 depth 排序（低 depth 先渲染）
    let mut camera_list: Vec<_> = cameras.collect();
    camera_list.sort_by_key(|(_, _, cam)| cam.depth);
    
    for (entity, (node, transform, camera)) in camera_list {
        if node.node_type != NodeType::Camera {
            continue;
        }
        
        let view = if camera.orthographic {
            // UI 摄像机：使用正交投影
            create_orthographic_view(transform, camera, screen_width, screen_height)
        } else {
            // 3D 摄像机：使用透视投影
            create_perspective_view(transform, camera, screen_width, screen_height)
        };
        
        let view_bundle = ViewBundle {
            view,
            target: ViewTarget::Screen,
            clear: if camera.depth < 0 {
                // 第一个摄像机（通常是默认 UI 摄像机）清除屏幕
                ClearConfig::color(camera.background_color)
            } else {
                // 后续摄像机不清除（叠加渲染）
                ClearConfig::none()
            },
        };
        
        render_world.add_view(view_bundle);
    }
}

fn create_orthographic_view(
    transform: &Transform3D,
    camera: &Camera3D,
    screen_width: f32,
    screen_height: f32,
) -> View {
    let viewport = Rect::new(
        camera.viewport.x * screen_width,
        camera.viewport.y * screen_height,
        camera.viewport.width * screen_width,
        camera.viewport.height * screen_height,
    );
    
    // 正交投影
    let half_width = camera.orthographic_size * (screen_width / screen_height);
    let half_height = camera.orthographic_size;
    
    let projection = Mat4::orthographic_rh(
        -half_width, half_width,
        -half_height, half_height,
        camera.near, camera.far,
    );
    
    // 视图矩阵（摄像机看向 -Z 方向）
    let view = Mat4::look_at_rh(
        transform.position,
        transform.position + Vec3::NEG_Z,  // 看向 -Z
        Vec3::Y,
    );
    
    View {
        viewport,
        projection,
        view,
        view_projection: projection * view,
        camera_position: transform.position,
        near: camera.near,
        far: camera.far,
        orthographic: true,
    }
}
```

### 4. UI 元素坐标系统

```rust
// 用户创建 UI 时，使用屏幕坐标
fn setup_ui(mut commands: Commands) {
    // 按钮在屏幕 (100, 200) 位置，大小 120x40
    commands.spawn((
        Node::ui_control(NodeType::Button),
        Transform3D::from_position(100.0, 200.0, 0.0),  // z=0 在 UI 平面
        Style {
            width: Dimension::Pixel(120.0),
            height: Dimension::Pixel(40.0),
            ..default()
        },
    ));
    
    // 文本在屏幕 (110, 220) 位置
    commands.spawn((
        Node::ui_control(NodeType::Label),
        Transform3D::from_position(110.0, 220.0, 0.0),
        Text { content: "Click me", font_size: 16.0 },
    ));
}
```

## 用户自定义摄像机

用户可以选择：

### 选项 1：使用默认摄像机（推荐用于纯 UI 应用）
```rust
fn main() {
    let mut app = App::new();
    // 自动使用默认 UI 摄像机
    app.run();
}
```

### 选项 2：创建自己的 UI 摄像机
```rust
fn setup(mut commands: Commands) {
    // 删除默认摄像机
    commands.entity(default_camera).despawn();
    
    // 创建自定义 UI 摄像机
    commands.spawn((
        Node::ui_control(NodeType::Camera),
        Transform3D::from_position(400.0, 300.0, 100.0),
        Camera3D::new()
            .orthographic(300.0)
            .with_viewport(Rect::new(0.0, 0.0, 0.5, 1.0))  // 只渲染左半边
            .with_background_color(Color::DARK_GRAY),
    ));
}
```

### 选项 3：多摄像机（游戏 + UI）
```rust
fn setup(mut commands: Commands) {
    // 3D 游戏摄像机
    commands.spawn((
        Node::game_entity(NodeType::Camera),
        Transform3D::from_position(0.0, 5.0, -10.0),
        Camera3D::new()
            .with_fov(60.0)
            .with_viewport(Rect::new(0.0, 0.0, 1.0, 0.8))  // 上部 80%
            .with_depth(0),
    ));
    
    // UI 摄像机（正交）
    commands.spawn((
        Node::ui_control(NodeType::Camera),
        Transform3D::from_position(400.0, 450.0, 100.0),
        Camera3D::new()
            .orthographic(50.0)
            .with_viewport(Rect::new(0.0, 0.8, 1.0, 0.2))  // 底部 20%
            .with_depth(1)
            .with_clear_config(ClearConfig::none()),  // 不清除，叠加
    ));
}
```

## 总结

你的理解完全正确：

1. **默认 UI 摄像机**：App 自动创建，使用正交投影，处理所有 UI 渲染
2. **后端渲染**：基于默认摄像机生成 View，完成 3D→2D 投影
3. **坐标对应**：UI 元素的 (x, y) 直接对应屏幕像素坐标
4. **可覆盖**：用户可以删除默认摄像机，创建自己的摄像机

这就像：
- **默认 UI 摄像机** = 一个固定在屏幕上方的正交相机，专门拍摄 UI 层
- **用户不需要关心投影细节**，只需要在屏幕坐标系中放置 UI 元素

---

**文档版本**: v1.0  
**日期**: 2026-04-15  
**作者**: FeatherOS Team
