use crate::{
    draw::FontId,
    text::glyph_5x7,
    ttf::{GlyphRun, ShapeOptions},
};

pub const GLYPH_RUN_RESOLVER_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphA8 {
    pub width: u8,
    pub height: u8,
    pub advance: u8,
    rows: [u8; 7],
}

impl GlyphA8 {
    pub const fn new(width: u8, height: u8, advance: u8, rows: [u8; 7]) -> Self {
        Self {
            width,
            height,
            advance,
            rows,
        }
    }

    pub fn alpha_at(self, x: u8, y: u8) -> u8 {
        if x >= self.width || y >= self.height as u8 {
            return 0;
        }
        if (self.rows[y as usize] >> (self.width - 1 - x)) & 1 != 0 {
            255
        } else {
            0
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphBitmap {
    pub width: u8,
    pub height: u8,
    pub advance: u8,
    pub bearing_x: i8,
    pub bearing_y: i8,
    pub offset: usize,
    pub len: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphView {
    pub width: u8,
    pub height: u8,
    pub advance: u8,
    pub bearing_x: i8,
    pub bearing_y: i8,
    pub data: *const u8,
    pub len: usize,
}

impl GlyphView {
    pub fn alpha_at(self, x: u8, y: u8) -> u8 {
        if self.data.is_null() || x >= self.width || y >= self.height {
            return 0;
        }
        let offset = y as usize * self.width as usize + x as usize;
        if offset >= self.len {
            0
        } else {
            unsafe { *self.data.add(offset) }
        }
    }
}

pub type GlyphResolver = fn(FontId, u32, u16) -> Option<GlyphView>;
pub type GlyphIdResolver = fn(FontId, u16, u16) -> Option<GlyphView>;
pub type KerningResolver = fn(FontId, u32, u32, u16) -> i16;
pub type GlyphRunResolver =
    fn(FontId, &str, u16, ShapeOptions) -> Option<GlyphRun<GLYPH_RUN_RESOLVER_CAPACITY>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphRunCacheStats {
    pub hits: u32,
    pub misses: u32,
    pub inserts: u32,
    pub evictions: u32,
    pub overflows: u32,
    pub slots: usize,
}

impl GlyphRunCacheStats {
    pub const fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            inserts: 0,
            evictions: 0,
            overflows: 0,
            slots: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GlyphRunCacheEntry {
    font: FontId,
    size: u16,
    options: ShapeOptions,
    text_hash: u32,
    text_len: u16,
    last_used: u32,
    valid: bool,
    run: GlyphRun<GLYPH_RUN_RESOLVER_CAPACITY>,
}

impl GlyphRunCacheEntry {
    const EMPTY: Self = Self {
        font: FontId(0),
        size: 0,
        options: ShapeOptions::FAST,
        text_hash: 0,
        text_len: 0,
        last_used: 0,
        valid: false,
        run: GlyphRun::new(),
    };

    fn matches(self, font: FontId, text: &str, size: u16, options: ShapeOptions) -> bool {
        self.valid
            && self.font == font
            && self.size == size
            && self.options == options
            && self.text_hash == glyph_run_text_hash(text)
            && self.text_len == text.len().min(u16::MAX as usize) as u16
    }
}

pub struct GlyphRunCache<const RUNS: usize> {
    entries: [GlyphRunCacheEntry; RUNS],
    clock: u32,
    stats: GlyphRunCacheStats,
}

impl<const RUNS: usize> GlyphRunCache<RUNS> {
    pub const fn new() -> Self {
        Self {
            entries: [GlyphRunCacheEntry::EMPTY; RUNS],
            clock: 0,
            stats: GlyphRunCacheStats::new(),
        }
    }

    pub fn clear(&mut self) {
        let mut index = 0usize;
        while index < RUNS {
            self.entries[index] = GlyphRunCacheEntry::EMPTY;
            index += 1;
        }
        self.clock = 0;
        self.stats = GlyphRunCacheStats::new();
    }

    pub fn stats(&self) -> GlyphRunCacheStats {
        let mut stats = self.stats;
        stats.slots = RUNS;
        stats
    }

    pub fn lookup(
        &mut self,
        font: FontId,
        text: &str,
        size: u16,
        options: ShapeOptions,
    ) -> Option<GlyphRun<GLYPH_RUN_RESOLVER_CAPACITY>> {
        self.clock = self.clock.wrapping_add(1).max(1);
        let mut index = 0usize;
        while index < RUNS {
            if self.entries[index].matches(font, text, size, options) {
                self.entries[index].last_used = self.clock;
                self.stats.hits = self.stats.hits.saturating_add(1);
                return Some(self.entries[index].run);
            }
            index += 1;
        }
        self.stats.misses = self.stats.misses.saturating_add(1);
        None
    }

    pub fn prewarm(
        &mut self,
        font: FontId,
        text: &str,
        size: u16,
        options: ShapeOptions,
        resolver: GlyphRunResolver,
    ) -> bool {
        if self.lookup(font, text, size, options).is_some() {
            return true;
        }
        let Some(run) = resolver(font, text, size, options) else {
            self.stats.overflows = self.stats.overflows.saturating_add(1);
            return false;
        };
        self.insert(font, text, size, options, run)
    }

    pub fn insert(
        &mut self,
        font: FontId,
        text: &str,
        size: u16,
        options: ShapeOptions,
        run: GlyphRun<GLYPH_RUN_RESOLVER_CAPACITY>,
    ) -> bool {
        if RUNS == 0 {
            self.stats.overflows = self.stats.overflows.saturating_add(1);
            return false;
        }

        self.clock = self.clock.wrapping_add(1).max(1);
        let mut target = 0usize;
        let mut oldest_used = u32::MAX;
        let mut found_free = false;
        let mut index = 0usize;
        while index < RUNS {
            if !self.entries[index].valid {
                target = index;
                found_free = true;
                break;
            }
            if self.entries[index].last_used < oldest_used {
                oldest_used = self.entries[index].last_used;
                target = index;
            }
            index += 1;
        }

        if !found_free {
            self.stats.evictions = self.stats.evictions.saturating_add(1);
        }

        self.entries[target] = GlyphRunCacheEntry {
            font,
            size,
            options,
            text_hash: glyph_run_text_hash(text),
            text_len: text.len().min(u16::MAX as usize) as u16,
            last_used: self.clock,
            valid: true,
            run,
        };
        self.stats.inserts = self.stats.inserts.saturating_add(1);
        true
    }
}

fn glyph_run_text_hash(text: &str) -> u32 {
    let mut hash = 2_166_136_261u32;
    for byte in text.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphCacheEntry {
    pub codepoint: u32,
    pub bitmap: GlyphBitmap,
    pub used: bool,
}

impl GlyphCacheEntry {
    pub const EMPTY: Self = Self {
        codepoint: 0,
        bitmap: GlyphBitmap {
            width: 0,
            height: 0,
            advance: 0,
            bearing_x: 0,
            bearing_y: 0,
            offset: 0,
            len: 0,
        },
        used: false,
    };
}

pub struct GlyphCache<const GLYPHS: usize, const BYTES: usize> {
    entries: [GlyphCacheEntry; GLYPHS],
    bytes: [u8; BYTES],
    len: usize,
    next_byte: usize,
    overflowed: bool,
}

impl<const GLYPHS: usize, const BYTES: usize> GlyphCache<GLYPHS, BYTES> {
    pub const fn new() -> Self {
        Self {
            entries: [GlyphCacheEntry::EMPTY; GLYPHS],
            bytes: [0; BYTES],
            len: 0,
            next_byte: 0,
            overflowed: false,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.next_byte = 0;
        self.overflowed = false;
    }

    pub fn lookup(&self, codepoint: u32) -> Option<GlyphBitmap> {
        let mut i = 0;
        while i < self.len {
            let entry = self.entries[i];
            if entry.used && entry.codepoint == codepoint {
                return Some(entry.bitmap);
            }
            i += 1;
        }
        None
    }

    pub fn lookup_or_insert_debug(&mut self, byte: u8) -> Option<GlyphBitmap> {
        let codepoint = byte as u32;
        if let Some(bitmap) = self.lookup(codepoint) {
            return Some(bitmap);
        }
        if self.len >= GLYPHS || self.next_byte.saturating_add(35) > BYTES {
            self.overflowed = true;
            return None;
        }

        let glyph = glyph_5x7_a8(byte);
        let offset = self.next_byte;
        let mut dst = offset;
        let mut y = 0;
        while y < glyph.height {
            let mut x = 0;
            while x < glyph.width {
                self.bytes[dst] = glyph.alpha_at(x, y);
                dst += 1;
                x += 1;
            }
            y += 1;
        }

        let bitmap = GlyphBitmap {
            width: glyph.width,
            height: glyph.height,
            advance: glyph.advance,
            bearing_x: 0,
            bearing_y: glyph.height as i8,
            offset,
            len: 35,
        };
        self.entries[self.len] = GlyphCacheEntry {
            codepoint,
            bitmap,
            used: true,
        };
        self.len += 1;
        self.next_byte = dst;
        Some(bitmap)
    }

    pub fn insert_a8(
        &mut self,
        codepoint: u32,
        width: u8,
        height: u8,
        advance: u8,
        data: &[u8],
    ) -> Option<GlyphBitmap> {
        self.insert_a8_metrics(codepoint, width, height, advance, 0, height as i8, data)
    }

    pub fn insert_a8_metrics(
        &mut self,
        codepoint: u32,
        width: u8,
        height: u8,
        advance: u8,
        bearing_x: i8,
        bearing_y: i8,
        data: &[u8],
    ) -> Option<GlyphBitmap> {
        if let Some(bitmap) = self.lookup(codepoint) {
            return Some(bitmap);
        }
        let len = width as usize * height as usize;
        if len == 0
            || data.len() < len
            || self.len >= GLYPHS
            || self.next_byte.saturating_add(len) > BYTES
        {
            self.overflowed = true;
            return None;
        }

        let offset = self.next_byte;
        let end = offset + len;
        self.bytes[offset..end].copy_from_slice(&data[..len]);
        let bitmap = GlyphBitmap {
            width,
            height,
            advance,
            bearing_x,
            bearing_y,
            offset,
            len,
        };
        self.entries[self.len] = GlyphCacheEntry {
            codepoint,
            bitmap,
            used: true,
        };
        self.len += 1;
        self.next_byte = end;
        Some(bitmap)
    }

    pub fn bitmap_bytes(&self, bitmap: GlyphBitmap) -> Option<&[u8]> {
        let end = bitmap.offset.checked_add(bitmap.len)?;
        self.bytes.get(bitmap.offset..end)
    }

    pub fn view(&self, codepoint: u32) -> Option<GlyphView> {
        let bitmap = self.lookup(codepoint)?;
        let bytes = self.bitmap_bytes(bitmap)?;
        Some(GlyphView {
            width: bitmap.width,
            height: bitmap.height,
            advance: bitmap.advance,
            bearing_x: bitmap.bearing_x,
            bearing_y: bitmap.bearing_y,
            data: bytes.as_ptr(),
            len: bytes.len(),
        })
    }
}

pub fn glyph_5x7_a8(byte: u8) -> GlyphA8 {
    GlyphA8::new(5, 7, 6, glyph_5x7(byte))
}
