use alloc::vec::Vec;
use crate::{
    parse_svg_path, Color, MaskRasterOptions, Point, Rect, Surface, SvgDocument, SvgFillRule,
    SvgId, SvgPaint, SvgFilterEffect, SvgPath, SvgPathCommand, SvgStrokeCap, SvgStrokeJoin,
    VectorMaskScratch,
};

struct SvgA8Mask {
    rect: Rect,
    width: u16,
    data: Vec<u8>,
}

impl SvgA8Mask {
    fn alpha_at(&self, x: i32, y: i32) -> u8 {
        if !self.rect.contains_point(Point::new(x, y)) {
            return 0;
        }
        let lx = (x - self.rect.x) as usize;
        let ly = (y - self.rect.y) as usize;
        let index = ly.saturating_mul(self.width as usize).saturating_add(lx);
        self.data.get(index).copied().unwrap_or(0)
    }
}

impl Surface {
    pub fn draw_svg_icon(&mut self, rect: Rect, icon: SvgId, color: Color) {
        if rect.is_empty() || color.a == 0 {
            return;
        }

        if let Some(data) = builtin_svg_icon_path(icon) {
            let path = parse_svg_path::<64>(Rect::new(0, 0, 16, 16), data, 1);
            let stroke = (rect.w.min(rect.h) / 9).max(1);
            self.draw_svg_path(rect, &path, color, stroke);
            return;
        }

        self.draw_svg_icon_placeholder(rect, icon, color);
    }

    pub fn draw_svg_path<const N: usize>(
        &mut self,
        rect: Rect,
        path: &SvgPath<N>,
        color: Color,
        stroke: u16,
    ) {
        if rect.is_empty() || path.len == 0 || path.view_box.is_empty() {
            return;
        }

        let mut current = Point::new(0, 0);
        let mut sub_start = Point::new(0, 0);
        let mut have_current = false;
        let mut index = 0;
        while index < path.len {
            match path.commands[index] {
                SvgPathCommand::Empty => {}
                SvgPathCommand::MoveTo(point) => {
                    current = map_svg_point(rect, path.view_box, point);
                    sub_start = current;
                    have_current = true;
                }
                SvgPathCommand::LineTo(point) => {
                    let next = map_svg_point(rect, path.view_box, point);
                    if have_current {
                        self.draw_wide_line(current, next, stroke, color);
                    }
                    current = next;
                    have_current = true;
                }
                SvgPathCommand::Close => {
                    if have_current {
                        self.draw_wide_line(current, sub_start, stroke, color);
                        current = sub_start;
                    }
                }
            }
            index += 1;
        }
    }

    pub fn draw_svg_document<const PATHS: usize, const CMDS: usize>(
        &mut self,
        rect: Rect,
        document: &SvgDocument<PATHS, CMDS>,
        color: Color,
    ) {
        if rect.is_empty() || document.len == 0 || color.a == 0 {
            return;
        }
        let mut i = 0usize;
        while i < document.len {
            let node = document.paths[i];
            let saved_clip = self.clip;
            if let Some(clip) = node.clip {
                let mapped = map_svg_rect(rect, document.view_box, clip);
                self.clip = Some(match saved_clip {
                    Some(existing) => mapped.clipped_to(existing),
                    None => mapped,
                });
            }
            match node.filter {
                SvgFilterEffect::DropShadow { dx, dy, radius, color: shadow } => {
                    let shadow_rect = Rect::new(
                        rect.x + dx as i32,
                        rect.y + dy as i32,
                        rect.w,
                        rect.h,
                    );
                    let shadow_color = Color::rgba(shadow.r, shadow.g, shadow.b, shadow.a.saturating_sub((radius.min(24) * 3) as u8));
                    self.fill_svg_path_rule(shadow_rect, &node.path, shadow_color, node.fill_rule);
                }
                SvgFilterEffect::Blur(radius) => {
                    if radius > 0 {
                        let blur_color = Color::rgba(color.r, color.g, color.b, node.opacity / 3);
                        self.fill_svg_path_rule(expand_rect(rect, radius as i32), &node.path, blur_color, node.fill_rule);
                    }
                }
                SvgFilterEffect::None => {}
            }
            let clip_path = if node.has_clip_path { Some(&node.clip_path) } else { None };
            let mask_path = if node.has_mask_path { Some(&node.mask_path) } else { None };
            let mask_scratch = self.build_svg_a8_mask(rect, clip_path, mask_path);
            if let Some(fill) = resolve_svg_fill(node.fill, color, node.opacity) {
                self.fill_svg_path_paint_masked(rect, &node.path, fill, node.fill_rule, clip_path, mask_path, mask_scratch.as_ref());
            }
            if let Some(stroke) = resolve_svg_paint(node.stroke, color, node.opacity) {
                self.draw_svg_path_styled_masked(
                    rect,
                    &node.path,
                    stroke,
                    node.stroke_width.max(1),
                    node.stroke_cap,
                    node.stroke_join,
                    clip_path,
                    mask_path,
                    mask_scratch.as_ref(),
                );
            }
            self.clip = saved_clip;
            i += 1;
        }
    }

