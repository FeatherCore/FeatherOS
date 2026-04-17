#![no_std]
#![no_main]

//! FHRE 3D Demo - 纯声明式 ECS 版本
//!
//! 展示旋转立方体或足球（截角二十面体），底部有三个按钮
//! 通过 feature "cube" 选择绘制对象
//!
//! 架构说明：
//! - 使用纯声明式 ECS (Entity-Component-System) 架构，完全对齐 Bevy 风格
//! - 动画通过 System 更新，使用 Res/ResMut/Query 声明式参数
//! - 使用 Resource 存储全局状态
//! - 使用 Commands 创建实体（非直接 World 操作）
//! - ECS 系统使用纯声明式参数（Res/ResMut/Query）

extern crate alloc;

// Import libc functions for NuttX platform
extern "C" {
    fn printf(format: *const u8, ...) -> i32;
    fn clock() -> i64;
    fn usleep(usec: u32) -> i32;
}

// X11 Window module (SIM platform only)
#[cfg(feature = "sim")]
mod x11_window;

// 宏定义：选择绘制对象
#[cfg(feature = "cube")]
const USE_CUBE: bool = true;
#[cfg(not(feature = "cube"))]
const USE_CUBE: bool = false;

// Import FHRE modules
use fhre::ui::{SoccerBall, Cube};

use fhre::{
    App, FHRE_VERSION, AppExit,
    node::{Node, NodeType, Transform2D, Transform3D},
    ui::Button,
    math::{Color, Vec3},
    resources::{Time, PrimaryScreen},
    // 声明式 ECS
    Res, ResMut, Query, Commands,
    // 输入系统
    ButtonInput, MouseButton, KeyCode,
    // Plugin系统
    Plugin,
    // Schedule系统 - Bevy风格
    Update, PreUpdate, Startup,
    // 默认摄像机资源
    DefaultUiCamera,
    // System 辅助函数
    system2, system3,
};

/// Demo 全局状态资源
#[derive(Clone, Debug)]
pub struct DemoState {
    pub rotation_y: f32,
    pub is_rotating: bool,
    pub rotation_speed: f32,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            rotation_y: 0.0,
            is_rotating: true,
            rotation_speed: 60.0,
        }
    }
}

impl Default for DemoState {
    fn default() -> Self {
        Self::new()
    }
}

impl fhre::resources::Resource for DemoState {}

/// Screen Plugin - 注册主屏幕资源
#[derive(Default)]
pub struct ScreenPlugin {
    width: u32,
    height: u32,
}

impl ScreenPlugin {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        let primary_screen = PrimaryScreen::new(self.width, self.height);
        app.main_world.resources_mut().insert(primary_screen);
        unsafe {
            printf(b"[INFO] ScreenPlugin: Registered PrimaryScreen %dx%d\n\0".as_ptr(),
                   self.width, self.height);
        }
    }

    fn name(&self) -> &str {
        "fhre_demo::ScreenPlugin"
    }
}

/// Camera Plugin - 注册默认UI摄像机
#[derive(Default)]
pub struct CameraPlugin {
    width: u32,
    height: u32,
}

impl CameraPlugin {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        let camera_entity = app.main_world.spawn();
        app.main_world.insert_component(camera_entity, Node::ui_control(NodeType::Camera));

        let camera_x = self.width as f32 / 2.0;
        let camera_y = self.height as f32 / 2.0;
        let camera_z = 400.0;

        app.main_world.insert_component(camera_entity, Transform3D::from_position(camera_x, camera_y, camera_z));
        // Note: Camera3D component temporarily removed

        app.main_world.resources_mut().insert(DefaultUiCamera { entity: camera_entity });
        unsafe {
            printf(b"[INFO] CameraPlugin: Registered DefaultUiCamera\n\0".as_ptr());
        }
    }

    fn name(&self) -> &str {
        "fhre_demo::CameraPlugin"
    }
}

/// Setup Plugin - 使用 Commands 设置场景
#[derive(Default)]
pub struct SetupPlugin {
    width: u32,
    height: u32,
}

impl SetupPlugin {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        unsafe {
            printf(b"[SETUP_PLUGIN] Building setup plugin\n\0".as_ptr());
        }
        
