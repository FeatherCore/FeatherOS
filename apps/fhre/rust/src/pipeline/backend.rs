//! Software Rendering Backend
//!
//! CPU-based software rasterization implementation.
//! This is the default fallback backend when GPU is not available.

use crate::math::{Color, Rect, Vec2};
use crate::render_world::RenderCommand;
use alloc::vec::Vec;

/// Software rendering backend - simulates a GPU pipeline on CPU
pub struct SoftwareBackend {
    framebuffer: Vec<u32>,
    width: u32,
    height: u32,
    viewport: Rect,
}

impl SoftwareBackend {
    /// Create a new software backend with framebuffer dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            framebuffer: alloc::vec![0; pixel_count],
            width,
            height,
            viewport: Rect::new(0.0, 0.0, width as f32, height as f32),
        }
    }

    /// Get framebuffer reference
    pub fn framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    /// Get mutable framebuffer reference
    pub fn framebuffer_mut(&mut self) -> &mut [u32] {
        &mut self.framebuffer
    }

    /// Execute a single render command (software implementation)
    pub fn execute_command(&mut self, command: &RenderCommand) {
        match command {
            RenderCommand::Clear { color } => {
                self.clear(*color);
            }
            RenderCommand::DrawRect { rect, color } => {
                self.fill_rect(*rect, *color);
            }
            RenderCommand::DrawLine { start, end, color, thickness } => {
                self.draw_line(*start, *end, *color, *thickness);
            }
            RenderCommand::DrawTriangle { p0, p1, p2, color } => {
                self.fill_triangle(*p0, *p1, *p2, *color);
            }
            RenderCommand::DrawPolygon { vertices, color } => {
                self.fill_polygon(vertices, *color);
            }
            RenderCommand::DrawText { position, text: _, color, size: _ } => {
                // Simplified text rendering - draw placeholder
                let rect = Rect::new(position.x, position.y, 100.0, 20.0);
                self.fill_rect(rect, *color);
            }
            RenderCommand::SetScissor { rect } => {
                self.viewport = *rect;
            }
            RenderCommand::DisableScissor => {
                self.viewport = Rect::new(0.0, 0.0, self.width as f32, self.height as f32);
            }
        }
    }

    /// Execute multiple commands in batch
    pub fn execute_commands(&mut self, commands: &[RenderCommand]) {
        for command in commands {
            self.execute_command(command);
        }
    }

    /// Resize the framebuffer
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let pixel_count = (width * height) as usize;
        self.framebuffer = alloc::vec![0; pixel_count];
        self.viewport = Rect::new(0.0, 0.0, width as f32, height as f32);
    }

    /// Set viewport
    pub fn set_viewport(&mut self, rect: Rect) {
        self.viewport = rect;
    }

    /// Reset/clear the framebuffer
    pub fn reset(&mut self) {
        for pixel in self.framebuffer.iter_mut() {
            *pixel = 0;
        }
    }

    // ========== Low-level Software Rasterization ==========

    /// Clear the framebuffer with a specific color
    pub fn clear(&mut self, color: Color) {
        let color_value = color.to_u32();
        for pixel in self.framebuffer.iter_mut() {
            *pixel = color_value;
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(self.viewport.x) as u32;
        let y0 = rect.y.max(self.viewport.y) as u32;
        let x1 = (rect.x + rect.width).min(self.viewport.x + self.viewport.width) as u32;
        let y1 = (rect.y + rect.height).min(self.viewport.y + self.viewport.height) as u32;

        let color_value = color.to_u32();
        for y in y0..y1 {
            for x in x0..x1 {
                let index = (y * self.width + x) as usize;
                self.framebuffer[index] = color_value;
            }
        }
    }

    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color, _thickness: f32) {
        // Bresenham's line algorithm
        let x0 = start.x as i32;
        let y0 = start.y as i32;
        let x1 = end.x as i32;
        let y1 = end.y as i32;

        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            self.draw_pixel(x as u32, y as u32, color);

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn fill_triangle(&mut self, p0: Vec2, p1: Vec2, p2: Vec2, color: Color) {
        // Bounding box triangle rasterization
        // Clamp to viewport bounds, handling negative coordinates properly
        let min_x_f = p0.x.min(p1.x).min(p2.x).max(self.viewport.x);
        let min_y_f = p0.y.min(p1.y).min(p2.y).max(self.viewport.y);
        let max_x_f = p0.x.max(p1.x).max(p2.x).min(self.viewport.x + self.viewport.width);
        let max_y_f = p0.y.max(p1.y).max(p2.y).min(self.viewport.y + self.viewport.height);

        // Convert to u32 safely (negative values become 0)
        let min_x = min_x_f.max(0.0) as u32;
        let min_y = min_y_f.max(0.0) as u32;
        let max_x = max_x_f.max(0.0) as u32;
        let max_y = max_y_f.max(0.0) as u32;

        // Skip degenerate triangles
        if min_x >= max_x || min_y >= max_y {
            return;
        }

        let color_value = color.to_u32();

        for y in min_y..max_y {
            for x in min_x..max_x {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                if Self::point_in_triangle(p, p0, p1, p2) {
                    let index = (y * self.width + x) as usize;
                    if index < self.framebuffer.len() {
                        self.framebuffer[index] = color_value;
                    }
                }
            }
        }
    }

    fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.framebuffer[index] = color.to_u32();
        }
    }

    fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
        let s1 = c.y - a.y;
        let s2 = c.x - a.x;
        let s3 = b.y - a.y;
        let s4 = p.y - a.y;

        let w1 = (a.x * s1 + s4 * s2 - p.x * s1) / (s3 * s2 - (b.x - a.x) * s1);
        let w2 = (s4 - w1 * s3) / s1;

        w1 >= 0.0 && w2 >= 0.0 && (w1 + w2) <= 1.0
    }

    /// Fill a convex polygon using scanline algorithm
    fn fill_polygon(&mut self, vertices: &[Vec2], color: Color) {
        if vertices.len() < 3 {
            return;
        }

        // Find bounding box
        let mut min_x = vertices[0].x;
        let mut max_x = vertices[0].x;
        let mut min_y = vertices[0].y;
        let mut max_y = vertices[0].y;

        for v in vertices.iter().skip(1) {
            min_x = min_x.min(v.x);
            max_x = max_x.max(v.x);
            min_y = min_y.min(v.y);
            max_y = max_y.max(v.y);
        }

        // Clamp to viewport
        let min_x = min_x.max(self.viewport.x) as i32;
        let max_x = max_x.min(self.viewport.x + self.viewport.width) as i32;
        let min_y = min_y.max(self.viewport.y) as i32;
        let max_y = max_y.min(self.viewport.y + self.viewport.height) as i32;

        // Scanline fill
        for y in min_y..=max_y {
            let mut intersections: Vec<f32> = Vec::new();
            let yf = y as f32;

            // Find intersections with all edges
            for i in 0..vertices.len() {
                let j = (i + 1) % vertices.len();
                let v1 = vertices[i];
                let v2 = vertices[j];

                // Check if scanline intersects this edge
                if (v1.y <= yf && v2.y > yf) || (v2.y <= yf && v1.y > yf) {
                    // Calculate x intersection
                    let t = (yf - v1.y) / (v2.y - v1.y);
                    let x = v1.x + t * (v2.x - v1.x);
                    intersections.push(x);
                }
            }

            // Sort intersections
            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // Fill between pairs of intersections
            for i in (0..intersections.len()).step_by(2) {
                if i + 1 < intersections.len() {
                    let x_start = intersections[i].max(self.viewport.x) as i32;
                    let x_end = intersections[i + 1].min(self.viewport.x + self.viewport.width) as i32;

                    for x in x_start..=x_end {
                        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                            let index = (y as u32 * self.width + x as u32) as usize;
                            if index < self.framebuffer.len() {
                                self.framebuffer[index] = color.to_u32();
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for SoftwareBackend {
    fn default() -> Self {
        Self::new(800, 600)
    }
}
