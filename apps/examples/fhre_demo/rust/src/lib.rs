#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_int;
use core::ffi::c_void;
use core::panic::PanicInfo;
use fhre::{
    fixed_from_i32, fixed_to_i32, parse_svg_path, ArcStyle, BlendMode, BorderAlign, BorderSides,
    BorderStyle, Camera, CodecErrorKind, CodecPipelineStats, CodecStats, Color, ComponentStorage,
    DefaultSvgDocument, DirtyTracker, DrawCommand, DrawList, DrawTaskKind, Entity, FillStyle,
    FontId, FrameClock, FramePolicy, FrameStats, GameRuntime, GlyphCache, GlyphRasterOptions,
    GlyphRun, GlyphRunCache,
    GlyphRunCacheStats, GlyphRunItem, GlyphView, GradientStyle, ImageCache, ImageCacheStats,
    ImageDrawStyle, ImageFit, ImageId, ImageView, InputEvent, KeyCode, LayerSpec, LineStyle,
    MaskSpec, MeshDrawOptions, MeshRef, Point, PresentStats, Rect, RenderNode, RenderStats,
    ResourceLoader, Schedule, ShadowStyle, ShapeOptions, Size, SvgCache, SvgDocumentCache,
    SvgDocumentCacheStats, SvgDocumentView, SvgFilterEffect, SvgId, SvgPaint,
    SvgUnsupportedFeature, TexCoord, TextAlign, TextDecor, TextStyle, TexturedMeshRef,
    TexturedVertex3D, Transform3D, TriangleStyle, TtfDecoder, TtfError, Vec3, Vertex3D,
    DEFAULT_CODEC_PIPELINE_STAGES, FHRE_VERSION, GLYPH_RUN_FLAG_GPOS, GLYPH_RUN_FLAG_GSUB,
    GLYPH_RUN_FLAG_KERN, GLYPH_RUN_RESOLVER_CAPACITY, IMAGE_MASK_DOT, IMAGE_SWATCH,
    SVG_DOCUMENT_COMMANDS, SVG_DOCUMENT_PATHS,
};

#[path = "../../../common/fhre_nuttx_runtime.rs"]
mod fhre_nuttx_runtime;

use fhre_nuttx_runtime::{
    elapsed_us, now_us, poll_sim_events, sleep_remaining, NuttxFramebuffer, NuttxInput,
    NuttxResourceLoader,
};

const FRAME_US: u32 = 16_666;
const KEY_1: u32 = b'1' as u32;
const KEY_2: u32 = b'2' as u32;
const KEY_3: u32 = b'3' as u32;
const KEY_4: u32 = b'4' as u32;
const KEY_5: u32 = b'5' as u32;
const KEY_6: u32 = b'6' as u32;
const KEY_7: u32 = b'7' as u32;
const KEY_8: u32 = b'8' as u32;
const KEY_9: u32 = b'9' as u32;
const KEY_0: u32 = b'0' as u32;
const KEY_N: u32 = b'n' as u32;
const KEY_CAP_N: u32 = b'N' as u32;
const KEY_P: u32 = b'p' as u32;
const KEY_CAP_P: u32 = b'P' as u32;
const KEY_Q: u32 = b'q' as u32;
const KEY_CAP_Q: u32 = b'Q' as u32;
const KEY_W: u32 = b'w' as u32;
const KEY_CAP_W: u32 = b'W' as u32;
const FHRE_RESOURCE_PREFIX: &[u8] = b"/etc/fhre/resource/";
const DEMO_IMAGE_FRAW: ImageId = ImageId(900);
const DEMO_IMAGE_PNG: ImageId = ImageId(901);
const DEMO_IMAGE_JPEG: ImageId = ImageId(902);
const DEMO_IMAGE_FILTERS_RGBA: ImageId = ImageId(903);
const DEMO_IMAGE_PALETTE_TRNS: ImageId = ImageId(904);
const DEMO_IMAGE_GRAY_ALPHA16: ImageId = ImageId(905);
const DEMO_IMAGE_GRAY_TRNS: ImageId = ImageId(906);
const DEMO_IMAGE_LVGL_16BIT: ImageId = ImageId(907);
const DEMO_IMAGE_LVGL_PALETTE: ImageId = ImageId(908);
const DEMO_IMAGE_LVGL_CMYK: ImageId = ImageId(909);
const DEMO_IMAGE_LVGL_EXIF90: ImageId = ImageId(910);
const DEMO_IMAGE_LVGL_PROGRESSIVE: ImageId = ImageId(911);
const DEMO_IMAGE_PROGRESSIVE_CMYK_UNSUPPORTED: ImageId = ImageId(915);
const DEMO_TTF_FONT: FontId = FontId(1);
const DEMO_TTF_SIZE: u16 = 18;
const DEMO_TEXT: &str = "FHRE Tiny TTF";
const DEMO_KERNING_TEXT: &str = "AVATAR To WA Yo";
const DEMO_SVG_IDS: [SvgId; 11] = [
    SvgId(900),
    SvgId(901),
    SvgId(902),
    SvgId(903),
    SvgId(904),
    SvgId(905),
    SvgId(906),
    SvgId(907),
    SvgId(908),
    SvgId(909),
    SvgId(910),
];

