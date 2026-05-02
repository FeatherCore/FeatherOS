use crate::{
    backend::{CodecAcceleratorCapabilities, CodecPipelinePlan, CodecStageKind, CodecStagePlan},
    png::DecodedImage,
    ImageFormat,
};
use alloc::vec::Vec;

const JPEG_SOI: &[u8; 2] = b"\xff\xd8";
const JPEG_EOI: u8 = 0xd9;
const JPEG_SOF0: u8 = 0xc0;
const JPEG_SOF2: u8 = 0xc2;
const JPEG_DHT: u8 = 0xc4;
const JPEG_DQT: u8 = 0xdb;
const JPEG_DRI: u8 = 0xdd;
const JPEG_SOS: u8 = 0xda;
const JPEG_APP1: u8 = 0xe1;
const JPEG_APP14: u8 = 0xee;
const PROGRESSIVE_COEFF_BUDGET_BYTES: usize = 2 * 1024 * 1024;

const ZIGZAG: [usize; 64] = [
    0, 1, 8, 16, 9, 2, 3, 10, 17, 24, 32, 25, 18, 11, 4, 5, 12, 19, 26, 33, 40, 48, 41, 34, 27, 20,
    13, 6, 7, 14, 21, 28, 35, 42, 49, 56, 57, 50, 43, 36, 29, 22, 15, 23, 30, 37, 44, 51, 58, 59,
    52, 45, 38, 31, 39, 46, 53, 60, 61, 54, 47, 55, 62, 63,
];

const IDCT_BASIS: [[i32; 8]; 8] = [
    [2896, 2896, 2896, 2896, 2896, 2896, 2896, 2896],
    [4017, 3406, 2276, 799, -799, -2276, -3406, -4017],
    [3784, 1567, -1567, -3784, -3784, -1567, 1567, 3784],
    [3406, -799, -4017, -2276, 2276, 4017, 799, -3406],
    [2896, -2896, -2896, 2896, 2896, -2896, -2896, 2896],
    [2276, -4017, 799, 3406, -3406, -799, 4017, -2276],
    [1567, -3784, 3784, -1567, -1567, 3784, -3784, 1567],
    [799, -2276, 3406, -4017, 4017, -3406, 2276, -799],
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JpegInfo {
    pub width: u16,
    pub height: u16,
    pub components: u8,
    pub baseline: bool,
    pub progressive: bool,
    pub exif_orientation: u8,
    pub adobe_transform: Option<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JpegDecodeOptions {
    pub apply_exif_orientation: bool,
    pub allow_progressive: bool,
    pub allow_cmyk: bool,
}

impl JpegDecodeOptions {
    pub const DEFAULT: Self = Self {
        apply_exif_orientation: true,
        allow_progressive: true,
        allow_cmyk: true,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JpegError {
    BadSignature,
    Truncated,
    Unsupported,
    UnsupportedProgressive,
    UnsupportedRestart,
    UnexpectedRestartMarker,
    InvalidTable,
    Decode,
}

#[derive(Clone, Copy)]
struct Component {
    id: u8,
    h: u8,
    v: u8,
    tq: u8,
    td: u8,
    ta: u8,
}

#[derive(Clone, Copy)]
struct ScanComponent {
    index: usize,
    td: u8,
    ta: u8,
}

impl ScanComponent {
    const EMPTY: Self = Self {
        index: 0,
        td: 0,
        ta: 0,
    };
}

#[derive(Clone, Copy)]
struct ScanData<'a> {
    components: [ScanComponent; 4],
    component_count: usize,
    data: &'a [u8],
    spectral_start: u8,
    spectral_end: u8,
    successive_high: u8,
    successive_low: u8,
}

impl<'a> ScanData<'a> {
    const EMPTY: Self = Self {
        components: [ScanComponent::EMPTY; 4],
        component_count: 0,
        data: &[],
        spectral_start: 0,
        spectral_end: 0,
        successive_high: 0,
        successive_low: 0,
    };
}

impl Component {
    const EMPTY: Self = Self {
        id: 0,
        h: 0,
        v: 0,
        tq: 0,
        td: 0,
        ta: 0,
    };
}

#[derive(Clone, Copy)]
struct HuffEntry {
    code: u16,
    len: u8,
    symbol: u8,
}

impl HuffEntry {
    const EMPTY: Self = Self {
        code: 0,
        len: 0,
        symbol: 0,
    };
}

#[derive(Clone, Copy)]
struct HuffTable {
    entries: [HuffEntry; 256],
    len: usize,
    valid: bool,
}

impl HuffTable {
    const fn new() -> Self {
        Self {
            entries: [HuffEntry::EMPTY; 256],
            len: 0,
            valid: false,
        }
    }

    fn build(&mut self, counts: &[u8], symbols: &[u8]) -> Result<(), JpegError> {
        if counts.len() != 16 {
            return Err(JpegError::InvalidTable);
        }
        let mut total = 0usize;
        let mut i = 0;
        while i < 16 {
            total = total.saturating_add(counts[i] as usize);
            i += 1;
        }
        if total > symbols.len() || total > self.entries.len() {
            return Err(JpegError::InvalidTable);
        }

        self.len = 0;
        let mut code = 0u16;
        let mut symbol_index = 0usize;
        let mut bit_len = 1u8;
        while bit_len <= 16 {
            let count = counts[(bit_len - 1) as usize] as usize;
            let mut n = 0usize;
            while n < count {
                self.entries[self.len] = HuffEntry {
                    code,
                    len: bit_len,
                    symbol: symbols[symbol_index],
                };
                self.len += 1;
                symbol_index += 1;
                code = code.saturating_add(1);
                n += 1;
            }
            code <<= 1;
            bit_len += 1;
        }
        self.valid = true;
        Ok(())
    }

    fn decode(&self, bits: &mut BitReader<'_>) -> Result<u8, JpegError> {
        if !self.valid {
            return Err(JpegError::InvalidTable);
        }
        let mut code = 0u16;
        let mut len = 1u8;
        while len <= 16 {
            code = (code << 1) | bits.read_bit()? as u16;
            let mut i = 0usize;
            while i < self.len {
                let entry = self.entries[i];
                if entry.len == len && entry.code == code {
                    return Ok(entry.symbol);
                }
                i += 1;
            }
            len += 1;
        }
        Err(JpegError::Decode)
    }
}

struct JpegSource<'a> {
    width: u16,
    height: u16,
    components: [Component; 4],
    component_count: usize,
    quant: [[u16; 64]; 4],
    quant_valid: [bool; 4],
    huff_dc: [HuffTable; 4],
    huff_ac: [HuffTable; 4],
    scan: &'a [u8],
    scans: Vec<ScanData<'a>>,
    restart_interval: u16,
    progressive: bool,
    exif_orientation: u8,
    adobe_transform: Option<u8>,
}

impl<'a> JpegSource<'a> {
    fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            components: [Component::EMPTY; 4],
            component_count: 0,
            quant: [[0; 64]; 4],
            quant_valid: [false; 4],
            huff_dc: [HuffTable::new(); 4],
            huff_ac: [HuffTable::new(); 4],
            scan: &[],
            scans: Vec::new(),
            restart_interval: 0,
            progressive: false,
            exif_orientation: 1,
            adobe_transform: None,
        }
    }
}

