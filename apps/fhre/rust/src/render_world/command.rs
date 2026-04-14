//! Render Command Implementation
//!
//! Render commands represent low-level drawing operations
//! that the renderer executes.

use crate::math::{Color, Rect, Vec2};
use alloc::vec::Vec;

/// Render Command - A single drawing operation
///
/// These commands are queued and executed by the renderer.
#[derive(Clone, Debug)]
pub enum RenderCommand {
    /// Clear the framebuffer
    Clear { color: Color },
    /// Draw a rectangle
    DrawRect { rect: Rect, color: Color },
    /// Draw a line
    DrawLine { start: Vec2, end: Vec2, color: Color, thickness: f32 },
    /// Draw a triangle
    DrawTriangle { p0: Vec2, p1: Vec2, p2: Vec2, color: Color },
    /// Draw text (simplified - just rectangles for now)
    DrawText { position: Vec2, text: &'static str, color: Color, size: f32 },
    /// Set scissor rectangle
    SetScissor { rect: Rect },
    /// Disable scissor
    DisableScissor,
}

impl RenderCommand {
    /// Create a clear command
    pub fn clear(color: Color) -> Self {
        Self::Clear { color }
    }

    /// Create a draw rect command
    pub fn draw_rect(rect: Rect, color: Color) -> Self {
        Self::DrawRect { rect, color }
    }

    /// Create a draw line command
    pub fn draw_line(start: Vec2, end: Vec2, color: Color) -> Self {
        Self::DrawLine { start, end, color, thickness: 1.0 }
    }

    /// Create a draw line command with thickness
    pub fn draw_line_thick(start: Vec2, end: Vec2, color: Color, thickness: f32) -> Self {
        Self::DrawLine { start, end, color, thickness }
    }

    /// Create a draw triangle command
    pub fn draw_triangle(p0: Vec2, p1: Vec2, p2: Vec2, color: Color) -> Self {
        Self::DrawTriangle { p0, p1, p2, color }
    }

    /// Create a draw text command
    pub fn draw_text(position: Vec2, text: &'static str, color: Color, size: f32) -> Self {
        Self::DrawText { position, text, color, size }
    }

    /// Create a set scissor command
    pub fn set_scissor(rect: Rect) -> Self {
        Self::SetScissor { rect }
    }

    /// Create a disable scissor command
    pub fn disable_scissor() -> Self {
        Self::DisableScissor
    }
}

/// Draw Call - A batch of geometry to render
///
/// Multiple draw calls can be merged for efficiency.
#[derive(Clone, Debug)]
pub struct DrawCall {
    /// Type of primitive to draw
    pub primitive: PrimitiveType,
    /// Vertex data
    pub vertices: Vec<Vertex>,
    /// Index data (optional)
    pub indices: Vec<u16>,
    /// Transform matrix
    pub transform: [f32; 16],
    /// Color tint
    pub color: Color,
}

impl DrawCall {
    /// Create a new empty draw call
    pub fn new(primitive: PrimitiveType) -> Self {
        Self {
            primitive,
            vertices: Vec::new(),
            indices: Vec::new(),
            transform: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
            color: Color::WHITE,
        }
    }

    /// Add a vertex
    pub fn add_vertex(&mut self, position: Vec2, uv: Vec2, color: Color) {
        self.vertices.push(Vertex {
            position,
            uv,
            color,
        });
    }

    /// Add an index
    pub fn add_index(&mut self, index: u16) {
        self.indices.push(index);
    }

    /// Add a triangle
    pub fn add_triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) {
        let base = self.vertices.len() as u16;
        self.vertices.push(v0);
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
    }

    /// Add a quad (as two triangles)
    pub fn add_quad(&mut self, tl: Vertex, tr: Vertex, br: Vertex, bl: Vertex) {
        let base = self.vertices.len() as u16;
        self.vertices.push(tl);
        self.vertices.push(tr);
        self.vertices.push(br);
        self.vertices.push(bl);
        // First triangle: tl, tr, br
        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
        // Second triangle: tl, br, bl
        self.indices.push(base);
        self.indices.push(base + 2);
        self.indices.push(base + 3);
    }

    /// Set transform matrix
    pub fn set_transform(&mut self, transform: [f32; 16]) {
        self.transform = transform;
    }

    /// Set color tint
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

/// Primitive type for draw calls
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    /// Points
    Points,
    /// Lines
    Lines,
    /// Line strip
    LineStrip,
    /// Triangles
    Triangles,
    /// Triangle strip
    TriangleStrip,
}

/// Vertex data structure
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    /// Position in screen space
    pub position: Vec2,
    /// Texture coordinates
    pub uv: Vec2,
    /// Vertex color
    pub color: Color,
}

impl Vertex {
    /// Create a new vertex
    pub fn new(position: Vec2, uv: Vec2, color: Color) -> Self {
        Self { position, uv, color }
    }

    /// Create a vertex with position only (white color, zero UV)
    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            uv: Vec2::ZERO,
            color: Color::WHITE,
        }
    }

    /// Create a vertex with position and color
    pub fn with_color(position: Vec2, color: Color) -> Self {
        Self {
            position,
            uv: Vec2::ZERO,
            color,
        }
    }
}