#[derive(Clone, Copy)]
enum CodecFixture {
    Image {
        id: ImageId,
        path: &'static [u8],
        pinned: bool,
        fallback_fraw: bool,
        feature: CodecFixtureFeature,
        expect: CodecExpectation,
    },
    Ttf {
        path: &'static [u8],
        cache_demo_font: bool,
        feature: CodecFixtureFeature,
        expect: CodecExpectation,
    },
    Svg {
        id: SvgId,
        path: &'static [u8],
        slot: usize,
        feature: CodecFixtureFeature,
        expect: CodecExpectation,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CodecFixtureFeature {
    Fraw,
    PngBasic,
    PngTrns,
    Png16Bit,
    PngAdam7,
    JpegBaseline,
    JpegCmyk,
    JpegExif,
    JpegProgressive,
    JpegProgressiveCmyk,
    TtfGpos,
    OtfCff,
    SvgPath,
    SvgShapeStyle,
    SvgClipMaskFilter,
    Negative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CodecExpectation {
    Success,
    Failure(CodecErrorKind),
}

#[derive(Clone, Copy)]
struct CodecFixtureStats {
    total: u32,
    complex_total: u32,
    expected_successes: u32,
    expected_failures: u32,
    passed: u32,
    mismatches: u32,
    progressive_jpeg: u32,
    progressive_jpeg_scan_fallbacks: u32,
    cff_raster_glyphs: u32,
    cff_fallback_glyphs: u32,
    opentype_shaping_runs: u32,
    opentype_gsub_hits: u32,
    opentype_gpos_hits: u32,
    opentype_kern_hits: u32,
    svg_clip_paths: u32,
    svg_masks: u32,
    svg_filters: u32,
    svg_gradients: u32,
    svg_real_clip_paths: u32,
    svg_real_masks: u32,
    svg_filter_fallbacks: u32,
    svg_gradient_fallbacks: u32,
    vector_mask_rasters: u32,
    vector_mask_scratch_overflows: u32,
    text_shaped_layouts: u32,
    text_layout_overflows: u32,
    text_shaping_fallbacks: u32,
    svg_filter_budget_exceeded: u32,
}

impl CodecFixtureStats {
    const fn new() -> Self {
        Self {
            total: 0,
            complex_total: 0,
            expected_successes: 0,
            expected_failures: 0,
            passed: 0,
            mismatches: 0,
            progressive_jpeg: 0,
            progressive_jpeg_scan_fallbacks: 0,
            cff_raster_glyphs: 0,
            cff_fallback_glyphs: 0,
            opentype_shaping_runs: 0,
            opentype_gsub_hits: 0,
            opentype_gpos_hits: 0,
            opentype_kern_hits: 0,
            svg_clip_paths: 0,
            svg_masks: 0,
            svg_filters: 0,
            svg_gradients: 0,
            svg_real_clip_paths: 0,
            svg_real_masks: 0,
            svg_filter_fallbacks: 0,
            svg_gradient_fallbacks: 0,
            vector_mask_rasters: 0,
            vector_mask_scratch_overflows: 0,
            text_shaped_layouts: 0,
            text_layout_overflows: 0,
            text_shaping_fallbacks: 0,
            svg_filter_budget_exceeded: 0,
        }
    }

    fn record(
        &mut self,
        feature: CodecFixtureFeature,
        expect: CodecExpectation,
        actual: Option<CodecErrorKind>,
    ) {
        self.total = self.total.saturating_add(1);
        if !matches!(
            feature,
            CodecFixtureFeature::Fraw
                | CodecFixtureFeature::PngBasic
                | CodecFixtureFeature::JpegBaseline
                | CodecFixtureFeature::SvgPath
                | CodecFixtureFeature::Negative
        ) {
            self.complex_total = self.complex_total.saturating_add(1);
        }
        match expect {
            CodecExpectation::Success => {
                self.expected_successes = self.expected_successes.saturating_add(1);
            }
            CodecExpectation::Failure(_) => {
                self.expected_failures = self.expected_failures.saturating_add(1);
            }
        }

        let matched = match (expect, actual) {
            (CodecExpectation::Success, None) => true,
            (CodecExpectation::Failure(expected), Some(actual)) => expected == actual,
            _ => false,
        };
        if matched {
            self.passed = self.passed.saturating_add(1);
        } else {
            self.mismatches = self.mismatches.saturating_add(1);
        }
    }

    fn mark_progressive_jpeg(&mut self) {
        self.progressive_jpeg = self.progressive_jpeg.saturating_add(1);
    }

    fn mark_progressive_jpeg_scan_fallback(&mut self) {
        self.progressive_jpeg_scan_fallbacks =
            self.progressive_jpeg_scan_fallbacks.saturating_add(1);
    }

    fn mark_cff_raster_glyph(&mut self) {
        self.cff_raster_glyphs = self.cff_raster_glyphs.saturating_add(1);
    }

    fn mark_cff_fallback_glyph(&mut self) {
        self.cff_fallback_glyphs = self.cff_fallback_glyphs.saturating_add(1);
    }

    fn mark_opentype_shaping_run(&mut self) {
        self.opentype_shaping_runs = self.opentype_shaping_runs.saturating_add(1);
    }

    fn mark_opentype_gsub_hit(&mut self) {
        self.opentype_gsub_hits = self.opentype_gsub_hits.saturating_add(1);
    }

    fn mark_opentype_gpos_hit(&mut self) {
        self.opentype_gpos_hits = self.opentype_gpos_hits.saturating_add(1);
    }

    fn mark_opentype_kern_hit(&mut self) {
        self.opentype_kern_hits = self.opentype_kern_hits.saturating_add(1);
    }

    fn mark_svg_clip_path(&mut self) {
        self.svg_clip_paths = self.svg_clip_paths.saturating_add(1);
    }

    fn mark_svg_mask(&mut self) {
        self.svg_masks = self.svg_masks.saturating_add(1);
    }

    fn mark_svg_filter(&mut self) {
        self.svg_filters = self.svg_filters.saturating_add(1);
    }

    fn mark_svg_gradient(&mut self) {
        self.svg_gradients = self.svg_gradients.saturating_add(1);
    }

    fn mark_svg_real_clip_path(&mut self) {
        self.svg_real_clip_paths = self.svg_real_clip_paths.saturating_add(1);
    }

    fn mark_svg_real_mask(&mut self) {
        self.svg_real_masks = self.svg_real_masks.saturating_add(1);
    }

    fn mark_svg_filter_fallback(&mut self) {
        self.svg_filter_fallbacks = self.svg_filter_fallbacks.saturating_add(1);
    }

    fn mark_svg_gradient_fallback(&mut self) {
        self.svg_gradient_fallbacks = self.svg_gradient_fallbacks.saturating_add(1);
    }

    fn mark_vector_mask_raster(&mut self) {
        self.vector_mask_rasters = self.vector_mask_rasters.saturating_add(1);
    }

    fn mark_vector_mask_scratch_overflow(&mut self) {
        self.vector_mask_scratch_overflows = self.vector_mask_scratch_overflows.saturating_add(1);
    }

    fn mark_text_layout_overflow(&mut self) {
        self.text_layout_overflows = self.text_layout_overflows.saturating_add(1);
    }

    fn mark_text_shaped_layout(&mut self) {
        self.text_shaped_layouts = self.text_shaped_layouts.saturating_add(1);
    }

    fn mark_text_shaping_fallback(&mut self) {
        self.text_shaping_fallbacks = self.text_shaping_fallbacks.saturating_add(1);
    }

    fn mark_svg_filter_budget_exceeded(&mut self) {
        self.svg_filter_budget_exceeded = self.svg_filter_budget_exceeded.saturating_add(1);
    }
}

const DEMO_CODEC_FIXTURES: [CodecFixture; 32] = [
    CodecFixture::Image {
        id: DEMO_IMAGE_FRAW,
        path: b"images/demo_icon.fraw\0",
        pinned: true,
        fallback_fraw: true,
        feature: CodecFixtureFeature::Fraw,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_PNG,
        path: b"images/demo_logo.png\0",
        pinned: true,
        fallback_fraw: false,
        feature: CodecFixtureFeature::PngBasic,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_JPEG,
        path: b"images/demo_photo.jpg\0",
        pinned: true,
        fallback_fraw: false,
        feature: CodecFixtureFeature::JpegBaseline,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_FILTERS_RGBA,
        path: b"images/fixture_filter_rgba.png\0",
        pinned: true,
        fallback_fraw: false,
        feature: CodecFixtureFeature::PngTrns,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_PALETTE_TRNS,
        path: b"images/fixture_palette_trns.png\0",
        pinned: true,
        fallback_fraw: false,
        feature: CodecFixtureFeature::PngTrns,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_GRAY_ALPHA16,
        path: b"images/fixture_gray_alpha16.png\0",
        pinned: true,
        fallback_fraw: false,
        feature: CodecFixtureFeature::Png16Bit,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_GRAY_TRNS,
        path: b"images/fixture_gray_trns.png\0",
        pinned: true,
        fallback_fraw: false,
        feature: CodecFixtureFeature::PngTrns,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_LVGL_16BIT,
        path: b"images/lvgl_16bit_rgba.png\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::Png16Bit,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_LVGL_PALETTE,
        path: b"images/lvgl_palette.png\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::PngTrns,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: ImageId(912),
        path: b"images/fixture_adam7_rgba.png\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::PngAdam7,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_LVGL_CMYK,
        path: b"images/lvgl_cmyk.jpg\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::JpegCmyk,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_LVGL_EXIF90,
        path: b"images/lvgl_exif_90.jpg\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::JpegExif,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: ImageId(913),
        path: b"images/lvgl_exif_180.jpg\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::JpegExif,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: ImageId(914),
        path: b"images/lvgl_exif_270.jpg\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::JpegExif,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: ImageId(950),
        path: b"images/missing_fixture.png\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::Negative,
        expect: CodecExpectation::Failure(CodecErrorKind::MissingResource),
    },
    CodecFixture::Image {
        id: ImageId(951),
        path: b"images/fixture_invalid.bin\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::Negative,
        expect: CodecExpectation::Failure(CodecErrorKind::Invalid),
    },
    CodecFixture::Image {
        id: ImageId(952),
        path: b"images/fixture_truncated.png\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::Negative,
        expect: CodecExpectation::Failure(CodecErrorKind::Truncated),
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_LVGL_PROGRESSIVE,
        path: b"images/lvgl_progressive.jpg\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::JpegProgressive,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Image {
        id: DEMO_IMAGE_PROGRESSIVE_CMYK_UNSUPPORTED,
        path: b"images/fixture_progressive_cmyk.jpg\0",
        pinned: false,
        fallback_fraw: false,
        feature: CodecFixtureFeature::JpegProgressiveCmyk,
        expect: CodecExpectation::Failure(CodecErrorKind::Unsupported),
    },
    CodecFixture::Ttf {
        path: b"fonts/test_gpos_one.ttf\0",
        cache_demo_font: true,
        feature: CodecFixtureFeature::TtfGpos,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Ttf {
        path: b"fonts/test_kern_one.otf\0",
        cache_demo_font: false,
        feature: CodecFixtureFeature::OtfCff,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(900),
        path: b"svg/windows.svg\0",
        slot: 0,
        feature: CodecFixtureFeature::SvgPath,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(901),
        path: b"svg/microsoft.svg\0",
        slot: 1,
        feature: CodecFixtureFeature::SvgPath,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(902),
        path: b"svg/qr-code.svg\0",
        slot: 2,
        feature: CodecFixtureFeature::SvgPath,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(903),
        path: b"svg/award.svg\0",
        slot: 3,
        feature: CodecFixtureFeature::SvgPath,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(904),
        path: b"svg/grouped.svg\0",
        slot: 4,
        feature: CodecFixtureFeature::SvgPath,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(905),
        path: b"svg/complex_style_gradient.svg\0",
        slot: 5,
        feature: CodecFixtureFeature::SvgShapeStyle,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(906),
        path: b"svg/bootstrap_bezier2.svg\0",
        slot: 6,
        feature: CodecFixtureFeature::SvgPath,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(907),
        path: b"svg/bootstrap_cloud_rain.svg\0",
        slot: 7,
        feature: CodecFixtureFeature::SvgPath,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(908),
        path: b"svg/bootstrap_star_fill.svg\0",
        slot: 8,
        feature: CodecFixtureFeature::SvgPath,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(910),
        path: b"svg/clip_mask_filter.svg\0",
        slot: 10,
        feature: CodecFixtureFeature::SvgClipMaskFilter,
        expect: CodecExpectation::Success,
    },
    CodecFixture::Svg {
        id: SvgId(904),
        path: b"svg/overflow.svg\0",
        slot: 4,
        feature: CodecFixtureFeature::Negative,
        expect: CodecExpectation::Failure(CodecErrorKind::Overflow),
    },
];

static mut DEMO_IMAGE_CACHE: Option<ImageCache> = None;
static mut DEMO_TTF_GLYPHS: GlyphCache<96, 65536> = GlyphCache::new();
static mut DEMO_GLYPH_RUN_CACHE: GlyphRunCache<16> = GlyphRunCache::new();
static mut DEMO_TTF_KERNING: [DemoKerningPair; 16] = [DemoKerningPair::EMPTY; 16];
static mut DEMO_SVG_DOCUMENT_CACHE: SvgDocumentCache<
    11,
    SVG_DOCUMENT_PATHS,
    SVG_DOCUMENT_COMMANDS,
> = SvgDocumentCache::new();
static mut DEMO_CODEC_STATS: CodecStats = CodecStats::new();
static mut DEMO_FIXTURE_STATS: CodecFixtureStats = CodecFixtureStats::new();

#[derive(Clone, Copy)]
struct DemoKerningPair {
    left: u32,
    right: u32,
    adjust: i16,
}

impl DemoKerningPair {
    const EMPTY: Self = Self {
        left: 0,
        right: 0,
        adjust: 0,
    };
}

const MESH_VERTICES: [Vertex3D; 4] = [
    Vertex3D::new(Vec3::from_i32(-34, -20, 14), Color::rgba(82, 214, 232, 172)),
    Vertex3D::new(Vec3::from_i32(34, -12, 18), Color::rgba(88, 132, 248, 184)),
    Vertex3D::new(Vec3::from_i32(24, 34, 16), Color::rgba(255, 255, 255, 126)),
    Vertex3D::new(Vec3::from_i32(-28, 26, 12), Color::rgba(0, 150, 136, 168)),
];
const MESH_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];
const TEXTURED_VERTICES: [TexturedVertex3D; 4] = [
    TexturedVertex3D::new(Vec3::from_i32(-46, -28, 18), TexCoord::new(0, 0)),
    TexturedVertex3D::new(Vec3::from_i32(46, -18, 18), TexCoord::new(255, 0)),
    TexturedVertex3D::new(Vec3::from_i32(38, 34, 18), TexCoord::new(255, 255)),
    TexturedVertex3D::new(Vec3::from_i32(-42, 26, 18), TexCoord::new(0, 255)),
];
const TEXTURED_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];
const DEMO_FRAW: [u8; 86] = [
    b'F', b'H', b'R', b'E', b'I', b'M', b'G', b'1', 4, 0, 4, 0, 3, 0, 16, 0, 0, 0, 64, 0, 0, 0,
    255, 255, 255, 255, 80, 132, 248, 255, 82, 214, 232, 255, 255, 255, 255, 255, 80, 132, 248,
    255, 82, 214, 232, 190, 255, 255, 255, 220, 82, 214, 232, 255, 82, 214, 232, 255, 255, 255,
    220, 80, 132, 248, 190, 255, 255, 255, 255, 255, 255, 255, 255, 82, 214, 232, 255, 80, 132,
    248, 255, 255, 255, 255, 255, 255,
];

struct DemoFrawLoader;

impl ResourceLoader for DemoFrawLoader {
    fn load(&mut self, _path: &[u8], out: &mut Vec<u8>) -> bool {
        out.clear();
        out.extend_from_slice(&DEMO_FRAW);
        true
    }
}

extern "C" {
    fn printf(format: *const u8, ...) -> c_int;
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
}

struct NuttxAllocator;

unsafe impl GlobalAlloc for NuttxAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size().max(1)).cast::<u8>()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr.cast::<c_void>());
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        realloc(ptr.cast::<c_void>(), new_size.max(1)).cast::<u8>()
    }
}

#[global_allocator]
static ALLOCATOR: NuttxAllocator = NuttxAllocator;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

fn demo_image_resolver(image: ImageId) -> Option<ImageView> {
    unsafe {
        let cache_slot = core::ptr::addr_of_mut!(DEMO_IMAGE_CACHE);
        match &mut *cache_slot {
            Some(cache) => cache.view(image),
            None => None,
        }
    }
}

fn demo_glyph_resolver(font: FontId, codepoint: u32, _size: u16) -> Option<GlyphView> {
    if font != DEMO_TTF_FONT {
        return None;
    }
    unsafe {
        let cache = core::ptr::addr_of!(DEMO_TTF_GLYPHS);
        (*cache).view(codepoint)
    }
}

fn demo_glyph_id_resolver(font: FontId, glyph_id: u16, size: u16) -> Option<GlyphView> {
    demo_glyph_resolver(font, glyph_id as u32, size)
}

fn demo_kerning_resolver(font: FontId, left: u32, right: u32, _size: u16) -> i16 {
    if font != DEMO_TTF_FONT {
        return 0;
    }
    unsafe {
        let pairs = core::ptr::addr_of!(DEMO_TTF_KERNING);
        let mut index = 0usize;
        while index < (*pairs).len() {
            let pair = (*pairs)[index];
            if pair.left == left && pair.right == right {
                return pair.adjust;
            }
            index += 1;
        }
    }
    0
}

fn demo_glyph_run_resolver(
    font: FontId,
    text: &str,
    size: u16,
    options: ShapeOptions,
) -> Option<GlyphRun<GLYPH_RUN_RESOLVER_CAPACITY>> {
    unsafe {
        let cache = &mut *core::ptr::addr_of_mut!(DEMO_GLYPH_RUN_CACHE);
        cache.lookup(font, text, size, options)
    }
}

