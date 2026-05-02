use crate::backend::{
    CodecAcceleratorCapabilities, CodecPipelinePlan, CodecStageKind, CodecStagePlan,
};
use alloc::vec::Vec;

const TYPE2_STACK_LIMIT: usize = 48;
const TYPE2_SUBR_DEPTH_LIMIT: u8 = 8;
pub const GLYPH_RUN_FLAG_GSUB: u8 = 1 << 0;
pub const GLYPH_RUN_FLAG_GPOS: u8 = 1 << 1;
pub const GLYPH_RUN_FLAG_KERN: u8 = 1 << 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphRasterOptions {
    pub pixel_size: u8,
}

impl GlyphRasterOptions {
    pub const DEFAULT: Self = Self { pixel_size: 16 };
}

#[derive(Debug)]
pub struct RasterGlyph {
    pub codepoint: u32,
    pub width: u8,
    pub height: u8,
    pub advance: u8,
    pub bearing_x: i8,
    pub bearing_y: i8,
    pub data: Vec<u8>,
}

impl RasterGlyph {
    pub fn byte_size(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0
        } else {
            self.width as f32 / self.height as f32
        }
    }

    pub fn scaled_width(&self, scale: u16) -> u16 {
        self.width as u16 * scale / 16
    }

    pub fn scaled_height(&self, scale: u16) -> u16 {
        self.height as u16 * scale / 16
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TtfError {
    BadSignature,
    MissingTable,
    Truncated,
    Unsupported,
    InvalidGlyph,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontFaceKind {
    TrueTypeGlyf,
    OpenTypeCff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShapeOptions {
    pub kerning: bool,
    pub gsub: bool,
    pub gpos: bool,
}

impl ShapeOptions {
    pub const fn new(enable_gsub: bool, enable_gpos: bool, enable_kern: bool) -> Self {
        Self {
            kerning: enable_kern,
            gsub: enable_gsub,
            gpos: enable_gpos,
        }
    }

    pub const FAST: Self = Self {
        kerning: false,
        gsub: false,
        gpos: false,
    };

    pub const LATIN: Self = Self {
        kerning: true,
        gsub: true,
        gpos: true,
    };

    pub const fn enable_gsub(self) -> bool {
        self.gsub
    }

    pub const fn enable_gpos(self) -> bool {
        self.gpos
    }

    pub const fn enable_kern(self) -> bool {
        self.kerning
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphRunItem {
    pub codepoint: u32,
    pub glyph_id: u16,
    pub char_start: u16,
    pub char_len: u16,
    pub advance: i16,
    pub x_offset: i16,
    pub shaping: u8,
}

impl GlyphRunItem {
    pub const EMPTY: Self = Self {
        codepoint: 0,
        glyph_id: 0,
        char_start: 0,
        char_len: 0,
        advance: 0,
        x_offset: 0,
        shaping: 0,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphRun<const N: usize> {
    pub items: [GlyphRunItem; N],
    pub len: usize,
    pub overflowed: bool,
}

impl<const N: usize> GlyphRun<N> {
    pub const fn new() -> Self {
        Self {
            items: [GlyphRunItem::EMPTY; N],
            len: 0,
            overflowed: false,
        }
    }

    pub fn push(&mut self, item: GlyphRunItem) {
        if self.len >= N {
            self.overflowed = true;
            return;
        }
        self.items[self.len] = item;
        self.len += 1;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenTypeLayout {
    pub has_gsub: bool,
    pub has_gpos: bool,
    pub has_kern: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Table {
    offset: usize,
    len: usize,
}

impl Table {
    const EMPTY: Self = Self { offset: 0, len: 0 };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Tables {
    cmap: Table,
    head: Table,
    hhea: Table,
    hmtx: Table,
    maxp: Table,
    loca: Table,
    glyf: Table,
    kern: Table,
    cff: Table,
    gpos: Table,
    gsub: Table,
}

impl Tables {
    const fn empty() -> Self {
        Self {
            cmap: Table::EMPTY,
            head: Table::EMPTY,
            hhea: Table::EMPTY,
            hmtx: Table::EMPTY,
            maxp: Table::EMPTY,
            loca: Table::EMPTY,
            glyf: Table::EMPTY,
            kern: Table::EMPTY,
            cff: Table::EMPTY,
            gpos: Table::EMPTY,
            gsub: Table::EMPTY,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FontInfo {
    pub units_per_em: u16,
    pub glyph_count: u16,
    pub ascent: i16,
    pub descent: i16,
    pub kerning_pairs: u16,
}

pub struct FontFace<'a> {
    data: &'a [u8],
    tables: Tables,
    info: FontInfo,
    kind: FontFaceKind,
    index_to_loc_format: i16,
    hmetric_count: u16,
}

pub struct TtfDecoder;

impl TtfDecoder {
    pub fn parse(bytes: &[u8]) -> Result<FontFace<'_>, TtfError> {
        FontFace::parse(bytes)
    }

    pub fn plan_pipeline<const STAGES: usize>() -> CodecPipelinePlan<STAGES> {
        Self::plan_pipeline_with_caps(CodecAcceleratorCapabilities::NONE)
    }

    pub fn plan_pipeline_with_caps<const STAGES: usize>(
        caps: CodecAcceleratorCapabilities,
    ) -> CodecPipelinePlan<STAGES> {
        let supported = caps.ttf && caps.max_stages >= 7;
        let mut plan = CodecPipelinePlan::new();
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Read, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Inspect, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Header, true, supported));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Parse, true, supported));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Raster, true, supported));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Pack, true, supported));
        let _ = plan.push(CodecStagePlan::new(
            CodecStageKind::CacheInsert,
            false,
            true,
        ));
        plan
    }
}

impl<'a> FontFace<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self, TtfError> {
        let sfnt = read_u32(data, 0).ok_or(TtfError::Truncated)?;
        if sfnt != 0x0001_0000
            && sfnt != u32::from_be_bytes(*b"true")
            && sfnt != u32::from_be_bytes(*b"OTTO")
        {
            return Err(TtfError::BadSignature);
        }
        let count = read_u16(data, 4).ok_or(TtfError::Truncated)? as usize;
        let mut tables = Tables::empty();
        let mut i = 0usize;
        while i < count {
            let base = 12 + i * 16;
            let tag = data.get(base..base + 4).ok_or(TtfError::Truncated)?;
            let offset = read_u32(data, base + 8).ok_or(TtfError::Truncated)? as usize;
            let len = read_u32(data, base + 12).ok_or(TtfError::Truncated)? as usize;
            if offset.checked_add(len).ok_or(TtfError::Truncated)? > data.len() {
                return Err(TtfError::Truncated);
            }
            let table = Table { offset, len };
            match tag {
                b"cmap" => tables.cmap = table,
                b"head" => tables.head = table,
                b"hhea" => tables.hhea = table,
                b"hmtx" => tables.hmtx = table,
                b"maxp" => tables.maxp = table,
                b"loca" => tables.loca = table,
                b"glyf" => tables.glyf = table,
                b"kern" => tables.kern = table,
                b"CFF " => tables.cff = table,
                b"GPOS" => tables.gpos = table,
                b"GSUB" => tables.gsub = table,
                _ => {}
            }
            i += 1;
        }
        let kind = if tables.glyf.len != 0 && tables.loca.len != 0 {
            FontFaceKind::TrueTypeGlyf
        } else if sfnt == u32::from_be_bytes(*b"OTTO") && tables.cff.len != 0 {
            FontFaceKind::OpenTypeCff
        } else {
            return Err(TtfError::MissingTable);
        };
        if tables.cmap.len == 0
            || tables.head.len < 54
            || tables.hhea.len < 36
            || tables.hmtx.len == 0
            || tables.maxp.len < 6
        {
            return Err(TtfError::MissingTable);
        }

        let units_per_em = read_u16(data, tables.head.offset + 18).ok_or(TtfError::Truncated)?;
        let index_to_loc_format =
            read_i16(data, tables.head.offset + 50).ok_or(TtfError::Truncated)?;
        let ascent = read_i16(data, tables.hhea.offset + 4).ok_or(TtfError::Truncated)?;
        let descent = read_i16(data, tables.hhea.offset + 6).ok_or(TtfError::Truncated)?;
        let hmetric_count = read_u16(data, tables.hhea.offset + 34).ok_or(TtfError::Truncated)?;
        let glyph_count = read_u16(data, tables.maxp.offset + 4).ok_or(TtfError::Truncated)?;
        let kerning_pairs = count_kern_pairs(data, tables.kern).unwrap_or(0);

        Ok(Self {
            data,
            tables,
            info: FontInfo {
                units_per_em,
                glyph_count,
                ascent,
                descent,
                kerning_pairs,
            },
            kind,
            index_to_loc_format,
            hmetric_count,
        })
    }

    pub const fn info(&self) -> FontInfo {
        self.info
    }

    pub const fn kind(&self) -> FontFaceKind {
        self.kind
    }

    pub const fn layout(&self) -> OpenTypeLayout {
        OpenTypeLayout {
            has_gsub: self.tables.gsub.len != 0,
            has_gpos: self.tables.gpos.len != 0,
            has_kern: self.tables.kern.len != 0,
        }
    }

    pub fn shape_text<const N: usize>(&self, text: &str, options: ShapeOptions) -> GlyphRun<N> {
        let mut run = GlyphRun::new();
        let mut previous = None;
        let mut chars = text.chars().enumerate().peekable();
        while let Some((char_index, ch)) = chars.next() {
            let mut codepoint = ch as u32;
            let mut char_len = 1u16;
            let mut shaping = 0u8;
            let mut glyph_id = self.glyph_index(codepoint).unwrap_or(0);
            if options.gsub {
                if let Some((_, next)) = chars.peek().copied() {
                    let next_glyph = self.glyph_index(next as u32).unwrap_or(0);
                    if let Some(ligature_glyph) =
                        gsub_ligature_substitute(self.data, self.tables.gsub, glyph_id, next_glyph)
                    {
                        let _ = chars.next();
                        glyph_id = ligature_glyph;
                        char_len = 2;
                        shaping |= GLYPH_RUN_FLAG_GSUB;
                    } else if let Some(ligature) = latin_ligature_codepoint(ch, next) {
                        if let Some(found) = self.glyph_index(ligature) {
                            let _ = chars.next();
                            codepoint = ligature;
                            glyph_id = found;
                            char_len = 2;
                            shaping |= GLYPH_RUN_FLAG_GSUB;
                        }
                    }
                }
            }
            if options.gsub {
                if let Some(substitute) =
                    gsub_single_substitute(self.data, self.tables.gsub, glyph_id)
                {
                    glyph_id = substitute;
                    shaping |= GLYPH_RUN_FLAG_GSUB;
                }
            }
            let mut advance = self
                .advance_width(glyph_id)
                .unwrap_or(self.info.units_per_em / 2) as i16;
            if let Some(left) = previous {
                if options.gpos {
                    let adjust = gpos_pair_adjust(self.data, self.tables.gpos, left, glyph_id);
                    if adjust != 0 {
                        advance = advance.saturating_add(adjust);
                        shaping |= GLYPH_RUN_FLAG_GPOS;
                    }
                }
                if options.kerning {
                    let adjust = self.kerning(left, glyph_id);
                    if adjust != 0 {
                        advance = advance.saturating_add(adjust);
                        shaping |= GLYPH_RUN_FLAG_KERN;
                    }
                }
            }
            run.push(GlyphRunItem {
                codepoint,
                glyph_id,
                char_start: char_index.min(u16::MAX as usize) as u16,
                char_len,
                advance,
                x_offset: 0,
                shaping,
            });
            previous = Some(glyph_id);
        }
        run
    }

    pub fn glyph_index(&self, codepoint: u32) -> Option<u16> {
        cmap_lookup(self.data, self.tables.cmap, codepoint)
    }

    pub fn kerning(&self, left_glyph: u16, right_glyph: u16) -> i16 {
        kern_format0_lookup(self.data, self.tables.kern, left_glyph, right_glyph).unwrap_or(0)
    }

    pub fn rasterize_glyph(
        &self,
        codepoint: u32,
        options: GlyphRasterOptions,
    ) -> Result<RasterGlyph, TtfError> {
        let glyph_id = self.glyph_index(codepoint).ok_or(TtfError::InvalidGlyph)?;
        let advance_units = self
            .advance_width(glyph_id)
            .unwrap_or(self.info.units_per_em / 2);
        if self.kind == FontFaceKind::OpenTypeCff {
            if let Ok(glyph) =
                rasterize_cff_glyph(self, codepoint, glyph_id, advance_units, options.pixel_size)
            {
                return Ok(glyph);
            }
            return Ok(cff_fallback_glyph(
                codepoint,
                advance_units,
                self.info.units_per_em,
                options.pixel_size,
            ));
        }
        let mut outline = Outline::new();
        self.load_outline(glyph_id, &mut outline, 0)?;
        if outline.points.is_empty() || outline.contours.is_empty() {
            return Ok(empty_glyph(
                codepoint,
                advance_units,
                self.info.units_per_em,
                options.pixel_size,
            ));
        }
        rasterize_outline(
            codepoint,
            &outline,
            advance_units,
            self.info.units_per_em,
            options.pixel_size,
        )
    }

    fn advance_width(&self, glyph_id: u16) -> Option<u16> {
        if self.hmetric_count == 0 {
            return None;
        }
        let metric_index = glyph_id.min(self.hmetric_count - 1) as usize;
        read_u16(self.data, self.tables.hmtx.offset + metric_index * 4)
    }

    fn glyph_slice(&self, glyph_id: u16) -> Result<&'a [u8], TtfError> {
        if glyph_id >= self.info.glyph_count {
            return Err(TtfError::InvalidGlyph);
        }
        let start = self.glyph_offset(glyph_id)?;
        let end = self.glyph_offset(glyph_id + 1)?;
        if end < start {
            return Err(TtfError::InvalidGlyph);
        }
        self.data
            .get(self.tables.glyf.offset + start..self.tables.glyf.offset + end)
            .ok_or(TtfError::Truncated)
    }

    fn glyph_offset(&self, glyph_id: u16) -> Result<usize, TtfError> {
        if self.index_to_loc_format == 0 {
            Ok(
                read_u16(self.data, self.tables.loca.offset + glyph_id as usize * 2)
                    .ok_or(TtfError::Truncated)? as usize
                    * 2,
            )
        } else {
            Ok(
                read_u32(self.data, self.tables.loca.offset + glyph_id as usize * 4)
                    .ok_or(TtfError::Truncated)? as usize,
            )
        }
    }

    fn load_outline(
        &self,
        glyph_id: u16,
        outline: &mut Outline,
        depth: u8,
    ) -> Result<(), TtfError> {
        if depth > 4 {
            return Err(TtfError::Unsupported);
        }
        let glyph = self.glyph_slice(glyph_id)?;
        if glyph.len() < 10 {
            return Ok(());
        }
        let contours = read_i16(glyph, 0).ok_or(TtfError::Truncated)?;
        if contours >= 0 {
            parse_simple_glyph(glyph, contours as usize, outline)
        } else {
            parse_composite_glyph(self, glyph, outline, depth + 1)
        }
    }
}

#[derive(Clone, Copy)]
struct OutlinePoint {
    x: i32,
    y: i32,
    _on_curve: bool,
}

struct Outline {
    points: Vec<OutlinePoint>,
    contours: Vec<usize>,
}

impl Outline {
    fn new() -> Self {
        Self {
            points: Vec::new(),
            contours: Vec::new(),
        }
    }
}

fn latin_ligature_codepoint(left: char, right: char) -> Option<u32> {
    match (left, right) {
        ('f', 'i') => Some(0xfb01),
        ('f', 'l') => Some(0xfb02),
        _ => None,
    }
}

fn parse_simple_glyph(
    glyph: &[u8],
    contour_count: usize,
    outline: &mut Outline,
) -> Result<(), TtfError> {
    let mut offset = 10usize;
    let mut ends = Vec::new();
    let mut i = 0usize;
    while i < contour_count {
        ends.push(read_u16(glyph, offset).ok_or(TtfError::Truncated)? as usize);
        offset += 2;
        i += 1;
    }
    let instruction_len = read_u16(glyph, offset).ok_or(TtfError::Truncated)? as usize;
    offset = offset
        .checked_add(2 + instruction_len)
        .ok_or(TtfError::Truncated)?;
    let point_count = ends.last().copied().unwrap_or(0).saturating_add(1);
    let mut flags = Vec::new();
    while flags.len() < point_count {
        let flag = *glyph.get(offset).ok_or(TtfError::Truncated)?;
        offset += 1;
        flags.push(flag);
        if flag & 0x08 != 0 {
            let repeat = *glyph.get(offset).ok_or(TtfError::Truncated)? as usize;
            offset += 1;
            let mut r = 0usize;
            while r < repeat {
                flags.push(flag);
                r += 1;
            }
        }
    }

    let mut xs = Vec::new();
    let mut x = 0i32;
    for flag in flags.iter().copied() {
        let dx = if flag & 0x02 != 0 {
            let v = *glyph.get(offset).ok_or(TtfError::Truncated)? as i32;
            offset += 1;
            if flag & 0x10 != 0 {
                v
            } else {
                -v
            }
        } else if flag & 0x10 != 0 {
            0
        } else {
            let v = read_i16(glyph, offset).ok_or(TtfError::Truncated)? as i32;
            offset += 2;
            v
        };
        x += dx;
        xs.push(x);
    }

    let mut ys = Vec::new();
    let mut y = 0i32;
    for flag in flags.iter().copied() {
        let dy = if flag & 0x04 != 0 {
            let v = *glyph.get(offset).ok_or(TtfError::Truncated)? as i32;
            offset += 1;
            if flag & 0x20 != 0 {
                v
            } else {
                -v
            }
        } else if flag & 0x20 != 0 {
            0
        } else {
            let v = read_i16(glyph, offset).ok_or(TtfError::Truncated)? as i32;
            offset += 2;
            v
        };
        y += dy;
        ys.push(y);
    }

    let start_len = outline.points.len();
    let mut p = 0usize;
    while p < point_count {
        outline.points.push(OutlinePoint {
            x: xs[p],
            y: ys[p],
            _on_curve: flags[p] & 0x01 != 0,
        });
        p += 1;
    }
    for end in ends {
        outline.contours.push(start_len + end);
    }
    Ok(())
}

fn parse_composite_glyph(
    face: &FontFace<'_>,
    glyph: &[u8],
    outline: &mut Outline,
    depth: u8,
) -> Result<(), TtfError> {
    let mut offset = 10usize;
    loop {
        let flags = read_u16(glyph, offset).ok_or(TtfError::Truncated)?;
        let sub_glyph = read_u16(glyph, offset + 2).ok_or(TtfError::Truncated)?;
        offset += 4;
        let (arg1, arg2) = if flags & 0x0001 != 0 {
            let a = read_i16(glyph, offset).ok_or(TtfError::Truncated)? as i32;
            let b = read_i16(glyph, offset + 2).ok_or(TtfError::Truncated)? as i32;
            offset += 4;
            (a, b)
        } else {
            let a = *glyph.get(offset).ok_or(TtfError::Truncated)? as i8 as i32;
            let b = *glyph.get(offset + 1).ok_or(TtfError::Truncated)? as i8 as i32;
            offset += 2;
            (a, b)
        };
        let (dx, dy) = if flags & 0x0002 != 0 {
            (arg1, arg2)
        } else {
            (0, 0)
        };
        let mut xx = 1.0f32;
        let mut yy = 1.0f32;
        if flags & 0x0008 != 0 {
            let s = f2dot14(read_i16(glyph, offset).ok_or(TtfError::Truncated)?);
            offset += 2;
            xx = s;
            yy = s;
        } else if flags & 0x0040 != 0 {
            xx = f2dot14(read_i16(glyph, offset).ok_or(TtfError::Truncated)?);
            yy = f2dot14(read_i16(glyph, offset + 2).ok_or(TtfError::Truncated)?);
            offset += 4;
        } else if flags & 0x0080 != 0 {
            xx = f2dot14(read_i16(glyph, offset).ok_or(TtfError::Truncated)?);
            yy = f2dot14(read_i16(glyph, offset + 6).ok_or(TtfError::Truncated)?);
            offset += 8;
        }
        let before = outline.points.len();
        face.load_outline(sub_glyph, outline, depth)?;
        let mut i = before;
        while i < outline.points.len() {
            outline.points[i].x = (outline.points[i].x as f32 * xx) as i32 + dx;
            outline.points[i].y = (outline.points[i].y as f32 * yy) as i32 + dy;
            i += 1;
        }
        if flags & 0x0020 == 0 {
            break;
        }
    }
    Ok(())
}

fn rasterize_outline(
    codepoint: u32,
    outline: &Outline,
    advance_units: u16,
    units_per_em: u16,
    pixel_size: u8,
) -> Result<RasterGlyph, TtfError> {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for point in outline.points.iter().copied() {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    if min_x >= max_x || min_y >= max_y {
        return Ok(empty_glyph(
            codepoint,
            advance_units,
            units_per_em,
            pixel_size,
        ));
    }
    let scale_num = pixel_size.max(1) as i32;
    let scale_den = units_per_em.max(1) as i32;
    let width = (((max_x - min_x) * scale_num + scale_den - 1) / scale_den).clamp(1, 64) as u8;
    let height = (((max_y - min_y) * scale_num + scale_den - 1) / scale_den).clamp(1, 64) as u8;
    let mut data = Vec::new();
    data.resize(width as usize * height as usize, 0);
    let mut py = 0u8;
    while py < height {
        let fy = max_y - ((py as i32 * scale_den) / scale_num);
        let mut px = 0u8;
        while px < width {
            let fx = min_x + ((px as i32 * scale_den) / scale_num);
            if point_in_outline(outline, fx, fy) {
                data[py as usize * width as usize + px as usize] = 255;
            }
            px += 1;
        }
        py += 1;
    }
    Ok(RasterGlyph {
        codepoint,
        width,
        height,
        advance: scale_advance(advance_units, units_per_em, pixel_size),
        bearing_x: ((min_x * scale_num) / scale_den).clamp(i8::MIN as i32, i8::MAX as i32) as i8,
        bearing_y: ((max_y * scale_num) / scale_den).clamp(i8::MIN as i32, i8::MAX as i32) as i8,
        data,
    })
}

fn point_in_outline(outline: &Outline, x: i32, y: i32) -> bool {
    let mut inside = false;
    let mut start = 0usize;
    for end in outline.contours.iter().copied() {
        let mut i = start;
        while i <= end {
            let a = outline.points[i];
            let b = outline.points[if i == end { start } else { i + 1 }];
            if (a.y > y) != (b.y > y) {
                let cross = (b.x - a.x) as i64 * (y - a.y) as i64 / (b.y - a.y) as i64 + a.x as i64;
                if x as i64 <= cross {
                    inside = !inside;
                }
            }
            i += 1;
        }
        start = end + 1;
    }
    inside
}

fn empty_glyph(
    codepoint: u32,
    advance_units: u16,
    units_per_em: u16,
    pixel_size: u8,
) -> RasterGlyph {
    RasterGlyph {
        codepoint,
        width: 1,
        height: pixel_size.max(1),
        advance: scale_advance(advance_units, units_per_em, pixel_size),
        bearing_x: 0,
        bearing_y: 0,
        data: {
            let mut data = Vec::new();
            data.resize(pixel_size.max(1) as usize, 0);
            data
        },
    }
}

struct CffFont<'a> {
    charstrings: Vec<&'a [u8]>,
    local_subrs: Vec<&'a [u8]>,
    global_subrs: Vec<&'a [u8]>,
}

#[derive(Default)]
struct CffDict {
    charstrings_offset: usize,
    private_offset: usize,
    private_len: usize,
    local_subrs_offset: usize,
}

struct Type2State {
    x: i32,
    y: i32,
    contour_start: Option<usize>,
    hints: usize,
}

fn rasterize_cff_glyph(
    face: &FontFace<'_>,
    codepoint: u32,
    glyph_id: u16,
    advance_units: u16,
    pixel_size: u8,
) -> Result<RasterGlyph, TtfError> {
    let cff = parse_cff_font(face)?;
    let mut outline = Outline::new();
    let mut state = Type2State {
        x: 0,
        y: 0,
        contour_start: None,
        hints: 0,
    };
    execute_type2_charstring(&cff, glyph_id as usize, &mut outline, &mut state, 0)?;
    close_type2_contour(&mut outline, &mut state);
    if outline.points.is_empty() || outline.contours.is_empty() {
        return Ok(empty_glyph(
            codepoint,
            advance_units,
            face.info.units_per_em,
            pixel_size,
        ));
    }
    rasterize_outline(
        codepoint,
        &outline,
        advance_units,
        face.info.units_per_em,
        pixel_size,
    )
}

fn parse_cff_font<'a>(face: &FontFace<'a>) -> Result<CffFont<'a>, TtfError> {
    let table = face
        .data
        .get(face.tables.cff.offset..face.tables.cff.offset + face.tables.cff.len)
        .ok_or(TtfError::Truncated)?;
    if table.len() < 4 {
        return Err(TtfError::Truncated);
    }
    let header_len = table[2] as usize;
    if header_len > table.len() {
        return Err(TtfError::Truncated);
    }
    let (pos, _names) = read_cff_index(table, header_len)?;
    let (pos, top_dicts) = read_cff_index(table, pos)?;
    let top = top_dicts.first().copied().ok_or(TtfError::MissingTable)?;
    let mut dict = parse_cff_dict(top)?;
    let (pos, _strings) = read_cff_index(table, pos)?;
    let (_pos, global_subrs) = read_cff_index(table, pos)?;
    if dict.charstrings_offset == 0 || dict.charstrings_offset >= table.len() {
        return Err(TtfError::MissingTable);
    }
    let (_end, charstrings) = read_cff_index(table, dict.charstrings_offset)?;
    let mut local_subrs = Vec::new();
    if dict.private_len != 0
        && dict
            .private_offset
            .checked_add(dict.private_len)
            .ok_or(TtfError::Truncated)?
            <= table.len()
    {
        let private = table
            .get(dict.private_offset..dict.private_offset + dict.private_len)
            .ok_or(TtfError::Truncated)?;
        let private_dict = parse_cff_dict(private)?;
        dict.local_subrs_offset = private_dict.local_subrs_offset;
        if dict.local_subrs_offset != 0 {
            let subr_offset = dict
                .private_offset
                .checked_add(dict.local_subrs_offset)
                .ok_or(TtfError::Truncated)?;
            if subr_offset < table.len() {
                let (_end, subrs) = read_cff_index(table, subr_offset)?;
                local_subrs = subrs;
            }
        }
    }
    Ok(CffFont {
        charstrings,
        local_subrs,
        global_subrs,
    })
}

