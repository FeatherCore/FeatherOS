# FHRE 摄像机与视图架构设计

## 1. 架构概述

### 1.1 核心设计原则

FHRE 采用**默认 3D**的设计理念：
- 所有渲染本质上都是 3D 的
- 2D 只是 z=0 且使用正交投影的特殊情况
- 摄像机是场景中的普通 Node 实体
- 视图（View）是渲染世界的运行时结构

### 1.2 双世界职责分离

| 世界 | 职责 | 摄像机相关 |
|------|------|-----------|
| **Main World** | 声明式场景描述 | Camera3D 组件定义摄像机属性 |
| **Render World** | 运行时渲染执行 | View 结构执行实际渲染计算 |
| **Extract** | 数据转换桥梁 | 将 Camera3D + Transform3D → View |

## 2. Main World 的摄像机系统

### 2.1 Camera3D 组件

```rust
// node3d.rs - Main World 的摄像机组件
#[derive(Component)]
pub struct Camera3D {
    /// 视野角度（度），默认 60°
    pub fov: f32,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 背景颜色
    pub background_color: Color,
    /// 是否正交投影（2D 模式用）
    pub orthographic: bool,
    /// 正交投影尺寸
    pub orthographic_size: f32,
    /// 渲染层掩码（决定渲染哪些层）
    pub culling_mask: u32,
    /// 深度（决定渲染顺序，高值在后）
    pub depth: i32,
    /// 视口（相对于屏幕的归一化坐标）
    pub viewport: Rect, // x, y, width, height (0-1)
}
```

### 2.2 摄像机作为 Node

```rust
// 创建主摄像机（3D 透视）
let main_camera = main_world.spawn();
main_world.insert_component(main_camera, Node::game_entity(NodeType::Camera));
main_world.insert_component(main_camera, Transform3D::from_position(0.0, 0.0, 10.0));
main_world.insert_component(main_camera, Camera3D::new()
    .with_fov(60.0)
    .with_clip(0.1, 1000.0));

// 创建 UI 摄像机（正交投影）
let ui_camera = main_world.spawn();
main_world.insert_component(ui_camera, Node::ui_control(NodeType::Camera));
main_world.insert_component(ui_camera, Transform3D::from_position(0.0, 0.0, 100.0));
main_world.insert_component(ui_camera, Camera3D::new()
    .orthographic(5.0)  // 正交投影
    .with_viewport(Rect::new(0.0, 0.0, 1.0, 1.0)));  // 全屏
```

### 2.3 2D/3D 统一处理

由于 FHRE 默认 3D，2D 元素也是 3D 空间中的实体：

```rust
// 2D 精灵（z=0，与窗口平面重合）
let sprite = main_world.spawn();
main_world.insert_component(sprite, Node::game_entity(NodeType::Sprite2D));
main_world.insert_component(sprite, Transform3D::from_position(100.0, 200.0, 0.0)); // z=0

// 3D 模型（任意 z 值）
let model = main_world.spawn();
main_world.insert_component(model, Node::game_entity(NodeType::Model3D));
main_world.insert_component(model, Transform3D::from_position(0.0, 0.0, 50.0)); // z=50

// 2.5D UI 卡片（轻微 z 偏移，支持翻转效果）
let card = main_world.spawn();
main_world.insert_component(card, Node::ui_control(NodeType::Card));
main_world.insert_component(card, Transform3D::from_position(300.0, 200.0, 10.0)); // z=10
main_world.insert_component(card, Transform3D::with_rotation(0.0, 0.2, 0.0)); // Y轴旋转
```

## 3. Extract 阶段

### 3.1 摄像机提取系统

