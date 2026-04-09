//! Feather Hybrid Render Engine (Rust version)
//! 
//! Lightweight hybrid rendering engine supporting 2D, 2.5D, and 3D rendering

/// FHRE version
pub const FHRE_VERSION: &str = "1.0.0";

/// Render modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// 2D rendering mode
    Mode2D,
    /// 2.5D rendering mode
    Mode25D,
    /// 3D rendering mode
    Mode3D,
}

/// Color structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
    
    /// Convert color to RGBA32
    pub fn to_rgba32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
}

/// Vector 2D structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Create a new 2D vector
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }
}

/// Vector 3D structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// Create a new 3D vector
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
}

/// Render context
pub struct Context {
    pub mode: RenderMode,
    pub width: u32,
    pub height: u32,
    framebuffer: Vec<u32>,
}

impl Context {
    /// Initialize FHRE context
    pub fn new(width: u32, height: u32, mode: RenderMode) -> Self {
        let size = (width * height) as usize;
        let framebuffer = vec![0; size];
        
        Context {
            mode,
            width,
            height,
            framebuffer,
        }
    }
    
    /// Set render mode
    pub fn set_mode(&mut self, mode: RenderMode) {
        self.mode = mode;
    }
    
    /// Clear framebuffer
    pub fn clear(&mut self, color: Color) {
        let color_val = color.to_rgba32();
        for pixel in &mut self.framebuffer {
            *pixel = color_val;
        }
    }
    
    /// Draw a 2D point
    pub fn draw_point(&mut self, pos: Vec2, color: Color) {
        let x = pos.x as i32;
        let y = pos.y as i32;
        
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let index = (y * self.width as i32 + x) as usize;
            if index < self.framebuffer.len() {
                self.framebuffer[index] = color.to_rgba32();
            }
        }
    }
    
    /// Draw a 2D line
    pub fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color) {
        // Bresenham's line algorithm
        let mut x0 = start.x as i32;
        let mut y0 = start.y as i32;
        let x1 = end.x as i32;
        let y1 = end.y as i32;
        
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        
        loop {
            self.draw_point(Vec2::new(x0 as f32, y0 as f32), color);
            
            if x0 == x1 && y0 == y1 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }
    
    /// Draw a 2D rectangle
    pub fn draw_rect(&mut self, pos: Vec2, width: f32, height: f32, color: Color) {
        // Top line
        let top_start = Vec2::new(pos.x, pos.y);
        let top_end = Vec2::new(pos.x + width, pos.y);
        self.draw_line(top_start, top_end, color);
        
        // Right line
        let right_start = Vec2::new(pos.x + width, pos.y);
        let right_end = Vec2::new(pos.x + width, pos.y + height);
        self.draw_line(right_start, right_end, color);
        
        // Bottom line
        let bottom_start = Vec2::new(pos.x + width, pos.y + height);
        let bottom_end = Vec2::new(pos.x, pos.y + height);
        self.draw_line(bottom_start, bottom_end, color);
        
        // Left line
        let left_start = Vec2::new(pos.x, pos.y + height);
        let left_end = Vec2::new(pos.x, pos.y);
        self.draw_line(left_start, left_end, color);
    }
    
    /// Get framebuffer
    pub fn get_framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }
    
    /// Get mutable framebuffer
    pub fn get_framebuffer_mut(&mut self) -> &mut [u32] {
        &mut self.framebuffer
    }
}

/// Get FHRE version
pub fn version() -> &'static str {
    FHRE_VERSION
}