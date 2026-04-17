#![no_std]
#![no_main]

//! FHRE 3D Demo
//!
//! 展示旋转立方体或足球（截角二十面体），底部有三个按钮
//! 通过 feature "cube" 选择绘制对象
//!
//! 架构说明：
//! - 使用 ECS (Entity-Component-System) 架构
//! - 动画通过 System 更新，不直接操作组件
//! - 使用 Resource 存储全局状态

extern crate alloc;

// Import libc functions for NuttX platform
extern "C" {
    fn printf(format: *const u8, ...) -> i32;
    fn usleep(usec: u32) -> i32;
}

// 宏定义：选择绘制对象
#[cfg(feature = "cube")]
const USE_CUBE: bool = true;
#[cfg(not(feature = "cube"))]
const USE_CUBE: bool = false;

// Import FHRE modules
#[cfg(not(feature = "cube"))]
use fhre::ui::SoccerBall;
#[cfg(feature = "cube")]
use fhre::ui::Cube;

use fhre::{
    App, FHRE_VERSION,
    node::{Node, NodeType, Transform2D, Transform3D},
    ui::Button,
    math::{Color, Vec3},
    resources::Time,
    main_world::{MainWorld, Entity},
};

use alloc::vec::Vec;

/// Demo 全局状态资源
/// 
/// 存储旋转状态，由 System 读取并应用到对象
#[derive(Clone, Debug)]
pub struct DemoState {
    /// 当前 Y 轴旋转角度
    pub rotation_y: f32,
    /// 是否正在旋转
    pub is_rotating: bool,
    /// 旋转速度（度/秒）
    pub rotation_speed: f32,
}

impl DemoState {
    pub fn new() -> Self {
        Self {
            rotation_y: 0.0,
            is_rotating: true,
            rotation_speed: 60.0, // 60度/秒 = 6秒/圈
        }
    }
}

impl Default for DemoState {
    fn default() -> Self {
        Self::new()
    }
}

impl fhre::resources::Resource for DemoState {}

/// Demo 应用
/// 
/// 简化的结构，动画逻辑完全交给 ECS System
struct DemoApp {
    app: App,
    object_entity: Option<fhre::main_world::Entity>,
}

impl DemoApp {
    /// 创建新的 demo 应用
    fn new() -> Option<Self> {
        unsafe {
            if USE_CUBE {
                printf(b"[DEBUG] Starting FHRE Cube Demo (ECS version)\n\0".as_ptr());
            } else {
                printf(b"[DEBUG] Starting FHRE Soccer Ball Demo (ECS version)\n\0".as_ptr());
            }
        }

        // 创建 FHRE App with X11 window for input (640x480)
        let mut app = App::new_with_x11_window(640, 480, "FHRE Demo");

        // 获取窗口尺寸
        let (width, height) = app.screen_dimensions();

        // 设置场景
        let object_entity = Self::setup_scene(&mut app, width, height);

        // 添加 DemoState 资源
        app.main_world.resources_mut().insert(DemoState::new());

        // 添加旋转系统到主世界
        app.main_world.add_system(rotation_system);

        unsafe {
            printf(b"[INFO] X11 window created: %dx%d\n\0".as_ptr(), width, height);
            printf(b"[INFO] ECS System registered for animation\n\0".as_ptr());
        }

        Some(Self {
            app,
            object_entity,
        })
    }

