use alloc::{vec, vec::Vec};
use core::cell::RefCell;
use core::ops::Add;

pub const SVG_STORE_CAPACITY: usize = 32;
pub const SVG_CACHE_TINY_CAPACITY: usize = 16;
pub const SVG_CACHE_BALANCED_CAPACITY: usize = 40;
pub const SVG_CACHE_LARGE_CAPACITY: usize = 64;
pub const SVG_CACHE_MIN_CAPACITY: usize = 4;
pub const SVG_CACHE_MAX_CAPACITY: usize = 96;
pub const SVG_RASTER_MAX_SIDE: u16 = 192;

const SVG_COORD_SCALE: i32 = 64;
const SVG_COORD_EXTENT: i32 = 16 * SVG_COORD_SCALE;
const SVG_AA_SCALE: i32 = 4;
const SVG_AA_SAMPLE_OFFSETS: [i32; 2] = [1, 3];
const SVG_AA_SAMPLE_COUNT: u8 = 4;
const CUBIC_SEGMENTS: usize = 12;
const QUAD_SEGMENTS: usize = 10;
const ARC_MAX_RADIANS_PER_SEGMENT: f32 = PI / 8.0;
const PI: f32 = 3.1415927;
const HALF_PI: f32 = PI * 0.5;
const TWO_PI: f32 = PI * 2.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SvgImageId(pub u16);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SvgResourceSource {
    #[default]
    Empty,
    RuntimeSvg,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SvgCacheProfile {
    Tiny,
    #[default]
    Balanced,
    Large,
    Custom,
}

impl SvgCacheProfile {
    pub const fn from_capacity(capacity: usize) -> Self {
        if capacity <= SVG_CACHE_TINY_CAPACITY {
            Self::Tiny
        } else if capacity <= SVG_CACHE_BALANCED_CAPACITY {
            Self::Balanced
        } else if capacity <= SVG_CACHE_LARGE_CAPACITY {
            Self::Large
        } else {
            Self::Custom
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SvgCacheSummary {
    pub entries: u16,
    pub capacity: u16,
    pub profile: SvgCacheProfile,
    pub bytes: u32,
    pub hits: u32,
    pub misses: u32,
    pub evictions: u32,
}

impl SvgCacheSummary {
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

    pub fn pressure(self) -> SvgCachePressure {
        if self.requests() == 0 {
            SvgCachePressure::Cold
        } else if self.evictions != 0 && self.hit_rate_percent() < 60 {
            SvgCachePressure::Thrashing
        } else if self.evictions != 0 || self.fill_percent() >= 90 {
            SvgCachePressure::Tight
        } else {
            SvgCachePressure::Healthy
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SvgCachePressure {
    #[default]
    Cold,
    Healthy,
    Tight,
    Thrashing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SvgWarmupSummary {
    pub requested: u16,
    pub inserted: u16,
    pub present: u16,
    pub skipped: u16,
    pub bytes: u32,
}

impl SvgWarmupSummary {
    pub const fn empty() -> Self {
        Self {
            requested: 0,
            inserted: 0,
            present: 0,
            skipped: 0,
            bytes: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SvgPoint {
    x: i16,
    y: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SvgContour {
    start: u16,
    len: u16,
}

struct SvgImage {
    id: SvgImageId,
    contours: Vec<SvgContour>,
    points: Vec<SvgPoint>,
}

pub struct SvgStore {
    images: Vec<SvgImage>,
    source: SvgResourceSource,
    cache: RefCell<SvgRasterCache>,
}

impl Default for SvgStore {
    fn default() -> Self {
        Self::with_cache_capacity(default_svg_cache_capacity())
    }
}

impl SvgStore {
    pub fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            images: Vec::new(),
            source: SvgResourceSource::Empty,
            cache: RefCell::new(SvgRasterCache::with_capacity(capacity)),
        }
    }

    pub fn register(&mut self, id: SvgImageId, bytes: &[u8]) -> bool {
        let Some(mut image) = parse_svg_image(bytes) else {
            return false;
        };
        image.id = id;

        for item in &mut self.images {
            if item.id == id {
                *item = image;
                self.source = SvgResourceSource::RuntimeSvg;
                self.cache.get_mut().clear_id(id);
                return true;
            }
        }

        if self.images.len() >= SVG_STORE_CAPACITY {
            return false;
        }

        self.images.push(image);
        self.source = SvgResourceSource::RuntimeSvg;
        self.cache.get_mut().clear_id(id);
        true
    }

    pub const fn source(&self) -> SvgResourceSource {
        self.source
    }

    pub fn contains(&self, id: SvgImageId) -> bool {
        self.image(id).is_some()
    }

    pub fn len(&self) -> usize {
        self.images.len()
    }

    pub const fn capacity(&self) -> usize {
        SVG_STORE_CAPACITY
    }

    pub fn cache_summary(&self) -> SvgCacheSummary {
        self.cache.borrow().summary()
    }

    pub fn set_cache_capacity(&mut self, capacity: usize) {
        self.cache.get_mut().set_capacity(capacity);
    }

    pub fn cache_capacity(&self) -> usize {
        self.cache.borrow().capacity()
    }

    pub fn with_rasterized<F>(&self, id: SvgImageId, width: u16, height: u16, draw: F) -> bool
    where
        F: FnOnce(&SvgRasterMask),
    {
        let width = sanitize_raster_side(width);
        let height = sanitize_raster_side(height);
        let source = self.source;

        {
            let mut cache = self.cache.borrow_mut();
            if let Some(index) = cache.find(id, width, height, source) {
                cache.record_hit();
                cache.touch(index);
                draw(cache.mask(index));
                return true;
            }
            cache.record_miss();
        }

        let Some(image) = self.image(id) else {
            return false;
        };

        let mask = rasterize_svg(image, width, height);
        let mut cache = self.cache.borrow_mut();
        let index = cache.insert(id, width, height, source, mask);
        draw(cache.mask(index));
        true
    }

    pub fn warmup(&self, ids: &[SvgImageId], size: u16) -> SvgWarmupSummary {
        let size = sanitize_raster_side(size);
        let mut summary = SvgWarmupSummary::empty();

        for id in ids.iter().copied() {
            summary.requested = summary.requested.saturating_add(1);
            if !self.contains(id) {
                summary.skipped = summary.skipped.saturating_add(1);
                continue;
            }

            {
                let cache = self.cache.borrow();
                if cache.find(id, size, size, self.source).is_some() {
                    summary.present = summary.present.saturating_add(1);
                    continue;
                }
                if cache.is_full() {
                    summary.skipped = summary.skipped.saturating_add(1);
                    continue;
                }
            }

            let Some(image) = self.image(id) else {
                summary.skipped = summary.skipped.saturating_add(1);
                continue;
            };
            let mask = rasterize_svg(image, size, size);
            let bytes = mask.data.len() as u32;
            let mut cache = self.cache.borrow_mut();
            if cache.insert_if_room(id, size, size, self.source, mask) {
                summary.inserted = summary.inserted.saturating_add(1);
                summary.bytes = summary.bytes.saturating_add(bytes);
            } else {
                summary.skipped = summary.skipped.saturating_add(1);
            }
        }

        summary
    }

    fn image(&self, id: SvgImageId) -> Option<&SvgImage> {
        self.images.iter().find(|image| image.id == id)
    }
}

pub struct SvgRasterMask {
    pub width: u16,
    pub height: u16,
    data: Vec<u8>,
}

impl SvgRasterMask {
    pub fn alpha_at(&self, x: u16, y: u16) -> u8 {
        self.data
            .get(y as usize * self.width as usize + x as usize)
            .copied()
            .unwrap_or(0)
    }
}

struct SvgRasterCache {
    entries: Vec<CachedSvgRaster>,
    capacity: usize,
    clock: u32,
    hits: u32,
    misses: u32,
    evictions: u32,
}

impl SvgRasterCache {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity: sanitize_svg_cache_capacity(capacity),
            clock: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    fn clear_id(&mut self, id: SvgImageId) {
        self.entries.retain(|entry| entry.id != id);
    }

    fn set_capacity(&mut self, capacity: usize) {
        self.capacity = sanitize_svg_cache_capacity(capacity);
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

    fn find(
        &self,
        id: SvgImageId,
        width: u16,
        height: u16,
        source: SvgResourceSource,
    ) -> Option<usize> {
        self.entries.iter().position(|entry| {
            entry.id == id && entry.width == width && entry.height == height && entry.source == source
        })
    }

    fn mask(&self, index: usize) -> &SvgRasterMask {
        &self.entries[index].mask
    }

    fn insert(
        &mut self,
        id: SvgImageId,
        width: u16,
        height: u16,
        source: SvgResourceSource,
        mask: SvgRasterMask,
    ) -> usize {
        let age = self.next_age();
        let entry = CachedSvgRaster {
            id,
            width,
            height,
            source,
            age,
            mask,
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
        id: SvgImageId,
        width: u16,
        height: u16,
        source: SvgResourceSource,
        mask: SvgRasterMask,
    ) -> bool {
        if self.is_full() {
            return false;
        }

        let age = self.next_age();
        self.entries.push(CachedSvgRaster {
            id,
            width,
            height,
            source,
            age,
            mask,
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

    fn summary(&self) -> SvgCacheSummary {
        SvgCacheSummary {
            entries: self.entries.len().min(u16::MAX as usize) as u16,
            capacity: self.capacity.min(u16::MAX as usize) as u16,
            profile: SvgCacheProfile::from_capacity(self.capacity),
            bytes: self.raster_bytes(),
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
        }
    }

    fn raster_bytes(&self) -> u32 {
        self.entries.iter().fold(0u32, |bytes, entry| {
            bytes.saturating_add(entry.mask.data.len() as u32)
        })
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

struct CachedSvgRaster {
    id: SvgImageId,
    width: u16,
    height: u16,
    source: SvgResourceSource,
    age: u32,
    mask: SvgRasterMask,
}

fn rasterize_svg(image: &SvgImage, width: u16, height: u16) -> SvgRasterMask {
    let mut data = vec![0; width as usize * height as usize];
    for y in 0..height {
        for x in 0..width {
            let coverage = svg_coverage(image, width, height, x, y);
            let alpha = (coverage as u16 * 255 / SVG_AA_SAMPLE_COUNT as u16) as u8;
            data[y as usize * width as usize + x as usize] = alpha;
        }
    }
    SvgRasterMask {
        width,
        height,
        data,
    }
}

fn svg_coverage(image: &SvgImage, width: u16, height: u16, x: u16, y: u16) -> u8 {
    let mut coverage = 0u8;
    for oy in SVG_AA_SAMPLE_OFFSETS {
        for ox in SVG_AA_SAMPLE_OFFSETS {
            let local_x = ((x as i32 * SVG_AA_SCALE + ox) as i64 * SVG_COORD_EXTENT as i64
                / (width.max(1) as i64 * SVG_AA_SCALE as i64)) as i32;
            let local_y = ((y as i32 * SVG_AA_SCALE + oy) as i64 * SVG_COORD_EXTENT as i64
                / (height.max(1) as i64 * SVG_AA_SCALE as i64)) as i32;
            if svg_contains(image, local_x, local_y) {
                coverage = coverage.saturating_add(1);
            }
        }
    }
    coverage
}

fn svg_contains(image: &SvgImage, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= SVG_COORD_EXTENT || y >= SVG_COORD_EXTENT {
        return false;
    }

    let mut inside = false;
    for contour in &image.contours {
        if svg_contour_crosses(&image.points, *contour, x, y) {
            inside = !inside;
        }
    }
    inside
}

fn svg_contour_crosses(points: &[SvgPoint], contour: SvgContour, x: i32, y: i32) -> bool {
    let start = contour.start as usize;
    let len = contour.len as usize;
    if len < 3 || start.saturating_add(len) > points.len() {
        return false;
    }

    let mut crosses = false;
    let mut previous = points[start + len - 1];
    for index in 0..len {
        let current = points[start + index];
        let y0 = previous.y as i32;
        let y1 = current.y as i32;
        if (y0 > y) != (y1 > y) {
            let x0 = previous.x as i32;
            let x1 = current.x as i32;
            let intersection =
                x0 as i64 + (y - y0) as i64 * (x1 - x0) as i64 / (y1 - y0) as i64;
            if x as i64 <= intersection {
                crosses = !crosses;
            }
        }
        previous = current;
    }
    crosses
}

fn parse_svg_image(bytes: &[u8]) -> Option<SvgImage> {
    let svg = core::str::from_utf8(bytes).ok()?;
    let view = parse_view_box(svg).unwrap_or_else(|| {
        let width = attr_value(svg, "width")
            .and_then(parse_dimension)
            .unwrap_or(16.0)
            .max(1.0);
        let height = attr_value(svg, "height")
            .and_then(parse_dimension)
            .unwrap_or(16.0)
            .max(1.0);
        SvgViewBox {
            min_x: 0.0,
            min_y: 0.0,
            width,
            height,
        }
    });

    if view.width <= 0.0 || view.height <= 0.0 {
        return None;
    }

    let mut float_contours = Vec::new();
    let mut offset = 0usize;
    while let Some(index) = svg[offset..].find("<path") {
        let tag_start = offset + index;
        let Some(tag_end) = svg[tag_start..].find('>') else {
            return None;
        };
        let tag = &svg[tag_start..tag_start + tag_end + 1];
        offset = tag_start + tag_end + 1;

        if attr_value(tag, "fill").is_some_and(|fill| fill.trim() == "none") {
            continue;
        }

        let Some(data) = attr_value(tag, "d") else {
            continue;
        };
        let mut contours = parse_path_data(data).ok()?;
        float_contours.append(&mut contours);
    }

    if float_contours.is_empty() {
        return None;
    }

    let mut points = Vec::new();
    let mut contours = Vec::new();
    for contour in float_contours {
        if contour.points.len() < 3 {
            continue;
        }

        let start = points.len();
        let mut previous = None;
        for point in contour.points {
            let fixed = normalize_svg_point(point, view);
            if previous == Some(fixed) {
                continue;
            }
            previous = Some(fixed);
            points.push(fixed);
        }

        let len = points.len().saturating_sub(start);
        if len >= 3 && start <= u16::MAX as usize && len <= u16::MAX as usize {
            contours.push(SvgContour {
                start: start as u16,
                len: len as u16,
            });
        } else {
            points.truncate(start);
        }
    }

    if points.is_empty() || contours.is_empty() {
        return None;
    }

    Some(SvgImage {
        id: SvgImageId(0),
        contours,
        points,
    })
}

fn normalize_svg_point(point: SvgFloatPoint, view: SvgViewBox) -> SvgPoint {
    let x = (point.x - view.min_x) * SVG_COORD_EXTENT as f32 / view.width;
    let y = (point.y - view.min_y) * SVG_COORD_EXTENT as f32 / view.height;
    SvgPoint {
        x: clamp_i16(round_to_i32(x)),
        y: clamp_i16(round_to_i32(y)),
    }
}

fn attr_value<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let bytes = text.as_bytes();
    let name_bytes = name.as_bytes();
    let mut offset = 0usize;

    while let Some(index) = text[offset..].find(name) {
        let start = offset + index;
        let before = start.checked_sub(1).and_then(|i| bytes.get(i).copied());
        if before.is_some_and(is_attr_name_byte) {
            offset = start + name.len();
            continue;
        }

        let mut pos = start + name_bytes.len();
        while matches!(bytes.get(pos), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            pos += 1;
        }
        if bytes.get(pos) != Some(&b'=') {
            offset = start + name.len();
            continue;
        }
        pos += 1;
        while matches!(bytes.get(pos), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            pos += 1;
        }

        let quote = *bytes.get(pos)?;
        if quote != b'"' && quote != b'\'' {
            return None;
        }
        pos += 1;
        let value_start = pos;
        while bytes.get(pos).copied()? != quote {
            pos += 1;
        }
        return text.get(value_start..pos);
    }

    None
}

fn is_attr_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':')
}

fn parse_view_box(svg: &str) -> Option<SvgViewBox> {
    let raw = attr_value(svg, "viewBox")?;
    let mut parser = PathDataParser::new(raw);
    Some(SvgViewBox {
        min_x: parser.parse_number().ok()?,
        min_y: parser.parse_number().ok()?,
        width: parser.parse_number().ok()?,
        height: parser.parse_number().ok()?,
    })
}

fn parse_dimension(raw: &str) -> Option<f32> {
    let mut parser = PathDataParser::new(raw);
    parser.parse_number().ok()
}

#[derive(Clone, Copy)]
struct SvgViewBox {
    min_x: f32,
    min_y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug)]
struct SvgFloatPoint {
    x: f32,
    y: f32,
}

impl SvgFloatPoint {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn scale(self, scale: f32) -> Self {
        Self::new(self.x * scale, self.y * scale)
    }
}

impl Add for SvgFloatPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

#[derive(Debug)]
struct SvgFloatContour {
    points: Vec<SvgFloatPoint>,
}

struct PathBuilder {
    contours: Vec<SvgFloatContour>,
    current: SvgFloatPoint,
    subpath_start: SvgFloatPoint,
    active: Option<usize>,
    last_cubic_control: Option<SvgFloatPoint>,
    last_quad_control: Option<SvgFloatPoint>,
}

impl PathBuilder {
    fn new() -> Self {
        Self {
            contours: Vec::new(),
            current: SvgFloatPoint::new(0.0, 0.0),
            subpath_start: SvgFloatPoint::new(0.0, 0.0),
            active: None,
            last_cubic_control: None,
            last_quad_control: None,
        }
    }

    fn move_to(&mut self, point: SvgFloatPoint) {
        self.current = point;
        self.subpath_start = point;
        self.contours.push(SvgFloatContour {
            points: vec![point],
        });
        self.active = Some(self.contours.len() - 1);
        self.reset_controls();
    }

    fn line_to(&mut self, point: SvgFloatPoint) {
        if self.active.is_none() {
            self.move_to(self.current);
        }
        if let Some(index) = self.active {
            self.contours[index].points.push(point);
        }
        self.current = point;
        self.reset_controls();
    }

    fn cubic_to(&mut self, c1: SvgFloatPoint, c2: SvgFloatPoint, end: SvgFloatPoint) {
        let start = self.current;
        for step in 1..=CUBIC_SEGMENTS {
            let t = step as f32 / CUBIC_SEGMENTS as f32;
            self.line_to(cubic_point(start, c1, c2, end, t));
        }
        self.last_cubic_control = Some(c2);
        self.last_quad_control = None;
    }

    fn smooth_cubic_to(&mut self, c2: SvgFloatPoint, end: SvgFloatPoint) {
        let c1 = self
            .last_cubic_control
            .map(|control| reflect_point(control, self.current))
            .unwrap_or(self.current);
        self.cubic_to(c1, c2, end);
    }

    fn quad_to(&mut self, c: SvgFloatPoint, end: SvgFloatPoint) {
        let start = self.current;
        for step in 1..=QUAD_SEGMENTS {
            let t = step as f32 / QUAD_SEGMENTS as f32;
            self.line_to(quad_point(start, c, end, t));
        }
        self.last_quad_control = Some(c);
        self.last_cubic_control = None;
    }

    fn smooth_quad_to(&mut self, end: SvgFloatPoint) {
        let c = self
            .last_quad_control
            .map(|control| reflect_point(control, self.current))
            .unwrap_or(self.current);
        self.quad_to(c, end);
    }

    fn arc_to(
        &mut self,
        rx: f32,
        ry: f32,
        angle_degrees: f32,
        large_arc: bool,
        sweep: bool,
        end: SvgFloatPoint,
    ) {
        let points = flatten_arc(self.current, rx, ry, angle_degrees, large_arc, sweep, end);
        for point in points {
            self.line_to(point);
        }
        self.reset_controls();
    }

    fn close(&mut self) {
        self.current = self.subpath_start;
        self.active = None;
        self.reset_controls();
    }

    fn reset_controls(&mut self) {
        self.last_cubic_control = None;
        self.last_quad_control = None;
    }

    fn into_contours(self) -> Vec<SvgFloatContour> {
        self.contours
    }
}

fn parse_path_data(data: &str) -> Result<Vec<SvgFloatContour>, ()> {
    let mut parser = PathDataParser::new(data);
    let mut builder = PathBuilder::new();
    let mut command = None;

    while parser.has_more() {
        if let Some(next) = parser.consume_command() {
            command = Some(next);
        }
        let active_command = command.ok_or(())?;

        match active_command {
            'M' | 'm' => {
                let relative = active_command == 'm';
                let mut first = true;
                while parser.next_is_number() {
                    let point = absolute_point(builder.current, parser.parse_pair()?, relative);
                    if first {
                        builder.move_to(point);
                        first = false;
                    } else {
                        builder.line_to(point);
                    }
                }
                command = Some(if relative { 'l' } else { 'L' });
            }
            'L' | 'l' => {
                let relative = active_command == 'l';
                while parser.next_is_number() {
                    let point = absolute_point(builder.current, parser.parse_pair()?, relative);
                    builder.line_to(point);
                }
            }
            'H' | 'h' => {
                let relative = active_command == 'h';
                while parser.next_is_number() {
                    let value = parser.parse_number()?;
                    let x = if relative {
                        builder.current.x + value
                    } else {
                        value
                    };
                    builder.line_to(SvgFloatPoint::new(x, builder.current.y));
                }
            }
            'V' | 'v' => {
                let relative = active_command == 'v';
                while parser.next_is_number() {
                    let value = parser.parse_number()?;
                    let y = if relative {
                        builder.current.y + value
                    } else {
                        value
                    };
                    builder.line_to(SvgFloatPoint::new(builder.current.x, y));
                }
            }
            'C' | 'c' => {
                let relative = active_command == 'c';
                while parser.next_is_number() {
                    let c1 = absolute_point(builder.current, parser.parse_pair()?, relative);
                    let c2 = absolute_point(builder.current, parser.parse_pair()?, relative);
                    let end = absolute_point(builder.current, parser.parse_pair()?, relative);
                    builder.cubic_to(c1, c2, end);
                }
            }
            'S' | 's' => {
                let relative = active_command == 's';
                while parser.next_is_number() {
                    let c2 = absolute_point(builder.current, parser.parse_pair()?, relative);
                    let end = absolute_point(builder.current, parser.parse_pair()?, relative);
                    builder.smooth_cubic_to(c2, end);
                }
            }
            'Q' | 'q' => {
                let relative = active_command == 'q';
                while parser.next_is_number() {
                    let c = absolute_point(builder.current, parser.parse_pair()?, relative);
                    let end = absolute_point(builder.current, parser.parse_pair()?, relative);
                    builder.quad_to(c, end);
                }
            }
            'T' | 't' => {
                let relative = active_command == 't';
                while parser.next_is_number() {
                    let end = absolute_point(builder.current, parser.parse_pair()?, relative);
                    builder.smooth_quad_to(end);
                }
            }
            'A' | 'a' => {
                let relative = active_command == 'a';
                while parser.next_is_number() {
                    let rx = parser.parse_number()?;
                    let ry = parser.parse_number()?;
                    let angle = parser.parse_number()?;
                    let large_arc = parser.parse_flag()?;
                    let sweep = parser.parse_flag()?;
                    let end = absolute_point(builder.current, parser.parse_pair()?, relative);
                    builder.arc_to(rx, ry, angle, large_arc, sweep, end);
                }
            }
            'Z' | 'z' => builder.close(),
            _ => return Err(()),
        }
    }

    Ok(builder.into_contours())
}

struct PathDataParser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> PathDataParser<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            data: data.as_bytes(),
            pos: 0,
        }
    }

    fn has_more(&mut self) -> bool {
        self.skip_separators();
        self.pos < self.data.len()
    }

    fn consume_command(&mut self) -> Option<char> {
        self.skip_separators();
        let byte = *self.data.get(self.pos)?;
        let ch = byte as char;
        if ch.is_ascii_alphabetic() {
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn next_is_number(&mut self) -> bool {
        self.skip_separators();
        self.data
            .get(self.pos)
            .map(|byte| matches!(*byte, b'+' | b'-' | b'.' | b'0'..=b'9'))
            .unwrap_or(false)
    }

    fn parse_pair(&mut self) -> Result<SvgFloatPoint, ()> {
        Ok(SvgFloatPoint::new(self.parse_number()?, self.parse_number()?))
    }

    fn parse_flag(&mut self) -> Result<bool, ()> {
        Ok(self.parse_number()? != 0.0)
    }

    fn parse_number(&mut self) -> Result<f32, ()> {
        self.skip_separators();
        let sign = if matches!(self.data.get(self.pos), Some(b'+' | b'-')) {
            let sign = if self.data[self.pos] == b'-' { -1.0 } else { 1.0 };
            self.pos += 1;
            sign
        } else {
            1.0
        };

        let mut value = 0.0f32;
        let mut saw_digit = false;
        while matches!(self.data.get(self.pos), Some(b'0'..=b'9')) {
            value = value * 10.0 + (self.data[self.pos] - b'0') as f32;
            saw_digit = true;
            self.pos += 1;
        }

        if matches!(self.data.get(self.pos), Some(b'.')) {
            self.pos += 1;
            let mut place = 0.1f32;
            while matches!(self.data.get(self.pos), Some(b'0'..=b'9')) {
                value += (self.data[self.pos] - b'0') as f32 * place;
                place *= 0.1;
                saw_digit = true;
                self.pos += 1;
            }
        }

        if !saw_digit {
            return Err(());
        }

        if matches!(self.data.get(self.pos), Some(b'e' | b'E')) {
            let exponent = self.pos;
            self.pos += 1;
            let exponent_sign = if matches!(self.data.get(self.pos), Some(b'+' | b'-')) {
                let sign = if self.data[self.pos] == b'-' { -1 } else { 1 };
                self.pos += 1;
                sign
            } else {
                1
            };
            let exponent_digits = self.pos;
            let mut exponent_value = 0i32;
            while matches!(self.data.get(self.pos), Some(b'0'..=b'9')) {
                exponent_value = exponent_value
                    .saturating_mul(10)
                    .saturating_add((self.data[self.pos] - b'0') as i32);
                self.pos += 1;
            }
            if self.pos == exponent_digits {
                self.pos = exponent;
            } else {
                value *= pow10_i32(exponent_sign * exponent_value);
            }
        }

        Ok(sign * value)
    }

    fn skip_separators(&mut self) {
        while matches!(self.data.get(self.pos), Some(b' ' | b'\n' | b'\r' | b'\t' | b',')) {
            self.pos += 1;
        }
    }
}

fn absolute_point(current: SvgFloatPoint, point: SvgFloatPoint, relative: bool) -> SvgFloatPoint {
    if relative {
        SvgFloatPoint::new(current.x + point.x, current.y + point.y)
    } else {
        point
    }
}

fn reflect_point(point: SvgFloatPoint, around: SvgFloatPoint) -> SvgFloatPoint {
    SvgFloatPoint::new(around.x * 2.0 - point.x, around.y * 2.0 - point.y)
}

fn cubic_point(
    a: SvgFloatPoint,
    b: SvgFloatPoint,
    c: SvgFloatPoint,
    d: SvgFloatPoint,
    t: f32,
) -> SvgFloatPoint {
    let mt = 1.0 - t;
    a.scale(mt * mt * mt)
        + b.scale(3.0 * mt * mt * t)
        + c.scale(3.0 * mt * t * t)
        + d.scale(t * t * t)
}

fn quad_point(a: SvgFloatPoint, b: SvgFloatPoint, c: SvgFloatPoint, t: f32) -> SvgFloatPoint {
    let mt = 1.0 - t;
    a.scale(mt * mt) + b.scale(2.0 * mt * t) + c.scale(t * t)
}

fn flatten_arc(
    start: SvgFloatPoint,
    mut rx: f32,
    mut ry: f32,
    angle_degrees: f32,
    large_arc: bool,
    sweep: bool,
    end: SvgFloatPoint,
) -> Vec<SvgFloatPoint> {
    rx = f_abs(rx);
    ry = f_abs(ry);
    if rx == 0.0 || ry == 0.0 || nearly_same(start, end) {
        return vec![end];
    }

    let phi = angle_degrees * PI / 180.0;
    let cos_phi = cos_approx(phi);
    let sin_phi = sin_approx(phi);
    let dx = (start.x - end.x) / 2.0;
    let dy = (start.y - end.y) / 2.0;
    let x1p = cos_phi * dx + sin_phi * dy;
    let y1p = -sin_phi * dx + cos_phi * dy;

    let radii_check = x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry);
    if radii_check > 1.0 {
        let scale = sqrt_approx(radii_check);
        rx *= scale;
        ry *= scale;
    }

    let rx2 = rx * rx;
    let ry2 = ry * ry;
    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;
    let denominator = rx2 * y1p2 + ry2 * x1p2;
    if denominator == 0.0 {
        return vec![end];
    }

    let ratio = (rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2) / denominator;
    let mut factor = sqrt_approx(f_max(0.0, ratio));
    if large_arc == sweep {
        factor = -factor;
    }

    let cxp = factor * rx * y1p / ry;
    let cyp = -factor * ry * x1p / rx;
    let cx = cos_phi * cxp - sin_phi * cyp + (start.x + end.x) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (start.y + end.y) / 2.0;

    let ux = (x1p - cxp) / rx;
    let uy = (y1p - cyp) / ry;
    let vx = (-x1p - cxp) / rx;
    let vy = (-y1p - cyp) / ry;

    let theta = vector_angle(1.0, 0.0, ux, uy);
    let mut delta = vector_angle(ux, uy, vx, vy);
    if !sweep && delta > 0.0 {
        delta -= TWO_PI;
    } else if sweep && delta < 0.0 {
        delta += TWO_PI;
    }

    let segments = ceil_to_usize(f_abs(delta) / ARC_MAX_RADIANS_PER_SEGMENT).max(1);
    let mut points = Vec::with_capacity(segments);
    for segment in 1..=segments {
        let angle = theta + delta * segment as f32 / segments as f32;
        let cos_angle = cos_approx(angle);
        let sin_angle = sin_approx(angle);
        let x = cos_phi * rx * cos_angle - sin_phi * ry * sin_angle + cx;
        let y = sin_phi * rx * cos_angle + cos_phi * ry * sin_angle + cy;
        points.push(SvgFloatPoint::new(x, y));
    }
    points
}

fn vector_angle(ux: f32, uy: f32, vx: f32, vy: f32) -> f32 {
    atan2_approx(ux * vy - uy * vx, ux * vx + uy * vy)
}

fn nearly_same(a: SvgFloatPoint, b: SvgFloatPoint) -> bool {
    f_abs(a.x - b.x) < 0.00001 && f_abs(a.y - b.y) < 0.00001
}

fn sanitize_raster_side(side: u16) -> u16 {
    side.clamp(1, SVG_RASTER_MAX_SIDE)
}

pub fn default_svg_cache_capacity() -> usize {
    if let Some(raw) = option_env!("WING_SVG_CACHE_CAPACITY") {
        if let Some(capacity) = parse_cache_capacity(raw) {
            return sanitize_svg_cache_capacity(capacity);
        }
    }

    match option_env!("WING_SVG_CACHE_PROFILE") {
        Some("tiny") | Some("TINY") | Some("mcu") | Some("MCU") => SVG_CACHE_TINY_CAPACITY,
        Some("large") | Some("LARGE") | Some("mpu") | Some("MPU") => SVG_CACHE_LARGE_CAPACITY,
        Some("balanced") | Some("BALANCED") | Some("base") | Some("BASE") | None => {
            SVG_CACHE_BALANCED_CAPACITY
        }
        Some(_) => SVG_CACHE_BALANCED_CAPACITY,
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

fn sanitize_svg_cache_capacity(capacity: usize) -> usize {
    capacity.clamp(SVG_CACHE_MIN_CAPACITY, SVG_CACHE_MAX_CAPACITY)
}

fn clamp_i16(value: i32) -> i16 {
    value.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn round_to_i32(value: f32) -> i32 {
    if value >= 0.0 {
        (value + 0.5) as i32
    } else {
        (value - 0.5) as i32
    }
}

fn ceil_to_usize(value: f32) -> usize {
    if value <= 0.0 {
        return 0;
    }
    let whole = value as usize;
    if value > whole as f32 {
        whole.saturating_add(1)
    } else {
        whole
    }
}

fn f_abs(value: f32) -> f32 {
    if value < 0.0 {
        -value
    } else {
        value
    }
}

fn f_max(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

fn pow10_i32(exponent: i32) -> f32 {
    let exponent = exponent.clamp(-38, 38);
    let mut value = 1.0f32;
    if exponent >= 0 {
        for _ in 0..exponent {
            value *= 10.0;
        }
    } else {
        for _ in 0..(-exponent) {
            value *= 0.1;
        }
    }
    value
}

fn sqrt_approx(value: f32) -> f32 {
    if value <= 0.0 {
        return 0.0;
    }

    let mut x = if value >= 1.0 { value } else { 1.0 };
    for _ in 0..8 {
        x = 0.5 * (x + value / x);
    }
    x
}

fn sin_approx(value: f32) -> f32 {
    let mut x = wrap_pi(value);
    if x > HALF_PI {
        x = PI - x;
    } else if x < -HALF_PI {
        x = -PI - x;
    }

    let x2 = x * x;
    x * (1.0 - x2 / 6.0 + x2 * x2 / 120.0 - x2 * x2 * x2 / 5040.0)
}

fn cos_approx(value: f32) -> f32 {
    sin_approx(HALF_PI - value)
}

fn atan2_approx(y: f32, x: f32) -> f32 {
    if x == 0.0 {
        return if y > 0.0 {
            HALF_PI
        } else if y < 0.0 {
            -HALF_PI
        } else {
            0.0
        };
    }

    let atan = atan_approx(y / x);
    if x > 0.0 {
        atan
    } else if y >= 0.0 {
        atan + PI
    } else {
        atan - PI
    }
}

fn atan_approx(value: f32) -> f32 {
    let abs = f_abs(value);
    if abs <= 1.0 {
        value / (1.0 + 0.28 * value * value)
    } else {
        let base = HALF_PI - atan_approx(1.0 / abs);
        if value < 0.0 {
            -base
        } else {
            base
        }
    }
}

fn wrap_pi(mut value: f32) -> f32 {
    while value > PI {
        value -= TWO_PI;
    }
    while value < -PI {
        value += TWO_PI;
    }
    value
}