    pub fn draw_svg_path_styled<const N: usize>(
        &mut self,
        rect: Rect,
        path: &SvgPath<N>,
        color: Color,
        stroke: u16,
        cap: SvgStrokeCap,
        join: SvgStrokeJoin,
    ) {
        self.draw_svg_path_styled_masked(rect, path, color, stroke, cap, join, None, None, None);
    }

    fn draw_svg_path_styled_masked<const N: usize>(
        &mut self,
        rect: Rect,
        path: &SvgPath<N>,
        color: Color,
        stroke: u16,
        cap: SvgStrokeCap,
        join: SvgStrokeJoin,
        clip_path: Option<&SvgPath<N>>,
        mask_path: Option<&SvgPath<N>>,
        a8_mask: Option<&SvgA8Mask>,
    ) {
        if rect.is_empty() || path.len == 0 || path.view_box.is_empty() {
            return;
        }

        let mut current = Point::new(0, 0);
        let mut sub_start = Point::new(0, 0);
        let mut have_current = false;
        let radius = (stroke as i32 / 2).max(1);
        let mut index = 0;
        while index < path.len {
            match path.commands[index] {
                SvgPathCommand::Empty => {}
                SvgPathCommand::MoveTo(point) => {
                    current = map_svg_point(rect, path.view_box, point);
                    sub_start = current;
                    have_current = true;
                    if cap == SvgStrokeCap::Round {
                        self.fill_masked_svg_circle(rect, current.x, current.y, radius, color, clip_path, mask_path, a8_mask);
                    }
                }
                SvgPathCommand::LineTo(point) => {
                    let next = map_svg_point(rect, path.view_box, point);
                    if have_current {
                        self.draw_masked_svg_line(rect, current, next, stroke, color, clip_path, mask_path, a8_mask);
                        match join {
                            SvgStrokeJoin::Round => {
                                self.fill_masked_svg_circle(rect, current.x, current.y, radius, color, clip_path, mask_path, a8_mask);
                            }
                            SvgStrokeJoin::Bevel | SvgStrokeJoin::Miter => {}
                        }
                        if cap == SvgStrokeCap::Round {
                            self.fill_masked_svg_circle(rect, next.x, next.y, radius, color, clip_path, mask_path, a8_mask);
                        } else if cap == SvgStrokeCap::Square {
                            self.fill_masked_svg_rect(
                                rect,
                                Rect::new(next.x - radius, next.y - radius, stroke, stroke),
                                color,
                                clip_path,
                                mask_path,
                                a8_mask,
                            );
                        }
                    }
                    current = next;
                    have_current = true;
                }
                SvgPathCommand::Close => {
                    if have_current {
                        self.draw_masked_svg_line(rect, current, sub_start, stroke, color, clip_path, mask_path, a8_mask);
                        if join == SvgStrokeJoin::Round {
                            self.fill_masked_svg_circle(rect, sub_start.x, sub_start.y, radius, color, clip_path, mask_path, a8_mask);
                        }
                        current = sub_start;
                    }
                }
            }
            index += 1;
        }
    }

    pub fn draw_svg_document_id(&mut self, rect: Rect, document: SvgId, color: Color) {
        if rect.is_empty() || color.a == 0 {
            return;
        }

        if let Some(resolver) = self.svg_resolver {
            if let Some(view) = resolver(document) {
                if let Some(svg) = view.get() {
                    self.draw_svg_document(rect, svg, color);
                    return;
                }
            }
        }

        self.draw_svg_icon_placeholder(rect, document, color);
    }

    pub fn fill_svg_path<const N: usize>(&mut self, rect: Rect, path: &SvgPath<N>, color: Color) {
        self.fill_svg_path_rule(rect, path, color, SvgFillRule::EvenOdd);
    }

    pub fn fill_svg_path_paint<const N: usize>(
        &mut self,
        rect: Rect,
        path: &SvgPath<N>,
        paint: SvgPaint,
        rule: SvgFillRule,
    ) {
        self.fill_svg_path_paint_masked(rect, path, paint, rule, None, None, None);
    }

