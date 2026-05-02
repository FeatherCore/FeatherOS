use crate::{
    backend::{
        CodecAcceleratorCapabilities, CodecErrorKind, CodecPipelinePlan, CodecPipelineStats,
        CodecStageKind, CodecStagePlan, DEFAULT_CODEC_PIPELINE_STAGES,
    },
    image::ResourceLoader,
};
use crate::{Color, Point, Rect, SvgId};
use alloc::vec::Vec;

pub const SVG_DOCUMENT_PATHS: usize = 12;
pub const SVG_DOCUMENT_COMMANDS: usize = 256;
pub const VECTOR_MASK_SCRATCH_DEFAULT_BYTES: usize = 64 * 1024;

pub type DefaultSvgDocument = SvgDocument<SVG_DOCUMENT_PATHS, SVG_DOCUMENT_COMMANDS>;
pub type SvgResolver = fn(SvgId) -> Option<SvgDocumentView>;
const SVG_GROUP_STACK: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgPaint {
    None,
    CurrentColor,
    Color(Color),
    LinearGradient(Color, Color),
    RadialGradient(Color, Color),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgFillRule {
    NonZero,
    EvenOdd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgStrokeCap {
    Butt,
    Square,
    Round,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgStrokeJoin {
    Miter,
    Bevel,
    Round,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgUnsupportedFeature {
    ClipPath,
    Mask,
    Filter,
    GradientSpread,
    CssSelector,
    ClipPathComplex,
    MaskComplex,
    FilterBudgetExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgFilterEffect {
    None,
    Blur(u16),
    DropShadow {
        dx: i16,
        dy: i16,
        radius: u16,
        color: Color,
    },
}

impl SvgFilterEffect {
    pub const fn is_none(&self) -> bool {
        matches!(self, SvgFilterEffect::None)
    }

    pub const fn is_blur(&self) -> bool {
        matches!(self, SvgFilterEffect::Blur(_))
    }

    pub const fn is_shadow(&self) -> bool {
        matches!(self, SvgFilterEffect::DropShadow { .. })
    }

    pub const fn effective_radius(&self, max_radius: u16) -> u16 {
        match self {
            SvgFilterEffect::Blur(r) => if *r > max_radius { max_radius } else { *r },
            SvgFilterEffect::DropShadow { radius, .. } => {
                if *radius > max_radius { max_radius } else { *radius }
            }
            SvgFilterEffect::None => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgRasterOptions {
    pub viewport: Rect,
    pub color: Color,
    pub stroke_width: u16,
    pub opacity: u8,
    pub layer_budget_bytes: usize,
    pub filter_max_radius: u16,
    pub clip_mask_path_capacity: usize,
    pub mask_scratch_bytes: usize,
}

impl SvgRasterOptions {
    pub const fn new(viewport: Rect, color: Color) -> Self {
        Self {
            viewport,
            color,
            stroke_width: 1,
            opacity: 255,
            layer_budget_bytes: 64 * 1024,
            filter_max_radius: 16,
            clip_mask_path_capacity: 64,
            mask_scratch_bytes: VECTOR_MASK_SCRATCH_DEFAULT_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaskRasterOptions {
    pub max_bytes: usize,
    pub opacity: u8,
    pub invert: bool,
}

impl MaskRasterOptions {
    pub const DEFAULT: Self = Self {
        max_bytes: VECTOR_MASK_SCRATCH_DEFAULT_BYTES,
        opacity: 255,
        invert: false,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VectorMaskScratch {
    pub rect: Rect,
    pub width: u16,
    pub height: u16,
    pub used_bytes: usize,
    pub overflowed: bool,
}

impl VectorMaskScratch {
    pub const fn empty() -> Self {
        Self {
            rect: Rect::new(0, 0, 0, 0),
            width: 0,
            height: 0,
            used_bytes: 0,
            overflowed: false,
        }
    }

    pub fn try_for_rect(rect: Rect, options: MaskRasterOptions) -> Self {
        let used_bytes = rect.w as usize * rect.h as usize;
        Self {
            rect,
            width: rect.w,
            height: rect.h,
            used_bytes,
            overflowed: used_bytes > options.max_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgTransform {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
    pub e: i32,
    pub f: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SvgGroupState {
    fill: SvgPaint,
    stroke: SvgPaint,
    opacity: u8,
    fill_rule: SvgFillRule,
    stroke_cap: SvgStrokeCap,
    stroke_join: SvgStrokeJoin,
    transform: SvgTransform,
}

impl SvgGroupState {
    const fn root() -> Self {
        Self {
            fill: SvgPaint::CurrentColor,
            stroke: SvgPaint::None,
            opacity: 255,
            fill_rule: SvgFillRule::NonZero,
            stroke_cap: SvgStrokeCap::Butt,
            stroke_join: SvgStrokeJoin::Miter,
            transform: SvgTransform::IDENTITY,
        }
    }

    fn inherit(self, data: &str, tag: &str) -> Self {
        Self {
            fill: parse_document_paint(data, tag_value(data, tag, "fill")).unwrap_or(self.fill),
            stroke: parse_document_paint(data, tag_value(data, tag, "stroke"))
                .unwrap_or(self.stroke),
            opacity: mul_opacity(
                self.opacity,
                tag_value(data, tag, "opacity")
                    .and_then(parse_opacity)
                    .unwrap_or(255),
            ),
            fill_rule: parse_fill_rule(tag_value(data, tag, "fill-rule")).unwrap_or(self.fill_rule),
            stroke_cap: parse_stroke_cap(tag_value(data, tag, "stroke-linecap"))
                .unwrap_or(self.stroke_cap),
            stroke_join: parse_stroke_join(tag_value(data, tag, "stroke-linejoin"))
                .unwrap_or(self.stroke_join),
            transform: self.transform.then(
                parse_transform(parse_attr(tag, "transform")).unwrap_or(SvgTransform::IDENTITY),
            ),
        }
    }
}

impl SvgTransform {
    pub const IDENTITY: Self = Self {
        a: 1024,
        b: 0,
        c: 0,
        d: 1024,
        e: 0,
        f: 0,
    };

    pub fn apply(self, point: Point) -> Point {
        Point::new(
            ((self.a as i64 * point.x as i64 + self.c as i64 * point.y as i64) / 1024) as i32
                + self.e,
            ((self.b as i64 * point.x as i64 + self.d as i64 * point.y as i64) / 1024) as i32
                + self.f,
        )
    }

    pub fn then(self, next: SvgTransform) -> SvgTransform {
        SvgTransform {
            a: ((next.a as i64 * self.a as i64 + next.c as i64 * self.b as i64) / 1024) as i32,
            b: ((next.b as i64 * self.a as i64 + next.d as i64 * self.b as i64) / 1024) as i32,
            c: ((next.a as i64 * self.c as i64 + next.c as i64 * self.d as i64) / 1024) as i32,
            d: ((next.b as i64 * self.c as i64 + next.d as i64 * self.d as i64) / 1024) as i32,
            e: ((next.a as i64 * self.e as i64 + next.c as i64 * self.f as i64) / 1024) as i32
                + next.e,
            f: ((next.b as i64 * self.e as i64 + next.d as i64 * self.f as i64) / 1024) as i32
                + next.f,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgPathNode<const N: usize> {
    pub path: SvgPath<N>,
    pub fill: SvgPaint,
    pub stroke: SvgPaint,
    pub stroke_width: u16,
    pub stroke_cap: SvgStrokeCap,
    pub stroke_join: SvgStrokeJoin,
    pub opacity: u8,
    pub fill_rule: SvgFillRule,
    pub clip: Option<Rect>,
    pub clip_path: SvgPath<N>,
    pub has_clip_path: bool,
    pub mask: Option<Rect>,
    pub mask_path: SvgPath<N>,
    pub has_mask_path: bool,
    pub filter: SvgFilterEffect,
}

impl<const N: usize> SvgPathNode<N> {
    pub const EMPTY: Self = Self {
        path: SvgPath::new(Rect::new(0, 0, 0, 0)),
        fill: SvgPaint::None,
        stroke: SvgPaint::None,
        stroke_width: 1,
        stroke_cap: SvgStrokeCap::Butt,
        stroke_join: SvgStrokeJoin::Miter,
        opacity: 255,
        fill_rule: SvgFillRule::NonZero,
        clip: None,
        clip_path: SvgPath::new(Rect::new(0, 0, 0, 0)),
        has_clip_path: false,
        mask: None,
        mask_path: SvgPath::new(Rect::new(0, 0, 0, 0)),
        has_mask_path: false,
        filter: SvgFilterEffect::None,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgDocument<const PATHS: usize, const CMDS: usize> {
    pub view_box: Rect,
    pub paths: [SvgPathNode<CMDS>; PATHS],
    pub len: usize,
    pub overflowed: bool,
    pub unsupported_features: u32,
}

impl<const PATHS: usize, const CMDS: usize> SvgDocument<PATHS, CMDS> {
    pub const EMPTY: Self = Self {
        view_box: Rect::EMPTY,
        paths: [SvgPathNode::EMPTY; PATHS],
        len: 0,
        overflowed: false,
        unsupported_features: 0,
    };

    pub fn reset(&mut self, view_box: Rect, unsupported_features: u32) {
        self.view_box = view_box;
        self.len = 0;
        self.overflowed = false;
        self.unsupported_features = unsupported_features;
    }

    pub fn plan_pipeline<const STAGES: usize>() -> CodecPipelinePlan<STAGES> {
        Self::plan_pipeline_with_caps(CodecAcceleratorCapabilities::NONE)
    }

    pub fn plan_pipeline_with_caps<const STAGES: usize>(
        caps: CodecAcceleratorCapabilities,
    ) -> CodecPipelinePlan<STAGES> {
        let supported = caps.svg && caps.max_stages >= 7;
        let mut plan = CodecPipelinePlan::new();
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Read, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Inspect, false, true));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Header, true, supported));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Parse, true, supported));
        let _ = plan.push(CodecStagePlan::new(
            CodecStageKind::Transform,
            true,
            supported,
        ));
        let _ = plan.push(CodecStagePlan::new(CodecStageKind::Raster, true, supported));
        let _ = plan.push(CodecStagePlan::new(
            CodecStageKind::CacheInsert,
            false,
            true,
        ));
        plan
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgDocumentView {
    pub document: *const DefaultSvgDocument,
}

impl SvgDocumentView {
    pub const fn from_ptr(document: *const DefaultSvgDocument) -> Self {
        Self { document }
    }

    pub fn from_ref(document: &DefaultSvgDocument) -> Self {
        Self {
            document: document as *const DefaultSvgDocument,
        }
    }

    pub fn get(self) -> Option<&'static DefaultSvgDocument> {
        if self.document.is_null() {
            None
        } else {
            Some(unsafe { &*self.document })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgDocumentCacheStats {
    pub slots: usize,
    pub capacity: usize,
    pub hits: u32,
    pub misses: u32,
    pub loads: u32,
    pub load_failures: u32,
    pub decode_failures: u32,
    pub overflows: u32,
    pub pipeline_candidates: u32,
    pub pipeline_stages: u32,
    pub pipeline_hardware_candidates: u32,
    pub pipeline_fallbacks: u32,
    pub pipeline_unsupported: u32,
    pub pipeline_overflows: u32,
}

impl SvgDocumentCacheStats {
    pub const fn new() -> Self {
        Self {
            slots: 0,
            capacity: 0,
            hits: 0,
            misses: 0,
            loads: 0,
            load_failures: 0,
            decode_failures: 0,
            overflows: 0,
            pipeline_candidates: 0,
            pipeline_stages: 0,
            pipeline_hardware_candidates: 0,
            pipeline_fallbacks: 0,
            pipeline_unsupported: 0,
            pipeline_overflows: 0,
        }
    }

    pub const fn fallbacks(self) -> u32 {
        self.load_failures
            .saturating_add(self.decode_failures)
            .saturating_add(self.overflows)
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

    pub fn record_pipeline_fallback(&mut self, stage_kind: Option<CodecErrorKind>) {
        self.pipeline_fallbacks = self.pipeline_fallbacks.saturating_add(1);
        if matches!(stage_kind, Some(CodecErrorKind::Unsupported)) {
            self.pipeline_unsupported = self.pipeline_unsupported.saturating_add(1);
        }
        if matches!(stage_kind, Some(CodecErrorKind::Overflow)) {
            self.pipeline_overflows = self.pipeline_overflows.saturating_add(1);
        }
    }
}

pub struct SvgDocumentCache<const DOCS: usize, const PATHS: usize, const CMDS: usize> {
    ids: [SvgId; DOCS],
    documents: [SvgDocument<PATHS, CMDS>; DOCS],
    used: [bool; DOCS],
    len: usize,
    stats: SvgDocumentCacheStats,
}

impl<const DOCS: usize, const PATHS: usize, const CMDS: usize> SvgDocumentCache<DOCS, PATHS, CMDS> {
    pub const fn new() -> Self {
        Self {
            ids: [SvgId(0); DOCS],
            documents: [SvgDocument::EMPTY; DOCS],
            used: [false; DOCS],
            len: 0,
            stats: SvgDocumentCacheStats::new(),
        }
    }

    pub fn clear(&mut self) {
        let mut index = 0usize;
        while index < DOCS {
            self.ids[index] = SvgId(0);
            self.documents[index] = SvgDocument::EMPTY;
            self.used[index] = false;
            index += 1;
        }
        self.len = 0;
        self.stats = SvgDocumentCacheStats::new();
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn capacity(&self) -> usize {
        DOCS
    }

    pub fn stats(&self) -> SvgDocumentCacheStats {
        let mut stats = self.stats;
        stats.slots = self.len;
        stats.capacity = DOCS;
        stats
    }

    fn find_index(&self, id: SvgId) -> Option<usize> {
        let mut index = 0usize;
        while index < self.len {
            if self.used[index] && self.ids[index] == id {
                return Some(index);
            }
            index += 1;
        }
        None
    }
}

impl<const DOCS: usize> SvgDocumentCache<DOCS, SVG_DOCUMENT_PATHS, SVG_DOCUMENT_COMMANDS> {
    pub fn view(&mut self, id: SvgId) -> Option<SvgDocumentView> {
        if let Some(index) = self.find_index(id) {
            self.stats.hits = self.stats.hits.saturating_add(1);
            Some(SvgDocumentView::from_ref(&self.documents[index]))
        } else {
            self.stats.misses = self.stats.misses.saturating_add(1);
            None
        }
    }

    pub fn prewarm<L: ResourceLoader>(
        &mut self,
        id: SvgId,
        path: &[u8],
        loader: &mut L,
        fallback_view_box: Rect,
    ) -> bool {
        self.prewarm_result(id, path, loader, fallback_view_box)
            .is_ok()
    }

    pub fn prewarm_result<L: ResourceLoader>(
        &mut self,
        id: SvgId,
        path: &[u8],
        loader: &mut L,
        fallback_view_box: Rect,
    ) -> Result<SvgDocumentView, CodecErrorKind> {
        if let Some(view) = self.view(id) {
            return Ok(view);
        }

        if self.len >= DOCS {
            self.stats.overflows = self.stats.overflows.saturating_add(1);
            self.stats
                .record_pipeline_fallback(Some(CodecErrorKind::Overflow));
            return Err(CodecErrorKind::Overflow);
        }

        let mut bytes = Vec::new();
        if !loader.load(path, &mut bytes) {
            self.stats.load_failures = self.stats.load_failures.saturating_add(1);
            self.stats
                .record_pipeline_fallback(Some(CodecErrorKind::MissingResource));
            return Err(CodecErrorKind::MissingResource);
        }

        let plan = DefaultSvgDocument::plan_pipeline::<DEFAULT_CODEC_PIPELINE_STAGES>();
        self.stats.record_pipeline(plan.stats());

        let Ok(data) = core::str::from_utf8(&bytes) else {
            self.stats.decode_failures = self.stats.decode_failures.saturating_add(1);
            self.stats
                .record_pipeline_fallback(Some(CodecErrorKind::Invalid));
            return Err(CodecErrorKind::Invalid);
        };

        let slot = self.len;
        if !parse_svg_document_into(data, fallback_view_box, &mut self.documents[slot]) {
            self.documents[slot] = SvgDocument::EMPTY;
            self.stats.decode_failures = self.stats.decode_failures.saturating_add(1);
            self.stats
                .record_pipeline_fallback(Some(CodecErrorKind::Invalid));
            return Err(CodecErrorKind::Invalid);
        }
        if self.documents[slot].overflowed {
            self.documents[slot] = SvgDocument::EMPTY;
            self.stats.decode_failures = self.stats.decode_failures.saturating_add(1);
            self.stats.overflows = self.stats.overflows.saturating_add(1);
            self.stats
                .record_pipeline_fallback(Some(CodecErrorKind::Overflow));
            return Err(CodecErrorKind::Overflow);
        }

        self.ids[slot] = id;
        self.used[slot] = true;
        self.len += 1;
        self.stats.loads = self.stats.loads.saturating_add(1);
        Ok(SvgDocumentView::from_ref(&self.documents[slot]))
    }
}

pub fn parse_svg_document<const PATHS: usize, const CMDS: usize>(
    data: &str,
    fallback_view_box: Rect,
) -> Option<SvgDocument<PATHS, CMDS>> {
    let mut document = SvgDocument::EMPTY;
    if parse_svg_document_into(data, fallback_view_box, &mut document) {
        Some(document)
    } else {
        None
    }
}

pub fn parse_svg_document_into<const PATHS: usize, const CMDS: usize>(
    data: &str,
    fallback_view_box: Rect,
    document: &mut SvgDocument<PATHS, CMDS>,
) -> bool {
    let view_box = parse_view_box(data).unwrap_or(fallback_view_box);
    let root_tag = first_tag(data, "<svg").unwrap_or(data);
    let root_state = SvgGroupState::root().inherit(data, root_tag);
    document.reset(view_box, detect_svg_unsupported_features(data));

    let mut search = 0usize;
    while let Some(path_pos) = data.get(search..).and_then(|text| text.find("<path")) {
        let start = search + path_pos;
        let end = match data.get(start..).and_then(|text| text.find('>')) {
            Some(end) => start + end + 1,
            None => break,
        };
        let Some(tag) = data.get(start..end) else {
            break;
        };
        if let Some(path_data) = parse_attr(tag, "d") {
            let group_state = inherited_group_state(data, start, root_state);
            let path_transform =
                parse_transform(parse_attr(tag, "transform")).unwrap_or(SvgTransform::IDENTITY);
            let transform = group_state.transform.then(path_transform);
            let mut path = parse_svg_path(view_box, path_data, 1);
            transform_path(&mut path, transform);
            let node_opacity = parse_attr(tag, "opacity")
                .and_then(parse_opacity)
                .unwrap_or(255);
            let clip_path = parse_clip_path(data, tag, view_box);
            let mask_path = parse_mask_path(data, tag, view_box);
            let node = SvgPathNode {
                path,
                fill: parse_document_paint(data, tag_value(data, tag, "fill"))
                    .unwrap_or(group_state.fill),
                stroke: parse_document_paint(data, tag_value(data, tag, "stroke"))
                    .unwrap_or(group_state.stroke),
                stroke_width: tag_value(data, tag, "stroke-width")
                    .and_then(parse_u16)
                    .unwrap_or(1),
                stroke_cap: parse_stroke_cap(tag_value(data, tag, "stroke-linecap"))
                    .unwrap_or(group_state.stroke_cap),
                stroke_join: parse_stroke_join(tag_value(data, tag, "stroke-linejoin"))
                    .unwrap_or(group_state.stroke_join),
                opacity: mul_opacity(group_state.opacity, node_opacity),
                fill_rule: parse_fill_rule(tag_value(data, tag, "fill-rule"))
                    .unwrap_or(group_state.fill_rule),
                clip: clip_path.as_ref().and_then(path_bounds),
                clip_path: clip_path.unwrap_or_else(|| SvgPath::new(view_box)),
                has_clip_path: clip_path.is_some(),
                mask: mask_path.as_ref().and_then(path_bounds),
                mask_path: mask_path.unwrap_or_else(|| SvgPath::new(view_box)),
                has_mask_path: mask_path.is_some(),
                filter: parse_filter_effect(data, tag),
            };
            push_document_node(document, node);
        }
        search = end;
    }

    append_shape_nodes(document, data, view_box, root_state);

    document.len != 0
}

fn append_shape_nodes<const PATHS: usize, const CMDS: usize>(
    document: &mut SvgDocument<PATHS, CMDS>,
    data: &str,
    view_box: Rect,
    root_state: SvgGroupState,
) {
    let mut search = 0usize;
    while let Some(relative) = data.get(search..).and_then(|text| text.find('<')) {
        let start = search + relative;
        let Some(end_rel) = data.get(start..).and_then(|text| text.find('>')) else {
            break;
        };
        let end = start + end_rel + 1;
        let Some(tag) = data.get(start..end) else {
            break;
        };
        let path = if tag.starts_with("<rect") {
            rect_to_path(view_box, tag)
        } else if tag.starts_with("<circle") {
            circle_to_path(view_box, tag, false)
        } else if tag.starts_with("<ellipse") {
            circle_to_path(view_box, tag, true)
        } else if tag.starts_with("<line") {
            line_to_path(view_box, tag)
        } else if tag.starts_with("<polyline") {
            points_to_path(view_box, tag, false)
        } else if tag.starts_with("<polygon") {
            points_to_path(view_box, tag, true)
        } else if tag.starts_with("<use") {
            use_to_path(view_box, data, tag)
        } else {
            None
        };

        if let Some(mut path) = path {
            let group_state = inherited_group_state(data, start, root_state);
            let transform = group_state.transform.then(
                parse_transform(parse_attr(tag, "transform")).unwrap_or(SvgTransform::IDENTITY),
            );
            transform_path(&mut path, transform);
            let node_opacity = tag_value(data, tag, "opacity")
                .and_then(parse_opacity)
                .unwrap_or(255);
            let clip_path = parse_clip_path(data, tag, view_box);
            let mask_path = parse_mask_path(data, tag, view_box);
            let node = SvgPathNode {
                path,
                fill: parse_document_paint(data, tag_value(data, tag, "fill"))
                    .unwrap_or(group_state.fill),
                stroke: parse_document_paint(data, tag_value(data, tag, "stroke"))
                    .unwrap_or(group_state.stroke),
                stroke_width: tag_value(data, tag, "stroke-width")
                    .and_then(parse_u16)
                    .unwrap_or(1),
                stroke_cap: parse_stroke_cap(tag_value(data, tag, "stroke-linecap"))
                    .unwrap_or(group_state.stroke_cap),
                stroke_join: parse_stroke_join(tag_value(data, tag, "stroke-linejoin"))
                    .unwrap_or(group_state.stroke_join),
                opacity: mul_opacity(group_state.opacity, node_opacity),
                fill_rule: parse_fill_rule(tag_value(data, tag, "fill-rule"))
                    .unwrap_or(group_state.fill_rule),
                clip: clip_path.as_ref().and_then(path_bounds),
                clip_path: clip_path.unwrap_or_else(|| SvgPath::new(view_box)),
                has_clip_path: clip_path.is_some(),
                mask: mask_path.as_ref().and_then(path_bounds),
                mask_path: mask_path.unwrap_or_else(|| SvgPath::new(view_box)),
                has_mask_path: mask_path.is_some(),
                filter: parse_filter_effect(data, tag),
            };
            push_document_node(document, node);
        }
        search = end;
    }
}

fn push_document_node<const PATHS: usize, const CMDS: usize>(
    document: &mut SvgDocument<PATHS, CMDS>,
    node: SvgPathNode<CMDS>,
) {
    if document.len >= PATHS {
        document.overflowed = true;
    } else {
        document.overflowed |= node.path.overflowed;
        document.paths[document.len] = node;
        document.len += 1;
    }
}

fn rect_to_path<const N: usize>(view_box: Rect, tag: &str) -> Option<SvgPath<N>> {
    let x = attr_number(tag, "x").unwrap_or(0);
    let y = attr_number(tag, "y").unwrap_or(0);
    let w = attr_number(tag, "width")?.max(0);
    let h = attr_number(tag, "height")?.max(0);
    let mut path = SvgPath::new(view_box);
    path.push(SvgPathCommand::MoveTo(Point::new(x, y)));
    path.push(SvgPathCommand::LineTo(Point::new(x + w, y)));
    path.push(SvgPathCommand::LineTo(Point::new(x + w, y + h)));
    path.push(SvgPathCommand::LineTo(Point::new(x, y + h)));
    path.push(SvgPathCommand::Close);
    Some(path)
}

fn circle_to_path<const N: usize>(view_box: Rect, tag: &str, ellipse: bool) -> Option<SvgPath<N>> {
    let cx = attr_number(tag, "cx").unwrap_or(0);
    let cy = attr_number(tag, "cy").unwrap_or(0);
    let rx = if ellipse {
        attr_number(tag, "rx")?.max(1)
    } else {
        attr_number(tag, "r")?.max(1)
    };
    let ry = if ellipse {
        attr_number(tag, "ry")?.max(1)
    } else {
        rx
    };
    const CIRCLE: [(i32, i32); 16] = [
        (1024, 0),
        (946, 392),
        (724, 724),
        (392, 946),
        (0, 1024),
        (-392, 946),
        (-724, 724),
        (-946, 392),
        (-1024, 0),
        (-946, -392),
        (-724, -724),
        (-392, -946),
        (0, -1024),
        (392, -946),
        (724, -724),
        (946, -392),
    ];
    let mut path = SvgPath::new(view_box);
    let mut i = 0usize;
    while i < CIRCLE.len() {
        let point = Point::new(
            cx + (CIRCLE[i].0 * rx) / 1024,
            cy + (CIRCLE[i].1 * ry) / 1024,
        );
        if i == 0 {
            path.push(SvgPathCommand::MoveTo(point));
        } else {
            path.push(SvgPathCommand::LineTo(point));
        }
        i += 1;
    }
    path.push(SvgPathCommand::Close);
    Some(path)
}

fn line_to_path<const N: usize>(view_box: Rect, tag: &str) -> Option<SvgPath<N>> {
    let x1 = attr_number(tag, "x1")?;
    let y1 = attr_number(tag, "y1")?;
    let x2 = attr_number(tag, "x2")?;
    let y2 = attr_number(tag, "y2")?;
    let mut path = SvgPath::new(view_box);
    path.push(SvgPathCommand::MoveTo(Point::new(x1, y1)));
    path.push(SvgPathCommand::LineTo(Point::new(x2, y2)));
    Some(path)
}

fn points_to_path<const N: usize>(view_box: Rect, tag: &str, closed: bool) -> Option<SvgPath<N>> {
    let points = parse_attr(tag, "points")?;
    let bytes = points.as_bytes();
    let mut index = 0usize;
    let mut path = SvgPath::new(view_box);
    let mut first = true;
    while index < bytes.len() {
        let Some(x) = parse_number(bytes, &mut index, 1) else {
            break;
        };
        let Some(y) = parse_number(bytes, &mut index, 1) else {
            break;
        };
        let point = Point::new(x, y);
        if first {
            path.push(SvgPathCommand::MoveTo(point));
            first = false;
        } else {
            path.push(SvgPathCommand::LineTo(point));
        }
    }
    if first {
        return None;
    }
    if closed {
        path.push(SvgPathCommand::Close);
    }
    Some(path)
}

fn use_to_path<const N: usize>(view_box: Rect, data: &str, tag: &str) -> Option<SvgPath<N>> {
    let href = parse_attr(tag, "href").or_else(|| parse_attr(tag, "xlink:href"))?;
    let id = href.strip_prefix('#')?;
    let id_pos = data.find(id)?;
    let start = data.get(..id_pos)?.rfind('<')?;
    let end = data.get(start..)?.find('>')? + start + 1;
    let target = data.get(start..end)?;
    let mut path = if target.starts_with("<path") {
        parse_attr(target, "d").map(|d| parse_svg_path(view_box, d, 1))?
    } else if target.starts_with("<rect") {
        rect_to_path(view_box, target)?
    } else if target.starts_with("<circle") {
        circle_to_path(view_box, target, false)?
    } else if target.starts_with("<ellipse") {
        circle_to_path(view_box, target, true)?
    } else {
        return None;
    };
    let mut transform =
        parse_transform(parse_attr(tag, "transform")).unwrap_or(SvgTransform::IDENTITY);
    let x = attr_number(tag, "x").unwrap_or(0);
    let y = attr_number(tag, "y").unwrap_or(0);
    if x != 0 || y != 0 {
        transform = SvgTransform {
            e: x,
            f: y,
            ..SvgTransform::IDENTITY
        }
        .then(transform);
    }
    transform_path(&mut path, transform);
    Some(path)
}

fn attr_number(tag: &str, name: &str) -> Option<i32> {
    let value = parse_attr(tag, name)?;
    let mut index = 0usize;
    parse_number(value.as_bytes(), &mut index, 1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SvgPathCommand {
    Empty,
    MoveTo(Point),
    LineTo(Point),
    Close,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgPath<const N: usize> {
    pub view_box: Rect,
    pub commands: [SvgPathCommand; N],
    pub len: usize,
    pub overflowed: bool,
}

impl<const N: usize> SvgPath<N> {
    pub const fn new(view_box: Rect) -> Self {
        Self {
            view_box,
            commands: [SvgPathCommand::Empty; N],
            len: 0,
            overflowed: false,
        }
    }

    pub fn push(&mut self, command: SvgPathCommand) -> bool {
        if self.len >= N {
            self.overflowed = true;
            return false;
        }
        self.commands[self.len] = command;
        self.len += 1;
        true
    }
}

pub struct SvgCache<const ICONS: usize, const CMDS: usize> {
    ids: [SvgId; ICONS],
    paths: [SvgPath<CMDS>; ICONS],
    used: [bool; ICONS],
    len: usize,
    overflowed: bool,
}

impl<const ICONS: usize, const CMDS: usize> SvgCache<ICONS, CMDS> {
    pub const fn new() -> Self {
        Self {
            ids: [SvgId(0); ICONS],
            paths: [SvgPath::new(Rect::new(0, 0, 0, 0)); ICONS],
            used: [false; ICONS],
            len: 0,
            overflowed: false,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub fn insert_path(&mut self, id: SvgId, path: SvgPath<CMDS>) -> bool {
        let mut i = 0;
        while i < self.len {
            if self.used[i] && self.ids[i] == id {
                self.paths[i] = path;
                self.overflowed |= path.overflowed;
                return true;
            }
            i += 1;
        }
        if self.len >= ICONS {
            self.overflowed = true;
            return false;
        }
        self.ids[self.len] = id;
        self.paths[self.len] = path;
        self.used[self.len] = true;
        self.len += 1;
        self.overflowed |= path.overflowed;
        true
    }

    pub fn get(&self, id: SvgId) -> Option<&SvgPath<CMDS>> {
        let mut i = 0;
        while i < self.len {
            if self.used[i] && self.ids[i] == id {
                return Some(&self.paths[i]);
            }
            i += 1;
        }
        None
    }
}

pub fn parse_svg_path<const N: usize>(view_box: Rect, data: &str, scale: i32) -> SvgPath<N> {
    let bytes = data.as_bytes();
    let mut path = SvgPath::new(view_box);
    let mut index = 0;
    let mut cursor = Point::new(0, 0);
    let mut sub_start = Point::new(0, 0);
    let mut last_cmd = b'M';
    while index < bytes.len() {
        skip_separators(bytes, &mut index);
        if index >= bytes.len() {
            break;
        }

        let b = bytes[index];
        let cmd = if is_command(b) {
            index += 1;
            last_cmd = b;
            b
        } else {
            last_cmd
        };

        match cmd {
            b'M' | b'm' => {
                if let (Some(x), Some(y)) = (
                    parse_number(bytes, &mut index, scale),
                    parse_number(bytes, &mut index, scale),
                ) {
                    cursor = point_for(cmd, cursor, x, y);
                    sub_start = cursor;
                    path.push(SvgPathCommand::MoveTo(cursor));
                    last_cmd = if cmd == b'm' { b'l' } else { b'L' };
                } else {
                    break;
                }
            }
            b'L' | b'l' => {
                if let (Some(x), Some(y)) = (
                    parse_number(bytes, &mut index, scale),
                    parse_number(bytes, &mut index, scale),
                ) {
                    cursor = point_for(cmd, cursor, x, y);
                    path.push(SvgPathCommand::LineTo(cursor));
                } else {
                    break;
                }
            }
            b'H' | b'h' => {
                if let Some(x) = parse_number(bytes, &mut index, scale) {
                    cursor = if cmd == b'h' {
                        Point::new(cursor.x + x, cursor.y)
                    } else {
                        Point::new(x, cursor.y)
                    };
                    path.push(SvgPathCommand::LineTo(cursor));
                } else {
                    break;
                }
            }
            b'V' | b'v' => {
                if let Some(y) = parse_number(bytes, &mut index, scale) {
                    cursor = if cmd == b'v' {
                        Point::new(cursor.x, cursor.y + y)
                    } else {
                        Point::new(cursor.x, y)
                    };
                    path.push(SvgPathCommand::LineTo(cursor));
                } else {
                    break;
                }
            }
            b'C' | b'c' => {
                let _ = parse_number(bytes, &mut index, scale);
                let _ = parse_number(bytes, &mut index, scale);
                let _ = parse_number(bytes, &mut index, scale);
                let _ = parse_number(bytes, &mut index, scale);
                if let (Some(x), Some(y)) = (
                    parse_number(bytes, &mut index, scale),
                    parse_number(bytes, &mut index, scale),
                ) {
                    cursor = point_for(cmd, cursor, x, y);
                    path.push(SvgPathCommand::LineTo(cursor));
                } else {
                    break;
                }
            }
            b'Q' | b'q' => {
                let _ = parse_number(bytes, &mut index, scale);
                let _ = parse_number(bytes, &mut index, scale);
                if let (Some(x), Some(y)) = (
                    parse_number(bytes, &mut index, scale),
                    parse_number(bytes, &mut index, scale),
                ) {
                    cursor = point_for(cmd, cursor, x, y);
                    path.push(SvgPathCommand::LineTo(cursor));
                } else {
                    break;
                }
            }
            b'A' | b'a' => {
                let _ = parse_number(bytes, &mut index, scale);
                let _ = parse_number(bytes, &mut index, scale);
                let _ = parse_number(bytes, &mut index, scale);
                let _ = parse_number(bytes, &mut index, scale);
                let _ = parse_number(bytes, &mut index, scale);
                if let (Some(x), Some(y)) = (
                    parse_number(bytes, &mut index, scale),
                    parse_number(bytes, &mut index, scale),
                ) {
                    cursor = point_for(cmd, cursor, x, y);
                    path.push(SvgPathCommand::LineTo(cursor));
                } else {
                    break;
                }
            }
            b'S' | b's' => {
                let _ = parse_number(bytes, &mut index, scale);
                let _ = parse_number(bytes, &mut index, scale);
                if let (Some(x), Some(y)) = (
                    parse_number(bytes, &mut index, scale),
                    parse_number(bytes, &mut index, scale),
                ) {
                    cursor = point_for(cmd, cursor, x, y);
                    path.push(SvgPathCommand::LineTo(cursor));
                } else {
                    break;
                }
            }
            b'T' | b't' => {
                if let (Some(x), Some(y)) = (
                    parse_number(bytes, &mut index, scale),
                    parse_number(bytes, &mut index, scale),
                ) {
                    cursor = point_for(cmd, cursor, x, y);
                    path.push(SvgPathCommand::LineTo(cursor));
                } else {
                    break;
                }
            }
            b'Z' | b'z' => {
                cursor = sub_start;
                path.push(SvgPathCommand::Close);
            }
            _ => {
                index += 1;
            }
        }
    }
    path
}

fn parse_view_box(data: &str) -> Option<Rect> {
    let value = parse_attr(data, "viewBox")?;
    let bytes = value.as_bytes();
    let mut index = 0usize;
    let x = parse_number(bytes, &mut index, 1)?;
    let y = parse_number(bytes, &mut index, 1)?;
    let w = parse_number(bytes, &mut index, 1)?;
    let h = parse_number(bytes, &mut index, 1)?;
    Some(Rect::new(
        x,
        y,
        w.max(0).min(u16::MAX as i32) as u16,
        h.max(0).min(u16::MAX as i32) as u16,
    ))
}

fn parse_attr<'a>(data: &'a str, attr: &str) -> Option<&'a str> {
    let bytes = data.as_bytes();
    let mut search = 0usize;
    while search < data.len() {
        let pos = data.get(search..)?.find(attr)? + search;
        let before_ok = pos == 0
            || matches!(
                bytes.get(pos - 1).copied(),
                Some(b'<' | b' ' | b'\n' | b'\r' | b'\t' | b'/')
            );
        let mut index = pos + attr.len();
        while index < bytes.len() && matches!(bytes[index], b' ' | b'\n' | b'\r' | b'\t') {
            index += 1;
        }
        if before_ok && bytes.get(index) == Some(&b'=') {
            index += 1;
            while index < bytes.len() && matches!(bytes[index], b' ' | b'\n' | b'\r' | b'\t') {
                index += 1;
            }
            let quote = *bytes.get(index)?;
            if quote != b'"' && quote != b'\'' {
                return None;
            }
            index += 1;
            let start = index;
            while index < bytes.len() && bytes[index] != quote {
                index += 1;
            }
            return data.get(start..index);
        }
        search = pos + attr.len();
    }
    None
}

fn tag_value<'a>(document: &'a str, tag: &'a str, name: &str) -> Option<&'a str> {
    parse_attr(tag, name)
        .or_else(|| parse_attr(tag, "style").and_then(|style| style_decl_value(style, name)))
        .or_else(|| class_decl_value(document, tag, name))
}

fn style_decl_value<'a>(style: &'a str, name: &str) -> Option<&'a str> {
    let mut search = 0usize;
    while search < style.len() {
        let pos = style.get(search..)?.find(name)? + search;
        let after_name = pos.checked_add(name.len())?;
        let mut index = after_name;
        let bytes = style.as_bytes();
        while index < bytes.len() && matches!(bytes[index], b' ' | b'\n' | b'\r' | b'\t') {
            index += 1;
        }
        if bytes.get(index) != Some(&b':') {
            search = after_name;
            continue;
        }
        index += 1;
        while index < bytes.len() && matches!(bytes[index], b' ' | b'\n' | b'\r' | b'\t') {
            index += 1;
        }
        let start = index;
        while index < bytes.len() && !matches!(bytes[index], b';' | b'}') {
            index += 1;
        }
        return style.get(start..index).map(|value| value.trim());
    }
    None
}

fn class_decl_value<'a>(document: &'a str, tag: &'a str, name: &str) -> Option<&'a str> {
    let class = parse_attr(tag, "class")?;
    let class_name = class.split_whitespace().next()?;
    let mut search = 0usize;
    let pos = loop {
        let found = document.get(search..)?.find(class_name)? + search;
        if found > 0 && document.as_bytes().get(found - 1) == Some(&b'.') {
            let after = found + class_name.len();
            if matches!(
                document.as_bytes().get(after).copied(),
                Some(b' ' | b'\n' | b'\r' | b'\t' | b'{')
            ) {
                break found - 1;
            }
        }
        search = found + class_name.len();
    };
    let body_start = document.get(pos..)?.find('{')? + pos + 1;
    let body_end = document.get(body_start..)?.find('}')? + body_start;
    style_decl_value(document.get(body_start..body_end)?, name)
}

fn parse_document_paint(document: &str, value: Option<&str>) -> Option<SvgPaint> {
    let value = value?.trim();
    if let Some(id) = value
        .strip_prefix("url(#")
        .and_then(|rest| rest.split(')').next())
    {
        return parse_gradient_paint(document, id);
    }
    parse_paint(Some(value))
}

fn parse_paint(value: Option<&str>) -> Option<SvgPaint> {
    let value = value?.trim();
    if value == "none" {
        Some(SvgPaint::None)
    } else if value == "currentColor" || value == "currentcolor" {
        Some(SvgPaint::CurrentColor)
    } else if let Some(hex) = value.strip_prefix('#') {
        parse_hex_color(hex).map(SvgPaint::Color)
    } else {
        None
    }
}

fn parse_gradient_paint(document: &str, id: &str) -> Option<SvgPaint> {
    let needle = "id=\"";
    let id_pos = document.find(id)?;
    let tag_start = document.get(..id_pos)?.rfind('<')?;
    let radial = document.get(tag_start..)?.starts_with("<radialGradient");
    let linear = document.get(tag_start..)?.starts_with("<linearGradient");
    if !linear && !radial {
        let attr_pos = document.get(..id_pos)?.rfind(needle)?;
        if attr_pos + needle.len() != id_pos {
            return None;
        }
    }
    let tag_end = document
        .get(id_pos..)?
        .find("</")
        .map(|end| id_pos + end)
        .unwrap_or(document.len());
    let region = document.get(tag_start..tag_end)?;
    let mut first = None;
    let mut last = None;
    let mut search = 0usize;
    while let Some(stop_pos) = region.get(search..)?.find("<stop") {
        let start = search + stop_pos;
        let end = region.get(start..)?.find('>')? + start + 1;
        let tag = region.get(start..end)?;
        let color = tag_value(document, tag, "stop-color")
            .and_then(|value| parse_paint(Some(value)))
            .and_then(|paint| match paint {
                SvgPaint::Color(color) => Some(color),
                _ => None,
            });
        if let Some(color) = color {
            if first.is_none() {
                first = Some(color);
            }
            last = Some(color);
        }
        search = end;
    }
    match (first, last) {
        (Some(a), Some(b)) => Some(if radial {
            SvgPaint::RadialGradient(a, b)
        } else {
            SvgPaint::LinearGradient(a, b)
        }),
        (Some(a), None) => Some(SvgPaint::Color(a)),
        _ => None,
    }
}

fn detect_svg_unsupported_features(data: &str) -> u32 {
    let mut flags = 0u32;
    if data.contains("<clipPath")
        && !has_simple_clip_or_mask_shape(data, "<clipPath", "</clipPath>")
    {
        flags |= 1 << (SvgUnsupportedFeature::ClipPathComplex as u32);
    }
    if data.contains("<mask") && !has_simple_clip_or_mask_shape(data, "<mask", "</mask>") {
        flags |= 1 << (SvgUnsupportedFeature::MaskComplex as u32);
    }
    if data.contains("<filter") {
        if !(data.contains("<feGaussianBlur") || data.contains("<feDropShadow")) {
            flags |= 1 << (SvgUnsupportedFeature::FilterBudgetExceeded as u32);
        }
    }
    if data.contains("spreadMethod") || data.contains("<pattern") {
        flags |= 1 << (SvgUnsupportedFeature::GradientSpread as u32);
    }
    if data.contains('@') || data.contains(" > ") || data.contains(':') {
        flags |= 1 << (SvgUnsupportedFeature::CssSelector as u32);
    }
    flags
}

fn has_simple_clip_or_mask_shape(data: &str, start_tag: &str, end_tag: &str) -> bool {
    let mut search = 0usize;
    while let Some(start_rel) = data.get(search..).and_then(|text| text.find(start_tag)) {
        let start = search + start_rel;
        let end = data
            .get(start..)
            .and_then(|text| text.find(end_tag).map(|offset| start + offset))
            .unwrap_or(data.len());
        if let Some(region) = data.get(start..end) {
            if region.contains("<path") || region.contains("<rect") || region.contains("<circle") {
                return true;
            }
        }
        search = start + start_tag.len();
    }
    false
}

fn parse_clip_path<const N: usize>(data: &str, tag: &str, view_box: Rect) -> Option<SvgPath<N>> {
    let id = tag_value(data, tag, "clip-path")?
        .trim()
        .strip_prefix("url(#")?
        .split(')')
        .next()?;
    let pos = data.find(id)?;
    let start = data.get(..pos)?.rfind("<clipPath")?;
    let end = data
        .get(pos..)?
        .find("</clipPath>")
        .map(|end| pos + end)
        .unwrap_or(data.len());
    let region = data.get(start..end)?;
    first_shape_path(view_box, region)
}

fn parse_mask_path<const N: usize>(data: &str, tag: &str, view_box: Rect) -> Option<SvgPath<N>> {
    let id = tag_value(data, tag, "mask")?
        .trim()
        .strip_prefix("url(#")?
        .split(')')
        .next()?;
    let pos = data.find(id)?;
    let start = data.get(..pos)?.rfind("<mask")?;
    let end = data
        .get(pos..)?
        .find("</mask>")
        .map(|end| pos + end)
        .unwrap_or(data.len());
    let region = data.get(start..end)?;
    first_shape_path(view_box, region)
}

fn first_shape_path<const N: usize>(view_box: Rect, region: &str) -> Option<SvgPath<N>> {
    if let Some(path_start) = region.find("<path") {
        let tag_end = region.get(path_start..)?.find('>')? + path_start + 1;
        let tag = region.get(path_start..tag_end)?;
        parse_attr(tag, "d").map(|d| parse_svg_path::<N>(view_box, d, 1))
    } else if let Some(rect_start) = region.find("<rect") {
        let tag_end = region.get(rect_start..)?.find('>')? + rect_start + 1;
        rect_to_path(view_box, region.get(rect_start..tag_end)?)
    } else if let Some(circle_start) = region.find("<circle") {
        let tag_end = region.get(circle_start..)?.find('>')? + circle_start + 1;
        circle_to_path(view_box, region.get(circle_start..tag_end)?, false)
    } else {
        None
    }
}

fn parse_filter_effect(data: &str, tag: &str) -> SvgFilterEffect {
    let Some(id) = tag_value(data, tag, "filter")
        .and_then(|value| value.trim().strip_prefix("url(#"))
        .and_then(|rest| rest.split(')').next())
    else {
        return SvgFilterEffect::None;
    };
    let Some(pos) = data.find(id) else {
        return SvgFilterEffect::None;
    };
    let Some(start) = data.get(..pos).and_then(|text| text.rfind("<filter")) else {
        return SvgFilterEffect::None;
    };
    let end = data
        .get(pos..)
        .and_then(|text| text.find("</filter>").map(|end| pos + end))
        .unwrap_or(data.len());
    let Some(region) = data.get(start..end) else {
        return SvgFilterEffect::None;
    };
    if let Some(drop) = region.find("<feDropShadow") {
        let tag_end = region
            .get(drop..)
            .and_then(|text| text.find('>'))
            .map(|end| drop + end + 1)
            .unwrap_or(region.len());
        let shadow = region.get(drop..tag_end).unwrap_or("");
        let dx = attr_number(shadow, "dx")
            .unwrap_or(2)
            .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        let dy = attr_number(shadow, "dy")
            .unwrap_or(2)
            .clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        let radius = attr_number(shadow, "stdDeviation")
            .unwrap_or(2)
            .max(0)
            .min(32) as u16;
        let color = tag_value(data, shadow, "flood-color")
            .and_then(|value| parse_paint(Some(value)))
            .and_then(|paint| match paint {
                SvgPaint::Color(color) => Some(color),
                _ => None,
            })
            .unwrap_or(Color::rgba(0, 0, 0, 160));
        return SvgFilterEffect::DropShadow {
            dx,
            dy,
            radius,
            color,
        };
    }
    if let Some(blur) = region.find("<feGaussianBlur") {
        let tag_end = region
            .get(blur..)
            .and_then(|text| text.find('>'))
            .map(|end| blur + end + 1)
            .unwrap_or(region.len());
        let blur_tag = region.get(blur..tag_end).unwrap_or("");
        let radius = attr_number(blur_tag, "stdDeviation")
            .unwrap_or(2)
            .max(0)
            .min(32) as u16;
        return SvgFilterEffect::Blur(radius);
    }
    SvgFilterEffect::None
}

fn path_bounds<const N: usize>(path: &SvgPath<N>) -> Option<Rect> {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    let mut i = 0usize;
    while i < path.len {
        let point = match path.commands[i] {
            SvgPathCommand::MoveTo(point) | SvgPathCommand::LineTo(point) => Some(point),
            _ => None,
        };
        if let Some(point) = point {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }
        i += 1;
    }
    if min_x <= max_x && min_y <= max_y {
        Some(Rect::from_edges(min_x, min_y, max_x + 1, max_y + 1))
    } else {
        None
    }
}

fn parse_fill_rule(value: Option<&str>) -> Option<SvgFillRule> {
    match value? {
        "evenodd" | "even-odd" => Some(SvgFillRule::EvenOdd),
        "nonzero" | "non-zero" => Some(SvgFillRule::NonZero),
        _ => None,
    }
}

fn parse_stroke_cap(value: Option<&str>) -> Option<SvgStrokeCap> {
    match value? {
        "butt" => Some(SvgStrokeCap::Butt),
        "square" => Some(SvgStrokeCap::Square),
        "round" => Some(SvgStrokeCap::Round),
        _ => None,
    }
}

fn parse_stroke_join(value: Option<&str>) -> Option<SvgStrokeJoin> {
    match value? {
        "miter" | "miter-clip" => Some(SvgStrokeJoin::Miter),
        "bevel" => Some(SvgStrokeJoin::Bevel),
        "round" => Some(SvgStrokeJoin::Round),
        _ => None,
    }
}

fn mul_opacity(a: u8, b: u8) -> u8 {
    ((a as u16 * b as u16) / 255) as u8
}

fn first_tag<'a>(data: &'a str, name: &str) -> Option<&'a str> {
    let start = data.find(name)?;
    let end = data.get(start..)?.find('>')?;
    data.get(start..start + end + 1)
}

fn inherited_group_state(data: &str, path_start: usize, root: SvgGroupState) -> SvgGroupState {
    let mut stack = [root; SVG_GROUP_STACK];
    let mut depth = 0usize;
    let mut search = 0usize;
    while search < path_start {
        let Some(relative) = data.get(search..path_start).and_then(|s| s.find('<')) else {
            break;
        };
        let start = search + relative;
        let Some(end_rel) = data.get(start..path_start).and_then(|s| s.find('>')) else {
            break;
        };
        let end = start + end_rel + 1;
        let Some(tag) = data.get(start..end) else {
            break;
        };

        if tag.starts_with("</g") {
            depth = depth.saturating_sub(1);
        } else if is_group_start_tag(tag) {
            let parent = if depth == 0 { root } else { stack[depth - 1] };
            let state = parent.inherit(data, tag);
            if depth < SVG_GROUP_STACK {
                stack[depth] = state;
                depth += 1;
            } else {
                stack[SVG_GROUP_STACK - 1] = state;
            }
            if tag.ends_with("/>") {
                depth = depth.saturating_sub(1);
            }
        }
        search = end;
    }

    if depth == 0 {
        root
    } else {
        stack[depth.min(SVG_GROUP_STACK) - 1]
    }
}

fn is_group_start_tag(tag: &str) -> bool {
    let bytes = tag.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'<' || bytes[1] != b'g' {
        return false;
    }
    matches!(
        bytes.get(2).copied(),
        Some(b' ' | b'\n' | b'\r' | b'\t' | b'>' | b'/')
    )
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let bytes = hex.as_bytes();
    if bytes.len() == 6 {
        Some(Color::rgb(
            hex_byte(bytes[0], bytes[1])?,
            hex_byte(bytes[2], bytes[3])?,
            hex_byte(bytes[4], bytes[5])?,
        ))
    } else {
        None
    }
}

fn hex_byte(a: u8, b: u8) -> Option<u8> {
    Some((hex_nibble(a)? << 4) | hex_nibble(b)?)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn parse_u16(value: &str) -> Option<u16> {
    let mut out = 0u16;
    let mut seen = false;
    for byte in value.as_bytes().iter().copied() {
        if byte.is_ascii_digit() {
            seen = true;
            out = out.saturating_mul(10).saturating_add((byte - b'0') as u16);
        } else {
            break;
        }
    }
    if seen {
        Some(out)
    } else {
        None
    }
}

fn parse_opacity(value: &str) -> Option<u8> {
    let bytes = value.as_bytes();
    if bytes.first() == Some(&b'0') && bytes.get(1) == Some(&b'.') {
        let mut v = 0u16;
        let mut denom = 1u16;
        let mut i = 2usize;
        while i < bytes.len() && bytes[i].is_ascii_digit() && denom < 1000 {
            v = v
                .saturating_mul(10)
                .saturating_add((bytes[i] - b'0') as u16);
            denom = denom.saturating_mul(10);
            i += 1;
        }
        Some(((v * 255) / denom.max(1)) as u8)
    } else {
        parse_u16(value).map(|v| v.min(255) as u8)
    }
}

fn point_for(cmd: u8, cursor: Point, x: i32, y: i32) -> Point {
    if cmd.is_ascii_lowercase() {
        Point::new(cursor.x.saturating_add(x), cursor.y.saturating_add(y))
    } else {
        Point::new(x, y)
    }
}

fn is_command(byte: u8) -> bool {
    matches!(
        byte,
        b'M' | b'm'
            | b'L'
            | b'l'
            | b'H'
            | b'h'
            | b'V'
            | b'v'
            | b'C'
            | b'c'
            | b'Q'
            | b'q'
            | b'A'
            | b'a'
            | b'S'
            | b's'
            | b'T'
            | b't'
            | b'Z'
            | b'z'
    )
}

fn skip_separators(bytes: &[u8], index: &mut usize) {
    while *index < bytes.len() && matches!(bytes[*index], b' ' | b'\n' | b'\r' | b'\t' | b',') {
        *index += 1;
    }
}

fn transform_path<const N: usize>(path: &mut SvgPath<N>, transform: SvgTransform) {
    if transform == SvgTransform::IDENTITY {
        return;
    }
    let mut i = 0usize;
    while i < path.len {
        path.commands[i] = match path.commands[i] {
            SvgPathCommand::MoveTo(point) => SvgPathCommand::MoveTo(transform.apply(point)),
            SvgPathCommand::LineTo(point) => SvgPathCommand::LineTo(transform.apply(point)),
            other => other,
        };
        i += 1;
    }
}

fn parse_transform(value: Option<&str>) -> Option<SvgTransform> {
    let value = value?;
    if let Some(args) = transform_args(value, "translate") {
        let mut index = 0usize;
        let x = parse_number(args.as_bytes(), &mut index, 1).unwrap_or(0);
        let y = parse_number(args.as_bytes(), &mut index, 1).unwrap_or(0);
        return Some(SvgTransform {
            e: x,
            f: y,
            ..SvgTransform::IDENTITY
        });
    }
    if let Some(args) = transform_args(value, "scale") {
        let mut index = 0usize;
        let sx = parse_number(args.as_bytes(), &mut index, 1024).unwrap_or(1024);
        let sy = parse_number(args.as_bytes(), &mut index, 1024).unwrap_or(sx);
        return Some(SvgTransform {
            a: sx,
            d: sy,
            ..SvgTransform::IDENTITY
        });
    }
    if let Some(args) = transform_args(value, "matrix") {
        let mut index = 0usize;
        let bytes = args.as_bytes();
        return Some(SvgTransform {
            a: parse_number(bytes, &mut index, 1024).unwrap_or(1024),
            b: parse_number(bytes, &mut index, 1024).unwrap_or(0),
            c: parse_number(bytes, &mut index, 1024).unwrap_or(0),
            d: parse_number(bytes, &mut index, 1024).unwrap_or(1024),
            e: parse_number(bytes, &mut index, 1).unwrap_or(0),
            f: parse_number(bytes, &mut index, 1).unwrap_or(0),
        });
    }
    None
}

fn transform_args<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    let start = value.find(name)? + name.len();
    let rest = value.get(start..)?.trim_start();
    let rest = rest.strip_prefix('(')?;
    let end = rest.find(')')?;
    rest.get(..end)
}

fn parse_number(bytes: &[u8], index: &mut usize, scale: i32) -> Option<i32> {
    skip_separators(bytes, index);
    if *index >= bytes.len() {
        return None;
    }

    let mut sign = 1;
    if bytes[*index] == b'-' {
        sign = -1;
        *index += 1;
    } else if bytes[*index] == b'+' {
        *index += 1;
    }

    let mut whole = 0i32;
    let mut seen = false;
    while *index < bytes.len() && bytes[*index].is_ascii_digit() {
        seen = true;
        whole = whole
            .saturating_mul(10)
            .saturating_add((bytes[*index] - b'0') as i32);
        *index += 1;
    }

    let mut frac = 0i32;
    let mut denom = 1i32;
    if *index < bytes.len() && bytes[*index] == b'.' {
        *index += 1;
        while *index < bytes.len() && bytes[*index].is_ascii_digit() {
            seen = true;
            if denom < 1_000_000 {
                frac = frac
                    .saturating_mul(10)
                    .saturating_add((bytes[*index] - b'0') as i32);
                denom = denom.saturating_mul(10);
            }
            *index += 1;
        }
    }

    if seen {
        let scale = scale.max(1);
        let scaled_frac = if denom > 1 {
            frac.saturating_mul(scale).saturating_add(denom / 2) / denom
        } else {
            0
        };
        Some(
            whole
                .saturating_mul(scale)
                .saturating_add(scaled_frac)
                .saturating_mul(sign),
        )
    } else {
        None
    }
}
