use crate::{
    backend::{
        CodecAcceleratorCapabilities, CodecPipelinePlan, CodecStageKind, CodecStagePlan,
    },
    ImageFormat, ImageView,
};
use alloc::vec::Vec;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

#[derive(Debug)]
#[allow(dead_code)]
pub struct DecodedRgb565 {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PngInfo {
    pub width: u16,
    pub height: u16,
    pub bit_depth: u8,
    pub color_type: u8,
    pub indexed: bool,
    pub has_alpha: bool,
    pub interlaced: bool,
    pub srgb: bool,
    pub gamma: Option<u32>,
    pub icc_present: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PngError {
    BadSignature,
    Truncated,
    Unsupported,
    UnsupportedBitDepth,
    UnsupportedColorType,
    UnsupportedCompression,
    UnsupportedFilter,
    UnsupportedInterlace,
    InvalidChunk,
    InvalidPalette,
    InvalidDimensions,
    MissingImageData,
    MissingPalette,
    Decode,
    Overflow,
}

#[derive(Debug)]
pub struct DecodedImage {
    pub width: u16,
    pub height: u16,
    pub stride: usize,
    pub format: ImageFormat,
    pub data: Vec<u8>,
}

impl DecodedImage {
    pub fn view(&self) -> ImageView {
        ImageView::from_slice(
            self.width,
            self.height,
            self.stride,
            self.format,
            self.data.as_slice(),
        )
    }

    pub fn byte_len(&self) -> usize {
        self.data.len()
    }
}

pub struct PngDecoder;

impl PngDecoder {
    pub fn inspect(bytes: &[u8]) -> Option<PngInfo> {
        parse_png(bytes).map(|source| source.info())
    }

    pub fn inspect_result(bytes: &[u8]) -> Result<PngInfo, PngError> {
        Ok(parse_png_result(bytes)?.info())
    }

    pub fn decode(bytes: &[u8]) -> Option<DecodedImage> {
        decode_png(bytes)
    }

    pub fn decode_result(bytes: &[u8]) -> Result<DecodedImage, PngError> {
        decode_png_result(bytes)
    }

    pub fn plan_pipeline<const STAGES: usize>() -> CodecPipelinePlan<STAGES> {
        Self::plan_pipeline_with_caps(CodecAcceleratorCapabilities::NONE)
    }

    pub fn plan_pipeline_with_caps<const STAGES: usize>(
        caps: CodecAcceleratorCapabilities,
    ) -> CodecPipelinePlan<STAGES> {
        let supported = caps.png && caps.max_stages >= 8;
        let mut plan = CodecPipelinePlan::new();
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Read, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Inspect, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Header, true, supported));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Entropy, true, supported));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Transform, true, supported));
        let _ = plan.push(CodecStagePlan::new(
            CodecStageKind::ColorConvert,
            true,
            supported,
        ));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Pack, true, supported));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::CacheInsert, false, true));
        plan
    }
}

pub fn decode_png(bytes: &[u8]) -> Option<DecodedImage> {
    decode_png_result(bytes).ok()
}

pub fn decode_png_result(bytes: &[u8]) -> Result<DecodedImage, PngError> {
    let source = parse_png_result(bytes)?;
    let raw_len = source.raw_len().ok_or(PngError::Overflow)?;
    let raw = inflate_zlib(&source.idat, raw_len).ok_or(PngError::Decode)?;
    let pixels = source.decode_pixels(&raw).ok_or(PngError::Decode)?;
    convert_png_pixels(&pixels, &source).ok_or(PngError::Decode)
}

#[allow(dead_code)]
pub fn decode_png_rgb565_cover(bytes: &[u8], width: u16, height: u16) -> Option<DecodedRgb565> {
    let source = parse_png(bytes)?;
    let raw = inflate_zlib(&source.idat, source.raw_len()?)?;
    let pixels = source.decode_pixels(&raw)?;
    Some(DecodedRgb565 {
        width,
        height,
        data: scale_cover_rgb565(&pixels, source.width, source.height, &source, width, height)?,
    })
}

#[allow(dead_code)]
pub const fn png_runtime_decode_enabled() -> bool {
    true
}

struct PngSource {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    interlace: u8,
    idat: Vec<u8>,
    palette: Vec<[u8; 3]>,
    transparency: Vec<u8>,
    gamma: Option<u32>,
    srgb: bool,
    icc_present: bool,
}

