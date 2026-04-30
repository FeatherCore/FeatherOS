use alloc::vec::Vec;

use crate::core::FixedList;
use crate::math::{Color, Point, Rect};
use crate::surface::{SurfaceFrame, SurfacePixelFormat};

mod font;
mod resource;
mod svg;
mod texture;

pub use font::{
    default_glyph_cache_capacity, font_cell_advance, font_line_height, font_text_bounds,
    FontCacheSummary, FontResourceSource, FontStore, FontWarmupSummary, GlyphCachePressure,
    GlyphCacheProfile, GLYPH_CACHE_CAPACITY, GLYPH_CACHE_BALANCED_CAPACITY,
    GLYPH_CACHE_LARGE_CAPACITY, GLYPH_CACHE_MAX_CAPACITY, GLYPH_CACHE_MIN_CAPACITY,
    GLYPH_CACHE_TINY_CAPACITY,
};
pub use resource::{
    visit_resource_blob, visit_resource_partition, ResourceEntry, ResourceManifest,
    ResourcePartition, ResourcePartitionSource, BUILTIN_RESOURCE_BLOB, BUILTIN_RESOURCE_PARTITION,
};
pub use svg::{
    default_svg_cache_capacity, SvgCacheProfile, SvgCacheSummary, SvgImageId, SvgRasterMask,
    SvgCachePressure, SvgResourceSource, SvgStore, SvgWarmupSummary, SVG_CACHE_BALANCED_CAPACITY,
    SVG_CACHE_LARGE_CAPACITY, SVG_CACHE_MAX_CAPACITY, SVG_CACHE_MIN_CAPACITY,
    SVG_CACHE_TINY_CAPACITY, SVG_RASTER_MAX_SIDE, SVG_STORE_CAPACITY,
};
pub use texture::{
    ImageId, PixelFormat, Texture, TextureResourceSource, TextureStore, TEXTURE_STORE_CAPACITY,
    IMAGE_WALLPAPER_AURORA, IMAGE_WALLPAPER_AURORA_LANDSCAPE,
    IMAGE_WALLPAPER_AURORA_PORTRAIT, IMAGE_WALLPAPER_DUSK,
    IMAGE_WALLPAPER_DUSK_LANDSCAPE, IMAGE_WALLPAPER_DUSK_PORTRAIT,
};