fn read_cff_index<'a>(
    data: &'a [u8],
    mut offset: usize,
) -> Result<(usize, Vec<&'a [u8]>), TtfError> {
    let count = read_u16(data, offset).ok_or(TtfError::Truncated)? as usize;
    offset += 2;
    let mut items = Vec::new();
    if count == 0 {
        return Ok((offset, items));
    }
    let off_size = *data.get(offset).ok_or(TtfError::Truncated)? as usize;
    if off_size == 0 || off_size > 4 {
        return Err(TtfError::Unsupported);
    }
    offset += 1;
    let mut offsets = Vec::new();
    let mut i = 0usize;
    while i <= count {
        let mut value = 0usize;
        let mut b = 0usize;
        while b < off_size {
            value = (value << 8) | *data.get(offset + b).ok_or(TtfError::Truncated)? as usize;
            b += 1;
        }
        offsets.push(value);
        offset += off_size;
        i += 1;
    }
    let data_start = offset;
    let mut n = 0usize;
    while n < count {
        let start = data_start
            .checked_add(offsets[n].saturating_sub(1))
            .ok_or(TtfError::Truncated)?;
        let end = data_start
            .checked_add(offsets[n + 1].saturating_sub(1))
            .ok_or(TtfError::Truncated)?;
        items.push(data.get(start..end).ok_or(TtfError::Truncated)?);
        n += 1;
    }
    Ok((data_start + offsets[count].saturating_sub(1), items))
}

