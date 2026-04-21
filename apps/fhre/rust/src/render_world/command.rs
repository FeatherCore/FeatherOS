//! Render Command Implementation
//!
//! Render commands represent low-level drawing operations
//! that the renderer executes.

use crate::math::{Color, Rect, Vec2};
use crate::pipeline::{Gradient, TextureRegion};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug)]
pub enum RenderCommand {
    Clear { color: Color },
    DrawRect { rect: Rect, color: Color },
    DrawRectGradient { rect: Rect, gradient: Gradient },
    DrawRectRounded { rect: Rect, color: Color, radius: f32 },
    DrawRectRoundedGradient { rect: Rect, gradient: Gradient, radius: f32 },
    DrawLine { start: Vec2, end: Vec2, color: Color, thickness: f32 },
    DrawTriangle { p0: Vec2, p1: Vec2, p2: Vec2, color: Color },
    DrawPolygon { vertices: Vec<Vec2>, color: Color },
    DrawPolygonTextured { vertices: Vec<Vec2>, uvs: Vec<Vec2>, texture_id: u32, color: Color },
    DrawText { position: Vec2, text: String, color: Color, size: f32 },
    DrawImage { rect: Rect, texture_id: u32, region: TextureRegion, color: Color },
    DrawImageTransformed { position: Vec2, size: Vec2, texture_id: u32, region: TextureRegion, rotation: f32, color: Color },
    SetScissor { rect: Rect },
    DisableScissor,
    PushMask,
    PopMask,
}

impl RenderCommand {
    pub fn clear(color: Color) -> Self {
        Self::Clear { color }
    }

    pub fn draw_rect(rect: Rect, color: Color) -> Self {
        Self::DrawRect { rect, color }
    }

    pub fn draw_rect_gradient(rect: Rect, gradient: Gradient) -> Self {
        Self::DrawRectGradient { rect, gradient }
    }

    pub fn draw_rect_rounded(rect: Rect, color: Color, radius: f32) -> Self {
        Self::DrawRectRounded { rect, color, radius }
    }

    pub fn draw_rect_rounded_gradient(rect: Rect, gradient: Gradient, radius: f32) -> Self {
        Self::DrawRectRoundedGradient { rect, gradient, radius }
    }

    pub fn draw_line(start: Vec2, end: Vec2, color: Color) -> Self {
        Self::DrawLine { start, end, color, thickness: 1.0 }
    }

    pub fn draw_line_thick(start: Vec2, end: Vec2, color: Color, thickness: f32) -> Self {
        Self::DrawLine { start, end, color, thickness }
    }

    pub fn draw_triangle(p0: Vec2, p1: Vec2, p2: Vec2, color: Color) -> Self {
        Self::DrawTriangle { p0, p1, p2, color }
    }

    pub fn draw_polygon(vertices: Vec<Vec2>, color: Color) -> Self {
        Self::DrawPolygon { vertices, color }
    }

    pub fn draw_polygon_textured(vertices: Vec<Vec2>, uvs: Vec<Vec2>, texture_id: u32, color: Color) -> Self {
        Self::DrawPolygonTextured { vertices, uvs, texture_id, color }
    }

    pub fn draw_text(position: Vec2, text: impl AsRef<str>, color: Color, size: f32) -> Self {
        Self::DrawText { position, text: text.as_ref().into(), color, size }
    }

    pub fn draw_image(rect: Rect, texture_id: u32, region: TextureRegion, color: Color) -> Self {
        Self::DrawImage { rect, texture_id, region, color }
    }

    pub fn draw_image_full(rect: Rect, texture_id: u32, color: Color) -> Self {
        Self::DrawImage {
            rect,
            texture_id,
            region: TextureRegion::full(texture_id),
            color,
        }
    }

    pub fn set_scissor(rect: Rect) -> Self {
        Self::SetScissor { rect }
    }

    pub fn disable_scissor() -> Self {
        Self::DisableScissor
    }
}

#[derive(Clone, Debug)]
pub struct DrawCall {
    pub primitive: PrimitiveType,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub transform: [f32; 16],
    pub color: Color,
    pub texture_id: Option<u32>,
}

impl DrawCall {
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
            texture_id: None,
        }
    }

    pub fn add_vertex(&mut self, position: Vec2, uv: Vec2, color: Color) {
        self.vertices.push(Vertex {
            position,
            uv,
            color,
        });
    }

    pub fn add_index(&mut self, index: u16) {
        self.indices.push(index);
    }

    pub fn add_triangle(&mut self, v0: Vertex, v1: Vertex, v2: Vertex) {
        let base = self.vertices.len() as u16;
        self.vertices.push(v0);
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
    }

    pub fn add_quad(&mut self, tl: Vertex, tr: Vertex, br: Vertex, bl: Vertex) {
        let base = self.vertices.len() as u16;
        self.vertices.push(tl);
        self.vertices.push(tr);
        self.vertices.push(br);
        self.vertices.push(bl);
        self.indices.push(base);
        self.indices.push(base + 1);
        self.indices.push(base + 2);
        self.indices.push(base);
        self.indices.push(base + 2);
        self.indices.push(base + 3);
    }

    pub fn set_transform(&mut self, transform: [f32; 16]) {
        self.transform = transform;
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_texture(&mut self, texture_id: u32) {
        self.texture_id = Some(texture_id);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec2,
    pub uv: Vec2,
    pub color: Color,
}

impl Vertex {
    pub fn new(position: Vec2, uv: Vec2, color: Color) -> Self {
        Self { position, uv, color }
    }

    pub fn from_position(position: Vec2) -> Self {
        Self {
            position,
            uv: Vec2::ZERO,
            color: Color::WHITE,
        }
    }

    pub fn with_color(position: Vec2, color: Color) -> Self {
        Self {
            position,
            uv: Vec2::ZERO,
            color,
        }
    }

    pub fn with_uv(position: Vec2, uv: Vec2) -> Self {
        Self {
            position,
            uv,
            color: Color::WHITE,
        }
    }

    pub fn with_uv_color(position: Vec2, uv: Vec2, color: Color) -> Self {
        Self { position, uv, color }
    }
}