    fn fill_svg_path_paint_masked<const N: usize>(
        &mut self,
        rect: Rect,
        path: &SvgPath<N>,
        paint: SvgPaint,
        rule: SvgFillRule,
        clip_path: Option<&SvgPath<N>>,
        mask_path: Option<&SvgPath<N>>,
        a8_mask: Option<&SvgA8Mask>,
    ) {
        match paint {
            SvgPaint::Color(color) => self.fill_svg_path_rule_masked(rect, path, color, rule, clip_path, mask_path, a8_mask),
            SvgPaint::CurrentColor => self.fill_svg_path_rule_masked(rect, path, Color::rgba(255, 255, 255, 255), rule, clip_path, mask_path, a8_mask),
            SvgPaint::LinearGradient(start, end) => self.fill_svg_path_gradient_masked(rect, path, start, end, false, rule, clip_path, mask_path, a8_mask),
            SvgPaint::RadialGradient(start, end) => self.fill_svg_path_gradient_masked(rect, path, start, end, true, rule, clip_path, mask_path, a8_mask),
            SvgPaint::None => {}
        }
    }

    pub fn fill_svg_path_rule<const N: usize>(
        &mut self,
        rect: Rect,
        path: &SvgPath<N>,
        color: Color,
        rule: SvgFillRule,
    ) {
        self.fill_svg_path_rule_masked(rect, path, color, rule, None, None, None);
    }

    fn fill_svg_path_rule_masked<const N: usize>(
        &mut self,
        rect: Rect,
        path: &SvgPath<N>,
        color: Color,
        rule: SvgFillRule,
        clip_path: Option<&SvgPath<N>>,
        mask_path: Option<&SvgPath<N>>,
        a8_mask: Option<&SvgA8Mask>,
    ) {
        if rect.is_empty() || path.len == 0 || path.view_box.is_empty() || color.a == 0 {
            return;
        }

        let clipped = match self.clip {
            Some(clip) => rect.clipped_to(clip),
            None => rect,
        };
        if clipped.is_empty() {
            return;
        }

        let mut y = clipped.y;
        while y < clipped.bottom() {
            let mut xs = [0i32; N];
            let mut winds = [0i8; N];
            let mut count = 0usize;
            collect_svg_scanline_intersections(rect, path, y, &mut xs, &mut winds, &mut count);
            sort_svg_intersections(&mut xs, &mut winds, count);

            match rule {
                SvgFillRule::EvenOdd => {
                    let mut i = 0usize;
                    while i + 1 < count {
                        let x0 = xs[i].max(clipped.x);
                        let x1 = xs[i + 1].min(clipped.right());
                        if x0 < x1 {
                            self.fill_masked_svg_span(rect, y, x0, x1, color, clip_path, mask_path, a8_mask);
                        }
                        i += 2;
                    }
                }
                SvgFillRule::NonZero => {
                    let mut winding = 0i32;
                    let mut start_x = 0i32;
                    let mut i = 0usize;
                    while i < count {
                        if winding == 0 {
                            start_x = xs[i];
                        }
                        winding += winds[i] as i32;
                        if winding == 0 {
                            let x0 = start_x.max(clipped.x);
                            let x1 = xs[i].min(clipped.right());
                            if x0 < x1 {
                                self.fill_masked_svg_span(rect, y, x0, x1, color, clip_path, mask_path, a8_mask);
                            }
                        }
                        i += 1;
                    }
                }
            }
            y += 1;
        }
    }

    fn fill_svg_path_gradient_masked<const N: usize>(
        &mut self,
        rect: Rect,
        path: &SvgPath<N>,
        start: Color,
        end: Color,
        radial: bool,
        rule: SvgFillRule,
        clip_path: Option<&SvgPath<N>>,
        mask_path: Option<&SvgPath<N>>,
        a8_mask: Option<&SvgA8Mask>,
    ) {
        if rect.is_empty() || path.len == 0 || path.view_box.is_empty() {
            return;
        }
        let clipped = match self.clip {
            Some(clip) => rect.clipped_to(clip),
            None => rect,
        };
        if clipped.is_empty() {
            return;
        }
        let cy = rect.y + rect.h as i32 / 2;
        let max_dist = (rect.w.max(rect.h) as i32 / 2).max(1);
        let mut y = clipped.y;
        while y < clipped.bottom() {
            let t = if radial {
                ((y - cy).abs().min(max_dist) * 255 / max_dist) as u8
            } else {
                ((y - rect.y).max(0).min(rect.h as i32) * 255 / rect.h.max(1) as i32) as u8
            };
            let color = lerp_color(start, end, t);
            let mut xs = [0i32; N];
            let mut winds = [0i8; N];
            let mut count = 0usize;
            collect_svg_scanline_intersections(rect, path, y, &mut xs, &mut winds, &mut count);
            sort_svg_intersections(&mut xs, &mut winds, count);
            match rule {
                SvgFillRule::EvenOdd => {
                    let mut i = 0usize;
                    while i + 1 < count {
                        let x0 = xs[i].max(clipped.x);
                        let x1 = xs[i + 1].min(clipped.right());
                        if x0 < x1 {
                            self.fill_masked_svg_span(rect, y, x0, x1, color, clip_path, mask_path, a8_mask);
                        }
                        i += 2;
                    }
                }
                SvgFillRule::NonZero => {
                    let mut winding = 0i32;
                    let mut start_x = 0i32;
                    let mut i = 0usize;
                    while i < count {
                        if winding == 0 {
                            start_x = xs[i];
                        }
                        winding += winds[i] as i32;
                        if winding == 0 {
                            let x0 = start_x.max(clipped.x);
                            let x1 = xs[i].min(clipped.right());
                            if x0 < x1 {
                                self.fill_masked_svg_span(rect, y, x0, x1, color, clip_path, mask_path, a8_mask);
                            }
                        }
                        i += 1;
                    }
                }
            }
            y += 1;
        }
    }