fn demo_shape_glyph_run_uncached(
    font: FontId,
    text: &str,
    size: u16,
    options: ShapeOptions,
) -> Option<GlyphRun<GLYPH_RUN_RESOLVER_CAPACITY>> {
    if font != DEMO_TTF_FONT {
        return None;
    }

    let mut chars = ['\0'; GLYPH_RUN_RESOLVER_CAPACITY];
    let mut char_len = 0usize;
    for ch in text.chars() {
        if char_len >= chars.len() {
            break;
        }
        chars[char_len] = ch;
        char_len += 1;
    }

    let mut run = GlyphRun::new();
    let mut previous = 0u32;
    let mut have_previous = false;
    let mut i = 0usize;
    while i < char_len {
        let source_start = i.min(u16::MAX as usize) as u16;
        let mut source_len = 1u16;
        let mut codepoint = chars[i] as u32;
        let mut advance =
            demo_cached_glyph_advance(codepoint, size).unwrap_or((size / 2).max(4) as i16);
        let mut shaping = 0u8;

        if options.enable_gsub() && chars[i] == 'f' {
            if i + 2 < char_len && chars[i + 1] == 'f' && chars[i + 2] == 'i' {
                if let Some(lig_advance) = demo_cached_glyph_advance(0xfb03, size) {
                    codepoint = 0xfb03;
                    advance = lig_advance;
                    source_len = 3;
                    shaping |= GLYPH_RUN_FLAG_GSUB;
                    i += 3;
                } else {
                    i += 1;
                }
            } else if i + 1 < char_len && chars[i + 1] == 'i' {
                if let Some(lig_advance) = demo_cached_glyph_advance(0xfb01, size) {
                    codepoint = 0xfb01;
                    advance = lig_advance;
                    source_len = 2;
                    shaping |= GLYPH_RUN_FLAG_GSUB;
                    i += 2;
                } else {
                    i += 1;
                }
            } else if i + 1 < char_len && chars[i + 1] == 'l' {
                if let Some(lig_advance) = demo_cached_glyph_advance(0xfb02, size) {
                    codepoint = 0xfb02;
                    advance = lig_advance;
                    source_len = 2;
                    shaping |= GLYPH_RUN_FLAG_GSUB;
                    i += 2;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }

        if have_previous {
            if options.enable_gpos() && (previous == 'T' as u32 || previous == 'A' as u32) {
                shaping |= GLYPH_RUN_FLAG_GPOS;
            }
            if options.enable_kern() {
                let adjust = demo_kerning_resolver(font, previous, codepoint, size);
                if adjust != 0 {
                    advance = advance.saturating_add(adjust);
                    shaping |= GLYPH_RUN_FLAG_KERN;
                }
            }
        }

        run.push(GlyphRunItem {
            codepoint,
            glyph_id: codepoint.min(u16::MAX as u32) as u16,
            char_start: source_start,
            char_len: source_len,
            advance: advance.max(1),
            x_offset: 0,
            shaping,
        });
        previous = codepoint;
        have_previous = true;
    }
    if char_len >= chars.len() && text.chars().count() > chars.len() {
        run.overflowed = true;
    }
    Some(run)
}

fn demo_cached_glyph_advance(codepoint: u32, size: u16) -> Option<i16> {
    demo_glyph_resolver(DEMO_TTF_FONT, codepoint, size).map(|glyph| glyph.advance.max(1) as i16)
}

fn demo_svg_resolver(id: SvgId) -> Option<SvgDocumentView> {
    unsafe { (*core::ptr::addr_of_mut!(DEMO_SVG_DOCUMENT_CACHE)).view(id) }
}

fn demo_image_cache_stats() -> ImageCacheStats {
    unsafe {
        let cache_slot = core::ptr::addr_of_mut!(DEMO_IMAGE_CACHE);
        match &mut *cache_slot {
            Some(cache) => cache.stats(),
            None => ImageCacheStats::new(),
        }
    }
}

fn demo_glyph_run_cache_stats() -> GlyphRunCacheStats {
    unsafe { (*core::ptr::addr_of!(DEMO_GLYPH_RUN_CACHE)).stats() }
}

fn demo_svg_document_cache_stats() -> SvgDocumentCacheStats {
    unsafe { (*core::ptr::addr_of!(DEMO_SVG_DOCUMENT_CACHE)).stats() }
}

fn reset_demo_codec_stats() {
    unsafe {
        *core::ptr::addr_of_mut!(DEMO_CODEC_STATS) = CodecStats::new();
        *core::ptr::addr_of_mut!(DEMO_FIXTURE_STATS) = CodecFixtureStats::new();
        (*core::ptr::addr_of_mut!(DEMO_GLYPH_RUN_CACHE)).clear();
    }
}

fn record_demo_codec_error(kind: CodecErrorKind) {
    unsafe {
        (*core::ptr::addr_of_mut!(DEMO_CODEC_STATS)).record(kind);
    }
}

fn record_demo_codec_pipeline(stats: CodecPipelineStats) {
    unsafe {
        (*core::ptr::addr_of_mut!(DEMO_CODEC_STATS)).record_pipeline(stats);
    }
}

fn demo_codec_stats() -> CodecStats {
    unsafe { *core::ptr::addr_of!(DEMO_CODEC_STATS) }
}

fn demo_fixture_stats() -> CodecFixtureStats {
    unsafe { *core::ptr::addr_of!(DEMO_FIXTURE_STATS) }
}

fn record_demo_fixture_result(
    feature: CodecFixtureFeature,
    expect: CodecExpectation,
    actual: Option<CodecErrorKind>,
) {
    unsafe {
        (*core::ptr::addr_of_mut!(DEMO_FIXTURE_STATS)).record(feature, expect, actual);
    }
}

fn record_demo_fixture_feature(mark: fn(&mut CodecFixtureStats)) {
    unsafe {
        mark(&mut *core::ptr::addr_of_mut!(DEMO_FIXTURE_STATS));
    }
}

fn apply_fixture_feature_stats(stats: &mut RenderStats, fixtures: CodecFixtureStats) {
    stats.progressive_jpeg_decodes = stats
        .progressive_jpeg_decodes
        .saturating_add(fixtures.progressive_jpeg);
    stats.progressive_jpeg_scan_fallbacks = stats
        .progressive_jpeg_scan_fallbacks
        .saturating_add(fixtures.progressive_jpeg_scan_fallbacks);
    stats.cff_raster_glyphs = stats
        .cff_raster_glyphs
        .saturating_add(fixtures.cff_raster_glyphs);
    stats.cff_fallback_glyphs = stats
        .cff_fallback_glyphs
        .saturating_add(fixtures.cff_fallback_glyphs);
    stats.opentype_shaping_runs = stats
        .opentype_shaping_runs
        .saturating_add(fixtures.opentype_shaping_runs);
    stats.opentype_gsub_hits = stats
        .opentype_gsub_hits
        .saturating_add(fixtures.opentype_gsub_hits);
    stats.opentype_gpos_hits = stats
        .opentype_gpos_hits
        .saturating_add(fixtures.opentype_gpos_hits);
    stats.opentype_kern_hits = stats
        .opentype_kern_hits
        .saturating_add(fixtures.opentype_kern_hits);
    stats.svg_clip_paths = stats.svg_clip_paths.saturating_add(fixtures.svg_clip_paths);
    stats.svg_masks = stats.svg_masks.saturating_add(fixtures.svg_masks);
    stats.svg_filters = stats.svg_filters.saturating_add(fixtures.svg_filters);
    stats.svg_gradients = stats.svg_gradients.saturating_add(fixtures.svg_gradients);
    stats.svg_real_clip_paths = stats
        .svg_real_clip_paths
        .saturating_add(fixtures.svg_real_clip_paths);
    stats.svg_real_masks = stats.svg_real_masks.saturating_add(fixtures.svg_real_masks);
    stats.svg_filter_fallbacks = stats
        .svg_filter_fallbacks
        .saturating_add(fixtures.svg_filter_fallbacks);
    stats.svg_gradient_fallbacks = stats
        .svg_gradient_fallbacks
        .saturating_add(fixtures.svg_gradient_fallbacks);
    stats.vector_mask_rasters = stats
        .vector_mask_rasters
        .saturating_add(fixtures.vector_mask_rasters);
    stats.vector_mask_scratch_overflows = stats
        .vector_mask_scratch_overflows
        .saturating_add(fixtures.vector_mask_scratch_overflows);
    stats.text_shaped_layouts = stats
        .text_shaped_layouts
        .saturating_add(fixtures.text_shaped_layouts);
    stats.text_layout_overflows = stats
        .text_layout_overflows
        .saturating_add(fixtures.text_layout_overflows);
    stats.text_shaping_fallbacks = stats
        .text_shaping_fallbacks
        .saturating_add(fixtures.text_shaping_fallbacks);
    stats.svg_filter_budget_exceeded = stats
        .svg_filter_budget_exceeded
        .saturating_add(fixtures.svg_filter_budget_exceeded);
}

fn prewarm_demo_resources() {
    reset_demo_codec_stats();
    unsafe {
        (*core::ptr::addr_of_mut!(DEMO_SVG_DOCUMENT_CACHE)).clear();
        let cache_slot = core::ptr::addr_of_mut!(DEMO_IMAGE_CACHE);
        if (*cache_slot).is_none() {
            *cache_slot = Some(ImageCache::new(8, 2 * 1024 * 1024));
        }
    }
    let mut index = 0usize;
    while index < DEMO_CODEC_FIXTURES.len() {
        prewarm_codec_fixture(DEMO_CODEC_FIXTURES[index]);
        index += 1;
    }
    prewarm_demo_glyph_runs();
}

fn prewarm_demo_glyph_runs() {
    const RUNS: [(&str, u16, bool); 5] = [
        (
            "wrap align selection underline strikethrough spans here",
            14,
            false,
        ),
        ("KERN OFF: AVATAR To WA Yo", 13, false),
        ("KERN ON : AVATAR To WA Yo", 13, true),
        ("MISSING: \u{E000}", 13, false),
        ("AVATAR To office ffi fl", DEMO_TTF_SIZE, true),
    ];
    let mut index = 0usize;
    while index < RUNS.len() {
        let (text, size, kerning) = RUNS[index];
        let options = ShapeOptions::new(true, true, kerning);
        if let Some(run) = demo_shape_glyph_run_uncached(DEMO_TTF_FONT, text, size, options) {
            unsafe {
                let cache = &mut *core::ptr::addr_of_mut!(DEMO_GLYPH_RUN_CACHE);
                let _ = cache.insert(DEMO_TTF_FONT, text, size, options, run);
            }
        }
        index += 1;
    }
}

fn prewarm_codec_fixture(fixture: CodecFixture) {
    match fixture {
        CodecFixture::Image {
            id,
            path,
            pinned,
            fallback_fraw,
            feature,
            expect,
        } => {
            let actual = prewarm_image_fixture(id, path, pinned, fallback_fraw);
            if actual.is_none() && feature == CodecFixtureFeature::JpegProgressive {
                record_demo_fixture_feature(CodecFixtureStats::mark_progressive_jpeg);
            }
            if actual == Some(CodecErrorKind::Unsupported)
                && feature == CodecFixtureFeature::JpegProgressiveCmyk
            {
                record_demo_fixture_feature(CodecFixtureStats::mark_progressive_jpeg_scan_fallback);
            }
            record_demo_fixture_result(feature, expect, actual);
        }
        CodecFixture::Ttf {
            path,
            cache_demo_font,
            feature,
            expect,
        } => {
            let actual = prewarm_ttf_fixture(path, cache_demo_font);
            if let Some(kind) = actual {
                record_demo_codec_error(kind);
            }
            record_demo_fixture_result(feature, expect, actual);
        }
        CodecFixture::Svg {
            id,
            path,
            slot,
            feature,
            expect,
        } => {
            let actual = prewarm_svg_fixture(id, path, slot);
            if let Some(kind) = actual {
                record_demo_codec_error(kind);
            }
            record_demo_fixture_result(feature, expect, actual);
        }
    }
}

fn prewarm_image_fixture(
    id: ImageId,
    path: &'static [u8],
    pinned: bool,
    fallback_fraw: bool,
) -> Option<CodecErrorKind> {
    unsafe {
        let cache_slot = core::ptr::addr_of_mut!(DEMO_IMAGE_CACHE);
        let Some(cache) = &mut *cache_slot else {
            return Some(CodecErrorKind::Overflow);
        };
        let mut loader = NuttxResourceLoader::new(FHRE_RESOURCE_PREFIX);
        match cache.prewarm_result(id, path, &mut loader, pinned) {
            Ok(_) => None,
            Err(primary_error) => {
                if fallback_fraw {
                    let mut fallback = DemoFrawLoader;
                    if cache
                        .prewarm_result(id, b"demo.fraw\0", &mut fallback, pinned)
                        .is_ok()
                    {
                        return None;
                    }
                }
                Some(primary_error)
            }
        }
    }
}

fn prewarm_ttf_fixture(path: &'static [u8], cache_demo_font: bool) -> Option<CodecErrorKind> {
    let mut loader = NuttxResourceLoader::new(FHRE_RESOURCE_PREFIX);
    let mut bytes = Vec::new();
    if !loader.load(path, &mut bytes) {
        return Some(CodecErrorKind::MissingResource);
    }
    let plan = TtfDecoder::plan_pipeline::<DEFAULT_CODEC_PIPELINE_STAGES>();
    record_demo_codec_pipeline(plan.stats());
    let face = match TtfDecoder::parse(&bytes) {
        Ok(face) => face,
        Err(error) => {
            return Some(map_ttf_error(error));
        }
    };
    let shaped = face.shape_text::<64>("AVATAR To office ffi fl", ShapeOptions::LATIN);
    if shaped.len == 0 {
        record_demo_fixture_feature(CodecFixtureStats::mark_text_shaping_fallback);
        return Some(CodecErrorKind::Invalid);
    }
    let layout = fhre::TextLayout::<1>::from_glyph_run(
        &shaped,
        fhre::TextLayoutOptions::new(48, DEMO_TTF_SIZE.saturating_add(4)),
    );
    if shaped.overflowed || layout.overflowed {
        record_demo_fixture_feature(CodecFixtureStats::mark_text_layout_overflow);
    } else {
        record_demo_fixture_feature(CodecFixtureStats::mark_text_shaped_layout);
    }
    record_demo_fixture_feature(CodecFixtureStats::mark_opentype_shaping_run);
    let mut i = 0usize;
    while i < shaped.len {
        let flags = shaped.items[i].shaping;
        if (flags & GLYPH_RUN_FLAG_GSUB) != 0 {
            record_demo_fixture_feature(CodecFixtureStats::mark_opentype_gsub_hit);
        }
        if (flags & GLYPH_RUN_FLAG_GPOS) != 0 {
            record_demo_fixture_feature(CodecFixtureStats::mark_opentype_gpos_hit);
        }
        if (flags & GLYPH_RUN_FLAG_KERN) != 0 {
            record_demo_fixture_feature(CodecFixtureStats::mark_opentype_kern_hit);
        }
        i += 1;
    }
    if face.kind() == fhre::FontFaceKind::OpenTypeCff {
        let mut rasterized = false;
        let mut cp = 32u32;
        while cp < 128 {
            if face.glyph_index(cp).is_some()
                && face
                    .rasterize_glyph(
                        cp,
                        GlyphRasterOptions {
                            pixel_size: DEMO_TTF_SIZE as u8,
                        },
                    )
                    .is_ok()
            {
                rasterized = true;
                break;
            }
            cp += 1;
        }
        if rasterized {
            record_demo_fixture_feature(CodecFixtureStats::mark_cff_raster_glyph);
        } else {
            record_demo_fixture_feature(CodecFixtureStats::mark_cff_fallback_glyph);
        }
    }
    if !cache_demo_font {
        return None;
    }
    unsafe {
        let cache = &mut *core::ptr::addr_of_mut!(DEMO_TTF_GLYPHS);
        cache.clear();
        for ch in DEMO_TEXT
            .chars()
            .chain(DEMO_KERNING_TEXT.chars())
            .chain(" FHRE 0123456789?□".chars())
            .chain(['\u{fb01}', '\u{fb02}', '\u{fb03}'])
        {
            let glyph = match face.rasterize_glyph(
                ch as u32,
                GlyphRasterOptions {
                    pixel_size: DEMO_TTF_SIZE as u8,
                },
            ) {
                Ok(glyph) => glyph,
                Err(error) => {
                    record_demo_codec_error(map_ttf_error(error));
                    continue;
                }
            };
            let _ = cache.insert_a8_metrics(
                glyph.codepoint,
                glyph.width,
                glyph.height,
                glyph.advance,
                glyph.bearing_x,
                glyph.bearing_y,
                &glyph.data,
            );
        }
        let pairs = &mut *core::ptr::addr_of_mut!(DEMO_TTF_KERNING);
        *pairs = [DemoKerningPair::EMPTY; 16];
        let candidates = [
            ('A', 'V'),
            ('V', 'A'),
            ('T', 'o'),
            ('W', 'A'),
            ('Y', 'o'),
            ('T', 'A'),
            ('A', 'T'),
            ('L', 'T'),
        ];
        let units = face.info().units_per_em.max(1) as i32;
        let mut out = 0usize;
        for (left, right) in candidates {
            let Some(left_glyph) = face.glyph_index(left as u32) else {
                continue;
            };
            let Some(right_glyph) = face.glyph_index(right as u32) else {
                continue;
            };
            let raw = face.kerning(left_glyph, right_glyph) as i32;
            if raw == 0 || out >= pairs.len() {
                continue;
            }
            pairs[out] = DemoKerningPair {
                left: left as u32,
                right: right as u32,
                adjust: ((raw * DEMO_TTF_SIZE as i32) / units) as i16,
            };
            out += 1;
        }
    }
    None
}

fn map_ttf_error(error: TtfError) -> CodecErrorKind {
    match error {
        TtfError::Truncated => CodecErrorKind::Truncated,
        TtfError::Unsupported => CodecErrorKind::Unsupported,
        TtfError::BadSignature | TtfError::MissingTable | TtfError::InvalidGlyph => {
            CodecErrorKind::Invalid
        }
    }
}

fn prewarm_svg_fixture(id: SvgId, path: &'static [u8], slot: usize) -> Option<CodecErrorKind> {
    if slot >= DEMO_SVG_IDS.len() || DEMO_SVG_IDS[slot] != id {
        return Some(CodecErrorKind::Invalid);
    }
    let plan = DefaultSvgDocument::plan_pipeline::<DEFAULT_CODEC_PIPELINE_STAGES>();
    record_demo_codec_pipeline(plan.stats());
    let mut loader = NuttxResourceLoader::new(FHRE_RESOURCE_PREFIX);
    let view = unsafe {
        let cache = core::ptr::addr_of_mut!(DEMO_SVG_DOCUMENT_CACHE);
        match (*cache).prewarm_result(id, path, &mut loader, Rect::new(0, 0, 16, 16)) {
            Ok(view) => view,
            Err(error) => return Some(error),
        }
    };
    let Some(doc) = view.get() else {
        return Some(CodecErrorKind::Invalid);
    };
    record_demo_svg_document_features(doc);
    None
}

fn record_demo_svg_document_features(doc: &DefaultSvgDocument) {
    if svg_unsupported(doc, SvgUnsupportedFeature::ClipPath) {
        record_demo_fixture_feature(CodecFixtureStats::mark_svg_clip_path);
    }
    if svg_unsupported(doc, SvgUnsupportedFeature::Mask) {
        record_demo_fixture_feature(CodecFixtureStats::mark_svg_mask);
    }
    if svg_unsupported(doc, SvgUnsupportedFeature::Filter) {
        record_demo_fixture_feature(CodecFixtureStats::mark_svg_filter);
    }
    if svg_unsupported(doc, SvgUnsupportedFeature::GradientSpread) {
        record_demo_fixture_feature(CodecFixtureStats::mark_svg_gradient);
        record_demo_fixture_feature(CodecFixtureStats::mark_svg_gradient_fallback);
    }
    if svg_unsupported(doc, SvgUnsupportedFeature::ClipPathComplex) {
        record_demo_fixture_feature(CodecFixtureStats::mark_svg_clip_path);
    }
    if svg_unsupported(doc, SvgUnsupportedFeature::MaskComplex) {
        record_demo_fixture_feature(CodecFixtureStats::mark_svg_mask);
    }
    if svg_unsupported(doc, SvgUnsupportedFeature::FilterBudgetExceeded) {
        record_demo_fixture_feature(CodecFixtureStats::mark_svg_filter_fallback);
        record_demo_fixture_feature(CodecFixtureStats::mark_svg_filter_budget_exceeded);
    }
    let mut i = 0usize;
    while i < doc.len {
        let node = doc.paths[i];
        if node.has_clip_path {
            record_demo_fixture_feature(CodecFixtureStats::mark_svg_clip_path);
            record_demo_fixture_feature(CodecFixtureStats::mark_svg_real_clip_path);
            record_demo_fixture_feature(CodecFixtureStats::mark_vector_mask_raster);
            if node
                .clip
                .map(|rect| rect.w as usize * rect.h as usize > 128 * 1024)
                .unwrap_or(false)
            {
                record_demo_fixture_feature(CodecFixtureStats::mark_vector_mask_scratch_overflow);
            }
        }
        if node.has_mask_path {
            record_demo_fixture_feature(CodecFixtureStats::mark_svg_mask);
            record_demo_fixture_feature(CodecFixtureStats::mark_svg_real_mask);
            record_demo_fixture_feature(CodecFixtureStats::mark_vector_mask_raster);
            if node
                .mask
                .map(|rect| rect.w as usize * rect.h as usize > 128 * 1024)
                .unwrap_or(false)
            {
                record_demo_fixture_feature(CodecFixtureStats::mark_vector_mask_scratch_overflow);
            }
        }
        if node.filter != SvgFilterEffect::None {
            record_demo_fixture_feature(CodecFixtureStats::mark_svg_filter);
        }
        if matches!(
            node.fill,
            SvgPaint::LinearGradient(_, _) | SvgPaint::RadialGradient(_, _)
        ) || matches!(
            node.stroke,
            SvgPaint::LinearGradient(_, _) | SvgPaint::RadialGradient(_, _)
        ) {
            record_demo_fixture_feature(CodecFixtureStats::mark_svg_gradient);
        }
        i += 1;
    }
}

fn svg_unsupported(doc: &DefaultSvgDocument, feature: SvgUnsupportedFeature) -> bool {
    (doc.unsupported_features & (1 << (feature as u32))) != 0
}

#[derive(Clone, Copy)]
struct Bubble {
    base_x: i32,
    base_y: i32,
    phase: u8,
    radius: u16,
    primary: Color,
    secondary: Color,
}

struct DemoState {
    runtime: GameRuntime<8, 16, 4>,
    transforms: ComponentStorage<Transform3D, 8>,
    bubbles: ComponentStorage<Bubble, 8>,
    dirty: DirtyTracker<12>,
    glyph_cache: GlyphCache<16, 560>,
    svg_cache: SvgCache<2, 16>,
    previous_bubble_bounds: [Option<Rect>; 8],
    previous_progress: Option<Rect>,
    last_render_stats: RenderStats,
    mode: StressMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StressMode {
    Rect,
    Border,
    Shadow,
    Arc,
    Text,
    Image,
    Vector,
    Layer,
    Mask,
    Triangle,
    ThreeD,
    Stress,
}

impl StressMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Rect => "RECT",
            Self::Border => "BORDER",
            Self::Shadow => "SHADOW",
            Self::Arc => "ARC",
            Self::Text => "TEXT",
            Self::Image => "IMAGE",
            Self::Vector => "VECTOR",
            Self::Layer => "LAYER",
            Self::Mask => "MASK",
            Self::Triangle => "TRIANGLE",
            Self::ThreeD => "3D",
            Self::Stress => "STRESS",
        }
    }

    const fn benchmark_profile(self) -> &'static str {
        match self {
            Self::Rect => "DIRTY RECT BASE",
            Self::Border => "DIRTY BORDER/LINE",
            Self::Shadow => "DIRTY SHADOW",
            Self::Arc => "DIRTY ARC",
            Self::Text => "TEXT HEAVY",
            Self::Image => "IMAGE HEAVY",
            Self::Vector => "VECTOR HEAVY",
            Self::Layer => "LAYER HEAVY",
            Self::Mask => "MASK HEAVY",
            Self::Triangle => "TRIANGLE HEAVY",
            Self::ThreeD => "3D HEAVY",
            Self::Stress => "FULL REDRAW STRESS",
        }
    }

    const fn next(self) -> Self {
        match self {
            Self::Rect => Self::Border,
            Self::Border => Self::Shadow,
            Self::Shadow => Self::Arc,
            Self::Arc => Self::Text,
            Self::Text => Self::Image,
            Self::Image => Self::Vector,
            Self::Vector => Self::Layer,
            Self::Layer => Self::Mask,
            Self::Mask => Self::Triangle,
            Self::Triangle => Self::ThreeD,
            Self::ThreeD => Self::Stress,
            Self::Stress => Self::Rect,
        }
    }

    const fn previous(self) -> Self {
        match self {
            Self::Rect => Self::Stress,
            Self::Border => Self::Rect,
            Self::Shadow => Self::Border,
            Self::Arc => Self::Shadow,
            Self::Text => Self::Arc,
            Self::Image => Self::Text,
            Self::Vector => Self::Image,
            Self::Layer => Self::Vector,
            Self::Mask => Self::Layer,
            Self::Triangle => Self::Mask,
            Self::ThreeD => Self::Triangle,
            Self::Stress => Self::ThreeD,
        }
    }
}

