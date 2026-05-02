use crate::{
    backend::{
        CodecAcceleratorCapabilities, CodecPipelineJob, CodecPipelinePlan, CodecPipelineStats,
        CodecStageKind, CodecStagePlan, DEFAULT_CODEC_PIPELINE_STAGES,
    },
    jpeg::{JpegDecoder, JpegError},
    png::{PngDecoder, PngError},
    CodecErrorKind, Color, ImageId,
};
use alloc::vec::Vec;

const FRAW_SIGNATURE: &[u8; 8] = b"FHREIMG1";
const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
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

    pub fn plan_pipeline<const STAGES: usize>() -> CodecPipelinePlan<STAGES> {
        Self::plan_pipeline_with_caps(CodecAcceleratorCapabilities::NONE)
    }

    pub fn plan_pipeline_with_caps<const STAGES: usize>(
        caps: CodecAcceleratorCapabilities,
    ) -> CodecPipelinePlan<STAGES> {
        let supported = caps.fraw && caps.max_stages >= 5;
        let mut plan = CodecPipelinePlan::new();
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Read, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Inspect, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Header, true, supported));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Pack, true, supported));
        let _ = plan.push(CodecStagePlan::new(
            CodecStageKind::CacheInsert,
            false,
            true,
        ));
        plan
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
    Missing,
    Invalid,
    Truncated,
    Unsupported,
    Overflow,
}