    fn fill_masked_svg_span<const N: usize>(
        &mut self,
        rect: Rect,
        y: i32,
        x0: i32,
        x1: i32,
        color: Color,
        clip_path: Option<&SvgPath<N>>,
        mask_path: Option<&SvgPath<N>>,
        a8_mask: Option<&SvgA8Mask>,
    ) {
        if clip_path.is_none() && mask_path.is_none() && a8_mask.is_none() {
            self.fill_rect(Rect::from_edges(x0, y, x1, y + 1), color);
            return;
        }
        if let Some(mask) = a8_mask {
            let mut run_start: Option<i32> = None;
            let mut x = x0;
            while x < x1 {
                if mask.alpha_at(x, y) != 0 {
                    if run_start.is_none() {
                        run_start = Some(x);
                    }
                } else if let Some(start) = run_start {
                    self.fill_rect(Rect::from_edges(start, y, x, y + 1), color);
                    run_start = None;
                }
                x += 1;
            }
            if let Some(start) = run_start {
                self.fill_rect(Rect::from_edges(start, y, x1, y + 1), color);
            }
            return;
        }
        const ROW_MASK: usize = 256;
        let mut x = x0;
        while x < x1 {
            let chunk_end = (x + ROW_MASK as i32).min(x1);
            let mut mask = [0u8; ROW_MASK];
            raster_svg_mask_row(rect, y, x, chunk_end, clip_path, mask_path, &mut mask);
            let mut run_start: Option<i32> = None;
            let mut i = 0usize;
            while x + (i as i32) < chunk_end {
                if mask[i] != 0 {
                    if run_start.is_none() {
                        run_start = Some(x + i as i32);
                    }
                } else if let Some(start) = run_start {
                    self.fill_rect(Rect::from_edges(start, y, x + i as i32, y + 1), color);
                    run_start = None;
                }
                i += 1;
            }
            if let Some(start) = run_start {
                self.fill_rect(Rect::from_edges(start, y, chunk_end, y + 1), color);
            }
            x = chunk_end;
        }
    }

    fn draw_masked_svg_line<const N: usize>(
        &mut self,
        rect: Rect,
        from: Point,
        to: Point,
        stroke: u16,
        color: Color,
        clip_path: Option<&SvgPath<N>>,
        mask_path: Option<&SvgPath<N>>,
        a8_mask: Option<&SvgA8Mask>,
    ) {
        if clip_path.is_none() && mask_path.is_none() && a8_mask.is_none() {
            self.draw_wide_line(from, to, stroke, color);
            return;
        }
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let steps = dx.abs().max(dy.abs()).max(1);
        let radius = (stroke as i32 / 2).max(1);
        let mut i = 0i32;
        while i <= steps {
            let x = from.x + dx.saturating_mul(i) / steps;
            let y = from.y + dy.saturating_mul(i) / steps;
            self.fill_masked_svg_circle(rect, x, y, radius, color, clip_path, mask_path, a8_mask);
            i += 1;
        }
    }

    fn fill_masked_svg_rect<const N: usize>(
        &mut self,
        svg_rect: Rect,
        draw_rect: Rect,
        color: Color,
        clip_path: Option<&SvgPath<N>>,
        mask_path: Option<&SvgPath<N>>,
        a8_mask: Option<&SvgA8Mask>,
    ) {
        if clip_path.is_none() && mask_path.is_none() && a8_mask.is_none() {
            self.fill_rect(draw_rect, color);
            return;
        }
        let clipped = match self.clip {
            Some(clip) => draw_rect.clipped_to(clip),
            None => draw_rect,
        };
        let mut y = clipped.y;
        while y < clipped.bottom() {
            self.fill_masked_svg_span(svg_rect, y, clipped.x, clipped.right(), color, clip_path, mask_path, a8_mask);
            y += 1;
        }
    }

