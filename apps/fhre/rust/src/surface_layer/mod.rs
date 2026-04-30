use alloc::vec::Vec;

use crate::{
    Color, DrawBackendDispatch, DrawCommand, LayerSpec, MaskSpec, Point, Rect, RenderBackend,
    RenderStats, ShadowStyle, Surface,
};

pub struct LayerScratch {
    pixels: Vec<u8>,
    width: u16,
    height: u16,
    stride: usize,
}

impl LayerScratch {
    pub fn new(width: u16, height: u16, budget_bytes: usize) -> Option<Self> {
        if width == 0 || height == 0 {
            return None;
        }
        let stride = width as usize * 4;
        let bytes = stride.checked_mul(height as usize)?;
        if bytes > budget_bytes {
            return None;
        }
        let mut pixels = Vec::new();
        if pixels.try_reserve_exact(bytes).is_err() {
            return None;
        }
        pixels.resize(bytes, 0);
        Some(Self {
            pixels,
            width,
            height,
            stride,
        })
    }

    pub const fn bytes(&self) -> usize {
        self.stride * self.height as usize
    }

    fn surface(&mut self) -> Surface {
        let mut surface = unsafe {
            Surface::from_raw(
                self.pixels.as_mut_ptr(),
                self.width,
                self.height,
                self.stride,
                32,
                0,
            )
        };
        surface.set_layer_budget(crate::LayerBudget {
            max_bytes: self.bytes(),
        });
        surface
    }
}

impl Surface {
    pub(crate) fn draw_blurred_shadow(&mut self, rect: Rect, style: ShadowStyle) -> bool {
        let spread = style.spread as i32;
        let base = Rect::from_edges(
            rect.x + style.offset.x - spread,
            rect.y + style.offset.y - spread,
            rect.right() + style.offset.x + spread,
            rect.bottom() + style.offset.y + spread,
        );
        let blur = style.width.min(32) as i32;
        let shadow = Rect::from_edges(
            base.x - blur,
            base.y - blur,
            base.right() + blur,
            base.bottom() + blur,
        )
        .clipped_to(Rect::new(0, 0, self.width(), self.height()));
        if shadow.is_empty() {
            return true;
        }

        let Some(mut scratch) = LayerScratch::new(shadow.w, shadow.h, self.layer_budget.max_bytes)
        else {
            return false;
        };
        fill_shadow_mask(&mut scratch.pixels, shadow, base, style);
        blur_rgba(
            &mut scratch.pixels,
            shadow.w,
            shadow.h,
            scratch.stride,
            style.width.min(32),
        );
        self.composite_layer(&scratch.pixels, shadow, scratch.stride);
        true
    }

    pub(crate) fn draw_layer_commands_impl(
        &mut self,
        rect: Rect,
        spec: LayerSpec,
        cmds: &[DrawCommand],
        clips: &[Option<Rect>],
        stats: &mut RenderStats,
    ) -> bool {
        let screen = Rect::new(0, 0, self.width(), self.height());
        let target = rect.clipped_to(screen);
        if target.is_empty() || spec.opacity == 0 {
            return true;
        }

        let Some(mut scratch) = LayerScratch::new(target.w, target.h, self.layer_budget.max_bytes)
        else {
            stats.mark_layer_alloc_failure();
            self.draw_layer_fallback(target, spec);
            return false;
        };
        let scratch_bytes = scratch.bytes().min(u32::MAX as usize) as u32;
        stats.mark_layer_bytes(scratch_bytes);

        let mut layer = scratch.surface();
        layer.set_image_resolver(self.image_resolver);
        layer.set_glyph_resolver(self.glyph_resolver);
        layer.set_glyph_id_resolver(self.glyph_id_resolver);
        layer.set_glyph_run_resolver(self.glyph_run_resolver);
        layer.set_kerning_resolver(self.kerning_resolver);
        layer.set_svg_resolver(self.svg_resolver);
        layer.clear(Color::TRANSPARENT);

        self.execute_layer_inner(&mut layer, target, cmds, clips, stats);

        if spec.blur_radius != 0 {
            let pixels = (target.w as u32).saturating_mul(target.h as u32);
            stats.mark_blur_pixels(pixels);
            blur_rgba(
                &mut scratch.pixels,
                target.w,
                target.h,
                scratch.stride,
                spec.blur_radius.min(32),
            );
        }

        apply_layer_effects(
            &mut scratch.pixels,
            target.w,
            target.h,
            scratch.stride,
            spec,
            target,
            self,
        );
        self.composite_layer(&scratch.pixels, target, scratch.stride);
        true
    }

