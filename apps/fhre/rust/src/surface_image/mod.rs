use crate::{
    image::{builtin_image, ImageFormat, ImageView},
    raster::{clamp_i32, rgb565},
    surface_pixels::blend_rgb565_raw,
    BlendMode, Color, ImageDrawStyle, ImageFit, ImageId, Point, Rect, Surface,
    TexCoord,
};

impl Surface {
    pub fn draw_image_id(&mut self, rect: Rect, image: ImageId, opacity: u8) {
        self.draw_image_id_fit(rect, image, opacity, ImageFit::Stretch);
    }

    pub fn draw_image_id_fit(&mut self, rect: Rect, image: ImageId, opacity: u8, fit: ImageFit) {
        if let Some(view) = builtin_image(image) {
            self.draw_image_view_fit(rect, view, opacity, fit);
        } else if let Some(resolve) = self.image_resolver {
            if let Some(view) = resolve(image) {
                self.draw_image_view_fit(rect, view, opacity, fit);
                return;
            }
            self.draw_missing_image(rect, opacity, None);
        } else {
            self.draw_missing_image(rect, opacity, None);
        }
    }

    pub fn draw_image_id_tint(
        &mut self,
        rect: Rect,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
        tint: Color,
    ) {
        if let Some(view) = builtin_image(image) {
            self.draw_image_view_tint(rect, view, opacity, fit, tint);
        } else if let Some(resolve) = self.image_resolver {
            if let Some(view) = resolve(image) {
                self.draw_image_view_tint(rect, view, opacity, fit, tint);
                return;
            }
            self.draw_missing_image(rect, opacity, Some(tint));
        } else {
            self.draw_missing_image(rect, opacity, Some(tint));
        }
    }

    pub fn draw_image_id_styled(&mut self, rect: Rect, image: ImageId, style: ImageDrawStyle) {
        if style.opacity == 0 || rect.is_empty() {
            return;
        }

        let view = if let Some(view) = builtin_image(image) {
            Some(view)
        } else if let Some(resolve) = self.image_resolver {
            resolve(image)
        } else {
            None
        };

        let Some(view) = view else {
            self.draw_missing_image(rect, style.opacity, style.tint);
            return;
        };

        let pushed_radius_mask = style.clip_radius != 0
            && self.push_mask(crate::MaskSpec::rounded(rect, style.clip_radius));

        if style.tile {
            let old_clip = self.clip();
            self.set_clip(Some(match old_clip {
                Some(clip) => rect.clipped_to(clip),
                None => rect,
            }));
            let tw = view.width.max(1) as i32;
            let th = view.height.max(1) as i32;
            let mut y = rect.y;
            while y < rect.bottom() {
                let mut x = rect.x;
                while x < rect.right() {
                    let tile = Rect::new(x, y, view.width, view.height);
                    self.draw_image_view_mapped(
                        tile,
                        view,
                        style.opacity,
                        ImageFit::Stretch,
                        style.tint,
                        style.blend,
                    );
                    x += tw;
                }
                y += th;
            }
            self.set_clip(old_clip);
        } else {
            self.draw_image_view_mapped(
                rect,
                view,
                style.opacity,
                style.fit,
                style.tint,
                style.blend,
            );
        }

        if pushed_radius_mask {
            let _ = self.pop_mask();
        }
    }

    pub fn draw_image_view(&mut self, rect: Rect, image: ImageView, opacity: u8) {
        self.draw_image_view_fit(rect, image, opacity, ImageFit::Stretch);
    }

    pub fn draw_image_view_fit(
        &mut self,
        rect: Rect,
        image: ImageView,
        opacity: u8,
        fit: ImageFit,
    ) {
        self.draw_image_view_mapped(rect, image, opacity, fit, None, BlendMode::Normal);
    }

    pub fn draw_image_view_tint(
        &mut self,
        rect: Rect,
        image: ImageView,
        opacity: u8,
        fit: ImageFit,
        tint: Color,
    ) {
        self.draw_image_view_mapped(rect, image, opacity, fit, Some(tint), BlendMode::Normal);
    }