```rust
// extract/camera.rs
pub fn extract_cameras(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    // 查询所有摄像机节点
    let cameras = main_world.query::<(Node, Transform3D, Camera3D)>();
    
    for (entity, (node, transform, camera)) in cameras {
        if node.node_type != NodeType::Camera {
            continue;
        }
        
        // 计算视图矩阵
        let view_matrix = calculate_view_matrix(transform);
        
        // 计算投影矩阵
        let projection_matrix = if camera.orthographic {
            Mat4::orthographic_rh(
                -camera.orthographic_size * aspect_ratio,
                camera.orthographic_size * aspect_ratio,
                -camera.orthographic_size,
                camera.orthographic_size,
                camera.near,
                camera.far,
            )
        } else {
            Mat4::perspective_rh(
                camera.fov.to_radians(),
                aspect_ratio,
                camera.near,
                camera.far,
            )
        };
        
        // 计算视口（像素坐标）
        let viewport = Rect::new(
            camera.viewport.x * screen_width,
            camera.viewport.y * screen_height,
            camera.viewport.width * screen_width,
            camera.viewport.height * screen_height,
        );
        
        // 创建 View
        let view = View {
            viewport,
            projection: projection_matrix,
            view: view_matrix,
            view_projection: projection_matrix * view_matrix,
            camera_position: transform.position,
            near: camera.near,
            far: camera.far,
            orthographic: camera.orthographic,
        };
        
        // 创建 ViewBundle
        let view_bundle = ViewBundle {
            view,
            target: ViewTarget::Screen,
            clear: ClearConfig::color(camera.background_color),
        };
        
        // 添加到 Render World，按 depth 排序
        render_world.add_view_with_depth(view_bundle, camera.depth);
    }
}
```

### 3.2 渲染对象提取

```rust
// extract/objects.rs
pub fn extract_render_objects(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    // 查询所有可见的渲染节点
    let renderables = main_world.query::<(Node, Transform3D, Option<Sprite>)>();
    
    for (entity, (node, transform, sprite)) in renderables {
        if !node.state.visible {
            continue;
        }
        
        // 确定渲染阶段
        let phase_type = determine_phase(&node);
        
        // 创建渲染对象
        let render_object = RenderObject {
            position: transform.position,
            rotation: transform.rotation,
            scale: transform.scale,
            color: sprite.map(|s| s.color).unwrap_or(Color::WHITE),
            size: sprite.map(|s| s.size).unwrap_or(Vec2::ONE),
            visible: node.state.visible,
            z_order: node.z_order,
        };
        
        // 创建绘制命令
        let command_index = render_world.commands().len();
        render_world.add_command(RenderCommand::DrawRect {
            rect: calculate_screen_rect(&render_object),
            color: render_object.color,
        });
        
        // 创建阶段项
        let phase_item = PhaseItem::new(command_index)
            .with_entity(entity.id())
            .with_z_depth(transform.position.z)  // 使用 3D z 坐标排序
            .with_batch_key(calculate_batch_key(&render_object));
        
        // 添加到对应阶段
        render_world.add_phase_item(phase_type, phase_item);
    }
}

fn determine_phase(node: &Node) -> RenderPhaseType {
    match node.node_type {
        NodeType::Sprite2D | NodeType::Model3D if node.opacity >= 1.0 => {
            RenderPhaseType::Opaque3d
        }
        NodeType::Sprite2D | NodeType::Model3D => {
            RenderPhaseType::Transparent
        }
        NodeType::Button | NodeType::Label | NodeType::Panel => {
            RenderPhaseType::Ui
        }
        _ => RenderPhaseType::Opaque3d,
    }
}
```

## 4. Render World 的视图系统

### 4.1 View 结构（运行时）

