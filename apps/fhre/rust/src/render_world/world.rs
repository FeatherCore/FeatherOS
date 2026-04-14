//! Render World Implementation
//!
//! The Render World contains GPU resources, draw commands, and framebuffer data.
//! It is separate from Main World to allow parallel processing.

use super::command::RenderCommand;
use super::object::RenderObject;
use crate::math::{Color, Rect};
use alloc::vec::Vec;

/// Render World - Container for rendering data
///
/// The Render World is populated during the Extract phase
/// and consumed during the Render phase.
pub struct RenderWorld {
    /// Width of the render target
    width: u32,
    /// Height of the render target
    height: u32,
    /// Framebuffer data (RGBA)
    framebuffer: Vec<u32>,
    /// Render objects to draw
    objects: Vec<RenderObject>,
    /// Render commands queue
    commands: Vec<RenderCommand>,
    /// Clear color
    clear_color: Color,
    /// Viewport rectangle
    viewport: Rect,
}

impl RenderWorld {
    /// Create a new Render World with specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            width,
            height,
            framebuffer: alloc::vec![0; pixel_count],
            objects: Vec::new(),
            commands: Vec::new(),
            clear_color: Color::BLACK,
            viewport: Rect::new(0.0, 0.0, width as f32, height as f32),
        }
    }

    /// Get framebuffer width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get framebuffer height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get framebuffer reference
    pub fn framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    /// Get mutable framebuffer reference
    pub fn framebuffer_mut(&mut self) -> &mut [u32] {
        &mut self.framebuffer
    }

    /// Clear the framebuffer with the clear color
    pub fn clear(&mut self) {
        let clear_value = self.clear_color.to_u32();
        for pixel in self.framebuffer.iter_mut() {
            *pixel = clear_value;
        }
    }

    /// Set clear color
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Get clear color
    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    /// Set viewport
    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.viewport = Rect::new(x, y, width, height);
    }

    /// Get viewport
    pub fn viewport(&self) -> Rect {
        self.viewport
    }

    /// Add a render object
    pub fn add_object(&mut self, object: RenderObject) {
        self.objects.push(object);
    }

    /// Get all render objects
    pub fn objects(&self) -> &[RenderObject] {
        &self.objects
    }

    /// Get mutable render objects
    pub fn objects_mut(&mut self) -> &mut Vec<RenderObject> {
        &mut self.objects
    }

    /// Clear all render objects
    pub fn clear_objects(&mut self) {
        self.objects.clear();
    }

    /// Add a render command
    pub fn add_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }

    /// Get all render commands
    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }

    /// Get mutable render commands
    pub fn commands_mut(&mut self) -> &mut Vec<RenderCommand> {
        &mut self.commands
    }

    /// Clear all render commands
    pub fn clear_commands(&mut self) {
        self.commands.clear();
    }

    /// Draw a pixel at (x, y) with color
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.framebuffer[index] = color.to_u32();
        }
    }

    /// Draw a filled rectangle
    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(0.0) as u32;
        let y0 = rect.y.max(0.0) as u32;
        let x1 = (rect.x + rect.width).min(self.width as f32) as u32;
        let y1 = (rect.y + rect.height).min(self.height as f32) as u32;

        let color_value = color.to_u32();
        for y in y0..y1 {
            for x in x0..x1 {
                let index = (y * self.width + x) as usize;
                self.framebuffer[index] = color_value;
            }
        }
    }

    /// Draw a line from (x0, y0) to (x1, y1)
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        // Bresenham's line algorithm
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            if x0 >= 0 && y0 >= 0 {
                self.draw_pixel(x0 as u32, y0 as u32, color);
            }

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

    /// Reset the render world for a new frame
    pub fn reset(&mut self) {
        self.clear();
        self.clear_objects();
        self.clear_commands();
    }
}

impl Default for RenderWorld {
    fn default() -> Self {
        Self::new(800, 600)
    }
}