    fn fill_masked_svg_circle<const N: usize>(
        &mut self,
        svg_rect: Rect,
        cx: i32,
        cy: i32,
        radius: i32,
        color: Color,
        clip_path: Option<&SvgPath<N>>,
        mask_path: Option<&SvgPath<N>>,
        a8_mask: Option<&SvgA8Mask>,
    ) {
        if clip_path.is_none() && mask_path.is_none() && a8_mask.is_none() {
            self.fill_circle(cx, cy, radius, color);
            return;
        }
        let r2 = radius.saturating_mul(radius);
        let mut y = cy - radius;
        while y <= cy + radius {
            let dy = y - cy;
            let mut x = cx - radius;
            while x <= cx + radius {
                let dx = x - cx;
                let allowed = match a8_mask {
                    Some(mask) => mask.alpha_at(x, y) != 0,
                    None => svg_mask_allows(svg_rect, clip_path, mask_path, x, y),
                };
                if dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)) <= r2 && allowed {
                    self.put_pixel(x, y, color);
                }
                x += 1;
            }
            y += 1;
        }
    }

    fn build_svg_a8_mask<const N: usize>(
        &self,
        rect: Rect,
        clip_path: Option<&SvgPath<N>>,
        mask_path: Option<&SvgPath<N>>,
    ) -> Option<SvgA8Mask> {
        if clip_path.is_none() && mask_path.is_none() {
            return None;
        }
        let mut bounds = match self.clip {
            Some(clip) => rect.clipped_to(clip),
            None => rect,
        };
        if let Some(path) = clip_path {
            if let Some(path_bounds) = svg_path_screen_bounds(rect, path) {
                bounds = bounds.clipped_to(path_bounds);
            }
        }
        if let Some(path) = mask_path {
            if let Some(path_bounds) = svg_path_screen_bounds(rect, path) {
                bounds = bounds.clipped_to(path_bounds);
            }
        }
        if bounds.is_empty() {
            return Some(SvgA8Mask {
                rect: bounds,
                width: 0,
                data: Vec::new(),
            });
        }
        let scratch = VectorMaskScratch::try_for_rect(bounds, MaskRasterOptions::DEFAULT);
        if scratch.overflowed {
            return None;
        }
        let len = scratch.used_bytes;
        let mut data = Vec::new();
        data.resize(len, 0);
        let mut y = bounds.y;
        while y < bounds.bottom() {
            let mut x = bounds.x;
            while x < bounds.right() {
                let index = (y - bounds.y) as usize * bounds.w as usize + (x - bounds.x) as usize;
                if svg_mask_allows(rect, clip_path, mask_path, x, y) {
                    data[index] = 255;
                }
                x += 1;
            }
            y += 1;
        }
        Some(SvgA8Mask {
            rect: bounds,
            width: bounds.w,
            data,
        })
    }

    pub fn draw_svg_icon_placeholder(&mut self, rect: Rect, icon: SvgId, color: Color) {
        let cx = rect.x + rect.w as i32 / 2;
        let cy = rect.y + rect.h as i32 / 2;
        let r = (rect.w.min(rect.h) as i32 / 2).max(3);
        let stroke = (r / 5).max(1) as u16;

        match icon.0 {
            1 => {
                self.fill_circle(cx, cy, r, color);
                let cut = Color::rgba(0, 0, 0, 120);
                self.draw_wide_line(Point::new(cx - r / 2, cy), Point::new(cx + r / 2, cy), stroke, cut);
                self.draw_wide_line(Point::new(cx, cy - r / 2), Point::new(cx, cy + r / 2), stroke, cut);
            }
            2 => {
                self.fill_round_rect(rect, (r / 3) as u16, color);
                let cut = Color::rgba(0, 0, 0, 120);
                self.draw_wide_line(
                    Point::new(rect.x + r / 2, cy - r / 3),
                    Point::new(rect.right() - r / 2, cy - r / 3),
                    stroke,
                    cut,
                );
                self.draw_wide_line(
                    Point::new(rect.x + r / 2, cy + r / 3),
                    Point::new(rect.right() - r / 2, cy + r / 3),
                    stroke,
                    cut,
                );
            }
            3 => {
                self.fill_circle(cx, cy, r, color);
                let cut = Color::rgba(0, 0, 0, 120);
                self.fill_circle(cx - r / 3, cy - r / 5, (r / 5).max(1), cut);
                self.fill_circle(cx + r / 3, cy - r / 5, (r / 5).max(1), cut);
                self.draw_wide_line(Point::new(cx - r / 3, cy + r / 3), Point::new(cx + r / 4, cy + r / 3), stroke, cut);
            }
            4 => {
                self.draw_wide_line(Point::new(cx - r / 2, cy + r / 2), Point::new(cx, cy - r / 2), stroke, color);
                self.draw_wide_line(Point::new(cx, cy - r / 2), Point::new(cx + r / 2, cy + r / 2), stroke, color);
                self.draw_wide_line(Point::new(cx - r / 4, cy + r / 8), Point::new(cx + r / 4, cy + r / 8), stroke, color);
            }
            5 => self.draw_vector_placeholder(rect, color),
            6 => {
                self.fill_circle(cx - r / 2, cy - r / 3, (r / 4).max(1), color);
                self.fill_circle(cx + r / 2, cy - r / 3, (r / 4).max(1), color);
                self.fill_circle(cx, cy + r / 2, (r / 4).max(1), color);
                self.draw_wide_line(Point::new(cx - r / 2, cy - r / 3), Point::new(cx, cy + r / 2), stroke, color);
                self.draw_wide_line(Point::new(cx + r / 2, cy - r / 3), Point::new(cx, cy + r / 2), stroke, color);
            }
            7 => {
                self.fill_circle(cx, cy, r, color);
                let cut = Color::rgba(0, 0, 0, 150);
                self.fill_circle(cx, cy, (r / 2).max(1), cut);
                let spoke = stroke.max(2);
                self.draw_wide_line(Point::new(cx - r, cy), Point::new(cx - r / 2, cy), spoke, color);
                self.draw_wide_line(Point::new(cx + r / 2, cy), Point::new(cx + r, cy), spoke, color);
                self.draw_wide_line(Point::new(cx, cy - r), Point::new(cx, cy - r / 2), spoke, color);
                self.draw_wide_line(Point::new(cx, cy + r / 2), Point::new(cx, cy + r), spoke, color);
                self.draw_wide_line(Point::new(cx - r / 2, cy - r / 2), Point::new(cx - r / 4, cy - r / 4), spoke, color);
                self.draw_wide_line(Point::new(cx + r / 4, cy + r / 4), Point::new(cx + r / 2, cy + r / 2), spoke, color);
                self.draw_wide_line(Point::new(cx + r / 2, cy - r / 2), Point::new(cx + r / 4, cy - r / 4), spoke, color);
                self.draw_wide_line(Point::new(cx - r / 4, cy + r / 4), Point::new(cx - r / 2, cy + r / 2), spoke, color);
            }
            _ => self.draw_vector_placeholder(rect, color),
        }
    }

    pub fn draw_vector_placeholder(&mut self, rect: Rect, color: Color) {
        let cx = rect.x + rect.w as i32 / 2;
        let cy = rect.y + rect.h as i32 / 2;
        let r = (rect.w.min(rect.h) as i32 / 2).max(1);
        self.draw_wide_line(Point::new(cx - r, cy), Point::new(cx, cy - r), 2, color);
        self.draw_wide_line(Point::new(cx, cy - r), Point::new(cx + r, cy), 2, color);
        self.draw_wide_line(Point::new(cx + r, cy), Point::new(cx, cy + r), 2, color);
        self.draw_wide_line(Point::new(cx, cy + r), Point::new(cx - r, cy), 2, color);
    }
}