```rust
// render_world/view.rs
pub struct View {
    /// 视口（像素坐标）
    pub viewport: Rect,
    /// 投影矩阵
    pub projection: Mat4,
    /// 视图矩阵（摄像机变换的逆）
    pub view: Mat4,
    /// 视图投影矩阵（缓存）
    pub view_projection: Mat4,
    /// 摄像机在世界的坐标
    pub camera_position: Vec3,
    /// 近裁剪面
    pub near: f32,
    /// 远裁剪面
    pub far: f32,
    /// 是否正交投影
    pub orthographic: bool,
}

impl View {
    /// 将世界坐标转换为屏幕坐标
    pub fn world_to_screen(&self, world_pos: Vec3) -> Option<Vec2> {
        // 1. 转换到裁剪空间
        let clip_pos = self.view_projection * world_pos.extend(1.0);
        
        // 2. 检查是否在摄像机后面
        if clip_pos.w <= 0.0 {
            return None;
        }
        
        // 3. 透视除法得到 NDC
        let ndc_x = clip_pos.x / clip_pos.w;
        let ndc_y = clip_pos.y / clip_pos.w;
        
        // 4. 转换到屏幕坐标
        let screen_x = (ndc_x + 1.0) * 0.5 * self.viewport.width + self.viewport.x;
        let screen_y = (1.0 - ndc_y) * 0.5 * self.viewport.height + self.viewport.y;
        
        Some(Vec2::new(screen_x, screen_y))
    }
    
    /// 将屏幕坐标转换为世界射线（用于拾取）
    pub fn screen_to_world_ray(&self, screen_x: f32, screen_y: f32) -> Ray {
        // 1. 转换到 NDC
        let ndc_x = (screen_x - self.viewport.x) / self.viewport.width * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y - self.viewport.y) / self.viewport.height * 2.0;
        
        // 2. 计算射线方向
        let clip_near = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let clip_far = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        
        let inv_view_proj = self.view_projection.inverse();
        let world_near = (inv_view_proj * clip_near).truncate();
        let world_far = (inv_view_proj * clip_far).truncate();
        
        Ray {
            origin: self.camera_position,
            direction: (world_far - world_near).normalize(),
        }
    }
}
```

### 4.2 多视图渲染

```rust
// render_world/world.rs
impl RenderWorld {
    /// 渲染所有视图
    pub fn render_all_views(&mut self) {
        // 按 depth 排序视图（高 depth 在后渲染）
        self.views.sort_by_key(|v| v.depth);
        
        for view_idx in 0..self.views.len() {
            self.render_view(view_idx);
        }
    }
    
    /// 渲染单个视图
    fn render_view(&mut self, view_idx: usize) {
        let view = &self.views[view_idx];
        
        // 设置视口
        self.set_viewport(
            view.view.viewport.x as u32,
            view.view.viewport.y as u32,
            view.view.viewport.width as u32,
            view.view.viewport.height as u32,
        );
        
        // 清除
        self.clear_with_config(&view.clear);
        
        // 渲染所有阶段
        self.render_phase(RenderPhaseType::Background);
        self.render_phase(RenderPhaseType::Opaque3d);
        self.render_phase(RenderPhaseType::Opaque2d);
        self.render_phase(RenderPhaseType::AlphaMask);
        self.render_phase(RenderPhaseType::Transparent);
        self.render_phase(RenderPhaseType::Ui);
    }
}
```

## 5. Pipeline 渲染流程

