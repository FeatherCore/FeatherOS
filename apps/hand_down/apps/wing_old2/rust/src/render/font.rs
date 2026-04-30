use alloc::vec::Vec;
use core::cell::RefCell;

use crate::math::{Color, Rect};

pub const GLYPH_CACHE_TINY_CAPACITY: usize = 32;
pub const GLYPH_CACHE_BALANCED_CAPACITY: usize = 96;
pub const GLYPH_CACHE_LARGE_CAPACITY: usize = 192;
pub const GLYPH_CACHE_MIN_CAPACITY: usize = 8;
pub const GLYPH_CACHE_MAX_CAPACITY: usize = 256;
pub const GLYPH_CACHE_CAPACITY: usize = GLYPH_CACHE_BALANCED_CAPACITY;

const LATIN_BITMAP_WIDTH: u16 = 5;
const WIDE_BITMAP_WIDTH: u16 = 7;
const GLYPH_BITMAP_HEIGHT: u16 = 7;
const LATIN_ADVANCE: i32 = 6;
const WIDE_ADVANCE: i32 = 8;
const LINE_HEIGHT: i32 = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontResourceSource {
    #[default]
    PlaceholderFallback,
    RuntimeTrueType,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlyphCacheProfile {
    Tiny,
    #[default]
    Balanced,
    Large,
    Custom,
}

impl GlyphCacheProfile {
    pub const fn from_capacity(capacity: usize) -> Self {
        if capacity <= GLYPH_CACHE_TINY_CAPACITY {
            Self::Tiny
        } else if capacity <= GLYPH_CACHE_BALANCED_CAPACITY {
            Self::Balanced
        } else if capacity <= GLYPH_CACHE_LARGE_CAPACITY {
            Self::Large
        } else {
            Self::Custom
        }
    }
}

pub struct FontStore {
    default: Option<TrueTypeFont>,
    source: FontResourceSource,
    cache: RefCell<GlyphCache>,
}

impl Default for FontStore {
    fn default() -> Self {
        Self::with_cache_capacity(default_glyph_cache_capacity())
    }
}

impl FontStore {
    pub fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            default: None,
            source: FontResourceSource::PlaceholderFallback,
            cache: RefCell::new(GlyphCache::with_capacity(capacity)),
        }
    }

    pub fn register_default_ttf(&mut self, bytes: &'static [u8]) -> bool {
        let Some(font) = TrueTypeFont::parse(bytes) else {
            return false;
        };
        self.default = Some(font);
        self.source = FontResourceSource::RuntimeTrueType;
        self.cache.get_mut().clear();
        true
    }

    pub const fn source(&self) -> FontResourceSource {
        self.source
    }

    pub const fn has_default_ttf(&self) -> bool {
        matches!(self.source, FontResourceSource::RuntimeTrueType)
    }

    pub fn rasterize_cell(&self, ch: char, scale: u8) -> GlyphBitmap {
        let mut glyph = None;
        self.with_rasterized_cell(ch, scale, |cached| glyph = Some(cached.clone()));
        glyph.unwrap_or_else(|| self.build_cell(ch, scale))
    }

    pub fn with_rasterized_cell<F>(&self, ch: char, scale: u8, draw: F)
    where
        F: FnOnce(&GlyphBitmap),
    {
        let scale = scale.max(1);
        {
            let mut cache = self.cache.borrow_mut();
            if let Some(index) = cache.find(ch, scale, self.source) {
                cache.record_hit();
                cache.touch(index);
                draw(cache.glyph(index));
                return;
            }
            cache.record_miss();
        }

        let glyph = self.build_cell(ch, scale);
        let mut cache = self.cache.borrow_mut();
        let index = cache.insert(ch, scale, self.source, glyph);
        draw(cache.glyph(index));
    }

    pub fn cache_summary(&self) -> FontCacheSummary {
        self.cache.borrow().summary()
    }

    pub fn set_cache_capacity(&mut self, capacity: usize) {
        self.cache.get_mut().set_capacity(capacity);
    }

    pub fn cache_capacity(&self) -> usize {
        self.cache.borrow().capacity()
    }

    pub fn warmup_chars(&self, chars: &str, scale: u8) -> FontWarmupSummary {
        let scale = scale.max(1);
        let mut summary = FontWarmupSummary::default();

        for ch in chars.chars() {
            summary.requested = summary.requested.saturating_add(1);
            {
                let cache = self.cache.borrow();
                if cache.find(ch, scale, self.source).is_some() {
                    summary.present = summary.present.saturating_add(1);
                    continue;
                }
                if cache.is_full() {
                    summary.skipped = summary.skipped.saturating_add(1);
                    continue;
                }
            }

            let glyph = self.build_cell(ch, scale);
            let bytes = glyph.data.len() as u32;
            let mut cache = self.cache.borrow_mut();
            if cache.insert_if_room(ch, scale, self.source, glyph) {
                summary.inserted = summary.inserted.saturating_add(1);
                summary.bytes = summary.bytes.saturating_add(bytes);
            } else {
                summary.skipped = summary.skipped.saturating_add(1);
            }
        }

        summary
    }

    fn build_cell(&self, ch: char, scale: u8) -> GlyphBitmap {
        if let Some(font) = self.default.as_ref() {
            if let Some(glyph) = font.rasterize_cell(ch, scale) {
                return glyph;
            }
        }

        placeholder_cell(ch, scale)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FontCacheSummary {
    pub entries: u16,
    pub capacity: u16,
    pub profile: GlyphCacheProfile,
    pub bytes: u32,
    pub hits: u32,
    pub misses: u32,
    pub evictions: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FontWarmupSummary {
    pub requested: u16,
    pub inserted: u16,
    pub present: u16,
    pub skipped: u16,
    pub bytes: u32,
}

impl FontWarmupSummary {
    pub const fn empty() -> Self {
        Self {
            requested: 0,
            inserted: 0,
            present: 0,
            skipped: 0,
            bytes: 0,
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.requested = self.requested.saturating_add(other.requested);
        self.inserted = self.inserted.saturating_add(other.inserted);
        self.present = self.present.saturating_add(other.present);
        self.skipped = self.skipped.saturating_add(other.skipped);
        self.bytes = self.bytes.saturating_add(other.bytes);
    }
}

impl FontCacheSummary {
    pub fn requests(self) -> u32 {
        self.hits.saturating_add(self.misses)
    }

    pub fn hit_rate_percent(self) -> u8 {
        let requests = self.requests();
        if requests == 0 {
            0
        } else {
            ((self.hits.saturating_mul(100) / requests).min(100)) as u8
        }
    }

    pub fn fill_percent(self) -> u8 {
        if self.capacity == 0 {
            0
        } else {
            (((self.entries as u32).saturating_mul(100) / self.capacity as u32).min(100)) as u8
        }
    }

    pub fn pressure(self) -> GlyphCachePressure {
        if self.requests() == 0 {
            GlyphCachePressure::Cold
        } else if self.evictions != 0 && self.hit_rate_percent() < 60 {
            GlyphCachePressure::Thrashing
        } else if self.evictions != 0 || self.fill_percent() >= 90 {
            GlyphCachePressure::Tight
        } else {
            GlyphCachePressure::Healthy
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlyphCachePressure {
    #[default]
    Cold,
    Healthy,
    Tight,
    Thrashing,
}

struct GlyphCache {
    entries: Vec<CachedGlyph>,
    capacity: usize,
    clock: u32,
    hits: u32,
    misses: u32,
    evictions: u32,
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::with_capacity(default_glyph_cache_capacity())
    }
}

impl GlyphCache {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity: sanitize_glyph_cache_capacity(capacity),
            clock: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.clock = 0;
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
    }

    fn set_capacity(&mut self, capacity: usize) {
        self.capacity = sanitize_glyph_cache_capacity(capacity);
        while self.entries.len() > self.capacity {
            let index = self.least_recent_index();
            self.entries.swap_remove(index);
            self.evictions = self.evictions.saturating_add(1);
        }
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn is_full(&self) -> bool {
        self.entries.len() >= self.capacity
    }

    fn find(&self, ch: char, scale: u8, source: FontResourceSource) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| entry.ch == ch && entry.scale == scale && entry.source == source)
    }

    fn glyph(&self, index: usize) -> &GlyphBitmap {
        &self.entries[index].glyph
    }

    fn insert(
        &mut self,
        ch: char,
        scale: u8,
        source: FontResourceSource,
        glyph: GlyphBitmap,
    ) -> usize {
        let age = self.next_age();
        let entry = CachedGlyph {
            ch,
            scale,
            source,
            age,
            glyph,
        };

        if self.entries.len() < self.capacity {
            self.entries.push(entry);
            return self.entries.len().saturating_sub(1);
        }

        let index = self.least_recent_index();
        self.entries[index] = entry;
        self.evictions = self.evictions.saturating_add(1);
        index
    }

    fn insert_if_room(
        &mut self,
        ch: char,
        scale: u8,
        source: FontResourceSource,
        glyph: GlyphBitmap,
    ) -> bool {
        if self.is_full() {
            return false;
        }

        let age = self.next_age();
        self.entries.push(CachedGlyph {
            ch,
            scale,
            source,
            age,
            glyph,
        });
        true
    }

    fn touch(&mut self, index: usize) {
        let age = self.next_age();
        if let Some(entry) = self.entries.get_mut(index) {
            entry.age = age;
        }
    }

    fn record_hit(&mut self) {
        self.hits = self.hits.saturating_add(1);
    }

    fn record_miss(&mut self) {
        self.misses = self.misses.saturating_add(1);
    }

    fn summary(&self) -> FontCacheSummary {
        FontCacheSummary {
            entries: self.entries.len().min(u16::MAX as usize) as u16,
            capacity: self.capacity.min(u16::MAX as usize) as u16,
            profile: GlyphCacheProfile::from_capacity(self.capacity),
            bytes: self.glyph_bytes(),
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
        }
    }

    fn glyph_bytes(&self) -> u32 {
        self.entries
            .iter()
            .fold(0u32, |bytes, entry| bytes.saturating_add(entry.glyph.data.len() as u32))
    }

    fn least_recent_index(&self) -> usize {
        let mut best = 0usize;
        let mut best_age = u32::MAX;
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.age < best_age {
                best = index;
                best_age = entry.age;
            }
        }
        best
    }

    fn next_age(&mut self) -> u32 {
        self.clock = self.clock.saturating_add(1);
        self.clock
    }
}

pub fn default_glyph_cache_capacity() -> usize {
    if let Some(raw) = option_env!("WING_GLYPH_CACHE_CAPACITY") {
        if let Some(capacity) = parse_cache_capacity(raw) {
            return sanitize_glyph_cache_capacity(capacity);
        }
    }

    match option_env!("WING_GLYPH_CACHE_PROFILE") {
        Some("tiny") | Some("TINY") | Some("mcu") | Some("MCU") => GLYPH_CACHE_TINY_CAPACITY,
        Some("large") | Some("LARGE") | Some("mpu") | Some("MPU") => GLYPH_CACHE_LARGE_CAPACITY,
        Some("balanced") | Some("BALANCED") | Some("base") | Some("BASE") | None => {
            GLYPH_CACHE_BALANCED_CAPACITY
        }
        Some(_) => GLYPH_CACHE_BALANCED_CAPACITY,
    }
}

fn parse_cache_capacity(raw: &str) -> Option<usize> {
    let mut value = 0usize;
    let mut seen_digit = false;

    for byte in raw.as_bytes().iter().copied() {
        if byte == b' ' || byte == b'\t' || byte == b'\n' || byte == b'\r' {
            continue;
        }
        if !byte.is_ascii_digit() {
            return None;
        }
        seen_digit = true;
        value = value
            .saturating_mul(10)
            .saturating_add((byte - b'0') as usize);
    }

    if seen_digit {
        Some(value)
    } else {
        None
    }
}

fn sanitize_glyph_cache_capacity(capacity: usize) -> usize {
    capacity.clamp(GLYPH_CACHE_MIN_CAPACITY, GLYPH_CACHE_MAX_CAPACITY)
}

struct CachedGlyph {
    ch: char,
    scale: u8,
    source: FontResourceSource,
    age: u32,
    glyph: GlyphBitmap,
}

#[derive(Clone)]
pub struct GlyphBitmap {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

impl GlyphBitmap {
    pub fn alpha_at(&self, x: u16, y: u16, color: Color) -> Option<Color> {
        let alpha = *self.data.get(y as usize * self.width as usize + x as usize)?;
        if alpha == 0 {
            None
        } else {
            Some(color.with_alpha(((alpha as u16 * color.a as u16) / 255) as u8))
        }
    }
}

#[derive(Clone, Copy)]
struct TrueTypeFont {
    bytes: &'static [u8],
    cmap: usize,
    glyf: usize,
    loca: usize,
    cmap4: usize,
    index_to_loc_format: i16,
    num_glyphs: u16,
}

impl TrueTypeFont {
    fn parse(bytes: &'static [u8]) -> Option<Self> {
        if bytes.len() < 12 || read_be_u32(bytes, 0)? != 0x0001_0000 {
            return None;
        }

        let table_count = read_be_u16(bytes, 4)? as usize;
        let head = table_offset(bytes, table_count, b"head")?;
        let maxp = table_offset(bytes, table_count, b"maxp")?;
        let cmap = table_offset(bytes, table_count, b"cmap")?;
        let loca = table_offset(bytes, table_count, b"loca")?;
        let glyf = table_offset(bytes, table_count, b"glyf")?;
        let cmap4 = find_cmap4(bytes, cmap)?;

        Some(Self {
            bytes,
            cmap,
            glyf,
            loca,
            cmap4,
            index_to_loc_format: read_be_i16(bytes, head + 50)?,
            num_glyphs: read_be_u16(bytes, maxp + 4)?,
        })
    }

    fn rasterize_cell(&self, ch: char, scale: u8) -> Option<GlyphBitmap> {
        if ch == ' ' {
            return Some(blank_cell(scale));
        }

        let glyph = self.glyph_id(ch as u32)?;
        if glyph == 0 || glyph >= self.num_glyphs {
            return None;
        }

        let outline = self.simple_outline(glyph)?;
        if outline.points.is_empty() {
            return Some(blank_cell(scale));
        }

        let metrics = cell_metrics(ch, scale);
        let width = metrics.bitmap_width;
        let height = metrics.bitmap_height;
        let bounds_w = (outline.x_max as i32 - outline.x_min as i32).max(1) as f32;
        let bounds_h = (outline.y_max as i32 - outline.y_min as i32).max(1) as f32;
        let target_w = width.max(1) as f32;
        let target_h = height.max(1) as f32;
        let outline_scale = (target_w / bounds_w).min(target_h / bounds_h);
        let draw_w = bounds_w * outline_scale;
        let draw_h = bounds_h * outline_scale;
        let offset_x = (target_w - draw_w) * 0.5 - outline.x_min as f32 * outline_scale;
        let offset_y = (target_h - draw_h) * 0.5 + outline.y_max as f32 * outline_scale;

        let mut segments = Vec::new();
        outline.flatten(width, height, outline_scale, offset_x, offset_y, &mut segments);
        if segments.is_empty() {
            return None;
        }

        let mut data = Vec::new();
        data.resize(width as usize * height as usize, 0);
        for y in 0..height {
            for x in 0..width {
                let mut coverage = 0u8;
                for (sx, sy) in [(0.25f32, 0.25f32), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                    if point_inside_segments(x as f32 + sx, y as f32 + sy, &segments) {
                        coverage += 1;
                    }
                }
                data[y as usize * width as usize + x as usize] =
                    ((coverage as u16 * 255) / 4) as u8;
            }
        }

        Some(GlyphBitmap { width, height, data })
    }

    fn glyph_id(&self, codepoint: u32) -> Option<u16> {
        if codepoint > u16::MAX as u32 {
            return None;
        }
        let cp = codepoint as u16;
        let table = self.cmap + self.cmap4;
        let seg_count = read_be_u16(self.bytes, table + 6)? as usize / 2;
        let end_codes = table + 14;
        let start_codes = end_codes + seg_count * 2 + 2;
        let id_deltas = start_codes + seg_count * 2;
        let id_range_offsets = id_deltas + seg_count * 2;

        for index in 0..seg_count {
            let end = read_be_u16(self.bytes, end_codes + index * 2)?;
            let start = read_be_u16(self.bytes, start_codes + index * 2)?;
            if cp < start || cp > end {
                continue;
            }

            let delta = read_be_i16(self.bytes, id_deltas + index * 2)? as i32;
            let range_offset_pos = id_range_offsets + index * 2;
            let range_offset = read_be_u16(self.bytes, range_offset_pos)?;
            if range_offset == 0 {
                return Some(((cp as i32 + delta) & 0xffff) as u16);
            }

            let glyph_pos = range_offset_pos
                .checked_add(range_offset as usize)?
                .checked_add((cp - start) as usize * 2)?;
            let glyph = read_be_u16(self.bytes, glyph_pos)?;
            if glyph == 0 {
                return Some(0);
            }
            return Some(((glyph as i32 + delta) & 0xffff) as u16);
        }

        None
    }

    fn glyph_bounds(&self, glyph: u16) -> Option<(usize, usize)> {
        let gid = glyph as usize;
        if gid >= self.num_glyphs as usize {
            return None;
        }

        let (start, end) = if self.index_to_loc_format == 0 {
            (
                read_be_u16(self.bytes, self.loca + gid * 2)? as usize * 2,
                read_be_u16(self.bytes, self.loca + (gid + 1) * 2)? as usize * 2,
            )
        } else {
            (
                read_be_u32(self.bytes, self.loca + gid * 4)? as usize,
                read_be_u32(self.bytes, self.loca + (gid + 1) * 4)? as usize,
            )
        };

        if end <= start {
            None
        } else {
            Some((self.glyf + start, self.glyf + end))
        }
    }

    fn simple_outline(&self, glyph: u16) -> Option<GlyphOutline> {
        let (start, end) = self.glyph_bounds(glyph)?;
        if end.checked_sub(start)? < 10 {
            return None;
        }
        let contour_count = read_be_i16(self.bytes, start)?;
        if contour_count < 0 {
            return None;
        }
        let x_min = read_be_i16(self.bytes, start + 2)?;
        let y_min = read_be_i16(self.bytes, start + 4)?;
        let x_max = read_be_i16(self.bytes, start + 6)?;
        let y_max = read_be_i16(self.bytes, start + 8)?;

        let contour_count = contour_count as usize;
        if contour_count == 0 {
            return Some(GlyphOutline {
                points: Vec::new(),
                ends: Vec::new(),
                x_min,
                y_min,
                x_max,
                y_max,
            });
        }

        let mut ends = Vec::new();
        let mut point_count = 0usize;
        for index in 0..contour_count {
            let end_point = read_be_u16(self.bytes, start + 10 + index * 2)? as usize;
            point_count = end_point + 1;
            ends.push(end_point);
        }

        let instruction_len = read_be_u16(self.bytes, start + 10 + contour_count * 2)? as usize;
        let mut cursor = start + 10 + contour_count * 2 + 2 + instruction_len;
        if cursor >= end {
            return None;
        }

        let mut flags = Vec::new();
        while flags.len() < point_count {
            let flag = *self.bytes.get(cursor)?;
            cursor += 1;
            flags.push(flag);
            if flag & 0x08 != 0 {
                let repeat = *self.bytes.get(cursor)? as usize;
                cursor += 1;
                for _ in 0..repeat {
                    flags.push(flag);
                }
            }
        }

        let xs = decode_coords(self.bytes, &flags, &mut cursor, true)?;
        let ys = decode_coords(self.bytes, &flags, &mut cursor, false)?;
        let mut points = Vec::new();
        for index in 0..point_count {
            points.push(OutlinePoint {
                x: xs[index],
                y: ys[index],
                on: flags[index] & 0x01 != 0,
            });
        }

        Some(GlyphOutline {
            points,
            ends,
            x_min,
            y_min,
            x_max,
            y_max,
        })
    }
}

struct GlyphOutline {
    points: Vec<OutlinePoint>,
    ends: Vec<usize>,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
}

impl GlyphOutline {
    fn flatten(
        &self,
        width: u16,
        height: u16,
        outline_scale: f32,
        offset_x: f32,
        offset_y: f32,
        out: &mut Vec<Segment>,
    ) {
        let mut start = 0usize;
        for &end in &self.ends {
            if end >= self.points.len() || start > end {
                return;
            }
            flatten_contour(
                &self.points[start..=end],
                width,
                height,
                outline_scale,
                offset_x,
                offset_y,
                out,
            );
            start = end + 1;
        }
    }
}

#[derive(Clone, Copy)]
struct OutlinePoint {
    x: i16,
    y: i16,
    on: bool,
}

#[derive(Clone, Copy)]
struct Segment {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

fn flatten_contour(
    points: &[OutlinePoint],
    width: u16,
    height: u16,
    outline_scale: f32,
    offset_x: f32,
    offset_y: f32,
    out: &mut Vec<Segment>,
) {
    if points.is_empty() {
        return;
    }

    let first = points[0];
    let last = points[points.len() - 1];
    let start = if first.on {
        first
    } else if last.on {
        last
    } else {
        midpoint(first, last)
    };
    let mut current = transform_point(start, width, height, outline_scale, offset_x, offset_y);

    let mut index = if first.on { 1usize } else { 0usize };
    let limit = points.len() + if first.on { 0 } else { 1 };
    while index < limit {
        let point = points[index % points.len()];
        if point.on {
            let next = transform_point(point, width, height, outline_scale, offset_x, offset_y);
            push_segment(out, current, next);
            current = next;
            index += 1;
        } else {
            let raw_next = points[(index + 1) % points.len()];
            let end = if raw_next.on {
                raw_next
            } else {
                midpoint(point, raw_next)
            };
            let control = transform_point(point, width, height, outline_scale, offset_x, offset_y);
            let next = transform_point(end, width, height, outline_scale, offset_x, offset_y);
            flatten_quad(current, control, next, out);
            current = next;
            index += if raw_next.on { 2 } else { 1 };
        }
    }

    let start = transform_point(start, width, height, outline_scale, offset_x, offset_y);
    push_segment(out, current, start);
}

fn transform_point(
    point: OutlinePoint,
    _width: u16,
    _height: u16,
    outline_scale: f32,
    offset_x: f32,
    offset_y: f32,
) -> (f32, f32) {
    (
        point.x as f32 * outline_scale + offset_x,
        offset_y - point.y as f32 * outline_scale,
    )
}

fn midpoint(a: OutlinePoint, b: OutlinePoint) -> OutlinePoint {
    OutlinePoint {
        x: ((a.x as i32 + b.x as i32) / 2) as i16,
        y: ((a.y as i32 + b.y as i32) / 2) as i16,
        on: true,
    }
}

fn flatten_quad(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), out: &mut Vec<Segment>) {
    let mut previous = p0;
    for step in 1..=6 {
        let t = step as f32 / 6.0;
        let mt = 1.0 - t;
        let next = (
            mt * mt * p0.0 + 2.0 * mt * t * p1.0 + t * t * p2.0,
            mt * mt * p0.1 + 2.0 * mt * t * p1.1 + t * t * p2.1,
        );
        push_segment(out, previous, next);
        previous = next;
    }
}

fn push_segment(out: &mut Vec<Segment>, a: (f32, f32), b: (f32, f32)) {
    if (a.0 - b.0).abs() < 0.001 && (a.1 - b.1).abs() < 0.001 {
        return;
    }
    out.push(Segment {
        x0: a.0,
        y0: a.1,
        x1: b.0,
        y1: b.1,
    });
}

fn point_inside_segments(x: f32, y: f32, segments: &[Segment]) -> bool {
    let mut inside = false;
    for segment in segments {
        let y0 = segment.y0;
        let y1 = segment.y1;
        if (y0 > y) == (y1 > y) {
            continue;
        }
        let t = (y - y0) / (y1 - y0);
        let hit_x = segment.x0 + t * (segment.x1 - segment.x0);
        if hit_x > x {
            inside = !inside;
        }
    }
    inside
}

fn blank_cell(scale: u8) -> GlyphBitmap {
    let metrics = cell_metrics(' ', scale);
    let width = metrics.bitmap_width;
    let height = metrics.bitmap_height;
    let mut data = Vec::new();
    data.resize(width as usize * height as usize, 0);
    GlyphBitmap { width, height, data }
}

pub fn font_text_bounds(x: i32, y: i32, text: &'static str, scale: u8) -> Rect {
    let scale = scale.max(1);
    let mut line_width = 0i32;
    let mut max_width = 0i32;
    let mut lines = 1i32;

    for ch in text.chars() {
        if ch == '\n' {
            max_width = max_width.max(line_width);
            line_width = 0;
            lines += 1;
        } else {
            line_width += cell_metrics(ch, scale).advance;
        }
    }
    max_width = max_width.max(line_width);

    Rect::new(
        x,
        y,
        max_width.clamp(0, u16::MAX as i32) as u16,
        (lines * line_height(scale)).clamp(0, u16::MAX as i32) as u16,
    )
}

pub fn font_cell_advance(ch: char, scale: u8) -> i32 {
    cell_metrics(ch, scale).advance
}

pub fn font_line_height(scale: u8) -> i32 {
    line_height(scale)
}

#[derive(Clone, Copy)]
struct CellMetrics {
    bitmap_width: u16,
    bitmap_height: u16,
    advance: i32,
}

fn cell_metrics(ch: char, scale: u8) -> CellMetrics {
    let scale = scale.max(1);
    let wide = is_wide_char(ch);
    let bitmap_width = if wide {
        WIDE_BITMAP_WIDTH
    } else {
        LATIN_BITMAP_WIDTH
    }
    .saturating_mul(scale as u16)
    .max(1);
    let bitmap_height = GLYPH_BITMAP_HEIGHT
        .saturating_mul(scale as u16)
        .max(1);
    let advance = if wide { WIDE_ADVANCE } else { LATIN_ADVANCE } * scale as i32;

    CellMetrics {
        bitmap_width,
        bitmap_height,
        advance,
    }
}

fn line_height(scale: u8) -> i32 {
    LINE_HEIGHT * scale.max(1) as i32
}

fn is_wide_char(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x11ff
            | 0x2e80..=0xa4cf
            | 0xac00..=0xd7a3
            | 0xf900..=0xfaff
            | 0xfe10..=0xfe6f
            | 0xff01..=0xff60
            | 0xffe0..=0xffe6
            | 0x1f300..=0x1faff
            | 0x20000..=0x3fffd
    )
}

fn placeholder_cell(ch: char, scale: u8) -> GlyphBitmap {
    if ch == ' ' {
        return blank_cell(scale);
    }

    let metrics = cell_metrics(ch, scale);
    let width = metrics.bitmap_width;
    let height = metrics.bitmap_height;
    let mut data = Vec::new();
    data.resize(width as usize * height as usize, 0);

    for y in 0..height {
        for x in 0..width {
            let border = x == 0 || y == 0 || x + 1 == width || y + 1 == height;
            let slash = x as u32 * height as u32 / width.max(1) as u32 == y as u32;
            let backslash =
                ((width - 1 - x) as u32 * height as u32 / width.max(1) as u32) == y as u32;
            if border || slash || backslash {
                data[y as usize * width as usize + x as usize] = 220;
            }
        }
    }

    GlyphBitmap { width, height, data }
}

fn decode_coords(bytes: &[u8], flags: &[u8], cursor: &mut usize, x_axis: bool) -> Option<Vec<i16>> {
    let short_bit = if x_axis { 0x02 } else { 0x04 };
    let same_bit = if x_axis { 0x10 } else { 0x20 };
    let mut coords = Vec::new();
    coords.resize(flags.len(), 0);
    let mut value = 0i16;

    for (index, flag) in flags.iter().copied().enumerate() {
        let delta = if flag & short_bit != 0 {
            let amount = *bytes.get(*cursor)? as i16;
            *cursor += 1;
            if flag & same_bit != 0 {
                amount
            } else {
                -amount
            }
        } else if flag & same_bit != 0 {
            0
        } else {
            let amount = read_be_i16(bytes, *cursor)?;
            *cursor += 2;
            amount
        };
        value = value.wrapping_add(delta);
        coords[index] = value;
    }

    Some(coords)
}

fn table_offset(bytes: &[u8], table_count: usize, tag: &[u8; 4]) -> Option<usize> {
    for index in 0..table_count {
        let entry = 12usize.checked_add(index.checked_mul(16)?)?;
        if bytes.get(entry..entry + 4)? == tag {
            return Some(read_be_u32(bytes, entry + 8)? as usize);
        }
    }
    None
}

fn find_cmap4(bytes: &[u8], cmap: usize) -> Option<usize> {
    let subtables = read_be_u16(bytes, cmap + 2)? as usize;
    let mut fallback = None;
    for index in 0..subtables {
        let entry = cmap + 4 + index * 8;
        let platform = read_be_u16(bytes, entry)?;
        let encoding = read_be_u16(bytes, entry + 2)?;
        let offset = read_be_u32(bytes, entry + 4)? as usize;
        if read_be_u16(bytes, cmap + offset)? != 4 {
            continue;
        }
        if platform == 3 && encoding == 1 {
            return Some(offset);
        }
        fallback = Some(offset);
    }
    fallback
}

fn read_be_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let slice = bytes.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_be_bytes([slice[0], slice[1]]))
}

fn read_be_i16(bytes: &[u8], offset: usize) -> Option<i16> {
    let slice = bytes.get(offset..offset.checked_add(2)?)?;
    Some(i16::from_be_bytes([slice[0], slice[1]]))
}

fn read_be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}