fn parse_cff_dict(data: &[u8]) -> Result<CffDict, TtfError> {
    let mut dict = CffDict::default();
    let mut stack = Vec::<i32>::new();
    let mut offset = 0usize;
    while offset < data.len() {
        let byte = data[offset];
        offset += 1;
        if byte <= 21 {
            let op = if byte == 12 {
                let escaped = *data.get(offset).ok_or(TtfError::Truncated)?;
                offset += 1;
                0x0c00u16 | escaped as u16
            } else {
                byte as u16
            };
            match op {
                17 => {
                    if let Some(value) = stack.last().copied() {
                        dict.charstrings_offset = value.max(0) as usize;
                    }
                }
                18 => {
                    if stack.len() >= 2 {
                        dict.private_len = stack[stack.len() - 2].max(0) as usize;
                        dict.private_offset = stack[stack.len() - 1].max(0) as usize;
                    }
                }
                19 => {
                    if let Some(value) = stack.last().copied() {
                        dict.local_subrs_offset = value.max(0) as usize;
                    }
                }
                _ => {}
            }
            stack.clear();
        } else if let Some(value) = read_cff_number(data, byte, &mut offset)? {
            if stack.len() >= TYPE2_STACK_LIMIT {
                return Err(TtfError::Unsupported);
            }
            stack.push(value);
        } else {
            return Err(TtfError::Unsupported);
        }
    }
    Ok(dict)
}