impl ImageDecodeErrorKind {
    pub const fn as_codec_error(self) -> CodecErrorKind {
        match self {
            Self::Missing => CodecErrorKind::MissingResource,
            Self::Invalid => CodecErrorKind::Invalid,
            Self::Truncated => CodecErrorKind::Truncated,
            Self::Unsupported => CodecErrorKind::Unsupported,
            Self::Overflow => CodecErrorKind::Overflow,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceDecodeResult {
    Rendered(ImageView),
    Placeholder(ImageView, ImageDecodeErrorKind),
    Failed(CodecErrorKind),
}

impl ResourceDecodeResult {
    pub const fn is_rendered(self) -> bool {
        matches!(self, Self::Rendered(_))
    }

    pub const fn is_placeholder(self) -> bool {
        matches!(self, Self::Placeholder(_, _))
    }

    pub const fn is_failed(self) -> bool {
        matches!(self, Self::Failed(_))
    }

    pub const fn as_result(self) -> Result<ImageView, CodecErrorKind> {
        match self {
            Self::Rendered(view) => Ok(view),
            Self::Placeholder(_, kind) => Err(kind.as_codec_error()),
            Self::Failed(kind) => Err(kind),
        }
    }

    pub const fn error_kind(self) -> Option<CodecErrorKind> {
        match self {
            Self::Rendered(_) => None,
            Self::Placeholder(_, kind) => Some(kind.as_codec_error()),
            Self::Failed(kind) => Some(kind),
        }
    }

    pub const fn placeholder_kind(self) -> Option<ImageDecodeErrorKind> {
        match self {
            Self::Placeholder(_, kind) => Some(kind),
            _ => None,
        }
    }

    pub const fn image(self) -> Option<ImageView> {
        match self {
            Self::Rendered(view) | Self::Placeholder(view, _) => Some(view),
            Self::Failed(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageResourceKind {
    Fraw,
    Png,
    Jpeg,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageResourceInfo {
    pub kind: ImageResourceKind,
    pub width: u16,
    pub height: u16,
    pub stride: usize,
    pub format: ImageFormat,
    pub data_len: usize,
    pub has_alpha: bool,
    pub interlaced: bool,
    pub progressive: bool,
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
    pub decode_placeholders: u32,
    pub decode_placeholder_missing: u32,
    pub decode_placeholder_invalid: u32,
    pub decode_placeholder_truncated: u32,
    pub decode_placeholder_unsupported: u32,
    pub decode_placeholder_overflow: u32,
    pub pipeline_candidates: u32,
    pub pipeline_stages: u32,
    pub pipeline_hardware_candidates: u32,
    pub pipeline_fallbacks: u32,
    pub pipeline_unsupported: u32,
    pub pipeline_overflows: u32,
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
            decode_placeholders: 0,
            decode_placeholder_missing: 0,
            decode_placeholder_invalid: 0,
            decode_placeholder_truncated: 0,
            decode_placeholder_unsupported: 0,
            decode_placeholder_overflow: 0,
            pipeline_candidates: 0,
            pipeline_stages: 0,
            pipeline_hardware_candidates: 0,
            pipeline_fallbacks: 0,
            pipeline_unsupported: 0,
            pipeline_overflows: 0,
            pinned: 0,
        }
    }

    pub fn record_pipeline(&mut self, stats: CodecPipelineStats) {
        self.pipeline_candidates = self.pipeline_candidates.saturating_add(stats.candidates);
        self.pipeline_stages = self.pipeline_stages.saturating_add(stats.stages);
        self.pipeline_hardware_candidates = self
            .pipeline_hardware_candidates
            .saturating_add(stats.hardware_candidates);
        self.pipeline_fallbacks = self.pipeline_fallbacks.saturating_add(stats.fallbacks);
        self.pipeline_unsupported = self.pipeline_unsupported.saturating_add(stats.unsupported);
        self.pipeline_overflows = self.pipeline_overflows.saturating_add(stats.overflows);
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
        match self.load_internal_result(id, path, loader, false) {
            ResourceDecodeResult::Rendered(view) | ResourceDecodeResult::Placeholder(view, _) => {
                Some(view)
            }
            ResourceDecodeResult::Failed(_) => None,
        }
    }

    pub fn get_or_load_result<L: ResourceLoader>(
        &mut self,
        id: ImageId,
        path: &[u8],
        loader: &mut L,
    ) -> Result<ImageView, CodecErrorKind> {
        self.load_internal_result(id, path, loader, false)
            .as_result()
    }

    pub fn prewarm<L: ResourceLoader>(
        &mut self,
        id: ImageId,
        path: &[u8],
        loader: &mut L,
        pinned: bool,
    ) -> bool {
        matches!(
            self.load_internal_result(id, path, loader, pinned),
            ResourceDecodeResult::Rendered(_)
        )
    }

    pub fn prewarm_result<L: ResourceLoader>(
        &mut self,
        id: ImageId,
        path: &[u8],
        loader: &mut L,
        pinned: bool,
    ) -> Result<ImageView, CodecErrorKind> {
        self.load_internal_result(id, path, loader, pinned)
            .as_result()
    }

    pub fn get_or_load_with_placeholder<L: ResourceLoader>(
        &mut self,
        id: ImageId,
        path: &[u8],
        loader: &mut L,
    ) -> ResourceDecodeResult {
        self.load_internal_result(id, path, loader, false)
    }

    pub fn prewarm_with_placeholder<L: ResourceLoader>(
        &mut self,
        id: ImageId,
        path: &[u8],
        loader: &mut L,
        pinned: bool,
    ) -> ResourceDecodeResult {
        self.load_internal_result(id, path, loader, pinned)
    }

    fn load_internal_result<L: ResourceLoader>(
        &mut self,
        id: ImageId,
        path: &[u8],
        loader: &mut L,
        pinned: bool,
    ) -> ResourceDecodeResult {
        if let Some(view) = self.view(id) {
            if pinned {
                if let Some(index) = self.find_index(id) {
                    self.slots[index].pinned = true;
                }
            }
            return ResourceDecodeResult::Rendered(view);
        }

        let mut bytes = Vec::new();
        if !loader.load(path, &mut bytes) {
            self.stats.load_failures = self.stats.load_failures.saturating_add(1);
            return self.make_decode_placeholder(ImageDecodeErrorKind::Missing);
        }

        let plan = plan_resource_image_pipeline::<DEFAULT_CODEC_PIPELINE_STAGES>(bytes.as_slice());
        self.stats.record_pipeline(plan.stats());

        let image = match decode_resource_image_result(bytes.as_slice()) {
            Ok(image) => image,
            Err(kind) => {
                self.record_decode_error(kind);
                return self.make_decode_placeholder(kind);
            }
        };

        let image_bytes = image.byte_len();
        if image_bytes > self.max_bytes || self.max_slots == 0 {
            self.record_decode_error(ImageDecodeErrorKind::Overflow);
            return self.make_decode_placeholder(ImageDecodeErrorKind::Overflow);
        }

        self.evict_until(image_bytes);
        if self.slots.len() >= self.max_slots {
            if !self.evict_one() {
                self.record_decode_error(ImageDecodeErrorKind::Overflow);
                return self.make_decode_placeholder(ImageDecodeErrorKind::Overflow);
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
        match self.view(id) {
            Some(view) => ResourceDecodeResult::Rendered(view),
            None => self.make_decode_placeholder(ImageDecodeErrorKind::Overflow),
        }
    }

    fn make_decode_placeholder(&mut self, reason: ImageDecodeErrorKind) -> ResourceDecodeResult {
        let Some(placeholder) = builtin_image(IMAGE_SWATCH) else {
            return ResourceDecodeResult::Failed(reason.as_codec_error());
        };
        self.record_decode_placeholder(reason);
        ResourceDecodeResult::Placeholder(placeholder, reason)
    }

    fn record_decode_placeholder(&mut self, kind: ImageDecodeErrorKind) {
        self.stats.decode_placeholders = self.stats.decode_placeholders.saturating_add(1);
        match kind {
            ImageDecodeErrorKind::Missing => {
                self.stats.decode_placeholder_missing =
                    self.stats.decode_placeholder_missing.saturating_add(1);
            }
            ImageDecodeErrorKind::Invalid => {
                self.stats.decode_placeholder_invalid =
                    self.stats.decode_placeholder_invalid.saturating_add(1);
            }
            ImageDecodeErrorKind::Truncated => {
                self.stats.decode_placeholder_truncated =
                    self.stats.decode_placeholder_truncated.saturating_add(1);
            }
            ImageDecodeErrorKind::Unsupported => {
                self.stats.decode_placeholder_unsupported =
                    self.stats.decode_placeholder_unsupported.saturating_add(1);
            }
            ImageDecodeErrorKind::Overflow => {
                self.stats.decode_placeholder_overflow =
                    self.stats.decode_placeholder_overflow.saturating_add(1);
            }
        }
    }

    fn record_decode_error(&mut self, kind: ImageDecodeErrorKind) {
        self.stats.decode_failures = self.stats.decode_failures.saturating_add(1);
        match kind {
            ImageDecodeErrorKind::Missing => {}
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
        while !self.slots.is_empty() && self.bytes_used().saturating_add(incoming) > self.max_bytes
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

pub fn inspect_resource_image(bytes: &[u8]) -> Option<ImageResourceInfo> {
    inspect_resource_image_result(bytes).ok()
}

pub fn inspect_resource_image_result(
    bytes: &[u8],
) -> Result<ImageResourceInfo, ImageDecodeErrorKind> {
    if bytes.get(..FRAW_SIGNATURE.len()) == Some(FRAW_SIGNATURE) {
        let info = FrawDecoder::inspect_result(bytes).map_err(map_fraw_error)?;
        Ok(ImageResourceInfo {
            kind: ImageResourceKind::Fraw,
            width: info.width,
            height: info.height,
            stride: info.stride,
            format: info.format,
            data_len: info.data_len,
            has_alpha: matches!(info.format, ImageFormat::Rgba8888 | ImageFormat::A8),
            interlaced: false,
            progressive: false,
        })
    } else if bytes.get(..2) == Some(b"\xff\xd8") {
        let info = JpegDecoder::inspect_result(bytes).map_err(map_jpeg_error)?;
        let (stride, data_len) = image_resource_size(info.width, info.height, 2)?;
        Ok(ImageResourceInfo {
            kind: ImageResourceKind::Jpeg,
            width: info.width,
            height: info.height,
            stride,
            format: ImageFormat::Rgb565,
            data_len,
            has_alpha: false,
            interlaced: false,
            progressive: info.progressive,
        })
    } else if bytes.get(..PNG_SIGNATURE.len()) == Some(PNG_SIGNATURE) {
        let info = PngDecoder::inspect_result(bytes).map_err(map_png_error)?;
        let format = if info.has_alpha {
            ImageFormat::Rgba8888
        } else {
            ImageFormat::Rgb565
        };
        let bytes_per_pixel = if info.has_alpha { 4 } else { 2 };
        let (stride, data_len) = image_resource_size(info.width, info.height, bytes_per_pixel)?;
        Ok(ImageResourceInfo {
            kind: ImageResourceKind::Png,
            width: info.width,
            height: info.height,
            stride,
            format,
            data_len,
            has_alpha: info.has_alpha,
            interlaced: info.interlaced,
            progressive: false,
        })
    } else if bytes.is_empty() {
        Err(ImageDecodeErrorKind::Truncated)
    } else {
        Err(ImageDecodeErrorKind::Unsupported)
    }
}

pub fn plan_resource_image_pipeline<const STAGES: usize>(
    bytes: &[u8],
) -> CodecPipelinePlan<STAGES> {
    plan_resource_image_pipeline_with_caps(bytes, CodecAcceleratorCapabilities::NONE)
}

pub fn plan_resource_image_pipeline_with_caps<const STAGES: usize>(
    bytes: &[u8],
    caps: CodecAcceleratorCapabilities,
) -> CodecPipelinePlan<STAGES> {
    if bytes.get(..FRAW_SIGNATURE.len()) == Some(FRAW_SIGNATURE) {
        FrawDecoder::plan_pipeline_with_caps(caps)
    } else if bytes.get(..2) == Some(b"\xff\xd8") {
        JpegDecoder::plan_pipeline_with_caps(caps)
    } else if bytes.get(..PNG_SIGNATURE.len()) == Some(PNG_SIGNATURE) {
        PngDecoder::plan_pipeline_with_caps(caps)
    } else {
        let mut plan = CodecPipelinePlan::new();
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Read, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Inspect, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Fallback, false, true));
        plan
    }
}

pub fn plan_resource_image_pipeline_job<const STAGES: usize>(
    bytes: &[u8],
    caps: CodecAcceleratorCapabilities,
) -> CodecPipelineJob<STAGES> {
    CodecPipelineJob::new(plan_resource_image_pipeline_with_caps(bytes, caps), caps)
}

fn image_resource_size(
    width: u16,
    height: u16,
    bytes_per_pixel: usize,
) -> Result<(usize, usize), ImageDecodeErrorKind> {
    let stride = (width as usize)
        .checked_mul(bytes_per_pixel)
        .ok_or(ImageDecodeErrorKind::Overflow)?;
    let data_len = stride
        .checked_mul(height as usize)
        .ok_or(ImageDecodeErrorKind::Overflow)?;
    Ok((stride, data_len))
}

pub fn decode_resource_image(bytes: &[u8]) -> Option<crate::png::DecodedImage> {
    decode_resource_image_result(bytes).ok()
}

pub fn decode_resource_image_result(
    bytes: &[u8],
) -> Result<crate::png::DecodedImage, ImageDecodeErrorKind> {
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
        FrawError::UnsupportedFormat | FrawError::UnsupportedFlags => {
            ImageDecodeErrorKind::Unsupported
        }
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
        | JpegError::Decode => ImageDecodeErrorKind::Invalid,
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
    0x1f, 0x04, 0xff, 0x07, 0xff, 0x5f, 0x1f, 0xf8, 0xff, 0x07, 0x9f, 0x4f, 0x1f, 0xf8, 0xff, 0xff,
    0xff, 0x5f, 0x1f, 0xf8, 0xff, 0xff, 0xff, 0x07, 0x1f, 0xf8, 0xff, 0xff, 0xff, 0x07, 0x1f, 0x04,
];

const MASK_DOT_A8: [u8; 64] = [
    0, 0, 18, 80, 80, 18, 0, 0, 0, 42, 160, 228, 228, 160, 42, 0, 18, 160, 255, 255, 255, 255, 160,
    18, 80, 228, 255, 255, 255, 255, 228, 80, 80, 228, 255, 255, 255, 255, 228, 80, 18, 160, 255,
    255, 255, 255, 160, 18, 0, 42, 160, 228, 228, 160, 42, 0, 0, 0, 18, 80, 80, 18, 0, 0,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::{
        CodecPipelineJobError, CodecPipelineJobState, CodecPipelineJobToken, MockCodecBackend,
        MockCodecBackendFailure,
    };
    use alloc::vec::Vec;

    struct TestLoader {
        payload: Option<&'static [u8]>,
    }

    impl TestLoader {
        fn with_payload(payload: &'static [u8]) -> Self {
            Self {
                payload: Some(payload),
            }
        }

        fn missing() -> Self {
            Self { payload: None }
        }
    }

    impl ResourceLoader for TestLoader {
        fn load(&mut self, _path: &[u8], out: &mut Vec<u8>) -> bool {
            match self.payload {
                Some(payload) => {
                    out.extend_from_slice(payload);
                    true
                }
                None => false,
            }
        }
    }

    const TEST_FRAW_2X2_RGB565: [u8; 30] = [
        70, 72, 82, 69, 73, 77, 71, 49, 2, 0, 2, 0, 1, 0, 4, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    const TEST_FRAW_INVALID_DIMENSIONS: [u8; 22] = [
        70, 72, 82, 69, 73, 77, 71, 49, 0, 0, 1, 0, 1, 0, 4, 0, 0, 0, 0, 0, 0, 0,
    ];

    #[test]
    fn prewarm_result_reports_missing_and_placeholder() {
        let mut cache = ImageCache::new(2, 1024);
        let mut loader = TestLoader::missing();
        let result = cache.prewarm_result(ImageId(100), b"missing.bin\0", &mut loader, false);
        assert_eq!(result, Err(CodecErrorKind::MissingResource));
        let stats = cache.stats();
        assert_eq!(stats.load_failures, 1);
        assert_eq!(stats.decode_failures, 0);
        assert_eq!(stats.decode_placeholders, 1);
        assert_eq!(stats.decode_placeholder_missing, 1);
    }

    #[test]
    fn get_or_load_with_placeholder_keeps_cached_result() {
        let mut cache = ImageCache::new(2, 1024);
        let mut loader = TestLoader::with_payload(&TEST_FRAW_2X2_RGB565);
        let first = cache
            .prewarm_result(ImageId(101), b"valid.fraw\0", &mut loader, false)
            .expect("first preload should succeed");

        let mut miss_loader = TestLoader::missing();
        let second = cache.get_or_load_with_placeholder(
            ImageId(101),
            b"missing_later.fraw\0",
            &mut miss_loader,
        );

        match second {
            ResourceDecodeResult::Rendered(view) => {
                assert_eq!(view.width, first.width);
                assert_eq!(view.height, first.height);
            }
            _ => panic!("expected cache hit to return rendered view"),
        }

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
    }

    #[test]
    fn placeholder_path_stays_distinguishable_from_rendered() {
        let mut cache = ImageCache::new(2, 1024);
        let mut loader = TestLoader::with_payload(&TEST_FRAW_INVALID_DIMENSIONS);
        let result = cache.prewarm_result(ImageId(102), b"invalid.bin\0", &mut loader, false);
        let err = match result {
            Err(err) => err,
            Ok(_) => panic!("invalid image bytes should not parse"),
        };
        assert_eq!(err, CodecErrorKind::Invalid);
        assert_eq!(cache.stats().decode_placeholder_invalid, 1);
        let mut loader = TestLoader::missing();
        let fallback =
            cache.get_or_load_with_placeholder(ImageId(102), b"missing.bin\0", &mut loader);
        match fallback {
            ResourceDecodeResult::Placeholder(view, _) => {
                assert_eq!(view.len > 0, true);
            }
            _ => panic!("expected placeholder after decode failure"),
        }
    }

    #[test]
    fn prewarm_result_records_pipeline_fallback() {
        let mut cache = ImageCache::new(2, 1024);
        const TEST_JPEG_UNSUPPORTED: [u8; 2] = [0xff, 0xd8];
        let mut loader = TestLoader::with_payload(&TEST_JPEG_UNSUPPORTED);
        let result = cache.prewarm_result(ImageId(103), b"unsupported.jpg\0", &mut loader, false);
        assert_eq!(result, Err(CodecErrorKind::Unsupported));

        let stats = cache.stats();
        assert_eq!(stats.pipeline_candidates, 1);
        assert!(stats.pipeline_stages > 0);
        assert!(stats.pipeline_fallbacks > 0);
        assert!(stats.pipeline_unsupported > 0);
        assert_eq!(stats.decode_placeholders, 1);
        assert_eq!(stats.decode_placeholder_unsupported, 1);
    }

    #[test]
    fn resource_decode_result_exposes_placeholder_reason() {
        let mut cache = ImageCache::new(2, 1024);
        let mut loader = TestLoader::missing();
        let result =
            cache.get_or_load_with_placeholder(ImageId(104), b"missing.bin\0", &mut loader);

        assert!(result.is_placeholder());
        assert_eq!(
            result.placeholder_kind(),
            Some(ImageDecodeErrorKind::Missing)
        );
        assert_eq!(result.error_kind(), Some(CodecErrorKind::MissingResource));
        assert!(result.image().is_some());
    }

    #[test]
    fn resource_image_pipeline_marks_hardware_candidates_when_caps_allow() {
        let caps = CodecAcceleratorCapabilities::new(false, false, true, false, false, 5);
        let plan = plan_resource_image_pipeline_with_caps::<DEFAULT_CODEC_PIPELINE_STAGES>(
            &TEST_FRAW_2X2_RGB565,
            caps,
        );
        let stats = plan.stats();

        assert_eq!(stats.candidates, 1);
        assert!(stats.hardware_candidates > 0);
        assert_eq!(stats.unsupported, 0);
        assert_eq!(stats.fallbacks, 0);
    }

    #[test]
    fn resource_image_pipeline_job_runs_supported_state_flow() {
        let caps = CodecAcceleratorCapabilities::new(false, false, true, false, false, 5);
        let mut job = plan_resource_image_pipeline_job::<DEFAULT_CODEC_PIPELINE_STAGES>(
            &TEST_FRAW_2X2_RGB565,
            caps,
        );

        assert_eq!(job.state(), CodecPipelineJobState::Planned);
        assert_eq!(job.capabilities(), caps);
        assert!(job.stats().hardware_candidates > 0);
        assert_eq!(job.prepare(), Ok(()));
        assert_eq!(job.state(), CodecPipelineJobState::Prepared);
        assert_eq!(job.submit(), Ok(()));
        assert_eq!(job.state(), CodecPipelineJobState::Submitted);
        assert_eq!(job.complete(), Ok(()));
        assert_eq!(job.state(), CodecPipelineJobState::Completed);
        assert_eq!(job.submit(), Err(CodecPipelineJobError::InvalidTransition));
    }

    #[test]
    fn resource_image_pipeline_job_falls_back_when_caps_do_not_support() {
        let caps = CodecAcceleratorCapabilities::NONE;
        let mut job = plan_resource_image_pipeline_job::<DEFAULT_CODEC_PIPELINE_STAGES>(
            &TEST_FRAW_2X2_RGB565,
            caps,
        );

        assert_eq!(job.prepare(), Err(CodecPipelineJobError::Unsupported));
        assert_eq!(job.state(), CodecPipelineJobState::Fallback);
        assert_eq!(job.failure(), Some(CodecErrorKind::Unsupported));
        assert_eq!(job.submit(), Err(CodecPipelineJobError::InvalidTransition));
    }

    #[test]
    fn mock_codec_backend_drives_supported_job_flow() {
        let caps = CodecAcceleratorCapabilities::new(false, false, true, false, false, 5);
        let mut job = plan_resource_image_pipeline_job::<DEFAULT_CODEC_PIPELINE_STAGES>(
            &TEST_FRAW_2X2_RGB565,
            caps,
        );
        let mut backend = MockCodecBackend::new(caps);

        assert_eq!(job.prepare_with(&mut backend), Ok(()));
        assert_eq!(job.submit_with(&mut backend), Ok(()));
        assert_eq!(job.complete_with(&mut backend), Ok(()));
        assert_eq!(job.state(), CodecPipelineJobState::Completed);
        assert_eq!(backend.prepare_calls(), 1);
        assert_eq!(backend.submit_calls(), 1);
        assert_eq!(backend.complete_calls(), 1);
        assert_eq!(backend.submitted_token(), None);
        assert_eq!(backend.completed_token(), Some(CodecPipelineJobToken(1)));
    }

    #[test]
    fn mock_codec_backend_reports_unsupported_and_failed_states() {
        let caps = CodecAcceleratorCapabilities::new(false, false, true, false, false, 5);
        let mut job = plan_resource_image_pipeline_job::<DEFAULT_CODEC_PIPELINE_STAGES>(
            &TEST_FRAW_2X2_RGB565,
            caps,
        );
        let mut unsupported_backend = MockCodecBackend::new(CodecAcceleratorCapabilities::NONE);

        assert_eq!(
            job.prepare_with(&mut unsupported_backend),
            Err(CodecPipelineJobError::Unsupported)
        );
        assert_eq!(job.state(), CodecPipelineJobState::Fallback);
        assert_eq!(job.failure(), Some(CodecErrorKind::Unsupported));

        let mut fail_job = plan_resource_image_pipeline_job::<DEFAULT_CODEC_PIPELINE_STAGES>(
            &TEST_FRAW_2X2_RGB565,
            caps,
        );
        let mut failing_backend =
            MockCodecBackend::new(caps).with_next_failure(MockCodecBackendFailure::Submit);

        assert_eq!(fail_job.prepare_with(&mut failing_backend), Ok(()));
        assert_eq!(
            fail_job.submit_with(&mut failing_backend),
            Err(CodecPipelineJobError::Unsupported)
        );
        assert_eq!(fail_job.state(), CodecPipelineJobState::Failed);
        assert_eq!(fail_job.failure(), Some(CodecErrorKind::Unsupported));
        assert_eq!(failing_backend.submitted_token(), None);
        assert_eq!(
            fail_job.complete_with(&mut failing_backend),
            Err(CodecPipelineJobError::InvalidTransition)
        );
    }

    #[test]
    fn mock_codec_backend_completion_failure_keeps_submitted_token_uncompleted() {
        let caps = CodecAcceleratorCapabilities::new(false, false, true, false, false, 5);
        let mut job = plan_resource_image_pipeline_job::<DEFAULT_CODEC_PIPELINE_STAGES>(
            &TEST_FRAW_2X2_RGB565,
            caps,
        );
        let mut backend =
            MockCodecBackend::new(caps).with_next_failure(MockCodecBackendFailure::Complete);

        assert_eq!(job.prepare_with(&mut backend), Ok(()));
        assert_eq!(job.submit_with(&mut backend), Ok(()));
        assert_eq!(backend.submitted_token(), Some(CodecPipelineJobToken(1)));
        assert_eq!(
            job.complete_with(&mut backend),
            Err(CodecPipelineJobError::Unsupported)
        );
        assert_eq!(job.state(), CodecPipelineJobState::Failed);
        assert_eq!(backend.submitted_token(), Some(CodecPipelineJobToken(1)));
        assert_eq!(backend.completed_token(), None);
    }
}
