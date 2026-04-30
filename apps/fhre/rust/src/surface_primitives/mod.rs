use crate::{
    raster::{abs_i32, clamp_i32, max3_i32, min3_i32},
    ArcStyle, BlendMode, BorderAlign, BorderSides, BorderStyle, Color, FillStyle, GradientStyle,
    LayerSpec, LineCap, LineStyle, MaskSpec, Point, Rect, ShadowStyle, Surface, TriangleStyle,
};

impl Surface {
    pub fn draw_line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = abs_i32(x1 - x0);
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -abs_i32(y1 - y0);
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.put_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = err.saturating_mul(2);
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn draw_wide_line(&mut self, from: Point, to: Point, width: u16, color: Color) {
        let width = width.max(1) as i32;
        if width == 1 {
            self.draw_line(from.x, from.y, to.x, to.y, color);
            return;
        }

        let radius = width / 2;
        for offset in -radius..=radius {
            self.draw_line(from.x + offset, from.y, to.x + offset, to.y, color);
            self.draw_line(from.x, from.y + offset, to.x, to.y + offset, color);
        }
    }

    pub fn draw_styled_line(&mut self, from: Point, to: Point, style: LineStyle) {
        if style.color.a == 0 || style.width == 0 {
            return;
        }

        let original_from = from;
        let original_to = to;
        let from = line_cap_point(original_from, original_to, style.width, style.cap_start);
        let to = line_cap_point(original_to, original_from, style.width, style.cap_end);

        if style.blend == BlendMode::Normal && style.dash_width == 0 && style.dash_gap == 0 {
            self.draw_wide_line(from, to, style.width, style.color);
        } else {
            self.draw_dashed_line(from, to, style);
        }

        let radius = style.width.max(1) as i32 / 2;
        if radius > 0 && (style.round_start || style.cap_start == LineCap::Round) {
            self.fill_circle(from.x, from.y, radius, style.color);
        }
        if radius > 0 && (style.round_end || style.cap_end == LineCap::Round) {
            self.fill_circle(to.x, to.y, radius, style.color);
        }
    }

    fn draw_dashed_line(&mut self, from: Point, to: Point, style: LineStyle) {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let steps = abs_i32(dx).max(abs_i32(dy)).max(1);
        let dash = style.dash_width.max(1) as i32;
        let gap = style.dash_gap as i32;
        let period = (dash + gap).max(1);
        let radius = style.width.max(1) as i32 / 2;
        let mut i = 0;
        while i <= steps {
            if gap == 0 || i % period < dash {
                let x = from.x + dx.saturating_mul(i) / steps;
                let y = from.y + dy.saturating_mul(i) / steps;
                if radius <= 1 {
                    self.put_pixel_blend(x, y, style.color, style.blend);
                } else {
                    self.fill_circle(x, y, radius, style.color);
                }
            }
            i += 1;
        }
    }