impl DemoState {
    fn new(width: u16, height: u16) -> Self {
        let mut state = Self {
            runtime: GameRuntime::new(Camera::screen_canvas(width, height)),
            transforms: ComponentStorage::new(),
            bubbles: ComponentStorage::new(),
            dirty: DirtyTracker::new(Rect::new(0, 0, width, height)),
            glyph_cache: GlyphCache::new(),
            svg_cache: SvgCache::new(),
            previous_bubble_bounds: [None; 8],
            previous_progress: None,
            last_render_stats: RenderStats::new(),
            mode: StressMode::Rect,
        };

        let _ = state.glyph_cache.lookup_or_insert_debug(b'F');
        let _ = state.glyph_cache.lookup_or_insert_debug(b'H');
        let _ = state.glyph_cache.lookup_or_insert_debug(b'R');
        let _ = state.glyph_cache.lookup_or_insert_debug(b'E');
        let parsed = parse_svg_path(Rect::new(0, 0, 16, 16), "M2 2 L14 8 L2 14 Z", 1);
        let _ = state.svg_cache.insert_path(SvgId(42), parsed);
        let ttf_like = [255u8; 35];
        let _ = state.glyph_cache.insert_a8(b'T' as u32, 5, 7, 6, &ttf_like);

        let panel = Rect::new(24, 36, width.saturating_sub(48), height.saturating_sub(72));
        let base_y = panel.y + panel.h as i32 - 64;
        for i in 0..5 {
            let x = panel.x + 26 + i * 38;
            state.spawn_bubble(
                x,
                base_y - (i & 1) * 18,
                (i * 19) as u8,
                12 + (i as u16 * 3),
                Color::rgba(60, 210, 225, 170),
                Color::rgba(85, 112, 248, 210),
            );
        }

        state
    }