impl PngSource {
    fn info(&self) -> PngInfo {
        PngInfo {
            width: self.width.min(u16::MAX as u32) as u16,
            height: self.height.min(u16::MAX as u32) as u16,
            bit_depth: self.bit_depth,
            color_type: self.color_type,
            indexed: self.color_type == 3,
            has_alpha: self.has_alpha(),
            interlaced: self.interlace == 1,
            srgb: self.srgb,
            gamma: self.gamma,
            icc_present: self.icc_present,
        }
    }

    fn channels(&self) -> usize {
        png_channels(self.color_type).unwrap_or(0)
    }

    fn sample_bytes(&self) -> usize {
        if self.bit_depth == 16 {
            2
        } else {
            1
        }
    }

    fn pixel_bytes(&self) -> usize {
        self.channels().saturating_mul(self.sample_bytes())
    }

    fn has_alpha(&self) -> bool {
        matches!(self.color_type, 4 | 6) || !self.transparency.is_empty()
    }

    fn raw_len(&self) -> Option<usize> {
        if self.interlace == 1 {
            return adam7_raw_len(self.width, self.height, self.pixel_bytes());
        }
        let stride = (self.width as usize).checked_mul(self.pixel_bytes())?;
        stride.checked_add(1)?.checked_mul(self.height as usize)
    }

    fn decode_pixels(&self, raw: &[u8]) -> Option<Vec<u8>> {
        if self.interlace == 1 {
            deinterlace_adam7(raw, self.width, self.height, self.pixel_bytes())
        } else {
            unfilter_png(raw, self.width, self.height, self.pixel_bytes())
        }
    }
}

fn parse_png(bytes: &[u8]) -> Option<PngSource> {
    parse_png_result(bytes).ok()
}

fn parse_png_result(bytes: &[u8]) -> Result<PngSource, PngError> {
    if bytes.len() < PNG_SIGNATURE.len() {
        return Err(PngError::Truncated);
    }
    if bytes.get(..PNG_SIGNATURE.len()) != Some(PNG_SIGNATURE) {
        return Err(PngError::BadSignature);
    }
    let mut offset = PNG_SIGNATURE.len();
    let mut width = 0u32;
    let mut height = 0u32;
    let mut bit_depth = 0u8;
    let mut color_type = 0u8;
    let mut compression = 0u8;
    let mut filter = 0u8;
    let mut interlace = 0u8;
    let mut idat = Vec::new();
    let mut palette: Vec<[u8; 3]> = Vec::new();
    let mut transparency = Vec::new();
    let mut gamma = None;
    let mut srgb = false;
    let mut icc_present = false;
    let mut seen_ihdr = false;
    let mut seen_iend = false;

    while offset < bytes.len() {
        let header_end = offset.checked_add(8).ok_or(PngError::Overflow)?;
        if header_end > bytes.len() {
            return Err(PngError::Truncated);
        }
        let len = read_be_u32(bytes, offset).ok_or(PngError::Truncated)? as usize;
        let chunk_type = bytes
            .get(offset + 4..offset + 8)
            .ok_or(PngError::Truncated)?;
        let chunk_start = offset.checked_add(8).ok_or(PngError::Overflow)?;
        let chunk_end = chunk_start.checked_add(len).ok_or(PngError::Overflow)?;
        let crc_end = chunk_end.checked_add(4).ok_or(PngError::Overflow)?;
        if crc_end > bytes.len() {
            return Err(PngError::Truncated);
        }

        let chunk = bytes
            .get(chunk_start..chunk_end)
            .ok_or(PngError::Truncated)?;
        let stored_crc = read_be_u32(bytes, chunk_end).ok_or(PngError::Truncated)?;
        if is_critical_chunk(chunk_type) && png_crc32(chunk_type, chunk) != stored_crc {
            return Err(PngError::InvalidChunk);
        }
        offset = crc_end;

        match chunk_type {
            b"IHDR" => {
                if chunk.len() != 13 {
                    return Err(PngError::InvalidChunk);
                }
                width = read_be_u32(chunk, 0).ok_or(PngError::Truncated)?;
                height = read_be_u32(chunk, 4).ok_or(PngError::Truncated)?;
                bit_depth = *chunk.get(8).ok_or(PngError::Truncated)?;
                color_type = *chunk.get(9).ok_or(PngError::Truncated)?;
                compression = *chunk.get(10).ok_or(PngError::Truncated)?;
                filter = *chunk.get(11).ok_or(PngError::Truncated)?;
                interlace = *chunk.get(12).ok_or(PngError::Truncated)?;
                seen_ihdr = true;
            }
            b"PLTE" => {
                if chunk.len() % 3 != 0 {
                    return Err(PngError::InvalidPalette);
                }
                let mut i = 0usize;
                while i + 2 < chunk.len() {
                    palette.push([chunk[i], chunk[i + 1], chunk[i + 2]]);
                    i += 3;
                }
            }
            b"tRNS" => transparency.extend_from_slice(chunk),
            b"gAMA" => {
                if chunk.len() == 4 {
                    gamma = read_be_u32(chunk, 0);
                }
            }
            b"sRGB" => {
                if chunk.len() == 1 {
                    srgb = true;
                }
            }
            b"iCCP" => {
                icc_present = true;
            }
            b"IDAT" => idat.extend_from_slice(chunk),
            b"IEND" => {
                seen_iend = true;
                break;
            }
            _ => {}
        }
    }

    if !seen_ihdr || width == 0 || height == 0 {
        return Err(PngError::InvalidDimensions);
    }
    if png_channels(color_type).is_none() {
        return Err(PngError::UnsupportedColorType);
    }
    if !(bit_depth == 8 || (bit_depth == 16 && color_type != 3)) {
        return Err(PngError::UnsupportedBitDepth);
    }
    if compression != 0 {
        return Err(PngError::UnsupportedCompression);
    }
    if filter != 0 {
        return Err(PngError::UnsupportedFilter);
    }
    if interlace > 1 {
        return Err(PngError::UnsupportedInterlace);
    }
    if idat.is_empty() {
        return Err(PngError::MissingImageData);
    }
    if color_type == 3 && palette.is_empty() {
        return Err(PngError::MissingPalette);
    }
    if !seen_iend {
        return Err(PngError::Truncated);
    }

    Ok(PngSource {
        width,
        height,
        bit_depth,
        color_type,
        interlace,
        idat,
        palette,
        transparency,
        gamma,
        srgb,
        icc_present,
    })
}

