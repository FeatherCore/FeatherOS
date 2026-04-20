//! Software Rendering Backend
//!
//! CPU-based software rasterization implementation.
//! This is the default fallback backend when GPU is not available.
//!
//! Features:
//! - Alpha blending with multiple blend modes
//! - Texture mapping (nearest/linear sampling)
//! - Gradient fills (linear, radial)
//! - Rounded rectangles
//! - Scissor clipping

use crate::math::{Color, Rect, Vec2, BlendMode};
use crate::render_world::RenderCommand;
use crate::pipeline::{Gradient, TextureRegion, Texture};
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

const BLEND_LUT_SIZE: usize = 256;

pub struct SoftwareBackend {
    framebuffer: Vec<u32>,
    width: u32,
    height: u32,
    viewport: Rect,
    blend_lut: Box<[[u8; BLEND_LUT_SIZE]]>,
    textures: BTreeMap<u32, Texture>,
}

impl SoftwareBackend {
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        
        let blend_lut = {
            let mut data: Vec<[u8; BLEND_LUT_SIZE]> = Vec::with_capacity(BLEND_LUT_SIZE);
            for _ in 0..BLEND_LUT_SIZE {
                data.push([0u8; BLEND_LUT_SIZE]);
            }
            data.into_boxed_slice()
        };
        