    fn spawn_bubble(
        &mut self,
        x: i32,
        y: i32,
        phase: u8,
        radius: u16,
        primary: Color,
        secondary: Color,
    ) {
        if let Some(entity) = self.runtime.entities.spawn() {
            self.transforms.insert(entity, Transform3D::screen(x, y, 8));
            self.bubbles.insert(
                entity,
                Bubble {
                    base_x: x,
                    base_y: y,
                    phase,
                    radius,
                    primary,
                    secondary,
                },
            );
        }
    }
}

fn handle_demo_input(state: &mut DemoState) -> u32 {
    let mut count = 0u32;
    while let Some(event) = state.runtime.pop_input() {
        count = count.saturating_add(1);
        if let InputEvent::Key(key) = event {
            if !key.pressed {
                continue;
            }
            match key.code {
                KeyCode::Right
                | KeyCode::Down
                | KeyCode::Char(KEY_N)
                | KeyCode::Char(KEY_CAP_N) => {
                    state.mode = state.mode.next();
                    state.dirty.force_full();
                }
                KeyCode::Left | KeyCode::Up | KeyCode::Char(KEY_P) | KeyCode::Char(KEY_CAP_P) => {
                    state.mode = state.mode.previous();
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_1) => {
                    state.mode = StressMode::Rect;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_2) => {
                    state.mode = StressMode::Border;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_3) => {
                    state.mode = StressMode::Shadow;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_4) => {
                    state.mode = StressMode::Arc;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_5) => {
                    state.mode = StressMode::Text;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_6) => {
                    state.mode = StressMode::Image;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_7) => {
                    state.mode = StressMode::Vector;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_8) => {
                    state.mode = StressMode::Layer;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_9) => {
                    state.mode = StressMode::Mask;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_0) => {
                    state.mode = StressMode::Triangle;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_Q) | KeyCode::Char(KEY_CAP_Q) => {
                    state.mode = StressMode::ThreeD;
                    state.dirty.force_full();
                }
                KeyCode::Char(KEY_W) | KeyCode::Char(KEY_CAP_W) => {
                    state.mode = StressMode::Stress;
                    state.dirty.force_full();
                }
                _ => {}
            }
        }
    }
    count
}

fn advance_demo(state: &mut DemoState) {
    state.runtime.tick(FRAME_US);
    let frame = state.runtime.render.time.frame as i32;
    let bubbles = &state.bubbles;
    state.transforms.for_each_mut(|entity, transform| {
        if let Some(bubble) = bubbles.get(entity) {
            let wave = ((frame + bubble.phase as i32) % 48) - 24;
            transform.position.x = fixed_from_i32(bubble.base_x + wave / 6);
            transform.position.y = fixed_from_i32(bubble.base_y + demo_abs_i32(wave) / 3 - 8);
        }
    });
}

fn demo_abs_i32(value: i32) -> i32 {
    if value < 0 {
        value.saturating_neg()
    } else {
        value
    }
}

#[no_mangle]
pub extern "C" fn fhre_demo_main(_argc: i32, _argv: *mut *mut u8) -> i32 {
    let Some(mut fb) = NuttxFramebuffer::open() else {
        unsafe {
            printf(b"fhre_demo: cannot open /dev/fb0\n\0".as_ptr());
        }
        return 1;
    };

    unsafe {
        printf(b"fhre_demo: rust no_std FHRE demo\n\0".as_ptr());
    }

    prewarm_demo_resources();
    let mut demo = Box::new(DemoState::new(fb.surface.width(), fb.surface.height()));
    let mut input = NuttxInput::open();
    let mut schedule: Schedule<DemoState, 1> = Schedule::new();
    schedule.add_system(advance_demo);
    let mut clock = FrameClock::new(FramePolicy::SIXTY_FPS);
    let mut last_frame_stats = FrameStats::new(FramePolicy::SIXTY_FPS);

    loop {
        poll_sim_events();
        let frame_start = now_us();
        let pumped = input.pump(&mut demo.runtime.input, frame_start) as u32;
        let handled = handle_demo_input(&mut demo);
        schedule.run(&mut demo);
        let draw_start = now_us();
        let drew_frame = draw_scene(&mut fb, &mut demo, last_frame_stats);
        let draw_us = elapsed_us(draw_start, now_us());
        let present_start = now_us();
        let present = if drew_frame {
            fb.present()
        } else {
            PresentStats::skipped()
        };
        let present_us = elapsed_us(present_start, now_us());
        last_frame_stats =
            clock.finish_frame(draw_us, present_us, pumped.saturating_add(handled), present);
        sleep_remaining(frame_start, last_frame_stats);
    }
}