fn is_critical_chunk(chunk_type: &[u8]) -> bool {
    chunk_type
        .first()
        .copied()
        .map(|byte| byte & 0x20 == 0)
        .unwrap_or(false)
}

fn png_crc32(chunk_type: &[u8], chunk: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in chunk_type.iter().chain(chunk.iter()).copied() {
        crc ^= byte as u32;
        let mut bit = 0u8;
        while bit < 8 {
            let mask = if crc & 1 != 0 { 0xedb8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
            bit += 1;
        }
    }
    !crc
}

fn inflate_zlib(bytes: &[u8], expected_len: usize) -> Option<Vec<u8>> {
    if bytes.len() < 6 {
        return None;
    }

    let cmf = bytes[0];
    let flg = bytes[1];
    if cmf & 0x0f != 8
        || (cmf >> 4) > 7
        || ((cmf as u16) << 8 | flg as u16) % 31 != 0
        || (flg & 0x20) != 0
    {
        return None;
    }

    let deflate = bytes.get(2..bytes.len().checked_sub(4)?)?;
    let mut reader = BitReader::new(deflate);
    let mut out = Vec::new();
    out.reserve_exact(expected_len);

    loop {
        let final_block = reader.read_bits(1)? != 0;
        match reader.read_bits(2)? {
            0 => inflate_stored_block(&mut reader, &mut out, expected_len)?,
            1 => {
                let (litlen, dist) = fixed_huffman()?;
                inflate_huffman_block(&mut reader, &mut out, expected_len, &litlen, &dist)?;
            }
            2 => {
                let (litlen, dist) = dynamic_huffman(&mut reader)?;
                inflate_huffman_block(&mut reader, &mut out, expected_len, &litlen, &dist)?;
            }
            _ => return None,
        }

        if final_block {
            break;
        }
    }

    if out.len() == expected_len {
        Some(out)
    } else {
        None
    }
}

fn inflate_stored_block(
    reader: &mut BitReader<'_>,
    out: &mut Vec<u8>,
    expected_len: usize,
) -> Option<()> {
    reader.align_byte();
    let len = reader.read_bits(16)? as usize;
    let nlen = reader.read_bits(16)? as usize;
    if len ^ nlen != 0xffff {
        return None;
    }
    if out.len().checked_add(len)? > expected_len {
        return None;
    }
    for _ in 0..len {
        out.push(reader.read_bits(8)? as u8);
    }
    Some(())
}

fn inflate_huffman_block(
    reader: &mut BitReader<'_>,
    out: &mut Vec<u8>,
    expected_len: usize,
    litlen: &Huffman,
    dist: &Huffman,
) -> Option<()> {
    loop {
        let symbol = litlen.decode(reader)?;
        if symbol < 256 {
            if out.len() >= expected_len {
                return None;
            }
            out.push(symbol as u8);
        } else if symbol == 256 {
            return Some(());
        } else if symbol <= 285 {
            let index = (symbol - 257) as usize;
            let extra = LENGTH_EXTRA[index];
            let length = LENGTH_BASE[index] as usize + reader.read_bits(extra)? as usize;
            let dist_symbol = dist.decode(reader)? as usize;
            if dist_symbol >= DIST_BASE.len() {
                return None;
            }
            let extra = DIST_EXTRA[dist_symbol];
            let distance = DIST_BASE[dist_symbol] as usize + reader.read_bits(extra)? as usize;
            copy_match(out, expected_len, distance, length)?;
        } else {
            return None;
        }
    }
}

fn copy_match(
    out: &mut Vec<u8>,
    expected_len: usize,
    distance: usize,
    length: usize,
) -> Option<()> {
    if distance == 0 || distance > out.len() || out.len().checked_add(length)? > expected_len {
        return None;
    }
    for _ in 0..length {
        let index = out.len() - distance;
        let value = *out.get(index)?;
        out.push(value);
    }
    Some(())
}

fn fixed_huffman() -> Option<(Huffman, Huffman)> {
    let mut litlen_lengths = [0u8; 288];
    for item in litlen_lengths.iter_mut().take(144) {
        *item = 8;
    }
    for item in litlen_lengths.iter_mut().take(256).skip(144) {
        *item = 9;
    }
    for item in litlen_lengths.iter_mut().take(280).skip(256) {
        *item = 7;
    }
    for item in litlen_lengths.iter_mut().skip(280) {
        *item = 8;
    }
    let dist_lengths = [5u8; 32];
    Some((Huffman::new(&litlen_lengths)?, Huffman::new(&dist_lengths)?))
}

fn dynamic_huffman(reader: &mut BitReader<'_>) -> Option<(Huffman, Huffman)> {
    let hlit = reader.read_bits(5)? as usize + 257;
    let hdist = reader.read_bits(5)? as usize + 1;
    let hclen = reader.read_bits(4)? as usize + 4;

    let mut code_lengths = [0u8; 19];
    for slot in CODE_LENGTH_ORDER.iter().take(hclen) {
        code_lengths[*slot] = reader.read_bits(3)? as u8;
    }
    let code_huffman = Huffman::new(&code_lengths)?;

    let total = hlit.checked_add(hdist)?;
    let mut lengths = Vec::new();
    lengths.resize(total, 0);
    let mut index = 0usize;
    while index < total {
        let symbol = code_huffman.decode(reader)?;
        match symbol {
            0..=15 => {
                lengths[index] = symbol as u8;
                index += 1;
            }
            16 => {
                if index == 0 {
                    return None;
                }
                let repeat = reader.read_bits(2)? as usize + 3;
                let previous = lengths[index - 1];
                repeat_length(&mut lengths, &mut index, repeat, previous)?;
            }
            17 => {
                let repeat = reader.read_bits(3)? as usize + 3;
                repeat_length(&mut lengths, &mut index, repeat, 0)?;
            }
            18 => {
                let repeat = reader.read_bits(7)? as usize + 11;
                repeat_length(&mut lengths, &mut index, repeat, 0)?;
            }
            _ => return None,
        }
    }

    let litlen = Huffman::new(lengths.get(..hlit)?)?;
    let dist = Huffman::new(lengths.get(hlit..total)?)?;
    Some((litlen, dist))
}

fn repeat_length(lengths: &mut [u8], index: &mut usize, repeat: usize, value: u8) -> Option<()> {
    if index.checked_add(repeat)? > lengths.len() {
        return None;
    }
    for _ in 0..repeat {
        lengths[*index] = value;
        *index += 1;
    }
    Some(())
}

#[derive(Clone, Copy)]
struct HuffmanEntry {
    symbol: u16,
    len: u8,
}

struct Huffman {
    table: Vec<HuffmanEntry>,
    max_bits: u8,
}

impl Huffman {
    fn new(lengths: &[u8]) -> Option<Self> {
        let max_bits = lengths.iter().copied().max().unwrap_or(0);
        if max_bits == 0 || max_bits > 15 {
            return None;
        }

        let mut counts = [0u16; 16];
        for &len in lengths {
            if len > 15 {
                return None;
            }
            if len != 0 {
                counts[len as usize] = counts[len as usize].checked_add(1)?;
            }
        }

        let mut code = 0u16;
        let mut next_code = [0u16; 16];
        for bits in 1..=15 {
            code = code.checked_add(counts[bits - 1])?.checked_shl(1)?;
            next_code[bits] = code;
        }

        let table_len = 1usize.checked_shl(max_bits as u32)?;
        let mut table = Vec::new();
        table.resize(table_len, HuffmanEntry { symbol: 0, len: 0 });

        for (symbol, &len) in lengths.iter().enumerate() {
            if len == 0 {
                continue;
            }
            let canonical = next_code[len as usize];
            next_code[len as usize] = next_code[len as usize].checked_add(1)?;
            let reversed = reverse_bits(canonical, len) as usize;
            let step = 1usize.checked_shl(len as u32)?;
            let mut index = reversed;
            while index < table_len {
                table[index] = HuffmanEntry {
                    symbol: symbol as u16,
                    len,
                };
                index = index.checked_add(step)?;
            }
        }

        Some(Self { table, max_bits })
    }

    fn decode(&self, reader: &mut BitReader<'_>) -> Option<u16> {
        let bits = reader.peek_bits_padded(self.max_bits)? as usize;
        let entry = *self.table.get(bits)?;
        if entry.len == 0 {
            return None;
        }
        reader.drop_bits(entry.len)?;
        Some(entry.symbol)
    }
}

fn reverse_bits(mut value: u16, len: u8) -> u16 {
    let mut out = 0u16;
    for _ in 0..len {
        out = (out << 1) | (value & 1);
        value >>= 1;
    }
    out
}

struct BitReader<'a> {
    bytes: &'a [u8],
    byte_pos: usize,
    bits: u32,
    bit_count: u8,
}