    /// 设置场景 - 3D 对象和按钮
    fn setup_scene(app: &mut App, width: u32, height: u32) -> Option<fhre::main_world::Entity> {
        unsafe {
            if USE_CUBE {
                printf(b"[DEBUG] Setting up cube scene...\n\0".as_ptr());
            } else {
                printf(b"[DEBUG] Setting up soccer ball scene...\n\0".as_ptr());
            }
        }

        // 创建 3D 对象
        let object_entity = app.main_world.spawn();
        app.main_world.insert_component(object_entity, Node::game_entity(NodeType::Empty));

        // 对象位置在屏幕中央偏上
        let obj_x = width as f32 / 2.0;
        let obj_y = height as f32 / 3.0;
        let obj_z = 0.0;

        app.main_world.insert_component(object_entity, Transform3D::from_position(obj_x, obj_y, obj_z));

        #[cfg(feature = "cube")]
        {
            // 创建立方体组件 - 基于顶角立起来的旋转
            // 初始旋转 45 度 around X 和 Z 轴，让一个顶角朝下
            let cube = Cube::new(120.0)
                .with_face_colors([
                    Color::rgb(255, 100, 100), // Front - red
                    Color::rgb(100, 255, 100), // Back - green
                    Color::rgb(100, 100, 255), // Top - blue
                    Color::rgb(255, 255, 100), // Bottom - yellow
                    Color::rgb(255, 255, 255), // Left - white
                    Color::rgb(100, 255, 255), // Right - cyan
                ])
                .with_rotation(Vec3::new(45.0, 0.0, 45.0)) // 基于顶角立起来
                .with_wireframe(true, Color::WHITE);

            app.main_world.insert_component(object_entity, cube);
        }

        #[cfg(not(feature = "cube"))]
        {
            // 创建足球组件（截角二十面体：12个五边形 + 20个六边形）
            let soccer_ball = SoccerBall::new(120.0)
                .with_rotation(Vec3::new(0.0, 0.0, 0.0))
                .with_wireframe(true, Color::WHITE);

            app.main_world.insert_component(object_entity, soccer_ball);
        }

        // 底部三个按钮
        let button_y = height as f32 - 80.0;
        let button_spacing = 140.0;
        let center_x = width as f32 / 2.0;

        // 按钮 1 - 重置旋转
        let btn1 = app.main_world.spawn();
        app.main_world.insert_component(btn1, Node::ui_control(NodeType::Button));
        app.main_world.insert_component(btn1, Transform2D::from_position(center_x - button_spacing, button_y));
        app.main_world.insert_component(btn1, Button::new(100.0, 40.0)
            .with_text("Reset")
            .with_colors(
                Color::rgb(70, 130, 180),   // normal - steel blue
                Color::rgb(100, 160, 210),  // hover
                Color::rgb(50, 100, 150),   // pressed
            ));

        // 按钮 2 - 暂停/继续
        let btn2 = app.main_world.spawn();
        app.main_world.insert_component(btn2, Node::ui_control(NodeType::Button));
        app.main_world.insert_component(btn2, Transform2D::from_position(center_x, button_y));
        app.main_world.insert_component(btn2, Button::new(100.0, 40.0)
            .with_text("Pause")
            .with_colors(
                Color::rgb(60, 150, 80),    // normal - green
                Color::rgb(90, 180, 110),   // hover
                Color::rgb(40, 120, 60),    // pressed
            ));

        // 按钮 3 - 退出
        let btn3 = app.main_world.spawn();
        app.main_world.insert_component(btn3, Node::ui_control(NodeType::Button));
        app.main_world.insert_component(btn3, Transform2D::from_position(center_x + button_spacing, button_y));
        app.main_world.insert_component(btn3, Button::new(100.0, 40.0)
            .with_text("Exit")
            .with_colors(
                Color::rgb(180, 70, 70),    // normal - red
                Color::rgb(210, 100, 100),  // hover
                Color::rgb(150, 50, 50),    // pressed
            ));

        unsafe {
            if USE_CUBE {
                printf(b"[DEBUG] Cube scene setup complete\n\0".as_ptr());
            } else {
                printf(b"[DEBUG] Soccer ball scene setup complete\n\0".as_ptr());
            }
        }

        Some(object_entity)
    }