pub struct JpegDecoder;

impl JpegDecoder {
    pub fn inspect(bytes: &[u8]) -> Option<JpegInfo> {
        Self::inspect_result(bytes).ok()
    }

    pub fn inspect_result(bytes: &[u8]) -> Result<JpegInfo, JpegError> {
        let source = parse_jpeg(bytes, false)?;
        Ok(JpegInfo {
            width: source.width,
            height: source.height,
            components: source.component_count as u8,
            baseline: !source.progressive,
            progressive: source.progressive,
            exif_orientation: source.exif_orientation,
            adobe_transform: source.adobe_transform,
        })
    }

    pub fn decode(bytes: &[u8]) -> Option<DecodedImage> {
        Self::decode_result(bytes).ok()
    }

    pub fn decode_result(bytes: &[u8]) -> Result<DecodedImage, JpegError> {
        Self::decode_with_options(bytes, JpegDecodeOptions::DEFAULT)
    }

    pub fn plan_pipeline<const STAGES: usize>() -> CodecPipelinePlan<STAGES> {
        Self::plan_pipeline_with_caps(CodecAcceleratorCapabilities::NONE)
    }

    pub fn plan_pipeline_with_caps<const STAGES: usize>(
        caps: CodecAcceleratorCapabilities,
    ) -> CodecPipelinePlan<STAGES> {
        let supported = caps.jpeg && caps.max_stages >= 8;
        let mut plan = CodecPipelinePlan::new();
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Read, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Inspect, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Header, true, supported));
        let _ = plan.push(CodecStagePlan::new(
            CodecStageKind::Entropy,
            true,
            supported,
        ));
        let _ = plan.push(CodecStagePlan::new(
            CodecStageKind::Transform,
            true,
            supported,
        ));
        let _ = plan.push(CodecStagePlan::new(
            CodecStageKind::ColorConvert,
            true,
            supported,
        ));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Pack, true, supported));
        let _ = plan.push(CodecStagePlan::new(
            CodecStageKind::CacheInsert,
            false,
            true,
        ));
        plan
    }

    pub fn decode_with_options(
        bytes: &[u8],
        options: JpegDecodeOptions,
    ) -> Result<DecodedImage, JpegError> {
        let source = parse_jpeg(bytes, true)?;
        if source.progressive && !options.allow_progressive {
            return Err(JpegError::UnsupportedProgressive);
        }
        if source.component_count == 4 && !options.allow_cmyk {
            return Err(JpegError::Unsupported);
        }
        let image = if source.progressive {
            decode_progressive(&source)?
        } else {
            decode_baseline(&source)?
        };
        if options.apply_exif_orientation {
            Ok(apply_exif_orientation(image, source.exif_orientation)
                .ok_or(JpegError::Unsupported)?)
        } else {
            Ok(image)
        }
    }
}

pub fn decode_jpeg(bytes: &[u8]) -> Option<DecodedImage> {
    JpegDecoder::decode(bytes)
}

fn parse_jpeg<'a>(bytes: &'a [u8], require_scan: bool) -> Result<JpegSource<'a>, JpegError> {
    if bytes.get(..2) != Some(JPEG_SOI) {
        return Err(JpegError::BadSignature);
    }

    let mut source = JpegSource::new();
    let mut offset = 2usize;
    while offset < bytes.len() {
        let marker = next_marker(bytes, &mut offset)?;
        if marker == JPEG_EOI {
            break;
        }
        if marker_is_standalone(marker) {
            continue;
        }
        let len = read_be_u16(bytes, offset).ok_or(JpegError::Truncated)? as usize;
        if len < 2 {
            return Err(JpegError::Truncated);
        }
        let start = offset.checked_add(2).ok_or(JpegError::Truncated)?;
        let end = offset.checked_add(len).ok_or(JpegError::Truncated)?;
        let segment = bytes.get(start..end).ok_or(JpegError::Truncated)?;
        offset = end;

        match marker {
            JPEG_APP1 => {
                if let Some(orientation) = parse_exif_orientation(segment) {
                    source.exif_orientation = orientation.clamp(1, 8);
                }
            }
            JPEG_APP14 => {
                source.adobe_transform = parse_adobe_transform(segment);
            }
            JPEG_DQT => parse_dqt(segment, &mut source)?,
            JPEG_DHT => parse_dht(segment, &mut source)?,
            JPEG_SOF0 => parse_sof(segment, &mut source, false)?,
            JPEG_SOF2 => parse_sof(segment, &mut source, true)?,
            JPEG_DRI => {
                source.restart_interval = read_be_u16(segment, 0).ok_or(JpegError::Truncated)?;
            }
            JPEG_SOS => {
                let mut scan = parse_sos(segment, &mut source)?;
                let scan_start = offset;
                let scan_end = find_scan_end(bytes, scan_start)?;
                scan.data = bytes
                    .get(scan_start..scan_end)
                    .ok_or(JpegError::Truncated)?;
                if source.scan.is_empty() {
                    source.scan = scan.data;
                }
                source.scans.push(scan);
                offset = scan_end;
                if !source.progressive {
                    break;
                }
            }
            _ => {}
        }
    }

    if source.width == 0 || source.height == 0 || source.component_count == 0 {
        return Err(JpegError::Unsupported);
    }
    if require_scan && source.scan.is_empty() && source.scans.is_empty() {
        return Err(JpegError::Truncated);
    }
    Ok(source)
}