impl<'a> BitReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_pos: 0,
            bits: 0,
            bit_count: 0,
        }
    }

    fn read_bits(&mut self, count: u8) -> Option<u16> {
        let value = self.peek_bits(count)?;
        self.drop_bits(count)?;
        Some(value)
    }

    fn peek_bits(&mut self, count: u8) -> Option<u16> {
        self.ensure_bits(count)?;
        Some((self.bits & bit_mask(count)) as u16)
    }

    fn peek_bits_padded(&mut self, count: u8) -> Option<u16> {
        self.ensure_bits_padded(count);
        Some((self.bits & bit_mask(count)) as u16)
    }

    fn drop_bits(&mut self, count: u8) -> Option<()> {
        if self.bit_count < count {
            self.ensure_bits(count)?;
        }
        self.bits >>= count;
        self.bit_count -= count;
        Some(())
    }

    fn align_byte(&mut self) {
        let drop = self.bit_count % 8;
        self.bits >>= drop;
        self.bit_count -= drop;
    }

    fn ensure_bits(&mut self, count: u8) -> Option<()> {
        while self.bit_count < count {
            let byte = *self.bytes.get(self.byte_pos)?;
            self.byte_pos += 1;
            self.bits |= (byte as u32) << self.bit_count;
            self.bit_count += 8;
        }
        Some(())
    }

    fn ensure_bits_padded(&mut self, count: u8) {
        while self.bit_count < count {
            if let Some(byte) = self.bytes.get(self.byte_pos) {
                self.byte_pos += 1;
                self.bits |= (*byte as u32) << self.bit_count;
            }
            self.bit_count += 8;
        }
    }
}