### 5.1 完整渲染管线

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FHRE 完整渲染管线                                    │
│                                                                             │
│  Main World                    Extract                    Render World      │
│  ─────────                    ────────                    ────────────      │
│                                                                             │
│  ┌─────────────┐             ┌─────────────┐             ┌─────────────┐   │
│  │ Camera Node │────────────▶│ Extract     │────────────▶│ ViewBundle  │   │
│  │ Transform3D │             │ Camera      │             │ - View      │   │
│  │ Camera3D    │             │             │             │ - Target    │   │
│  └─────────────┘             └─────────────┘             └─────────────┘   │
│                                                                             │
│  ┌─────────────┐             ┌─────────────┐             ┌─────────────┐   │
│  │ Sprite Node │────────────▶│ Extract     │────────────▶│ PhaseItem   │   │
│  │ Transform3D │             │ Objects     │             │ - Opaque3d  │   │
│  │ (z=0)       │             │             │             │ - Sort by z │   │
│  └─────────────┘             └─────────────┘             └─────────────┘   │
│                                                                             │
│  ┌─────────────┐             ┌─────────────┐             ┌─────────────┐   │
│  │ Model Node  │────────────▶│ Extract     │────────────▶│ PhaseItem   │   │
│  │ Transform3D │             │ Objects     │             │ - Opaque3d  │   │
│  │ (z=50)      │             │             │             │ - Sort by z │   │
│  └─────────────┘             └─────────────┘             └─────────────┘   │
│                                                                             │
│                                    │                                        │
│                                    ▼                                        │
│                           ┌─────────────────┐                               │
│                           │   Pipeline      │                               │
│                           │   (batch.rs)    │                               │
│                           │                 │                               │
│                           │ ┌─────────────┐ │                               │
│                           │ │ GpuTask     │ │                               │
│                           │ │ Collector   │ │                               │
│                           │ │ - Batch     │ │                               │
│                           │ │   similar   │ │                               │
│                           │ │   draws     │ │                               │
│                           │ └─────────────┘ │                               │
│                           │                 │                               │
│                           │ ┌─────────────┐ │                               │
│                           │ │ Hybrid      │ │                               │
│                           │ │ Scheduler   │ │                               │
│                           │ │ - GPU tasks │ │                               │
│                           │ │ - CPU tasks │ │                               │
│                           │ └─────────────┘ │                               │
│                           └─────────────────┘                               │
│                                    │                                        │
│                                    ▼                                        │
│                           ┌─────────────────┐                               │
│                           │  Framebuffer    │                               │
│                           │  Output         │                               │
│                           └─────────────────┘                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 渲染阶段与视图的配合

```rust
// 每个视图独立渲染所有阶段
for view in &render_world.views {
    // 设置视口
    set_viewport(view.viewport);
    
    // 渲染阶段（按顺序）
    for phase in [Background, Opaque3d, Opaque2d, AlphaMask, Transparent, Ui] {
        // 获取该视图、该阶段的可见对象
        let items = collect_visible_items(view, phase);
        
        // 排序（透明阶段需要）
        if phase == Transparent {
            items.sort_by_z_depth();
        }
        
        // 批处理
        let batches = build_batches(items);
        
        // 提交到 Pipeline
        for batch in batches {
            pipeline.submit(batch);
        }
    }
}
```

## 6. 摄像机控制与动画

### 6.1 第一人称/第三人称视角控制

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
        let right = transform.right();
        
        if input.key_pressed(Key::W) {
            transform.position += forward * MOVE_SPEED * time.delta();
        }
        if input.key_pressed(Key::S) {
            transform.position -= forward * MOVE_SPEED * time.delta();
        }
        // ... A, D 同理
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

// 轨道摄像机（围绕目标旋转）
fn orbit_camera(
    mut camera_query: Query<(&mut Transform3D, &mut OrbitCamera)>,
    input: Res<Input>,
) {
    for (mut transform, mut orbit) in camera_query.iter_mut() {
        // 更新轨道角度
        orbit.azimuth += input.mouse_delta.x * ORBIT_SPEED;
        orbit.elevation += input.mouse_delta.y * ORBIT_SPEED;
        orbit.elevation = orbit.elevation.clamp(10.0_f32.to_radians(), 80.0_f32.to_radians());
        
        // 计算新位置
        let radius = orbit.distance;
        let x = orbit.target.x + radius * orbit.elevation.cos() * orbit.azimuth.sin();
        let y = orbit.target.y + radius * orbit.elevation.sin();
        let z = orbit.target.z + radius * orbit.elevation.cos() * orbit.azimuth.cos();
        
        transform.position = Vec3::new(x, y, z);
        transform.look_at(orbit.target);
    }
}
```

### 6.2 摄像机视角变换动画

```rust
// 摄像机切换动画（如从游戏视角切换到地图视角）
fn camera_transition(
    mut commands: Commands,
    mut camera_query: Query<(Entity, &mut Transform3D, &mut Camera3D)>,
    mut transition: ResMut<CameraTransition>,
    time: Res<Time>,
) {
    if let CameraTransitionState::Transitioning { from, to, progress } = transition.state {
        // 更新进度
        let new_progress = (progress + time.delta() / transition.duration).min(1.0);
        transition.state = CameraTransitionState::Transitioning { 
            from, to, progress: new_progress 
        };
        
        // 插值位置和旋转
        for (_, mut transform, _) in camera_query.iter_mut() {
            transform.position = from.position.lerp(to.position, ease_in_out_cubic(new_progress));
            transform.rotation = from.rotation.slerp(to.rotation, new_progress);
        }
        
        // 完成过渡
        if new_progress >= 1.0 {
            transition.state = CameraTransitionState::Complete;
        }
    }
}

