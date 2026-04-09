//! FHRE Rust language demo
//! 
//! Demonstrates the basic usage of Feather Hybrid Render Engine

use fhre::{Context, Color, Vec2, RenderMode, version};

fn main() {
    println!("FHRE Rust language demo");
    println!("Version: {}", version());
    
    // Initialize FHRE context with 640x480 resolution and 2D mode
    let mut ctx = Context::new(640, 480, RenderMode::Mode2D);
    println!("FHRE context initialized: {}x{}, mode: {:?}", 
           ctx.width, ctx.height, ctx.mode);
    
    // Clear framebuffer with black color
    let black = Color::new(0, 0, 0, 255);
    ctx.clear(black);
    println!("Framebuffer cleared with black");
    
    // Draw a red point
    let red = Color::new(255, 0, 0, 255);
    let point = Vec2::new(100.0, 100.0);
    ctx.draw_point(point, red);
    println!("Drew red point at (100, 100)");
    
    // Draw a green line
    let green = Color::new(0, 255, 0, 255);
    let line_start = Vec2::new(50.0, 200.0);
    let line_end = Vec2::new(200.0, 350.0);
    ctx.draw_line(line_start, line_end, green);
    println!("Drew green line from (50, 200) to (200, 350)");
    
    // Draw a blue rectangle
    let blue = Color::new(0, 0, 255, 255);
    let rect_pos = Vec2::new(300.0, 150.0);
    ctx.draw_rect(rect_pos, 150.0, 100.0, blue);
    println!("Drew blue rectangle at (300, 150) with size 150x100");
    
    // Get framebuffer
    let fb = ctx.get_framebuffer();
    println!("Framebuffer size: {} pixels", fb.len());
    println!("First pixel value: 0x{:08x}", fb[0]);
    
    // Here you could write code to display the framebuffer
    // For example, copy it to a display device
    
    println!("FHRE Rust demo completed");
}