fn bit_mask(count: u8) -> u32 {
    if count == 32 {
        u32::MAX
    } else {
        (1u32 << count) - 1
    }
}

fn unfilter_png(raw: &[u8], width: u32, height: u32, bpp: usize) -> Option<Vec<u8>> {
    let stride = (width as usize).checked_mul(bpp)?;
    let expected = stride.checked_add(1)?.checked_mul(height as usize)?;
    if raw.len() != expected {
        return None;
    }

    let mut out = Vec::new();
    out.resize(stride.checked_mul(height as usize)?, 0);

    let mut source = 0usize;
    let mut row = 0usize;
    while row < height as usize {
        let filter = *raw.get(source)?;
        source += 1;
        let row_start = row.checked_mul(stride)?;
        let prev_start = row_start.saturating_sub(stride);

        let mut i = 0usize;
        while i < stride {
            let encoded = *raw.get(source + i)?;
            let left = if i >= bpp {
                out[row_start + i - bpp]
            } else {
                0
            };
            let up = if row > 0 { out[prev_start + i] } else { 0 };
            let up_left = if row > 0 && i >= bpp {
                out[prev_start + i - bpp]
            } else {
                0
            };
            let recon = match filter {
                0 => encoded,
                1 => encoded.wrapping_add(left),
                2 => encoded.wrapping_add(up),
                3 => encoded.wrapping_add(((left as u16 + up as u16) >> 1) as u8),
                4 => encoded.wrapping_add(paeth(left, up, up_left)),
                _ => return None,
            };
            out[row_start + i] = recon;
            i += 1;
        }

        source += stride;
        row += 1;
    }

    Some(out)
}