// 震动效果（受伤、爆炸等）
fn camera_shake(
    mut camera_query: Query<&mut Transform3D, With<MainCamera>>,
    mut shake: ResMut<CameraShake>,
    time: Res<Time>,
) {
    if shake.intensity > 0.0 {
        // 衰减
        shake.intensity *= shake.decay;
        
        if shake.intensity < 0.01 {
            shake.intensity = 0.0;
            return;
        }
        
        // 应用随机偏移
        for mut transform in camera_query.iter_mut() {
            let offset = Vec3::new(
                random_range(-shake.intensity, shake.intensity),
                random_range(-shake.intensity, shake.intensity),
                0.0,
            );
            transform.position += offset;
        }
    }
}

// 过肩视角（TPS 游戏）
fn over_shoulder_camera(
    mut camera_query: Query<&mut Transform3D, With<ShoulderCamera>>,
    player_query: Query<&Transform3D, With<Player>>,
    input: Res<Input>,
) {
    let player = player_query.single();
    
    for mut camera in camera_query.iter_mut() {
        // 基础位置：玩家右肩后方
        let shoulder_offset = player.right() * 0.5 + player.up() * 1.6 - player.forward() * 2.0;
        let base_pos = player.position + shoulder_offset;
        
        // 瞄准偏移（根据鼠标位置微调）
        let aim_offset = Vec3::new(
            input.mouse_normalized.x * 0.3,
            input.mouse_normalized.y * 0.2,
            0.0,
        );
        
        camera.position = base_pos + aim_offset;
        camera.look_at(player.position + player.forward() * 10.0);
    }
}
```

### 6.3 多摄像机配合场景

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
            .with_clear_config(ClearConfig::none()), // 透明背景
        MinimapCamera,
    ));
    
    // 后视镜
    commands.spawn((
        Node::ui_control(NodeType::Camera),
        Transform3D::from_position(0.0, 1.5, -0.5) // 车后上方
            .with_rotation(0.0, 180.0_f32.to_radians(), 0.0), // 朝向后方
        Camera3D::new()
            .with_fov(60.0)
            .with_viewport(Rect::new(0.35, 0.85, 0.3, 0.12)) // 顶部中央小窗口
            .with_depth(2),
        RearviewCamera,
    ));
}

// 每帧更新所有摄像机
fn update_racing_cameras(
    car_query: Query<&Transform3D, With<Car>>,
    mut driver_cam: Query<&mut Transform3D, With<DriverCamera>>,
    mut minimap_cam: Query<&mut Transform3D, (With<MinimapCamera>, Without<DriverCamera>)>,
) {
    let car = car_query.single();
    
    // 驾驶视角跟随车辆
    for mut cam in driver_cam.iter_mut() {
        let offset = car.rotation * Vec3::new(0.0, 1.2, 0.5);
        cam.position = car.position + offset;
        cam.rotation = car.rotation; // 与车辆同向
    }
    
    // 小地图跟随车辆位置（保持俯视）
    for mut cam in minimap_cam.iter_mut() {
        cam.position.x = car.position.x;
        cam.position.z = car.position.z;
    }
}
```