fn parse_adobe_transform(segment: &[u8]) -> Option<u8> {
    if segment.len() >= 12 && segment.get(..5) == Some(b"Adobe") {
        segment.get(11).copied()
    } else {
        None
    }
}

fn parse_exif_orientation(segment: &[u8]) -> Option<u8> {
    if segment.len() < 14 || segment.get(..6) != Some(b"Exif\0\0") {
        return None;
    }
    let tiff = segment.get(6..)?;
    let le = match tiff.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let magic = read_tiff_u16(tiff, 2, le)?;
    if magic != 42 {
        return None;
    }
    let ifd_offset = read_tiff_u32(tiff, 4, le)? as usize;
    let count = read_tiff_u16(tiff, ifd_offset, le)? as usize;
    let mut i = 0usize;
    while i < count {
        let entry = ifd_offset.checked_add(2)?.checked_add(i.checked_mul(12)?)?;
        let tag = read_tiff_u16(tiff, entry, le)?;
        if tag == 0x0112 {
            let field_type = read_tiff_u16(tiff, entry + 2, le)?;
            let values = read_tiff_u32(tiff, entry + 4, le)?;
            if field_type == 3 && values >= 1 {
                let value = if le {
                    read_tiff_u16(tiff, entry + 8, le)?
                } else {
                    read_tiff_u16(tiff, entry + 8, le)?
                };
                return Some(value.clamp(1, 8) as u8);
            }
        }
        i += 1;
    }
    None
}

fn read_tiff_u16(bytes: &[u8], offset: usize, le: bool) -> Option<u16> {
    let raw = [*bytes.get(offset)?, *bytes.get(offset + 1)?];
    Some(if le {
        u16::from_le_bytes(raw)
    } else {
        u16::from_be_bytes(raw)
    })
}

fn read_tiff_u32(bytes: &[u8], offset: usize, le: bool) -> Option<u32> {
    let raw = [
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
        *bytes.get(offset + 2)?,
        *bytes.get(offset + 3)?,
    ];
    Some(if le {
        u32::from_le_bytes(raw)
    } else {
        u32::from_be_bytes(raw)
    })
}

fn parse_dqt(segment: &[u8], source: &mut JpegSource<'_>) -> Result<(), JpegError> {
    let mut offset = 0usize;
    while offset < segment.len() {
        let table_spec = *segment.get(offset).ok_or(JpegError::Truncated)?;
        offset += 1;
        let precision = table_spec >> 4;
        let table_id = (table_spec & 0x0f) as usize;
        if precision != 0 || table_id >= 4 {
            return Err(JpegError::Unsupported);
        }
        let table = segment
            .get(offset..offset + 64)
            .ok_or(JpegError::Truncated)?;
        let mut i = 0usize;
        while i < 64 {
            source.quant[table_id][ZIGZAG[i]] = table[i] as u16;
            i += 1;
        }
        source.quant_valid[table_id] = true;
        offset += 64;
    }
    Ok(())
}

fn parse_dht(segment: &[u8], source: &mut JpegSource<'_>) -> Result<(), JpegError> {
    let mut offset = 0usize;
    while offset < segment.len() {
        let table_spec = *segment.get(offset).ok_or(JpegError::Truncated)?;
        offset += 1;
        let class = table_spec >> 4;
        let table_id = (table_spec & 0x0f) as usize;
        if table_id >= 4 || class > 1 {
            return Err(JpegError::Unsupported);
        }
        let counts = segment
            .get(offset..offset + 16)
            .ok_or(JpegError::Truncated)?;
        offset += 16;
        let mut total = 0usize;
        let mut i = 0usize;
        while i < 16 {
            total += counts[i] as usize;
            i += 1;
        }
        let symbols = segment
            .get(offset..offset + total)
            .ok_or(JpegError::Truncated)?;
        offset += total;
        if class == 0 {
            source.huff_dc[table_id].build(counts, symbols)?;
        } else {
            source.huff_ac[table_id].build(counts, symbols)?;
        }
    }
    Ok(())
}

fn parse_sof(
    segment: &[u8],
    source: &mut JpegSource<'_>,
    progressive: bool,
) -> Result<(), JpegError> {
    if segment.len() < 6 || segment[0] != 8 {
        return Err(JpegError::Unsupported);
    }
    let height = read_be_u16(segment, 1).ok_or(JpegError::Truncated)?;
    let width = read_be_u16(segment, 3).ok_or(JpegError::Truncated)?;
    let components = *segment.get(5).ok_or(JpegError::Truncated)? as usize;
    if width == 0
        || height == 0
        || !(components == 1 || components == 3 || components == 4)
        || segment.len() < 6 + components * 3
    {
        return Err(JpegError::Unsupported);
    }
    source.width = width;
    source.height = height;
    source.component_count = components;
    source.progressive = progressive;
    let mut i = 0usize;
    while i < components {
        let base = 6 + i * 3;
        let hv = segment[base + 1];
        source.components[i] = Component {
            id: segment[base],
            h: (hv >> 4).max(1),
            v: (hv & 0x0f).max(1),
            tq: segment[base + 2],
            td: 0,
            ta: 0,
        };
        if source.components[i].h > 2 || source.components[i].v > 2 || source.components[i].tq >= 4
        {
            return Err(JpegError::Unsupported);
        }
        i += 1;
    }
    Ok(())
}

