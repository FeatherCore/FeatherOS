#![no_std]
#![no_main]

//! FHRE 立方体旋转 Demo
//!
//! 展示基于顶角立起来的旋转立方体，底部有三个按钮

extern crate alloc;

use alloc::vec::Vec;

// Import libc functions for NuttX platform
extern "C" {
    fn printf(format: *const u8, ...) -> i32;
    fn open(path: *const u8, flags: i32) -> i32;
    fn close(fd: i32) -> i32;
    fn ioctl(fd: i32, request: u32, ...) -> i32;
    fn mmap(
        addr: *mut core::ffi::c_void,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: isize,
    ) -> *mut core::ffi::c_void;
    fn munmap(addr: *mut core::ffi::c_void, length: usize) -> i32;
    fn usleep(usec: u32) -> i32;
    fn getpid() -> i32;
    fn __errno() -> *mut i32;
}

const O_RDWR: i32 = 2;
const PROT_READ: i32 = 1;
const PROT_WRITE: i32 = 2;
const MAP_SHARED: i32 = 1;

// Framebuffer IOCTL commands
const FBIOGET_VIDEOINFO: u32 = 0x2801;
const FBIOGET_PLANEINFO: u32 = 0x2802;

/// Framebuffer video info structure
#[repr(C)]
struct FbVideoInfo {
    fmt: u8,
    xres: u16,
    yres: u16,
    nplanes: u8,
}

/// Framebuffer plane info structure
#[repr(C)]
struct FbPlaneInfo {
    fbmem: *mut u8,
    fblen: usize,
    stride: u16,
    display: u8,
    bpp: u8,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
}

// Import FHRE modules
use fhre::{
    App, FHRE_VERSION,
    node::{Node, NodeType, Transform2D, Transform3D},
    ui::{Button, Cube},
    math::{Color, Vec3},
};

/// Demo 应用状态
struct DemoApp {
    app: App,
    fb_fd: i32,
    fb_mem: *mut u32,
    fb_size: usize,
    width: u32,
    height: u32,
    frame_count: u32,
    cube_entity: Option<fhre::main_world::Entity>,
    rotation_y: f32,
    is_rotating: bool,
}

impl DemoApp {
    /// 创建新的 demo 应用
    fn new() -> Option<Self> {
        unsafe {
            printf(b"[DEBUG] Starting FHRE Cube Demo\n\0".as_ptr());
        }

        // 初始化 framebuffer
        let (fb_fd, fb_mem, fb_size, width, height) = Self::init_framebuffer()?;

        // 创建 FHRE App
        let mut app = App::new();

        // 设置场景
        let cube_entity = Self::setup_scene(&mut app, width, height);

        Some(Self {
            app,
            fb_fd,
            fb_mem,
            fb_size,
            width,
            height,
            frame_count: 0,
            cube_entity,
            rotation_y: 0.0,
            is_rotating: true,
        })
    }

    /// 初始化 framebuffer
    fn init_framebuffer() -> Option<(i32, *mut u32, usize, u32, u32)> {
        unsafe {
            let fb_fd = open(b"/dev/fb0\0".as_ptr(), O_RDWR);
            if fb_fd < 0 {
                printf(b"[ERROR] Failed to open /dev/fb0\n\0".as_ptr());
                return None;
            }

            let mut vinfo: FbVideoInfo = core::mem::zeroed();
            if ioctl(fb_fd, FBIOGET_VIDEOINFO, &mut vinfo as *mut _) < 0 {
                printf(b"[ERROR] Failed to get video info\n\0".as_ptr());
                close(fb_fd);
                return None;
            }

            let mut pinfo: FbPlaneInfo = core::mem::zeroed();
            if ioctl(fb_fd, FBIOGET_PLANEINFO, &mut pinfo as *mut _) < 0 {
                printf(b"[ERROR] Failed to get plane info\n\0".as_ptr());
                close(fb_fd);
                return None;
            }

            let width = vinfo.xres as u32;
            let height = vinfo.yres as u32;
            let fb_size = (width * height * 4) as usize;

            printf(
                b"[INFO] Framebuffer: %dx%d, bpp=%d\n\0".as_ptr(),
                width, height, pinfo.bpp as u32,
            );

            let fb_mem = pinfo.fbmem as *mut u32;
            if fb_mem.is_null() {
                printf(b"[ERROR] Null framebuffer address\n\0".as_ptr());
                close(fb_fd);
                return None;
            }

            Some((fb_fd, fb_mem, fb_size, width, height))
        }
    }