fn read_cff_number(data: &[u8], byte: u8, offset: &mut usize) -> Result<Option<i32>, TtfError> {
    Ok(Some(match byte {
        28 => {
            let value = read_i16(data, *offset).ok_or(TtfError::Truncated)? as i32;
            *offset += 2;
            value
        }
        29 => {
            let value = read_u32(data, *offset).ok_or(TtfError::Truncated)? as i32;
            *offset += 4;
            value
        }
        30 => return Ok(None),
        32..=246 => byte as i32 - 139,
        247..=250 => {
            let b1 = *data.get(*offset).ok_or(TtfError::Truncated)? as i32;
            *offset += 1;
            (byte as i32 - 247) * 256 + b1 + 108
        }
        251..=254 => {
            let b1 = *data.get(*offset).ok_or(TtfError::Truncated)? as i32;
            *offset += 1;
            -((byte as i32 - 251) * 256) - b1 - 108
        }
        _ => return Ok(None),
    }))
}

fn execute_type2_charstring(
    cff: &CffFont<'_>,
    glyph_id: usize,
    outline: &mut Outline,
    state: &mut Type2State,
    depth: u8,
) -> Result<(), TtfError> {
    if depth >= TYPE2_SUBR_DEPTH_LIMIT {
        return Err(TtfError::Unsupported);
    }
    let bytes = *cff
        .charstrings
        .get(glyph_id)
        .ok_or(TtfError::InvalidGlyph)?;
    execute_type2_bytes(cff, bytes, outline, state, depth)
}