fn draw_scene(fb: &mut NuttxFramebuffer, state: &mut DemoState, frame_stats: FrameStats) -> bool {
    fb.surface.set_image_resolver(Some(demo_image_resolver));
    fb.surface.set_glyph_resolver(Some(demo_glyph_resolver));
    fb.surface
        .set_glyph_id_resolver(Some(demo_glyph_id_resolver));
    fb.surface
        .set_glyph_run_resolver(Some(demo_glyph_run_resolver));
    fb.surface.set_kerning_resolver(Some(demo_kerning_resolver));
    fb.surface.set_svg_resolver(Some(demo_svg_resolver));
    let width = fb.surface.width();
    let height = fb.surface.height();
    let w = width as i32;
    let h = height as i32;
    let frame = state.runtime.render.time.frame;
    let whole = Rect::new(0, 0, width, height);
    let camera = Camera::screen_canvas(width, height);
    let mut list: DrawList<192> = DrawList::new();
    state.dirty.begin_frame(whole);

    list.push(DrawCommand::FillGradient {
        rect: whole,
        depth: fixed_from_i32(-64),
        top: Color::rgb(4, 8, 18),
        bottom: Color::rgb(18, 48, 64),
    });

    let cx = w / 2;
    let cy = h / 2 - 10;
    for i in 0..24 {
        let angle = ((i * 15) & 255) - 128;
        let dx = ((angle * 37) % (w.max(2) / 2)).clamp(-(w / 2), w / 2);
        let dy = (((angle + i * 11) * 23) % (h.max(2) / 2)).clamp(-(h / 2), h / 2);
        let tint = (64 + i * 7) as u8;
        list.push(DrawCommand::StrokeLine {
            from: Point::new(cx, cy),
            to: Point::new(cx + dx, cy + dy),
            depth: fixed_from_i32(-8),
            width: 1,
            color: Color::rgba(70, 160, 240, tint),
        });
    }

    let panel_node = RenderNode::new(
        Transform3D::screen(24, 36, 0),
        Size::new(width.saturating_sub(48), height.saturating_sub(72)),
    );
    let panel = panel_node.project(&camera);
    list.push(DrawCommand::FillRoundRect {
        rect: panel.rect,
        depth: panel.depth,
        radius: 22,
        color: Color::rgba(242, 248, 255, 46),
    });
    let mesh = MeshRef::new(&MESH_VERTICES, &MESH_INDICES);
    let mesh_options = if state.mode == StressMode::ThreeD || state.mode == StressMode::Stress {
        MeshDrawOptions {
            wireframe: true,
            fill: true,
            painter_sort: true,
            wire_color: Color::rgba(255, 255, 255, 220),
        }
    } else {
        MeshDrawOptions::FLAT
    };
    let _ = mesh.emit_with_options(
        &camera,
        Transform3D::screen(panel.rect.right() - 88, panel.rect.y + 82, 0),
        mesh_options,
        &mut list,
    );
    if state.mode == StressMode::ThreeD || state.mode == StressMode::Stress {
        let textured = TexturedMeshRef::new(&TEXTURED_VERTICES, &TEXTURED_INDICES, DEMO_IMAGE_JPEG);
        let _ = textured.emit(
            &camera,
            Transform3D::screen(panel.rect.x + 110, panel.rect.y + 192, 2),
            220,
            true,
            &mut list,
        );
    }
    list.push(DrawCommand::DrawText {
        pos: Point::new(panel.rect.x + 22, panel.rect.y + 20),
        depth: fixed_from_i32(4),
        text: "FHRE",
        font: fhre::FontId(0),
        color: Color::WHITE,
        scale: 4,
    });
    list.push(DrawCommand::DrawText {
        pos: Point::new(panel.rect.x + 22, panel.rect.y + 58),
        depth: fixed_from_i32(4),
        text: FHRE_VERSION,
        font: fhre::FontId(0),
        color: Color::rgba(198, 222, 248, 220),
        scale: 1,
    });
    list.push(DrawCommand::DrawImage {
        rect: Rect::new(panel.rect.right() - 94, panel.rect.y + 138, 54, 38),
        depth: fixed_from_i32(8),
        image: IMAGE_SWATCH,
        opacity: 228,
    });
    list.push(DrawCommand::DrawImageFit {
        rect: Rect::new(panel.rect.right() - 94, panel.rect.y + 184, 54, 38),
        depth: fixed_from_i32(8),
        image: DEMO_IMAGE_JPEG,
        opacity: 230,
        fit: ImageFit::Contain,
    });
    list.push(DrawCommand::DrawSvgIcon {
        rect: Rect::new(panel.rect.right() - 142, panel.rect.y + 132, 38, 38),
        depth: fixed_from_i32(9),
        icon: SvgId(5),
        color: Color::rgba(238, 250, 255, 232),
        opacity: 232,
    });

    if state.mode == StressMode::Rect || state.mode == StressMode::Stress {
        let y = panel.rect.y + 132;
        list.push(DrawCommand::FillStyled {
            rect: Rect::new(panel.rect.x + 22, y, 74, 46),
            depth: fixed_from_i32(10),
            style: FillStyle {
                color: Color::rgba(82, 214, 232, 220),
                gradient: GradientStyle::Horizontal {
                    start: Color::rgba(82, 214, 232, 230),
                    end: Color::rgba(80, 132, 248, 230),
                },
                radius: 12,
                blend: BlendMode::Normal,
            },
        });
        list.push(DrawCommand::FillStyled {
            rect: Rect::new(panel.rect.x + 108, y, 74, 46),
            depth: fixed_from_i32(10),
            style: FillStyle {
                color: Color::rgba(255, 255, 255, 180),
                gradient: GradientStyle::Radial {
                    center: Point::new(panel.rect.x + 145, y + 23),
                    radius: 44,
                    inner: Color::rgba(255, 255, 255, 220),
                    outer: Color::rgba(0, 150, 136, 120),
                },
                radius: 20,
                blend: BlendMode::Normal,
            },
        });
        list.push(DrawCommand::FillStyled {
            rect: Rect::new(panel.rect.x + 194, y, 74, 46),
            depth: fixed_from_i32(10),
            style: FillStyle {
                color: Color::rgba(80, 132, 248, 160),
                gradient: GradientStyle::Conical {
                    center: Point::new(panel.rect.x + 231, y + 23),
                    start: Color::rgba(80, 132, 248, 220),
                    end: Color::rgba(255, 255, 255, 120),
                },
                radius: 8,
                blend: BlendMode::Additive,
            },
        });
        list.push(DrawCommand::DrawText {
            pos: Point::new(panel.rect.x + 22, y + 58),
            depth: fixed_from_i32(10),
            text: "FILL: H/RADIAL/CONICAL + BLEND",
            font: FontId(0),
            color: Color::rgba(238, 250, 255, 210),
            scale: 1,
        });
    }

    if state.mode == StressMode::Border || state.mode == StressMode::Stress {
        let area = Rect::new(panel.rect.x + 22, panel.rect.y + 132, 92, 58);
        list.push(DrawCommand::DrawBorder {
            rect: area,
            depth: fixed_from_i32(11),
            style: BorderStyle {
                color: Color::rgba(82, 214, 232, 230),
                width: 4,
                radius: 16,
                sides: BorderSides::FULL,
                align: BorderAlign::Inside,
            },
        });
        list.push(DrawCommand::DrawBorder {
            rect: Rect::new(panel.rect.x + 136, panel.rect.y + 132, 92, 58),
            depth: fixed_from_i32(11),
            style: BorderStyle {
                color: Color::rgba(255, 255, 255, 210),
                width: 5,
                radius: 0,
                sides: BorderSides(
                    BorderSides::LEFT.0 | BorderSides::BOTTOM.0 | BorderSides::RIGHT.0,
                ),
                align: BorderAlign::Center,
            },
        });
        list.push(DrawCommand::StrokeStyledLine {
            from: Point::new(panel.rect.x + 22, panel.rect.y + 210),
            to: Point::new(panel.rect.x + 238, panel.rect.y + 226),
            depth: fixed_from_i32(11),
            style: LineStyle {
                color: Color::rgba(80, 132, 248, 230),
                width: 4,
                dash_width: 12,
                dash_gap: 7,
                round_start: true,
                round_end: true,
                cap_start: fhre::LineCap::Round,
                cap_end: fhre::LineCap::Round,
                join: fhre::LineJoin::Round,
                blend: BlendMode::Normal,
            },
        });
    }

    if state.mode == StressMode::Shadow || state.mode == StressMode::Stress {
        let card = Rect::new(panel.rect.x + 52, panel.rect.y + 140, 154, 76);
        list.push(DrawCommand::DrawShadow {
            rect: card,
            depth: fixed_from_i32(9),
            style: ShadowStyle {
                color: Color::rgba(0, 0, 0, 180),
                width: 18,
                spread: 2,
                offset: Point::new(8, 10),
                radius: 18,
            },
        });
        list.push(DrawCommand::FillStyled {
            rect: card,
            depth: fixed_from_i32(10),
            style: FillStyle::rounded(Color::rgba(255, 255, 255, 210), 18),
        });
        list.push(DrawCommand::DrawText {
            pos: Point::new(card.x + 18, card.y + 28),
            depth: fixed_from_i32(11),
            text: "BOX SHADOW",
            font: FontId(0),
            color: Color::rgba(18, 48, 64, 230),
            scale: 2,
        });
    }

    if state.mode == StressMode::Arc || state.mode == StressMode::Stress {
        let c = Point::new(panel.rect.x + 92, panel.rect.y + 174);
        list.push(DrawCommand::DrawArc {
            center: c,
            depth: fixed_from_i32(10),
            radius: 42,
            style: ArcStyle {
                color: Color::rgba(82, 214, 232, 235),
                width: 7,
                start_angle: 15,
                end_angle: 270,
                rounded: true,
            },
        });
        list.push(DrawCommand::DrawArc {
            center: Point::new(c.x + 116, c.y),
            depth: fixed_from_i32(10),
            radius: 36,
            style: ArcStyle {
                color: Color::rgba(80, 132, 248, 230),
                width: 5,
                start_angle: 210,
                end_angle: 110,
                rounded: false,
            },
        });
    }

    if state.mode == StressMode::Triangle || state.mode == StressMode::Stress {
        let y = panel.rect.y + 140;
        list.push(DrawCommand::DrawTriangle {
            p0: Point::new(panel.rect.x + 32, y + 72),
            p1: Point::new(panel.rect.x + 94, y + 6),
            p2: Point::new(panel.rect.x + 154, y + 76),
            depth: fhre::DepthSpan {
                a: fixed_from_i32(10),
                b: fixed_from_i32(12),
                c: fixed_from_i32(11),
            },
            color: Color::rgba(80, 132, 248, 190),
        });
        list.push(DrawCommand::DrawGradientTriangle {
            p0: Point::new(panel.rect.x + 148, y + 74),
            p1: Point::new(panel.rect.x + 222, y + 10),
            p2: Point::new(panel.rect.x + 274, y + 82),
            depth: fhre::DepthSpan {
                a: fixed_from_i32(12),
                b: fixed_from_i32(13),
                c: fixed_from_i32(12),
            },
            style: TriangleStyle {
                colors: [
                    Color::rgba(82, 214, 232, 230),
                    Color::rgba(255, 255, 255, 210),
                    Color::rgba(0, 150, 136, 220),
                ],
                blend: BlendMode::Normal,
            },
        });
        list.push(DrawCommand::DrawText {
            pos: Point::new(panel.rect.x + 22, y + 98),
            depth: fixed_from_i32(14),
            text: "TRIANGLE: FLAT + VERTEX COLOR",
            font: FontId(0),
            color: Color::rgba(238, 250, 255, 210),
            scale: 1,
        });
    }

    if state.mode == StressMode::Layer || state.mode == StressMode::Stress {
        let layer = Rect::new(panel.rect.x + 34, panel.rect.y + 132, 186, 96);
        list.push(DrawCommand::BeginLayer {
            rect: layer,
            depth: fixed_from_i32(12),
            spec: LayerSpec {
                opacity: 210,
                recolor: Some(Color::rgba(82, 214, 232, 54)),
                blur_radius: 6,
                mask: Some(MaskSpec::rounded(layer, 20)),
            },
        });
        list.push(DrawCommand::FillStyled {
            rect: Rect::new(layer.x + 10, layer.y + 10, layer.w - 20, layer.h - 20),
            depth: fixed_from_i32(13),
            style: FillStyle {
                color: Color::rgba(12, 34, 48, 190),
                gradient: GradientStyle::Linear {
                    start: Point::new(layer.x, layer.y),
                    end: Point::new(layer.right(), layer.bottom()),
                    start_color: Color::rgba(80, 132, 248, 210),
                    end_color: Color::rgba(0, 150, 136, 210),
                },
                radius: 16,
                blend: BlendMode::Normal,
            },
        });
        list.push(DrawCommand::DrawImageFit {
            rect: Rect::new(layer.x + 128, layer.y + 14, 40, 40),
            depth: fixed_from_i32(14),
            image: IMAGE_SWATCH,
            fit: ImageFit::Cover,
            opacity: 190,
        });
        list.push(DrawCommand::DrawText {
            pos: Point::new(layer.x + 18, layer.y + 38),
            depth: fixed_from_i32(15),
            text: "REAL LAYER",
            font: FontId(0),
            color: Color::rgba(255, 255, 255, 220),
            scale: 2,
        });
        list.push(DrawCommand::EndLayer);
    }

    if state.mode == StressMode::Mask || state.mode == StressMode::Stress {
        let mask = Rect::new(panel.rect.x + 42, panel.rect.y + 132, 160, 86);
        let rounded = Rect::new(
            mask.x + 6,
            mask.y + 6,
            mask.w.saturating_sub(12),
            mask.h.saturating_sub(12),
        );
        list.push(DrawCommand::PushMask {
            depth: fixed_from_i32(10),
            spec: MaskSpec::rounded(rounded, 22),
        });
        list.push(DrawCommand::FillStyled {
            rect: mask,
            depth: fixed_from_i32(11),
            style: FillStyle {
                color: Color::rgba(80, 132, 248, 180),
                gradient: GradientStyle::Linear {
                    start: Point::new(mask.x, mask.y),
                    end: Point::new(mask.right(), mask.bottom()),
                    start_color: Color::rgba(80, 132, 248, 220),
                    end_color: Color::rgba(0, 150, 136, 210),
                },
                radius: 22,
                blend: BlendMode::Normal,
            },
        });
        list.push(DrawCommand::PushBitmapMask {
            rect: Rect::new(mask.x + 18, mask.y + 14, 70, 58),
            depth: fixed_from_i32(12),
            image: IMAGE_MASK_DOT,
            inverted: false,
            opacity: 210,
        });
        list.push(DrawCommand::FillStyled {
            rect: Rect::new(mask.x + 12, mask.y + 12, 88, 64),
            depth: fixed_from_i32(13),
            style: FillStyle {
                color: Color::rgba(255, 255, 255, 230),
                gradient: GradientStyle::None,
                radius: 14,
                blend: BlendMode::Normal,
            },
        });
        list.push(DrawCommand::PopMask);
        list.push(DrawCommand::DrawText {
            pos: Point::new(mask.x + 18, mask.y + 34),
            depth: fixed_from_i32(14),
            text: "MASK",
            font: FontId(0),
            color: Color::rgba(255, 255, 255, 235),
            scale: 2,
        });
        list.push(DrawCommand::PopMask);
        let blur_layer = Rect::new(mask.x + 42, mask.y + 24, 74, 38);
        list.push(DrawCommand::BeginLayer {
            rect: blur_layer,
            depth: fixed_from_i32(15),
            spec: LayerSpec {
                opacity: 180,
                recolor: Some(Color::rgba(82, 214, 232, 38)),
                blur_radius: 14,
                mask: Some(MaskSpec::rounded(blur_layer, 14)),
            },
        });
        list.push(DrawCommand::FillStyled {
            rect: Rect::new(
                blur_layer.x + 10,
                blur_layer.y + 8,
                blur_layer.w - 20,
                blur_layer.h - 16,
            ),
            depth: fixed_from_i32(16),
            style: FillStyle::rounded(Color::rgba(255, 255, 255, 168), 12),
        });
        list.push(DrawCommand::EndLayer);
    }

    if state.mode == StressMode::Image || state.mode == StressMode::Stress {
        let ids = [
            DEMO_IMAGE_PNG,
            DEMO_IMAGE_FRAW,
            DEMO_IMAGE_JPEG,
            DEMO_IMAGE_FILTERS_RGBA,
            DEMO_IMAGE_PALETTE_TRNS,
            DEMO_IMAGE_GRAY_ALPHA16,
            DEMO_IMAGE_GRAY_TRNS,
            DEMO_IMAGE_LVGL_16BIT,
            DEMO_IMAGE_LVGL_PALETTE,
            DEMO_IMAGE_LVGL_CMYK,
            DEMO_IMAGE_LVGL_EXIF90,
            IMAGE_SWATCH,
        ];
        for i in 0..24 {
            let image = ids[(i as usize) % ids.len()];
            let rect = Rect::new(
                panel.rect.x + 22 + (i % 6) * 34,
                panel.rect.y + 132 + (i / 6) * 28,
                28,
                22,
            );
            if i & 1 == 0 {
                list.push(DrawCommand::DrawImageFit {
                    rect,
                    depth: fixed_from_i32(10),
                    image,
                    opacity: 220,
                    fit: ImageFit::Contain,
                });
            } else {
                list.push(DrawCommand::DrawImageTint {
                    rect,
                    depth: fixed_from_i32(10),
                    image,
                    opacity: 215,
                    fit: ImageFit::Contain,
                    tint: Color::rgba(82, 214, 232, 220),
                });
            }
        }
        list.push(DrawCommand::DrawImageStyled {
            rect: Rect::new(panel.rect.x + 22, panel.rect.y + 252, 118, 42),
            depth: fixed_from_i32(10),
            image: DEMO_IMAGE_FRAW,
            style: ImageDrawStyle {
                opacity: 190,
                fit: ImageFit::Stretch,
                tint: Some(Color::rgba(82, 214, 232, 210)),
                clip_radius: 10,
                tile: true,
                blend: BlendMode::Multiply,
            },
        });
    }

    if state.mode == StressMode::Text || state.mode == StressMode::Stress {
        list.push(DrawCommand::DrawText {
            pos: Point::new(panel.rect.x + 22, panel.rect.y + 134),
            depth: fixed_from_i32(10),
            text: DEMO_TEXT,
            font: DEMO_TTF_FONT,
            color: Color::rgba(255, 255, 255, 235),
            scale: DEMO_TTF_SIZE,
        });
        list.push(DrawCommand::DrawText {
            pos: Point::new(panel.rect.x + 22, panel.rect.y + 162),
            depth: fixed_from_i32(10),
            text: "debug 5x7 fallback",
            font: fhre::FontId(0),
            color: Color::rgba(82, 214, 232, 220),
            scale: 1,
        });
        for i in 0..4 {
            list.push(DrawCommand::DrawText {
                pos: Point::new(panel.rect.x + 22, panel.rect.y + 184 + i * 10),
                depth: fixed_from_i32(10),
                text: "TEXT GLYPH CACHE A8 TTF SUBSET",
                font: fhre::FontId(0),
                color: Color::rgba(238, 250, 255, 185),
                scale: 1,
            });
        }
        list.push(DrawCommand::DrawLabel {
            rect: Rect::new(panel.rect.x + 22, panel.rect.y + 230, 220, 52),
            depth: fixed_from_i32(10),
            text: "wrap align selection underline strikethrough spans here",
            style: TextStyle {
                font: DEMO_TTF_FONT,
                color: Color::rgba(255, 255, 255, 220),
                scale: 14,
                align: TextAlign::Center,
                letter_spacing: 0,
                line_spacing: 2,
                kerning: false,
                decor: TextDecor(TextDecor::UNDERLINE.0 | TextDecor::STRIKETHROUGH.0),
                selection: Some((
                    11,
                    20,
                    Color::rgba(8, 16, 24, 240),
                    Color::rgba(82, 214, 232, 150),
                )),
            },
        });
        list.push(DrawCommand::DrawLabel {
            rect: Rect::new(panel.rect.x + 22, panel.rect.y + 292, 220, 24),
            depth: fixed_from_i32(10),
            text: "KERN OFF: AVATAR To WA Yo",
            style: TextStyle {
                font: DEMO_TTF_FONT,
                color: Color::rgba(238, 250, 255, 210),
                scale: 13,
                align: TextAlign::Left,
                letter_spacing: 0,
                line_spacing: 0,
                kerning: false,
                decor: TextDecor::NONE,
                selection: None,
            },
        });
        list.push(DrawCommand::DrawLabel {
            rect: Rect::new(panel.rect.x + 22, panel.rect.y + 320, 220, 24),
            depth: fixed_from_i32(10),
            text: "KERN ON : AVATAR To WA Yo",
            style: TextStyle {
                font: DEMO_TTF_FONT,
                color: Color::rgba(82, 214, 232, 230),
                scale: 13,
                align: TextAlign::Left,
                letter_spacing: 0,
                line_spacing: 0,
                kerning: true,
                decor: TextDecor::NONE,
                selection: None,
            },
        });
        list.push(DrawCommand::DrawLabel {
            rect: Rect::new(panel.rect.x + 22, panel.rect.y + 348, 220, 24),
            depth: fixed_from_i32(10),
            text: "MISSING: \u{E000}",
            style: TextStyle {
                font: DEMO_TTF_FONT,
                color: Color::rgba(255, 170, 80, 230),
                scale: 13,
                align: TextAlign::Left,
                letter_spacing: 0,
                line_spacing: 0,
                kerning: false,
                decor: TextDecor::NONE,
                selection: None,
            },
        });
    }

    if state.mode == StressMode::Vector || state.mode == StressMode::Stress {
        for i in 0..16 {
            list.push(DrawCommand::DrawSvgIcon {
                rect: Rect::new(
                    panel.rect.x + 22 + (i % 8) * 24,
                    panel.rect.y + 210 + (i / 8) * 24,
                    18,
                    18,
                ),
                depth: fixed_from_i32(10),
                icon: SvgId((i % 8) as u16),
                color: Color::rgba(238, 250, 255, 190),
                opacity: 210,
            });
        }
        let colors = [
            Color::rgba(82, 214, 232, 230),
            Color::rgba(80, 132, 248, 230),
            Color::rgba(255, 255, 255, 220),
            Color::rgba(0, 150, 136, 230),
            Color::rgba(194, 92, 230, 225),
        ];
        for i in 0..DEMO_SVG_IDS.len() {
            list.push(DrawCommand::DrawSvgDocument {
                rect: Rect::new(
                    panel.rect.x + 22 + (i as i32 % 5) * 45,
                    panel.rect.y + 132 + (i as i32 / 5) * 44,
                    38,
                    38,
                ),
                depth: fixed_from_i32(10),
                document: DEMO_SVG_IDS[i],
                color: colors[i % colors.len()],
                opacity: colors[i % colors.len()].a,
            });
        }
        list.push(DrawCommand::DrawSvgDocument {
            rect: Rect::new(panel.rect.x + 22 + 5 * 45, panel.rect.y + 132, 38, 38),
            depth: fixed_from_i32(10),
            document: SvgId(999),
            color: Color::rgba(255, 170, 80, 230),
            opacity: 230,
        });
        list.push(DrawCommand::DrawText {
            pos: Point::new(panel.rect.x + 22, panel.rect.y + 182),
            depth: fixed_from_i32(10),
            text: "SVG FILES: path/shape/style/use/tiger + fallback",
            font: FontId(0),
            color: Color::rgba(238, 250, 255, 210),
            scale: 1,
        });
    }

    let base_y = panel.rect.y + panel.rect.h as i32 - 64;
    let mut current_bubble_bounds = [None; 8];
    let mut bubble_index = 0usize;
    state.bubbles.for_each(|entity: Entity, bubble| {
        if let Some(transform) = state.transforms.get(entity) {
            let x = fixed_to_i32(transform.position.x);
            let y = fixed_to_i32(transform.position.y);
            let pulse = ((frame as i32 + bubble.phase as i32 / 2) & 7) as u16;
            let radius = bubble.radius.saturating_add(pulse);
            list.push(DrawCommand::FillCircle {
                center: Point::new(x, y),
                depth: transform.position.z,
                radius,
                color: bubble.primary,
            });
            list.push(DrawCommand::FillCircle {
                center: Point::new(x + 13, base_y + 13),
                depth: transform.position.z + fixed_from_i32(1),
                radius: 9,
                color: bubble.secondary,
            });
            if bubble_index < current_bubble_bounds.len() {
                let large = Rect::from_edges(
                    x - radius as i32 - 2,
                    y - radius as i32 - 2,
                    x + radius as i32 + 3,
                    y + radius as i32 + 3,
                );
                let small = Rect::from_edges(x + 2, base_y + 4, x + 25, base_y + 27);
                current_bubble_bounds[bubble_index] = Some(large.union(small));
                bubble_index += 1;
            }
        }
    });

    list.push(DrawCommand::FillRoundRect {
        rect: Rect::new(
            panel.rect.x + 22,
            panel.rect.y + 92,
            panel.rect.w.saturating_sub(44),
            34,
        ),
        depth: fixed_from_i32(6),
        radius: 12,
        color: Color::rgba(255, 255, 255, 210),
    });
    let progress_rect = Rect::new(
        panel.rect.x + 34,
        panel.rect.y + 105,
        ((panel.rect.w as u32 * ((frame % 180) + 30) / 240) as u16)
            .min(panel.rect.w.saturating_sub(68)),
        8,
    );
    list.push(DrawCommand::FillRoundRect {
        rect: progress_rect,
        depth: fixed_from_i32(7),
        radius: 4,
        color: Color::rgb(80, 132, 248),
    });
    list.push(DrawCommand::DrawText {
        pos: Point::new(panel.rect.x + 22, panel.rect.bottom() - 32),
        depth: fixed_from_i32(6),
        text: "ECS ENTITIES  3D WORLD  Z0 CANVAS",
        font: fhre::FontId(0),
        color: Color::rgba(230, 238, 250, 230),
        scale: 1,
    });
    list.push(DrawCommand::DrawText {
        pos: Point::new(panel.rect.x + 22, panel.rect.bottom() - 18),
        depth: fixed_from_i32(6),
        text: "A8 GLYPH  SVG PATH  IMAGE BLIT",
        font: fhre::FontId(0),
        color: Color::rgba(230, 238, 250, 210),
        scale: 1,
    });

    list.sort_by_depth();
    if state.mode == StressMode::Stress {
        state.dirty.force_full();
    }
    state
        .dirty
        .mark_transition(state.previous_progress, progress_rect);
    state.previous_progress = Some(progress_rect);
    for (index, bounds) in current_bubble_bounds.iter().copied().enumerate() {
        if let Some(bounds) = bounds {
            state
                .dirty
                .mark_transition(state.previous_bubble_bounds[index], bounds);
            state.previous_bubble_bounds[index] = Some(bounds);
        }
    }
    let stats_rect = Rect::new(
        panel.rect.x + 18,
        panel.rect.y + 80,
        panel.rect.w.saturating_sub(118),
        panel.rect.h.saturating_sub(82),
    );
    state.dirty.mark_current(stats_rect);
    if state.dirty.region().is_empty() {
        state.dirty.finish_frame();
        return false;
    }
    let dirty_copy_bytes = fb.prepare_dirty_frame(state.dirty.region());
    let mut stats = RenderStats::new();
    list.execute_dirty_tracked_on(&mut fb.surface, state.dirty.region(), &mut stats);
    let cache_stats = demo_image_cache_stats();
    let svg_doc_cache_stats = demo_svg_document_cache_stats();
    let glyph_run_cache_stats = demo_glyph_run_cache_stats();
    let codec_stats = demo_codec_stats();
    let fixture_stats = demo_fixture_stats();
    stats.mark_dirty_copy_bytes(dirty_copy_bytes);
    stats.mark_frame_stats(frame_stats);
    stats.mark_image_cache(cache_stats);
    stats.mark_svg_document_cache(svg_doc_cache_stats);
    stats.mark_glyph_run_cache(glyph_run_cache_stats);
    stats.mark_codec_stats(codec_stats);
    apply_fixture_feature_stats(&mut stats, fixture_stats);
    let mut display_stats = state.last_render_stats;
    display_stats.mark_frame_stats(frame_stats);
    display_stats.mark_image_cache(cache_stats);
    display_stats.mark_svg_document_cache(svg_doc_cache_stats);
    display_stats.mark_glyph_run_cache(glyph_run_cache_stats);
    display_stats.mark_codec_stats(codec_stats);
    let mut hud: DrawList<208> = DrawList::new();
    push_stats_commands(
        &mut hud,
        panel.rect,
        display_stats,
        cache_stats,
        frame_stats,
        fixture_stats,
        state.mode,
    );
    let mut hud_stats = RenderStats::new();
    hud.execute_dirty_tracked_on(&mut fb.surface, state.dirty.region(), &mut hud_stats);
    stats.merge_draw_stats(hud_stats);
    state.last_render_stats = stats;
    state.dirty.finish_frame();
    true
}