    /// 设置场景 - 立方体和按钮
    fn setup_scene(app: &mut App, width: u32, height: u32) -> Option<fhre::main_world::Entity> {
        unsafe {
            printf(b"[DEBUG] Setting up scene...\n\0".as_ptr());
        }

        // 创建立方体 - 基于顶角立起来
        let cube_entity = app.main_world.spawn();
        app.main_world.insert_component(cube_entity, Node::game_entity(NodeType::Empty));
        
        // 立方体位置在屏幕中央偏上
        let cube_x = width as f32 / 2.0;
        let cube_y = height as f32 / 3.0;
        let cube_z = 0.0;
        
        app.main_world.insert_component(cube_entity, Transform3D::from_position(cube_x, cube_y, cube_z));
        
        // 创建立方体组件 - 基于顶角立起来的旋转
        // 初始旋转 45 度 around X 和 Z 轴，让一个顶角朝下
        let cube = Cube::new(120.0)
            .with_face_colors([
                Color::rgb(255, 100, 100), // Front - red
                Color::rgb(100, 255, 100), // Back - green
                Color::rgb(100, 100, 255), // Top - blue
                Color::rgb(255, 255, 100), // Bottom - yellow
                Color::rgb(255, 100, 255), // Left - magenta
                Color::rgb(100, 255, 255), // Right - cyan
            ])
            .with_rotation(Vec3::new(45.0, 0.0, 45.0)) // 基于顶角立起来
            .with_wireframe(true, Color::WHITE);
        
        app.main_world.insert_component(cube_entity, cube);

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
            printf(b"[DEBUG] Scene setup complete: cube + 3 buttons\n\0".as_ptr());
        }

        Some(cube_entity)
    }

    /// 更新立方体旋转
    fn update_cube(&mut self) {
        if !self.is_rotating {
            return;
        }

        // 每帧增加 Y 轴旋转
        self.rotation_y += 1.0;
        if self.rotation_y >= 360.0 {
            self.rotation_y = 0.0;
        }

        // 更新立方体的旋转
        if let Some(cube_entity) = self.cube_entity {
            if let Some(cube) = self.app.main_world.get_component_mut::<Cube>(cube_entity) {
                // 保持 X 和 Z 轴 45 度（顶角朝下），Y 轴持续旋转
                cube.rotation = Vec3::new(45.0, self.rotation_y, 45.0);
            }
        }
    }

    /// 渲染一帧
    fn render(&mut self) {
        self.frame_count += 1;

        // 更新立方体旋转
        self.update_cube();

        // 运行 FHRE 更新
        self.app.update();

        // 复制 framebuffer 到硬件
        let fb = self.app.get_framebuffer();
        let size = fb.len().min(self.fb_size / 4);

        unsafe {
            // 每 60 帧打印一次状态
            if self.frame_count % 60 == 0 {
                let mut non_zero = 0;
                for i in 0..fb.len() {
                    if fb[i] != 0 {
                        non_zero += 1;
                    }
                }
                printf(
                    b"[DEBUG] Frame %d: pixels=%d/%d, rotation=%.1f\n\0".as_ptr(),
                    self.frame_count, non_zero, fb.len(), self.rotation_y as f64
                );
            }

            core::ptr::copy_nonoverlapping(fb.as_ptr(), self.fb_mem, size);
        }
    }

    /// 运行主循环
    fn run(&mut self) {
        unsafe {
            printf(b"\n========================================\n\0".as_ptr());
            printf(b"FHRE Cube Demo\n\0".as_ptr());
            printf(b"Version: %s\n\0".as_ptr(), FHRE_VERSION.as_ptr());
            printf(b"========================================\n\0".as_ptr());
            printf(b"\nCube standing on corner, rotating...\n\0".as_ptr());
            printf(b"Bottom: [Reset] [Pause] [Exit]\n\n\0".as_ptr());
        }

        // 主循环
        loop {
            self.render();

            // 60 FPS 帧率控制
            unsafe {
                usleep(16_667);
            }
        }
    }
}

impl Drop for DemoApp {
    fn drop(&mut self) {
        unsafe {
            printf(b"[DEBUG] Cleaning up DemoApp\n\0".as_ptr());
            close(self.fb_fd);
        }
    }
}

/// 主入口点
#[no_mangle]
pub extern "C" fn fhre_rust_main(_argc: i32, _argv: *const *const u8) -> i32 {
    match DemoApp::new() {
        Some(mut demo) => {
            demo.run();
            0
        }
        None => {
            unsafe {
                printf(b"[ERROR] Failed to create demo application\n\0".as_ptr());
            }
            -1
        }
    }
}