fn execute_type2_subr(
    cff: &CffFont<'_>,
    subrs: &[&[u8]],
    operand: i32,
    outline: &mut Outline,
    state: &mut Type2State,
    depth: u8,
) -> Result<(), TtfError> {
    if depth >= TYPE2_SUBR_DEPTH_LIMIT {
        return Err(TtfError::Unsupported);
    }
    let index = operand + cff_subr_bias(subrs.len());
    if index < 0 {
        return Err(TtfError::InvalidGlyph);
    }
    let bytes = *subrs.get(index as usize).ok_or(TtfError::InvalidGlyph)?;
    execute_type2_bytes(cff, bytes, outline, state, depth + 1)
}

fn execute_type2_bytes(
    cff: &CffFont<'_>,
    bytes: &[u8],
    outline: &mut Outline,
    state: &mut Type2State,
    depth: u8,
) -> Result<(), TtfError> {
    let mut offset = 0usize;
    let mut stack = Vec::<i32>::new();
    while offset < bytes.len() {
        let byte = bytes[offset];
        offset += 1;
        if byte == 28 || byte >= 32 {
            let value = read_type2_number(bytes, byte, &mut offset)?;
            if stack.len() >= TYPE2_STACK_LIMIT {
                return Err(TtfError::Unsupported);
            }
            stack.push(value);
            continue;
        }
        match byte {
            1 | 3 | 18 | 23 => {
                state.hints = state.hints.saturating_add(stack.len() / 2);
                stack.clear();
            }
            4 => {
                let dy = pop_i32(&mut stack)?;
                type2_move(outline, state, 0, dy);
                stack.clear();
            }
            5 => {
                let mut i = 0usize;
                while i + 1 < stack.len() {
                    type2_line(outline, state, stack[i], stack[i + 1]);
                    i += 2;
                }
                stack.clear();
            }
            6 | 7 => {
                let mut horizontal = byte == 6;
                for value in stack.iter().copied() {
                    if horizontal {
                        type2_line(outline, state, value, 0);
                    } else {
                        type2_line(outline, state, 0, value);
                    }
                    horizontal = !horizontal;
                }
                stack.clear();
            }
            8 => {
                let mut i = 0usize;
                while i + 5 < stack.len() {
                    type2_curve(
                        outline,
                        state,
                        stack[i],
                        stack[i + 1],
                        stack[i + 2],
                        stack[i + 3],
                        stack[i + 4],
                        stack[i + 5],
                    );
                    i += 6;
                }
                stack.clear();
            }
            10 => {
                let operand = pop_i32(&mut stack)?;
                execute_type2_subr(cff, &cff.local_subrs, operand, outline, state, depth)?;
                stack.clear();
            }
            11 => return Ok(()),
            12 => {
                let escaped = *bytes.get(offset).ok_or(TtfError::Truncated)?;
                offset += 1;
                execute_type2_escaped(escaped, &mut stack, outline, state)?;
            }
            14 => {
                close_type2_contour(outline, state);
                return Ok(());
            }
            19 | 20 => {
                let mask_bytes = state.hints.saturating_add(7) / 8;
                offset = offset.checked_add(mask_bytes).ok_or(TtfError::Truncated)?;
                if offset > bytes.len() {
                    return Err(TtfError::Truncated);
                }
                stack.clear();
            }
            21 => {
                if stack.len() < 2 {
                    return Err(TtfError::InvalidGlyph);
                }
                let dx = stack[stack.len() - 2];
                let dy = stack[stack.len() - 1];
                type2_move(outline, state, dx, dy);
                stack.clear();
            }
            22 => {
                let dx = pop_i32(&mut stack)?;
                type2_move(outline, state, dx, 0);
                stack.clear();
            }
            24 => {
                while stack.len() > 2 {
                    let args = take_six(&mut stack)?;
                    type2_curve(
                        outline, state, args[0], args[1], args[2], args[3], args[4], args[5],
                    );
                }
                if stack.len() == 2 {
                    type2_line(outline, state, stack[0], stack[1]);
                }
                stack.clear();
            }
            25 => {
                while stack.len() > 6 {
                    let dx = stack.remove(0);
                    let dy = stack.remove(0);
                    type2_line(outline, state, dx, dy);
                }
                if stack.len() == 6 {
                    type2_curve(
                        outline, state, stack[0], stack[1], stack[2], stack[3], stack[4], stack[5],
                    );
                }
                stack.clear();
            }
            26 | 27 => {
                execute_type2_hv_curve(byte == 27, &stack, outline, state)?;
                stack.clear();
            }
            29 => {
                let operand = pop_i32(&mut stack)?;
                execute_type2_subr(cff, &cff.global_subrs, operand, outline, state, depth)?;
                stack.clear();
            }
            30 | 31 => {
                execute_type2_alternating_curve(byte == 31, &stack, outline, state)?;
                stack.clear();
            }
            _ => return Err(TtfError::Unsupported),
        }
    }
    Ok(())
}