        Self {
            framebuffer: alloc::vec![0; pixel_count],
            width,
            height,
            viewport: Rect::new(0.0, 0.0, width as f32, height as f32),
            blend_lut,
            textures: BTreeMap::new(),
        }
    }

    fn build_blend_lut(&mut self) {
        for fg_alpha in 0..BLEND_LUT_SIZE {
            for bg_alpha in 0..BLEND_LUT_SIZE {
                let result = fg_alpha + ((255 - fg_alpha) * bg_alpha + 127) / 255;
                self.blend_lut[fg_alpha][bg_alpha] = result.min(255) as u8;
            }
        }
    }

    pub fn framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    pub fn framebuffer_mut(&mut self) -> &mut [u32] {
        &mut self.framebuffer
    }

    pub fn execute_command(&mut self, command: &RenderCommand) {
        match command {
            RenderCommand::Clear { color } => {
                self.clear(*color);
            }
            RenderCommand::DrawRect { rect, color } => {
                self.fill_rect(*rect, *color);
            }
            RenderCommand::DrawRectGradient { rect, gradient } => {
                self.fill_rect_gradient(*rect, gradient);
            }
            RenderCommand::DrawRectRounded { rect, color, radius } => {
                self.fill_rect_rounded(*rect, *color, *radius);
            }
            RenderCommand::DrawRectRoundedGradient { rect, gradient, radius } => {
                self.fill_rect_rounded_gradient(*rect, gradient, *radius);
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
            RenderCommand::DrawPolygonTextured { vertices, uvs, texture_id, color } => {
                self.fill_polygon_textured(vertices, uvs, *texture_id, *color);
            }
            RenderCommand::DrawText { position, text: _, color, size: _ } => {
                let rect = Rect::new(position.x, position.y, 100.0, 20.0);
                self.fill_rect(rect, *color);
            }
            RenderCommand::DrawImage { rect, texture_id: _, region, color } => {
                self.fill_rect_tinted(*rect, *region, *color);
            }
            RenderCommand::DrawImageTransformed { position, size, texture_id: _, region, rotation, color } => {
                self.fill_rect_transformed(*position, *size, *region, *rotation, *color);
            }
            RenderCommand::SetScissor { rect } => {
                self.viewport = *rect;
            }
            RenderCommand::DisableScissor => {
                self.viewport = Rect::new(0.0, 0.0, self.width as f32, self.height as f32);
            }
            RenderCommand::PushMask | RenderCommand::PopMask => {
                // TODO: Implement mask stack
            }
        }
    }

    pub fn execute_commands(&mut self, commands: &[RenderCommand]) {
        for command in commands {
            self.execute_command(command);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let pixel_count = (width * height) as usize;
        self.framebuffer = alloc::vec![0; pixel_count];
        self.viewport = Rect::new(0.0, 0.0, width as f32, height as f32);
    }

    pub fn set_viewport(&mut self, rect: Rect) {
        self.viewport = rect;
    }

    pub fn reset(&mut self) {
        for pixel in self.framebuffer.iter_mut() {
            *pixel = 0;
        }
    }

    pub fn upload_texture(&mut self, id: u32, texture: Texture) {
        self.textures.insert(id, texture);
    }

    pub fn remove_texture(&mut self, id: u32) -> Option<Texture> {
        self.textures.remove(&id)
    }

    pub fn get_texture(&self, id: u32) -> Option<&Texture> {
        self.textures.get(&id)
    }

    // ========== Low-level Software Rasterization ==========

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

        if color.a == 255 {
            let color_value = color.to_u32();
            for y in y0..y1 {
                for x in x0..x1 {
                    let index = (y * self.width + x) as usize;
                    self.framebuffer[index] = color_value;
                }
            }
        } else {
            for y in y0..y1 {
                for x in x0..x1 {
                    self.blend_pixel(x, y, color);
                }
            }
        }
    }

    fn draw_line(&mut self, start: Vec2, end: Vec2, color: Color, _thickness: f32) {
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
        let min_x_f = p0.x.min(p1.x).min(p2.x).max(self.viewport.x);
        let min_y_f = p0.y.min(p1.y).min(p2.y).max(self.viewport.y);
        let max_x_f = p0.x.max(p1.x).max(p2.x).min(self.viewport.x + self.viewport.width);
        let max_y_f = p0.y.max(p1.y).max(p2.y).min(self.viewport.y + self.viewport.height);

        let min_x = min_x_f.max(0.0) as u32;
        let min_y = min_y_f.max(0.0) as u32;
        let max_x = max_x_f.max(0.0) as u32;
        let max_y = max_y_f.max(0.0) as u32;

        if min_x >= max_x || min_y >= max_y {
            return;
        }

        if color.a == 255 {
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
        } else {
            for y in min_y..max_y {
                for x in min_x..max_x {
                    let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                    if Self::point_in_triangle(p, p0, p1, p2) {
                        self.blend_pixel(x, y, color);
                    }
                }
            }
        }
    }

    fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            if color.a == 255 {
                let index = (y * self.width + x) as usize;
                self.framebuffer[index] = color.to_u32();
            } else {
                self.blend_pixel(x, y, color);
            }
        }
    }

    #[inline]
    fn blend_pixel(&mut self, x: u32, y: u32, fg: Color) {
        let index = (y * self.width + x) as usize;
        if index >= self.framebuffer.len() {
            return;
        }

        let bg = Color::from_u32(self.framebuffer[index]);
        let blended = fg.blend(bg, BlendMode::Normal);
        self.framebuffer[index] = blended.to_u32();
    }

    #[inline]
    fn blend_pixel_fast(&mut self, index: usize, fg: Color) {
        if index >= self.framebuffer.len() {
            return;
        }

        let bg = Color::from_u32(self.framebuffer[index]);
        let blended = Self::blend_colors_fast(fg, bg, &self.blend_lut);
        self.framebuffer[index] = blended.to_u32();
    }

    #[inline]
    fn blend_colors_fast(fg: Color, bg: Color, lut: &[[u8; BLEND_LUT_SIZE]]) -> Color {
        if fg.a == 0 {
            return bg;
        }
        if fg.a == 255 {
            return fg;
        }

        let out_alpha = lut[fg.a as usize][bg.a as usize];
        if out_alpha == 0 {
            return Color::TRANSPARENT;
        }

        let fg_a = fg.a as u32;
        let bg_a = bg.a as u32;
        let inv_fg_a = 255 - fg_a;

        let blend_channel = |fg_c: u8, bg_c: u8| -> u8 {
            let fg_val = fg_c as u32;
            let bg_val = bg_c as u32;
            let numerator = fg_val * 255 * fg_a + bg_val * bg_a * inv_fg_a;
            let denominator = out_alpha as u32 * 255;
            ((numerator + denominator / 2) / denominator) as u8
        };

        Color::new(
            blend_channel(fg.r, bg.r),
            blend_channel(fg.g, bg.g),
            blend_channel(fg.b, bg.b),
            out_alpha,
        )
    }

    fn point_in_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
        let s1 = c.y - a.y;
        let s2 = c.x - a.x;
        let s3 = b.y - a.y;
        let s4 = p.y - a.y;

        let denom = s3 * s2 - (b.x - a.x) * s1;
        if denom.abs() < 0.0001 {
            return false;
        }

        let w1 = (a.x * s1 + s4 * s2 - p.x * s1) / denom;
        let w2 = (s4 - w1 * s3) / s1;

        w1 >= 0.0 && w2 >= 0.0 && (w1 + w2) <= 1.0
    }

    fn fill_polygon(&mut self, vertices: &[Vec2], color: Color) {
        if vertices.len() < 3 {
            return;
        }

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

        let _min_x = min_x.max(self.viewport.x) as i32;
        let _max_x = max_x.min(self.viewport.x + self.viewport.width) as i32;
        let min_y = min_y.max(self.viewport.y) as i32;
        let max_y = max_y.min(self.viewport.y + self.viewport.height) as i32;

        for y in min_y..=max_y {
            let mut intersections: Vec<f32> = Vec::new();
            let yf = y as f32;

            for i in 0..vertices.len() {
                let j = (i + 1) % vertices.len();
                let v1 = vertices[i];
                let v2 = vertices[j];

                if (v1.y <= yf && v2.y > yf) || (v2.y <= yf && v1.y > yf) {
                    let t = (yf - v1.y) / (v2.y - v1.y);
                    let x = v1.x + t * (v2.x - v1.x);
                    intersections.push(x);
                }
            }

            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());

            for i in (0..intersections.len()).step_by(2) {
                if i + 1 < intersections.len() {
                    let x_start = intersections[i].max(self.viewport.x) as i32;
                    let x_end = intersections[i + 1].min(self.viewport.x + self.viewport.width) as i32;

                    for x in x_start..=x_end {
                        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                            let index = (y as u32 * self.width + x as u32) as usize;
                            if index < self.framebuffer.len() {
                                if color.a == 255 {
                                    self.framebuffer[index] = color.to_u32();
                                } else {
                                    self.blend_pixel_fast(index, color);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn fill_polygon_textured(&mut self, vertices: &[Vec2], uvs: &[Vec2], texture_id: u32, tint: Color) {
        if vertices.len() < 3 || uvs.len() != vertices.len() {
            return;
        }

        let texture = match self.textures.get(&texture_id) {
            Some(t) => t.clone(),
            None => {
                let avg_color = tint;
                self.fill_polygon(vertices, avg_color);
                return;
            }
        };

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

        let _min_x_i = min_x.max(self.viewport.x) as i32;
        let _max_x_i = max_x.min(self.viewport.x + self.viewport.width) as i32;
        let min_y_i = min_y.max(self.viewport.y) as i32;
        let max_y_i = max_y.min(self.viewport.y + self.viewport.height) as i32;

        for y in min_y_i..=max_y_i {
            let yf = y as f32;
            let mut intersections: Vec<(f32, Vec2)> = Vec::new();

            for i in 0..vertices.len() {
                let j = (i + 1) % vertices.len();
                let v1 = vertices[i];
                let v2 = vertices[j];
                let uv1 = uvs[i];
                let uv2 = uvs[j];

                if (v1.y <= yf && v2.y > yf) || (v2.y <= yf && v1.y > yf) {
                    let t = (yf - v1.y) / (v2.y - v1.y);
                    let x = v1.x + t * (v2.x - v1.x);
                    let uv = Vec2::new(
                        uv1.x + t * (uv2.x - uv1.x),
                        uv1.y + t * (uv2.y - uv1.y),
                    );
                    intersections.push((x, uv));
                }
            }

            intersections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            for i in (0..intersections.len()).step_by(2) {
                if i + 1 < intersections.len() {
                    let (x1_start, uv1) = intersections[i];
                    let (x2_end, uv2) = intersections[i + 1];

                    let x_start = x1_start.max(self.viewport.x) as i32;
                    let x_end = x2_end.min(self.viewport.x + self.viewport.width) as i32;

                    let dx = x2_end - x1_start;
                    let duv_dx = if dx.abs() > 0.001 {
                        Vec2::new((uv2.x - uv1.x) / dx, (uv2.y - uv1.y) / dx)
                    } else {
                        Vec2::ZERO
                    };

                    for x in x_start..=x_end {
                        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                            let t = (x as f32 - x1_start).max(0.0);
                            let uv = Vec2::new(
                                uv1.x + t * duv_dx.x,
                                uv1.y + t * duv_dx.y,
                            );

                            let tex_color = texture.sample(uv);
                            
                            let final_color = if tint == Color::WHITE {
                                tex_color
                            } else {
                                Color::new(
                                    (tex_color.r as u16 * tint.r as u16 / 255) as u8,
                                    (tex_color.g as u16 * tint.g as u16 / 255) as u8,
                                    (tex_color.b as u16 * tint.b as u16 / 255) as u8,
                                    (tex_color.a as u16 * tint.a as u16 / 255) as u8,
                                )
                            };

                            let index = (y as u32 * self.width + x as u32) as usize;
                            if index < self.framebuffer.len() {
                                if final_color.a == 255 {
                                    self.framebuffer[index] = final_color.to_u32();
                                } else {
                                    self.blend_pixel_fast(index, final_color);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ========== Gradient Rendering ==========

    fn fill_rect_gradient(&mut self, rect: Rect, gradient: &Gradient) {
        let x0 = rect.x.max(self.viewport.x) as u32;
        let y0 = rect.y.max(self.viewport.y) as u32;
        let x1 = (rect.x + rect.width).min(self.viewport.x + self.viewport.width) as u32;
        let y1 = (rect.y + rect.height).min(self.viewport.y + self.viewport.height) as u32;

        let grad_with_bounds = gradient.clone().with_bounds(
            Vec2::new(rect.x, rect.y),
            Vec2::new(rect.x + rect.width, rect.y + rect.height),
        );

        for y in y0..y1 {
            for x in x0..x1 {
                let color = grad_with_bounds.sample(x as f32, y as f32);
                self.blend_pixel(x, y, color);
            }
        }
    }

    // ========== Rounded Rectangle Rendering ==========

    fn fill_rect_rounded(&mut self, rect: Rect, color: Color, radius: f32) {
        let max_radius = (rect.width.min(rect.height) / 2.0).min(radius);
        let r = max_radius.max(0.0);

        if r < 1.0 {
            self.fill_rect(rect, color);
            return;
        }

        let x0 = rect.x.max(self.viewport.x) as u32;
        let y0 = rect.y.max(self.viewport.y) as u32;
        let x1 = (rect.x + rect.width).min(self.viewport.x + self.viewport.width) as u32;
        let y1 = (rect.y + rect.height).min(self.viewport.y + self.viewport.height) as u32;

        let cx1 = (rect.x + r) as i32;
        let cy1 = (rect.y + r) as i32;
        let cx2 = (rect.x + rect.width - r) as i32;
        let cy2 = (rect.y + rect.height - r) as i32;

        for y in y0..y1 {
            for x in x0..x1 {
                let mut in_rect = true;

                let px = x as i32;
                let py = y as i32;

                if px < cx1 && py < cy1 {
                    let dx = cx1 - px;
                    let dy = cy1 - py;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    in_rect = dist_sq <= r * r;
                } else if px > cx2 && py < cy1 {
                    let dx = px - cx2;
                    let dy = cy1 - py;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    in_rect = dist_sq <= r * r;
                } else if px > cx2 && py > cy2 {
                    let dx = px - cx2;
                    let dy = py - cy2;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    in_rect = dist_sq <= r * r;
                } else if px < cx1 && py > cy2 {
                    let dx = cx1 - px;
                    let dy = py - cy2;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    in_rect = dist_sq <= r * r;
                }

                if in_rect {
                    self.blend_pixel(x, y, color);
                }
            }
        }
    }

    fn fill_rect_rounded_gradient(&mut self, rect: Rect, gradient: &Gradient, radius: f32) {
        let max_radius = (rect.width.min(rect.height) / 2.0).min(radius);
        let r = max_radius.max(0.0);

        if r < 1.0 {
            self.fill_rect_gradient(rect, gradient);
            return;
        }

        let x0 = rect.x.max(self.viewport.x) as u32;
        let y0 = rect.y.max(self.viewport.y) as u32;
        let x1 = (rect.x + rect.width).min(self.viewport.x + self.viewport.width) as u32;
        let y1 = (rect.y + rect.height).min(self.viewport.y + self.viewport.height) as u32;

        let cx1 = (rect.x + r) as i32;
        let cy1 = (rect.y + r) as i32;
        let cx2 = (rect.x + rect.width - r) as i32;
        let cy2 = (rect.y + rect.height - r) as i32;

        let grad_with_bounds = gradient.clone().with_bounds(
            Vec2::new(rect.x, rect.y),
            Vec2::new(rect.x + rect.width, rect.y + rect.height),
        );

        for y in y0..y1 {
            for x in x0..x1 {
                let mut in_rect = true;

                let px = x as i32;
                let py = y as i32;

                if px < cx1 && py < cy1 {
                    let dx = cx1 - px;
                    let dy = cy1 - py;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    in_rect = dist_sq <= r * r;
                } else if px > cx2 && py < cy1 {
                    let dx = px - cx2;
                    let dy = cy1 - py;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    in_rect = dist_sq <= r * r;
                } else if px > cx2 && py > cy2 {
                    let dx = px - cx2;
                    let dy = py - cy2;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    in_rect = dist_sq <= r * r;
                } else if px < cx1 && py > cy2 {
                    let dx = cx1 - px;
                    let dy = py - cy2;
                    let dist_sq = (dx * dx + dy * dy) as f32;
                    in_rect = dist_sq <= r * r;
                }

                if in_rect {
                    let color = grad_with_bounds.sample(x as f32, y as f32);
                    self.blend_pixel(x, y, color);
                }
            }
        }
    }

    // ========== Texture Rendering (Placeholder) ==========

    fn fill_rect_tinted(&mut self, rect: Rect, region: TextureRegion, color: Color) {
        let texture = self.textures.get(&region.texture_id).cloned();
        if let Some(texture) = texture {
            self.fill_rect_textured(rect, &texture, &region, color);
        } else {
            self.fill_rect(rect, color);
        }
    }

    fn fill_rect_textured(&mut self, rect: Rect, texture: &Texture, region: &TextureRegion, tint: Color) {
        let x0 = rect.x.max(self.viewport.x) as u32;
        let y0 = rect.y.max(self.viewport.y) as u32;
        let x1 = (rect.x + rect.width).min(self.viewport.x + self.viewport.width) as u32;
        let y1 = (rect.y + rect.height).min(self.viewport.y + self.viewport.height) as u32;

        let rect_width = rect.width.max(1.0);
        let rect_height = rect.height.max(1.0);

        for y in y0..y1 {
            for x in x0..x1 {
                let u = (x as f32 - rect.x) / rect_width;
                let v = (y as f32 - rect.y) / rect_height;
                
                let tex_uv = region.sample_uv(u, v);
                let tex_color = texture.sample(tex_uv);
                
                let final_color = if tint == Color::WHITE {
                    tex_color
                } else {
                    Color::new(
                        (tex_color.r as u16 * tint.r as u16 / 255) as u8,
                        (tex_color.g as u16 * tint.g as u16 / 255) as u8,
                        (tex_color.b as u16 * tint.b as u16 / 255) as u8,
                        (tex_color.a as u16 * tint.a as u16 / 255) as u8,
                    )
                };
                
                if final_color.a == 255 {
                    let index = (y * self.width + x) as usize;
                    if index < self.framebuffer.len() {
                        self.framebuffer[index] = final_color.to_u32();
                    }
                } else {
                    self.blend_pixel(x, y, final_color);
                }
            }
        }
    }

    fn fill_rect_transformed(&mut self, position: Vec2, size: Vec2, region: TextureRegion, rotation: f32, color: Color) {
        let texture = self.textures.get(&region.texture_id).cloned();
        if let Some(texture) = texture {
            self.fill_rect_textured_transformed(position, size, &texture, &region, rotation, color);
        } else {
            let cos_r = libm::cosf(rotation);
            let sin_r = libm::sinf(rotation);

            let cx = position.x + size.x / 2.0;
            let cy = position.y + size.y / 2.0;

            let corners = [
                Vec2::new(-size.x / 2.0, -size.y / 2.0),
                Vec2::new(size.x / 2.0, -size.y / 2.0),
                Vec2::new(size.x / 2.0, size.y / 2.0),
                Vec2::new(-size.x / 2.0, size.y / 2.0),
            ];

            let transformed: [Vec2; 4] = corners.map(|c| {
                Vec2::new(
                    cx + c.x * cos_r - c.y * sin_r,
                    cy + c.x * sin_r + c.y * cos_r,
                )
            });

            self.fill_polygon(&transformed, color);
        }
    }

    fn fill_rect_textured_transformed(&mut self, position: Vec2, size: Vec2, texture: &Texture, region: &TextureRegion, rotation: f32, tint: Color) {
        let cos_r = libm::cosf(rotation);
        let sin_r = libm::sinf(rotation);

        let cx = position.x + size.x / 2.0;
        let cy = position.y + size.y / 2.0;

        let corners = [
            Vec2::new(-size.x / 2.0, -size.y / 2.0),
            Vec2::new(size.x / 2.0, -size.y / 2.0),
            Vec2::new(size.x / 2.0, size.y / 2.0),
            Vec2::new(-size.x / 2.0, size.y / 2.0),
        ];

        let transformed: [Vec2; 4] = corners.map(|c| {
            Vec2::new(
                cx + c.x * cos_r - c.y * sin_r,
                cy + c.x * sin_r + c.y * cos_r,
            )
        });

        let min_x = transformed.iter().map(|p| p.x).fold(f32::INFINITY, f32::min).max(self.viewport.x) as u32;
        let max_x = transformed.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max).min(self.viewport.x + self.viewport.width) as u32;
        let min_y = transformed.iter().map(|p| p.y).fold(f32::INFINITY, f32::min).max(self.viewport.y) as u32;
        let max_y = transformed.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max).min(self.viewport.y + self.viewport.height) as u32;

        let half_size_x = size.x / 2.0;
        let half_size_y = size.y / 2.0;

        for y in min_y..max_y {
            for x in min_x..max_x {
                let px = x as f32 - cx;
                let py = y as f32 - cy;
                
                let local_x = px * cos_r + py * sin_r;
                let local_y = -px * sin_r + py * cos_r;
                
                if local_x < -half_size_x || local_x > half_size_x || local_y < -half_size_y || local_y > half_size_y {
                    continue;
                }
                
                let u = (local_x + half_size_x) / size.x;
                let v = (local_y + half_size_y) / size.y;
                
                let tex_uv = region.sample_uv(u, v);
                let tex_color = texture.sample(tex_uv);
                
                let final_color = if tint == Color::WHITE {
                    tex_color
                } else {
                    Color::new(
                        (tex_color.r as u16 * tint.r as u16 / 255) as u8,
                        (tex_color.g as u16 * tint.g as u16 / 255) as u8,
                        (tex_color.b as u16 * tint.b as u16 / 255) as u8,
                        (tex_color.a as u16 * tint.a as u16 / 255) as u8,
                    )
                };
                
                self.blend_pixel(x, y, final_color);
            }
        }
    }
}

impl Default for SoftwareBackend {
    fn default() -> Self {
        Self::new(800, 600)
    }
}