    pub fn fill_style(&mut self, rect: Rect, style: FillStyle) {
        if rect.is_empty() || style.color.a == 0 {
            return;
        }

        if style.gradient == GradientStyle::None && style.blend == BlendMode::Normal {
            if style.radius == 0 {
                self.fill_rect(rect, style.color);
            } else {
                self.fill_round_rect(rect, style.radius, style.color);
            }
            return;
        }

        let clipped = match self.clip {
            Some(clip) => rect.clipped_to(clip),
            None => rect,
        };
        let x0 = clamp_i32(clipped.x, 0, self.width as i32);
        let y0 = clamp_i32(clipped.y, 0, self.height as i32);
        let x1 = clamp_i32(clipped.right(), 0, self.width as i32);
        let y1 = clamp_i32(clipped.bottom(), 0, self.height as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let radius = style.radius as i32;
        let r2 = radius.saturating_mul(radius);
        let left_center = rect.x + radius;
        let right_center = rect.right() - radius - 1;
        let top_center = rect.y + radius;
        let bottom_center = rect.bottom() - radius - 1;
        let mut y = y0;
        while y < y1 {
            let mut x = x0;
            while x < x1 {
                if radius > 0 {
                    let in_left = x < left_center;
                    let in_right = x > right_center;
                    let in_top = y < top_center;
                    let in_bottom = y > bottom_center;
                    if (in_left || in_right) && (in_top || in_bottom) {
                        let dx = if in_left {
                            left_center - x
                        } else {
                            x - right_center
                        };
                        let dy = if in_top {
                            top_center - y
                        } else {
                            y - bottom_center
                        };
                        if dx.saturating_mul(dx) + dy.saturating_mul(dy) > r2 {
                            x += 1;
                            continue;
                        }
                    }
                }
                self.put_pixel_blend(
                    x,
                    y,
                    gradient_color(style.gradient, style.color, rect, Point::new(x, y)),
                    style.blend,
                );
                x += 1;
            }
            y += 1;
        }
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        if radius <= 0 {
            return;
        }

        let r2 = radius.saturating_mul(radius);
        for y in -radius..=radius {
            let yy = y.saturating_mul(y);
            let mut x = radius;
            while x > 0 && x.saturating_mul(x).saturating_add(yy) > r2 {
                x -= 1;
            }
            if x >= 0 {
                self.fill_rect(
                    Rect::new(cx - x, cy + y, (x.saturating_mul(2) + 1) as u16, 1),
                    color,
                );
            }
        }
    }

    pub fn fill_round_rect(&mut self, rect: Rect, radius: u16, color: Color) {
        if rect.w == 0 || rect.h == 0 {
            return;
        }

        let radius = (radius as i32)
            .min(rect.w as i32 / 2)
            .min(rect.h as i32 / 2);
        if radius <= 0 {
            self.fill_rect(rect, color);
            return;
        }

        let inner_w = rect.w.saturating_sub((radius as u16).saturating_mul(2));
        let inner_h = rect.h.saturating_sub((radius as u16).saturating_mul(2));
        self.fill_rect(Rect::new(rect.x + radius, rect.y, inner_w, rect.h), color);
        self.fill_rect(
            Rect::new(rect.x, rect.y + radius, radius as u16, inner_h),
            color,
        );
        self.fill_rect(
            Rect::new(
                rect.right() - radius,
                rect.y + radius,
                radius as u16,
                inner_h,
            ),
            color,
        );

        let x0 = clamp_i32(rect.x, 0, self.width as i32);
        let y0 = clamp_i32(rect.y, 0, self.height as i32);
        let x1 = clamp_i32(rect.right(), 0, self.width as i32);
        let y1 = clamp_i32(rect.bottom(), 0, self.height as i32);
        let left_center = rect.x + radius;
        let right_center = rect.right() - radius - 1;
        let top_center = rect.y + radius;
        let bottom_center = rect.bottom() - radius - 1;
        let r2 = radius.saturating_mul(radius);

        for y in y0..y1 {
            for x in x0..x1 {
                let in_left = x < left_center;
                let in_right = x > right_center;
                let in_top = y < top_center;
                let in_bottom = y > bottom_center;
                if !(in_left || in_right) || !(in_top || in_bottom) {
                    continue;
                }
                let dx = if in_left {
                    left_center - x
                } else {
                    x - right_center
                };
                let dy = if in_top {
                    top_center - y
                } else {
                    y - bottom_center
                };

                if dx.saturating_mul(dx) + dy.saturating_mul(dy) <= r2 {
                    self.put_pixel(x, y, color);
                }
            }
        }
    }

    pub fn draw_border(&mut self, rect: Rect, style: BorderStyle) {
        if rect.is_empty()
            || style.width == 0
            || style.color.a == 0
            || style.sides == BorderSides::NONE
        {
            return;
        }

        let (outset, inset) = border_outset_inset(style.align, style.width);

        if style.sides == BorderSides::FULL {
            let mut i = 0u16;
            while i < style.width {
                let edge = i as i32 - outset;
                let outline = Rect::from_edges(
                    rect.x + edge,
                    rect.y + edge,
                    rect.right() - edge,
                    rect.bottom() - edge,
                );
                if outline.is_empty() {
                    break;
                }
                self.draw_round_rect_outline(outline, style.radius.saturating_sub(i), style.color);
                i += 1;
            }
            return;
        }

        let outer = Rect::from_edges(
            rect.x - outset,
            rect.y - outset,
            rect.right() + outset,
            rect.bottom() + outset,
        );
        if style.sides.contains(BorderSides::TOP) {
            self.fill_rect(
                Rect::from_edges(outer.x, rect.y - outset, outer.right(), rect.y + inset),
                style.color,
            );
        }
        if style.sides.contains(BorderSides::BOTTOM) {
            self.fill_rect(
                Rect::from_edges(
                    outer.x,
                    rect.bottom() - inset,
                    outer.right(),
                    rect.bottom() + outset,
                ),
                style.color,
            );
        }
        if style.sides.contains(BorderSides::LEFT) {
            self.fill_rect(
                Rect::from_edges(rect.x - outset, outer.y, rect.x + inset, outer.bottom()),
                style.color,
            );
        }
        if style.sides.contains(BorderSides::RIGHT) {
            self.fill_rect(
                Rect::from_edges(
                    rect.right() - inset,
                    outer.y,
                    rect.right() + outset,
                    outer.bottom(),
                ),
                style.color,
            );
        }
    }

    fn draw_round_rect_outline(&mut self, rect: Rect, radius: u16, color: Color) {
        if rect.is_empty() {
            return;
        }
        if radius == 0 {
            self.draw_line(rect.x, rect.y, rect.right() - 1, rect.y, color);
            self.draw_line(
                rect.x,
                rect.bottom() - 1,
                rect.right() - 1,
                rect.bottom() - 1,
                color,
            );
            self.draw_line(rect.x, rect.y, rect.x, rect.bottom() - 1, color);
            self.draw_line(
                rect.right() - 1,
                rect.y,
                rect.right() - 1,
                rect.bottom() - 1,
                color,
            );
            return;
        }

        let r = (radius as i32)
            .min(rect.w as i32 / 2)
            .min(rect.h as i32 / 2);
        self.draw_line(rect.x + r, rect.y, rect.right() - r - 1, rect.y, color);
        self.draw_line(
            rect.x + r,
            rect.bottom() - 1,
            rect.right() - r - 1,
            rect.bottom() - 1,
            color,
        );
        self.draw_line(rect.x, rect.y + r, rect.x, rect.bottom() - r - 1, color);
        self.draw_line(
            rect.right() - 1,
            rect.y + r,
            rect.right() - 1,
            rect.bottom() - r - 1,
            color,
        );

        let cx0 = rect.x + r;
        let cx1 = rect.right() - r - 1;
        let cy0 = rect.y + r;
        let cy1 = rect.bottom() - r - 1;
        let mut dx = -r;
        while dx <= r {
            let yy = r.saturating_mul(r).saturating_sub(dx.saturating_mul(dx));
            let dy = int_sqrt(yy);
            self.put_pixel(cx0 + dx, cy0 - dy, color);
            self.put_pixel(cx0 + dx, cy1 + dy, color);
            self.put_pixel(cx1 + dx, cy0 - dy, color);
            self.put_pixel(cx1 + dx, cy1 + dy, color);
            dx += 1;
        }
    }

    pub fn draw_shadow(&mut self, rect: Rect, style: ShadowStyle) {
        if rect.is_empty() || style.width == 0 || style.color.a == 0 {
            return;
        }

        if self.draw_blurred_shadow(rect, style) {
            return;
        }

        let spread = style.spread as i32;
        let base = Rect::from_edges(
            rect.x + style.offset.x - spread,
            rect.y + style.offset.y - spread,
            rect.right() + style.offset.x + spread,
            rect.bottom() + style.offset.y + spread,
        );
        if base.is_empty() {
            return;
        }

        let mut i = style.width;
        while i > 0 {
            let k = i as i32;
            let alpha = ((style.color.a as u16 * i as u16) / style.width.max(1) as u16 / 2) as u8;
            let color = Color::rgba(style.color.r, style.color.g, style.color.b, alpha);
            let shadow =
                Rect::from_edges(base.x - k, base.y - k, base.right() + k, base.bottom() + k);
            self.fill_round_rect(shadow, style.radius.saturating_add(i), color);
            i -= 1;
        }
    }

    pub fn draw_arc(&mut self, center: Point, radius: u16, style: ArcStyle) {
        if radius == 0 || style.width == 0 || style.color.a == 0 {
            return;
        }

        let start = normalize_angle(style.start_angle as i32);
        let mut end = normalize_angle(style.end_angle as i32);
        if end <= start {
            end += 360;
        }
        let mut angle = start;
        let mut prev = arc_point(center, radius as i32, angle);
        while angle <= end {
            let next_angle = (angle + 3).min(end);
            let next = arc_point(center, radius as i32, next_angle);
            self.draw_wide_line(prev, next, style.width, style.color);
            prev = next;
            angle = next_angle + 3;
        }
        if style.rounded {
            let cap = style.width as i32 / 2;
            if cap > 0 {
                let a = arc_point(center, radius as i32, start);
                let b = arc_point(center, radius as i32, end);
                self.fill_circle(a.x, a.y, cap, style.color);
                self.fill_circle(b.x, b.y, cap, style.color);
            }
        }
    }

    pub fn draw_mask_rect(&mut self, rect: Rect, spec: MaskSpec) {
        let area = spec.area.clipped_to(rect);
        if area.is_empty() {
            return;
        }
        let color = if spec.inverted {
            Color::rgba(255, 255, 255, 72)
        } else {
            Color::rgba(0, 0, 0, 110)
        };
        self.draw_border(
            area,
            BorderStyle {
                color,
                width: 2,
                radius: spec.radius,
                sides: BorderSides::FULL,
                align: BorderAlign::Inside,
            },
        );
        if !spec.inverted {
            self.fill_round_rect(area, spec.radius, color);
        }
    }

    pub fn draw_blur_fallback(&mut self, rect: Rect, radius: u16, opacity: u8) {
        if rect.is_empty() || opacity == 0 {
            return;
        }
        let alpha = ((opacity as u16 * radius.max(1).min(32) as u16) / 64).min(160) as u8;
        self.fill_rect(rect, Color::rgba(255, 255, 255, alpha));
        self.draw_border(
            rect,
            BorderStyle {
                color: Color::rgba(255, 255, 255, alpha.saturating_add(24)),
                width: 1,
                radius: 0,
                sides: BorderSides::FULL,
                align: BorderAlign::Inside,
            },
        );
    }

    pub fn draw_layer_fallback(&mut self, rect: Rect, spec: LayerSpec) {
        if rect.is_empty() || spec.opacity == 0 {
            return;
        }
        if let Some(color) = spec.recolor {
            let alpha = ((color.a as u16 * spec.opacity as u16) / 255) as u8;
            self.fill_rect(rect, Color::rgba(color.r, color.g, color.b, alpha));
        }
        if spec.blur_radius != 0 {
            self.draw_blur_fallback(rect, spec.blur_radius, spec.opacity);
        }
        if let Some(mask) = spec.mask {
            self.draw_mask_rect(rect, mask);
        }
    }

    pub fn fill_triangle(&mut self, p0: Point, p1: Point, p2: Point, color: Color) {
        let min_x = clamp_i32(min3_i32(p0.x, p1.x, p2.x), 0, self.width as i32);
        let max_x = clamp_i32(max3_i32(p0.x, p1.x, p2.x), 0, self.width as i32 - 1);
        let min_y = clamp_i32(min3_i32(p0.y, p1.y, p2.y), 0, self.height as i32);
        let max_y = clamp_i32(max3_i32(p0.y, p1.y, p2.y), 0, self.height as i32 - 1);

        if min_x > max_x || min_y > max_y {
            return;
        }

        let mut y = min_y;
        while y <= max_y {
            let mut xs = [0i32; 3];
            let mut count = 0usize;
            add_edge_intersection(p0, p1, y, &mut xs, &mut count);
            add_edge_intersection(p1, p2, y, &mut xs, &mut count);
            add_edge_intersection(p2, p0, y, &mut xs, &mut count);
            if count >= 2 {
                sort_intersections(&mut xs, count);
                let x0 = clamp_i32(xs[0], min_x, max_x);
                let x1 = clamp_i32(xs[count - 1], min_x, max_x);
                if x0 <= x1 {
                    self.fill_rect(Rect::new(x0, y, (x1 - x0 + 1) as u16, 1), color);
                }
            }
            y += 1;
        }
    }

    pub fn fill_gradient_triangle(
        &mut self,
        p0: Point,
        p1: Point,
        p2: Point,
        style: TriangleStyle,
    ) {
        let min_x = clamp_i32(min3_i32(p0.x, p1.x, p2.x), 0, self.width as i32);
        let max_x = clamp_i32(max3_i32(p0.x, p1.x, p2.x), 0, self.width as i32 - 1);
        let min_y = clamp_i32(min3_i32(p0.y, p1.y, p2.y), 0, self.height as i32);
        let max_y = clamp_i32(max3_i32(p0.y, p1.y, p2.y), 0, self.height as i32 - 1);
        if min_x > max_x || min_y > max_y {
            return;
        }

        let area = edge_function(p0, p1, p2);
        if area == 0 {
            return;
        }
        let positive = area > 0;
        let denom = if area < 0 { -area } else { area }.max(1);

        let mut y = min_y;
        while y <= max_y {
            let mut x = min_x;
            while x <= max_x {
                let p = Point::new(x, y);
                let mut w0 = edge_function(p1, p2, p);
                let mut w1 = edge_function(p2, p0, p);
                let mut w2 = edge_function(p0, p1, p);
                let inside = if positive {
                    w0 >= 0 && w1 >= 0 && w2 >= 0
                } else {
                    w0 <= 0 && w1 <= 0 && w2 <= 0
                };
                if inside {
                    if w0 < 0 {
                        w0 = -w0;
                    }
                    if w1 < 0 {
                        w1 = -w1;
                    }
                    if w2 < 0 {
                        w2 = -w2;
                    }
                    let color = bary_color(style.colors, w0, w1, w2, denom);
                    self.put_pixel_blend(x, y, color, style.blend);
                }
                x += 1;
            }
            y += 1;
        }
    }
}

fn border_outset_inset(align: BorderAlign, width: u16) -> (i32, i32) {
    let width = width as i32;
    match align {
        BorderAlign::Inside => (0, width),
        BorderAlign::Center => (width / 2, width - width / 2),
        BorderAlign::Outside => (width, 0),
    }
}

fn line_cap_point(point: Point, other: Point, width: u16, cap: LineCap) -> Point {
    if cap != LineCap::Square || width <= 1 {
        return point;
    }
    let dx = point.x - other.x;
    let dy = point.y - other.y;
    let steps = abs_i32(dx).max(abs_i32(dy)).max(1);
    let extend = width.max(1) as i32 / 2;
    Point::new(
        point.x + dx.saturating_mul(extend) / steps,
        point.y + dy.saturating_mul(extend) / steps,
    )
}

fn edge_function(a: Point, b: Point, c: Point) -> i32 {
    (c.x - a.x)
        .saturating_mul(b.y - a.y)
        .saturating_sub((c.y - a.y).saturating_mul(b.x - a.x))
}

fn bary_color(colors: [Color; 3], w0: i32, w1: i32, w2: i32, denom: i32) -> Color {
    let d = denom.max(1) as i64;
    let w0 = w0 as i64;
    let w1 = w1 as i64;
    let w2 = w2 as i64;
    Color::rgba(
        (((colors[0].r as i64 * w0) + (colors[1].r as i64 * w1) + (colors[2].r as i64 * w2)) / d)
            .clamp(0, 255) as u8,
        (((colors[0].g as i64 * w0) + (colors[1].g as i64 * w1) + (colors[2].g as i64 * w2)) / d)
            .clamp(0, 255) as u8,
        (((colors[0].b as i64 * w0) + (colors[1].b as i64 * w1) + (colors[2].b as i64 * w2)) / d)
            .clamp(0, 255) as u8,
        (((colors[0].a as i64 * w0) + (colors[1].a as i64 * w1) + (colors[2].a as i64 * w2)) / d)
            .clamp(0, 255) as u8,
    )
}

fn gradient_color(style: GradientStyle, fallback: Color, rect: Rect, point: Point) -> Color {
    match style {
        GradientStyle::None => fallback,
        GradientStyle::Vertical { start, end } => {
            let h = rect.h.max(1) as i32;
            let t = ((point.y.saturating_sub(rect.y).clamp(0, h) * 255) / h) as u8;
            start.mix(end, t)
        }
        GradientStyle::Horizontal { start, end } => {
            let w = rect.w.max(1) as i32;
            let t = ((point.x.saturating_sub(rect.x).clamp(0, w) * 255) / w) as u8;
            start.mix(end, t)
        }
        GradientStyle::Linear {
            start,
            end,
            start_color,
            end_color,
        } => {
            let vx = end.x - start.x;
            let vy = end.y - start.y;
            let len2 = vx
                .saturating_mul(vx)
                .saturating_add(vy.saturating_mul(vy))
                .max(1);
            let px = point.x - start.x;
            let py = point.y - start.y;
            let dot = px
                .saturating_mul(vx)
                .saturating_add(py.saturating_mul(vy))
                .clamp(0, len2);
            start_color.mix(end_color, ((dot * 255) / len2) as u8)
        }
        GradientStyle::Radial {
            center,
            radius,
            inner,
            outer,
        } => {
            let dx = point.x - center.x;
            let dy = point.y - center.y;
            let dist = int_sqrt(dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)));
            let radius = radius.max(1) as i32;
            inner.mix(outer, ((dist.clamp(0, radius) * 255) / radius) as u8)
        }
        GradientStyle::Conical { center, start, end } => {
            let dx = point.x - center.x;
            let dy = point.y - center.y;
            let t = if dx == 0 && dy == 0 {
                0
            } else {
                let angle = pseudo_angle_0_255(dx, dy);
                angle as u8
            };
            start.mix(end, t)
        }
    }
}