fn parse_sos<'a>(segment: &[u8], source: &mut JpegSource<'a>) -> Result<ScanData<'a>, JpegError> {
    if segment.is_empty() {
        return Err(JpegError::Truncated);
    }
    let count = segment[0] as usize;
    if count == 0
        || count > source.component_count
        || count > 4
        || segment.len() < 1 + count * 2 + 3
    {
        return Err(JpegError::Unsupported);
    }
    if !source.progressive && count != source.component_count {
        return Err(JpegError::Unsupported);
    }
    let mut scan = ScanData::EMPTY;
    scan.component_count = count;
    let mut i = 0usize;
    while i < count {
        let id = segment[1 + i * 2];
        let table = segment[2 + i * 2];
        let mut found = false;
        let mut c = 0usize;
        while c < source.component_count {
            if source.components[c].id == id {
                source.components[c].td = table >> 4;
                source.components[c].ta = table & 0x0f;
                if source.components[c].td >= 4 || source.components[c].ta >= 4 {
                    return Err(JpegError::Unsupported);
                }
                scan.components[i] = ScanComponent {
                    index: c,
                    td: source.components[c].td,
                    ta: source.components[c].ta,
                };
                found = true;
                break;
            }
            c += 1;
        }
        if !found {
            return Err(JpegError::Unsupported);
        }
        i += 1;
    }
    let spectral = 1 + count * 2;
    scan.spectral_start = *segment.get(spectral).ok_or(JpegError::Truncated)?;
    scan.spectral_end = *segment.get(spectral + 1).ok_or(JpegError::Truncated)?;
    let successive = *segment.get(spectral + 2).ok_or(JpegError::Truncated)?;
    scan.successive_high = successive >> 4;
    scan.successive_low = successive & 0x0f;
    if !source.progressive
        && (scan.spectral_start != 0
            || scan.spectral_end != 63
            || scan.successive_high != 0
            || scan.successive_low != 0)
    {
        return Err(JpegError::Unsupported);
    }
    if source.progressive {
        if scan.spectral_start > scan.spectral_end
            || scan.spectral_end > 63
            || scan.successive_high > 13
            || scan.successive_low > 13
        {
            return Err(JpegError::Unsupported);
        }
        if count > 1 && !(scan.spectral_start == 0 && scan.spectral_end == 0) {
            return Err(JpegError::Unsupported);
        }
    }
    Ok(scan)
}

fn decode_baseline(source: &JpegSource<'_>) -> Result<DecodedImage, JpegError> {
    let mut max_h = 1u8;
    let mut max_v = 1u8;
    let mut i = 0usize;
    while i < source.component_count {
        let c = source.components[i];
        if !source.quant_valid[c.tq as usize] {
            return Err(JpegError::InvalidTable);
        }
        if !source.huff_dc[c.td as usize].valid || !source.huff_ac[c.ta as usize].valid {
            return Err(JpegError::InvalidTable);
        }
        max_h = max_h.max(c.h);
        max_v = max_v.max(c.v);
        i += 1;
    }

    let mcu_w = max_h as usize * 8;
    let mcu_h = max_v as usize * 8;
    let mcu_cols = div_ceil(source.width as usize, mcu_w);
    let mcu_rows = div_ceil(source.height as usize, mcu_h);
    let mut output = Vec::new();
    output.resize(source.width as usize * source.height as usize * 2, 0);
    let mut bits = BitReader::new(source.scan);
    let mut last_dc = [0i16; 4];
    let mut blocks = [[[0i16; 64]; 4]; 4];
    let mut mcu_index = 0usize;

    let mut my = 0usize;
    while my < mcu_rows {
        let mut mx = 0usize;
        while mx < mcu_cols {
            let mut ci = 0usize;
            while ci < source.component_count {
                let c = source.components[ci];
                let mut by = 0usize;
                while by < c.v as usize {
                    let mut bx = 0usize;
                    while bx < c.h as usize {
                        let block_index = by * c.h as usize + bx;
                        decode_block(
                            source,
                            &mut bits,
                            c,
                            &mut last_dc[ci],
                            &mut blocks[ci][block_index],
                        )?;
                        bx += 1;
                    }
                    by += 1;
                }
                ci += 1;
            }
            write_mcu(source, mx, my, max_h, max_v, &blocks, &mut output);
            mcu_index = mcu_index.saturating_add(1);
            if source.restart_interval != 0
                && mcu_index < mcu_rows.saturating_mul(mcu_cols)
                && mcu_index % source.restart_interval as usize == 0
            {
                bits.consume_restart_marker()?;
                last_dc = [0i16; 4];
            }
            mx += 1;
        }
        my += 1;
    }

    Ok(DecodedImage {
        width: source.width,
        height: source.height,
        stride: source.width as usize * 2,
        format: ImageFormat::Rgb565,
        data: output,
    })
}

