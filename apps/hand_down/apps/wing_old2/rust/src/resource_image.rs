use alloc::vec::Vec;

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
    131, 163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025,
    1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12,
    12, 13, 13,
];
const CODE_LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

#[derive(Debug)]
pub struct DecodedRgb565 {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

pub fn decode_png_rgb565_cover(bytes: &[u8], width: u16, height: u16) -> Option<DecodedRgb565> {
    let source = parse_png(bytes)?;
    let raw = inflate_zlib(&source.idat, source.raw_len()?)?;
    let pixels = unfilter_png(&raw, source.width, source.height, source.channels())?;
    Some(DecodedRgb565 {
        width,
        height,
        data: scale_cover_rgb565(
            &pixels,
            source.width,
            source.height,
            source.color_type,
            width,
            height,
        )?,
    })
}

pub const fn png_runtime_decode_enabled() -> bool {
    true
}

struct PngSource {
    width: u32,
    height: u32,
    color_type: u8,
    idat: Vec<u8>,
}

impl PngSource {
    fn channels(&self) -> usize {
        png_channels(self.color_type).unwrap_or(0)
    }

    fn raw_len(&self) -> Option<usize> {
        let stride = (self.width as usize).checked_mul(self.channels())?;
        stride.checked_add(1)?.checked_mul(self.height as usize)
    }
}

fn parse_png(bytes: &[u8]) -> Option<PngSource> {
    if bytes.get(..PNG_SIGNATURE.len())? != PNG_SIGNATURE {
        return None;
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

    while offset.checked_add(8)? <= bytes.len() {
        let len = read_be_u32(bytes, offset)? as usize;
        let chunk_type = bytes.get(offset + 4..offset + 8)?;
        let chunk_start = offset.checked_add(8)?;
        let chunk_end = chunk_start.checked_add(len)?;
        let crc_end = chunk_end.checked_add(4)?;
        if crc_end > bytes.len() {
            return None;
        }

        let chunk = bytes.get(chunk_start..chunk_end)?;
        offset = crc_end;

        match chunk_type {
            b"IHDR" => {
                if chunk.len() != 13 {
                    return None;
                }
                width = read_be_u32(chunk, 0)?;
                height = read_be_u32(chunk, 4)?;
                bit_depth = *chunk.get(8)?;
                color_type = *chunk.get(9)?;
                compression = *chunk.get(10)?;
                filter = *chunk.get(11)?;
                interlace = *chunk.get(12)?;
            }
            b"IDAT" => idat.extend_from_slice(chunk),
            b"IEND" => break,
            _ => {}
        }
    }

    if width == 0
        || height == 0
        || bit_depth != 8
        || png_channels(color_type).is_none()
        || compression != 0
        || filter != 0
        || interlace != 0
        || idat.is_empty()
    {
        return None;
    }

    Some(PngSource {
        width,
        height,
        color_type,
        idat,
    })
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

fn copy_match(out: &mut Vec<u8>, expected_len: usize, distance: usize, length: usize) -> Option<()> {
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
        table.resize(
            table_len,
            HuffmanEntry {
                symbol: 0,
                len: 0,
            },
        );

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
            let left = if i >= bpp { out[row_start + i - bpp] } else { 0 };
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

fn scale_cover_rgb565(
    pixels: &[u8],
    src_w: u32,
    src_h: u32,
    color_type: u8,
    dst_w: u16,
    dst_h: u16,
) -> Option<Vec<u8>> {
    let channels = png_channels(color_type)?;
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
    out.resize((dst_w as usize).checked_mul(dst_h as usize)?.checked_mul(2)?, 0);

    let mut offset = 0usize;
    let mut y = 0u32;
    while y < dst_h_u32 {
        let sy = crop_y0 + (y.checked_mul(crop_h)? / dst_h_u32).min(crop_h - 1);
        let mut x = 0u32;
        while x < dst_w_u32 {
            let sx = crop_x0 + (x.checked_mul(crop_w)? / dst_w_u32).min(crop_w - 1);
            let (r, g, b) = sample_rgb(pixels, src_w, sx, sy, channels, color_type)?;
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