fn map_svg_point(rect: Rect, view_box: Rect, point: Point) -> Point {
    let vw = view_box.w.max(1) as i32;
    let vh = view_box.h.max(1) as i32;
    Point::new(
        rect.x + (point.x.saturating_sub(view_box.x)).saturating_mul(rect.w as i32) / vw,
        rect.y + (point.y.saturating_sub(view_box.y)).saturating_mul(rect.h as i32) / vh,
    )
}

fn map_svg_rect(rect: Rect, view_box: Rect, local: Rect) -> Rect {
    let p0 = map_svg_point(rect, view_box, Point::new(local.x, local.y));
    let p1 = map_svg_point(rect, view_box, Point::new(local.right(), local.bottom()));
    Rect::from_edges(p0.x.min(p1.x), p0.y.min(p1.y), p0.x.max(p1.x), p0.y.max(p1.y))
}

fn svg_path_screen_bounds<const N: usize>(rect: Rect, path: &SvgPath<N>) -> Option<Rect> {
    if path.len == 0 || path.view_box.is_empty() {
        return None;
    }
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    let mut seen = false;
    let mut i = 0usize;
    while i < path.len {
        match path.commands[i] {
            SvgPathCommand::MoveTo(point) | SvgPathCommand::LineTo(point) => {
                let mapped = map_svg_point(rect, path.view_box, point);
                min_x = min_x.min(mapped.x);
                min_y = min_y.min(mapped.y);
                max_x = max_x.max(mapped.x);
                max_y = max_y.max(mapped.y);
                seen = true;
            }
            SvgPathCommand::Empty | SvgPathCommand::Close => {}
        }
        i += 1;
    }
    if seen {
        Some(Rect::from_edges(min_x, min_y, max_x.saturating_add(1), max_y.saturating_add(1)))
    } else {
        None
    }
}