    fn execute_layer_inner(
        &self,
        layer: &mut Surface,
        target: Rect,
        cmds: &[DrawCommand],
        clips: &[Option<Rect>],
        stats: &mut RenderStats,
    ) {
        let dispatch = DrawBackendDispatch::from_capabilities(layer.capabilities());
        let mut i = 0usize;
        while i < cmds.len() {
            let cmd = cmds[i];
            match cmd {
                DrawCommand::EndLayer => break,
                DrawCommand::BeginLayer { rect, spec, .. } => {
                    let end = find_layer_end(cmds, i + 1);
                    let local_rect = translate_rect(rect, -target.x, -target.y);
                    let translated_spec = translate_layer_spec(spec, -target.x, -target.y);
                    let path = dispatch.classify(crate::DrawTaskKind::Layer);
                    let _ = layer.draw_dispatched_layer_commands(
                        local_rect,
                        translated_spec,
                        &cmds[i + 1..end],
                        &clips[i + 1..end],
                        path,
                        stats,
                    );
                    stats.mark_command_kind(cmd);
                    i = end.saturating_add(1);
                    continue;
                }
                DrawCommand::PushMask { spec, .. } => {
                    let spec = translate_mask_spec(spec, -target.x, -target.y);
                    if !layer.push_mask(spec) {
                        stats.mark_mask_stack_overflow();
                    }
                    stats.mark_draw_dispatch_for(
                        crate::DrawTaskKind::MaskRect,
                        dispatch.classify(crate::DrawTaskKind::MaskRect),
                    );
                    stats.mark_command_kind(cmd);
                    i += 1;
                    continue;
                }
                DrawCommand::PushBitmapMask {
                    rect,
                    image,
                    inverted,
                    opacity,
                    ..
                } => {
                    let spec = MaskSpec::bitmap(translate_rect(rect, -target.x, -target.y), image)
                        .inverted(inverted)
                        .opacity(opacity);
                    if !layer.push_mask(spec) {
                        stats.mark_mask_stack_overflow();
                    }
                    stats.mark_draw_dispatch_for(
                        crate::DrawTaskKind::MaskBitmap,
                        dispatch.classify(crate::DrawTaskKind::MaskBitmap),
                    );
                    stats.mark_command_kind(cmd);
                    i += 1;
                    continue;
                }
                DrawCommand::PopMask => {
                    let _ = layer.pop_mask();
                    i += 1;
                    continue;
                }
                _ => {}
            }

            let translated = cmd.translated(-target.x, -target.y);
            let clip = clips
                .get(i)
                .copied()
                .flatten()
                .map(|clip| translate_rect(clip.clipped_to(target), -target.x, -target.y));
            layer.set_clip(clip);
            stats.command_seen();
            if let Some(kind) = translated.task_kind() {
                layer.draw_dispatched_command(translated, dispatch.classify(kind), stats);
            } else {
                layer.draw_command(translated);
            }
            stats.mark_command_kind(cmd);
            stats.command_drawn(cmd.bounds().map(|bounds| bounds.clipped_to(target)));
            i += 1;
        }
        layer.set_clip(None);
        layer.clear_masks();
    }