fn execute_type2_escaped(
    op: u8,
    stack: &mut Vec<i32>,
    outline: &mut Outline,
    state: &mut Type2State,
) -> Result<(), TtfError> {
    match op {
        34 | 35 | 36 | 37 => {
            // flex operators are approximated by the final curve endpoints.
            let mut i = 0usize;
            while i + 5 < stack.len() {
                type2_curve(
                    outline,
                    state,
                    stack[i],
                    stack[i + 1],
                    stack[i + 2],
                    stack[i + 3],
                    stack[i + 4],
                    stack[i + 5],
                );
                i += 6;
            }
            stack.clear();
            Ok(())
        }
        _ => {
            stack.clear();
            Ok(())
        }
    }
}

fn execute_type2_hv_curve(
    horizontal_first: bool,
    stack: &[i32],
    outline: &mut Outline,
    state: &mut Type2State,
) -> Result<(), TtfError> {
    let mut i = 0usize;
    let mut horizontal = horizontal_first;
    let mut first_extra = 0i32;
    if stack.len() % 4 == 1 {
        first_extra = stack[0];
        i = 1;
    }
    while i + 3 < stack.len() {
        if horizontal {
            type2_curve(
                outline,
                state,
                stack[i],
                first_extra,
                stack[i + 1],
                stack[i + 2],
                stack[i + 3],
                0,
            );
        } else {
            type2_curve(
                outline,
                state,
                first_extra,
                stack[i],
                stack[i + 1],
                stack[i + 2],
                0,
                stack[i + 3],
            );
        }
        first_extra = 0;
        horizontal = !horizontal;
        i += 4;
    }
    Ok(())
}

fn execute_type2_alternating_curve(
    horizontal_first: bool,
    stack: &[i32],
    outline: &mut Outline,
    state: &mut Type2State,
) -> Result<(), TtfError> {
    let mut i = 0usize;
    let mut horizontal = horizontal_first;
    while i + 3 < stack.len() {
        let last = if i + 4 == stack.len() - 1 {
            stack[i + 4]
        } else {
            0
        };
        if horizontal {
            type2_curve(
                outline,
                state,
                stack[i],
                0,
                stack[i + 1],
                stack[i + 2],
                last,
                stack[i + 3],
            );
        } else {
            type2_curve(
                outline,
                state,
                0,
                stack[i],
                stack[i + 1],
                stack[i + 2],
                stack[i + 3],
                last,
            );
        }
        horizontal = !horizontal;
        i += if i + 4 == stack.len() - 1 { 5 } else { 4 };
    }
    Ok(())
}

fn read_type2_number(bytes: &[u8], byte: u8, offset: &mut usize) -> Result<i32, TtfError> {
    Ok(match byte {
        28 => {
            let value = read_i16(bytes, *offset).ok_or(TtfError::Truncated)? as i32;
            *offset += 2;
            value
        }
        32..=246 => byte as i32 - 139,
        247..=250 => {
            let b1 = *bytes.get(*offset).ok_or(TtfError::Truncated)? as i32;
            *offset += 1;
            (byte as i32 - 247) * 256 + b1 + 108
        }
        251..=254 => {
            let b1 = *bytes.get(*offset).ok_or(TtfError::Truncated)? as i32;
            *offset += 1;
            -((byte as i32 - 251) * 256) - b1 - 108
        }
        255 => {
            let raw = read_u32(bytes, *offset).ok_or(TtfError::Truncated)? as i32;
            *offset += 4;
            raw >> 16
        }
        _ => return Err(TtfError::Unsupported),
    })
}

fn pop_i32(stack: &mut Vec<i32>) -> Result<i32, TtfError> {
    stack.pop().ok_or(TtfError::InvalidGlyph)
}

fn take_six(stack: &mut Vec<i32>) -> Result<[i32; 6], TtfError> {
    if stack.len() < 6 {
        return Err(TtfError::InvalidGlyph);
    }
    Ok([
        stack.remove(0),
        stack.remove(0),
        stack.remove(0),
        stack.remove(0),
        stack.remove(0),
        stack.remove(0),
    ])
}

fn type2_move(outline: &mut Outline, state: &mut Type2State, dx: i32, dy: i32) {
    close_type2_contour(outline, state);
    state.x += dx;
    state.y += dy;
    state.contour_start = Some(outline.points.len());
    outline.points.push(OutlinePoint {
        x: state.x,
        y: state.y,
        _on_curve: true,
    });
}

fn type2_line(outline: &mut Outline, state: &mut Type2State, dx: i32, dy: i32) {
    if state.contour_start.is_none() {
        state.contour_start = Some(outline.points.len());
    }
    state.x += dx;
    state.y += dy;
    outline.points.push(OutlinePoint {
        x: state.x,
        y: state.y,
        _on_curve: true,
    });
}

fn type2_curve(
    outline: &mut Outline,
    state: &mut Type2State,
    dx1: i32,
    dy1: i32,
    dx2: i32,
    dy2: i32,
    dx3: i32,
    dy3: i32,
) {
    let x0 = state.x;
    let y0 = state.y;
    let x1 = x0 + dx1;
    let y1 = y0 + dy1;
    let x2 = x1 + dx2;
    let y2 = y1 + dy2;
    let x3 = x2 + dx3;
    let y3 = y2 + dy3;
    let mut step = 1i32;
    while step <= 4 {
        let t = step;
        let inv = 4 - step;
        let x =
            (inv * inv * inv * x0 + 3 * inv * inv * t * x1 + 3 * inv * t * t * x2 + t * t * t * x3)
                / 64;
        let y =
            (inv * inv * inv * y0 + 3 * inv * inv * t * y1 + 3 * inv * t * t * y2 + t * t * t * y3)
                / 64;
        type2_line(outline, state, x - state.x, y - state.y);
        step += 1;
    }
    state.x = x3;
    state.y = y3;
}

fn close_type2_contour(outline: &mut Outline, state: &mut Type2State) {
    if let Some(start) = state.contour_start {
        if outline.points.len() > start {
            outline.contours.push(outline.points.len() - 1);
        }
    }
    state.contour_start = None;
}

fn cff_subr_bias(count: usize) -> i32 {
    if count < 1240 {
        107
    } else if count < 33900 {
        1131
    } else {
        32768
    }
}

fn cff_fallback_glyph(
    codepoint: u32,
    advance_units: u16,
    units_per_em: u16,
    pixel_size: u8,
) -> RasterGlyph {
    let height = pixel_size.max(8).min(48);
    let width = (height / 2).max(4);
    let mut data = Vec::new();
    data.resize(width as usize * height as usize, 0);
    let mut y = 0u8;
    while y < height {
        let mut x = 0u8;
        while x < width {
            let border = x == 0 || y == 0 || x + 1 == width || y + 1 == height;
            let diagonal = x as u16 * height as u16 / width as u16 == y as u16;
            if border || diagonal {
                data[y as usize * width as usize + x as usize] = 220;
            }
            x += 1;
        }
        y += 1;
    }
    RasterGlyph {
        codepoint,
        width,
        height,
        advance: scale_advance(advance_units, units_per_em, pixel_size),
        bearing_x: 0,
        bearing_y: height.min(i8::MAX as u8) as i8,
        data,
    }
}

