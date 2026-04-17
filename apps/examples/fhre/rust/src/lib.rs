#![no_std]
#![no_main]

//! FHRE 3D Demo
//!
//! 展示旋转立方体或足球（截角二十面体），底部有三个按钮
//! 通过宏 FEATURE_CUBE 选择绘制对象

extern crate alloc;

// Import libc functions for NuttX platform
extern "C" {
    fn printf(format: *const u8, ...) -> i32;
    fn usleep(usec: u32) -> i32;
}

// 宏定义：选择绘制对象
// 默认绘制足球，定义 FEATURE_CUBE 则绘制立方体
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
};

/// Demo 应用状态
struct DemoApp {
    app: App,
    width: u32,
    height: u32,
    object_entity: Option<fhre::main_world::Entity>,
    rotation_y: f32,
    is_rotating: bool,
}

impl DemoApp {
    /// 创建新的 demo 应用
    fn new() -> Option<Self> {
        unsafe {
            if USE_CUBE {
                printf(b"[DEBUG] Starting FHRE Cube Demo\n\0".as_ptr());
            } else {
                printf(b"[DEBUG] Starting FHRE Soccer Ball Demo\n\0".as_ptr());
            }
        }

        // 创建 FHRE App with X11 window for input (640x480)
        let mut app = App::new_with_x11_window(640, 480, "FHRE Demo");

        // 获取窗口尺寸
        let (width, height) = app.screen_dimensions();

        // 设置场景
        let object_entity = Self::setup_scene(&mut app, width, height);

        unsafe {
            printf(b"[INFO] X11 window created: %dx%d\n\0".as_ptr(), width, height);
        }

        Some(Self {
            app,
            width,
            height,
            object_entity,
            rotation_y: 0.0,
            is_rotating: true,
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

        // 按钮 1 - 重置
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

    /// 更新对象旋转
    fn update_object(&mut self) {
        if !self.is_rotating {
            return;
        }

        // 每帧增加 Y 轴旋转
        self.rotation_y += 1.0;
        if self.rotation_y >= 360.0 {
            self.rotation_y = 0.0;
        }

        // 更新对象的旋转
        if let Some(entity) = self.object_entity {
            #[cfg(feature = "cube")]
            {
                if let Some(cube) = self.app.main_world.get_component_mut::<Cube>(entity) {
                    cube.rotation.y = self.rotation_y;
                    // 保持 X 和 Z 轴的初始旋转（基于顶角立起来）
                    cube.rotation.x = 45.0;
                    cube.rotation.z = 45.0;
                }
            }

            #[cfg(not(feature = "cube"))]
            {
                if let Some(soccer_ball) = self.app.main_world.get_component_mut::<SoccerBall>(entity) {
                    soccer_ball.rotation.y = self.rotation_y;
                }
            }
        }
    }

    /// 运行 demo 主循环
    fn run(&mut self) {
        unsafe {
            printf(b"[INFO] FHRE Demo started - Entering main loop\n\0".as_ptr());
            printf(b"[INFO] X11 window should be visible now\n\0".as_ptr());
            printf(b"[INFO] Press 'q' or click X button to exit\n\0".as_ptr());
        }

        // 在后台线程运行渲染循环
        loop {
            // 检查是否应该退出
            if !self.app.is_running() {
                unsafe {
                    printf(b"[INFO] FHRE Demo exiting...\n\0".as_ptr());
                }
                break;
            }

            // 更新对象旋转
            self.update_object();

            // 更新 FHRE App (包含 X11 事件轮询)
            self.app.update();

            // 控制帧率约 60 FPS
            unsafe {
                usleep(16_667); // 16.67ms = 60 FPS
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