    fn composite_layer(&mut self, pixels: &[u8], target: Rect, stride: usize) {
        let old_clip = self.clip();
        let clip = match old_clip {
            Some(clip) => target.clipped_to(clip),
            None => target,
        };
        if clip.is_empty() {
            return;
        }
        let mut y = clip.y;
        while y < clip.bottom() {
            let sy = (y - target.y) as usize;
            let mut x = clip.x;
            while x < clip.right() {
                let sx = (x - target.x) as usize;
                let off = sy * stride + sx * 4;
                if off + 3 < pixels.len() {
                    let color = Color::rgba(
                        pixels[off],
                        pixels[off + 1],
                        pixels[off + 2],
                        pixels[off + 3],
                    );
                    if color.a != 0 {
                        self.put_pixel(x, y, color);
                    }
                }
                x += 1;
            }
            y += 1;
        }
    }
}

fn fill_shadow_mask(pixels: &mut [u8], shadow: Rect, base: Rect, style: ShadowStyle) {
    let mut y = 0usize;
    while y < shadow.h as usize {
        let global_y = shadow.y + y as i32;
        let mut x = 0usize;
        while x < shadow.w as usize {
            let global_x = shadow.x + x as i32;
            let off = y * shadow.w as usize * 4 + x * 4;
            if off + 3 >= pixels.len() {
                return;
            }
            let point = Point::new(global_x, global_y);
            if rounded_rect_contains(
                base,
                style.radius.saturating_add(style.spread.max(0) as u16),
                point,
            ) {
                pixels[off] = style.color.r;
                pixels[off + 1] = style.color.g;
                pixels[off + 2] = style.color.b;
                pixels[off + 3] = style.color.a;
            }
            x += 1;
        }
        y += 1;
    }
}

fn rounded_rect_contains(rect: Rect, radius: u16, point: Point) -> bool {
    if rect.is_empty() || !rect.contains_point(point) {
        return false;
    }
    let radius = (radius as i32)
        .min(rect.w as i32 / 2)
        .min(rect.h as i32 / 2);
    if radius <= 0 {
        return true;
    }
    let left_center = rect.x + radius;
    let right_center = rect.right() - radius - 1;
    let top_center = rect.y + radius;
    let bottom_center = rect.bottom() - radius - 1;
    let in_left = point.x < left_center;
    let in_right = point.x > right_center;
    let in_top = point.y < top_center;
    let in_bottom = point.y > bottom_center;
    if !(in_left || in_right) || !(in_top || in_bottom) {
        return true;
    }
    let dx = if in_left {
        left_center - point.x
    } else {
        point.x - right_center
    };
    let dy = if in_top {
        top_center - point.y
    } else {
        point.y - bottom_center
    };
    dx.saturating_mul(dx) + dy.saturating_mul(dy) <= radius.saturating_mul(radius)
}

fn apply_layer_effects(
    pixels: &mut [u8],
    width: u16,
    height: u16,
    stride: usize,
    spec: LayerSpec,
    target: Rect,
    parent: &Surface,
) {
    let mut y = 0usize;
    while y < height as usize {
        let mut x = 0usize;
        while x < width as usize {
            let off = y * stride + x * 4;
            if off + 3 >= pixels.len() {
                return;
            }
            let mut a = pixels[off + 3];
            if a != 0 {
                if let Some(mask) = spec.mask {
                    let global = Point::new(target.x + x as i32, target.y + y as i32);
                    let mut mask_alpha = parent.mask_alpha_for_spec(mask, global);
                    if mask.inverted {
                        mask_alpha = 255u8.saturating_sub(mask_alpha);
                    }
                    mask_alpha = ((mask_alpha as u16 * mask.opacity as u16) / 255) as u8;
                    a = ((a as u16 * mask_alpha as u16) / 255) as u8;
                }
                a = ((a as u16 * spec.opacity as u16) / 255) as u8;
                if let Some(recolor) = spec.recolor {
                    let mix = recolor.a as u16;
                    let inv = 255u16.saturating_sub(mix);
                    pixels[off] = ((pixels[off] as u16 * inv + recolor.r as u16 * mix) / 255) as u8;
                    pixels[off + 1] =
                        ((pixels[off + 1] as u16 * inv + recolor.g as u16 * mix) / 255) as u8;
                    pixels[off + 2] =
                        ((pixels[off + 2] as u16 * inv + recolor.b as u16 * mix) / 255) as u8;
                }
                pixels[off + 3] = a;
            }
            x += 1;
        }
        y += 1;
    }
}