fn expand_rect(rect: Rect, amount: i32) -> Rect {
    Rect::from_edges(
        rect.x - amount,
        rect.y - amount,
        rect.right() + amount,
        rect.bottom() + amount,
    )
}

fn resolve_svg_fill(paint: SvgPaint, current: Color, opacity: u8) -> Option<SvgPaint> {
    match paint {
        SvgPaint::None => None,
        SvgPaint::CurrentColor => Some(SvgPaint::Color(Color::rgba(
            current.r,
            current.g,
            current.b,
            alpha_mul(current.a, opacity),
        ))),
        SvgPaint::Color(color) => {
            let alpha = alpha_mul(alpha_mul(color.a, opacity), current.a);
            if alpha == 0 {
                None
            } else {
                Some(SvgPaint::Color(Color::rgba(color.r, color.g, color.b, alpha)))
            }
        }
        SvgPaint::LinearGradient(start, end) => Some(SvgPaint::LinearGradient(
            Color::rgba(start.r, start.g, start.b, alpha_mul(alpha_mul(start.a, opacity), current.a)),
            Color::rgba(end.r, end.g, end.b, alpha_mul(alpha_mul(end.a, opacity), current.a)),
        )),
        SvgPaint::RadialGradient(start, end) => Some(SvgPaint::RadialGradient(
            Color::rgba(start.r, start.g, start.b, alpha_mul(alpha_mul(start.a, opacity), current.a)),
            Color::rgba(end.r, end.g, end.b, alpha_mul(alpha_mul(end.a, opacity), current.a)),
        )),
    }
}

fn resolve_svg_paint(paint: SvgPaint, current: Color, opacity: u8) -> Option<Color> {
    let (base, alpha) = match paint {
        SvgPaint::None => return None,
        SvgPaint::CurrentColor => (current, alpha_mul(current.a, opacity)),
        SvgPaint::Color(color) => (
            color,
            alpha_mul(alpha_mul(color.a, opacity), current.a),
        ),
        SvgPaint::LinearGradient(start, _) | SvgPaint::RadialGradient(start, _) => (
            start,
            alpha_mul(alpha_mul(start.a, opacity), current.a),
        ),
    };
    if alpha == 0 {
        None
    } else {
        Some(Color::rgba(base.r, base.g, base.b, alpha))
    }
}

fn lerp_color(a: Color, b: Color, t: u8) -> Color {
    let inv = 255u16.saturating_sub(t as u16);
    let t = t as u16;
    Color::rgba(
        ((a.r as u16 * inv + b.r as u16 * t) / 255) as u8,
        ((a.g as u16 * inv + b.g as u16 * t) / 255) as u8,
        ((a.b as u16 * inv + b.b as u16 * t) / 255) as u8,
        ((a.a as u16 * inv + b.a as u16 * t) / 255) as u8,
    )
}

fn alpha_mul(a: u8, b: u8) -> u8 {
    ((a as u16 * b as u16) / 255) as u8
}

fn collect_svg_scanline_intersections<const N: usize>(
    rect: Rect,
    path: &SvgPath<N>,
    y: i32,
    xs: &mut [i32; N],
    winds: &mut [i8; N],
    count: &mut usize,
) {
    let mut current = Point::new(0, 0);
    let mut sub_start = Point::new(0, 0);
    let mut have_current = false;
    let scan_y = y.saturating_mul(2).saturating_add(1);
    let mut index = 0usize;
    while index < path.len {
        match path.commands[index] {
            SvgPathCommand::Empty => {}
            SvgPathCommand::MoveTo(point) => {
                current = map_svg_point(rect, path.view_box, point);
                sub_start = current;
                have_current = true;
            }
            SvgPathCommand::LineTo(point) => {
                let next = map_svg_point(rect, path.view_box, point);
                if have_current {
                    push_svg_edge_intersection(current, next, scan_y, xs, winds, count);
                }
                current = next;
                have_current = true;
            }
            SvgPathCommand::Close => {
                if have_current {
                    push_svg_edge_intersection(current, sub_start, scan_y, xs, winds, count);
                    current = sub_start;
                }
            }
        }
        index += 1;
    }
}