        // Use declarative ECS system with Commands to setup scene
        app.add_systems(Startup, system2::<
            Commands,
            Res<PrimaryScreen>,
            _,
        >(setup_scene_system));
        
        unsafe {
            printf(b"[SETUP_PLUGIN] Registered setup_scene_system in Startup schedule\n\0".as_ptr());
        }
    }

    fn name(&self) -> &str {
        "fhre_demo::SetupPlugin"
    }
}

/// 场景设置系统 - 使用 Commands 创建实体
/// 
/// 这是纯声明式 ECS 风格，不直接操作 World
fn setup_scene_system(
    mut commands: Commands,
    screen: Res<PrimaryScreen>,
) {
    unsafe {
        printf(b"[SETUP_SYSTEM] Running setup_scene_system\n\0".as_ptr());
    }
    
    let (width, height) = screen.dimensions();
    
    // 创建 3D 对象 - 使用 Commands.spawn()
    let obj_x = width as f32 / 2.0;
    let obj_y = height as f32 / 3.0;
    let obj_z = 0.0;

    #[cfg(feature = "cube")]
    {
        let cube = Cube::new(120.0)
            .with_face_colors([
                Color::rgb(255, 100, 100),
                Color::rgb(100, 255, 100),
                Color::rgb(100, 100, 255),
                Color::rgb(255, 255, 100),
                Color::rgb(255, 255, 255),
                Color::rgb(100, 255, 255),
            ])
            .with_rotation(Vec3::new(45.0, 0.0, 45.0))
            .with_wireframe(true, Color::WHITE);
        
        commands.spawn()
            .insert(Node::game_entity(NodeType::Empty))
            .insert(Transform3D::from_position(obj_x, obj_y, obj_z))
            .insert(cube);
        
        unsafe {
            printf(b"[SETUP_SYSTEM] Spawned cube entity\n\0".as_ptr());
        }
    }

    #[cfg(not(feature = "cube"))]
    {
        let soccer_ball = SoccerBall::new(120.0)
            .with_rotation(Vec3::new(0.0, 0.0, 0.0))
            .with_wireframe(true, Color::WHITE);
        
        commands.spawn()
            .insert(Node::game_entity(NodeType::Empty))
            .insert(Transform3D::from_position(obj_x, obj_y, obj_z))
            .insert(soccer_ball);
        
        unsafe {
            printf(b"[SETUP_SYSTEM] Spawned soccer ball entity\n\0".as_ptr());
        }
    }

    // 底部三个按钮 - 使用 Commands.spawn()
    let button_y = height as f32 - 80.0;
    let button_spacing = 140.0;
    let center_x = width as f32 / 2.0;

    // 按钮 1 - 重置旋转
    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform2D::from_position(center_x - button_spacing, button_y))
        .insert(Button::new(100.0, 40.0)
            .with_text("Reset")
            .with_colors(
                Color::rgb(70, 130, 180),
                Color::rgb(100, 160, 210),
                Color::rgb(50, 100, 150),
            ));

    // 按钮 2 - 暂停/继续
    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform2D::from_position(center_x, button_y))
        .insert(Button::new(100.0, 40.0)
            .with_text("Pause")
            .with_colors(
                Color::rgb(60, 150, 80),
                Color::rgb(90, 180, 110),
                Color::rgb(40, 120, 60),
            ));

    // 按钮 3 - 退出
    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform2D::from_position(center_x + button_spacing, button_y))
        .insert(Button::new(100.0, 40.0)
            .with_text("Exit")
            .with_colors(
                Color::rgb(180, 70, 70),
                Color::rgb(210, 100, 100),
                Color::rgb(150, 50, 50),
            ));

    unsafe {
        printf(b"[SETUP_SYSTEM] Scene setup complete\n\0".as_ptr());
    }
}