fn blur_rgba(pixels: &mut [u8], width: u16, height: u16, stride: usize, radius: u16) {
    if radius == 0 || width == 0 || height == 0 {
        return;
    }
    let bytes = stride.saturating_mul(height as usize);
    let mut temp = Vec::new();
    if temp.try_reserve_exact(bytes).is_err() {
        return;
    }
    temp.resize(bytes, 0);
    box_blur_horizontal(pixels, &mut temp, width, height, stride, radius as i32);
    box_blur_vertical(&temp, pixels, width, height, stride, radius as i32);
}

fn box_blur_horizontal(
    src: &[u8],
    dst: &mut [u8],
    width: u16,
    height: u16,
    stride: usize,
    radius: i32,
) {
    let width = width as i32;
    let height = height as i32;
    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let mut sum = [0u32; 4];
            let mut count = 0u32;
            let mut k = -radius;
            while k <= radius {
                let sx = (x + k).clamp(0, width - 1) as usize;
                let off = y as usize * stride + sx * 4;
                if off + 3 < src.len() {
                    sum[0] += src[off] as u32;
                    sum[1] += src[off + 1] as u32;
                    sum[2] += src[off + 2] as u32;
                    sum[3] += src[off + 3] as u32;
                    count += 1;
                }
                k += 1;
            }
            let off = y as usize * stride + x as usize * 4;
            if count != 0 && off + 3 < dst.len() {
                dst[off] = (sum[0] / count) as u8;
                dst[off + 1] = (sum[1] / count) as u8;
                dst[off + 2] = (sum[2] / count) as u8;
                dst[off + 3] = (sum[3] / count) as u8;
            }
            x += 1;
        }
        y += 1;
    }
}

fn box_blur_vertical(
    src: &[u8],
    dst: &mut [u8],
    width: u16,
    height: u16,
    stride: usize,
    radius: i32,
) {
    let width = width as i32;
    let height = height as i32;
    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < width {
            let mut sum = [0u32; 4];
            let mut count = 0u32;
            let mut k = -radius;
            while k <= radius {
                let sy = (y + k).clamp(0, height - 1) as usize;
                let off = sy * stride + x as usize * 4;
                if off + 3 < src.len() {
                    sum[0] += src[off] as u32;
                    sum[1] += src[off + 1] as u32;
                    sum[2] += src[off + 2] as u32;
                    sum[3] += src[off + 3] as u32;
                    count += 1;
                }
                k += 1;
            }
            let off = y as usize * stride + x as usize * 4;
            if count != 0 && off + 3 < dst.len() {
                dst[off] = (sum[0] / count) as u8;
                dst[off + 1] = (sum[1] / count) as u8;
                dst[off + 2] = (sum[2] / count) as u8;
                dst[off + 3] = (sum[3] / count) as u8;
            }
            x += 1;
        }
        y += 1;
    }
}

fn translate_rect(rect: Rect, dx: i32, dy: i32) -> Rect {
    Rect::new(
        rect.x.saturating_add(dx),
        rect.y.saturating_add(dy),
        rect.w,
        rect.h,
    )
}

fn translate_mask_spec(mut spec: MaskSpec, dx: i32, dy: i32) -> MaskSpec {
    spec.area = translate_rect(spec.area, dx, dy);
    spec
}

fn translate_layer_spec(mut spec: LayerSpec, dx: i32, dy: i32) -> LayerSpec {
    if let Some(mask) = spec.mask {
        spec.mask = Some(translate_mask_spec(mask, dx, dy));
    }
    spec
}

fn find_layer_end(cmds: &[DrawCommand], start: usize) -> usize {
    let mut depth = 0usize;
    let mut i = start;
    while i < cmds.len() {
        match cmds[i] {
            DrawCommand::BeginLayer { .. } => depth = depth.saturating_add(1),
            DrawCommand::EndLayer => {
                if depth == 0 {
                    return i;
                }
                depth -= 1;
            }
            _ => {}
        }
        i += 1;
    }
    cmds.len()
}
