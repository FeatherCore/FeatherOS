//! Renderer Implementation
//!
//! The renderer processes render commands and outputs to the framebuffer.

use crate::render_world::RenderWorld;
use crate::math::{Color, Rect, Vec2};
use alloc::vec::Vec;

/// Renderer - Processes render commands
///
/// Takes render commands from RenderWorld and executes them.
pub struct Renderer {
    /// Current scissor rectangle (None = no scissor)
    scissor: Option<Rect>,
    /// Statistics
    stats: RenderStats,
}

impl Renderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self {
            scissor: None,
            stats: RenderStats::new(),
        }
    }

    /// Render the current frame
    pub fn render(&mut self, render_world: &mut RenderWorld) {
        self.stats.reset();

        // Clear the framebuffer
        render_world.clear();

        // Process render commands
        let commands: Vec<crate::RenderCommand> = render_world.commands().to_vec();
        for command in commands {
            self.execute_command(render_world, &command);
        }

        // Render all objects
        let objects: Vec<_> = render_world.objects().to_vec();
        for object in objects {
            if object.visible {
                self.render_object(render_world, &object);
                self.stats.draw_calls += 1;
            }
        }

        self.stats.objects_rendered = render_world.objects().len();
    }

    /// Present the rendered frame (copy to display)
    pub fn present(&self, _render_world: &RenderWorld) {
        // The actual presentation is handled by the platform layer
        // (e.g., SimDisplay::present())
    }

    /// Execute a single render command
    fn execute_command(&mut self, render_world: &mut RenderWorld, command: &crate::RenderCommand) {
        use crate::RenderCommand;
        match command {
            RenderCommand::Clear { color } => {
                render_world.set_clear_color(*color);
                render_world.clear();
            }
            RenderCommand::DrawRect { rect, color } => {
                if let Some(scissor) = self.scissor {
                    if let Some(clipped) = rect.intersect(&scissor) {
                        render_world.draw_rect(clipped, *color);
                    }
                } else {
                    render_world.draw_rect(*rect, *color);
                }
                self.stats.draw_calls += 1;
            }
            RenderCommand::DrawLine { start, end, color, thickness: _ } => {
                render_world.draw_line(
                    start.x as i32,
                    start.y as i32,
                    end.x as i32,
                    end.y as i32,
                    *color
                );
                self.stats.draw_calls += 1;
            }
            RenderCommand::DrawTriangle { p0, p1, p2, color } => {
                self.draw_triangle(render_world, *p0, *p1, *p2, *color);
                self.stats.draw_calls += 1;
            }
            RenderCommand::DrawText { position, text: _, color, size: _ } => {
                // Simplified text rendering - just draw a rectangle
                let rect = Rect::new(position.x, position.y, 8.0, 8.0);
                render_world.draw_rect(rect, *color);
                self.stats.draw_calls += 1;
            }
            RenderCommand::SetScissor { rect } => {
                self.scissor = Some(*rect);
            }
            RenderCommand::DisableScissor => {
                self.scissor = None;
            }
        }
    }

    /// Render a render object
    fn render_object(&self, render_world: &mut RenderWorld, object: &crate::RenderObject) {
        let bounds = object.get_bounds();

        // Apply scissor if set
        if let Some(scissor) = self.scissor {
            if let Some(_clipped) = bounds.intersect(&scissor) {
                self.draw_quad(render_world, &object.get_corners(), object.color);
            }
        } else {
            self.draw_quad(render_world, &object.get_corners(), object.color);
        }
    }

    /// Draw a triangle
    fn draw_triangle(&self, render_world: &mut RenderWorld, p0: Vec2, p1: Vec2, p2: Vec2, color: Color) {
        // Simple scanline triangle rasterization
        let min_x = p0.x.min(p1.x).min(p2.x) as i32;
        let max_x = p0.x.max(p1.x).max(p2.x) as i32;
        let min_y = p0.y.min(p1.y).min(p2.y) as i32;
        let max_y = p0.y.max(p1.y).max(p2.y) as i32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                if Self::point_in_triangle(p, p0, p1, p2) {
                    render_world.draw_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    /// Draw a quad (two triangles)
    fn draw_quad(&self, render_world: &mut RenderWorld, corners: &[Vec2; 4], color: Color) {
        // corners: [tl, tr, br, bl]
        // First triangle: tl, tr, br
        self.draw_triangle(render_world, corners[0], corners[1], corners[2], color);
        // Second triangle: tl, br, bl
        self.draw_triangle(render_world, corners[0], corners[2], corners[3], color);
    }

    /// Check if a point is inside a triangle
    fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
        let s = a.y * c.x - a.x * c.y + (c.y - a.y) * p.x + (a.x - c.x) * p.y;
        let t = a.x * b.y - a.y * b.x + (a.y - b.y) * p.x + (b.x - a.x) * p.y;

        if (s < 0.0) != (t < 0.0) && s != 0.0 && t != 0.0 {
            return false;
        }

        let d = -b.y * c.x + a.y * (c.x - b.x) + a.x * (b.y - c.y) + b.x * c.y;
        if d < 0.0 {
            s <= 0.0 && s + t >= d
        } else {
            s >= 0.0 && s + t <= d
        }
    }

    /// Get render statistics
    pub fn stats(&self) -> &RenderStats {
        &self.stats
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Render statistics
#[derive(Clone, Debug)]
pub struct RenderStats {
    /// Number of draw calls this frame
    pub draw_calls: usize,
    /// Number of objects rendered
    pub objects_rendered: usize,
    /// Frame time in milliseconds
    pub frame_time_ms: f32,
}

impl RenderStats {
    /// Create new stats
    pub fn new() -> Self {
        Self {
            draw_calls: 0,
            objects_rendered: 0,
            frame_time_ms: 0.0,
        }
    }

    /// Reset stats
    pub fn reset(&mut self) {
        self.draw_calls = 0;
        self.objects_rendered = 0;
    }
}

impl Default for RenderStats {
    fn default() -> Self {
        Self::new()
    }
}
