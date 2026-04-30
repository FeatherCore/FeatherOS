use crate::{raster::rgb565, BlendMode, Color, ImageFormat, ImageId, MaskKind, MaskSpec, PixelFormat, Point, Surface};

impl Surface {
    pub(crate) fn put_pixel_blend(&mut self, x: i32, y: i32, color: Color, blend: BlendMode) {
        if blend == BlendMode::Normal {
            self.put_pixel(x, y, color);
            return;
        }
        if !self.is_valid() || x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        if let Some(clip) = self.clip {
            if !clip.contains_point(crate::Point::new(x, y)) {
                return;
            }
        }

        let Some(color) = self.apply_masks_to_color(crate::Point::new(x, y), color) else {
            return;
        };

        unsafe {
            let offset = y as usize * self.stride + x as usize * (self.bpp as usize / 8);
            let ptr = self.pixels.add(offset);
            let dst = self.read_pixel_ptr(ptr);
            let mixed = blend_color(color, dst, blend);
            self.write_pixel_ptr(ptr, mixed);
        }
    }

    pub(crate) fn apply_masks_to_color(&self, point: Point, color: Color) -> Option<Color> {
        if color.a == 0 {
            return None;
        }
        if self.mask_len == 0 {
            return Some(color);
        }
        let alpha = self.mask_alpha(point);
        if alpha == 0 {
            return None;
        }
        Some(Color::rgba(
            color.r,
            color.g,
            color.b,
            ((color.a as u16 * alpha as u16) / 255) as u8,
        ))
    }

    pub(crate) fn mask_alpha(&self, point: Point) -> u8 {
        let mut alpha = 255u16;
        let mut i = 0;
        while i < self.mask_len {
            if let Some(spec) = self.mask_stack[i] {
                let mut mask = self.mask_alpha_for_spec(spec, point);
                if spec.inverted {
                    mask = 255u8.saturating_sub(mask);
                }
                mask = ((mask as u16 * spec.opacity as u16) / 255) as u8;
                alpha = (alpha * mask as u16) / 255;
                if alpha == 0 {
                    return 0;
                }
            }
            i += 1;
        }
        alpha as u8
    }

    pub(crate) fn mask_alpha_for_spec(&self, spec: MaskSpec, point: Point) -> u8 {
        if spec.area.is_empty() || !spec.area.contains_point(point) {
            return 0;
        }
        match spec.kind {
            MaskKind::Rect => 255,
            MaskKind::RoundedRect => rounded_rect_mask_alpha(spec.area, spec.radius, point),
            MaskKind::Bitmap { image } => self.bitmap_mask_alpha(spec.area, image, point),
        }
    }

    fn bitmap_mask_alpha(&self, rect: crate::Rect, image: ImageId, point: Point) -> u8 {
        let view = if let Some(resolve) = self.image_resolver {
            resolve(image)
        } else {
            None
        };
        let Some(view) = view else {
            return 255;
        };
        if view.width == 0 || view.height == 0 {
            return 255;
        }
        let sx = ((point.x - rect.x)
            .max(0)
            .saturating_mul(view.width as i32)
            / rect.w.max(1) as i32)
            .min(view.width as i32 - 1) as u16;
        let sy = ((point.y - rect.y)
            .max(0)
            .saturating_mul(view.height as i32)
            / rect.h.max(1) as i32)
            .min(view.height as i32 - 1) as u16;
        match view.format {
            ImageFormat::A8 | ImageFormat::Rgba8888 => view.sample(sx, sy, 255).map(|c| c.a).unwrap_or(255),
            ImageFormat::Rgb565 | ImageFormat::Rgb888 => view.sample(sx, sy, 255).map(|c| {
                ((c.r as u16 + c.g as u16 + c.b as u16) / 3) as u8
            }).unwrap_or(255),
        }
    }