/// Demo Plugin - 包含所有Demo相关的系统和资源
#[derive(Default)]
pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.insert_resource(DemoState::new());
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.insert_resource(ButtonInput::<MouseButton>::default());
        
        // Register input handling system in PreUpdate schedule
        app.add_systems(PreUpdate, system3::<
            Res<ButtonInput<KeyCode>>,
            Res<ButtonInput<MouseButton>>,
            ResMut<DemoState>,
            _,
        >(input_system));
        
        // Register rotation system in Update schedule
        #[cfg(feature = "cube")]
        app.add_systems(Update, system3::<
            Res<Time>,
            ResMut<DemoState>,
            Query<Cube>,
            _,
        >(rotation_system));
        
        #[cfg(not(feature = "cube"))]
        app.add_systems(Update, system3::<
            Res<Time>,
            ResMut<DemoState>,
            Query<SoccerBall>,
            _,
        >(rotation_system));
        
        unsafe {
            printf(b"[DemoPlugin] Systems registered with add_systems\n\0".as_ptr());
        }
    }

    fn name(&self) -> &str {
        "fhre_demo::DemoPlugin"
    }
}

// Input handling system - declarative ECS style
fn input_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<DemoState>,
) {
    // Handle Space key - pause/resume
    if key_input.just_pressed(KeyCode::Space) {
        state.is_rotating = !state.is_rotating;
        unsafe {
            if state.is_rotating {
                printf(b"[INPUT] Resumed rotation\n\0".as_ptr());
            } else {
                printf(b"[INPUT] Paused rotation\n\0".as_ptr());
            }
        }
    }

    // Handle R key - reset
    if key_input.just_pressed(KeyCode::KeyR) {
        state.rotation_y = 0.0;
        unsafe {
            printf(b"[INPUT] Reset rotation\n\0".as_ptr());
        }
    }

    // Handle left mouse button
    if mouse_input.just_pressed(MouseButton::Left) {
        unsafe {
            printf(b"[INPUT] Left mouse clicked\n\0".as_ptr());
        }
    }
}

/// Rotation system - declarative ECS style
/// 
/// Uses Res/ResMut/Query declarative parameters, pure Bevy style
#[cfg(feature = "cube")]
fn rotation_system(
    time: Res<Time>,
    mut state: ResMut<DemoState>,
    mut cubes: Query<Cube>,
) {
    if !state.is_rotating {
        return;
    }

    let delta_time = time.delta();
    let rotation_delta = state.rotation_speed * delta_time;

    // Update global rotation state
    state.rotation_y += rotation_delta;
    if state.rotation_y >= 360.0 {
        state.rotation_y -= 360.0;
    }

    // Use declarative Query to update all Cube components
    for cube in cubes.iter_mut() {
        cube.rotation.y = state.rotation_y;
        cube.rotation.x = 45.0;
        cube.rotation.z = 45.0;
    }
}

/// Rotation system - declarative ECS style (SoccerBall version)
#[cfg(not(feature = "cube"))]
fn rotation_system(
    time: Res<Time>,
    mut state: ResMut<DemoState>,
    mut balls: Query<SoccerBall>,
) {
    if !state.is_rotating {
        return;
    }

    let delta_time = time.delta();
    let rotation_delta = state.rotation_speed * delta_time;

    // Update global rotation state
    state.rotation_y += rotation_delta;
    if state.rotation_y >= 360.0 {
        state.rotation_y -= 360.0;
    }

    // Use declarative Query to update all SoccerBall components
    for ball in balls.iter_mut() {
        ball.rotation.y = state.rotation_y;
    }
}