fn svg_path_contains<const N: usize>(rect: Rect, path: &SvgPath<N>, x: i32, y: i32) -> bool {
    if path.len == 0 || path.view_box.is_empty() {
        return false;
    }
    let mut xs = [0i32; N];
    let mut winds = [0i8; N];
    let mut count = 0usize;
    collect_svg_scanline_intersections(rect, path, y, &mut xs, &mut winds, &mut count);
    sort_svg_intersections(&mut xs, &mut winds, count);
    let mut i = 0usize;
    while i + 1 < count {
        if x >= xs[i] && x < xs[i + 1] {
            return true;
        }
        i += 2;
    }
    false
}

fn svg_mask_allows<const N: usize>(
    rect: Rect,
    clip_path: Option<&SvgPath<N>>,
    mask_path: Option<&SvgPath<N>>,
    x: i32,
    y: i32,
) -> bool {
    let in_clip = clip_path.map(|path| svg_path_contains(rect, path, x, y)).unwrap_or(true);
    let in_mask = mask_path.map(|path| svg_path_contains(rect, path, x, y)).unwrap_or(true);
    in_clip && in_mask
}

fn raster_svg_mask_row<const N: usize>(
    rect: Rect,
    y: i32,
    x0: i32,
    x1: i32,
    clip_path: Option<&SvgPath<N>>,
    mask_path: Option<&SvgPath<N>>,
    out: &mut [u8],
) {
    let mut i = 0usize;
    while i < out.len() {
        out[i] = 0;
        i += 1;
    }
    let mut x = x0;
    let mut index = 0usize;
    while x < x1 && index < out.len() {
        if svg_mask_allows(rect, clip_path, mask_path, x, y) {
            out[index] = 255;
        }
        x += 1;
        index += 1;
    }
}

fn push_svg_edge_intersection<const N: usize>(
    from: Point,
    to: Point,
    scan_y2: i32,
    xs: &mut [i32; N],
    winds: &mut [i8; N],
    count: &mut usize,
) {
    let y0 = from.y.saturating_mul(2);
    let y1 = to.y.saturating_mul(2);
    if y0 == y1 {
        return;
    }
    let ymin = y0.min(y1);
    let ymax = y0.max(y1);
    if scan_y2 < ymin || scan_y2 >= ymax || *count >= N {
        return;
    }

    let dy = y1 - y0;
    let dx = to.x - from.x;
    let x = from.x + ((scan_y2 - y0) * dx) / dy;
    xs[*count] = x;
    winds[*count] = if dy > 0 { 1 } else { -1 };
    *count += 1;
}

fn sort_svg_intersections<const N: usize>(xs: &mut [i32; N], winds: &mut [i8; N], count: usize) {
    let mut i = 1usize;
    while i < count {
        let item = xs[i];
        let wind = winds[i];
        let mut j = i;
        while j > 0 && xs[j - 1] > item {
            xs[j] = xs[j - 1];
            winds[j] = winds[j - 1];
            j -= 1;
        }
        xs[j] = item;
        winds[j] = wind;
        i += 1;
    }
}

fn builtin_svg_icon_path(icon: SvgId) -> Option<&'static str> {
    match icon.0 {
        1 => Some("M8 1 L15 8 L8 15 L1 8 Z M8 4 V12 M4 8 H12"),
        2 => Some("M3 3 H13 V13 H3 Z M5 6 H11 M5 10 H11"),
        3 => Some("M8 2 C5 2 3 4 3 7 C3 11 6 14 8 14 C10 14 13 11 13 7 C13 4 11 2 8 2 M5 7 H6 M10 7 H11 M6 10 H10"),
        4 => Some("M2 14 L6 2 H10 L14 14 M4 10 H12"),
        5 => Some("M2 4 L8 1 L14 4 V12 L8 15 L2 12 Z M5 8 H11 M8 5 V11"),
        6 => Some("M4 4 H12 V12 H4 Z M8 1 V4 M8 12 V15 M1 8 H4 M12 8 H15"),
        7 => Some("M8 1 V4 M8 12 V15 M1 8 H4 M12 8 H15 M3 3 L5 5 M11 11 L13 13 M13 3 L11 5 M5 11 L3 13 M5 8 C5 6 6 5 8 5 C10 5 11 6 11 8 C11 10 10 11 8 11 C6 11 5 10 5 8"),
        _ => None,
    }
}