    pub(crate) unsafe fn read_pixel_ptr(&self, ptr: *mut u8) -> Color {
        match self.format {
            PixelFormat::Rgb565 => {
                let raw = core::ptr::read_unaligned(ptr as *const u16);
                let r = (((raw >> 11) & 0x1f) * 255 / 31) as u8;
                let g = (((raw >> 5) & 0x3f) * 255 / 63) as u8;
                let b = ((raw & 0x1f) * 255 / 31) as u8;
                Color::rgb(r, g, b)
            }
            PixelFormat::Rgb888 => Color::rgb(*ptr, *ptr.add(1), *ptr.add(2)),
            PixelFormat::Rgba8888 | PixelFormat::Unknown(_) => {
                Color::rgba(*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3))
            }
        }
    }

    pub(crate) unsafe fn write_pixel_ptr(&self, ptr: *mut u8, color: Color) {
        match self.format {
            PixelFormat::Rgb565 => {
                let r = (color.r as u16 >> 3) & 0x1f;
                let g = (color.g as u16 >> 2) & 0x3f;
                let b = (color.b as u16 >> 3) & 0x1f;
                core::ptr::write_unaligned(ptr as *mut u16, (r << 11) | (g << 5) | b);
            }
            PixelFormat::Rgb888 => {
                *ptr = color.r;
                *ptr.add(1) = color.g;
                *ptr.add(2) = color.b;
            }
            PixelFormat::Rgba8888 | PixelFormat::Unknown(_) => {
                *ptr = color.r;
                *ptr.add(1) = color.g;
                *ptr.add(2) = color.b;
                *ptr.add(3) = color.a;
            }
        }
    }

    pub(crate) fn fill_rect_opaque_fast(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) -> bool {
        match self.format {
            PixelFormat::Rgb565 => {
                let raw = rgb565(color);
                for y in y0..y1 {
                    unsafe {
                        let mut ptr = self.pixels.add(y as usize * self.stride + x0 as usize * 2) as *mut u16;
                        for _ in x0..x1 {
                            core::ptr::write_unaligned(ptr, raw);
                            ptr = ptr.add(1);
                        }
                    }
                }
                true
            }
            PixelFormat::Rgb888 => {
                for y in y0..y1 {
                    unsafe {
                        let mut ptr = self.pixels.add(y as usize * self.stride + x0 as usize * 3);
                        for _ in x0..x1 {
                            *ptr = color.r;
                            *ptr.add(1) = color.g;
                            *ptr.add(2) = color.b;
                            ptr = ptr.add(3);
                        }
                    }
                }
                true
            }
            PixelFormat::Rgba8888 | PixelFormat::Unknown(_) => {
                for y in y0..y1 {
                    unsafe {
                        let mut ptr = self.pixels.add(y as usize * self.stride + x0 as usize * 4);
                        for _ in x0..x1 {
                            *ptr = color.r;
                            *ptr.add(1) = color.g;
                            *ptr.add(2) = color.b;
                            *ptr.add(3) = color.a;
                            ptr = ptr.add(4);
                        }
                    }
                }
                true
            }
        }
    }

    pub(crate) fn fill_rect_alpha_fast(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) -> bool {
        if color.a == 0 {
            return true;
        }

        match self.format {
            PixelFormat::Rgb565 => {
                let src_r = color.r as u32;
                let src_g = color.g as u32;
                let src_b = color.b as u32;
                let alpha = color.a as u32;
                let inv_alpha = 255u32.saturating_sub(alpha);
                for y in y0..y1 {
                    unsafe {
                        let mut ptr = self.pixels.add(y as usize * self.stride + x0 as usize * 2) as *mut u16;
                        for _ in x0..x1 {
                            let raw = core::ptr::read_unaligned(ptr);
                            let dst_r = (((raw >> 11) & 0x1f) as u32 * 255) / 31;
                            let dst_g = (((raw >> 5) & 0x3f) as u32 * 255) / 63;
                            let dst_b = ((raw & 0x1f) as u32 * 255) / 31;
                            let r = (src_r * alpha + dst_r * inv_alpha) / 255;
                            let g = (src_g * alpha + dst_g * inv_alpha) / 255;
                            let b = (src_b * alpha + dst_b * inv_alpha) / 255;
                            let out = (((r as u16 >> 3) & 0x1f) << 11)
                                | (((g as u16 >> 2) & 0x3f) << 5)
                                | ((b as u16 >> 3) & 0x1f);
                            core::ptr::write_unaligned(ptr, out);
                            ptr = ptr.add(1);
                        }
                    }
                }
                true
            }
            PixelFormat::Rgb888 | PixelFormat::Rgba8888 | PixelFormat::Unknown(_) => {
                let step = self.bpp as usize / 8;
                if step == 0 {
                    return false;
                }
                for y in y0..y1 {
                    unsafe {
                        let mut ptr = self.pixels.add(y as usize * self.stride + x0 as usize * step);
                        for _ in x0..x1 {
                            let out = color.over(self.read_pixel_ptr(ptr));
                            self.write_pixel_ptr(ptr, out);
                            ptr = ptr.add(step);
                        }
                    }
                }
                true
            }
        }
    }
}

fn rounded_rect_mask_alpha(rect: crate::Rect, radius: u16, point: Point) -> u8 {
    if radius == 0 {
        return 255;
    }
    let radius = (radius as i32).min(rect.w as i32 / 2).min(rect.h as i32 / 2);
    if radius <= 0 {
        return 255;
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
        return 255;
    }
    let dx = if in_left { left_center - point.x } else { point.x - right_center };
    let dy = if in_top { top_center - point.y } else { point.y - bottom_center };
    if dx.saturating_mul(dx) + dy.saturating_mul(dy) <= radius.saturating_mul(radius) {
        255
    } else {
        0
    }
}

fn blend_color(src: Color, dst: Color, mode: BlendMode) -> Color {
    if src.a == 0 {
        return dst;
    }
    let r = blend_channel(src.r, dst.r, mode);
    let g = blend_channel(src.g, dst.g, mode);
    let b = blend_channel(src.b, dst.b, mode);
    Color::rgba(r, g, b, src.a).over(dst)
}

fn blend_channel(src: u8, dst: u8, mode: BlendMode) -> u8 {
    match mode {
        BlendMode::Normal => src,
        BlendMode::Additive => src.saturating_add(dst),
        BlendMode::Subtractive => dst.saturating_sub(src),
        BlendMode::Multiply => ((src as u16 * dst as u16) / 255) as u8,
        BlendMode::Difference => src.abs_diff(dst),
    }
}