const ADAM7_PASSES: [(u32, u32, u32, u32); 7] = [
    (0, 0, 8, 8),
    (4, 0, 8, 8),
    (0, 4, 4, 8),
    (2, 0, 4, 4),
    (0, 2, 2, 4),
    (1, 0, 2, 2),
    (0, 1, 1, 2),
];

fn adam7_raw_len(width: u32, height: u32, bpp: usize) -> Option<usize> {
    let mut total = 0usize;
    for (x0, y0, dx, dy) in ADAM7_PASSES {
        let (pass_w, pass_h) = adam7_pass_size(width, height, x0, y0, dx, dy);
        if pass_w == 0 || pass_h == 0 {
            continue;
        }
        let stride = (pass_w as usize).checked_mul(bpp)?;
        total = total.checked_add(stride.checked_add(1)?.checked_mul(pass_h as usize)?)?;
    }
    Some(total)
}

fn adam7_pass_size(width: u32, height: u32, x0: u32, y0: u32, dx: u32, dy: u32) -> (u32, u32) {
    if width <= x0 || height <= y0 {
        return (0, 0);
    }
    let pass_w = (width - x0 + dx - 1) / dx;
    let pass_h = (height - y0 + dy - 1) / dy;
    (pass_w, pass_h)
}

fn deinterlace_adam7(raw: &[u8], width: u32, height: u32, bpp: usize) -> Option<Vec<u8>> {
    let full_stride = (width as usize).checked_mul(bpp)?;
    let mut out = Vec::new();
    out.resize(full_stride.checked_mul(height as usize)?, 0);
    let mut source = 0usize;

    for (x0, y0, dx, dy) in ADAM7_PASSES {
        let (pass_w, pass_h) = adam7_pass_size(width, height, x0, y0, dx, dy);
        if pass_w == 0 || pass_h == 0 {
            continue;
        }
        let pass_stride = (pass_w as usize).checked_mul(bpp)?;
        let pass_raw_len = pass_stride.checked_add(1)?.checked_mul(pass_h as usize)?;
        let pass_raw = raw.get(source..source.checked_add(pass_raw_len)?)?;
        source = source.checked_add(pass_raw_len)?;
        let pass_pixels = unfilter_png(pass_raw, pass_w, pass_h, bpp)?;

        let mut py = 0u32;
        while py < pass_h {
            let dst_y = y0 + py * dy;
            let mut px = 0u32;
            while px < pass_w {
                let dst_x = x0 + px * dx;
                let src = (py as usize)
                    .checked_mul(pass_stride)?
                    .checked_add(px as usize * bpp)?;
                let dst = (dst_y as usize)
                    .checked_mul(full_stride)?
                    .checked_add(dst_x as usize * bpp)?;
                let mut byte = 0usize;
                while byte < bpp {
                    out[dst + byte] = *pass_pixels.get(src + byte)?;
                    byte += 1;
                }
                px += 1;
            }
            py += 1;
        }
    }

    if source == raw.len() {
        Some(out)
    } else {
        None
    }
}