fn scale_advance(advance_units: u16, units_per_em: u16, pixel_size: u8) -> u8 {
    ((advance_units as u32 * pixel_size.max(1) as u32) / units_per_em.max(1) as u32)
        .clamp(1, u8::MAX as u32) as u8
}

fn cmap_lookup(data: &[u8], cmap: Table, codepoint: u32) -> Option<u16> {
    if codepoint > 0xffff {
        return cmap_format12_lookup(data, cmap, codepoint);
    }
    cmap_format4_lookup(data, cmap, codepoint)
        .or_else(|| cmap_format12_lookup(data, cmap, codepoint))
}

fn cmap_format4_lookup(data: &[u8], cmap: Table, codepoint: u32) -> Option<u16> {
    if codepoint > 0xffff {
        return None;
    }
    let cmap_data = data.get(cmap.offset..cmap.offset + cmap.len)?;
    let tables = read_u16(cmap_data, 2)? as usize;
    let mut chosen = None;
    let mut i = 0usize;
    while i < tables {
        let base = 4 + i * 8;
        let platform = read_u16(cmap_data, base)?;
        let encoding = read_u16(cmap_data, base + 2)?;
        let offset = read_u32(cmap_data, base + 4)? as usize;
        if (platform == 3 && (encoding == 1 || encoding == 10)) || platform == 0 {
            if read_u16(cmap_data, offset)? == 4 {
                chosen = Some(offset);
                break;
            }
        }
        i += 1;
    }
    let offset = chosen?;
    let sub = cmap_data.get(offset..)?;
    let seg_count = read_u16(sub, 6)? as usize / 2;
    let end_codes = 14usize;
    let start_codes = end_codes + seg_count * 2 + 2;
    let id_deltas = start_codes + seg_count * 2;
    let id_range_offsets = id_deltas + seg_count * 2;
    let cp = codepoint as u16;
    let mut s = 0usize;
    while s < seg_count {
        let end = read_u16(sub, end_codes + s * 2)?;
        let start = read_u16(sub, start_codes + s * 2)?;
        if cp >= start && cp <= end {
            let delta = read_i16(sub, id_deltas + s * 2)? as i32;
            let ro = read_u16(sub, id_range_offsets + s * 2)? as usize;
            if ro == 0 {
                return Some(cp.wrapping_add(delta as u16));
            }
            let glyph_offset = id_range_offsets + s * 2 + ro + (cp - start) as usize * 2;
            let glyph = read_u16(sub, glyph_offset)?;
            if glyph == 0 {
                return Some(0);
            }
            return Some(glyph.wrapping_add(delta as u16));
        }
        s += 1;
    }
    None
}

fn cmap_format12_lookup(data: &[u8], cmap: Table, codepoint: u32) -> Option<u16> {
    let cmap_data = data.get(cmap.offset..cmap.offset + cmap.len)?;
    let tables = read_u16(cmap_data, 2)? as usize;
    let mut chosen = None;
    let mut i = 0usize;
    while i < tables {
        let base = 4 + i * 8;
        let platform = read_u16(cmap_data, base)?;
        let encoding = read_u16(cmap_data, base + 2)?;
        let offset = read_u32(cmap_data, base + 4)? as usize;
        if ((platform == 3 && encoding == 10) || platform == 0)
            && read_u16(cmap_data, offset)? == 12
        {
            chosen = Some(offset);
            break;
        }
        i += 1;
    }
    let sub = cmap_data.get(chosen?..)?;
    let length = read_u32(sub, 4)? as usize;
    if length > sub.len() {
        return None;
    }
    let groups = read_u32(sub, 12)? as usize;
    let mut g = 0usize;
    while g < groups {
        let base = 16 + g * 12;
        let start = read_u32(sub, base)?;
        let end = read_u32(sub, base + 4)?;
        let start_glyph = read_u32(sub, base + 8)?;
        if codepoint >= start && codepoint <= end {
            let glyph = start_glyph.checked_add(codepoint.checked_sub(start)?)?;
            return if glyph <= u16::MAX as u32 {
                Some(glyph as u16)
            } else {
                None
            };
        }
        g += 1;
    }
    None
}

fn count_kern_pairs(data: &[u8], kern: Table) -> Option<u16> {
    if kern.len < 14 {
        return None;
    }
    let table = data.get(kern.offset..kern.offset + kern.len)?;
    let subtables = read_u16(table, 2)? as usize;
    let mut offset = 4usize;
    let mut total = 0u16;
    let mut i = 0usize;
    while i < subtables && offset + 6 <= table.len() {
        let length = read_u16(table, offset + 2)? as usize;
        let coverage = read_u16(table, offset + 4)?;
        if length < 6 || offset + length > table.len() {
            return None;
        }
        if coverage & 0xff == 0 {
            total = total.saturating_add(read_u16(table, offset + 6).unwrap_or(0));
        }
        offset += length;
        i += 1;
    }
    Some(total)
}

fn kern_format0_lookup(data: &[u8], kern: Table, left_glyph: u16, right_glyph: u16) -> Option<i16> {
    if kern.len < 14 {
        return None;
    }
    let table = data.get(kern.offset..kern.offset + kern.len)?;
    let subtables = read_u16(table, 2)? as usize;
    let mut offset = 4usize;
    let mut s = 0usize;
    while s < subtables && offset + 14 <= table.len() {
        let length = read_u16(table, offset + 2)? as usize;
        let coverage = read_u16(table, offset + 4)?;
        if length < 14 || offset + length > table.len() {
            return None;
        }
        if coverage & 0xff == 0 {
            let pairs = read_u16(table, offset + 6)? as usize;
            let mut i = 0usize;
            while i < pairs {
                let base = offset + 14 + i * 6;
                if base + 6 > offset + length {
                    return None;
                }
                let left = read_u16(table, base)?;
                let right = read_u16(table, base + 2)?;
                if left == left_glyph && right == right_glyph {
                    return read_i16(table, base + 4);
                }
                i += 1;
            }
        }
        offset += length;
        s += 1;
    }
    None
}

fn gsub_single_substitute(data: &[u8], gsub: Table, glyph: u16) -> Option<u16> {
    for_layout_lookup(data, gsub, 1, |table, subtable| {
        let format = read_u16(table, subtable)?;
        let coverage = subtable.checked_add(read_u16(table, subtable + 2)? as usize)?;
        let index = coverage_index(table, coverage, glyph)?;
        match format {
            1 => Some(glyph.wrapping_add(read_i16(table, subtable + 4)? as u16)),
            2 => {
                let count = read_u16(table, subtable + 4)? as usize;
                if index < count {
                    read_u16(table, subtable + 6 + index * 2)
                } else {
                    None
                }
            }
            _ => None,
        }
    })
}