    fn draw_image_view_mapped(
        &mut self,
        rect: Rect,
        image: ImageView,
        opacity: u8,
        fit: ImageFit,
        tint: Option<Color>,
        blend: BlendMode,
    ) {
        if !self.is_valid()
            || rect.is_empty()
            || image.width == 0
            || image.height == 0
            || opacity == 0
        {
            return;
        }

        let clip_rect = match self.clip {
            Some(clip) => rect.clipped_to(clip),
            None => rect,
        };
        let (dst_rect, crop_rect) = image_fit_rect(rect, image.width, image.height, fit);
        let crop_rect = crop_rect.clipped_to(clip_rect);
        let x0 = clamp_i32(crop_rect.x, 0, self.width as i32);
        let y0 = clamp_i32(crop_rect.y, 0, self.height as i32);
        let x1 = clamp_i32(crop_rect.right(), 0, self.width as i32);
        let y1 = clamp_i32(crop_rect.bottom(), 0, self.height as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        if tint.is_none()
            && blend == BlendMode::Normal
            && self.blit_rgb565_normal_fast(rect, dst_rect, image, fit, opacity, x0, y0, x1, y1, tint)
        {
            return;
        }

        let dst_w = dst_rect.w.max(1) as i32;
        let dst_h = dst_rect.h.max(1) as i32;
        let src_w = image.width as i32;
        let src_h = image.height as i32;
        for y in y0..y1 {
            let sy = ((y - dst_rect.y).saturating_mul(src_h) / dst_h)
                .max(0)
                .min(src_h - 1) as u16;
            for x in x0..x1 {
                let sx = ((x - dst_rect.x).saturating_mul(src_w) / dst_w)
                    .max(0)
                    .min(src_w - 1) as u16;
                let color = match tint {
                    Some(tint) => image.sample_tinted(sx, sy, opacity, tint),
                    None => image.sample(sx, sy, opacity),
                };
                if let Some(color) = color {
                    self.put_pixel_blend(x, y, color, blend);
                }
            }
        }
    }

    fn copy_rgb565_image_rows(
        &mut self,
        rect: Rect,
        image: ImageView,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    ) {
        let copy_width = (x1 - x0).max(0) as usize * 2;
        if copy_width == 0 || image.data.is_null() {
            return;
        }

        let src_x = (x0 - rect.x).max(0) as usize * 2;
        let dst_x = x0 as usize * 2;
        let mut y = y0;
        while y < y1 {
            let src_y = (y - rect.y).max(0) as usize;
            unsafe {
                let src = image.data.add(src_y * image.stride + src_x);
                let dst = self.pixels.add(y as usize * self.stride + dst_x);
                core::ptr::copy_nonoverlapping(src, dst, copy_width);
            }
            y += 1;
        }
    }

    fn copy_rgba8888_image_rows(
        &mut self,
        rect: Rect,
        image: ImageView,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    ) {
        let copy_width = (x1 - x0).max(0) as usize * 4;
        if copy_width == 0 || image.data.is_null() {
            return;
        }

        let src_x = (x0 - rect.x).max(0) as usize * 4;
        let dst_x = x0 as usize * 4;
        let mut y = y0;
        while y < y1 {
            let src_y = (y - rect.y).max(0) as usize;
            unsafe {
                let src = image.data.add(src_y * image.stride + src_x);
                let dst = self.pixels.add(y as usize * self.stride + dst_x);
                core::ptr::copy_nonoverlapping(src, dst, copy_width);
            }
            y += 1;
        }
    }

    fn copy_a8_as_rgba8888_rows(
        &mut self,
        rect: Rect,
        image: ImageView,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        r: u8,
        g: u8,
        b: u8,
    ) {
        let copy_width = (x1 - x0).max(0) as usize;
        if copy_width == 0 || image.data.is_null() {
            return;
        }

        let src_x = (x0 - rect.x).max(0) as usize;
        let dst_x = x0 as usize * 4;
        let mut y = y0;
        while y < y1 {
            let src_y = (y - rect.y).max(0) as usize;
            unsafe {
                let src = image.data.add(src_y * image.stride + src_x);
                let mut dst_ptr = self.pixels.add(y as usize * self.stride + dst_x);
                let mut x = 0;
                while x < copy_width {
                    let a = *src.add(x);
                    *dst_ptr = r;
                    *dst_ptr.add(1) = g;
                    *dst_ptr.add(2) = b;
                    *dst_ptr.add(3) = a;
                    dst_ptr = dst_ptr.add(4);
                    x += 1;
                }
            }
            y += 1;
        }
    }

    fn blit_rgb565_scaled_nearest(
        &mut self,
        dst_rect: Rect,
        image: ImageView,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    ) {
        if image.data.is_null() || dst_rect.w == 0 || dst_rect.h == 0 {
            return;
        }

        let dst_w = dst_rect.w.max(1) as i32;
        let dst_h = dst_rect.h.max(1) as i32;
        let src_w = image.width.max(1) as i32;
        let src_h = image.height.max(1) as i32;
        let mut y = y0;
        while y < y1 {
            let sy = ((y - dst_rect.y).saturating_mul(src_h) / dst_h)
                .max(0)
                .min(src_h - 1) as usize;
            let dst_y = y as usize * self.stride;
            let mut x = x0;
            while x < x1 {
                let sx = ((x - dst_rect.x).saturating_mul(src_w) / dst_w)
                    .max(0)
                    .min(src_w - 1) as usize;
                unsafe {
                    let src = image.data.add(sy * image.stride + sx * 2);
                    let dst = self.pixels.add(dst_y + x as usize * 2);
                    core::ptr::copy_nonoverlapping(src, dst, 2);
                }
                x += 1;
            }
            y += 1;
        }
    }

    fn blit_rgba8888_scaled_nearest(
        &mut self,
        dst_rect: Rect,
        image: ImageView,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    ) {
        if image.data.is_null() || dst_rect.w == 0 || dst_rect.h == 0 {
            return;
        }

        let dst_w = dst_rect.w.max(1) as i32;
        let dst_h = dst_rect.h.max(1) as i32;
        let src_w = image.width.max(1) as i32;
        let src_h = image.height.max(1) as i32;
        let mut y = y0;
        while y < y1 {
            let sy = ((y - dst_rect.y).saturating_mul(src_h) / dst_h)
                .max(0)
                .min(src_h - 1) as usize;
            let dst_y = y as usize * self.stride;
            let mut x = x0;
            while x < x1 {
                let sx = ((x - dst_rect.x).saturating_mul(src_w) / dst_w)
                    .max(0)
                    .min(src_w - 1) as usize;
                unsafe {
                    let src = image.data.add(sy * image.stride + sx * 4);
                    let dst = self.pixels.add(dst_y + x as usize * 4);
                    core::ptr::copy_nonoverlapping(src, dst, 4);
                }
                x += 1;
            }
            y += 1;
        }
    }

    fn blit_a8_scaled_as_rgba8888(
        &mut self,
        dst_rect: Rect,
        image: ImageView,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        r: u8,
        g: u8,
        b: u8,
    ) {
        if image.data.is_null() || dst_rect.w == 0 || dst_rect.h == 0 {
            return;
        }

        let dst_w = dst_rect.w.max(1) as i32;
        let dst_h = dst_rect.h.max(1) as i32;
        let src_w = image.width.max(1) as i32;
        let src_h = image.height.max(1) as i32;
        let mut y = y0;
        while y < y1 {
            let sy = ((y - dst_rect.y).saturating_mul(src_h) / dst_h)
                .max(0)
                .min(src_h - 1) as usize;
            let dst_y = y as usize * self.stride;
            let mut x = x0;
            while x < x1 {
                let sx = ((x - dst_rect.x).saturating_mul(src_w) / dst_w)
                    .max(0)
                    .min(src_w - 1) as usize;
                unsafe {
                    let a = *image.data.add(sy * image.stride + sx);
                    let dst = self.pixels.add(dst_y + x as usize * 4);
                    *dst = r;
                    *dst.add(1) = g;
                    *dst.add(2) = b;
                    *dst.add(3) = a;
                }
                x += 1;
            }
            y += 1;
        }
    }

    fn blit_rgb565_normal_fast(
        &mut self,
        rect: Rect,
        dst_rect: Rect,
        image: ImageView,
        fit: ImageFit,
        opacity: u8,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        tint: Option<Color>,
    ) -> bool {
        if image.data.is_null() || opacity == 0 || dst_rect.w == 0 || dst_rect.h == 0 {
            return true;
        }

        let has_masks = self.has_masks();
        let has_tint = tint.is_some();

        if image.format == ImageFormat::Rgb565 && opacity == 255 && !has_masks && !has_tint {
            if fit == ImageFit::Stretch && rect.w == image.width && rect.h == image.height {
                self.copy_rgb565_image_rows(rect, image, x0, y0, x1, y1);
            } else {
                self.blit_rgb565_scaled_nearest(dst_rect, image, x0, y0, x1, y1);
            }
            return true;
        }

        if image.format == ImageFormat::Rgba8888 && opacity == 255 && !has_masks && !has_tint {
            if fit == ImageFit::Stretch && rect.w == image.width && rect.h == image.height {
                self.copy_rgba8888_image_rows(rect, image, x0, y0, x1, y1);
            } else {
                self.blit_rgba8888_scaled_nearest(dst_rect, image, x0, y0, x1, y1);
            }
            return true;
        }

        let dst_w = dst_rect.w.max(1) as i32;
        let dst_h = dst_rect.h.max(1) as i32;
        let src_w = image.width.max(1) as i32;
        let src_h = image.height.max(1) as i32;
        let mut y = y0;
        while y < y1 {
            let sy = ((y - dst_rect.y).saturating_mul(src_h) / dst_h)
                .max(0)
                .min(src_h - 1) as usize;
            let dst_y = y as usize * self.stride;
            let mut x = x0;
            while x < x1 {
                let sx = ((x - dst_rect.x).saturating_mul(src_w) / dst_w)
                    .max(0)
                    .min(src_w - 1) as usize;
                if let Some((src_r, src_g, src_b, mut src_a)) =
                    image_rgba8888_at(image, sx, sy, opacity)
                {
                    if has_masks {
                        let mask = self.mask_alpha(Point::new(x, y)) as u32;
                        src_a = (src_a * mask) / 255;
                    }
                    if src_a != 0 {
                        unsafe {
                            let dst = self.pixels.add(dst_y + x as usize * 2) as *mut u16;
                            if src_a == 255 {
                                core::ptr::write_unaligned(
                                    dst,
                                    rgb565(Color::rgb(src_r as u8, src_g as u8, src_b as u8)),
                                );
                            } else {
                                let raw = core::ptr::read_unaligned(dst);
                                core::ptr::write_unaligned(
                                    dst,
                                    blend_rgb565_raw(raw, src_r, src_g, src_b, src_a),
                                );
                            }
                        }
                    }
                }
                x += 1;
            }
            y += 1;
        }
        true
    }

    pub fn fill_textured_triangle(
        &mut self,
        p0: Point,
        p1: Point,
        p2: Point,
        uv0: TexCoord,
        uv1: TexCoord,
        uv2: TexCoord,
        image: ImageId,
        opacity: u8,
    ) {
        if opacity == 0 {
            return;
        }
        let view = if let Some(view) = builtin_image(image) {
            Some(view)
        } else if let Some(resolve) = self.image_resolver {
            resolve(image)
        } else {
            None
        };
        let Some(view) = view else {
            let bounds = Rect::from_edges(
                p0.x.min(p1.x).min(p2.x),
                p0.y.min(p1.y).min(p2.y),
                p0.x.max(p1.x).max(p2.x) + 1,
                p0.y.max(p1.y).max(p2.y) + 1,
            );
            self.draw_missing_image(bounds, opacity, None);
            return;
        };

        let bounds = Rect::from_edges(
            p0.x.min(p1.x).min(p2.x),
            p0.y.min(p1.y).min(p2.y),
            p0.x.max(p1.x).max(p2.x) + 1,
            p0.y.max(p1.y).max(p2.y) + 1,
        );
        let clipped = match self.clip {
            Some(clip) => bounds.clipped_to(clip),
            None => bounds,
        };
        let x0 = clamp_i32(clipped.x, 0, self.width as i32);
        let y0 = clamp_i32(clipped.y, 0, self.height as i32);
        let x1 = clamp_i32(clipped.right(), 0, self.width as i32);
        let y1 = clamp_i32(clipped.bottom(), 0, self.height as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let area = edge(p0, p1, p2);
        if area == 0 {
            return;
        }
        let sign = if area < 0 { -1 } else { 1 };
        let area_abs = (area * sign) as i64;
        let src_w = view.width.saturating_sub(1).max(1) as i64;
        let src_h = view.height.saturating_sub(1).max(1) as i64;
        let mut y = y0;
        while y < y1 {
            let mut x = x0;
            while x < x1 {
                let p = Point::new(x, y);
                let w0 = edge(p1, p2, p) * sign;
                let w1 = edge(p2, p0, p) * sign;
                let w2 = edge(p0, p1, p) * sign;
                if w0 >= 0 && w1 >= 0 && w2 >= 0 {
                    let u = (w0 as i64 * uv0.u as i64
                        + w1 as i64 * uv1.u as i64
                        + w2 as i64 * uv2.u as i64)
                        / area_abs;
                    let v = (w0 as i64 * uv0.v as i64
                        + w1 as i64 * uv1.v as i64
                        + w2 as i64 * uv2.v as i64)
                        / area_abs;
                    let sx = ((u.clamp(0, 255) * src_w) / 255) as u16;
                    let sy = ((v.clamp(0, 255) * src_h) / 255) as u16;
                    if let Some(color) = view.sample(sx, sy, opacity) {
                        if color.a != 0 {
                            self.put_pixel(x, y, color);
                        }
                    }
                }
                x += 1;
            }
            y += 1;
        }
    }

    fn draw_missing_image(&mut self, rect: Rect, opacity: u8, tint: Option<Color>) {
        if rect.is_empty() || opacity == 0 {
            return;
        }
        let base = tint.unwrap_or(Color::rgba(96, 112, 128, opacity));
        let color = Color::rgba(
            base.r,
            base.g,
            base.b,
            ((base.a as u16 * opacity as u16) / 255) as u8,
        );
        self.fill_round_rect(
            rect,
            (rect.w.min(rect.h) / 5).max(2),
            Color::rgba(color.r, color.g, color.b, color.a / 2),
        );
        self.draw_wide_line(
            Point::new(rect.x, rect.y),
            Point::new(rect.right() - 1, rect.bottom() - 1),
            1,
            color,
        );
        self.draw_wide_line(
            Point::new(rect.right() - 1, rect.y),
            Point::new(rect.x, rect.bottom() - 1),
            1,
            color,
        );
    }
}

fn edge(a: Point, b: Point, c: Point) -> i32 {
    (c.x - a.x).saturating_mul(b.y - a.y) - (c.y - a.y).saturating_mul(b.x - a.x)
}

fn image_fit_rect(rect: Rect, src_w: u16, src_h: u16, fit: ImageFit) -> (Rect, Rect) {
    if rect.is_empty() || src_w == 0 || src_h == 0 {
        return (rect, Rect::EMPTY);
    }

    if fit == ImageFit::Stretch {
        return (rect, rect);
    }

    let rw = rect.w.max(1) as u32;
    let rh = rect.h.max(1) as u32;
    let sw = src_w.max(1) as u32;
    let sh = src_h.max(1) as u32;
    let width_limited = rw.saturating_mul(sh) <= rh.saturating_mul(sw);
    let use_width = match fit {
        ImageFit::Contain => width_limited,
        ImageFit::Cover => !width_limited,
        ImageFit::Stretch => true,
    };

    let (dw, dh) = if use_width {
        let dh = rw.saturating_mul(sh).saturating_add(sw / 2) / sw;
        (rw, dh.max(1))
    } else {
        let dw = rh.saturating_mul(sw).saturating_add(sh / 2) / sh;
        (dw.max(1), rh)
    };

    let dw = dw.min(u16::MAX as u32) as u16;
    let dh = dh.min(u16::MAX as u32) as u16;
    let x = rect.x + (rect.w as i32 - dw as i32) / 2;
    let y = rect.y + (rect.h as i32 - dh as i32) / 2;
    let dst = Rect::new(x, y, dw, dh);
    let crop = match fit {
        ImageFit::Cover => rect,
        ImageFit::Contain => dst,
        ImageFit::Stretch => rect,
    };
    (dst, crop)
}

fn image_rgba8888_at(
    image: ImageView,
    x: usize,
    y: usize,
    opacity: u8,
) -> Option<(u32, u32, u32, u32)> {
    match image.format {
        ImageFormat::Rgb565 => {
            let offset = y
                .checked_mul(image.stride)?
                .checked_add(x.checked_mul(2)?)?;
            let lo = image_byte(image, offset)?;
            let hi = image_byte(image, offset + 1)?;
            let raw = u16::from_le_bytes([lo, hi]);
            Some((
                (((raw >> 11) & 0x1f) as u32 * 255) / 31,
                (((raw >> 5) & 0x3f) as u32 * 255) / 63,
                ((raw & 0x1f) as u32 * 255) / 31,
                opacity as u32,
            ))
        }
        ImageFormat::Rgb888 => {
            let offset = y
                .checked_mul(image.stride)?
                .checked_add(x.checked_mul(3)?)?;
            Some((
                image_byte(image, offset)? as u32,
                image_byte(image, offset + 1)? as u32,
                image_byte(image, offset + 2)? as u32,
                opacity as u32,
            ))
        }
        ImageFormat::Rgba8888 => {
            let offset = y
                .checked_mul(image.stride)?
                .checked_add(x.checked_mul(4)?)?;
            let alpha = ((image_byte(image, offset + 3)? as u32 * opacity as u32) / 255).min(255);
            Some((
                image_byte(image, offset)? as u32,
                image_byte(image, offset + 1)? as u32,
                image_byte(image, offset + 2)? as u32,
                alpha,
            ))
        }
        ImageFormat::A8 => {
            let offset = y.checked_mul(image.stride)?.checked_add(x)?;
            let alpha = ((image_byte(image, offset)? as u32 * opacity as u32) / 255).min(255);
            Some((255, 255, 255, alpha))
        }
    }
}

fn image_byte(image: ImageView, offset: usize) -> Option<u8> {
    if image.data.is_null() || offset >= image.len {
        None
    } else {
        Some(unsafe { *image.data.add(offset) })
    }
}