fn convert_png_pixels(pixels: &[u8], source: &PngSource) -> Option<DecodedImage> {
    let width = source.width.min(u16::MAX as u32) as u16;
    let height = source.height.min(u16::MAX as u32) as u16;
    if width == 0 || height == 0 {
        return None;
    }

    if source.has_alpha() {
        let stride = (width as usize).checked_mul(4)?;
        let mut data = Vec::new();
        data.resize(stride.checked_mul(height as usize)?, 0);
        let mut offset = 0usize;
        let mut y = 0u32;
        while y < source.height {
            let mut x = 0u32;
            while x < source.width {
                let (r, g, b, a) = sample_rgba(pixels, source, x, y)?;
                data[offset] = r;
                data[offset + 1] = g;
                data[offset + 2] = b;
                data[offset + 3] = a;
                offset += 4;
                x += 1;
            }
            y += 1;
        }
        Some(DecodedImage {
            width,
            height,
            stride,
            format: ImageFormat::Rgba8888,
            data,
        })
    } else {
        let stride = (width as usize).checked_mul(2)?;
        let mut data = Vec::new();
        data.resize(stride.checked_mul(height as usize)?, 0);
        let mut offset = 0usize;
        let mut y = 0u32;
        while y < source.height {
            let mut x = 0u32;
            while x < source.width {
                let (r, g, b, _) = sample_rgba(pixels, source, x, y)?;
                let packed = rgb565(r, g, b);
                data[offset] = packed as u8;
                data[offset + 1] = (packed >> 8) as u8;
                offset += 2;
                x += 1;
            }
            y += 1;
        }
        Some(DecodedImage {
            width,
            height,
            stride,
            format: ImageFormat::Rgb565,
            data,
        })
    }
}

#[allow(dead_code)]
fn scale_cover_rgb565(
    pixels: &[u8],
    src_w: u32,
    src_h: u32,
    source: &PngSource,
    dst_w: u16,
    dst_h: u16,
) -> Option<Vec<u8>> {
    let dst_w_u32 = dst_w as u32;
    let dst_h_u32 = dst_h as u32;
    if dst_w_u32 == 0 || dst_h_u32 == 0 {
        return None;
    }

    let (crop_w, crop_h) = if dst_w_u32.checked_mul(src_h)? >= dst_h_u32.checked_mul(src_w)? {
        let crop_h = dst_h_u32
            .checked_mul(src_w)?
            .checked_div(dst_w_u32)?
            .clamp(1, src_h);
        (src_w, crop_h)
    } else {
        let crop_w = dst_w_u32
            .checked_mul(src_h)?
            .checked_div(dst_h_u32)?
            .clamp(1, src_w);
        (crop_w, src_h)
    };
    let crop_x0 = (src_w - crop_w) / 2;
    let crop_y0 = (src_h - crop_h) / 2;

    let mut out = Vec::new();
    out.resize(
        (dst_w as usize)
            .checked_mul(dst_h as usize)?
            .checked_mul(2)?,
        0,
    );

    let mut offset = 0usize;
    let mut y = 0u32;
    while y < dst_h_u32 {
        let sy = crop_y0 + (y.checked_mul(crop_h)? / dst_h_u32).min(crop_h - 1);
        let mut x = 0u32;
        while x < dst_w_u32 {
            let sx = crop_x0 + (x.checked_mul(crop_w)? / dst_w_u32).min(crop_w - 1);
            let (r, g, b, _) = sample_rgba_at(pixels, src_w, source, sx, sy)?;
            let packed = rgb565(r, g, b);
            out[offset] = packed as u8;
            out[offset + 1] = (packed >> 8) as u8;
            offset += 2;
            x += 1;
        }
        y += 1;
    }

    Some(out)
}

#[allow(dead_code)]
fn sample_rgb(
    pixels: &[u8],
    width: u32,
    x: u32,
    y: u32,
    channels: usize,
    color_type: u8,
) -> Option<(u8, u8, u8)> {
    let index = (y as usize)
        .checked_mul(width as usize)?
        .checked_add(x as usize)?
        .checked_mul(channels)?;

    match color_type {
        0 => {
            let gray = *pixels.get(index)?;
            Some((gray, gray, gray))
        }
        3 => {
            let gray = *pixels.get(index)?;
            Some((gray, gray, gray))
        }
        2 => Some((
            *pixels.get(index)?,
            *pixels.get(index + 1)?,
            *pixels.get(index + 2)?,
        )),
        4 => {
            let gray = *pixels.get(index)?;
            let alpha = *pixels.get(index + 1)?;
            Some(premul_rgb(gray, gray, gray, alpha))
        }
        6 => {
            let r = *pixels.get(index)?;
            let g = *pixels.get(index + 1)?;
            let b = *pixels.get(index + 2)?;
            let alpha = *pixels.get(index + 3)?;
            Some(premul_rgb(r, g, b, alpha))
        }
        _ => None,
    }
}

fn sample_rgba(pixels: &[u8], source: &PngSource, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
    sample_rgba_at(pixels, source.width, source, x, y)
}

