use crate::{
    glyph::glyph_5x7_a8, Color, FontId, GlyphView, Point, Rect, Surface, TextAlign, TextDecor,
    TextLayout, TextLayoutLine, TextLayoutOptions, TextStyle, GLYPH_RUN_RESOLVER_CAPACITY,
};
use crate::ttf::{GlyphRun, GlyphRunItem, ShapeOptions};

impl Surface {
    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: Color, scale: u16) {
        self.draw_text_font(x, y, text, FontId(0), color, scale);
    }

    pub fn draw_text_font(&mut self, mut x: i32, mut y: i32, text: &str, font: FontId, color: Color, scale: u16) {
        let origin_x = x;
        let line_h = if font.0 == 0 {
            8i32.saturating_mul(scale.max(1) as i32)
        } else {
            scale.max(1) as i32 + 4
        };
        for ch in text.chars() {
            if ch == '\n' {
                x = origin_x;
                y += line_h;
                continue;
            }
            x += self.draw_char_font(x, y, ch, font, color, scale);
        }
    }

    pub fn draw_label(&mut self, rect: Rect, text: &str, style: TextStyle) {
        if rect.is_empty() || style.color.a == 0 {
            return;
        }

        let old_clip = self.clip();
        self.set_clip(Some(match old_clip {
            Some(clip) => rect.clipped_to(clip),
            None => rect,
        }));

        let scale = style.scale.max(1);
        let glyph_w = if style.font.0 == 0 { 6u16.saturating_mul(scale) } else { (scale / 2).max(4) };
        let line_h = if style.font.0 == 0 {
            8u16.saturating_mul(scale)
        } else {
            scale.saturating_add(4)
        }
        .saturating_add(style.line_spacing.max(0) as u16)
        .max(1);
        if let Some(run) = self.resolve_label_glyph_run(text, style) {
            let layout = TextLayout::<32>::from_glyph_run(
                &run,
                TextLayoutOptions {
                    max_width: rect.w,
                    line_height: line_h,
                    letter_spacing: style.letter_spacing,
                    line_spacing: style.line_spacing,
                    wrap: true,
                },
            );
            if !layout.is_empty() {
                self.draw_label_glyph_run(rect, &run, &layout, style, line_h);
                self.set_clip(old_clip);
                return;
            }
        }
        let layout = self.build_label_layout::<32>(rect, text, style, line_h);
        let mut index = 0usize;
        while index < layout.len {
            let line = layout.lines[index];
            let y = rect.y + line.y_offset;
            if y >= rect.bottom() {
                break;
            }
            self.draw_label_line(
                rect,
                &text[line.byte_start..line.byte_end],
                y,
                line.char_start,
                style,
                glyph_w,
                line_h,
            );
            index += 1;
        }

        self.set_clip(old_clip);
    }

    fn resolve_label_glyph_run(
        &self,
        text: &str,
        style: TextStyle,
    ) -> Option<GlyphRun<GLYPH_RUN_RESOLVER_CAPACITY>> {
        if style.font.0 == 0 {
            return None;
        }
        let resolver = self.glyph_run_resolver?;
        let options = ShapeOptions::new(true, true, style.kerning);
        let run = resolver(style.font, text, style.scale.max(1), options)?;
        if run.len == 0 {
            None
        } else {
            Some(run)
        }
    }

    fn draw_label_glyph_run<const G: usize, const L: usize>(
        &mut self,
        rect: Rect,
        run: &GlyphRun<G>,
        layout: &TextLayout<L>,
        style: TextStyle,
        line_h: u16,
    ) {
        let mut index = 0usize;
        while index < layout.len {
            let line = layout.lines[index];
            let y = rect.y + line.y_offset;
            if y >= rect.bottom() {
                break;
            }
            self.draw_glyph_run_line(rect, run, line, y, style, line_h);
            index += 1;
        }
    }

    fn draw_glyph_run_line<const G: usize>(
        &mut self,
        rect: Rect,
        run: &GlyphRun<G>,
        line: TextLayoutLine,
        y: i32,
        style: TextStyle,
        line_h: u16,
    ) {
        if line.glyph_len == 0 {
            return;
        }
        let width = line.width;
        let x = match style.align {
            TextAlign::Left => rect.x,
            TextAlign::Center => rect.x + rect.w.saturating_sub(width) as i32 / 2,
            TextAlign::Right => rect.right() - width as i32,
        };

        if let Some((start, end, fg, bg)) = style.selection {
            let line_start = line.char_start;
            let line_end = line.char_start.saturating_add(line.char_len);
            if start < line_end && end > line_start {
                let (sx, ex) = self.selection_x_range_run(run, line, x, style, start, end);
                if ex > sx {
                    self.fill_rect(Rect::new(sx, y, (ex - sx).min(u16::MAX as i32) as u16, line_h), bg);
                }
                self.draw_glyph_run_text(run, line, x, y, style, Some((start, end, fg)));
            } else {
                self.draw_glyph_run_text(run, line, x, y, style, None);
            }
        } else {
            self.draw_glyph_run_text(run, line, x, y, style, None);
        }

        if style.decor.contains(TextDecor::UNDERLINE) {
            let ly = y + line_h as i32 - 2;
            self.draw_wide_line(Point::new(x, ly), Point::new(x + width as i32, ly), 1, style.color);
        }
        if style.decor.contains(TextDecor::STRIKETHROUGH) {
            let ly = y + line_h as i32 / 2;
            self.draw_wide_line(Point::new(x, ly), Point::new(x + width as i32, ly), 1, style.color);
        }
    }

    fn draw_glyph_run_text<const G: usize>(
        &mut self,
        run: &GlyphRun<G>,
        line: TextLayoutLine,
        mut x: i32,
        y: i32,
        style: TextStyle,
        selection: Option<(u16, u16, Color)>,
    ) {
        let spacing = style.letter_spacing.max(0) as i32;
        let end = line.glyph_start.saturating_add(line.glyph_len).min(run.len);
        let mut i = line.glyph_start;
        while i < end {
            let item = run.items[i];
            let (char_start, char_len) = glyph_item_char_range(item, i);
            let color = match selection {
                Some((start, end, fg)) if ranges_intersect(char_start, char_len, start, end) => fg,
                _ => style.color,
            };
            self.draw_glyph_run_item_font(
                x.saturating_add(item.x_offset as i32),
                y,
                item,
                style.font,
                color,
                style.scale,
            );
            x = x.saturating_add(item.advance.max(1) as i32).saturating_add(spacing);
            i += 1;
        }
    }

    fn selection_x_range_run<const G: usize>(
        &self,
        run: &GlyphRun<G>,
        line: TextLayoutLine,
        x: i32,
        style: TextStyle,
        selection_start: u16,
        selection_end: u16,
    ) -> (i32, i32) {
        let spacing = style.letter_spacing.max(0) as i32;
        let end = line.glyph_start.saturating_add(line.glyph_len).min(run.len);
        let mut pen = x;
        let mut start_x: Option<i32> = None;
        let mut end_x = x;
        let mut i = line.glyph_start;
        while i < end {
            let item = run.items[i];
            let (char_start, char_len) = glyph_item_char_range(item, i);
            if char_range_end(char_start, char_len) > selection_start && start_x.is_none() {
                start_x = Some(pen);
            }
            pen = pen.saturating_add(item.advance.max(1) as i32).saturating_add(spacing);
            if char_start < selection_end {
                end_x = pen;
            }
            i += 1;
        }
        (start_x.unwrap_or(x), end_x)
    }

    fn build_label_layout<const N: usize>(
        &self,
        rect: Rect,
        text: &str,
        style: TextStyle,
        line_h: u16,
    ) -> TextLayout<N> {
        let mut layout = TextLayout::new();
        let mut line_start = 0usize;
        let mut total_chars = 0u16;
        let mut y = 0i32;
        while line_start < text.len() && rect.y + y < rect.bottom() {
            let (line_end, next_start, consumed_chars) =
                self.find_label_line_break(text, line_start, rect.w, style);
            let width = self.measure_label_line(
                &text[line_start..line_end],
                style,
                if style.font.0 == 0 {
                    6u16.saturating_mul(style.scale.max(1))
                } else {
                    (style.scale.max(1) / 2).max(4)
                },
                style.letter_spacing.max(0) as u16,
            );
            layout.push(TextLayoutLine {
                byte_start: line_start,
                byte_end: line_end,
                next_byte: next_start,
                glyph_start: total_chars as usize,
                glyph_len: consumed_chars as usize,
                char_start: total_chars,
                char_len: consumed_chars,
                width,
                y_offset: y,
            });
            if next_start <= line_start {
                break;
            }
            total_chars = total_chars.saturating_add(consumed_chars);
            line_start = next_start;
            y = y.saturating_add(line_h as i32);
        }
        if line_start < text.len() {
            layout.overflowed = true;
        }
        layout
    }

    fn find_label_line_break(
        &self,
        text: &str,
        start: usize,
        max_width: u16,
        style: TextStyle,
    ) -> (usize, usize, u16) {
        let limit = max_width.max(1) as i32;
        let letter_extra = style.letter_spacing.max(0) as i32;
        let mut width = 0i32;
        let mut chars = 0u16;
        let mut consumed = 0u16;
        let mut previous: Option<u32> = None;
        let mut last_space: Option<(usize, usize, u16, u16)> = None;

        for (relative, ch) in text[start..].char_indices() {
            let byte_index = start + relative;
            let next = byte_index + ch.len_utf8();
            if ch == '\n' {
                return (byte_index, next, consumed.saturating_add(1));
            }

            let codepoint = ch as u32;
            let kern = if style.kerning {
                previous
                    .map(|prev| self.kerning_adjust(style.font, prev, codepoint, style.scale) as i32)
                    .unwrap_or(0)
            } else {
                0
            };
            let advance = self.glyph_advance(style.font, ch, style.scale).max(1);
            let candidate_width = width.saturating_add(kern).saturating_add(advance);
            if chars != 0 && candidate_width > limit {
                if let Some((space_start, space_end, visible_chars, consumed_at_space)) = last_space {
                    if visible_chars != 0 {
                        return (space_start, space_end, consumed_at_space);
                    }
                }
                return (byte_index, byte_index, consumed.max(1));
            }

            width = candidate_width.saturating_add(letter_extra);
            consumed = consumed.saturating_add(1);
            if ch == ' ' || ch == '\t' {
                last_space = Some((byte_index, next, chars, consumed));
            } else {
                chars = chars.saturating_add(1);
            }
            previous = Some(codepoint);
        }

        (text.len(), text.len(), consumed)
    }

    fn draw_label_line(
        &mut self,
        rect: Rect,
        line: &str,
        y: i32,
        char_offset: u16,
        style: TextStyle,
        glyph_w: u16,
        line_h: u16,
    ) {
        if line.is_empty() {
            return;
        }
        let count = line.chars().count().min(u16::MAX as usize) as u16;
        let letter_extra = style.letter_spacing.max(0) as u16;
        let width = self.measure_label_line(line, style, glyph_w, letter_extra);
        let x = match style.align {
            TextAlign::Left => rect.x,
            TextAlign::Center => rect.x + rect.w.saturating_sub(width) as i32 / 2,
            TextAlign::Right => rect.right() - width as i32,
        };

        if let Some((start, end, fg, bg)) = style.selection {
            let line_start = char_offset;
            let line_end = char_offset.saturating_add(count);
            if start < line_end && end > line_start {
                let sel0 = start.saturating_sub(line_start).min(count);
                let sel1 = end.saturating_sub(line_start).min(count);
                if sel1 > sel0 {
                    let (sx, ex) =
                        self.selection_x_range(line, x, style, char_offset, start, end, glyph_w, letter_extra);
                    if ex > sx {
                        self.fill_rect(Rect::new(sx, y, (ex - sx).min(u16::MAX as i32) as u16, line_h), bg);
                    }
                    self.draw_label_selected_text(x, y, line, style, char_offset, start, end, fg);
                } else {
                    self.draw_label_text(x, y, line, style, style.color);
                }
            } else {
                self.draw_label_text(x, y, line, style, style.color);
            }
        } else {
            self.draw_label_text(x, y, line, style, style.color);
        }

        if style.decor.contains(TextDecor::UNDERLINE) {
            let ly = y + line_h as i32 - 2;
            self.draw_wide_line(Point::new(x, ly), Point::new(x + width as i32, ly), 1, style.color);
        }
        if style.decor.contains(TextDecor::STRIKETHROUGH) {
            let ly = y + line_h as i32 / 2;
            self.draw_wide_line(Point::new(x, ly), Point::new(x + width as i32, ly), 1, style.color);
        }
    }

    fn draw_text_font_spaced(
        &mut self,
        mut x: i32,
        y: i32,
        text: &str,
        font: FontId,
        color: Color,
        scale: u16,
        letter_spacing: i16,
    ) {
        let spacing = letter_spacing.max(0) as i32;
        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            x += self.draw_char_font(x, y, ch, font, color, scale) + spacing;
        }
    }

    fn draw_label_text(&mut self, x: i32, y: i32, text: &str, style: TextStyle, color: Color) {
        if style.kerning {
            self.draw_text_font_spaced_with_kerning(x, y, text, style, color);
        } else {
            self.draw_text_font_spaced(
                x,
                y,
                text,
                style.font,
                color,
                style.scale,
                style.letter_spacing,
            );
        }
    }

    fn draw_text_font_spaced_with_kerning(
        &mut self,
        mut x: i32,
        y: i32,
        text: &str,
        style: TextStyle,
        color: Color,
    ) {
        let spacing = style.letter_spacing.max(0) as i32;
        let mut previous: Option<u32> = None;
        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            let codepoint = ch as u32;
            if style.kerning {
                if let Some(prev) = previous {
                    x += self.kerning_adjust(style.font, prev, codepoint, style.scale) as i32;
                }
            }
            x += self.draw_char_font(x, y, ch, style.font, color, style.scale) + spacing;
            previous = Some(codepoint);
        }
    }

    fn draw_label_selected_text(
        &mut self,
        mut x: i32,
        y: i32,
        text: &str,
        style: TextStyle,
        char_offset: u16,
        selection_start: u16,
        selection_end: u16,
        selected_fg: Color,
    ) {
        let spacing = style.letter_spacing.max(0) as i32;
        let mut index = char_offset;
        let mut previous: Option<u32> = None;
        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            let codepoint = ch as u32;
            if style.kerning {
                if let Some(prev) = previous {
                    x += self.kerning_adjust(style.font, prev, codepoint, style.scale) as i32;
                }
            }
            let color = if index >= selection_start && index < selection_end {
                selected_fg
            } else {
                style.color
            };
            x += self.draw_char_font(x, y, ch, style.font, color, style.scale) + spacing;
            index = index.saturating_add(1);
            previous = Some(codepoint);
        }
    }

    fn measure_label_line(&self, line: &str, style: TextStyle, glyph_w: u16, letter_extra: u16) -> u16 {
        let mut width = 0i32;
        let mut count = 0usize;
        let mut previous: Option<u32> = None;
        for ch in line.chars() {
            if ch == '\n' {
                continue;
            }
            let codepoint = ch as u32;
            if style.kerning {
                if let Some(prev) = previous {
                    width += self.kerning_adjust(style.font, prev, codepoint, style.scale) as i32;
                }
            }
            width += self.glyph_advance(style.font, ch, style.scale).max(1);
            width += letter_extra as i32;
            previous = Some(codepoint);
            count += 1;
        }
        if count != 0 {
            width -= letter_extra as i32;
        }
        width.max(glyph_w as i32).min(u16::MAX as i32) as u16
    }

    fn selection_x_range(
        &self,
        line: &str,
        x: i32,
        style: TextStyle,
        char_offset: u16,
        selection_start: u16,
        selection_end: u16,
        glyph_w: u16,
        letter_extra: u16,
    ) -> (i32, i32) {
        let mut pen = x;
        let mut start_x: Option<i32> = None;
        let mut end_x = x;
        let mut index = char_offset;
        let mut previous: Option<u32> = None;
        for ch in line.chars() {
            if ch == '\n' {
                continue;
            }
            let codepoint = ch as u32;
            if style.kerning {
                if let Some(prev) = previous {
                    pen += self.kerning_adjust(style.font, prev, codepoint, style.scale) as i32;
                }
            }
            if index >= selection_start && start_x.is_none() {
                start_x = Some(pen);
            }
            let advance = self.glyph_advance(style.font, ch, style.scale).max(glyph_w as i32);
            pen += advance + letter_extra as i32;
            if index < selection_end {
                end_x = pen;
            }
            previous = Some(codepoint);
            index = index.saturating_add(1);
        }
        (start_x.unwrap_or(x), end_x)
    }

    fn glyph_advance(&self, font: FontId, ch: char, scale: u16) -> i32 {
        if font.0 != 0 {
            if let Some(resolve) = self.glyph_resolver {
                if let Some(glyph) = resolve(font, ch as u32, scale.max(1)) {
                    return glyph.advance.max(1) as i32;
                }
            }
            return (scale.max(1) / 2).max(4) as i32;
        }
        if ch.is_ascii() {
            glyph_5x7_a8(ch as u8).advance as i32 * scale.max(1) as i32
        } else {
            6 * scale.max(1) as i32
        }
    }

    fn kerning_adjust(&self, font: FontId, left: u32, right: u32, scale: u16) -> i16 {
        if font.0 == 0 {
            return 0;
        }
        match self.kerning_resolver {
            Some(resolve) => resolve(font, left, right, scale.max(1)),
            None => 0,
        }
    }

    fn draw_char_font(&mut self, x: i32, y: i32, ch: char, font: FontId, color: Color, scale: u16) -> i32 {
        if font.0 != 0 {
            let size = scale.max(1);
            let codepoint = ch as u32;
            if let Some(resolve) = self.glyph_resolver {
                if let Some(glyph) = resolve(font, codepoint, size) {
                    self.draw_glyph_view(x, y, glyph, color, size);
                    return glyph.advance.max(1) as i32;
                }
            }
            self.draw_missing_glyph(x, y, size, color);
            return (size / 2).max(4) as i32;
        }

        let scale = scale.max(1) as i32;
        if !ch.is_ascii() {
            self.draw_missing_glyph(x, y, (7 * scale).max(8) as u16, color);
            return 6 * scale;
        }

        let glyph = glyph_5x7_a8(ch as u8);
        let mut row = 0;
        while row < glyph.height {
            let mut col = 0;
            while col < glyph.width {
                let alpha = glyph.alpha_at(col, row);
                if alpha != 0 {
                    let a = ((color.a as u16 * alpha as u16) / 255) as u8;
                    self.fill_rect(
                        Rect::new(
                            x + col as i32 * scale,
                            y + row as i32 * scale,
                            scale as u16,
                            scale as u16,
                        ),
                        Color::rgba(color.r, color.g, color.b, a),
                    );
                }
                col += 1;
            }
            row += 1;
        }
        glyph.advance as i32 * scale
    }

    fn draw_codepoint_font(
        &mut self,
        x: i32,
        y: i32,
        codepoint: u32,
        font: FontId,
        color: Color,
        scale: u16,
    ) -> i32 {
        if let Some(ch) = core::char::from_u32(codepoint) {
            self.draw_char_font(x, y, ch, font, color, scale)
        } else {
            let size = scale.max(1);
            self.draw_missing_glyph(x, y, size, color);
            (size / 2).max(4) as i32
        }
    }

    fn draw_glyph_run_item_font(
        &mut self,
        x: i32,
        y: i32,
        item: GlyphRunItem,
        font: FontId,
        color: Color,
        scale: u16,
    ) -> i32 {
        let size = scale.max(1);
        if font.0 != 0 && item.glyph_id != 0 {
            if let Some(resolve) = self.glyph_id_resolver {
                if let Some(glyph) = resolve(font, item.glyph_id, size) {
                    self.draw_glyph_view(x, y, glyph, color, size);
                    return glyph.advance.max(1) as i32;
                }
            }
        }
        self.draw_codepoint_font(x, y, item.codepoint, font, color, scale)
    }

    fn draw_glyph_view(&mut self, x: i32, y: i32, glyph: GlyphView, color: Color, size: u16) {
        let baseline = y + size.max(1) as i32;
        let start_x = x + glyph.bearing_x as i32;
        let start_y = baseline - glyph.bearing_y as i32;
        let mut row = 0u8;
        while row < glyph.height {
            let mut col = 0u8;
            while col < glyph.width {
                let alpha = glyph.alpha_at(col, row);
                if alpha != 0 {
                    let a = ((color.a as u16 * alpha as u16) / 255) as u8;
                    self.put_pixel(
                        start_x + col as i32,
                        start_y + row as i32,
                        Color::rgba(color.r, color.g, color.b, a),
                    );
                }
                col += 1;
            }
            row += 1;
        }
    }

    fn draw_missing_glyph(&mut self, x: i32, y: i32, size: u16, color: Color) {
        let h = size.max(8);
        let w = (h / 2).max(4);
        self.draw_wide_line(Point::new(x, y), Point::new(x + w as i32, y), 1, color);
        self.draw_wide_line(Point::new(x + w as i32, y), Point::new(x + w as i32, y + h as i32), 1, color);
        self.draw_wide_line(Point::new(x + w as i32, y + h as i32), Point::new(x, y + h as i32), 1, color);
        self.draw_wide_line(Point::new(x, y + h as i32), Point::new(x, y), 1, color);
    }
}

fn glyph_item_char_range(item: GlyphRunItem, fallback_index: usize) -> (u16, u16) {
    if item.char_len == 0 {
        (fallback_index.min(u16::MAX as usize) as u16, 1)
    } else {
        (item.char_start, item.char_len)
    }
}

fn char_range_end(start: u16, len: u16) -> u16 {
    start.saturating_add(len.max(1))
}

fn ranges_intersect(start: u16, len: u16, selection_start: u16, selection_end: u16) -> bool {
    start < selection_end && char_range_end(start, len) > selection_start
}
