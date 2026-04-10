#![no_std]
#![no_main]

//! FHRE Rust language demo
//! 
//! Demonstrates the basic usage of Feather Hybrid Render Engine

use fhre::{Context, Color, Vec2, RenderMode, version};

extern "C" {
    pub fn printf(format: *const u8, ...) -> i32;
    pub fn open(pathname: *const u8, flags: i32) -> i32;
    pub fn close(fd: i32) -> i32;
    pub fn write(fd: i32, buf: *const core::ffi::c_void, count: usize) -> isize;
}

// Framebuffer definitions
const O_RDWR: i32 = 2;

#[no_mangle]
pub extern "C" fn fhre_rust_main(_argc: i32, _argv: *const *const u8) -> i32 {
    unsafe {
        printf(b"FHRE Rust language demo\n\0" as *const u8);
        printf(b"Version: %s\n\0" as *const u8, version().as_ptr() as *const u8);
        
        // Initialize FHRE context with 640x480 resolution and 2D mode
        let mut ctx = Context::new(640, 480, RenderMode::Mode2D);
        printf(b"FHRE context initialized: 640x480, mode: Mode2D\n\0" as *const u8);
        
        // Clear framebuffer with black color
        let black = Color::new(0, 0, 0, 255);
        ctx.clear(black);
        printf(b"Framebuffer cleared with black\n\0" as *const u8);
        
        // Draw a red point
        let red = Color::new(255, 0, 0, 255);
        let point = Vec2::new(100.0, 100.0);
        ctx.draw_point(point, red);
        printf(b"Drew red point at (100, 100)\n\0" as *const u8);
        
        // Draw a green line
        let green = Color::new(0, 255, 0, 255);
        let line_start = Vec2::new(50.0, 200.0);
        let line_end = Vec2::new(200.0, 350.0);
        ctx.draw_line(line_start, line_end, green);
        printf(b"Drew green line from (50, 200) to (200, 350)\n\0" as *const u8);
        
        // Draw a blue rectangle
        let blue = Color::new(0, 0, 255, 255);
        let rect_pos = Vec2::new(300.0, 150.0);
        ctx.draw_rect(rect_pos, 150.0, 100.0, blue);
        printf(b"Drew blue rectangle at (300, 150) with size 150x100\n\0" as *const u8);
        
        // Draw a yellow circle
        let yellow = Color::new(255, 255, 0, 255);
        let circle_center = Vec2::new(200.0, 300.0);
        let circle_radius = 50.0;
        ctx.draw_circle(circle_center, circle_radius, yellow);
        printf(b"Drew yellow circle at (200, 300) with radius 50\n\0" as *const u8);
        
        // Get framebuffer
        let fb = ctx.get_framebuffer();
        printf(b"Framebuffer size: %d pixels\n\0" as *const u8, fb.len() as i32);
        printf(b"First pixel value: 0x%08x\n\0" as *const u8, fb[0] as i32);
        
        // Open framebuffer device
        let fb_fd = open(b"/dev/fb0\0" as *const u8, O_RDWR);
        if fb_fd >= 0 {
            printf(b"Opened framebuffer device: /dev/fb0\n\0" as *const u8);
            
            // Write framebuffer to display directly
            let copy_size = (ctx.width * ctx.height * 4) as usize; // RGBA
            let written = write(fb_fd, fb.as_ptr() as *const core::ffi::c_void, copy_size);
            if written >= 0 {
                printf(b"Wrote %ld bytes to framebuffer\n\0" as *const u8, written as i32);
                printf(b"Updated display\n\0" as *const u8);
            } else {
                printf(b"Failed to write to framebuffer\n\0" as *const u8);
            }
            
            // Close framebuffer device
            close(fb_fd);
            printf(b"Closed framebuffer device\n\0" as *const u8);
        } else {
            printf(b"Failed to open framebuffer device\n\0" as *const u8);
        }
        
        printf(b"FHRE Rust demo completed\n\0" as *const u8);
    }
    0
}