fn sample_rgba_at(
    pixels: &[u8],
    width: u32,
    source: &PngSource,
    x: u32,
    y: u32,
) -> Option<(u8, u8, u8, u8)> {
    let sample_bytes = source.sample_bytes();
    let pixel_bytes = source.pixel_bytes();
    let index = (y as usize)
        .checked_mul(width as usize)?
        .checked_add(x as usize)?
        .checked_mul(pixel_bytes)?;

    match source.color_type {
        0 => {
            let gray = sample_u8(pixels, index, sample_bytes)?;
            let alpha = trns_gray_alpha(source, pixels, index).unwrap_or(255);
            Some((gray, gray, gray, alpha))
        }
        2 => Some((
            sample_u8(pixels, index, sample_bytes)?,
            sample_u8(pixels, index + sample_bytes, sample_bytes)?,
            sample_u8(pixels, index + sample_bytes * 2, sample_bytes)?,
            trns_rgb_alpha(source, pixels, index).unwrap_or(255),
        )),
        3 => {
            let palette_index = *pixels.get(index)? as usize;
            let rgb = *source.palette.get(palette_index)?;
            let alpha = source
                .transparency
                .get(palette_index)
                .copied()
                .unwrap_or(255);
            Some((rgb[0], rgb[1], rgb[2], alpha))
        }
        4 => {
            let gray = sample_u8(pixels, index, sample_bytes)?;
            let alpha = sample_u8(pixels, index + sample_bytes, sample_bytes)?;
            Some((gray, gray, gray, alpha))
        }
        6 => Some((
            sample_u8(pixels, index, sample_bytes)?,
            sample_u8(pixels, index + sample_bytes, sample_bytes)?,
            sample_u8(pixels, index + sample_bytes * 2, sample_bytes)?,
            sample_u8(pixels, index + sample_bytes * 3, sample_bytes)?,
        )),
        _ => None,
    }
}

fn sample_u8(pixels: &[u8], offset: usize, _sample_bytes: usize) -> Option<u8> {
    pixels.get(offset).copied()
}

fn sample_u16(pixels: &[u8], offset: usize, sample_bytes: usize) -> Option<u16> {
    if sample_bytes == 2 {
        Some(u16::from_be_bytes([
            *pixels.get(offset)?,
            *pixels.get(offset + 1)?,
        ]))
    } else {
        Some(*pixels.get(offset)? as u16)
    }
}

fn trns_gray_alpha(source: &PngSource, pixels: &[u8], index: usize) -> Option<u8> {
    if source.transparency.len() < 2 {
        return None;
    }
    let transparent = u16::from_be_bytes([source.transparency[0], source.transparency[1]]);
    let gray = sample_u16(pixels, index, source.sample_bytes())?;
    Some(if gray == transparent { 0 } else { 255 })
}

fn trns_rgb_alpha(source: &PngSource, pixels: &[u8], index: usize) -> Option<u8> {
    if source.transparency.len() < 6 {
        return None;
    }
    let sample_bytes = source.sample_bytes();
    let tr = u16::from_be_bytes([source.transparency[0], source.transparency[1]]);
    let tg = u16::from_be_bytes([source.transparency[2], source.transparency[3]]);
    let tb = u16::from_be_bytes([source.transparency[4], source.transparency[5]]);
    let r = sample_u16(pixels, index, sample_bytes)?;
    let g = sample_u16(pixels, index + sample_bytes, sample_bytes)?;
    let b = sample_u16(pixels, index + sample_bytes * 2, sample_bytes)?;
    Some(if r == tr && g == tg && b == tb {
        0
    } else {
        255
    })
}

#[allow(dead_code)]
fn premul_rgb(r: u8, g: u8, b: u8, alpha: u8) -> (u8, u8, u8) {
    if alpha == 255 {
        (r, g, b)
    } else {
        (
            ((r as u16 * alpha as u16) / 255) as u8,
            ((g as u16 * alpha as u16) / 255) as u8,
            ((b as u16 * alpha as u16) / 255) as u8,
        )
    }
}

fn png_channels(color_type: u8) -> Option<usize> {
    match color_type {
        0 => Some(1),
        2 => Some(3),
        3 => Some(1),
        4 => Some(2),
        6 => Some(4),
        _ => None,
    }
}

fn paeth(left: u8, up: u8, up_left: u8) -> u8 {
    let left = left as i32;
    let up = up as i32;
    let up_left = up_left as i32;
    let estimate = left + up - up_left;
    let pa = (estimate - left).abs();
    let pb = (estimate - up).abs();
    let pc = (estimate - up_left).abs();
    if pa <= pb && pa <= pc {
        left as u8
    } else if pb <= pc {
        up as u8
    } else {
        up_left as u8
    }
}

fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

fn read_be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}