fn int_sqrt(value: i32) -> i32 {
    if value <= 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x
}

fn normalize_angle(mut angle: i32) -> i32 {
    while angle < 0 {
        angle += 360;
    }
    while angle >= 360 {
        angle -= 360;
    }
    angle
}

fn arc_point(center: Point, radius: i32, angle: i32) -> Point {
    let sin = sin_deg(angle);
    let cos = sin_deg(angle + 90);
    Point::new(
        center.x + radius.saturating_mul(cos) / 1024,
        center.y + radius.saturating_mul(sin) / 1024,
    )
}

fn sin_deg(angle: i32) -> i32 {
    let mut a = normalize_angle(angle);
    let sign = if a >= 180 {
        a -= 180;
        -1
    } else {
        1
    };
    if a > 90 {
        a = 180 - a;
    }
    let x = a;
    let numerator = 4 * x * (180 - x) * 1024;
    let denominator = 40500 - x * (180 - x);
    if denominator == 0 {
        0
    } else {
        sign * numerator / denominator
    }
}

fn pseudo_angle_0_255(dx: i32, dy: i32) -> i32 {
    let ax = abs_i32(dx);
    let ay = abs_i32(dy);
    let base = if ax + ay == 0 { 0 } else { ay * 64 / (ax + ay) };
    if dx >= 0 && dy >= 0 {
        base
    } else if dx < 0 && dy >= 0 {
        128 - base
    } else if dx < 0 && dy < 0 {
        128 + base
    } else {
        256 - base
    }
}

fn add_edge_intersection(a: Point, b: Point, y: i32, xs: &mut [i32; 3], count: &mut usize) {
    if *count >= xs.len() || a.y == b.y {
        return;
    }

    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);
    if y < min_y || y > max_y {
        return;
    }

    let dy = b.y - a.y;
    let dx = b.x - a.x;
    xs[*count] = a.x + ((y - a.y) as i64 * dx as i64 / dy as i64) as i32;
    *count += 1;
}

fn sort_intersections(values: &mut [i32; 3], count: usize) {
    let mut i = 1usize;
    while i < count {
        let value = values[i];
        let mut j = i;
        while j > 0 && values[j - 1] > value {
            values[j] = values[j - 1];
            j -= 1;
        }
        values[j] = value;
        i += 1;
    }
}