## 7. 关键设计决策

### 7.1 为什么 Camera3D 在 Main World，View 在 Render World？

| 方面 | Camera3D (Main World) | View (Render World) |
|------|----------------------|---------------------|
| **性质** | 声明式组件 | 运行时结构 |
| **数据** | 属性配置（FOV、颜色等） | 计算结果（矩阵、视口等） |
| **生命周期** | 与实体共存 | 每帧重建 |
| **用途** | 游戏逻辑操作 | GPU 渲染使用 |

### 6.2 2D/3D 统一处理的优势

1. **简化架构**：不需要两套渲染管线
2. **支持 2.5D 效果**：UI 卡片翻转、等角视角等
3. **无缝过渡**：2D 精灵可以平滑进入 3D 空间
4. **统一排序**：所有对象按 z 深度排序
5. **完整 3D 摄像机控制**：支持 FPS、TPS、轨道视角等

### 7.2 多视图的使用场景

```rust
// 场景 1: 主游戏 + 小地图
let main_view = ViewBundle::new_screen(800.0, 600.0)
    .with_camera(main_camera);
    
let minimap_view = ViewBundle::new_screen(200.0, 200.0)
    .with_camera(minimap_camera)
    .with_viewport(Rect::new(600.0, 0.0, 200.0, 200.0));

// 场景 2: 分屏游戏
let left_view = ViewBundle::new_screen(400.0, 600.0)
    .with_camera(player1_camera);
    
let right_view = ViewBundle::new_screen(400.0, 600.0)
    .with_camera(player2_camera)
    .with_viewport(Rect::new(400.0, 0.0, 400.0, 600.0));

// 场景 3: UI 覆盖层
let game_view = ViewBundle::new_screen(800.0, 600.0)
    .with_camera(game_camera);
    
let ui_view = ViewBundle::new_screen(800.0, 600.0)
    .with_camera(ui_camera)
    .with_clear_config(ClearConfig::none()); // 不清除，叠加渲染
```

## 8. 总结

### 架构关系图

```
Main World (声明式)                          Render World (运行时)
─────────────────                           ───────────────────
                                                                              
Node (Entity)                                    ViewBundle
├── NodeType::Camera  ─────Extract──────▶        ├── View (矩阵计算结果)
├── Transform3D (位置)                           ├── ViewTarget
└── Camera3D (属性)                              └── ClearConfig
       │                                              ▲
       │    ┌─────────────────────────────────────────┘
       │    │
       └───▶│  1. 读取 Transform3D.position → camera_position
            │  2. 读取 Camera3D.fov/orthographic → projection
            │  3. 计算 view_matrix (基于位置和朝向)
            │  4. 计算 view_projection = projection × view
            │  5. 读取 Camera3D.viewport → viewport (像素)
            │
Node (Entity)                                    PhaseItem
├── NodeType::Sprite2D  ───Extract──────▶        ├── sort_key (z_depth)
├── Transform3D (z=0)                            ├── draw_command_index
└── Sprite (颜色/尺寸)                           └── batch_key
       │                                              ▲
       └───▶ 1. 使用 View.world_to_screen() 计算屏幕位置
            2. 根据 opacity 确定 RenderPhaseType
            3. 创建 RenderCommand
            4. 创建 PhaseItem (引用 command)
```

### 关键要点

1. **Camera3D 是声明，View 是实现** - 两者不重复，是不同阶段的数据
2. **所有渲染都是 3D** - 2D 是 z=0 的特殊情况
3. **Extract 是转换桥梁** - 将声明式数据转换为运行时数据
4. **Pipeline 执行最终渲染** - 接收批处理后的绘制命令

---

**文档版本**: v1.0  
**日期**: 2026-04-15  
**作者**: FeatherOS Team