fn push_stats_commands(
    list: &mut DrawList<208>,
    panel: Rect,
    stats: RenderStats,
    cache: ImageCacheStats,
    frame: FrameStats,
    fixtures: CodecFixtureStats,
    mode: StressMode,
) {
    let x = panel.x + 22;
    let y = panel.y + 82;
    let w = panel.w.saturating_sub(182).max(32);
    let bench = stats.benchmark_summary();
    push_text(list, x, y, mode.label(), Color::rgba(255, 255, 255, 245));
    push_text(
        list,
        x + 54,
        y,
        mode.benchmark_profile(),
        Color::rgba(238, 250, 255, 180),
    );
    push_bar_back(list, x, y + 14, w, 7);
    push_bar(
        list,
        x,
        y + 14,
        stats.commands_drawn.min(w as u32),
        7,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar_back(list, x, y + 27, w, 7);
    push_bar(
        list,
        x,
        y + 27,
        stats.effective_clip_changes.saturating_mul(8).min(w as u32),
        7,
        Color::rgba(80, 132, 248, 220),
    );
    push_text(
        list,
        x,
        y + 42,
        "CACHE LOAD/HIT/FAIL",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 55,
        cache.loads.saturating_mul(14).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 48,
        y + 55,
        cache.hits.saturating_mul(14).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 96,
        y + 55,
        cache
            .load_failures
            .saturating_add(cache.decode_failures)
            .saturating_mul(14)
            .min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_text(
        list,
        x,
        y + 66,
        "FRAME DRAW/PRESENT/LATE",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 79,
        frame.draw_us.saturating_div(100).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 58,
        y + 79,
        frame.present_us.saturating_div(100).min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 116,
        y + 79,
        frame.late_frames.saturating_mul(3).min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_text(
        list,
        x,
        y + 92,
        "DIRTY PASS/COPY/INPUT",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 105,
        stats.dirty_passes.saturating_mul(10).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 58,
        y + 105,
        stats.dirty_copy_bytes.saturating_div(2048).min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 116,
        y + 105,
        frame.input_events.saturating_mul(8).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_text(
        list,
        x,
        y + 118,
        "PRESENT PAN/COPY/SKIP",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 131,
        stats.pan_presents.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 58,
        y + 131,
        stats.copy_presents.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 116,
        y + 131,
        stats.skipped_presents.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_text(
        list,
        x,
        y + 144,
        "TASK F/B/L/A",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 157,
        stats.fill_commands.saturating_mul(8).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 157,
        stats.border_commands.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 157,
        stats.line_commands.saturating_mul(8).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 157,
        stats.arc_commands.saturating_mul(16).min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_text(
        list,
        x,
        y + 168,
        "TASK I/T/V/TRI",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 181,
        stats.image_commands.saturating_mul(10).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 181,
        stats.text_commands.saturating_mul(10).min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 181,
        stats.vector_commands.saturating_mul(10).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 181,
        stats.triangle_commands.saturating_mul(14).min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_text(
        list,
        x,
        y + 192,
        "LAYER/MASK/CODEC/FALL",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 205,
        stats.layer_commands.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 205,
        stats.mask_commands.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 205,
        stats.codec_fallbacks.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 205,
        stats
            .draw_task_fallbacks
            .saturating_add(stats.layer_alloc_failures)
            .saturating_add(stats.mask_stack_overflows)
            .saturating_mul(12)
            .min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_text(
        list,
        x,
        y + 216,
        "PATH SW/ACC/FB",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 229,
        stats.software_path_hits.min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 58,
        y + 229,
        stats.accelerated_path_hits.min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 116,
        y + 229,
        stats.fallback_path_hits.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_text(
        list,
        x,
        y + 240,
        "CODEC M/I/T/U/O",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 253,
        stats
            .codec_missing_resources
            .saturating_mul(12)
            .min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_bar(
        list,
        x + 36,
        y + 253,
        stats.codec_invalid.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_bar(
        list,
        x + 72,
        y + 253,
        stats.codec_truncated.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 108,
        y + 253,
        stats.codec_unsupported.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 144,
        y + 253,
        stats.codec_overflow.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(194, 92, 230, 220),
    );
    push_text(
        list,
        x,
        y + 264,
        "DISP F/I/T/V",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 277,
        task_dispatch_hits(stats, DrawTaskKind::Fill)
            .saturating_mul(8)
            .min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 277,
        task_dispatch_hits(stats, DrawTaskKind::Image)
            .saturating_mul(8)
            .min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 277,
        task_dispatch_hits(stats, DrawTaskKind::Label)
            .saturating_mul(8)
            .min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 277,
        task_dispatch_hits(stats, DrawTaskKind::Vector)
            .saturating_mul(8)
            .min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_text(
        list,
        x,
        y + 288,
        "DISP L/M/B/3D",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 301,
        task_dispatch_hits(stats, DrawTaskKind::Layer)
            .saturating_mul(8)
            .min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 301,
        task_dispatch_hits(stats, DrawTaskKind::MaskRect)
            .saturating_add(task_dispatch_hits(stats, DrawTaskKind::MaskBitmap))
            .saturating_mul(8)
            .min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 301,
        task_dispatch_hits(stats, DrawTaskKind::Blur)
            .saturating_mul(8)
            .min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 301,
        task_dispatch_hits(stats, DrawTaskKind::ThreeD)
            .saturating_add(task_dispatch_hits(stats, DrawTaskKind::Triangle))
            .saturating_mul(8)
            .min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_text(
        list,
        x,
        y + 312,
        top_dispatch_label(stats),
        Color::rgba(238, 250, 255, 210),
    );
    push_text(
        list,
        x + 82,
        y + 312,
        top_fallback_label(stats),
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 325,
        bench.top_dispatch_hits.saturating_mul(8).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 325,
        bench.top_fallback_hits.saturating_mul(18).min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 325,
        fixtures.passed.saturating_mul(5).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 325,
        fixtures.mismatches.saturating_mul(20).min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_text(
        list,
        x,
        y + 336,
        "V3 JPG/CFF/SH",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 349,
        stats
            .progressive_jpeg_decodes
            .saturating_mul(18)
            .min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 349,
        stats.cff_raster_glyphs.saturating_mul(18).min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 349,
        stats.opentype_shaping_runs.saturating_mul(18).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 349,
        stats
            .progressive_jpeg_scan_fallbacks
            .saturating_mul(18)
            .min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_text(
        list,
        x,
        y + 360,
        "SVG C/M/F/G",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 373,
        stats.svg_clip_paths.saturating_mul(16).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 373,
        stats.svg_masks.saturating_mul(16).min(w as u32),
        5,
        Color::rgba(80, 132, 248, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 373,
        stats.svg_filters.saturating_mul(16).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 373,
        stats.svg_gradients.saturating_mul(16).min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_text(
        list,
        x,
        y + 384,
        "V31 SVG/TXT",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 397,
        stats
            .svg_real_clip_paths
            .saturating_add(stats.svg_real_masks)
            .saturating_mul(16)
            .min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 397,
        stats
            .svg_filter_fallbacks
            .saturating_add(stats.svg_gradient_fallbacks)
            .saturating_mul(16)
            .min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 397,
        stats
            .opentype_gsub_hits
            .saturating_add(stats.opentype_gpos_hits)
            .saturating_mul(12)
            .min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 397,
        stats
            .opentype_kern_hits
            .saturating_add(stats.cff_fallback_glyphs)
            .saturating_mul(12)
            .min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_text(
        list,
        x,
        y + 408,
        "V38 SVG/TXT",
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 421,
        stats
            .svg_doc_cache_loads
            .saturating_add(stats.svg_doc_cache_hits)
            .saturating_mul(12)
            .min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 421,
        stats
            .svg_doc_cache_misses
            .saturating_add(stats.svg_doc_cache_fallbacks)
            .saturating_mul(12)
            .min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 421,
        stats
            .glyph_run_cache_hits
            .saturating_add(stats.glyph_id_draw_hits)
            .saturating_add(stats.text_shaped_layouts)
            .saturating_mul(12)
            .min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 421,
        stats
            .vector_mask_scratch_overflows
            .saturating_add(stats.svg_filter_budget_exceeded)
            .saturating_add(stats.svg_doc_cache_fallbacks)
            .saturating_add(stats.glyph_run_cache_overflows)
            .saturating_add(stats.text_layout_overflows)
            .saturating_add(stats.text_shaping_fallbacks)
            .saturating_add(stats.selection_fallbacks)
            .saturating_mul(12)
            .min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_text(
        list,
        x,
        y + 432,
        "V39 CHAIN/PIPE",
        Color::rgba(238, 250, 255, 210),
    );
    push_text(
        list,
        x + 86,
        y + 432,
        top_chain_label(stats),
        Color::rgba(238, 250, 255, 210),
    );
    push_bar(
        list,
        x,
        y + 445,
        stats.draw_chain_candidates.saturating_mul(6).min(w as u32),
        5,
        Color::rgba(82, 214, 232, 220),
    );
    push_bar(
        list,
        x + 46,
        y + 445,
        stats.draw_chain_submitted.saturating_mul(12).min(w as u32),
        5,
        Color::rgba(0, 150, 136, 220),
    );
    push_bar(
        list,
        x + 92,
        y + 445,
        stats
            .draw_chain_fallbacks
            .saturating_add(stats.draw_chain_overflows)
            .saturating_mul(6)
            .min(w as u32),
        5,
        Color::rgba(255, 96, 96, 220),
    );
    push_bar(
        list,
        x + 138,
        y + 445,
        stats
            .codec_pipeline_candidates
            .saturating_add(stats.codec_pipeline_fallbacks)
            .saturating_mul(4)
            .min(w as u32),
        5,
        Color::rgba(255, 170, 80, 220),
    );
}

fn task_dispatch_hits(stats: RenderStats, kind: DrawTaskKind) -> u32 {
    stats.dispatch_hits_for(kind)
}

fn top_dispatch_label(stats: RenderStats) -> &'static str {
    match stats.benchmark_summary().top_dispatch_task {
        Some(DrawTaskKind::Fill) => "TOP DP: FILL",
        Some(DrawTaskKind::Border) => "TOP DP: BORDER",
        Some(DrawTaskKind::BoxShadow) => "TOP DP: SHADOW",
        Some(DrawTaskKind::Letter) => "TOP DP: LETTER",
        Some(DrawTaskKind::Label) => "TOP DP: LABEL",
        Some(DrawTaskKind::Image) => "TOP DP: IMAGE",
        Some(DrawTaskKind::Layer) => "TOP DP: LAYER",
        Some(DrawTaskKind::Line) => "TOP DP: LINE",
        Some(DrawTaskKind::Arc) => "TOP DP: ARC",
        Some(DrawTaskKind::Triangle) => "TOP DP: TRI",
        Some(DrawTaskKind::MaskRect) => "TOP DP: MASK",
        Some(DrawTaskKind::MaskBitmap) => "TOP DP: BMASK",
        Some(DrawTaskKind::Blur) => "TOP DP: BLUR",
        Some(DrawTaskKind::Vector) => "TOP DP: VECTOR",
        Some(DrawTaskKind::ThreeD) => "TOP DP: 3D",
        None => "TOP DP: NONE",
    }
}

fn top_fallback_label(stats: RenderStats) -> &'static str {
    match stats.benchmark_summary().top_fallback_task {
        Some(DrawTaskKind::Fill) => "TOP FB: FILL",
        Some(DrawTaskKind::Border) => "TOP FB: BORDER",
        Some(DrawTaskKind::BoxShadow) => "TOP FB: SHADOW",
        Some(DrawTaskKind::Letter) => "TOP FB: LETTER",
        Some(DrawTaskKind::Label) => "TOP FB: LABEL",
        Some(DrawTaskKind::Image) => "TOP FB: IMAGE",
        Some(DrawTaskKind::Layer) => "TOP FB: LAYER",
        Some(DrawTaskKind::Line) => "TOP FB: LINE",
        Some(DrawTaskKind::Arc) => "TOP FB: ARC",
        Some(DrawTaskKind::Triangle) => "TOP FB: TRI",
        Some(DrawTaskKind::MaskRect) => "TOP FB: MASK",
        Some(DrawTaskKind::MaskBitmap) => "TOP FB: BMASK",
        Some(DrawTaskKind::Blur) => "TOP FB: BLUR",
        Some(DrawTaskKind::Vector) => "TOP FB: VECTOR",
        Some(DrawTaskKind::ThreeD) => "TOP FB: 3D",
        None => "TOP FB: NONE",
    }
}

fn top_chain_label(stats: RenderStats) -> &'static str {
    match stats.top_chain_task().map(|(kind, _)| kind) {
        Some(DrawTaskKind::Fill) => "TOP CH: FILL",
        Some(DrawTaskKind::Border) => "TOP CH: BORDER",
        Some(DrawTaskKind::BoxShadow) => "TOP CH: SHADOW",
        Some(DrawTaskKind::Letter) => "TOP CH: LETTER",
        Some(DrawTaskKind::Label) => "TOP CH: LABEL",
        Some(DrawTaskKind::Image) => "TOP CH: IMAGE",
        Some(DrawTaskKind::Layer) => "TOP CH: LAYER",
        Some(DrawTaskKind::Line) => "TOP CH: LINE",
        Some(DrawTaskKind::Arc) => "TOP CH: ARC",
        Some(DrawTaskKind::Triangle) => "TOP CH: TRI",
        Some(DrawTaskKind::MaskRect) => "TOP CH: MASK",
        Some(DrawTaskKind::MaskBitmap) => "TOP CH: BMASK",
        Some(DrawTaskKind::Blur) => "TOP CH: BLUR",
        Some(DrawTaskKind::Vector) => "TOP CH: VECTOR",
        Some(DrawTaskKind::ThreeD) => "TOP CH: 3D",
        None => "TOP CH: NONE",
    }
}

fn push_text<const N: usize>(
    list: &mut DrawList<N>,
    x: i32,
    y: i32,
    text: &'static str,
    color: Color,
) {
    list.push(DrawCommand::DrawText {
        pos: Point::new(x, y),
        depth: fixed_from_i32(40),
        text,
        font: FontId(0),
        color,
        scale: 1,
    });
}

fn push_bar_back<const N: usize>(list: &mut DrawList<N>, x: i32, y: i32, w: u16, h: u16) {
    push_bar(list, x, y, w as u32, h, Color::rgba(255, 255, 255, 48));
}

fn push_bar<const N: usize>(list: &mut DrawList<N>, x: i32, y: i32, w: u32, h: u16, color: Color) {
    let width = w.min(u16::MAX as u32) as u16;
    if width == 0 || h == 0 {
        return;
    }
    list.push(DrawCommand::FillRoundRect {
        rect: Rect::new(x, y, width, h),
        depth: fixed_from_i32(39),
        radius: (h / 2).max(1),
        color,
    });
}
