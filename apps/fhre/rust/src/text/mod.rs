use crate::ttf::GlyphRun;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLayoutOptions {
    pub max_width: u16,
    pub line_height: u16,
    pub letter_spacing: i16,
    pub line_spacing: i16,
    pub wrap: bool,
}

impl TextLayoutOptions {
    pub const fn new(max_width: u16, line_height: u16) -> Self {
        Self {
            max_width,
            line_height,
            letter_spacing: 0,
            line_spacing: 0,
            wrap: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLayoutLine {
    pub byte_start: usize,
    pub byte_end: usize,
    pub next_byte: usize,
    pub glyph_start: usize,
    pub glyph_len: usize,
    pub char_start: u16,
    pub char_len: u16,
    pub width: u16,
    pub y_offset: i32,
}

impl TextLayoutLine {
    pub const EMPTY: Self = Self {
        byte_start: 0,
        byte_end: 0,
        next_byte: 0,
        glyph_start: 0,
        glyph_len: 0,
        char_start: 0,
        char_len: 0,
        width: 0,
        y_offset: 0,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLayout<const N: usize> {
    pub lines: [TextLayoutLine; N],
    pub len: usize,
    pub overflowed: bool,
}

impl<const N: usize> TextLayout<N> {
    pub const fn new() -> Self {
        Self {
            lines: [TextLayoutLine::EMPTY; N],
            len: 0,
            overflowed: false,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn push(&mut self, line: TextLayoutLine) {
        if self.len >= N {
            self.overflowed = true;
            return;
        }
        self.lines[self.len] = line;
        self.len += 1;
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn from_glyph_run<const G: usize>(
        run: &GlyphRun<G>,
        options: TextLayoutOptions,
    ) -> Self {
        let mut layout = Self::new();
        let limit = options.max_width.max(1) as i32;
        let line_h = options
            .line_height
            .saturating_add(options.line_spacing.max(0) as u16)
            .max(1) as i32;
        let mut glyph_start = 0usize;
        let mut width = 0i32;
        let mut y = 0i32;
        let mut i = 0usize;
        while i < run.len {
            let advance = run.items[i].advance.max(1) as i32
                + options.letter_spacing.max(0) as i32;
            if options.wrap && i > glyph_start && width.saturating_add(advance) > limit {
                let (char_start, char_len) = glyph_run_line_char_range(run, glyph_start, i - glyph_start);
                layout.push(TextLayoutLine {
                    byte_start: 0,
                    byte_end: 0,
                    next_byte: 0,
                    glyph_start,
                    glyph_len: i - glyph_start,
                    char_start,
                    char_len,
                    width: width.max(0).min(u16::MAX as i32) as u16,
                    y_offset: y,
                });
                glyph_start = i;
                width = 0;
                y = y.saturating_add(line_h);
            }
            width = width.saturating_add(advance);
            i += 1;
        }
        if glyph_start < run.len || layout.is_empty() {
            let (char_start, char_len) =
                glyph_run_line_char_range(run, glyph_start, run.len.saturating_sub(glyph_start));
            layout.push(TextLayoutLine {
                byte_start: 0,
                byte_end: 0,
                next_byte: 0,
                glyph_start,
                glyph_len: run.len.saturating_sub(glyph_start),
                char_start,
                char_len,
                width: width.max(0).min(u16::MAX as i32) as u16,
                y_offset: y,
            });
        }
        layout.overflowed |= run.overflowed;
        layout
    }
}

fn glyph_run_line_char_range<const G: usize>(
    run: &GlyphRun<G>,
    glyph_start: usize,
    glyph_len: usize,
) -> (u16, u16) {
    if glyph_len == 0 || glyph_start >= run.len {
        return (glyph_start.min(u16::MAX as usize) as u16, 0);
    }
    let end = glyph_start.saturating_add(glyph_len).min(run.len);
    let mut index = glyph_start;
    let mut start = u16::MAX;
    let mut stop = 0u16;
    while index < end {
        let (item_start, item_len) = glyph_item_char_range(run.items[index], index);
        start = start.min(item_start);
        stop = stop.max(item_start.saturating_add(item_len));
        index += 1;
    }
    if start == u16::MAX {
        (glyph_start.min(u16::MAX as usize) as u16, glyph_len.min(u16::MAX as usize) as u16)
    } else {
        (start, stop.saturating_sub(start))
    }
}

fn glyph_item_char_range(item: crate::ttf::GlyphRunItem, fallback_index: usize) -> (u16, u16) {
    if item.char_len == 0 {
        (fallback_index.min(u16::MAX as usize) as u16, 1)
    } else {
        (item.char_start, item.char_len)
    }
}

pub(crate) fn glyph_5x7(byte: u8) -> [u8; 7] {
    match byte {
        b'0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        b'1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        b'2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        b'3' => [0x1f, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0e],
        b'4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        b'5' => [0x1f, 0x10, 0x1e, 0x01, 0x01, 0x11, 0x0e],
        b'6' => [0x06, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        b'7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        b'8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        b'9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x0c],
        b'A' | b'a' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        b'B' | b'b' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        b'C' | b'c' => [0x0e, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0e],
        b'D' | b'd' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        b'E' | b'e' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        b'F' | b'f' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        b'G' | b'g' => [0x0e, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0f],
        b'H' | b'h' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        b'I' | b'i' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        b'J' | b'j' => [0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x0e],
        b'K' | b'k' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        b'L' | b'l' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        b'M' | b'm' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        b'N' | b'n' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        b'O' | b'o' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        b'P' | b'p' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        b'Q' | b'q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        b'R' | b'r' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        b'S' | b's' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        b'T' | b't' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        b'U' | b'u' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        b'V' | b'v' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        b'W' | b'w' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0a],
        b'X' | b'x' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        b'Y' | b'y' => [0x11, 0x11, 0x0a, 0x04, 0x04, 0x04, 0x04],
        b'Z' | b'z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        b'-' => [0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00],
        b'_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1f],
        b':' => [0x00, 0x04, 0x04, 0x00, 0x04, 0x04, 0x00],
        b'.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0c, 0x0c],
        b'+' => [0x00, 0x04, 0x04, 0x1f, 0x04, 0x04, 0x00],
        b'/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        b'%' => [0x18, 0x19, 0x02, 0x04, 0x08, 0x13, 0x03],
        b' ' => [0, 0, 0, 0, 0, 0, 0],
        _ => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
    }
}