fn decode_progressive(source: &JpegSource<'_>) -> Result<DecodedImage, JpegError> {
    if source.component_count == 4 {
        return Err(JpegError::UnsupportedProgressive);
    }
    if source.scans.is_empty() {
        return Err(JpegError::Truncated);
    }
    validate_progressive_scan_sequence(source)?;
    let (max_h, max_v) = validate_components(source)?;
    let mut block_w = [0usize; 4];
    let mut block_h = [0usize; 4];
    let mut coeffs: [Vec<[i16; 64]>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    let mut ci = 0usize;
    while ci < source.component_count {
        let c = source.components[ci];
        let comp_w = div_ceil(source.width as usize * c.h as usize, max_h as usize);
        let comp_h = div_ceil(source.height as usize * c.v as usize, max_v as usize);
        block_w[ci] = div_ceil(comp_w, 8);
        block_h[ci] = div_ceil(comp_h, 8);
        let blocks = block_w[ci]
            .checked_mul(block_h[ci])
            .ok_or(JpegError::Decode)?;
        let bytes = blocks
            .checked_mul(core::mem::size_of::<[i16; 64]>())
            .ok_or(JpegError::Decode)?;
        if bytes > PROGRESSIVE_COEFF_BUDGET_BYTES {
            return Err(JpegError::Decode);
        }
        coeffs[ci].resize(blocks, [0; 64]);
        ci += 1;
    }

    let mut last_dc = [0i16; 4];
    let mut s = 0usize;
    while s < source.scans.len() {
        decode_progressive_scan(
            source,
            source.scans[s],
            &mut coeffs,
            block_w,
            block_h,
            &mut last_dc,
        )?;
        s += 1;
    }

    let mcu_w = max_h as usize * 8;
    let mcu_h = max_v as usize * 8;
    let mcu_cols = div_ceil(source.width as usize, mcu_w);
    let mcu_rows = div_ceil(source.height as usize, mcu_h);
    let mut output = Vec::new();
    output.resize(source.width as usize * source.height as usize * 2, 0);
    let mut blocks = [[[0i16; 64]; 4]; 4];

    let mut my = 0usize;
    while my < mcu_rows {
        let mut mx = 0usize;
        while mx < mcu_cols {
            let mut cidx = 0usize;
            while cidx < source.component_count {
                let c = source.components[cidx];
                let mut by = 0usize;
                while by < c.v as usize {
                    let mut bx = 0usize;
                    while bx < c.h as usize {
                        let block_index = by * c.h as usize + bx;
                        let src_x = mx * c.h as usize + bx;
                        let src_y = my * c.v as usize + by;
                        if src_x < block_w[cidx] && src_y < block_h[cidx] {
                            let coeff_index = src_y * block_w[cidx] + src_x;
                            dequant_idct_block(
                                source,
                                cidx,
                                &coeffs[cidx][coeff_index],
                                &mut blocks[cidx][block_index],
                            );
                        } else {
                            blocks[cidx][block_index] = [0; 64];
                        }
                        bx += 1;
                    }
                    by += 1;
                }
                cidx += 1;
            }
            write_mcu(source, mx, my, max_h, max_v, &blocks, &mut output);
            mx += 1;
        }
        my += 1;
    }

    Ok(DecodedImage {
        width: source.width,
        height: source.height,
        stride: source.width as usize * 2,
        format: ImageFormat::Rgb565,
        data: output,
    })
}

fn validate_progressive_scan_sequence(source: &JpegSource<'_>) -> Result<(), JpegError> {
    let mut saw_dc_first = [false; 4];
    let mut s = 0usize;
    while s < source.scans.len() {
        let scan = source.scans[s];
        if scan.component_count == 0 || scan.component_count > source.component_count {
            return Err(JpegError::Decode);
        }
        if scan.successive_high != 0 && scan.successive_low + 1 != scan.successive_high {
            return Err(JpegError::UnsupportedProgressive);
        }
        if scan.spectral_start == 0 && scan.spectral_end == 0 {
            let mut i = 0usize;
            while i < scan.component_count {
                let ci = scan.components[i].index;
                if ci >= source.component_count {
                    return Err(JpegError::Decode);
                }
                if scan.successive_high == 0 {
                    saw_dc_first[ci] = true;
                } else if !saw_dc_first[ci] {
                    return Err(JpegError::Decode);
                }
                if !source.huff_dc[scan.components[i].td as usize].valid {
                    return Err(JpegError::InvalidTable);
                }
                i += 1;
            }
        } else {
            if scan.component_count != 1 {
                return Err(JpegError::UnsupportedProgressive);
            }
            let ci = scan.components[0].index;
            if ci >= source.component_count || !saw_dc_first[ci] {
                return Err(JpegError::Decode);
            }
            if !source.huff_ac[scan.components[0].ta as usize].valid {
                return Err(JpegError::InvalidTable);
            }
        }
        s += 1;
    }
    Ok(())
}

fn validate_components(source: &JpegSource<'_>) -> Result<(u8, u8), JpegError> {
    let mut max_h = 1u8;
    let mut max_v = 1u8;
    let mut i = 0usize;
    while i < source.component_count {
        let c = source.components[i];
        if !source.quant_valid[c.tq as usize] {
            return Err(JpegError::InvalidTable);
        }
        max_h = max_h.max(c.h);
        max_v = max_v.max(c.v);
        i += 1;
    }
    Ok((max_h, max_v))
}

fn decode_progressive_scan(
    source: &JpegSource<'_>,
    scan: ScanData<'_>,
    coeffs: &mut [Vec<[i16; 64]>; 4],
    block_w: [usize; 4],
    block_h: [usize; 4],
    last_dc: &mut [i16; 4],
) -> Result<(), JpegError> {
    let mut bits = BitReader::new(scan.data);
    let mut eob_run = 0usize;
    let mut units = 0usize;
    if scan.spectral_start == 0 && scan.successive_high == 0 {
        *last_dc = [0; 4];
    }
    if scan.component_count > 1 {
        if scan.spectral_start != 0 || scan.spectral_end != 0 {
            return Err(JpegError::UnsupportedProgressive);
        }
        let (max_h, max_v) = validate_components(source)?;
        let mcu_cols = div_ceil(source.width as usize, max_h as usize * 8);
        let mcu_rows = div_ceil(source.height as usize, max_v as usize * 8);
        let mut my = 0usize;
        while my < mcu_rows {
            let mut mx = 0usize;
            while mx < mcu_cols {
                let mut si = 0usize;
                while si < scan.component_count {
                    let ci = scan.components[si].index;
                    let c = source.components[ci];
                    let mut by = 0usize;
                    while by < c.v as usize {
                        let mut bx = 0usize;
                        while bx < c.h as usize {
                            let x = mx * c.h as usize + bx;
                            let y = my * c.v as usize + by;
                            if x < block_w[ci] && y < block_h[ci] {
                                let index = y * block_w[ci] + x;
                                decode_progressive_dc_block(
                                    source,
                                    &mut bits,
                                    scan,
                                    scan.components[si],
                                    &mut last_dc[ci],
                                    &mut coeffs[ci][index],
                                )?;
                            }
                            bx += 1;
                        }
                        by += 1;
                    }
                    si += 1;
                }
                units = units.saturating_add(1);
                maybe_consume_restart(
                    source,
                    &mut bits,
                    &mut units,
                    mcu_rows.saturating_mul(mcu_cols),
                    last_dc,
                    &mut eob_run,
                )?;
                mx += 1;
            }
            my += 1;
        }
    } else if scan.component_count == 1 {
        let scan_component = scan.components[0];
        let ci = scan_component.index;
        let mut by = 0usize;
        while by < block_h[ci] {
            let mut bx = 0usize;
            while bx < block_w[ci] {
                let index = by * block_w[ci] + bx;
                if scan.spectral_start == 0 && scan.spectral_end == 0 {
                    decode_progressive_dc_block(
                        source,
                        &mut bits,
                        scan,
                        scan_component,
                        &mut last_dc[ci],
                        &mut coeffs[ci][index],
                    )?;
                } else {
                    decode_progressive_ac_block(
                        source,
                        &mut bits,
                        scan,
                        scan_component,
                        &mut coeffs[ci][index],
                        &mut eob_run,
                    )?;
                }
                units = units.saturating_add(1);
                maybe_consume_restart(
                    source,
                    &mut bits,
                    &mut units,
                    block_w[ci].saturating_mul(block_h[ci]),
                    last_dc,
                    &mut eob_run,
                )?;
                bx += 1;
            }
            by += 1;
        }
    } else {
        return Err(JpegError::UnsupportedProgressive);
    }
    Ok(())
}

fn maybe_consume_restart(
    source: &JpegSource<'_>,
    bits: &mut BitReader<'_>,
    units: &mut usize,
    total: usize,
    last_dc: &mut [i16; 4],
    eob_run: &mut usize,
) -> Result<(), JpegError> {
    if source.restart_interval != 0
        && *units < total
        && *units % source.restart_interval as usize == 0
    {
        bits.consume_restart_marker()?;
        *last_dc = [0; 4];
        *eob_run = 0;
    }
    Ok(())
}

fn decode_progressive_dc_block(
    source: &JpegSource<'_>,
    bits: &mut BitReader<'_>,
    scan: ScanData<'_>,
    sc: ScanComponent,
    last_dc: &mut i16,
    coeff: &mut [i16; 64],
) -> Result<(), JpegError> {
    if scan.successive_high == 0 {
        let symbol = source.huff_dc[sc.td as usize].decode(bits)?;
        let diff = receive_extend(bits, symbol)?;
        *last_dc = last_dc.saturating_add(diff);
        coeff[0] = shift_coeff(*last_dc, scan.successive_low);
    } else {
        let bit = bits.read_bit()? as i16;
        if bit != 0 {
            refine_coeff(&mut coeff[0], scan.successive_low);
        }
    }
    Ok(())
}

fn decode_progressive_ac_block(
    source: &JpegSource<'_>,
    bits: &mut BitReader<'_>,
    scan: ScanData<'_>,
    sc: ScanComponent,
    coeff: &mut [i16; 64],
    eob_run: &mut usize,
) -> Result<(), JpegError> {
    let mut k = scan.spectral_start as usize;
    let end = scan.spectral_end as usize;
    if scan.successive_high == 0 {
        if *eob_run > 0 {
            *eob_run -= 1;
            return Ok(());
        }
        while k <= end {
            let symbol = source.huff_ac[sc.ta as usize].decode(bits)?;
            let run = (symbol >> 4) as usize;
            let size = symbol & 0x0f;
            if size == 0 {
                if run == 15 {
                    k += 16;
                    continue;
                }
                *eob_run = 1usize << run;
                if run > 0 {
                    *eob_run = (*eob_run).saturating_add(bits.read_bits(run as u8)? as usize);
                }
                *eob_run = (*eob_run).saturating_sub(1);
                break;
            }
            k += run;
            if k > end || size > 11 {
                return Err(JpegError::Decode);
            }
            let natural = ZIGZAG[k];
            coeff[natural] = shift_coeff(receive_extend(bits, size)?, scan.successive_low);
            k += 1;
        }
    } else {
        if *eob_run > 0 {
            refine_nonzero_coeffs(bits, coeff, k, end, scan.successive_low)?;
            *eob_run -= 1;
            return Ok(());
        }
        while k <= end {
            let symbol = source.huff_ac[sc.ta as usize].decode(bits)?;
            let mut run = (symbol >> 4) as usize;
            let size = symbol & 0x0f;
            if size == 0 {
                if run == 15 {
                    let mut zeros = 16usize;
                    while k <= end && zeros > 0 {
                        let natural = ZIGZAG[k];
                        if coeff[natural] == 0 {
                            zeros -= 1;
                        } else {
                            read_refinement_bit(bits, &mut coeff[natural], scan.successive_low)?;
                        }
                        k += 1;
                    }
                    continue;
                }
                *eob_run = 1usize << run;
                if run > 0 {
                    *eob_run = (*eob_run).saturating_add(bits.read_bits(run as u8)? as usize);
                }
                refine_nonzero_coeffs(bits, coeff, k, end, scan.successive_low)?;
                *eob_run = (*eob_run).saturating_sub(1);
                break;
            }
            if size != 1 {
                return Err(JpegError::Decode);
            }
            let new_coeff = if bits.read_bit()? != 0 {
                shift_coeff(1, scan.successive_low)
            } else {
                shift_coeff(-1, scan.successive_low)
            };
            while k <= end {
                let natural = ZIGZAG[k];
                if coeff[natural] != 0 {
                    read_refinement_bit(bits, &mut coeff[natural], scan.successive_low)?;
                } else if run == 0 {
                    coeff[natural] = new_coeff;
                    k += 1;
                    break;
                } else {
                    run -= 1;
                }
                k += 1;
            }
        }
    }
    Ok(())
}

fn refine_nonzero_coeffs(
    bits: &mut BitReader<'_>,
    coeff: &mut [i16; 64],
    start: usize,
    end: usize,
    bit: u8,
) -> Result<(), JpegError> {
    let mut k = start;
    while k <= end {
        let natural = ZIGZAG[k];
        if coeff[natural] != 0 {
            read_refinement_bit(bits, &mut coeff[natural], bit)?;
        }
        k += 1;
    }
    Ok(())
}

fn read_refinement_bit(
    bits: &mut BitReader<'_>,
    coeff: &mut i16,
    bit: u8,
) -> Result<(), JpegError> {
    if bits.read_bit()? != 0 {
        refine_coeff(coeff, bit);
    }
    Ok(())
}

fn refine_coeff(coeff: &mut i16, bit: u8) {
    let delta = shift_coeff(1, bit);
    if *coeff >= 0 {
        *coeff = coeff.saturating_add(delta);
    } else {
        *coeff = coeff.saturating_sub(delta);
    }
}

fn shift_coeff(value: i16, bits: u8) -> i16 {
    ((value as i32) << bits.min(13)).clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn dequant_idct_block(source: &JpegSource<'_>, ci: usize, coeff: &[i16; 64], out: &mut [i16; 64]) {
    let c = source.components[ci];
    let mut dequant = [0i16; 64];
    let mut i = 0usize;
    while i < 64 {
        dequant[i] = coeff[i].saturating_mul(source.quant[c.tq as usize][i] as i16);
        i += 1;
    }
    idct_block(&dequant, out);
}

fn decode_block(
    source: &JpegSource<'_>,
    bits: &mut BitReader<'_>,
    c: Component,
    last_dc: &mut i16,
    out: &mut [i16; 64],
) -> Result<(), JpegError> {
    let mut coeff = [0i16; 64];
    let dc_symbol = source.huff_dc[c.td as usize].decode(bits)?;
    let diff = receive_extend(bits, dc_symbol)?;
    let dc = last_dc.saturating_add(diff);
    *last_dc = dc;
    coeff[0] = dc.saturating_mul(source.quant[c.tq as usize][0] as i16);

    let mut k = 1usize;
    while k < 64 {
        let symbol = source.huff_ac[c.ta as usize].decode(bits)?;
        if symbol == 0 {
            break;
        }
        if symbol == 0xf0 {
            k += 16;
            continue;
        }
        let run = (symbol >> 4) as usize;
        let size = symbol & 0x0f;
        k += run;
        if k >= 64 {
            return Err(JpegError::Decode);
        }
        let value = receive_extend(bits, size)?;
        let natural = ZIGZAG[k];
        coeff[natural] = value.saturating_mul(source.quant[c.tq as usize][natural] as i16);
        k += 1;
    }

    idct_block(&coeff, out);
    Ok(())
}

fn idct_block(coeff: &[i16; 64], out: &mut [i16; 64]) {
    let mut y = 0usize;
    while y < 8 {
        let mut x = 0usize;
        while x < 8 {
            let mut sum = 0i64;
            let mut v = 0usize;
            while v < 8 {
                let mut u = 0usize;
                while u < 8 {
                    let basis = IDCT_BASIS[u][x] as i64 * IDCT_BASIS[v][y] as i64;
                    sum += coeff[v * 8 + u] as i64 * basis;
                    u += 1;
                }
                v += 1;
            }
            let value = ((sum / (4 * 4096 * 4096)) + 128).clamp(0, 255) as i16;
            out[y * 8 + x] = value;
            x += 1;
        }
        y += 1;
    }
}

fn write_mcu(
    source: &JpegSource<'_>,
    mx: usize,
    my: usize,
    max_h: u8,
    max_v: u8,
    blocks: &[[[i16; 64]; 4]; 4],
    output: &mut [u8],
) {
    let mcu_w = max_h as usize * 8;
    let mcu_h = max_v as usize * 8;
    let base_x = mx * mcu_w;
    let base_y = my * mcu_h;
    let mut py = 0usize;
    while py < mcu_h {
        let y = base_y + py;
        if y >= source.height as usize {
            break;
        }
        let mut px = 0usize;
        while px < mcu_w {
            let x = base_x + px;
            if x >= source.width as usize {
                break;
            }
            let y_sample = sample_component(source.components[0], max_h, max_v, blocks[0], px, py);
            let (r, g, b) = if source.component_count == 1 {
                (y_sample, y_sample, y_sample)
            } else if source.component_count == 4 {
                let k = sample_component(source.components[3], max_h, max_v, blocks[3], px, py);
                if source.adobe_transform == Some(2) {
                    let cb =
                        sample_component(source.components[1], max_h, max_v, blocks[1], px, py)
                            - 128;
                    let cr =
                        sample_component(source.components[2], max_h, max_v, blocks[2], px, py)
                            - 128;
                    let (r, g, b) = ycbcr_to_rgb(y_sample, cb, cr);
                    apply_k(r, g, b, k)
                } else {
                    cmyk_to_rgb(
                        y_sample,
                        sample_component(source.components[1], max_h, max_v, blocks[1], px, py),
                        sample_component(source.components[2], max_h, max_v, blocks[2], px, py),
                        k,
                    )
                }
            } else {
                let cb =
                    sample_component(source.components[1], max_h, max_v, blocks[1], px, py) - 128;
                let cr =
                    sample_component(source.components[2], max_h, max_v, blocks[2], px, py) - 128;
                ycbcr_to_rgb(y_sample, cb, cr)
            };
            let rgb565 = rgb_to_565(r as u8, g as u8, b as u8);
            let offset = (y * source.width as usize + x) * 2;
            output[offset] = (rgb565 & 0xff) as u8;
            output[offset + 1] = (rgb565 >> 8) as u8;
            px += 1;
        }
        py += 1;
    }
}

fn sample_component(
    c: Component,
    max_h: u8,
    max_v: u8,
    blocks: [[i16; 64]; 4],
    px: usize,
    py: usize,
) -> i32 {
    let sx = px * c.h as usize / max_h as usize;
    let sy = py * c.v as usize / max_v as usize;
    let block_x = (sx / 8).min(c.h as usize - 1);
    let block_y = (sy / 8).min(c.v as usize - 1);
    let block = block_y * c.h as usize + block_x;
    blocks[block][(sy % 8) * 8 + (sx % 8)] as i32
}

fn ycbcr_to_rgb(y: i32, cb: i32, cr: i32) -> (i32, i32, i32) {
    let r = y + ((91881 * cr) >> 16);
    let g = y - ((22554 * cb + 46802 * cr) >> 16);
    let b = y + ((116130 * cb) >> 16);
    (r.clamp(0, 255), g.clamp(0, 255), b.clamp(0, 255))
}

fn apply_k(r: i32, g: i32, b: i32, k: i32) -> (i32, i32, i32) {
    let factor = (255 - k.clamp(0, 255)).max(0);
    (
        (r.clamp(0, 255) * factor / 255).clamp(0, 255),
        (g.clamp(0, 255) * factor / 255).clamp(0, 255),
        (b.clamp(0, 255) * factor / 255).clamp(0, 255),
    )
}

fn cmyk_to_rgb(c: i32, m: i32, y: i32, k: i32) -> (i32, i32, i32) {
    (
        (255 - (c.clamp(0, 255) + k.clamp(0, 255)).min(255)).clamp(0, 255),
        (255 - (m.clamp(0, 255) + k.clamp(0, 255)).min(255)).clamp(0, 255),
        (255 - (y.clamp(0, 255) + k.clamp(0, 255)).min(255)).clamp(0, 255),
    )
}

fn rgb_to_565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

fn apply_exif_orientation(image: DecodedImage, orientation: u8) -> Option<DecodedImage> {
    if orientation <= 1 || image.format != ImageFormat::Rgb565 {
        return Some(image);
    }
    let width = image.width as usize;
    let height = image.height as usize;
    let swapped = matches!(orientation, 5 | 6 | 7 | 8);
    let out_w = if swapped { height } else { width };
    let out_h = if swapped { width } else { height };
    if out_w > u16::MAX as usize || out_h > u16::MAX as usize {
        return None;
    }
    let mut out = Vec::new();
    out.resize(out_w.checked_mul(out_h)?.checked_mul(2)?, 0);
    let mut dy = 0usize;
    while dy < out_h {
        let mut dx = 0usize;
        while dx < out_w {
            let (sx, sy) = match orientation {
                2 => (width - 1 - dx, dy),
                3 => (width - 1 - dx, height - 1 - dy),
                4 => (dx, height - 1 - dy),
                5 => (dy, dx),
                6 => (dy, height - 1 - dx),
                7 => (width - 1 - dy, height - 1 - dx),
                8 => (width - 1 - dy, dx),
                _ => (dx, dy),
            };
            let src = sy
                .checked_mul(image.stride)?
                .checked_add(sx.checked_mul(2)?)?;
            let dst = dy.checked_mul(out_w)?.checked_add(dx)?.checked_mul(2)?;
            out[dst] = *image.data.get(src)?;
            out[dst + 1] = *image.data.get(src + 1)?;
            dx += 1;
        }
        dy += 1;
    }
    Some(DecodedImage {
        width: out_w as u16,
        height: out_h as u16,
        stride: out_w * 2,
        format: ImageFormat::Rgb565,
        data: out,
    })
}

fn receive_extend(bits: &mut BitReader<'_>, size: u8) -> Result<i16, JpegError> {
    if size == 0 {
        return Ok(0);
    }
    let value = bits.read_bits(size)? as i32;
    let vt = 1i32 << (size - 1);
    if value < vt {
        Ok((value + ((-1i32) << size) + 1) as i16)
    } else {
        Ok(value as i16)
    }
}

struct BitReader<'a> {
    bytes: &'a [u8],
    index: usize,
    bit_buf: u32,
    bit_len: u8,
}

impl<'a> BitReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            index: 0,
            bit_buf: 0,
            bit_len: 0,
        }
    }

    fn read_bit(&mut self) -> Result<u8, JpegError> {
        Ok(self.read_bits(1)? as u8)
    }

    fn read_bits(&mut self, count: u8) -> Result<u16, JpegError> {
        while self.bit_len < count {
            let byte = *self.bytes.get(self.index).ok_or(JpegError::Truncated)?;
            self.index += 1;
            if byte == 0xff {
                let stuffed = *self.bytes.get(self.index).ok_or(JpegError::Truncated)?;
                if stuffed == 0x00 {
                    self.index += 1;
                } else {
                    return Err(JpegError::Decode);
                }
            }
            self.bit_buf = (self.bit_buf << 8) | byte as u32;
            self.bit_len += 8;
        }
        let shift = self.bit_len - count;
        let mask = (1u32 << count) - 1;
        let value = ((self.bit_buf >> shift) & mask) as u16;
        self.bit_len -= count;
        self.bit_buf &= (1u32 << self.bit_len) - 1;
        Ok(value)
    }

    fn consume_restart_marker(&mut self) -> Result<(), JpegError> {
        self.bit_buf = 0;
        self.bit_len = 0;
        while self.bytes.get(self.index) == Some(&0xff) {
            self.index += 1;
        }
        let marker = *self.bytes.get(self.index).ok_or(JpegError::Truncated)?;
        if (0xd0..=0xd7).contains(&marker) {
            self.index += 1;
            Ok(())
        } else {
            Err(JpegError::UnexpectedRestartMarker)
        }
    }
}

fn next_marker(bytes: &[u8], offset: &mut usize) -> Result<u8, JpegError> {
    while *offset < bytes.len() && bytes[*offset] != 0xff {
        *offset += 1;
    }
    while *offset < bytes.len() && bytes[*offset] == 0xff {
        *offset += 1;
    }
    let marker = *bytes.get(*offset).ok_or(JpegError::Truncated)?;
    *offset += 1;
    Ok(marker)
}

fn marker_is_standalone(marker: u8) -> bool {
    marker == 0x01 || (0xd0..=0xd7).contains(&marker)
}

fn find_scan_end(bytes: &[u8], start: usize) -> Result<usize, JpegError> {
    let mut i = start;
    while i + 1 < bytes.len() {
        if bytes[i] == 0xff {
            let next = bytes[i + 1];
            if next == 0x00 {
                i += 2;
                continue;
            }
            if (0xd0..=0xd7).contains(&next) {
                i += 2;
                continue;
            }
            return Ok(i);
        }
        i += 1;
    }
    Err(JpegError::Truncated)
}

fn div_ceil(value: usize, div: usize) -> usize {
    value.saturating_add(div - 1) / div
}

fn read_be_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
    ]))
}
