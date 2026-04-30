use crate::{jpeg::{JpegDecoder, JpegError}, png::{PngDecoder, PngError}, Color, ImageId};
use alloc::vec::Vec;

const FRAW_SIGNATURE: &[u8; 8] = b"FHREIMG1";
const FRAW_FORMAT_RGB565: u8 = 1;
const FRAW_FORMAT_RGB888: u8 = 2;
const FRAW_FORMAT_RGBA8888: u8 = 3;
const FRAW_FORMAT_A8: u8 = 4;
const FRAW_HEADER_LEN: usize = 22;
const FRAW_FLAGS_LE: u8 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageFormat {
    Rgb565,
    Rgb888,
    Rgba8888,
    A8,
}

pub type ImageResolver = fn(ImageId) -> Option<ImageView>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageView {
    pub width: u16,
    pub height: u16,
    pub stride: usize,
    pub format: ImageFormat,
    pub data: *const u8,
    pub len: usize,
}

impl ImageView {
    pub const fn new(
        width: u16,
        height: u16,
        stride: usize,
        format: ImageFormat,
        data: &'static [u8],
    ) -> Self {
        Self::from_static_slice(width, height, stride, format, data)
    }

    pub const fn from_static_slice(
        width: u16,
        height: u16,
        stride: usize,
        format: ImageFormat,
        data: &'static [u8],
    ) -> Self {
        Self {
            width,
            height,
            stride,
            format,
            data: data.as_ptr(),
            len: data.len(),
        }
    }

    pub fn from_slice(
        width: u16,
        height: u16,
        stride: usize,
        format: ImageFormat,
        data: &[u8],
    ) -> Self {
        Self {
            width,
            height,
            stride,
            format,
            data: data.as_ptr(),
            len: data.len(),
        }
    }

    fn byte_at(self, offset: usize) -> Option<u8> {
        if self.data.is_null() || offset >= self.len {
            None
        } else {
            Some(unsafe { *self.data.add(offset) })
        }
    }