fn gsub_ligature_substitute(data: &[u8], gsub: Table, first: u16, second: u16) -> Option<u16> {
    for_layout_lookup(data, gsub, 4, |table, subtable| {
        let format = read_u16(table, subtable)?;
        if format != 1 {
            return None;
        }
        let coverage = subtable.checked_add(read_u16(table, subtable + 2)? as usize)?;
        let coverage_index = coverage_index(table, coverage, first)?;
        let set_count = read_u16(table, subtable + 4)? as usize;
        if coverage_index >= set_count {
            return None;
        }
        let set =
            subtable.checked_add(read_u16(table, subtable + 6 + coverage_index * 2)? as usize)?;
        let lig_count = read_u16(table, set)? as usize;
        let mut i = 0usize;
        while i < lig_count {
            let lig = set.checked_add(read_u16(table, set + 2 + i * 2)? as usize)?;
            let lig_glyph = read_u16(table, lig)?;
            let comp_count = read_u16(table, lig + 2)? as usize;
            if comp_count == 2 && read_u16(table, lig + 4)? == second {
                return Some(lig_glyph);
            }
            i += 1;
        }
        None
    })
}

fn gpos_pair_adjust(data: &[u8], gpos: Table, left: u16, right: u16) -> i16 {
    for_layout_lookup(data, gpos, 2, |table, subtable| {
        let format = read_u16(table, subtable)?;
        match format {
            1 => gpos_pair_pos_format1(table, subtable, left, right),
            2 => gpos_pair_pos_format2(table, subtable, left, right),
            _ => None,
        }
    })
    .unwrap_or(0)
}

fn for_layout_lookup<T, F>(data: &[u8], table: Table, lookup_type: u16, mut f: F) -> Option<T>
where
    F: FnMut(&[u8], usize) -> Option<T>,
{
    if table.len < 10 {
        return None;
    }
    let layout = data.get(table.offset..table.offset + table.len)?;
    let lookup_list = read_u16(layout, 8)? as usize;
    let count = read_u16(layout, lookup_list)? as usize;
    let mut i = 0usize;
    while i < count {
        let lookup = lookup_list.checked_add(read_u16(layout, lookup_list + 2 + i * 2)? as usize)?;
        if read_u16(layout, lookup)? == lookup_type {
            let sub_count = read_u16(layout, lookup + 4)? as usize;
            let mut s = 0usize;
            while s < sub_count {
                let subtable = lookup.checked_add(read_u16(layout, lookup + 6 + s * 2)? as usize)?;
                if let Some(value) = f(layout, subtable) {
                    return Some(value);
                }
                s += 1;
            }
        }
        i += 1;
    }
    None
}

fn coverage_index(table: &[u8], offset: usize, glyph: u16) -> Option<usize> {
    match read_u16(table, offset)? {
        1 => {
            let count = read_u16(table, offset + 2)? as usize;
            let mut i = 0usize;
            while i < count {
                if read_u16(table, offset + 4 + i * 2)? == glyph {
                    return Some(i);
                }
                i += 1;
            }
            None
        }
        2 => {
            let count = read_u16(table, offset + 2)? as usize;
            let mut i = 0usize;
            while i < count {
                let base = offset + 4 + i * 6;
                let start = read_u16(table, base)?;
                let end = read_u16(table, base + 2)?;
                let coverage_start = read_u16(table, base + 4)? as usize;
                if glyph >= start && glyph <= end {
                    return Some(coverage_start + (glyph - start) as usize);
                }
                i += 1;
            }
            None
        }
        _ => None,
    }
}

fn gpos_pair_pos_format1(table: &[u8], subtable: usize, left: u16, right: u16) -> Option<i16> {
    let coverage = subtable.checked_add(read_u16(table, subtable + 2)? as usize)?;
    let left_index = coverage_index(table, coverage, left)?;
    let value_format1 = read_u16(table, subtable + 4)?;
    let value_format2 = read_u16(table, subtable + 6)?;
    let pair_set_count = read_u16(table, subtable + 8)? as usize;
    if left_index >= pair_set_count {
        return None;
    }
    let pair_set =
        subtable.checked_add(read_u16(table, subtable + 10 + left_index * 2)? as usize)?;
    let pair_count = read_u16(table, pair_set)? as usize;
    let value_size1 = value_record_size(value_format1);
    let value_size2 = value_record_size(value_format2);
    let record_size = 2usize.checked_add(value_size1)?.checked_add(value_size2)?;
    let mut i = 0usize;
    while i < pair_count {
        let record = pair_set + 2 + i * record_size;
        if read_u16(table, record)? == right {
            return read_value_x_advance(table, record + 2, value_format1);
        }
        i += 1;
    }
    None
}

fn gpos_pair_pos_format2(table: &[u8], subtable: usize, left: u16, right: u16) -> Option<i16> {
    let coverage = subtable.checked_add(read_u16(table, subtable + 2)? as usize)?;
    coverage_index(table, coverage, left)?;
    let value_format1 = read_u16(table, subtable + 4)?;
    let value_format2 = read_u16(table, subtable + 6)?;
    let class_def1 = subtable.checked_add(read_u16(table, subtable + 8)? as usize)?;
    let class_def2 = subtable.checked_add(read_u16(table, subtable + 10)? as usize)?;
    let class1_count = read_u16(table, subtable + 12)? as usize;
    let class2_count = read_u16(table, subtable + 14)? as usize;
    let class1 = class_index(table, class_def1, left)? as usize;
    let class2 = class_index(table, class_def2, right)? as usize;
    if class1 >= class1_count || class2 >= class2_count {
        return None;
    }
    let value_size1 = value_record_size(value_format1);
    let value_size2 = value_record_size(value_format2);
    let record_size = value_size1.checked_add(value_size2)?;
    let record = subtable + 16 + (class1 * class2_count + class2) * record_size;
    read_value_x_advance(table, record, value_format1)
}

fn class_index(table: &[u8], offset: usize, glyph: u16) -> Option<u16> {
    match read_u16(table, offset)? {
        1 => {
            let start = read_u16(table, offset + 2)?;
            let count = read_u16(table, offset + 4)? as usize;
            let index = glyph.checked_sub(start)? as usize;
            if index < count {
                read_u16(table, offset + 6 + index * 2)
            } else {
                Some(0)
            }
        }
        2 => {
            let count = read_u16(table, offset + 2)? as usize;
            let mut i = 0usize;
            while i < count {
                let base = offset + 4 + i * 6;
                let start = read_u16(table, base)?;
                let end = read_u16(table, base + 2)?;
                if glyph >= start && glyph <= end {
                    return read_u16(table, base + 4);
                }
                i += 1;
            }
            Some(0)
        }
        _ => None,
    }
}

fn value_record_size(format: u16) -> usize {
    let mut size = 0usize;
    let mut bit = 0u16;
    while bit < 8 {
        if (format & (1 << bit)) != 0 {
            size += 2;
        }
        bit += 1;
    }
    size
}

fn read_value_x_advance(table: &[u8], offset: usize, format: u16) -> Option<i16> {
    let mut cursor = offset;
    let mut bit = 0u16;
    while bit < 8 {
        if (format & (1 << bit)) != 0 {
            let value = read_i16(table, cursor)?;
            if bit == 2 {
                return Some(value);
            }
            cursor += 2;
        }
        bit += 1;
    }
    Some(0)
}

fn f2dot14(value: i16) -> f32 {
    value as f32 / 16384.0
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
    ]))
}

fn read_i16(bytes: &[u8], offset: usize) -> Option<i16> {
    Some(i16::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
    ]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
        *bytes.get(offset + 2)?,
        *bytes.get(offset + 3)?,
    ]))
}
