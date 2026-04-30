#![no_std]

extern crate alloc;

mod animation;
mod backend;
mod color;
mod dirty;
mod draw;
mod ecs;
mod event;
mod geom;
mod glyph;
mod image;
mod input;
mod jpeg;
mod math;
mod platform;
pub mod prelude;
mod png;
mod raster;
mod runtime;
mod schedule;
mod scene;
mod surface;
mod surface_icons;
mod surface_image;
mod surface_layer;
mod surface_pixels;
mod surface_primitives;
mod surface_text;
mod svg;
mod text;
mod ttf;

pub use animation::{Easing, Tween};
pub use backend::{
    BackendCapabilities, CodecErrorKind, CodecStats, DrawBackendDispatch, DrawFeatureFlags,
    DrawPathKind, DrawTaskCounters, DrawTaskKind, RenderBackend, RenderBenchmarkSummary,
    RenderStats,
};
pub use color::{Color, PixelFormat};
pub use dirty::{DirtyRegion, DirtyTracker};
pub use draw::{
    ArcStyle, BlendMode, BorderAlign, BorderSides, BorderStyle, DepthSpan, DrawCommand, DrawList,
    FillStyle, FontId, GradientStyle, ImageDrawStyle, ImageFit, ImageId, LayerBudget, LayerSpec,
    LineCap, LineJoin, LineStyle, MaskKind, MaskSpec, MaskStack, ShadowStyle, SvgId, TexCoord,
    TextAlign, TextDecor, TextStyle, TriangleStyle,
};
pub use ecs::{ComponentEntry, ComponentStorage, Entity, EntityWorld, ResourceSlot};
pub use event::EventQueue;
pub use geom::{Point, Rect, Size};
pub use glyph::{
    glyph_5x7_a8, GlyphA8, GlyphBitmap, GlyphCache, GlyphCacheEntry, GlyphIdResolver, GlyphResolver,
    GlyphRunCache, GlyphRunCacheStats, GlyphRunResolver, GlyphView, KerningResolver,
    GLYPH_RUN_RESOLVER_CAPACITY,
};
pub use image::{
    builtin_image, decode_fraw, decode_fraw_result, FrawDecoder, FrawError, FrawInfo, ImageCache,
    ImageCacheStats, ImageDecodeErrorKind, ImageFormat, ImageResolver, ImageView, ResourceLoader,
    IMAGE_MASK_DOT, IMAGE_SWATCH,
};
pub use input::{
    GestureConfig, GestureDirection, GestureEvent, GestureKind, GestureRecognizer, InputEvent,
    InputQueue, KeyCode, KeyEvent, PointerEvent, PointerId, PointerPhase,
};
pub use jpeg::{decode_jpeg, JpegDecodeOptions, JpegDecoder, JpegError, JpegInfo};
pub use math::{fixed_div, fixed_from_i32, fixed_lerp, fixed_mul, fixed_to_i32, Fixed16, FIXED_ONE};
pub use platform::{FramebufferBackend, InputSource};
pub use png::{decode_png, decode_png_result, DecodedImage, PngDecoder, PngError, PngInfo};
pub use runtime::{
    FhreRuntime, FrameClock, FramePolicy, FrameStats, GameRuntime, PresentStats, RenderContext, Time,
};
pub use schedule::{Schedule, SystemFn};
pub use scene::{
    Camera, MeshDrawOptions, MeshRef, ProjectedPoint, ProjectedRect, Projection, RenderNode,
    Rotation, TexturedMeshRef, TexturedVertex3D, Transform3D, Vec3, Vertex3D,
};
pub use surface::Surface;
pub use surface_layer::LayerScratch;
pub use svg::{
    parse_svg_document, parse_svg_path, DefaultSvgDocument, SvgCache, SvgDocument,
    SvgDocumentView, SvgFillRule, SvgFilterEffect, SvgPaint, SvgPath, SvgPathCommand,
    SvgPathNode, SvgRasterOptions, SvgResolver, SvgStrokeCap, SvgStrokeJoin, SvgTransform,
    SvgUnsupportedFeature, MaskRasterOptions, VectorMaskScratch, SVG_DOCUMENT_COMMANDS,
    SVG_DOCUMENT_PATHS,
};
pub use text::{TextLayout, TextLayoutLine, TextLayoutOptions};
pub use ttf::{
    FontFace, FontFaceKind, FontInfo, GlyphRasterOptions, GlyphRun, GlyphRunItem,
    OpenTypeLayout, RasterGlyph, ShapeOptions, TtfDecoder, TtfError, GLYPH_RUN_FLAG_GPOS,
    GLYPH_RUN_FLAG_GSUB, GLYPH_RUN_FLAG_KERN,
};

pub const FHRE_VERSION: &str = "fhre-rust 0.1";