    pub fn sample(self, x: u16, y: u16, opacity: u8) -> Option<Color> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let color = match self.format {
            ImageFormat::Rgb565 => {
                let offset = y as usize * self.stride + x as usize * 2;
                let lo = self.byte_at(offset)?;
                let hi = self.byte_at(offset + 1)?;
                let raw = u16::from_le_bytes([lo, hi]);
                let r = (((raw >> 11) & 0x1f) * 255 / 31) as u8;
                let g = (((raw >> 5) & 0x3f) * 255 / 63) as u8;
                let b = ((raw & 0x1f) * 255 / 31) as u8;
                Color::rgba(r, g, b, opacity)
            }
            ImageFormat::Rgb888 => {
                let offset = y as usize * self.stride + x as usize * 3;
                Color::rgba(
                    self.byte_at(offset)?,
                    self.byte_at(offset + 1)?,
                    self.byte_at(offset + 2)?,
                    opacity,
                )
            }
            ImageFormat::Rgba8888 => {
                let offset = y as usize * self.stride + x as usize * 4;
                let a = ((self.byte_at(offset + 3)? as u16 * opacity as u16) / 255) as u8;
                Color::rgba(
                    self.byte_at(offset)?,
                    self.byte_at(offset + 1)?,
                    self.byte_at(offset + 2)?,
                    a,
                )
            }
            ImageFormat::A8 => {
                let offset = y as usize * self.stride + x as usize;
                let a = ((self.byte_at(offset)? as u16 * opacity as u16) / 255) as u8;
                Color::rgba(255, 255, 255, a)
            }
        };
        Some(color)
    }

    pub fn sample_tinted(self, x: u16, y: u16, opacity: u8, tint: Color) -> Option<Color> {
        let sample = self.sample(x, y, opacity)?;
        let a = ((sample.a as u16 * tint.a as u16) / 255) as u8;
        if a == 0 {
            return None;
        }
        Some(Color::rgba(tint.r, tint.g, tint.b, a))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrawInfo {
    pub width: u16,
    pub height: u16,
    pub stride: usize,
    pub format: ImageFormat,
    pub data_len: usize,
}

pub struct FrawDecoder;

impl FrawDecoder {
    pub fn inspect(bytes: &[u8]) -> Option<FrawInfo> {
        parse_fraw_header(bytes).map(|(info, _)| info)
    }

    pub fn inspect_result(bytes: &[u8]) -> Result<FrawInfo, FrawError> {
        parse_fraw_header_result(bytes).map(|(info, _)| info)
    }

    pub fn decode(bytes: &[u8]) -> Option<crate::png::DecodedImage> {
        decode_fraw(bytes)
    }

    pub fn decode_result(bytes: &[u8]) -> Result<crate::png::DecodedImage, FrawError> {
        decode_fraw_result(bytes)
    }
}

pub fn decode_fraw(bytes: &[u8]) -> Option<crate::png::DecodedImage> {
    decode_fraw_result(bytes).ok()
}

pub fn decode_fraw_result(bytes: &[u8]) -> Result<crate::png::DecodedImage, FrawError> {
    let (info, data_start) = parse_fraw_header_result(bytes)?;
    let end = data_start
        .checked_add(info.data_len)
        .ok_or(FrawError::Truncated)?;
    let payload = bytes.get(data_start..end).ok_or(FrawError::Truncated)?;
    let mut data = Vec::new();
    data.extend_from_slice(payload);
    Ok(crate::png::DecodedImage {
        width: info.width,
        height: info.height,
        stride: info.stride,
        format: info.format,
        data,
    })
}

fn parse_fraw_header(bytes: &[u8]) -> Option<(FrawInfo, usize)> {
    parse_fraw_header_result(bytes).ok()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrawError {
    BadSignature,
    Truncated,
    UnsupportedFormat,
    UnsupportedFlags,
    InvalidDimensions,
    InvalidStride,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageDecodeErrorKind {
    Invalid,
    Truncated,
    Unsupported,
    Overflow,
}

fn parse_fraw_header_result(bytes: &[u8]) -> Result<(FrawInfo, usize), FrawError> {
    if bytes.len() < FRAW_HEADER_LEN {
        return Err(FrawError::Truncated);
    }
    if bytes.get(..FRAW_SIGNATURE.len()) != Some(FRAW_SIGNATURE) {
        return Err(FrawError::BadSignature);
    }

    let width = read_le_u16(bytes, 8).ok_or(FrawError::Truncated)?;
    let height = read_le_u16(bytes, 10).ok_or(FrawError::Truncated)?;
    let format = match *bytes.get(12).ok_or(FrawError::Truncated)? {
        FRAW_FORMAT_RGB565 => ImageFormat::Rgb565,
        FRAW_FORMAT_RGB888 => ImageFormat::Rgb888,
        FRAW_FORMAT_RGBA8888 => ImageFormat::Rgba8888,
        FRAW_FORMAT_A8 => ImageFormat::A8,
        _ => return Err(FrawError::UnsupportedFormat),
    };
    if *bytes.get(13).ok_or(FrawError::Truncated)? != FRAW_FLAGS_LE {
        return Err(FrawError::UnsupportedFlags);
    }
    let stride = read_le_u32(bytes, 14).ok_or(FrawError::Truncated)? as usize;
    let data_len = read_le_u32(bytes, 18).ok_or(FrawError::Truncated)? as usize;
    if width == 0 || height == 0 || stride == 0 {
        return Err(FrawError::InvalidDimensions);
    }

    let min_stride = match format {
        ImageFormat::Rgb565 => width as usize * 2,
        ImageFormat::Rgb888 => width as usize * 3,
        ImageFormat::Rgba8888 => width as usize * 4,
        ImageFormat::A8 => width as usize,
    };
    if stride < min_stride {
        return Err(FrawError::InvalidStride);
    }
    let needed = stride
        .checked_mul(height as usize)
        .ok_or(FrawError::InvalidStride)?;
    if needed > data_len
        || FRAW_HEADER_LEN
            .checked_add(data_len)
            .ok_or(FrawError::Truncated)?
            > bytes.len()
    {
        return Err(FrawError::Truncated);
    }

    Ok((
        FrawInfo {
            width,
            height,
            stride,
            format,
            data_len,
        },
        FRAW_HEADER_LEN,
    ))
}

pub const IMAGE_SWATCH: ImageId = ImageId(1);
pub const IMAGE_MASK_DOT: ImageId = ImageId(2);

pub fn builtin_image(id: ImageId) -> Option<ImageView> {
    match id {
        IMAGE_SWATCH => Some(ImageView::new(4, 4, 8, ImageFormat::Rgb565, &SWATCH_RGB565)),
        IMAGE_MASK_DOT => Some(ImageView::new(8, 8, 8, ImageFormat::A8, &MASK_DOT_A8)),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageCacheStats {
    pub slots: usize,
    pub bytes: usize,
    pub hits: u32,
    pub misses: u32,
    pub loads: u32,
    pub evictions: u32,
    pub load_failures: u32,
    pub decode_failures: u32,
    pub decode_invalid: u32,
    pub decode_truncated: u32,
    pub decode_unsupported: u32,
    pub decode_overflow: u32,
    pub pinned: usize,
}

impl ImageCacheStats {
    pub const fn new() -> Self {
        Self {
            slots: 0,
            bytes: 0,
            hits: 0,
            misses: 0,
            loads: 0,
            evictions: 0,
            load_failures: 0,
            decode_failures: 0,
            decode_invalid: 0,
            decode_truncated: 0,
            decode_unsupported: 0,
            decode_overflow: 0,
            pinned: 0,
        }
    }
}

pub trait ResourceLoader {
    fn load(&mut self, path: &[u8], out: &mut Vec<u8>) -> bool;
}

struct ImageCacheSlot {
    id: ImageId,
    image: crate::png::DecodedImage,
    last_used: u32,
    pinned: bool,
}

pub struct ImageCache {
    slots: Vec<ImageCacheSlot>,
    max_slots: usize,
    max_bytes: usize,
    tick: u32,
    stats: ImageCacheStats,
}

impl ImageCache {
    pub fn new(max_slots: usize, max_bytes: usize) -> Self {
        Self {
            slots: Vec::new(),
            max_slots,
            max_bytes,
            tick: 0,
            stats: ImageCacheStats::new(),
        }
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.tick = 0;
        self.stats = ImageCacheStats::new();
    }

    pub fn stats(&self) -> ImageCacheStats {
        let mut stats = self.stats;
        stats.slots = self.slots.len();
        stats.bytes = self.bytes_used();
        stats.pinned = self.pinned_count();
        stats
    }

    pub fn view(&mut self, id: ImageId) -> Option<ImageView> {
        self.tick = self.tick.wrapping_add(1);
        if let Some(index) = self.find_index(id) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            self.slots[index].last_used = self.tick;
            Some(self.slots[index].image.view())
        } else {
            self.stats.misses = self.stats.misses.saturating_add(1);
            None
        }
    }

    pub fn get_or_load<L: ResourceLoader>(
        &mut self,
        id: ImageId,
        path: &[u8],
        loader: &mut L,
    ) -> Option<ImageView> {
        self.load_internal(id, path, loader, false)
    }

    pub fn prewarm<L: ResourceLoader>(
        &mut self,
        id: ImageId,
        path: &[u8],
        loader: &mut L,
        pinned: bool,
    ) -> bool {
        self.load_internal(id, path, loader, pinned).is_some()
    }

    fn load_internal<L: ResourceLoader>(
        &mut self,
        id: ImageId,
        path: &[u8],
        loader: &mut L,
        pinned: bool,
    ) -> Option<ImageView> {
        if let Some(view) = self.view(id) {
            if pinned {
                if let Some(index) = self.find_index(id) {
                    self.slots[index].pinned = true;
                }
            }
            return Some(view);
        }

        let mut bytes = Vec::new();
        if !loader.load(path, &mut bytes) {
            self.stats.load_failures = self.stats.load_failures.saturating_add(1);
            return None;
        }

        let image = match decode_resource_image_result(bytes.as_slice()) {
            Ok(image) => image,
            Err(kind) => {
                self.record_decode_error(kind);
                return None;
            }
        };

        let image_bytes = image.byte_len();
        if image_bytes > self.max_bytes || self.max_slots == 0 {
            self.record_decode_error(ImageDecodeErrorKind::Overflow);
            return None;
        }

        self.evict_until(image_bytes);
        if self.slots.len() >= self.max_slots {
            if !self.evict_one() {
                self.record_decode_error(ImageDecodeErrorKind::Overflow);
                return None;
            }
        }

        self.tick = self.tick.wrapping_add(1);
        self.slots.push(ImageCacheSlot {
            id,
            image,
            last_used: self.tick,
            pinned,
        });
        self.stats.loads = self.stats.loads.saturating_add(1);
        self.view(id)
    }

    fn record_decode_error(&mut self, kind: ImageDecodeErrorKind) {
        self.stats.decode_failures = self.stats.decode_failures.saturating_add(1);
        match kind {
            ImageDecodeErrorKind::Invalid => {
                self.stats.decode_invalid = self.stats.decode_invalid.saturating_add(1);
            }
            ImageDecodeErrorKind::Truncated => {
                self.stats.decode_truncated = self.stats.decode_truncated.saturating_add(1);
            }
            ImageDecodeErrorKind::Unsupported => {
                self.stats.decode_unsupported = self.stats.decode_unsupported.saturating_add(1);
            }
            ImageDecodeErrorKind::Overflow => {
                self.stats.decode_overflow = self.stats.decode_overflow.saturating_add(1);
            }
        }
    }

    fn find_index(&self, id: ImageId) -> Option<usize> {
        let mut index = 0usize;
        while index < self.slots.len() {
            if self.slots[index].id == id {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    fn bytes_used(&self) -> usize {
        let mut bytes = 0usize;
        let mut index = 0usize;
        while index < self.slots.len() {
            bytes = bytes.saturating_add(self.slots[index].image.byte_len());
            index += 1;
        }
        bytes
    }

    fn pinned_count(&self) -> usize {
        let mut count = 0usize;
        let mut index = 0usize;
        while index < self.slots.len() {
            if self.slots[index].pinned {
                count += 1;
            }
            index += 1;
        }
        count
    }

    fn evict_until(&mut self, incoming: usize) {
        while !self.slots.is_empty()
            && self.bytes_used().saturating_add(incoming) > self.max_bytes
        {
            if !self.evict_one() {
                break;
            }
        }
    }

    fn evict_one(&mut self) -> bool {
        if self.slots.is_empty() {
            return false;
        }

        let mut oldest: Option<usize> = None;
        let mut index = 0usize;
        while index < self.slots.len() {
            if !self.slots[index].pinned {
                match oldest {
                    Some(oldest_index)
                        if self.slots[index].last_used >= self.slots[oldest_index].last_used => {}
                    _ => oldest = Some(index),
                }
            }
            index += 1;
        }
        let Some(oldest) = oldest else {
            return false;
        };
        self.slots.remove(oldest);
        self.stats.evictions = self.stats.evictions.saturating_add(1);
        true
    }
}

fn decode_resource_image_result(bytes: &[u8]) -> Result<crate::png::DecodedImage, ImageDecodeErrorKind> {
    if bytes.get(..FRAW_SIGNATURE.len()) == Some(FRAW_SIGNATURE) {
        FrawDecoder::decode_result(bytes).map_err(map_fraw_error)
    } else if bytes.get(..2) == Some(b"\xff\xd8") {
        JpegDecoder::decode_result(bytes).map_err(map_jpeg_error)
    } else {
        PngDecoder::decode_result(bytes).map_err(map_png_error)
    }
}

fn map_fraw_error(error: FrawError) -> ImageDecodeErrorKind {
    match error {
        FrawError::Truncated => ImageDecodeErrorKind::Truncated,
        FrawError::UnsupportedFormat | FrawError::UnsupportedFlags => ImageDecodeErrorKind::Unsupported,
        FrawError::BadSignature | FrawError::InvalidDimensions | FrawError::InvalidStride => {
            ImageDecodeErrorKind::Invalid
        }
    }
}

fn map_png_error(error: PngError) -> ImageDecodeErrorKind {
    match error {
        PngError::Truncated | PngError::MissingImageData => ImageDecodeErrorKind::Truncated,
        PngError::Unsupported
        | PngError::UnsupportedBitDepth
        | PngError::UnsupportedColorType
        | PngError::UnsupportedCompression
        | PngError::UnsupportedFilter
        | PngError::UnsupportedInterlace
        | PngError::MissingPalette => ImageDecodeErrorKind::Unsupported,
        PngError::Overflow => ImageDecodeErrorKind::Overflow,
        PngError::BadSignature
        | PngError::InvalidChunk
        | PngError::InvalidPalette
        | PngError::InvalidDimensions
        | PngError::Decode => ImageDecodeErrorKind::Invalid,
    }
}

fn map_jpeg_error(error: JpegError) -> ImageDecodeErrorKind {
    match error {
        JpegError::Truncated => ImageDecodeErrorKind::Truncated,
        JpegError::Unsupported
        | JpegError::UnsupportedProgressive
        | JpegError::UnsupportedRestart => ImageDecodeErrorKind::Unsupported,
        JpegError::BadSignature
        | JpegError::UnexpectedRestartMarker
        | JpegError::InvalidTable
        | JpegError::Decode => {
            ImageDecodeErrorKind::Invalid
        }
    }
}

fn read_le_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
    ]))
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes([
        *bytes.get(offset)?,
        *bytes.get(offset + 1)?,
        *bytes.get(offset + 2)?,
        *bytes.get(offset + 3)?,
    ]))
}

const SWATCH_RGB565: [u8; 32] = [
    0x1f, 0x04, 0xff, 0x07, 0xff, 0x5f, 0x1f, 0xf8,
    0xff, 0x07, 0x9f, 0x4f, 0x1f, 0xf8, 0xff, 0xff,
    0xff, 0x5f, 0x1f, 0xf8, 0xff, 0xff, 0xff, 0x07,
    0x1f, 0xf8, 0xff, 0xff, 0xff, 0x07, 0x1f, 0x04,
];

const MASK_DOT_A8: [u8; 64] = [
    0, 0, 18, 80, 80, 18, 0, 0,
    0, 42, 160, 228, 228, 160, 42, 0,
    18, 160, 255, 255, 255, 255, 160, 18,
    80, 228, 255, 255, 255, 255, 228, 80,
    80, 228, 255, 255, 255, 255, 228, 80,
    18, 160, 255, 255, 255, 255, 160, 18,
    0, 42, 160, 228, 228, 160, 42, 0,
    0, 0, 18, 80, 80, 18, 0, 0,
];