    /// 运行 demo 主循环
    fn run(&mut self) {
        unsafe {
            printf(b"[INFO] FHRE Demo started - Entering main loop\n\0".as_ptr());
            printf(b"[INFO] Animation handled by ECS System\n\0".as_ptr());
            printf(b"[INFO] Press 'q' or click X button to exit\n\0".as_ptr());
        }

        // 主循环 - 所有动画逻辑都在 ECS System 中处理
        loop {
            // 检查是否应该退出
            if !self.app.is_running() {
                unsafe {
                    printf(b"[INFO] FHRE Demo exiting...\n\0".as_ptr());
                }
                break;
            }

            // 更新 FHRE App (包含 X11 事件轮询和 System 执行)
            self.app.update();

            // 控制帧率约 60 FPS
            unsafe {
                usleep(16_667); // 16.67ms = 60 FPS
            }
        }
    }
}

/// 旋转系统 - ECS System
/// 
/// 这个系统每帧自动执行，更新所有旋转对象的旋转角度
/// 符合 ECS 架构：System 读取 Resource (Time, DemoState)，修改 Component (Cube/SoccerBall)
fn rotation_system(world: &mut MainWorld) {
    // 获取时间和状态资源
    let (delta_time, is_rotating, rotation_speed) = {
        let time = world.resources().get::<Time>();
        let state = world.resources().get::<DemoState>();
        
        let dt = time.map(|t| t.delta()).unwrap_or(0.016);
        let rotating = state.map(|s| s.is_rotating).unwrap_or(true);
        let speed = state.map(|s| s.rotation_speed).unwrap_or(60.0);
        
        (dt, rotating, speed)
    };

    // 如果暂停，不更新旋转
    if !is_rotating {
        return;
    }

    // 计算旋转增量
    let rotation_delta = rotation_speed * delta_time;

    // 更新 DemoState 中的旋转角度
    if let Some(mut state) = world.resources_mut().get_mut::<DemoState>() {
        state.rotation_y += rotation_delta;
        if state.rotation_y >= 360.0 {
            state.rotation_y -= 360.0;
        }
    }

    // 获取当前旋转角度
    let current_rotation_y = world
        .resources()
        .get::<DemoState>()
        .map(|s| s.rotation_y)
        .unwrap_or(0.0);

    // 更新所有 Cube 组件
    #[cfg(feature = "cube")]
    {
        // 收集需要更新的实体和旋转值
        let updates: Vec<(u64, f32)> = {
            let mut updates = Vec::new();
            for (entity, cube) in world.query::<Cube>() {
                // 保持 X 和 Z 轴的初始旋转（基于顶角立起来）
                updates.push((entity.id(), current_rotation_y));
            }
            updates
        };

        // 应用更新
        for (id, rotation_y) in updates {
            if let Some(cube) = world.get_component_mut::<Cube>(Entity::new(id)) {
                cube.rotation.y = rotation_y;
                cube.rotation.x = 45.0; // 保持顶角朝下的姿态
                cube.rotation.z = 45.0;
            }
        }
    }

    // 更新所有 SoccerBall 组件
    #[cfg(not(feature = "cube"))]
    {
        // 收集需要更新的实体和旋转值
        let updates: Vec<(u64, f32)> = {
            let mut updates = Vec::new();
            for (entity, _) in world.query::<SoccerBall>() {
                updates.push((entity.id(), current_rotation_y));
            }
            updates
        };

        // 应用更新
        for (id, rotation_y) in updates {
            if let Some(soccer_ball) = world.get_component_mut::<SoccerBall>(Entity::new(id)) {
                soccer_ball.rotation.y = rotation_y;
            }
        }
    }
}

/// Demo 入口函数
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
        printf(b"  Architecture: ECS (Entity-Component-System)\n\0".as_ptr());
        printf(b"========================================\n\n\0".as_ptr());
    }

    match DemoApp::new() {
        Some(mut demo) => {
            demo.run();
            0
        }
        None => {
            unsafe {
                printf(b"[ERROR] Failed to create demo app\n\0".as_ptr());
            }
            -1
        }
    }
}

/// 模块初始化
#[no_mangle]
pub extern "C" fn rust_fhre_demo_init() {
    // 模块初始化代码
}