/// Demo 入口函数 - 纯声明式 ECS
#[no_mangle]
pub extern "C" fn fhre_rust_main() -> i32 {
    unsafe {
        printf(b"\n========================================\n\0".as_ptr());
        printf(b"  FHRE %s Demo\n\0".as_ptr(), FHRE_VERSION.as_ptr());
        if USE_CUBE {
            printf(b"  Mode: Cube (6 faces)\n\0".as_ptr());
        } else {
            printf(b"  Mode: Soccer Ball (12 pentagons + 20 hexagons)\n\0".as_ptr());
        }
        printf(b"  Architecture: Pure Declarative ECS\n\0".as_ptr());
        printf(b"========================================\n\n\0".as_ptr());
    }

    // 创建 App
    let mut app = App::new(640, 480);
    
    // 添加 Plugins
    app.add_plugin(ScreenPlugin::new(640, 480))
        .add_plugin(CameraPlugin::new(640, 480))
        .add_plugin(SetupPlugin::new(640, 480))
        .add_plugin(DemoPlugin);

    // SIM 平台：创建 X11 窗口并手动控制循环
    #[cfg(feature = "sim")]
    {
        // 创建 X11 窗口
        let mut window = match x11_window::X11Window::new(640, 480, "FHRE Demo") {
            Some(w) => w,
            None => {
                unsafe { printf(b"[ERROR] Failed to create X11 window\n\0".as_ptr()); }
                return 1;
            }
        };

        unsafe { printf(b"[MAIN] Starting main loop with X11 window\n\0".as_ptr()); }

        // 测试：运行一帧看看是否有输出
        unsafe { printf(b"[MAIN] Testing first frame...\n\0".as_ptr()); }
        app.update_and_render();
        let fb = app.framebuffer();
        unsafe { printf(b"[MAIN] First frame done, fb len=%d\n\0".as_ptr(), fb.len()); }

        // 主循环 - 平台适配层（非 ECS 逻辑）
        let mut frame_count = 0u32;
        loop {
            // 收集输入事件（这会处理 X11 事件，包括关闭事件）
            let events = window.collect_input_events();

            // 检查窗口是否关闭
            if !window.is_running() {
                unsafe { printf(b"[MAIN] Window closed, exiting...\n\0".as_ptr()); }
                break;
            }

            // 将 X11 事件转换为 FHRE ECS 输入资源
            // 在 PreUpdate 之前更新输入状态
            if let Some(mut key_input) = app.main_world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
                key_input.clear();
                for event in &events.keyboard_events {
                    if let Some(keycode) = x11_keycode_to_fhre(event.keycode) {
                        if event.pressed {
                            key_input.press(keycode);
                        } else {
                            key_input.release(keycode);
                        }
                    }
                }
            }

            if let Some(mut mouse_input) = app.main_world.resources_mut().get_mut::<ButtonInput<MouseButton>>() {
                mouse_input.clear();
                for event in &events.mouse_button_events {
                    if let Some(button) = x11_button_to_fhre(event.button) {
                        if event.pressed {
                            mouse_input.press(button);
                        } else {
                            mouse_input.release(button);
                        }
                    }
                }
            }

            // 更新 FHRE（运行 ECS Systems + 渲染）
            app.update_and_render();

            // 获取 framebuffer 并显示
            let framebuffer = app.framebuffer();
            let fb_len = framebuffer.len();
            
            unsafe {
                printf(b"[MAIN] Frame %d: framebuffer len=%d\n\0".as_ptr(),
                       frame_count, fb_len);
            }
            
            window.present(framebuffer);
            
            frame_count += 1;
            if frame_count > 100 {
                frame_count = 0; // 防止溢出
            }

            // 帧率控制（60 FPS）
            unsafe { usleep(16_000); } // 16ms = ~60 FPS
        }

        unsafe { printf(b"[MAIN] Main loop ended\n\0".as_ptr()); }
        return 0;
    }

    // NuttX 平台：使用默认行为
    #[cfg(not(feature = "sim"))]
    {
        app.run();
        return 0;
    }
}

/// X11 键码转换为 FHRE KeyCode
#[cfg(feature = "sim")]
fn x11_keycode_to_fhre(x11_keycode: u32) -> Option<KeyCode> {
    // 常见 X11 键码映射
    // 注意：这是简化的映射，实际应该使用完整的 X11 键码表
    match x11_keycode {
        65 => Some(KeyCode::Space),      // XK_space
        27 => Some(KeyCode::KeyR),       // XK_r
        9 => Some(KeyCode::Escape),      // XK_Escape
        111 => Some(KeyCode::ArrowUp),   // XK_Up
        116 => Some(KeyCode::ArrowDown), // XK_Down
        113 => Some(KeyCode::ArrowLeft), // XK_Left
        114 => Some(KeyCode::ArrowRight),// XK_Right
        // 添加更多键码映射...
        _ => None,
    }
}

/// X11 鼠标按钮转换为 FHRE MouseButton
#[cfg(feature = "sim")]
fn x11_button_to_fhre(x11_button: u32) -> Option<MouseButton> {
    match x11_button {
        1 => Some(MouseButton::Left),
        2 => Some(MouseButton::Middle),
        3 => Some(MouseButton::Right),
        _ => None,
    }
}

/// 模块初始化
#[no_mangle]
pub extern "C" fn rust_fhre_demo_init() {
    // 模块初始化代码
}