pub const DIRTY_RECT_CAPACITY: usize = 8;
pub const DRAW_CMD_CAPACITY: usize = 128;
const AA_SCALE: i32 = 4;
const AA_SAMPLE_OFFSETS: [i32; 2] = [1, 3];
const AA_SAMPLE_COUNT: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRegion {
    rects: [Rect; DIRTY_RECT_CAPACITY],
    len: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirtyRegionSummary {
    pub rects: u8,
    pub capacity: u8,
    pub covered_percent: u8,
    pub bounds_percent: u8,
    pub full: bool,
    pub tile_aligned: bool,
}

impl DirtyRegionSummary {
    pub const fn is_empty(self) -> bool {
        self.rects == 0
    }

    pub const fn is_fragmented(self) -> bool {
        self.rects >= 6 && !self.full
    }

    pub const fn is_broad(self) -> bool {
        self.full || self.covered_percent >= 50 || self.bounds_percent >= 75
    }
}

impl Default for DirtyRegion {
    fn default() -> Self {
        Self::empty()
    }
}

impl DirtyRegion {
    pub const fn empty() -> Self {
        Self {
            rects: [Rect::new(0, 0, 0, 0); DIRTY_RECT_CAPACITY],
            len: 0,
        }
    }

    pub fn full(width: u16, height: u16) -> Self {
        let mut region = Self::empty();
        region.include_rect(Rect::new(0, 0, width, height));
        region
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn rects(&self) -> &[Rect] {
        &self.rects[..self.len as usize]
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub const fn capacity(&self) -> usize {
        DIRTY_RECT_CAPACITY
    }

    pub fn bounds(&self) -> Option<Rect> {
        let mut rects = self.rects();
        let first = rects.first().copied()?;
        rects = &rects[1..];

        let mut bounds = first;
        for rect in rects {
            bounds = bounds.union(*rect);
        }
        Some(bounds)
    }

    pub fn rect(&self) -> Option<Rect> {
        self.bounds()
    }

    pub fn covered_area(&self) -> u64 {
        let mut area = 0u64;
        for rect in self.rects() {
            area = area.saturating_add(rect_area(*rect));
        }
        area
    }

    pub fn bounds_area(&self) -> u64 {
        self.bounds().map(rect_area).unwrap_or(0)
    }

    pub fn is_full_for(&self, width: u16, height: u16) -> bool {
        self.len == 1 && self.rects[0] == Rect::new(0, 0, width, height)
    }

    pub fn summary(&self, width: u16, height: u16) -> DirtyRegionSummary {
        self.summary_with_tile(width, height, 1)
    }

    pub fn summary_with_tile(&self, width: u16, height: u16, tile_size: u16) -> DirtyRegionSummary {
        let screen_area = width as u64 * height as u64;
        DirtyRegionSummary {
            rects: self.len,
            capacity: DIRTY_RECT_CAPACITY as u8,
            covered_percent: ratio_percent(self.covered_area(), screen_area),
            bounds_percent: ratio_percent(self.bounds_area(), screen_area),
            full: self.is_full_for(width, height),
            tile_aligned: self.is_tile_aligned(width, height, tile_size),
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn set_full(&mut self, width: u16, height: u16) {
        self.clear();
        self.include_rect(Rect::new(0, 0, width, height));
    }

    pub fn include_rect(&mut self, mut rect: Rect) {
        if rect.is_empty() {
            return;
        }

        let mut index = 0usize;
        while index < self.len as usize {
            let current = self.rects[index];
            if rects_touch_or_overlap(current, rect) {
                rect = current.union(rect);
                self.remove_at(index);
            } else {
                index += 1;
            }
        }

        self.push_or_merge(rect);
    }

    pub fn compact_to_budget(&mut self, target_len: usize, max_extra_area_percent: u8) -> bool {
        let target_len = target_len.clamp(1, DIRTY_RECT_CAPACITY);
        let mut changed = false;
        while self.len() > target_len {
            let Some((left, right, extra_area)) = self.best_pair_merge() else {
                break;
            };
            let covered_area = self.covered_area().max(1);
            if extra_area.saturating_mul(100)
                > covered_area.saturating_mul(max_extra_area_percent as u64)
            {
                break;
            }

            self.merge_pair(left, right);
            changed = true;
        }
        changed
    }

    pub fn align_to_tiles(
        &mut self,
        width: u16,
        height: u16,
        tile_size: u16,
        max_extra_area_percent: u8,
    ) -> bool {
        if tile_size <= 1 || self.is_empty() || self.is_full_for(width, height) {
            return false;
        }

        let original_area = self.covered_area().max(1);
        let mut aligned = Self::empty();
        for rect in self.rects() {
            if let Some(rect) = align_rect_to_tile(*rect, width, height, tile_size) {
                aligned.include_rect(rect);
            }
        }
        if aligned.is_empty() {
            return false;
        }

        let aligned_area = aligned.covered_area();
        if aligned_area.saturating_mul(100)
            > original_area.saturating_mul(100 + max_extra_area_percent as u64)
        {
            return false;
        }

        if aligned == *self {
            false
        } else {
            *self = aligned;
            true
        }
    }

    pub fn is_tile_aligned(&self, width: u16, height: u16, tile_size: u16) -> bool {
        if tile_size <= 1 || self.is_empty() {
            return false;
        }
        for rect in self.rects() {
            let Some(aligned) = align_rect_to_tile(*rect, width, height, tile_size) else {
                return false;
            };
            if aligned != *rect {
                return false;
            }
        }
        true
    }

    fn push_or_merge(&mut self, rect: Rect) {
        if (self.len as usize) < DIRTY_RECT_CAPACITY {
            self.rects[self.len as usize] = rect;
            self.len += 1;
            return;
        }

        let index = self.best_merge_index(rect);
        self.rects[index] = self.rects[index].union(rect);
        self.compact_overlaps();
    }

    fn remove_at(&mut self, index: usize) {
        let len = self.len as usize;
        for cursor in index..len.saturating_sub(1) {
            self.rects[cursor] = self.rects[cursor + 1];
        }
        self.len -= 1;
        self.rects[self.len as usize] = Rect::new(0, 0, 0, 0);
    }

    fn best_merge_index(&self, rect: Rect) -> usize {
        let mut best = 0usize;
        let mut best_cost = u64::MAX;

        for index in 0..self.len as usize {
            let current = self.rects[index];
            let cost = rect_area(current.union(rect)).saturating_sub(rect_area(current));
            if cost < best_cost {
                best = index;
                best_cost = cost;
            }
        }

        best
    }

    fn best_pair_merge(&self) -> Option<(usize, usize, u64)> {
        if self.len < 2 {
            return None;
        }

        let mut best = None;
        let mut best_cost = u64::MAX;
        for left in 0..self.len as usize {
            for right in (left + 1)..self.len as usize {
                let a = self.rects[left];
                let b = self.rects[right];
                let merged = a.union(b);
                let cost = rect_area(merged).saturating_sub(rect_area(a).saturating_add(rect_area(b)));
                if cost < best_cost {
                    best = Some((left, right, cost));
                    best_cost = cost;
                }
            }
        }
        best
    }

    fn merge_pair(&mut self, left: usize, right: usize) {
        if left >= self.len as usize || right >= self.len as usize || left == right {
            return;
        }

        let (left, right) = if left < right {
            (left, right)
        } else {
            (right, left)
        };
        self.rects[left] = self.rects[left].union(self.rects[right]);
        self.remove_at(right);
        self.compact_overlaps();
    }

    fn compact_overlaps(&mut self) {
        let mut index = 0usize;
        while index < self.len as usize {
            let mut cursor = index + 1;
            while cursor < self.len as usize {
                if rects_touch_or_overlap(self.rects[index], self.rects[cursor]) {
                    self.rects[index] = self.rects[index].union(self.rects[cursor]);
                    self.remove_at(cursor);
                } else {
                    cursor += 1;
                }
            }
            index += 1;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectKind {
    PreviewCube,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EffectParams {
    pub phase: u8,
    pub item_count: u8,
    pub selected: u8,
    pub alpha: u8,
}

impl EffectParams {
    pub const fn new(phase: u8, item_count: u8, selected: u8) -> Self {
        Self {
            phase,
            item_count,
            selected,
            alpha: 255,
        }
    }

    pub const fn with_alpha(self, alpha: u8) -> Self {
        Self { alpha, ..self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VectorIcon {
    Wifi,
    Bluetooth,
    Phone,
    Chat,
    Settings,
    Camera,
    Flashlight,
    Airplane,
    Moon,
    Sync,
    Mail,
    Cloud,
    Folder,
    Music,
    Play,
    Wing,
    Check,
    Close,
    Alert,
    More,
    Brightness,
    Haptic,
    Motion,
    Palette,
    Preview,
    Back,
    System,
    Terminal,
    Surface,
}

impl SvgStore {
    pub fn register_vector_icon(&mut self, icon: VectorIcon, bytes: &[u8]) -> bool {
        self.register(vector_icon_svg_id(icon), bytes)
    }

    pub fn warmup_vector_icon(&self, icon: VectorIcon, size: u16) -> SvgWarmupSummary {
        self.warmup(&[vector_icon_svg_id(icon)], size)
    }
}

pub const fn vector_icon_svg_id(icon: VectorIcon) -> SvgImageId {
    SvgImageId(match icon {
        VectorIcon::Wifi => 1,
        VectorIcon::Bluetooth => 2,
        VectorIcon::Phone => 3,
        VectorIcon::Chat => 4,
        VectorIcon::Settings => 5,
        VectorIcon::Camera => 6,
        VectorIcon::Flashlight => 7,
        VectorIcon::Airplane => 8,
        VectorIcon::Moon => 9,
        VectorIcon::Sync => 10,
        VectorIcon::Mail => 11,
        VectorIcon::Cloud => 12,
        VectorIcon::Folder => 13,
        VectorIcon::Music => 14,
        VectorIcon::Play => 15,
        VectorIcon::Wing => 16,
        VectorIcon::Check => 17,
        VectorIcon::Close => 18,
        VectorIcon::Alert => 19,
        VectorIcon::More => 20,
        VectorIcon::Brightness => 21,
        VectorIcon::Haptic => 22,
        VectorIcon::Motion => 23,
        VectorIcon::Palette => 24,
        VectorIcon::Preview => 25,
        VectorIcon::Back => 26,
        VectorIcon::System => 27,
        VectorIcon::Terminal => 28,
        VectorIcon::Surface => 29,
    })
}

include!(concat!(env!("OUT_DIR"), "/wing_bootstrap_icons.rs"));

const BUILTIN_VECTOR_ICON_IDS: &[SvgImageId] = &[
    vector_icon_svg_id(VectorIcon::Wifi),
    vector_icon_svg_id(VectorIcon::Bluetooth),
    vector_icon_svg_id(VectorIcon::Phone),
    vector_icon_svg_id(VectorIcon::Chat),
    vector_icon_svg_id(VectorIcon::Settings),
    vector_icon_svg_id(VectorIcon::Camera),
    vector_icon_svg_id(VectorIcon::Flashlight),
    vector_icon_svg_id(VectorIcon::Airplane),
    vector_icon_svg_id(VectorIcon::Moon),
    vector_icon_svg_id(VectorIcon::Sync),
    vector_icon_svg_id(VectorIcon::Mail),
    vector_icon_svg_id(VectorIcon::Cloud),
    vector_icon_svg_id(VectorIcon::Folder),
    vector_icon_svg_id(VectorIcon::Music),
    vector_icon_svg_id(VectorIcon::Play),
    vector_icon_svg_id(VectorIcon::Wing),
    vector_icon_svg_id(VectorIcon::Check),
    vector_icon_svg_id(VectorIcon::Close),
    vector_icon_svg_id(VectorIcon::Alert),
    vector_icon_svg_id(VectorIcon::More),
    vector_icon_svg_id(VectorIcon::Brightness),
    vector_icon_svg_id(VectorIcon::Haptic),
    vector_icon_svg_id(VectorIcon::Motion),
    vector_icon_svg_id(VectorIcon::Palette),
    vector_icon_svg_id(VectorIcon::Preview),
    vector_icon_svg_id(VectorIcon::Back),
    vector_icon_svg_id(VectorIcon::System),
    vector_icon_svg_id(VectorIcon::Terminal),
    vector_icon_svg_id(VectorIcon::Surface),
];

pub(crate) fn warm_builtin_shell_svg_cache(store: &SvgStore, height: u16) -> SvgWarmupSummary {
    let size = if height <= 320 {
        20
    } else if height <= 480 {
        28
    } else {
        36
    };
    store.warmup(BUILTIN_VECTOR_ICON_IDS, size)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawTaskKind {
    Clear,
    Clip,
    Fill,
    Shadow,
    Image,
    Vector,
    Surface,
    Text,
    Effect,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawTaskSummary {
    pub total: u16,
    pub clear: u16,
    pub clip: u16,
    pub fill: u16,
    pub shadow: u16,
    pub image: u16,
    pub vector: u16,
    pub surface: u16,
    pub text: u16,
    pub effect: u16,
}

impl DrawTaskSummary {
    pub fn include(&mut self, kind: DrawTaskKind) {
        self.total = self.total.saturating_add(1);
        match kind {
            DrawTaskKind::Clear => self.clear = self.clear.saturating_add(1),
            DrawTaskKind::Clip => self.clip = self.clip.saturating_add(1),
            DrawTaskKind::Fill => self.fill = self.fill.saturating_add(1),
            DrawTaskKind::Shadow => self.shadow = self.shadow.saturating_add(1),
            DrawTaskKind::Image => self.image = self.image.saturating_add(1),
            DrawTaskKind::Vector => self.vector = self.vector.saturating_add(1),
            DrawTaskKind::Surface => self.surface = self.surface.saturating_add(1),
            DrawTaskKind::Text => self.text = self.text.saturating_add(1),
            DrawTaskKind::Effect => self.effect = self.effect.saturating_add(1),
        }
    }

    pub const fn is_empty(self) -> bool {
        self.total == 0
    }

    pub const fn count(self, kind: DrawTaskKind) -> u16 {
        match kind {
            DrawTaskKind::Clear => self.clear,
            DrawTaskKind::Clip => self.clip,
            DrawTaskKind::Fill => self.fill,
            DrawTaskKind::Shadow => self.shadow,
            DrawTaskKind::Image => self.image,
            DrawTaskKind::Vector => self.vector,
            DrawTaskKind::Surface => self.surface,
            DrawTaskKind::Text => self.text,
            DrawTaskKind::Effect => self.effect,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawTaskRoute {
    Native,
    SoftwareFallback,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DrawRouteMask {
    bits: u8,
}

impl DrawRouteMask {
    pub const NATIVE: Self = Self { bits: 1 << 0 };
    pub const SOFTWARE_FALLBACK: Self = Self { bits: 1 << 1 };
    pub const SUPPORTED: Self = Self {
        bits: Self::NATIVE.bits | Self::SOFTWARE_FALLBACK.bits,
    };

    pub const fn accepts(self, route: DrawTaskRoute) -> bool {
        match route {
            DrawTaskRoute::Native => (self.bits & Self::NATIVE.bits) != 0,
            DrawTaskRoute::SoftwareFallback => {
                (self.bits & Self::SOFTWARE_FALLBACK.bits) != 0
            }
            DrawTaskRoute::Unsupported => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawRouteRun {
    pub route: DrawTaskRoute,
    pub clip: Option<Rect>,
    pub start: usize,
    pub end: usize,
}

impl DrawRouteRun {
    pub const fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderRunSummary {
    pub total: u16,
    pub native: u16,
    pub fallback: u16,
    pub unsupported: u16,
    pub clipped: u16,
    pub max_len: u16,
}

impl RenderRunSummary {
    fn include(&mut self, run: DrawRouteRun) {
        self.total = self.total.saturating_add(1);
        if run.clip.is_some() {
            self.clipped = self.clipped.saturating_add(1);
        }
        self.max_len = self.max_len.max(saturating_usize_to_u16(run.len()));
        match run.route {
            DrawTaskRoute::Native => self.native = self.native.saturating_add(1),
            DrawTaskRoute::SoftwareFallback => {
                self.fallback = self.fallback.saturating_add(1)
            }
            DrawTaskRoute::Unsupported => {
                self.unsupported = self.unsupported.saturating_add(1)
            }
        }
    }

    pub const fn is_fragmented(self) -> bool {
        self.total >= 32 && self.max_len <= 4
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderBackgroundKind {
    None,
    Gradient,
    GradientRgb565Image,
    GradientMissingImage,
}

impl Default for RenderBackgroundKind {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderBackgroundRoute {
    None,
    SoftwareFastPath,
    NativePlane,
    Unsupported,
}

impl Default for RenderBackgroundRoute {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderLayerSummary {
    pub background: RenderBackgroundKind,
    pub background_route: RenderBackgroundRoute,
    pub background_skip: u8,
}

impl RenderLayerSummary {
    pub const fn has_background(self) -> bool {
        !matches!(self.background, RenderBackgroundKind::None)
    }

    pub const fn has_background_fast_path(self) -> bool {
        matches!(
            self.background_route,
            RenderBackgroundRoute::SoftwareFastPath | RenderBackgroundRoute::NativePlane
        )
    }

    pub const fn has_background_image(self) -> bool {
        matches!(self.background, RenderBackgroundKind::GradientRgb565Image)
    }

    pub const fn background_supported(self) -> bool {
        matches!(
            self.background_route,
            RenderBackgroundRoute::None
                | RenderBackgroundRoute::SoftwareFastPath
                | RenderBackgroundRoute::NativePlane
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundPlaneSubmission {
    pub kind: RenderBackgroundKind,
    pub top: Color,
    pub bottom: Color,
    pub image: Option<ImageId>,
    pub tint: Color,
}

pub trait BackgroundPlaneOps {
    fn capabilities(&self) -> RendererCapabilities;
    fn submit_background(
        &mut self,
        submission: BackgroundPlaneSubmission,
        textures: &TextureStore,
        dirty: DirtyRegion,
    ) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimBackgroundPlaneOps {
    capabilities: RendererCapabilities,
    submissions: u32,
    rejected: u32,
    last_submission: Option<BackgroundPlaneSubmission>,
}

impl SimBackgroundPlaneOps {
    pub const fn new(capabilities: RendererCapabilities) -> Self {
        Self {
            capabilities,
            submissions: 0,
            rejected: 0,
            last_submission: None,
        }
    }

    pub const fn submissions(self) -> u32 {
        self.submissions
    }

    pub const fn rejected(self) -> u32 {
        self.rejected
    }

    pub const fn last_submission(self) -> Option<BackgroundPlaneSubmission> {
        self.last_submission
    }
}

impl Default for SimBackgroundPlaneOps {
    fn default() -> Self {
        Self::new(RendererCapabilities::SOFTWARE_WITH_BACKGROUND_PLANE)
    }
}

impl BackgroundPlaneOps for SimBackgroundPlaneOps {
    fn capabilities(&self) -> RendererCapabilities {
        self.capabilities
    }

    fn submit_background(
        &mut self,
        submission: BackgroundPlaneSubmission,
        textures: &TextureStore,
        dirty: DirtyRegion,
    ) -> bool {
        let supported = !dirty.is_empty()
            && match submission.image {
                Some(image) => textures
                    .get(image)
                    .map(|texture| texture.format == PixelFormat::Rgb565)
                    .unwrap_or(false),
                None => true,
            };

        if supported {
            self.submissions = self.submissions.saturating_add(1);
            self.last_submission = Some(submission);
        } else {
            self.rejected = self.rejected.saturating_add(1);
        }

        supported
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderPlan {
    pub tasks: DrawTaskSummary,
    pub runs: RenderRunSummary,
    pub layers: RenderLayerSummary,
    pub native: DrawTaskSummary,
    pub fallback: DrawTaskSummary,
    pub supported: DrawTaskSummary,
    pub unsupported: DrawTaskSummary,
}

impl RenderPlan {
    pub const fn fully_supported(self) -> bool {
        self.unsupported.is_empty()
    }

    pub const fn fully_native(self) -> bool {
        self.fallback.is_empty() && self.unsupported.is_empty()
    }

    pub const fn needs_software_fallback(self) -> bool {
        !self.fallback.is_empty()
    }

    pub const fn has_routes(self, mask: DrawRouteMask) -> bool {
        (mask.accepts(DrawTaskRoute::Native) && !self.native.is_empty())
            || (mask.accepts(DrawTaskRoute::SoftwareFallback) && !self.fallback.is_empty())
    }

    pub const fn has_fragmented_runs(self) -> bool {
        self.runs.is_fragmented()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawCmd {
    ClearGradient { top: Color, bottom: Color },
    SetClip { rect: Option<Rect> },
    FillRect { rect: Rect, color: Color },
    FillRoundRect { rect: Rect, radius: u8, color: Color },
    FillRoundRectGradient {
        rect: Rect,
        radius: u8,
        top: Color,
        bottom: Color,
    },
    ShadowRoundRect {
        rect: Rect,
        radius: u8,
        color: Color,
        offset_x: i16,
        offset_y: i16,
        blur: u8,
        spread: u8,
    },
    FillCircle { rect: Rect, color: Color },
    Image { rect: Rect, image: ImageId, tint: Color },
    VectorIcon { rect: Rect, icon: VectorIcon, color: Color },
    Surface { rect: Rect, frame: SurfaceFrame, alpha: u8 },
    Text { x: i32, y: i32, text: &'static str, color: Color, scale: u8 },
    Effect { rect: Rect, effect: EffectKind, params: EffectParams },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BackgroundLayer {
    top: Color,
    bottom: Color,
    image: Option<BackgroundImageLayer>,
    skip_until: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BackgroundImageLayer {
    image: ImageId,
    tint: Color,
}

impl BackgroundLayer {
    fn summary(self, capabilities: RendererCapabilities) -> RenderLayerSummary {
        let background = if self.image.is_some() {
            RenderBackgroundKind::GradientRgb565Image
        } else if self.skip_until > 1 {
            RenderBackgroundKind::GradientMissingImage
        } else {
            RenderBackgroundKind::Gradient
        };

        RenderLayerSummary {
            background,
            background_route: capabilities.route_background_layer(background),
            background_skip: self.skip_until.min(u8::MAX as usize) as u8,
        }
    }

    fn submission(self) -> BackgroundPlaneSubmission {
        BackgroundPlaneSubmission {
            kind: if self.image.is_some() {
                RenderBackgroundKind::GradientRgb565Image
            } else if self.skip_until > 1 {
                RenderBackgroundKind::GradientMissingImage
            } else {
                RenderBackgroundKind::Gradient
            },
            top: self.top,
            bottom: self.bottom,
            image: self.image.map(|image| image.image),
            tint: self.image.map(|image| image.tint).unwrap_or(Color::WHITE),
        }
    }
}

impl DrawCmd {
    pub const fn kind(self) -> DrawTaskKind {
        match self {
            Self::ClearGradient { .. } => DrawTaskKind::Clear,
            Self::SetClip { .. } => DrawTaskKind::Clip,
            Self::FillRect { .. }
            | Self::FillRoundRect { .. }
            | Self::FillRoundRectGradient { .. }
            | Self::FillCircle { .. } => DrawTaskKind::Fill,
            Self::ShadowRoundRect { .. } => DrawTaskKind::Shadow,
            Self::Image { .. } => DrawTaskKind::Image,
            Self::VectorIcon { .. } => DrawTaskKind::Vector,
            Self::Surface { .. } => DrawTaskKind::Surface,
            Self::Text { .. } => DrawTaskKind::Text,
            Self::Effect { .. } => DrawTaskKind::Effect,
        }
    }

    pub fn bounds(self, viewport: Rect) -> Rect {
        match self {
            Self::ClearGradient { .. } | Self::SetClip { .. } => viewport,
            Self::FillRect { rect, .. }
            | Self::FillRoundRect { rect, .. }
            | Self::FillRoundRectGradient { rect, .. }
            | Self::FillCircle { rect, .. }
            | Self::Image { rect, .. }
            | Self::VectorIcon { rect, .. }
            | Self::Surface { rect, .. }
            | Self::Effect { rect, .. } => rect,
            Self::ShadowRoundRect {
                rect,
                offset_x,
                offset_y,
                blur,
                spread,
                ..
            } => shadow_bounds(rect, offset_x, offset_y, blur, spread),
            Self::Text { x, y, text, scale, .. } => {
                font_text_bounds(x, y, text, scale.max(1))
            }
        }
    }
}

pub struct DrawRouteRuns<'a> {
    commands: &'a [DrawCmd],
    capabilities: RendererCapabilities,
    index: usize,
    active_clip: Option<Rect>,
}

impl<'a> DrawRouteRuns<'a> {
    fn new(commands: &'a [DrawCmd], capabilities: RendererCapabilities) -> Self {
        Self {
            commands,
            capabilities,
            index: 0,
            active_clip: None,
        }
    }
}

impl Iterator for DrawRouteRuns<'_> {
    type Item = DrawRouteRun;

    fn next(&mut self) -> Option<Self::Item> {
        let (start, route, clip) = loop {
            let command = *self.commands.get(self.index)?;
            match command {
                DrawCmd::SetClip { rect } => {
                    self.active_clip = rect;
                    self.index += 1;
                }
                command => {
                    let start = self.index;
                    self.index += 1;
                    break (
                        start,
                        self.capabilities.route_task_kind(command.kind()),
                        self.active_clip,
                    );
                }
            }
        };

        while let Some(command) = self.commands.get(self.index).copied() {
            match command {
                DrawCmd::SetClip { rect } => {
                    if rect != self.active_clip {
                        break;
                    }
                    self.index += 1;
                }
                command => {
                    let next_route = self.capabilities.route_task_kind(command.kind());
                    if next_route != route {
                        break;
                    }
                    self.index += 1;
                }
            }
        }

        Some(DrawRouteRun {
            route,
            clip,
            start,
            end: self.index,
        })
    }
}

#[derive(Default)]
pub struct DrawList {
    commands: FixedList<DrawCmd, DRAW_CMD_CAPACITY>,
    active_clip: Option<Rect>,
}

impl DrawList {
    pub fn clear(&mut self) {
        self.commands.clear();
        self.active_clip = None;
    }

    pub fn push(&mut self, cmd: DrawCmd) {
        let inserted = self.commands.push(cmd);
        if inserted {
            if let DrawCmd::SetClip { rect } = cmd {
                self.active_clip = rect;
            }
        }
    }

    pub fn push_clipped(&mut self, clip: Option<Rect>, cmd: DrawCmd) {
        if clip != self.active_clip {
            self.push(DrawCmd::SetClip { rect: clip });
        }
        self.push(cmd);
    }

    pub fn commands(&self) -> &[DrawCmd] {
        self.commands.as_slice()
    }

    pub fn route_runs(&self, capabilities: RendererCapabilities) -> DrawRouteRuns<'_> {
        DrawRouteRuns::new(self.commands(), capabilities)
    }

    pub fn task_summary(&self) -> DrawTaskSummary {
        let mut summary = DrawTaskSummary::default();
        for command in self.commands() {
            summary.include(command.kind());
        }
        summary
    }

    pub fn render_plan(&self, capabilities: RendererCapabilities) -> RenderPlan {
        let mut plan = RenderPlan::default();
        for command in self.commands() {
            let kind = command.kind();
            plan.tasks.include(kind);
            match capabilities.route_task_kind(kind) {
                DrawTaskRoute::Native => {
                    plan.native.include(kind);
                    plan.supported.include(kind);
                }
                DrawTaskRoute::SoftwareFallback => {
                    plan.fallback.include(kind);
                    plan.supported.include(kind);
                }
                DrawTaskRoute::Unsupported => plan.unsupported.include(kind),
            }
        }
        for run in self.route_runs(capabilities) {
            plan.runs.include(run);
        }
        plan
    }

    pub fn overflowed(&self) -> bool {
        self.commands.overflowed()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererKind {
    Software,
    Gpu2d,
    Gles,
    Custom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RendererCapabilities {
    pub cpu_framebuffer: bool,
    pub dirty_rects: bool,
    pub scissor_clip: bool,
    pub alpha_blend: bool,
    pub rounded_rect: bool,
    pub circle: bool,
    pub antialias: bool,
    pub shadow: bool,
    pub image_rgb565: bool,
    pub image_a8: bool,
    pub image_bilinear: bool,
    pub vector_icon: bool,
    pub background_layer: bool,
    pub background_plane: bool,
    pub mesh2d: bool,
    pub mesh3d: bool,
    pub effects: bool,
}

impl RendererCapabilities {
    pub const SOFTWARE: Self = Self {
        cpu_framebuffer: true,
        dirty_rects: true,
        scissor_clip: true,
        alpha_blend: true,
        rounded_rect: true,
        circle: true,
        antialias: true,
        shadow: true,
        image_rgb565: true,
        image_a8: true,
        image_bilinear: true,
        vector_icon: true,
        background_layer: true,
        background_plane: false,
        mesh2d: false,
        mesh3d: false,
        effects: true,
    };

    pub const SOFTWARE_WITH_BACKGROUND_PLANE: Self = Self {
        cpu_framebuffer: true,
        dirty_rects: true,
        scissor_clip: true,
        alpha_blend: true,
        rounded_rect: true,
        circle: true,
        antialias: true,
        shadow: true,
        image_rgb565: true,
        image_a8: true,
        image_bilinear: true,
        vector_icon: true,
        background_layer: true,
        background_plane: true,
        mesh2d: false,
        mesh3d: false,
        effects: true,
    };

    pub const fn supports_basic_2d(self) -> bool {
        self.dirty_rects
            && self.scissor_clip
            && self.alpha_blend
            && self.rounded_rect
            && self.circle
            && self.antialias
            && self.shadow
            && self.image_rgb565
            && self.image_a8
            && self.image_bilinear
    }

    pub const fn supports_effects(self) -> bool {
        self.effects || self.mesh2d || self.mesh3d
    }

    pub const fn supports_cube_preview(self) -> bool {
        self.effects || self.mesh3d
    }

    pub const fn route_background_layer(
        self,
        kind: RenderBackgroundKind,
    ) -> RenderBackgroundRoute {
        match kind {
            RenderBackgroundKind::None => RenderBackgroundRoute::None,
            RenderBackgroundKind::Gradient | RenderBackgroundKind::GradientMissingImage => {
                if self.background_plane {
                    RenderBackgroundRoute::NativePlane
                } else if self.background_layer && self.cpu_framebuffer {
                    RenderBackgroundRoute::SoftwareFastPath
                } else {
                    RenderBackgroundRoute::Unsupported
                }
            }
            RenderBackgroundKind::GradientRgb565Image => {
                if self.background_plane && self.image_rgb565 {
                    RenderBackgroundRoute::NativePlane
                } else if self.background_layer && self.cpu_framebuffer && self.image_rgb565 {
                    RenderBackgroundRoute::SoftwareFastPath
                } else {
                    RenderBackgroundRoute::Unsupported
                }
            }
        }
    }

    pub const fn supports_native_task_kind(self, kind: DrawTaskKind) -> bool {
        match kind {
            DrawTaskKind::Clear => self.cpu_framebuffer,
            DrawTaskKind::Clip => self.scissor_clip,
            DrawTaskKind::Fill => self.alpha_blend,
            DrawTaskKind::Shadow => self.shadow,
            DrawTaskKind::Image => self.image_rgb565 || self.image_a8,
            DrawTaskKind::Vector => self.vector_icon && self.alpha_blend,
            DrawTaskKind::Surface => self.cpu_framebuffer && self.alpha_blend,
            DrawTaskKind::Text => self.cpu_framebuffer && self.alpha_blend,
            DrawTaskKind::Effect => self.supports_effects(),
        }
    }

    pub const fn supports_software_fallback_task_kind(self, kind: DrawTaskKind) -> bool {
        self.cpu_framebuffer && RendererCapabilities::SOFTWARE.supports_native_task_kind(kind)
    }

    pub const fn route_task_kind(self, kind: DrawTaskKind) -> DrawTaskRoute {
        if self.supports_native_task_kind(kind) {
            DrawTaskRoute::Native
        } else if self.supports_software_fallback_task_kind(kind) {
            DrawTaskRoute::SoftwareFallback
        } else {
            DrawTaskRoute::Unsupported
        }
    }

    pub const fn supports_task_kind(self, kind: DrawTaskKind) -> bool {
        match self.route_task_kind(kind) {
            DrawTaskRoute::Native | DrawTaskRoute::SoftwareFallback => true,
            DrawTaskRoute::Unsupported => false,
        }
    }
}

pub trait RendererBackend {
    fn kind(&self) -> RendererKind;
    fn capabilities(&self) -> RendererCapabilities;
    fn render_plan(&self, list: &DrawList, textures: &TextureStore) -> RenderPlan {
        let _ = textures;
        list.render_plan(self.capabilities())
    }
    fn draw(
        &mut self,
        list: &DrawList,
        plan: RenderPlan,
        textures: &TextureStore,
        fonts: &FontStore,
        svgs: &SvgStore,
        dirty: DirtyRegion,
    );
    fn framebuffer(&self) -> &[u32];
}

pub struct BackgroundPlaneRenderer<O: BackgroundPlaneOps = SimBackgroundPlaneOps> {
    kind: RendererKind,
    software: SoftwareRenderer,
    ops: O,
}

impl BackgroundPlaneRenderer<SimBackgroundPlaneOps> {
    pub fn new_gpu2d(width: u16, height: u16) -> Self {
        Self::new_with_ops(RendererKind::Gpu2d, width, height, SimBackgroundPlaneOps::default())
    }

    pub fn new_custom(width: u16, height: u16, capabilities: RendererCapabilities) -> Self {
        Self::new_with_ops(
            RendererKind::Custom,
            width,
            height,
            SimBackgroundPlaneOps::new(capabilities),
        )
    }
}

impl<O: BackgroundPlaneOps> BackgroundPlaneRenderer<O> {
    pub fn new_with_ops(kind: RendererKind, width: u16, height: u16, ops: O) -> Self {
        Self {
            kind,
            software: SoftwareRenderer::new(width, height),
            ops,
        }
    }

    pub fn software(&self) -> &SoftwareRenderer {
        &self.software
    }

    pub fn ops(&self) -> &O {
        &self.ops
    }

    fn submit_background_plane(
        &mut self,
        list: &DrawList,
        textures: &TextureStore,
        dirty: DirtyRegion,
    ) -> bool {
        let viewport = Rect::new(0, 0, self.software.width, self.software.height);
        let capabilities = self.ops.capabilities();
        let Some(layer) = background_layer(
            list.commands(),
            viewport,
            capabilities,
            DrawRouteMask::SUPPORTED,
            textures,
        ) else {
            return false;
        };
        if !matches!(
            layer.summary(capabilities).background_route,
            RenderBackgroundRoute::NativePlane
        ) {
            return false;
        }

        self.ops
            .submit_background(layer.submission(), textures, dirty)
    }
}

pub struct SoftwareRenderer {
    width: u16,
    height: u16,
    framebuffer: Vec<u32>,
    dirty_clip: Option<Rect>,
    draw_clip: Option<Rect>,
}

impl SoftwareRenderer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            framebuffer: alloc::vec![0; width as usize * height as usize],
            dirty_clip: None,
            draw_clip: None,
        }
    }

    pub fn framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    pub fn draw(
        &mut self,
        list: &DrawList,
        plan: RenderPlan,
        textures: &TextureStore,
        fonts: &FontStore,
        svgs: &SvgStore,
        dirty: DirtyRegion,
    ) {
        self.draw_routes(
            list,
            plan,
            RendererCapabilities::SOFTWARE,
            DrawRouteMask::SUPPORTED,
            textures,
            fonts,
            svgs,
            dirty,
        );
    }

    pub fn draw_routes(
        &mut self,
        list: &DrawList,
        plan: RenderPlan,
        route_capabilities: RendererCapabilities,
        routes: DrawRouteMask,
        textures: &TextureStore,
        fonts: &FontStore,
        svgs: &SvgStore,
        dirty: DirtyRegion,
    ) {
        if dirty.is_empty() {
            return;
        }
        if !plan.has_routes(routes) {
            return;
        }

        let viewport = Rect::new(0, 0, self.width, self.height);
        let background = background_layer(
            list.commands(),
            viewport,
            route_capabilities,
            routes,
            textures,
        );
        let skip_until = background.map(|layer| layer.skip_until).unwrap_or(0);
        for rect in dirty.rects() {
            self.dirty_clip = self.clip_rect(*rect);
            self.draw_clip = None;
            let Some(clip) = self.dirty_clip else {
                continue;
            };

            if let Some(background) = background {
                self.draw_background_layer(background, textures);
            }

            for run in list.route_runs(route_capabilities) {
                if !routes.accepts(run.route) {
                    continue;
                }

                self.set_draw_clip(run.clip);
                for (offset, command) in list.commands()[run.start..run.end]
                    .iter()
                    .copied()
                    .enumerate()
                {
                    if run.start + offset < skip_until {
                        continue;
                    }
                    if !rects_overlap(command.bounds(viewport), clip) {
                        continue;
                    }

                    match command {
                        DrawCmd::SetClip { rect } => self.set_draw_clip(rect),
                        command => self.draw_command(command, textures, fonts, svgs),
                    }
                }
            }
        }

        self.dirty_clip = None;
        self.draw_clip = None;
    }

    fn draw_command(
        &mut self,
        command: DrawCmd,
        textures: &TextureStore,
        fonts: &FontStore,
        svgs: &SvgStore,
    ) {
        match command {
            DrawCmd::ClearGradient { top, bottom } => self.clear_gradient(top, bottom),
            DrawCmd::SetClip { rect } => self.set_draw_clip(rect),
            DrawCmd::FillRect { rect, color } => self.fill_rect(rect, color),
            DrawCmd::FillRoundRect { rect, radius, color } => {
                self.fill_round_rect(rect, radius, color)
            }
            DrawCmd::FillRoundRectGradient {
                rect,
                radius,
                top,
                bottom,
            } => self.fill_round_rect_gradient(rect, radius, top, bottom),
            DrawCmd::ShadowRoundRect {
                rect,
                radius,
                color,
                offset_x,
                offset_y,
                blur,
                spread,
            } => self.shadow_round_rect(rect, radius, color, offset_x, offset_y, blur, spread),
            DrawCmd::FillCircle { rect, color } => self.fill_circle(rect, color),
            DrawCmd::Image { rect, image, tint } => {
                if let Some(texture) = textures.get(image) {
                    self.draw_image(rect, texture, tint);
                }
            }
            DrawCmd::VectorIcon { rect, icon, color } => {
                self.draw_vector_icon(rect, icon, color, svgs)
            }
            DrawCmd::Surface { rect, frame, alpha } => self.draw_surface(rect, frame, alpha),
            DrawCmd::Text { x, y, text, color, scale } => {
                self.draw_text(x, y, text, color, scale, fonts)
            }
            DrawCmd::Effect { rect, effect, params } => match effect {
                EffectKind::PreviewCube => self.draw_preview_cube(rect, params),
            },
        }
    }

    fn set_draw_clip(&mut self, rect: Option<Rect>) {
        self.draw_clip = rect.and_then(|rect| self.clip_rect(rect));
    }

    fn draw_background_layer(&mut self, layer: BackgroundLayer, textures: &TextureStore) {
        if let Some(image) = layer.image {
            if let Some(texture) = textures.get(image.image) {
                if texture.format == PixelFormat::Rgb565 {
                    self.draw_rgb565_background_layer(layer.top, layer.bottom, texture, image.tint);
                    return;
                }
            }
        }

        self.clear_gradient(layer.top, layer.bottom);
    }

    fn draw_rgb565_background_layer(
        &mut self,
        top: Color,
        bottom: Color,
        texture: &Texture,
        tint: Color,
    ) {
        let viewport = Rect::new(0, 0, self.width, self.height);
        let Some((x0, y0, x1, y1)) = self.clip(viewport) else {
            return;
        };

        let h = self.height.max(1) as u32;
        let same_size = texture.width == self.width && texture.height == self.height;
        for y in y0..y1 {
            let t = ((y as u32 * 255) / h) as u8;
            let base = top.lerp(bottom, t);
            let row_start = y as usize * self.width as usize;
            for x in x0..x1 {
                let wallpaper = if same_size {
                    sample_rgb565_nearest(texture, x, y, tint)
                } else {
                    let sx = source_axis_fp(x as i32, self.width as i32, texture.width as i32);
                    let sy = source_axis_fp(y as i32, self.height as i32, texture.height as i32);
                    sample_rgb565_bilinear(texture, sx, sy, tint)
                };
                let color = wallpaper
                    .map(|wallpaper| blend_color_over(base, wallpaper))
                    .unwrap_or(base);
                self.framebuffer[row_start + x as usize] = color.argb8888();
            }
        }
    }

    fn draw_preview_cube(&mut self, rect: Rect, params: EffectParams) {
        if params.alpha == 0 || rect.w < 24 || rect.h < 24 {
            return;
        }

        let side = rect.w.min(rect.h) as i32;
        let face = (side * 58 / 100).max(20);
        let depth_x = (face * 34 / 100).max(8);
        let depth_y = -(face * 25 / 100).min(-6);
        let cx = rect.x + rect.w as i32 / 2;
        let cy = rect.y + rect.h as i32 / 2 + side / 16;
        let half = face / 2;

        let front_tl = Point::new(cx - half, cy - half);
        let front_tr = Point::new(cx + half, cy - half);
        let front_br = Point::new(cx + half, cy + half);
        let front_bl = Point::new(cx - half, cy + half);
        let back_tl = Point::new(front_tl.x + depth_x, front_tl.y + depth_y);
        let back_tr = Point::new(front_tr.x + depth_x, front_tr.y + depth_y);
        let back_br = Point::new(front_br.x + depth_x, front_br.y + depth_y);
        let back_bl = Point::new(front_bl.x + depth_x, front_bl.y + depth_y);

        let shadow = Rect::new(cx - half - depth_x / 4, cy + half - 8, face as u16, 22);
        self.fill_circle(shadow, alpha_color(0, 0, 0, 36, params.alpha));

        self.fill_quad(
            [back_tl, back_tr, front_tr, front_tl],
            alpha_color(98, 216, 255, 74, params.alpha),
        );
        self.fill_quad(
            [front_tr, back_tr, back_br, front_br],
            alpha_color(78, 135, 255, 86, params.alpha),
        );
        self.fill_quad(
            [front_tl, front_tr, front_br, front_bl],
            alpha_color(250, 252, 255, 58, params.alpha),
        );

        self.stroke_quad(
            [back_tl, back_tr, front_tr, front_tl],
            alpha_color(255, 255, 255, 96, params.alpha),
        );
        self.stroke_quad(
            [front_tr, back_tr, back_br, front_br],
            alpha_color(255, 255, 255, 84, params.alpha),
        );
        self.stroke_quad(
            [front_tl, front_tr, front_br, front_bl],
            alpha_color(255, 255, 255, 128, params.alpha),
        );
        self.draw_line(front_bl, back_bl, alpha_color(255, 255, 255, 54, params.alpha));
        self.draw_line(back_tl, back_bl, alpha_color(255, 255, 255, 44, params.alpha));
        self.draw_line(back_br, back_bl, alpha_color(255, 255, 255, 38, params.alpha));

        let item_count = params.item_count.min(4);
        for index in 0..item_count {
            let marker = cube_marker_rect(cx, cy, face, depth_x, depth_y, index);
            let color = match index {
                0 => alpha_color(255, 255, 255, 135, params.alpha),
                1 => alpha_color(120, 218, 255, 126, params.alpha),
                2 => alpha_color(255, 210, 118, 116, params.alpha),
                _ => alpha_color(176, 244, 168, 108, params.alpha),
            };
            self.fill_round_rect(marker, 8, color);
        }
    }

    fn clear_gradient(&mut self, top: Color, bottom: Color) {
        let Some((x0, y0, x1, y1)) = self.clip(Rect::new(0, 0, self.width, self.height)) else {
            return;
        };

        let h = self.height.max(1) as u32;
        for y in y0..y1 {
            let t = ((y as u32 * 255) / h) as u8;
            let color = top.lerp(bottom, t).argb8888();
            let start = y as usize * self.width as usize + x0 as usize;
            let end = y as usize * self.width as usize + x1 as usize;
            for pixel in &mut self.framebuffer[start..end] {
                *pixel = color;
            }
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        if color.a == 255 {
            let packed = color.argb8888();
            for y in y0..y1 {
                let start = y as usize * self.width as usize + x0 as usize;
                let end = y as usize * self.width as usize + x1 as usize;
                for pixel in &mut self.framebuffer[start..end] {
                    *pixel = packed;
                }
            }
        } else {
            for y in y0..y1 {
                for x in x0..x1 {
                    self.blend_pixel(x, y, color);
                }
            }
        }
    }

    fn fill_round_rect(&mut self, rect: Rect, radius: u8, color: Color) {
        if radius == 0 {
            self.fill_rect(rect, color);
            return;
        }
        if color.a == 0 {
            return;
        }

        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        let radius = (radius as i32).min(rect.w.min(rect.h) as i32 / 2);
        if radius <= 0 {
            self.fill_rect(rect, color);
            return;
        }

        for y in y0..y1 {
            for x in x0..x1 {
                let coverage = round_rect_coverage(rect, radius, x, y);
                self.blend_coverage_pixel(x, y, color, coverage);
            }
        }
    }

    fn fill_round_rect_gradient(&mut self, rect: Rect, radius: u8, top: Color, bottom: Color) {
        if rect.is_empty() || (top.a == 0 && bottom.a == 0) {
            return;
        }

        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        let radius = (radius as i32).min(rect.w.min(rect.h) as i32 / 2);
        let height = rect.h.saturating_sub(1).max(1) as u32;
        if radius <= 0 && top.a == 255 && bottom.a == 255 {
            for y in y0..y1 {
                let color = gradient_row_color(rect, top, bottom, y, height).argb8888();
                let start = y as usize * self.width as usize + x0 as usize;
                let end = y as usize * self.width as usize + x1 as usize;
                for pixel in &mut self.framebuffer[start..end] {
                    *pixel = color;
                }
            }
            return;
        }

        for y in y0..y1 {
            let color = gradient_row_color(rect, top, bottom, y, height);
            if color.a == 0 {
                continue;
            }
            for x in x0..x1 {
                let coverage = if radius <= 0 {
                    AA_SAMPLE_COUNT
                } else {
                    round_rect_coverage(rect, radius, x, y)
                };
                self.blend_coverage_pixel(x, y, color, coverage);
            }
        }
    }

    fn shadow_round_rect(
        &mut self,
        rect: Rect,
        radius: u8,
        color: Color,
        offset_x: i16,
        offset_y: i16,
        blur: u8,
        spread: u8,
    ) {
        if color.a == 0 || rect.is_empty() {
            return;
        }

        let base = expand_rect(
            offset_rect(rect, offset_x as i32, offset_y as i32),
            spread as u16,
        );
        let base_radius = radius.saturating_add(spread);
        let levels = blur.min(8);
        if levels == 0 {
            self.fill_round_rect(base, base_radius, color);
            return;
        }

        let layer_alpha = ((color.a as u16 + levels as u16 - 1) / levels as u16).max(1) as u8;
        let mut pass = levels;
        while pass > 0 {
            pass -= 1;
            let grow = shadow_pass_growth(blur, pass, levels);
            let expanded = expand_rect(base, grow as u16);
            self.fill_round_rect(
                expanded,
                base_radius.saturating_add(grow),
                color.with_alpha(layer_alpha),
            );
        }
    }

    fn fill_circle(&mut self, rect: Rect, color: Color) {
        if color.a == 0 {
            return;
        }

        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        for y in y0..y1 {
            for x in x0..x1 {
                let coverage = ellipse_coverage(rect, x, y);
                self.blend_coverage_pixel(x, y, color, coverage);
            }
        }
    }

    fn draw_image(&mut self, rect: Rect, texture: &Texture, tint: Color) {
        if rect.w == 0 || rect.h == 0 || texture.width == 0 || texture.height == 0 {
            return;
        }

        match texture.format {
            PixelFormat::Rgb565 => self.blit_rgb565(rect, texture, tint),
            PixelFormat::A8 => self.blit_a8(rect, texture, tint),
        }
    }

    fn draw_vector_icon(&mut self, rect: Rect, icon: VectorIcon, color: Color, svgs: &SvgStore) {
        if color.a == 0 {
            return;
        }

        let rect = icon_square(rect);
        if rect.w < 2 || rect.h < 2 {
            return;
        }

        let id = vector_icon_svg_id(icon);
        let _ = svgs.with_rasterized(id, rect.w, rect.h, |mask| {
            self.draw_svg_mask(rect, mask, color);
        });
    }

    fn draw_svg_mask(&mut self, rect: Rect, mask: &SvgRasterMask, color: Color) {
        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        for y in y0..y1 {
            for x in x0..x1 {
                let local_x = (x as i32 - rect.x).clamp(0, rect.w.saturating_sub(1) as i32);
                let local_y = (y as i32 - rect.y).clamp(0, rect.h.saturating_sub(1) as i32);
                let mask_x = (local_x as u32 * mask.width.saturating_sub(1) as u32
                    / rect.w.saturating_sub(1).max(1) as u32) as u16;
                let mask_y = (local_y as u32 * mask.height.saturating_sub(1) as u32
                    / rect.h.saturating_sub(1).max(1) as u32) as u16;
                let alpha = mask.alpha_at(mask_x, mask_y);
                if alpha != 0 {
                    self.blend_pixel(
                        x,
                        y,
                        color.with_alpha(((color.a as u16 * alpha as u16) / 255) as u8),
                    );
                }
            }
        }
    }

    fn draw_surface(&mut self, rect: Rect, frame: SurfaceFrame, alpha: u8) {
        if rect.w == 0 || rect.h == 0 || frame.is_empty() || alpha == 0 {
            return;
        }

        match frame.format {
            SurfacePixelFormat::Rgb565 => self.blit_surface_rgb565(rect, frame, alpha),
            SurfacePixelFormat::Argb8888 => self.blit_surface_argb8888(rect, frame, alpha),
        }
    }

    fn blit_rgb565(&mut self, rect: Rect, texture: &Texture, tint: Color) {
        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        let dst_w = rect.w as i32;
        let dst_h = rect.h as i32;

        for y in y0..y1 {
            let sy = source_axis_fp(y as i32 - rect.y, dst_h, texture.height as i32);
            for x in x0..x1 {
                let sx = source_axis_fp(x as i32 - rect.x, dst_w, texture.width as i32);
                let Some(color) = sample_rgb565_bilinear(texture, sx, sy, tint) else {
                    continue;
                };
                self.blend_or_store(x, y, color);
            }
        }
    }

    fn blit_a8(&mut self, rect: Rect, texture: &Texture, tint: Color) {
        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        let dst_w = rect.w as i32;
        let dst_h = rect.h as i32;

        for y in y0..y1 {
            let sy = source_axis_fp(y as i32 - rect.y, dst_h, texture.height as i32);
            for x in x0..x1 {
                let sx = source_axis_fp(x as i32 - rect.x, dst_w, texture.width as i32);
                let Some(mask) = sample_a8_bilinear(texture, sx, sy) else {
                    continue;
                };
                if mask == 0 {
                    continue;
                }
                let alpha = ((mask as u16 * tint.a as u16) / 255) as u8;
                self.blend_pixel(x, y, tint.with_alpha(alpha));
            }
        }
    }

    fn blit_surface_rgb565(&mut self, rect: Rect, frame: SurfaceFrame, alpha: u8) {
        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };
        let data = surface_bytes(frame);
        let src_w = frame.width as i32;
        let src_h = frame.height as i32;
        let dst_w = rect.w as i32;
        let dst_h = rect.h as i32;
        let stride = frame.stride_bytes as usize;

        for y in y0..y1 {
            let sy = ((y as i32 - rect.y) * src_h / dst_h).clamp(0, src_h - 1);
            for x in x0..x1 {
                let sx = ((x as i32 - rect.x) * src_w / dst_w).clamp(0, src_w - 1);
                let offset = sy as usize * stride + sx as usize * 2;
                if offset + 1 >= data.len() {
                    continue;
                }

                let packed = data[offset] as u16 | ((data[offset + 1] as u16) << 8);
                self.blend_or_store(x, y, rgb565_to_color(
                    packed,
                    Color::rgba(255, 255, 255, alpha),
                ));
            }
        }
    }

    fn blit_surface_argb8888(&mut self, rect: Rect, frame: SurfaceFrame, alpha: u8) {
        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };
        let data = surface_bytes(frame);
        let src_w = frame.width as i32;
        let src_h = frame.height as i32;
        let dst_w = rect.w as i32;
        let dst_h = rect.h as i32;
        let stride = frame.stride_bytes as usize;

        for y in y0..y1 {
            let sy = ((y as i32 - rect.y) * src_h / dst_h).clamp(0, src_h - 1);
            for x in x0..x1 {
                let sx = ((x as i32 - rect.x) * src_w / dst_w).clamp(0, src_w - 1);
                let offset = sy as usize * stride + sx as usize * 4;
                if offset + 3 >= data.len() {
                    continue;
                }

                let packed = u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                let src_alpha = ((packed >> 24) & 0xff) as u8;
                self.blend_or_store(
                    x,
                    y,
                    Color::rgba(
                        ((packed >> 16) & 0xff) as u8,
                        ((packed >> 8) & 0xff) as u8,
                        (packed & 0xff) as u8,
                        ((src_alpha as u16 * alpha as u16) / 255) as u8,
                    ),
                );
            }
        }
    }

    fn draw_text(
        &mut self,
        x: i32,
        y: i32,
        text: &'static str,
        color: Color,
        scale: u8,
        fonts: &FontStore,
    ) {
        let scale = scale.max(1);
        if self.clip(font_text_bounds(x, y, text, scale)).is_none() {
            return;
        }

        let mut cursor = x;
        let mut cursor_y = y;
        for ch in text.chars() {
            if ch == '\n' {
                cursor = x;
                cursor_y += font_line_height(scale);
                continue;
            }

            self.draw_font_cell(cursor, cursor_y, ch, color, scale, fonts);
            cursor += font_cell_advance(ch, scale);
        }
    }

    fn draw_font_cell(
        &mut self,
        x: i32,
        y: i32,
        ch: char,
        color: Color,
        scale: u8,
        fonts: &FontStore,
    ) {
        fonts.with_rasterized_cell(ch, scale, |glyph| {
            for gy in 0..glyph.height {
                for gx in 0..glyph.width {
                    if let Some(color) = glyph.alpha_at(gx, gy, color) {
                        if let Some((px, py)) = self.visible_pixel(x + gx as i32, y + gy as i32) {
                            self.blend_pixel(px, py, color);
                        }
                    }
                }
            }
        });
    }

    fn fill_quad(&mut self, points: [Point; 4], color: Color) {
        if color.a == 0 {
            return;
        }

        let bounds = quad_bounds(points);
        let Some((_, y0, _, y1)) = self.clip(bounds) else {
            return;
        };

        for y in y0..y1 {
            let mut xs = [0i32; 4];
            let mut count = 0usize;
            let yi = y as i32;

            for edge in 0..4 {
                let a = points[edge];
                let b = points[(edge + 1) & 3];
                if let Some(x) = edge_intersection_x(a, b, yi) {
                    xs[count] = x;
                    count += 1;
                    if count == xs.len() {
                        break;
                    }
                }
            }

            if count < 2 {
                continue;
            }
            sort_i32_prefix(&mut xs, count);

            let x0 = xs[0];
            let x1 = xs[count - 1];
            if x0 >= x1 {
                continue;
            }

            self.fill_rect(
                Rect::new(x0, yi, (x1 - x0).clamp(0, u16::MAX as i32) as u16, 1),
                color,
            );
        }
    }

    fn stroke_quad(&mut self, points: [Point; 4], color: Color) {
        self.draw_line(points[0], points[1], color);
        self.draw_line(points[1], points[2], color);
        self.draw_line(points[2], points[3], color);
        self.draw_line(points[3], points[0], color);
    }

    fn draw_line(&mut self, start: Point, end: Point, color: Color) {
        self.stroke_line(start, end, 1, color);
    }

    fn stroke_line(&mut self, start: Point, end: Point, width: u8, color: Color) {
        if color.a == 0 {
            return;
        }

        if start == end {
            let size = width.max(1) as u16;
            self.fill_circle(Rect::new(start.x - size as i32 / 2, start.y - size as i32 / 2, size, size), color);
            return;
        }

        let bounds = line_bounds(start, end, width);
        let Some((x0, y0, x1, y1)) = self.clip(bounds) else {
            return;
        };

        for y in y0..y1 {
            for x in x0..x1 {
                let coverage = line_coverage(start, end, width, x, y);
                self.blend_coverage_pixel(x, y, color, coverage);
            }
        }
    }

    fn clip(&self, rect: Rect) -> Option<(u16, u16, u16, u16)> {
        let mut x0 = rect.x.max(0);
        let mut y0 = rect.y.max(0);
        let mut x1 = (rect.x + rect.w as i32).min(self.width as i32).max(0);
        let mut y1 = (rect.y + rect.h as i32).min(self.height as i32).max(0);

        if let Some(clip) = self.dirty_clip {
            x0 = x0.max(clip.x);
            y0 = y0.max(clip.y);
            x1 = x1.min(clip.x + clip.w as i32);
            y1 = y1.min(clip.y + clip.h as i32);
        }
        if let Some(clip) = self.draw_clip {
            x0 = x0.max(clip.x);
            y0 = y0.max(clip.y);
            x1 = x1.min(clip.x + clip.w as i32);
            y1 = y1.min(clip.y + clip.h as i32);
        }

        if x0 >= x1 || y0 >= y1 {
            None
        } else {
            Some((x0 as u16, y0 as u16, x1 as u16, y1 as u16))
        }
    }

    fn clip_rect(&self, rect: Rect) -> Option<Rect> {
        let x0 = rect.x.max(0);
        let y0 = rect.y.max(0);
        let x1 = (rect.x + rect.w as i32).min(self.width as i32).max(0);
        let y1 = (rect.y + rect.h as i32).min(self.height as i32).max(0);

        if x0 >= x1 || y0 >= y1 {
            None
        } else {
            Some(Rect::new(x0, y0, (x1 - x0) as u16, (y1 - y0) as u16))
        }
    }

    fn visible_pixel(&self, x: i32, y: i32) -> Option<(u16, u16)> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }

        if let Some(clip) = self.dirty_clip {
            if !clip.contains(Point::new(x, y)) {
                return None;
            }
        }
        if let Some(clip) = self.draw_clip {
            if !clip.contains(Point::new(x, y)) {
                return None;
            }
        }

        Some((x as u16, y as u16))
    }

    fn blend_or_store(&mut self, x: u16, y: u16, color: Color) {
        if color.a == 255 {
            let index = y as usize * self.width as usize + x as usize;
            self.framebuffer[index] = color.argb8888();
        } else {
            self.blend_pixel(x, y, color);
        }
    }

    fn blend_coverage_pixel(&mut self, x: u16, y: u16, color: Color, coverage: u8) {
        match coverage {
            0 => {}
            AA_SAMPLE_COUNT..=u8::MAX => self.blend_or_store(x, y, color),
            _ => {
                let alpha = ((color.a as u16 * coverage as u16) / AA_SAMPLE_COUNT as u16) as u8;
                self.blend_pixel(x, y, color.with_alpha(alpha));
            }
        }
    }

    fn blend_pixel(&mut self, x: u16, y: u16, color: Color) {
        if color.a == 0 {
            return;
        }

        let index = y as usize * self.width as usize + x as usize;
        let dst = self.framebuffer[index];
        let da = ((dst >> 24) & 0xff) as u16;
        let dr = ((dst >> 16) & 0xff) as u16;
        let dg = ((dst >> 8) & 0xff) as u16;
        let db = (dst & 0xff) as u16;
        let sa = color.a as u16;
        let inv = 255 - sa;

        let r = (color.r as u16 * sa + dr * inv) / 255;
        let g = (color.g as u16 * sa + dg * inv) / 255;
        let b = (color.b as u16 * sa + db * inv) / 255;
        let a = sa + da * inv / 255;

        self.framebuffer[index] = ((a as u32) << 24)
            | ((r as u32) << 16)
            | ((g as u32) << 8)
            | b as u32;
    }
}

impl RendererBackend for SoftwareRenderer {
    fn kind(&self) -> RendererKind {
        RendererKind::Software
    }

    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities::SOFTWARE
    }

    fn render_plan(&self, list: &DrawList, textures: &TextureStore) -> RenderPlan {
        render_plan_with_layers(
            list,
            textures,
            self.width,
            self.height,
            RendererCapabilities::SOFTWARE,
        )
    }

    fn draw(
        &mut self,
        list: &DrawList,
        plan: RenderPlan,
        textures: &TextureStore,
        fonts: &FontStore,
        svgs: &SvgStore,
        dirty: DirtyRegion,
    ) {
        Self::draw(self, list, plan, textures, fonts, svgs, dirty);
    }

    fn framebuffer(&self) -> &[u32] {
        Self::framebuffer(self)
    }
}

impl<O: BackgroundPlaneOps> RendererBackend for BackgroundPlaneRenderer<O> {
    fn kind(&self) -> RendererKind {
        self.kind
    }

    fn capabilities(&self) -> RendererCapabilities {
        self.ops.capabilities()
    }

    fn render_plan(&self, list: &DrawList, textures: &TextureStore) -> RenderPlan {
        render_plan_with_layers(
            list,
            textures,
            self.software.width,
            self.software.height,
            self.ops.capabilities(),
        )
    }

    fn draw(
        &mut self,
        list: &DrawList,
        plan: RenderPlan,
        textures: &TextureStore,
        fonts: &FontStore,
        svgs: &SvgStore,
        dirty: DirtyRegion,
    ) {
        let _ = self.submit_background_plane(list, textures, dirty);
        self.software.draw_routes(
            list,
            plan,
            RendererCapabilities::SOFTWARE,
            DrawRouteMask::SUPPORTED,
            textures,
            fonts,
            svgs,
            dirty,
        );
    }

    fn framebuffer(&self) -> &[u32] {
        self.software.framebuffer()
    }
}

fn render_plan_with_layers(
    list: &DrawList,
    textures: &TextureStore,
    width: u16,
    height: u16,
    capabilities: RendererCapabilities,
) -> RenderPlan {
    let mut plan = list.render_plan(capabilities);
    let viewport = Rect::new(0, 0, width, height);
    plan.layers = background_layer_summary(
        list.commands(),
        viewport,
        capabilities,
        DrawRouteMask::SUPPORTED,
        textures,
    );
    plan
}

fn background_layer(
    commands: &[DrawCmd],
    viewport: Rect,
    route_capabilities: RendererCapabilities,
    routes: DrawRouteMask,
    textures: &TextureStore,
) -> Option<BackgroundLayer> {
    let DrawCmd::ClearGradient { top, bottom } = commands.first().copied()? else {
        return None;
    };
    if !routes.accepts(route_capabilities.route_task_kind(DrawTaskKind::Clear)) {
        return None;
    }

    let mut layer = BackgroundLayer {
        top,
        bottom,
        image: None,
        skip_until: 1,
    };

    let Some(DrawCmd::Image { rect, image, tint }) = commands.get(1).copied() else {
        return Some(layer);
    };
    if rect != viewport || !routes.accepts(route_capabilities.route_task_kind(DrawTaskKind::Image))
    {
        return Some(layer);
    }

    match textures.get(image).map(|texture| texture.format) {
        Some(PixelFormat::Rgb565) => {
            layer.image = Some(BackgroundImageLayer { image, tint });
            layer.skip_until = 2;
        }
        Some(PixelFormat::A8) => {}
        None => layer.skip_until = 2,
    }

    Some(layer)
}

fn background_layer_summary(
    commands: &[DrawCmd],
    viewport: Rect,
    route_capabilities: RendererCapabilities,
    routes: DrawRouteMask,
    textures: &TextureStore,
) -> RenderLayerSummary {
    background_layer(commands, viewport, route_capabilities, routes, textures)
        .map(|layer| layer.summary(route_capabilities))
        .unwrap_or_default()
}

fn rgb565_to_color(value: u16, tint: Color) -> Color {
    let (r, g, b) = rgb565_channels(value);
    Color::rgba(
        ((r as u16 * tint.r as u16) / 255) as u8,
        ((g as u16 * tint.g as u16) / 255) as u8,
        ((b as u16 * tint.b as u16) / 255) as u8,
        tint.a,
    )
}

fn sample_rgb565_nearest(texture: &Texture, x: u16, y: u16, tint: Color) -> Option<Color> {
    let (r, g, b) = read_rgb565_channels(texture.data, texture.width as usize, x as usize, y as usize)?;
    Some(Color::rgba(
        ((r as u16 * tint.r as u16) / 255) as u8,
        ((g as u16 * tint.g as u16) / 255) as u8,
        ((b as u16 * tint.b as u16) / 255) as u8,
        tint.a,
    ))
}

fn blend_color_over(base: Color, color: Color) -> Color {
    if color.a == 0 {
        return base;
    }
    if color.a == 255 {
        return Color::rgba(color.r, color.g, color.b, 255);
    }

    let sa = color.a as u16;
    let inv = 255 - sa;
    Color::rgba(
        ((color.r as u16 * sa + base.r as u16 * inv) / 255) as u8,
        ((color.g as u16 * sa + base.g as u16 * inv) / 255) as u8,
        ((color.b as u16 * sa + base.b as u16 * inv) / 255) as u8,
        255,
    )
}

fn gradient_row_color(rect: Rect, top: Color, bottom: Color, y: u16, height: u32) -> Color {
    let local_y = (y as i32 - rect.y)
        .clamp(0, rect.h.saturating_sub(1) as i32) as u32;
    let t = ((local_y * 255) / height) as u8;
    top.lerp(bottom, t)
}

fn rgb565_channels(value: u16) -> (u8, u8, u8) {
    let r = ((value >> 11) & 0x1f) as u16;
    let g = ((value >> 5) & 0x3f) as u16;
    let b = (value & 0x1f) as u16;
    (
        ((r * 255 + 15) / 31) as u8,
        ((g * 255 + 31) / 63) as u8,
        ((b * 255 + 15) / 31) as u8,
    )
}

fn offset_rect(rect: Rect, dx: i32, dy: i32) -> Rect {
    Rect::new(
        rect.x.saturating_add(dx),
        rect.y.saturating_add(dy),
        rect.w,
        rect.h,
    )
}

fn expand_rect(rect: Rect, amount: u16) -> Rect {
    if amount == 0 {
        return rect;
    }

    let amount_i32 = amount as i32;
    let grow = amount.saturating_mul(2);
    Rect::new(
        rect.x.saturating_sub(amount_i32),
        rect.y.saturating_sub(amount_i32),
        rect.w.saturating_add(grow),
        rect.h.saturating_add(grow),
    )
}

fn shadow_pass_growth(blur: u8, pass: u8, levels: u8) -> u8 {
    if levels <= 1 {
        return blur;
    }

    let divisor = levels as u16 - 1;
    ((blur as u16 * pass as u16 + divisor / 2) / divisor) as u8
}

fn round_rect_coverage(rect: Rect, radius: i32, x: u16, y: u16) -> u8 {
    let x = x as i32;
    let y = y as i32;
    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.w as i32;
    let bottom = rect.y + rect.h as i32;

    if (x >= left + radius && x + 1 <= right - radius)
        || (y >= top + radius && y + 1 <= bottom - radius)
    {
        return AA_SAMPLE_COUNT;
    }

    let left4 = left * AA_SCALE;
    let top4 = top * AA_SCALE;
    let right4 = right * AA_SCALE;
    let bottom4 = bottom * AA_SCALE;
    let radius4 = radius * AA_SCALE;
    let radius2 = (radius4 as i64) * (radius4 as i64);
    let mut coverage = 0u8;

    for oy in AA_SAMPLE_OFFSETS {
        for ox in AA_SAMPLE_OFFSETS {
            let sx = x * AA_SCALE + ox;
            let sy = y * AA_SCALE + oy;
            let cx = sx.clamp(left4 + radius4, right4 - radius4);
            let cy = sy.clamp(top4 + radius4, bottom4 - radius4);
            let dx = sx - cx;
            let dy = sy - cy;

            if (dx as i64) * (dx as i64) + (dy as i64) * (dy as i64) <= radius2 {
                coverage += 1;
            }
        }
    }

    coverage
}

fn ellipse_coverage(rect: Rect, x: u16, y: u16) -> u8 {
    if rect.w == 0 || rect.h == 0 {
        return 0;
    }

    let cx4 = rect.x * AA_SCALE + rect.w as i32 * (AA_SCALE / 2);
    let cy4 = rect.y * AA_SCALE + rect.h as i32 * (AA_SCALE / 2);
    let rx4 = (rect.w as i32 * (AA_SCALE / 2)).max(1);
    let ry4 = (rect.h as i32 * (AA_SCALE / 2)).max(1);
    let rx2 = (rx4 as i128) * (rx4 as i128);
    let ry2 = (ry4 as i128) * (ry4 as i128);
    let limit = rx2 * ry2;
    let x = x as i32;
    let y = y as i32;
    let mut coverage = 0u8;

    for oy in AA_SAMPLE_OFFSETS {
        for ox in AA_SAMPLE_OFFSETS {
            let dx = x * AA_SCALE + ox - cx4;
            let dy = y * AA_SCALE + oy - cy4;
            let value = (dx as i128) * (dx as i128) * ry2
                + (dy as i128) * (dy as i128) * rx2;

            if value <= limit {
                coverage += 1;
            }
        }
    }

    coverage
}

fn icon_square(rect: Rect) -> Rect {
    let side = rect.w.min(rect.h);
    let dx = (rect.w.saturating_sub(side) / 2) as i32;
    let dy = (rect.h.saturating_sub(side) / 2) as i32;
    Rect::new(rect.x.saturating_add(dx), rect.y.saturating_add(dy), side, side)
}

fn line_bounds(start: Point, end: Point, width: u8) -> Rect {
    let grow = (width.max(1) as i32 + 1) / 2 + 1;
    let min_x = start.x.min(end.x).saturating_sub(grow);
    let min_y = start.y.min(end.y).saturating_sub(grow);
    let max_x = start.x.max(end.x).saturating_add(grow);
    let max_y = start.y.max(end.y).saturating_add(grow);

    Rect::new(
        min_x,
        min_y,
        (max_x - min_x + 1).clamp(0, u16::MAX as i32) as u16,
        (max_y - min_y + 1).clamp(0, u16::MAX as i32) as u16,
    )
}

fn line_coverage(start: Point, end: Point, width: u8, x: u16, y: u16) -> u8 {
    let sx = start.x as i128 * AA_SCALE as i128 + (AA_SCALE / 2) as i128;
    let sy = start.y as i128 * AA_SCALE as i128 + (AA_SCALE / 2) as i128;
    let ex = end.x as i128 * AA_SCALE as i128 + (AA_SCALE / 2) as i128;
    let ey = end.y as i128 * AA_SCALE as i128 + (AA_SCALE / 2) as i128;
    let vx = ex - sx;
    let vy = ey - sy;
    let len2 = vx * vx + vy * vy;
    if len2 == 0 {
        return 0;
    }

    let half_width = ((width.max(1) as i32 * AA_SCALE + 1) / 2) as i128;
    let threshold = half_width * half_width * len2;
    let x = x as i128;
    let y = y as i128;
    let mut coverage = 0u8;

    for oy in AA_SAMPLE_OFFSETS {
        for ox in AA_SAMPLE_OFFSETS {
            let px = x * AA_SCALE as i128 + ox as i128;
            let py = y * AA_SCALE as i128 + oy as i128;
            let wx = px - sx;
            let wy = py - sy;
            let dot = wx * vx + wy * vy;

            let distance_times_len2 = if dot <= 0 {
                (wx * wx + wy * wy) * len2
            } else if dot >= len2 {
                let dx = px - ex;
                let dy = py - ey;
                (dx * dx + dy * dy) * len2
            } else {
                let cross = wx * vy - wy * vx;
                cross * cross
            };

            if distance_times_len2 <= threshold {
                coverage += 1;
            }
        }
    }

    coverage
}

fn source_axis_fp(dst_pos: i32, dst_len: i32, src_len: i32) -> i32 {
    if src_len <= 1 || dst_len <= 1 {
        return 0;
    }

    let numerator =
        (dst_pos.saturating_mul(2).saturating_add(1)) as i64 * src_len as i64 * 128;
    let fp = numerator / dst_len as i64 - 128;
    fp.clamp(0, (src_len.saturating_sub(1) * 256) as i64) as i32
}

fn sample_rgb565_bilinear(
    texture: &Texture,
    sx_fp: i32,
    sy_fp: i32,
    tint: Color,
) -> Option<Color> {
    let width = texture.width as usize;
    let height = texture.height as usize;
    if width == 0 || height == 0 {
        return None;
    }

    let x0 = ((sx_fp >> 8) as usize).min(width - 1);
    let y0 = ((sy_fp >> 8) as usize).min(height - 1);
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);
    let fx = (sx_fp & 0xff) as u16;
    let fy = (sy_fp & 0xff) as u16;

    let c00 = read_rgb565_channels(texture.data, width, x0, y0)?;
    let c10 = read_rgb565_channels(texture.data, width, x1, y0)?;
    let c01 = read_rgb565_channels(texture.data, width, x0, y1)?;
    let c11 = read_rgb565_channels(texture.data, width, x1, y1)?;

    let r = bilinear_u8(c00.0, c10.0, c01.0, c11.0, fx, fy);
    let g = bilinear_u8(c00.1, c10.1, c01.1, c11.1, fx, fy);
    let b = bilinear_u8(c00.2, c10.2, c01.2, c11.2, fx, fy);

    Some(Color::rgba(
        ((r as u16 * tint.r as u16) / 255) as u8,
        ((g as u16 * tint.g as u16) / 255) as u8,
        ((b as u16 * tint.b as u16) / 255) as u8,
        tint.a,
    ))
}

fn sample_a8_bilinear(texture: &Texture, sx_fp: i32, sy_fp: i32) -> Option<u8> {
    let width = texture.width as usize;
    let height = texture.height as usize;
    if width == 0 || height == 0 {
        return None;
    }

    let x0 = ((sx_fp >> 8) as usize).min(width - 1);
    let y0 = ((sy_fp >> 8) as usize).min(height - 1);
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);
    let fx = (sx_fp & 0xff) as u16;
    let fy = (sy_fp & 0xff) as u16;

    let a00 = *texture.data.get(y0 * width + x0)?;
    let a10 = *texture.data.get(y0 * width + x1)?;
    let a01 = *texture.data.get(y1 * width + x0)?;
    let a11 = *texture.data.get(y1 * width + x1)?;

    Some(bilinear_u8(a00, a10, a01, a11, fx, fy))
}

fn read_rgb565_channels(data: &[u8], width: usize, x: usize, y: usize) -> Option<(u8, u8, u8)> {
    let offset = (y.checked_mul(width)?.checked_add(x)?).checked_mul(2)?;
    let lo = *data.get(offset)? as u16;
    let hi = *data.get(offset + 1)? as u16;
    Some(rgb565_channels(lo | (hi << 8)))
}

fn bilinear_u8(c00: u8, c10: u8, c01: u8, c11: u8, fx: u16, fy: u16) -> u8 {
    let top = lerp_u8(c00, c10, fx);
    let bottom = lerp_u8(c01, c11, fx);
    lerp_u8(top, bottom, fy)
}

fn lerp_u8(a: u8, b: u8, t: u16) -> u8 {
    let t = t.min(255) as u32;
    let inv = 255u32.saturating_sub(t);
    ((a as u32 * inv + b as u32 * t + 127) / 255) as u8
}

fn surface_bytes(frame: SurfaceFrame) -> &'static [u8] {
    let len = frame.bytes.min(frame.frame_bytes());
    if frame.pixels.is_null() || len == 0 {
        &[]
    } else {
        unsafe { ::core::slice::from_raw_parts(frame.pixels, len) }
    }
}

fn shadow_bounds(rect: Rect, offset_x: i16, offset_y: i16, blur: u8, spread: u8) -> Rect {
    let grow = blur as u16 + spread as u16;
    let grow_i32 = grow as i32;
    Rect::new(
        rect.x
            .saturating_add(offset_x as i32)
            .saturating_sub(grow_i32),
        rect.y
            .saturating_add(offset_y as i32)
            .saturating_sub(grow_i32),
        rect.w.saturating_add(grow.saturating_mul(2)),
        rect.h.saturating_add(grow.saturating_mul(2)),
    )
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }

    let ax1 = a.x + a.w as i32;
    let ay1 = a.y + a.h as i32;
    let bx1 = b.x + b.w as i32;
    let by1 = b.y + b.h as i32;

    a.x < bx1 && b.x < ax1 && a.y < by1 && b.y < ay1
}

fn rects_touch_or_overlap(a: Rect, b: Rect) -> bool {
    let ax1 = a.x + a.w as i32;
    let ay1 = a.y + a.h as i32;
    let bx1 = b.x + b.w as i32;
    let by1 = b.y + b.h as i32;

    a.x <= bx1 && b.x <= ax1 && a.y <= by1 && b.y <= ay1
}

fn rect_area(rect: Rect) -> u64 {
    rect.w as u64 * rect.h as u64
}

fn align_rect_to_tile(rect: Rect, width: u16, height: u16, tile_size: u16) -> Option<Rect> {
    if rect.is_empty() || width == 0 || height == 0 || tile_size == 0 {
        return None;
    }

    let tile = tile_size as i32;
    let viewport = Rect::new(0, 0, width, height);
    let rect = rect.intersect(viewport)?;
    let x0 = floor_to_multiple(rect.x.max(0), tile);
    let y0 = floor_to_multiple(rect.y.max(0), tile);
    let x1 = ceil_to_multiple(rect.x.saturating_add(rect.w as i32), tile).min(width as i32);
    let y1 = ceil_to_multiple(rect.y.saturating_add(rect.h as i32), tile).min(height as i32);

    if x0 >= x1 || y0 >= y1 {
        None
    } else {
        Some(Rect::new(
            x0,
            y0,
            (x1 - x0).clamp(0, u16::MAX as i32) as u16,
            (y1 - y0).clamp(0, u16::MAX as i32) as u16,
        ))
    }
}

fn floor_to_multiple(value: i32, step: i32) -> i32 {
    if step <= 1 {
        value
    } else {
        value - value.rem_euclid(step)
    }
}

fn ceil_to_multiple(value: i32, step: i32) -> i32 {
    if step <= 1 {
        value
    } else {
        let rem = value.rem_euclid(step);
        if rem == 0 {
            value
        } else {
            value.saturating_add(step - rem)
        }
    }
}

fn ratio_percent(value: u64, total: u64) -> u8 {
    if total == 0 {
        0
    } else {
        ((value.min(total).saturating_mul(100)) / total) as u8
    }
}

fn saturating_usize_to_u16(value: usize) -> u16 {
    value.min(u16::MAX as usize) as u16
}

fn alpha_color(r: u8, g: u8, b: u8, alpha: u8, factor: u8) -> Color {
    Color::rgba(r, g, b, ((alpha as u16 * factor as u16) / 255) as u8)
}

fn cube_marker_rect(cx: i32, cy: i32, face: i32, depth_x: i32, depth_y: i32, index: u8) -> Rect {
    let size = (face / 8).clamp(6, 14);
    let (x, y) = match index {
        0 => (cx - size / 2, cy - size / 2),
        1 => (
            cx + face / 2 + depth_x / 2 - size / 2,
            cy - face / 6 + depth_y / 2 - size / 2,
        ),
        2 => (
            cx - face / 2 + depth_x / 3 - size / 2,
            cy - face / 2 + depth_y / 2 - size / 2,
        ),
        _ => (cx - size / 2, cy + face / 2 - size - 8),
    };

    Rect::new(x, y, size as u16, size as u16)
}

fn quad_bounds(points: [Point; 4]) -> Rect {
    let mut min_x = points[0].x;
    let mut min_y = points[0].y;
    let mut max_x = points[0].x;
    let mut max_y = points[0].y;

    for point in &points[1..] {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    Rect::new(
        min_x,
        min_y,
        (max_x - min_x + 1).clamp(0, u16::MAX as i32) as u16,
        (max_y - min_y + 1).clamp(0, u16::MAX as i32) as u16,
    )
}

fn edge_intersection_x(a: Point, b: Point, y: i32) -> Option<i32> {
    if a.y == b.y {
        return None;
    }

    let min_y = a.y.min(b.y);
    let max_y = a.y.max(b.y);
    if y < min_y || y >= max_y {
        return None;
    }

    let dy = b.y - a.y;
    let dx = b.x - a.x;
    Some(a.x + ((y - a.y) as i64 * dx as i64 / dy as i64) as i32)
}

fn sort_i32_prefix(values: &mut [i32; 4], len: usize) {
    let mut index = 1usize;
    while index < len {
        let value = values[index];
        let mut cursor = index;
        while cursor > 0 && values[cursor - 1] > value {
            values[cursor] = values[cursor - 1];
            cursor -= 1;
        }
        values[cursor] = value;
        index += 1;
    }
}
