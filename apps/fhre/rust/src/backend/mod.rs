use crate::svg::SvgDocumentCacheStats;
use crate::{
    Color, DrawCommand, FrameStats, GlyphRunCacheStats, ImageCacheStats, ImageFit, ImageId,
    LayerSpec, MaskSpec, PixelFormat, PresentStats, Rect,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawTaskKind {
    Fill,
    Border,
    BoxShadow,
    Letter,
    Label,
    Image,
    Layer,
    Line,
    Arc,
    Triangle,
    MaskRect,
    MaskBitmap,
    Blur,
    Vector,
    ThreeD,
}

impl DrawTaskKind {
    pub const COUNT: usize = 15;

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Fill),
            1 => Some(Self::Border),
            2 => Some(Self::BoxShadow),
            3 => Some(Self::Letter),
            4 => Some(Self::Label),
            5 => Some(Self::Image),
            6 => Some(Self::Layer),
            7 => Some(Self::Line),
            8 => Some(Self::Arc),
            9 => Some(Self::Triangle),
            10 => Some(Self::MaskRect),
            11 => Some(Self::MaskBitmap),
            12 => Some(Self::Blur),
            13 => Some(Self::Vector),
            14 => Some(Self::ThreeD),
            _ => None,
        }
    }

    pub const fn short_label(self) -> &'static str {
        match self {
            Self::Fill => "FILL",
            Self::Border => "BORDER",
            Self::BoxShadow => "SHADOW",
            Self::Letter => "LETTER",
            Self::Label => "LABEL",
            Self::Image => "IMAGE",
            Self::Layer => "LAYER",
            Self::Line => "LINE",
            Self::Arc => "ARC",
            Self::Triangle => "TRI",
            Self::MaskRect => "MASK",
            Self::MaskBitmap => "BMASK",
            Self::Blur => "BLUR",
            Self::Vector => "VECTOR",
            Self::ThreeD => "3D",
        }
    }

    pub const fn as_index(self) -> usize {
        match self {
            Self::Fill => 0,
            Self::Border => 1,
            Self::BoxShadow => 2,
            Self::Letter => 3,
            Self::Label => 4,
            Self::Image => 5,
            Self::Layer => 6,
            Self::Line => 7,
            Self::Arc => 8,
            Self::Triangle => 9,
            Self::MaskRect => 10,
            Self::MaskBitmap => 11,
            Self::Blur => 12,
            Self::Vector => 13,
            Self::ThreeD => 14,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawFeatureFlags {
    bits: u32,
}

impl DrawFeatureFlags {
    pub const NONE: Self = Self { bits: 0 };
    pub const FILL: Self = Self { bits: 1 << 0 };
    pub const BORDER: Self = Self { bits: 1 << 1 };
    pub const BOX_SHADOW: Self = Self { bits: 1 << 2 };
    pub const LETTER: Self = Self { bits: 1 << 3 };
    pub const LABEL: Self = Self { bits: 1 << 4 };
    pub const IMAGE: Self = Self { bits: 1 << 5 };
    pub const LAYER: Self = Self { bits: 1 << 6 };
    pub const LINE: Self = Self { bits: 1 << 7 };
    pub const ARC: Self = Self { bits: 1 << 8 };
    pub const TRIANGLE: Self = Self { bits: 1 << 9 };
    pub const MASK_RECT: Self = Self { bits: 1 << 10 };
    pub const MASK_BITMAP: Self = Self { bits: 1 << 11 };
    pub const BLUR: Self = Self { bits: 1 << 12 };
    pub const VECTOR: Self = Self { bits: 1 << 13 };
    pub const THREE_D: Self = Self { bits: 1 << 14 };
    pub const ALL_SOFTWARE: Self = Self {
        bits: (1 << 15) - 1,
    };

    pub const fn bits(self) -> u32 {
        self.bits
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }

    pub const fn supports(self, kind: DrawTaskKind) -> bool {
        self.contains(Self::for_kind(kind))
    }

    pub const fn for_kind(kind: DrawTaskKind) -> Self {
        match kind {
            DrawTaskKind::Fill => Self::FILL,
            DrawTaskKind::Border => Self::BORDER,
            DrawTaskKind::BoxShadow => Self::BOX_SHADOW,
            DrawTaskKind::Letter => Self::LETTER,
            DrawTaskKind::Label => Self::LABEL,
            DrawTaskKind::Image => Self::IMAGE,
            DrawTaskKind::Layer => Self::LAYER,
            DrawTaskKind::Line => Self::LINE,
            DrawTaskKind::Arc => Self::ARC,
            DrawTaskKind::Triangle => Self::TRIANGLE,
            DrawTaskKind::MaskRect => Self::MASK_RECT,
            DrawTaskKind::MaskBitmap => Self::MASK_BITMAP,
            DrawTaskKind::Blur => Self::BLUR,
            DrawTaskKind::Vector => Self::VECTOR,
            DrawTaskKind::ThreeD => Self::THREE_D,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawTaskCounters {
    pub counts: [u32; DrawTaskKind::COUNT],
}

impl DrawTaskCounters {
    pub const fn new() -> Self {
        Self {
            counts: [0; DrawTaskKind::COUNT],
        }
    }

    pub fn increment(&mut self, kind: DrawTaskKind) {
        let index = kind.as_index();
        self.counts[index] = self.counts[index].saturating_add(1);
    }

    pub const fn get(&self, kind: DrawTaskKind) -> u32 {
        self.counts[kind.as_index()]
    }

    pub fn merge(&mut self, other: Self) {
        let mut index = 0usize;
        while index < DrawTaskKind::COUNT {
            self.counts[index] = self.counts[index].saturating_add(other.counts[index]);
            index += 1;
        }
    }
}

pub const DEFAULT_DRAW_CHAIN_OPS: usize = 64;
pub const DEFAULT_CODEC_PIPELINE_STAGES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawChainRunDescriptor {
    pub command_start: u16,
    pub command_end: u16,
    pub has_clip: bool,
    pub has_mask: bool,
    pub has_layer: bool,
    pub bounds: Rect,
}

impl DrawChainRunDescriptor {
    pub const EMPTY: Self = Self {
        command_start: 0,
        command_end: 0,
        has_clip: false,
        has_mask: false,
        has_layer: false,
        bounds: Rect::EMPTY,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawChainRunContract<const OPS: usize> {
    pub kinds: [DrawChainOpKind; OPS],
    pub len: usize,
    pub descriptor: DrawChainRunDescriptor,
}

impl<const OPS: usize> DrawChainRunContract<OPS> {
    pub const EMPTY: Self = Self {
        kinds: [DrawChainOpKind::FallbackRange; OPS],
        len: 0,
        descriptor: DrawChainRunDescriptor::EMPTY,
    };

    pub fn push_kind(&mut self, kind: DrawChainOpKind) -> bool {
        if self.len >= OPS {
            return false;
        }
        self.kinds[self.len] = kind;
        self.len += 1;
        true
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn set_descriptor(&mut self, descriptor: DrawChainRunDescriptor) {
        self.descriptor = descriptor;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawChainOpKind {
    SolidFill,
    AlphaFill,
    ImageBlit,
    ImageBlend,
    Clip,
    MaskEnter,
    MaskExit,
    LayerEnter,
    LayerExit,
    FallbackRange,
}

impl DrawChainOpKind {
    pub(crate) const fn is_fallback(self) -> bool {
        matches!(self, Self::FallbackRange)
    }

    pub(crate) const fn is_stateful(self) -> bool {
        matches!(
            self,
            Self::Clip | Self::MaskEnter | Self::MaskExit | Self::LayerEnter | Self::LayerExit
        )
    }

    pub(crate) const fn required_feature(self) -> DrawFeatureFlags {
        match self {
            Self::SolidFill | Self::AlphaFill => DrawFeatureFlags::FILL,
            Self::ImageBlit | Self::ImageBlend => DrawFeatureFlags::IMAGE,
            Self::MaskEnter | Self::MaskExit => DrawFeatureFlags::MASK_RECT,
            Self::LayerEnter | Self::LayerExit => DrawFeatureFlags::LAYER,
            Self::Clip | Self::FallbackRange => DrawFeatureFlags::NONE,
        }
    }

    pub(crate) const fn supports(self, capabilities: BackendCapabilities) -> bool {
        if !capabilities.draw_chain {
            return false;
        }
        if self.is_fallback() {
            return false;
        }
        if capabilities.chain_draw_features.bits() == DrawFeatureFlags::NONE.bits() {
            return false;
        }
        if !capabilities
            .chain_draw_features
            .contains(self.required_feature())
        {
            return false;
        }
        match self {
            Self::Clip => capabilities.clip_rect,
            Self::MaskEnter | Self::MaskExit => capabilities.mask,
            Self::LayerEnter | Self::LayerExit => capabilities.layers,
            Self::SolidFill | Self::AlphaFill | Self::ImageBlit | Self::ImageBlend => capabilities
                .chain_draw_features
                .contains(self.required_feature()),
            Self::FallbackRange => false,
        }
    }

    pub(crate) const fn is_alpha_render(self) -> bool {
        matches!(self, Self::AlphaFill | Self::ImageBlend)
    }

    pub(crate) const fn is_non_alpha_render(self) -> bool {
        matches!(
            self,
            Self::SolidFill | Self::ImageBlit | Self::FallbackRange
        )
    }

    pub(crate) const fn is_clip_marker(self) -> bool {
        matches!(self, Self::Clip)
    }

    pub(crate) const fn is_mask_marker(self) -> bool {
        matches!(self, Self::MaskEnter | Self::MaskExit)
    }

    pub(crate) const fn is_layer_marker(self) -> bool {
        matches!(self, Self::LayerEnter | Self::LayerExit)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawChainOpPayload {
    None,
    FillRect {
        rect: Rect,
        color: Color,
    },
    Image {
        rect: Rect,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
        tint: Option<Color>,
    },
    Clip {
        clip: Option<Rect>,
    },
    Mask {
        spec: MaskSpec,
    },
    Layer {
        rect: Rect,
        spec: LayerSpec,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawChainOp {
    pub kind: DrawChainOpKind,
    pub task: Option<DrawTaskKind>,
    pub bounds: Rect,
    pub clip: Option<Rect>,
    pub payload: DrawChainOpPayload,
    pub command_start: u16,
    pub command_len: u16,
}

impl DrawChainOp {
    pub const EMPTY: Self = Self {
        kind: DrawChainOpKind::FallbackRange,
        task: None,
        bounds: Rect::EMPTY,
        clip: None,
        payload: DrawChainOpPayload::None,
        command_start: 0,
        command_len: 0,
    };

    pub const fn new(
        kind: DrawChainOpKind,
        task: Option<DrawTaskKind>,
        bounds: Rect,
        clip: Option<Rect>,
        payload: DrawChainOpPayload,
        command_start: u16,
        command_len: u16,
    ) -> Self {
        Self {
            kind,
            task,
            bounds,
            clip,
            payload,
            command_start,
            command_len,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawChainStats {
    pub candidates: u32,
    pub ops: u32,
    pub submitted: u32,
    pub fallbacks: u32,
    pub unsupported: u32,
    pub overflows: u32,
    pub task_hits: DrawTaskCounters,
}

impl DrawChainStats {
    pub const fn new() -> Self {
        Self {
            candidates: 0,
            ops: 0,
            submitted: 0,
            fallbacks: 0,
            unsupported: 0,
            overflows: 0,
            task_hits: DrawTaskCounters::new(),
        }
    }

    pub fn record_op(&mut self, op: DrawChainOp) {
        if self.candidates == 0 {
            self.candidates = 1;
        }
        self.ops = self.ops.saturating_add(1);
        if op.kind == DrawChainOpKind::FallbackRange {
            self.fallbacks = self.fallbacks.saturating_add(1);
        }
        if let Some(task) = op.task {
            self.task_hits.increment(task);
        }
    }

    pub fn record_submitted(&mut self) {
        self.submitted = self.submitted.saturating_add(1);
    }

    pub fn record_unsupported(&mut self) {
        self.unsupported = self.unsupported.saturating_add(1);
        self.fallbacks = self.fallbacks.saturating_add(1);
    }

    pub fn record_overflow(&mut self) {
        if self.candidates == 0 {
            self.candidates = 1;
        }
        self.overflows = self.overflows.saturating_add(1);
        self.fallbacks = self.fallbacks.saturating_add(1);
    }

    pub fn merge(&mut self, other: Self) {
        self.candidates = self.candidates.saturating_add(other.candidates);
        self.ops = self.ops.saturating_add(other.ops);
        self.submitted = self.submitted.saturating_add(other.submitted);
        self.fallbacks = self.fallbacks.saturating_add(other.fallbacks);
        self.unsupported = self.unsupported.saturating_add(other.unsupported);
        self.overflows = self.overflows.saturating_add(other.overflows);
        self.task_hits.merge(other.task_hits);
    }

    pub fn top_task(self) -> Option<(DrawTaskKind, u32)> {
        let mut best_kind = None;
        let mut best_count = 0u32;
        let mut index = 0usize;
        while index < DrawTaskKind::COUNT {
            if let Some(kind) = DrawTaskKind::from_index(index) {
                let count = self.task_hits.get(kind);
                if count > best_count {
                    best_kind = Some(kind);
                    best_count = count;
                }
            }
            index += 1;
        }
        best_kind.map(|kind| (kind, best_count))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawChainCapabilities {
    pub supported: bool,
    pub max_ops: usize,
    pub features: DrawFeatureFlags,
}

impl DrawChainCapabilities {
    pub const NONE: Self = Self {
        supported: false,
        max_ops: 0,
        features: DrawFeatureFlags::NONE,
    };

    pub const fn new(supported: bool, max_ops: usize, features: DrawFeatureFlags) -> Self {
        Self {
            supported,
            max_ops,
            features,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawChainSubmitResult {
    Submitted,
    Unsupported,
    Fallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParallelDrawChainSubmitResult {
    Queued,
    Submitted,
    Unsupported,
    Fallback,
}

impl ParallelDrawChainSubmitResult {
    pub const fn as_draw_chain_result(self) -> DrawChainSubmitResult {
        match self {
            Self::Queued | Self::Submitted => DrawChainSubmitResult::Submitted,
            Self::Unsupported => DrawChainSubmitResult::Unsupported,
            Self::Fallback => DrawChainSubmitResult::Fallback,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawChain<const OPS: usize> {
    ops: [DrawChainOp; OPS],
    len: usize,
    overflowed: bool,
    stats: DrawChainStats,
}

impl<const OPS: usize> DrawChain<OPS> {
    pub const fn new() -> Self {
        Self {
            ops: [DrawChainOp::EMPTY; OPS],
            len: 0,
            overflowed: false,
            stats: DrawChainStats::new(),
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.overflowed = false;
        self.stats = DrawChainStats::new();
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub const fn stats(&self) -> DrawChainStats {
        self.stats
    }

    pub fn ops(&self) -> &[DrawChainOp] {
        &self.ops[..self.len]
    }

    pub fn push(&mut self, op: DrawChainOp) -> bool {
        if self.len >= OPS {
            self.overflowed = true;
            self.stats.record_overflow();
            return false;
        }
        self.ops[self.len] = op;
        self.len += 1;
        self.stats.record_op(op);
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecStageKind {
    Read,
    Inspect,
    Header,
    Entropy,
    Parse,
    Transform,
    Raster,
    ColorConvert,
    Pack,
    CacheInsert,
    Fallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecStagePlan {
    pub kind: CodecStageKind,
    pub hardware_candidate: bool,
    pub supported: bool,
}

impl CodecStagePlan {
    pub const EMPTY: Self = Self {
        kind: CodecStageKind::Fallback,
        hardware_candidate: false,
        supported: false,
    };

    pub const fn new(kind: CodecStageKind, hardware_candidate: bool, supported: bool) -> Self {
        Self {
            kind,
            hardware_candidate,
            supported,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecAcceleratorCapabilities {
    pub png: bool,
    pub jpeg: bool,
    pub fraw: bool,
    pub ttf: bool,
    pub svg: bool,
    pub max_stages: usize,
}

impl CodecAcceleratorCapabilities {
    pub const NONE: Self = Self {
        png: false,
        jpeg: false,
        fraw: false,
        ttf: false,
        svg: false,
        max_stages: 0,
    };

    pub const fn new(
        png: bool,
        jpeg: bool,
        fraw: bool,
        ttf: bool,
        svg: bool,
        max_stages: usize,
    ) -> Self {
        Self {
            png,
            jpeg,
            fraw,
            ttf,
            svg,
            max_stages,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecPipelineStats {
    pub candidates: u32,
    pub stages: u32,
    pub hardware_candidates: u32,
    pub fallbacks: u32,
    pub unsupported: u32,
    pub overflows: u32,
}

impl CodecPipelineStats {
    pub const fn new() -> Self {
        Self {
            candidates: 0,
            stages: 0,
            hardware_candidates: 0,
            fallbacks: 0,
            unsupported: 0,
            overflows: 0,
        }
    }

    pub fn record_stage(&mut self, stage: CodecStagePlan) {
        if self.candidates == 0 {
            self.candidates = 1;
        }
        self.stages = self.stages.saturating_add(1);
        if stage.hardware_candidate {
            self.hardware_candidates = self.hardware_candidates.saturating_add(1);
            if !stage.supported {
                self.unsupported = self.unsupported.saturating_add(1);
                self.fallbacks = self.fallbacks.saturating_add(1);
            }
        }
        if stage.kind == CodecStageKind::Fallback {
            self.fallbacks = self.fallbacks.saturating_add(1);
        }
    }

    pub fn record_overflow(&mut self) {
        if self.candidates == 0 {
            self.candidates = 1;
        }
        self.overflows = self.overflows.saturating_add(1);
        self.fallbacks = self.fallbacks.saturating_add(1);
    }

    pub fn merge(&mut self, other: Self) {
        self.candidates = self.candidates.saturating_add(other.candidates);
        self.stages = self.stages.saturating_add(other.stages);
        self.hardware_candidates = self
            .hardware_candidates
            .saturating_add(other.hardware_candidates);
        self.fallbacks = self.fallbacks.saturating_add(other.fallbacks);
        self.unsupported = self.unsupported.saturating_add(other.unsupported);
        self.overflows = self.overflows.saturating_add(other.overflows);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecPipelinePlan<const STAGES: usize> {
    stages: [CodecStagePlan; STAGES],
    len: usize,
    overflowed: bool,
    stats: CodecPipelineStats,
}

impl<const STAGES: usize> CodecPipelinePlan<STAGES> {
    pub const fn new() -> Self {
        Self {
            stages: [CodecStagePlan::EMPTY; STAGES],
            len: 0,
            overflowed: false,
            stats: CodecPipelineStats::new(),
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub const fn stats(&self) -> CodecPipelineStats {
        self.stats
    }

    pub fn stages(&self) -> &[CodecStagePlan] {
        &self.stages[..self.len]
    }

    pub fn push(&mut self, stage: CodecStagePlan) -> bool {
        if self.len >= STAGES {
            self.overflowed = true;
            self.stats.record_overflow();
            return false;
        }
        self.stages[self.len] = stage;
        self.len += 1;
        self.stats.record_stage(stage);
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecPipelineJobState {
    Planned,
    Prepared,
    Submitted,
    Completed,
    Fallback,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecPipelineJobError {
    InvalidTransition,
    Unsupported,
    Overflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecPipelineJobToken(pub u32);

impl CodecPipelineJobToken {
    pub const INVALID: Self = Self(0);
}

pub trait CodecPipelineBackend {
    fn capabilities(&self) -> CodecAcceleratorCapabilities;

    fn prepare_job<const STAGES: usize>(
        &mut self,
        job: &mut CodecPipelineJob<STAGES>,
    ) -> Result<(), CodecPipelineJobError>;

    fn submit_job<const STAGES: usize>(
        &mut self,
        job: &mut CodecPipelineJob<STAGES>,
    ) -> Result<(), CodecPipelineJobError>;

    fn complete_job<const STAGES: usize>(
        &mut self,
        job: &mut CodecPipelineJob<STAGES>,
    ) -> Result<(), CodecPipelineJobError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecPipelineJob<const STAGES: usize> {
    plan: CodecPipelinePlan<STAGES>,
    capabilities: CodecAcceleratorCapabilities,
    state: CodecPipelineJobState,
    failure: Option<CodecErrorKind>,
}

impl<const STAGES: usize> CodecPipelineJob<STAGES> {
    pub const fn new(
        plan: CodecPipelinePlan<STAGES>,
        capabilities: CodecAcceleratorCapabilities,
    ) -> Self {
        Self {
            plan,
            capabilities,
            state: CodecPipelineJobState::Planned,
            failure: None,
        }
    }

    pub const fn plan(&self) -> &CodecPipelinePlan<STAGES> {
        &self.plan
    }

    pub const fn capabilities(&self) -> CodecAcceleratorCapabilities {
        self.capabilities
    }

    pub const fn state(&self) -> CodecPipelineJobState {
        self.state
    }

    pub const fn failure(&self) -> Option<CodecErrorKind> {
        self.failure
    }

    pub const fn stats(&self) -> CodecPipelineStats {
        self.plan.stats()
    }

    pub fn prepare(&mut self) -> Result<(), CodecPipelineJobError> {
        if self.state != CodecPipelineJobState::Planned {
            return Err(CodecPipelineJobError::InvalidTransition);
        }

        let stats = self.plan.stats();
        if self.plan.overflowed() || stats.overflows > 0 {
            self.state = CodecPipelineJobState::Fallback;
            self.failure = Some(CodecErrorKind::Overflow);
            return Err(CodecPipelineJobError::Overflow);
        }
        if stats.hardware_candidates == 0 || stats.unsupported > 0 {
            self.state = CodecPipelineJobState::Fallback;
            self.failure = Some(CodecErrorKind::Unsupported);
            return Err(CodecPipelineJobError::Unsupported);
        }

        self.state = CodecPipelineJobState::Prepared;
        Ok(())
    }

    pub fn prepare_with<B: CodecPipelineBackend>(
        &mut self,
        backend: &mut B,
    ) -> Result<(), CodecPipelineJobError> {
        backend.prepare_job(self)
    }

    pub fn submit(&mut self) -> Result<(), CodecPipelineJobError> {
        if self.state != CodecPipelineJobState::Prepared {
            return Err(CodecPipelineJobError::InvalidTransition);
        }
        self.state = CodecPipelineJobState::Submitted;
        Ok(())
    }

    pub fn submit_with<B: CodecPipelineBackend>(
        &mut self,
        backend: &mut B,
    ) -> Result<(), CodecPipelineJobError> {
        backend.submit_job(self)
    }

    pub fn complete(&mut self) -> Result<(), CodecPipelineJobError> {
        if self.state != CodecPipelineJobState::Submitted {
            return Err(CodecPipelineJobError::InvalidTransition);
        }
        self.state = CodecPipelineJobState::Completed;
        Ok(())
    }

    pub fn complete_with<B: CodecPipelineBackend>(
        &mut self,
        backend: &mut B,
    ) -> Result<(), CodecPipelineJobError> {
        backend.complete_job(self)
    }

    pub fn fallback(&mut self, kind: CodecErrorKind) -> Result<(), CodecPipelineJobError> {
        match self.state {
            CodecPipelineJobState::Planned
            | CodecPipelineJobState::Prepared
            | CodecPipelineJobState::Submitted => {
                self.state = CodecPipelineJobState::Fallback;
                self.failure = Some(kind);
                Ok(())
            }
            CodecPipelineJobState::Completed
            | CodecPipelineJobState::Fallback
            | CodecPipelineJobState::Failed => Err(CodecPipelineJobError::InvalidTransition),
        }
    }

    pub fn fail(&mut self, kind: CodecErrorKind) -> Result<(), CodecPipelineJobError> {
        match self.state {
            CodecPipelineJobState::Prepared | CodecPipelineJobState::Submitted => {
                self.state = CodecPipelineJobState::Failed;
                self.failure = Some(kind);
                Ok(())
            }
            CodecPipelineJobState::Planned
            | CodecPipelineJobState::Completed
            | CodecPipelineJobState::Fallback
            | CodecPipelineJobState::Failed => Err(CodecPipelineJobError::InvalidTransition),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MockCodecBackendFailure {
    Prepare,
    Submit,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MockCodecBackend {
    capabilities: CodecAcceleratorCapabilities,
    prepare_calls: u32,
    submit_calls: u32,
    complete_calls: u32,
    fail_next: Option<MockCodecBackendFailure>,
    next_token: u32,
    submitted_token: Option<CodecPipelineJobToken>,
    completed_token: Option<CodecPipelineJobToken>,
}

impl MockCodecBackend {
    pub const fn new(capabilities: CodecAcceleratorCapabilities) -> Self {
        Self {
            capabilities,
            prepare_calls: 0,
            submit_calls: 0,
            complete_calls: 0,
            fail_next: None,
            next_token: 1,
            submitted_token: None,
            completed_token: None,
        }
    }

    pub const fn with_next_failure(mut self, failure: MockCodecBackendFailure) -> Self {
        self.fail_next = Some(failure);
        self
    }

    pub const fn prepare_calls(&self) -> u32 {
        self.prepare_calls
    }

    pub const fn submit_calls(&self) -> u32 {
        self.submit_calls
    }

    pub const fn complete_calls(&self) -> u32 {
        self.complete_calls
    }

    pub const fn submitted_token(&self) -> Option<CodecPipelineJobToken> {
        self.submitted_token
    }

    pub const fn completed_token(&self) -> Option<CodecPipelineJobToken> {
        self.completed_token
    }

    fn take_failure(&mut self, failure: MockCodecBackendFailure) -> bool {
        if self.fail_next == Some(failure) {
            self.fail_next = None;
            true
        } else {
            false
        }
    }
}

impl CodecPipelineBackend for MockCodecBackend {
    fn capabilities(&self) -> CodecAcceleratorCapabilities {
        self.capabilities
    }

    fn prepare_job<const STAGES: usize>(
        &mut self,
        job: &mut CodecPipelineJob<STAGES>,
    ) -> Result<(), CodecPipelineJobError> {
        self.prepare_calls = self.prepare_calls.saturating_add(1);
        if job.state() != CodecPipelineJobState::Planned {
            return Err(CodecPipelineJobError::InvalidTransition);
        }
        if self.take_failure(MockCodecBackendFailure::Prepare)
            || self.capabilities != job.capabilities()
        {
            job.fallback(CodecErrorKind::Unsupported)?;
            return Err(CodecPipelineJobError::Unsupported);
        }
        job.prepare()
    }

    fn submit_job<const STAGES: usize>(
        &mut self,
        job: &mut CodecPipelineJob<STAGES>,
    ) -> Result<(), CodecPipelineJobError> {
        self.submit_calls = self.submit_calls.saturating_add(1);
        if self.take_failure(MockCodecBackendFailure::Submit) {
            job.fail(CodecErrorKind::Unsupported)?;
            return Err(CodecPipelineJobError::Unsupported);
        }
        job.submit()?;
        let token = CodecPipelineJobToken(self.next_token);
        self.next_token = self.next_token.saturating_add(1);
        self.submitted_token = Some(token);
        self.completed_token = None;
        Ok(())
    }

    fn complete_job<const STAGES: usize>(
        &mut self,
        job: &mut CodecPipelineJob<STAGES>,
    ) -> Result<(), CodecPipelineJobError> {
        self.complete_calls = self.complete_calls.saturating_add(1);
        if self.take_failure(MockCodecBackendFailure::Complete) {
            job.fail(CodecErrorKind::Unsupported)?;
            return Err(CodecPipelineJobError::Unsupported);
        }
        let Some(token) = self.submitted_token else {
            return Err(CodecPipelineJobError::InvalidTransition);
        };
        job.complete()?;
        self.submitted_token = None;
        self.completed_token = Some(token);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendCapabilities {
    pub pixel_format: PixelFormat,
    pub software: bool,
    pub accelerated_2d: bool,
    pub accelerated_3d: bool,
    pub clip_rect: bool,
    pub alpha_blend: bool,
    pub blend_modes: bool,
    pub gradients: bool,
    pub mask: bool,
    pub layers: bool,
    pub blur: bool,
    pub arc: bool,
    pub vector_icons: bool,
    pub vector_fill: bool,
    pub image_transform: bool,
    pub text_layout: bool,
    pub triangles: bool,
    pub draw_fill: bool,
    pub draw_border: bool,
    pub draw_box_shadow: bool,
    pub draw_letter: bool,
    pub draw_label: bool,
    pub draw_image: bool,
    pub draw_layer: bool,
    pub draw_line: bool,
    pub draw_arc: bool,
    pub draw_triangle: bool,
    pub draw_mask_rect: bool,
    pub draw_mask_bitmap: bool,
    pub draw_blur: bool,
    pub draw_vector: bool,
    pub draw_3d: bool,
    pub software_draw_features: DrawFeatureFlags,
    pub accelerated_draw_features: DrawFeatureFlags,
    pub draw_chain: bool,
    pub chain_continuous_submit: bool,
    pub chain_run_fence: bool,
    pub chain_mix_alpha: bool,
    pub chain_mix_clip: bool,
    pub chain_mix_mask: bool,
    pub chain_mix_layer: bool,
    pub max_chain_ops: usize,
    pub chain_draw_features: DrawFeatureFlags,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawPathKind {
    Software,
    Accelerated,
    Fallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawBackendDispatch {
    pub software_features: DrawFeatureFlags,
    pub accelerated_features: DrawFeatureFlags,
}

impl DrawBackendDispatch {
    pub const fn from_capabilities(capabilities: BackendCapabilities) -> Self {
        Self {
            software_features: capabilities.software_draw_features,
            accelerated_features: capabilities.accelerated_draw_features,
        }
    }

    pub const fn classify(self, kind: DrawTaskKind) -> DrawPathKind {
        if self.accelerated_features.supports(kind) {
            DrawPathKind::Accelerated
        } else if self.software_features.supports(kind) {
            DrawPathKind::Software
        } else {
            DrawPathKind::Fallback
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderBenchmarkSummary {
    pub draw_us: u32,
    pub present_us: u32,
    pub dirty_passes: u32,
    pub dirty_copy_bytes: u32,
    pub present_bytes: u32,
    pub top_dispatch_task: Option<DrawTaskKind>,
    pub top_dispatch_hits: u32,
    pub top_fallback_task: Option<DrawTaskKind>,
    pub top_fallback_hits: u32,
}

impl RenderBenchmarkSummary {
    pub const fn new() -> Self {
        Self {
            draw_us: 0,
            present_us: 0,
            dirty_passes: 0,
            dirty_copy_bytes: 0,
            present_bytes: 0,
            top_dispatch_task: None,
            top_dispatch_hits: 0,
            top_fallback_task: None,
            top_fallback_hits: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecErrorKind {
    MissingResource,
    Invalid,
    Truncated,
    Unsupported,
    Overflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CodecStats {
    pub missing_resources: u32,
    pub invalid: u32,
    pub truncated: u32,
    pub unsupported: u32,
    pub overflow: u32,
    pub pipeline_candidates: u32,
    pub pipeline_stages: u32,
    pub pipeline_hardware_candidates: u32,
    pub pipeline_fallbacks: u32,
    pub pipeline_unsupported: u32,
    pub pipeline_overflows: u32,
}

impl CodecStats {
    pub const fn new() -> Self {
        Self {
            missing_resources: 0,
            invalid: 0,
            truncated: 0,
            unsupported: 0,
            overflow: 0,
            pipeline_candidates: 0,
            pipeline_stages: 0,
            pipeline_hardware_candidates: 0,
            pipeline_fallbacks: 0,
            pipeline_unsupported: 0,
            pipeline_overflows: 0,
        }
    }

    pub const fn total_failures(self) -> u32 {
        self.missing_resources
            .saturating_add(self.invalid)
            .saturating_add(self.truncated)
            .saturating_add(self.unsupported)
            .saturating_add(self.overflow)
    }

    pub fn record(&mut self, kind: CodecErrorKind) {
        match kind {
            CodecErrorKind::MissingResource => {
                self.missing_resources = self.missing_resources.saturating_add(1);
            }
            CodecErrorKind::Invalid => {
                self.invalid = self.invalid.saturating_add(1);
            }
            CodecErrorKind::Truncated => {
                self.truncated = self.truncated.saturating_add(1);
            }
            CodecErrorKind::Unsupported => {
                self.unsupported = self.unsupported.saturating_add(1);
            }
            CodecErrorKind::Overflow => {
                self.overflow = self.overflow.saturating_add(1);
            }
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.missing_resources = self
            .missing_resources
            .saturating_add(other.missing_resources);
        self.invalid = self.invalid.saturating_add(other.invalid);
        self.truncated = self.truncated.saturating_add(other.truncated);
        self.unsupported = self.unsupported.saturating_add(other.unsupported);
        self.overflow = self.overflow.saturating_add(other.overflow);
        self.pipeline_candidates = self
            .pipeline_candidates
            .saturating_add(other.pipeline_candidates);
        self.pipeline_stages = self.pipeline_stages.saturating_add(other.pipeline_stages);
        self.pipeline_hardware_candidates = self
            .pipeline_hardware_candidates
            .saturating_add(other.pipeline_hardware_candidates);
        self.pipeline_fallbacks = self
            .pipeline_fallbacks
            .saturating_add(other.pipeline_fallbacks);
        self.pipeline_unsupported = self
            .pipeline_unsupported
            .saturating_add(other.pipeline_unsupported);
        self.pipeline_overflows = self
            .pipeline_overflows
            .saturating_add(other.pipeline_overflows);
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderStats {
    pub frame: u32,
    pub commands_seen: u32,
    pub commands_drawn: u32,
    pub commands_clipped: u32,
    pub blend_commands: u32,
    pub mask_commands: u32,
    pub layer_commands: u32,
    pub blur_commands: u32,
    pub vector_commands: u32,
    pub text_commands: u32,
    pub image_commands: u32,
    pub three_d_commands: u32,
    pub fill_commands: u32,
    pub border_commands: u32,
    pub shadow_commands: u32,
    pub line_commands: u32,
    pub arc_commands: u32,
    pub triangle_commands: u32,
    pub mask_rect_commands: u32,
    pub mask_bitmap_commands: u32,
    pub letter_commands: u32,
    pub label_commands: u32,
    pub fallback_count: u32,
    pub scratch_bytes: u32,
    pub mask_bytes: u32,
    pub layer_bytes: u32,
    pub scratch_peak_bytes: u32,
    pub fast_path_hits: u32,
    pub layer_alloc_failures: u32,
    pub mask_stack_overflows: u32,
    pub blur_pixels: u32,
    pub shadow_pixels: u32,
    pub layer_fallbacks: u32,
    pub clip_changes: u32,
    pub effective_clip_changes: u32,
    pub dirty_rects: u32,
    pub dirty_passes: u32,
    pub dirty_copy_bytes: u32,
    pub pixels_estimate: u32,
    pub presents: u32,
    pub cache_hits: u32,
    pub cache_misses: u32,
    pub cache_loads: u32,
    pub cache_bytes: u32,
    pub cache_slots: u32,
    pub cache_pinned: u32,
    pub cache_load_failures: u32,
    pub cache_decode_failures: u32,
    pub cache_decode_invalid: u32,
    pub cache_decode_truncated: u32,
    pub cache_decode_unsupported: u32,
    pub cache_decode_overflow: u32,
    pub cache_decode_placeholders: u32,
    pub cache_decode_placeholder_missing: u32,
    pub cache_decode_placeholder_invalid: u32,
    pub cache_decode_placeholder_truncated: u32,
    pub cache_decode_placeholder_unsupported: u32,
    pub cache_decode_placeholder_overflow: u32,
    pub cache_evictions: u32,
    pub codec_fallbacks: u32,
    pub codec_missing_resources: u32,
    pub codec_invalid: u32,
    pub codec_truncated: u32,
    pub codec_unsupported: u32,
    pub codec_overflow: u32,
    pub progressive_jpeg_decodes: u32,
    pub progressive_jpeg_scan_fallbacks: u32,
    pub cff_raster_glyphs: u32,
    pub cff_fallback_glyphs: u32,
    pub opentype_shaping_runs: u32,
    pub opentype_gsub_hits: u32,
    pub opentype_gpos_hits: u32,
    pub opentype_kern_hits: u32,
    pub svg_clip_paths: u32,
    pub svg_masks: u32,
    pub svg_filters: u32,
    pub svg_gradients: u32,
    pub svg_real_clip_paths: u32,
    pub svg_real_masks: u32,
    pub svg_filter_fallbacks: u32,
    pub svg_gradient_fallbacks: u32,
    pub vector_mask_rasters: u32,
    pub vector_mask_scratch_overflows: u32,
    pub text_layout_overflows: u32,
    pub text_shaped_layouts: u32,
    pub text_shaping_fallbacks: u32,
    pub glyph_run_cache_hits: u32,
    pub glyph_run_cache_misses: u32,
    pub glyph_run_cache_inserts: u32,
    pub glyph_run_cache_evictions: u32,
    pub glyph_run_cache_overflows: u32,
    pub glyph_run_cache_slots: u32,
    pub svg_doc_cache_hits: u32,
    pub svg_doc_cache_misses: u32,
    pub svg_doc_cache_loads: u32,
    pub svg_doc_cache_fallbacks: u32,
    pub svg_doc_cache_slots: u32,
    pub glyph_id_draw_hits: u32,
    pub codepoint_fallbacks: u32,
    pub selection_fallbacks: u32,
    pub svg_filter_budget_exceeded: u32,
    pub draw_task_fallbacks: u32,
    pub software_path_hits: u32,
    pub accelerated_path_hits: u32,
    pub fallback_path_hits: u32,
    pub draw_chain_candidates: u32,
    pub draw_chain_ops: u32,
    pub draw_chain_submitted: u32,
    pub draw_chain_fallbacks: u32,
    pub draw_chain_unsupported: u32,
    pub draw_chain_overflows: u32,
    pub draw_chain_runs: u32,
    pub draw_chain_hw_runs: u32,
    pub draw_chain_sw_runs: u32,
    pub draw_chain_splits: u32,
    pub draw_chain_parallel_hints: u32,
    pub draw_chain_parallel_queued: u32,
    pub draw_chain_parallel_completed: u32,
    pub draw_chain_parallel_barriers: u32,
    pub draw_chain_parallel_software_runs: u32,
    pub draw_chain_parallel_fallbacks: u32,
    pub accel2d_ring_submissions: u32,
    pub accel2d_ring_flushes: u32,
    pub accel2d_fences_issued: u32,
    pub accel2d_fences_completed: u32,
    pub accel2d_ring_overflows: u32,
    pub codec_prewarm_parallel_hints: u32,
    pub task_draw_chain_hits: DrawTaskCounters,
    pub codec_pipeline_candidates: u32,
    pub codec_pipeline_stages: u32,
    pub codec_pipeline_hardware_candidates: u32,
    pub codec_pipeline_fallbacks: u32,
    pub codec_pipeline_unsupported: u32,
    pub codec_pipeline_overflows: u32,
    pub task_software_hits: DrawTaskCounters,
    pub task_accelerated_hits: DrawTaskCounters,
    pub task_fallback_hits: DrawTaskCounters,
    pub draw_us: u32,
    pub present_us: u32,
    pub input_events: u32,
    pub dropped_frames: u32,
    pub late_frames: u32,
    pub fps_x1000: u32,
    pub pan_presents: u32,
    pub copy_presents: u32,
    pub skipped_presents: u32,
    pub present_bytes: u32,
    pub overflowed: bool,
}

impl RenderStats {
    pub const fn new() -> Self {
        Self {
            frame: 0,
            commands_seen: 0,
            commands_drawn: 0,
            commands_clipped: 0,
            blend_commands: 0,
            mask_commands: 0,
            layer_commands: 0,
            blur_commands: 0,
            vector_commands: 0,
            text_commands: 0,
            image_commands: 0,
            three_d_commands: 0,
            fill_commands: 0,
            border_commands: 0,
            shadow_commands: 0,
            line_commands: 0,
            arc_commands: 0,
            triangle_commands: 0,
            mask_rect_commands: 0,
            mask_bitmap_commands: 0,
            letter_commands: 0,
            label_commands: 0,
            fallback_count: 0,
            scratch_bytes: 0,
            mask_bytes: 0,
            layer_bytes: 0,
            scratch_peak_bytes: 0,
            fast_path_hits: 0,
            layer_alloc_failures: 0,
            mask_stack_overflows: 0,
            blur_pixels: 0,
            shadow_pixels: 0,
            layer_fallbacks: 0,
            clip_changes: 0,
            effective_clip_changes: 0,
            dirty_rects: 0,
            dirty_passes: 0,
            dirty_copy_bytes: 0,
            pixels_estimate: 0,
            presents: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_loads: 0,
            cache_bytes: 0,
            cache_slots: 0,
            cache_pinned: 0,
            cache_load_failures: 0,
            cache_decode_failures: 0,
            cache_decode_invalid: 0,
            cache_decode_truncated: 0,
            cache_decode_unsupported: 0,
            cache_decode_overflow: 0,
            cache_decode_placeholders: 0,
            cache_decode_placeholder_missing: 0,
            cache_decode_placeholder_invalid: 0,
            cache_decode_placeholder_truncated: 0,
            cache_decode_placeholder_unsupported: 0,
            cache_decode_placeholder_overflow: 0,
            cache_evictions: 0,
            codec_fallbacks: 0,
            codec_missing_resources: 0,
            codec_invalid: 0,
            codec_truncated: 0,
            codec_unsupported: 0,
            codec_overflow: 0,
            progressive_jpeg_decodes: 0,
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
            text_layout_overflows: 0,
            text_shaped_layouts: 0,
            text_shaping_fallbacks: 0,
            glyph_run_cache_hits: 0,
            glyph_run_cache_misses: 0,
            glyph_run_cache_inserts: 0,
            glyph_run_cache_evictions: 0,
            glyph_run_cache_overflows: 0,
            glyph_run_cache_slots: 0,
            svg_doc_cache_hits: 0,
            svg_doc_cache_misses: 0,
            svg_doc_cache_loads: 0,
            svg_doc_cache_fallbacks: 0,
            svg_doc_cache_slots: 0,
            glyph_id_draw_hits: 0,
            codepoint_fallbacks: 0,
            selection_fallbacks: 0,
            svg_filter_budget_exceeded: 0,
            draw_task_fallbacks: 0,
            software_path_hits: 0,
            accelerated_path_hits: 0,
            fallback_path_hits: 0,
            draw_chain_candidates: 0,
            draw_chain_ops: 0,
            draw_chain_submitted: 0,
            draw_chain_fallbacks: 0,
            draw_chain_unsupported: 0,
            draw_chain_overflows: 0,
            draw_chain_runs: 0,
            draw_chain_hw_runs: 0,
            draw_chain_sw_runs: 0,
            draw_chain_splits: 0,
            draw_chain_parallel_hints: 0,
            draw_chain_parallel_queued: 0,
            draw_chain_parallel_completed: 0,
            draw_chain_parallel_barriers: 0,
            draw_chain_parallel_software_runs: 0,
            draw_chain_parallel_fallbacks: 0,
            accel2d_ring_submissions: 0,
            accel2d_ring_flushes: 0,
            accel2d_fences_issued: 0,
            accel2d_fences_completed: 0,
            accel2d_ring_overflows: 0,
            codec_prewarm_parallel_hints: 0,
            task_draw_chain_hits: DrawTaskCounters::new(),
            codec_pipeline_candidates: 0,
            codec_pipeline_stages: 0,
            codec_pipeline_hardware_candidates: 0,
            codec_pipeline_fallbacks: 0,
            codec_pipeline_unsupported: 0,
            codec_pipeline_overflows: 0,
            task_software_hits: DrawTaskCounters::new(),
            task_accelerated_hits: DrawTaskCounters::new(),
            task_fallback_hits: DrawTaskCounters::new(),
            draw_us: 0,
            present_us: 0,
            input_events: 0,
            dropped_frames: 0,
            late_frames: 0,
            fps_x1000: 0,
            pan_presents: 0,
            copy_presents: 0,
            skipped_presents: 0,
            present_bytes: 0,
            overflowed: false,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn mark_dirty_rects(&mut self, dirty_rects: usize) {
        self.dirty_rects = dirty_rects.min(u32::MAX as usize) as u32;
    }

    pub fn dirty_pass_started(&mut self) {
        self.dirty_passes = self.dirty_passes.saturating_add(1);
    }

    pub fn mark_dirty_copy_bytes(&mut self, bytes: u32) {
        self.dirty_copy_bytes = self.dirty_copy_bytes.saturating_add(bytes);
    }

    pub fn mark_overflowed(&mut self, overflowed: bool) {
        self.overflowed |= overflowed;
    }

    pub fn mark_frame(&mut self, frame: u32) {
        self.frame = frame;
    }

    pub fn presented(&mut self) {
        self.presents = self.presents.saturating_add(1);
    }

    pub fn mark_present_stats(&mut self, present: PresentStats) {
        self.presents = self.presents.saturating_add(present.presents);
        self.pan_presents = self.pan_presents.saturating_add(present.pan_presents);
        self.copy_presents = self.copy_presents.saturating_add(present.copy_presents);
        self.skipped_presents = self
            .skipped_presents
            .saturating_add(present.skipped_presents);
        self.present_bytes = self.present_bytes.saturating_add(present.bytes_copied);
    }

    pub fn mark_frame_stats(&mut self, frame: FrameStats) {
        self.frame = frame.frame;
        self.draw_us = frame.draw_us;
        self.present_us = frame.present_us;
        self.input_events = frame.input_events;
        self.dropped_frames = frame.dropped_frames;
        self.late_frames = frame.late_frames;
        self.fps_x1000 = frame.fps_x1000;
        self.mark_present_stats(frame.present);
    }

    pub fn mark_cache(&mut self, hits: u32, misses: u32, bytes: usize) {
        self.cache_hits = hits;
        self.cache_misses = misses;
        self.cache_bytes = bytes.min(u32::MAX as usize) as u32;
    }

    pub fn mark_image_cache(&mut self, cache: ImageCacheStats) {
        self.cache_hits = cache.hits;
        self.cache_misses = cache.misses;
        self.cache_loads = cache.loads;
        self.cache_bytes = cache.bytes.min(u32::MAX as usize) as u32;
        self.cache_slots = cache.slots.min(u32::MAX as usize) as u32;
        self.cache_pinned = cache.pinned.min(u32::MAX as usize) as u32;
        self.cache_load_failures = cache.load_failures;
        self.cache_decode_failures = cache.decode_failures;
        self.cache_decode_invalid = cache.decode_invalid;
        self.cache_decode_truncated = cache.decode_truncated;
        self.cache_decode_unsupported = cache.decode_unsupported;
        self.cache_decode_overflow = cache.decode_overflow;
        self.cache_decode_placeholders = cache.decode_placeholders;
        self.cache_decode_placeholder_missing = cache.decode_placeholder_missing;
        self.cache_decode_placeholder_invalid = cache.decode_placeholder_invalid;
        self.cache_decode_placeholder_truncated = cache.decode_placeholder_truncated;
        self.cache_decode_placeholder_unsupported = cache.decode_placeholder_unsupported;
        self.cache_decode_placeholder_overflow = cache.decode_placeholder_overflow;
        self.cache_evictions = cache.evictions;
        self.codec_pipeline_candidates = cache.pipeline_candidates;
        self.codec_pipeline_stages = cache.pipeline_stages;
        self.codec_pipeline_hardware_candidates = cache.pipeline_hardware_candidates;
        self.codec_pipeline_fallbacks = cache.pipeline_fallbacks;
        self.codec_pipeline_unsupported = cache.pipeline_unsupported;
        self.codec_pipeline_overflows = cache.pipeline_overflows;
        self.codec_missing_resources = cache.load_failures;
        self.codec_invalid = cache.decode_invalid;
        self.codec_truncated = cache.decode_truncated;
        self.codec_unsupported = cache.decode_unsupported;
        self.codec_overflow = cache.decode_overflow;
        self.codec_fallbacks = self
            .codec_missing_resources
            .saturating_add(self.codec_invalid)
            .saturating_add(self.codec_truncated)
            .saturating_add(self.codec_unsupported)
            .saturating_add(self.codec_overflow);
    }

    pub fn mark_glyph_run_cache(&mut self, cache: GlyphRunCacheStats) {
        self.glyph_run_cache_hits = cache.hits;
        self.glyph_run_cache_misses = cache.misses;
        self.glyph_run_cache_inserts = cache.inserts;
        self.glyph_run_cache_evictions = cache.evictions;
        self.glyph_run_cache_overflows = cache.overflows;
        self.glyph_run_cache_slots = cache.slots.min(u32::MAX as usize) as u32;
    }

    pub fn mark_svg_document_cache(&mut self, cache: SvgDocumentCacheStats) {
        self.svg_doc_cache_hits = cache.hits;
        self.svg_doc_cache_misses = cache.misses;
        self.svg_doc_cache_loads = cache.loads;
        self.svg_doc_cache_fallbacks = cache.fallbacks();
        self.svg_doc_cache_slots = cache.slots.min(u32::MAX as usize) as u32;
        self.codec_pipeline_candidates = self
            .codec_pipeline_candidates
            .saturating_add(cache.pipeline_candidates);
        self.codec_pipeline_stages = self
            .codec_pipeline_stages
            .saturating_add(cache.pipeline_stages);
        self.codec_pipeline_hardware_candidates = self
            .codec_pipeline_hardware_candidates
            .saturating_add(cache.pipeline_hardware_candidates);
        self.codec_pipeline_fallbacks = self
            .codec_pipeline_fallbacks
            .saturating_add(cache.pipeline_fallbacks);
        self.codec_pipeline_unsupported = self
            .codec_pipeline_unsupported
            .saturating_add(cache.pipeline_unsupported);
        self.codec_pipeline_overflows = self
            .codec_pipeline_overflows
            .saturating_add(cache.pipeline_overflows);
    }

    pub fn mark_codec_error(&mut self, kind: CodecErrorKind) {
        match kind {
            CodecErrorKind::MissingResource => {
                self.codec_missing_resources = self.codec_missing_resources.saturating_add(1);
            }
            CodecErrorKind::Invalid => {
                self.codec_invalid = self.codec_invalid.saturating_add(1);
            }
            CodecErrorKind::Truncated => {
                self.codec_truncated = self.codec_truncated.saturating_add(1);
            }
            CodecErrorKind::Unsupported => {
                self.codec_unsupported = self.codec_unsupported.saturating_add(1);
            }
            CodecErrorKind::Overflow => {
                self.codec_overflow = self.codec_overflow.saturating_add(1);
            }
        }
        self.codec_fallbacks = self.codec_fallbacks.saturating_add(1);
    }

    pub fn mark_codec_stats(&mut self, stats: CodecStats) {
        self.codec_missing_resources = self
            .codec_missing_resources
            .saturating_add(stats.missing_resources);
        self.codec_invalid = self.codec_invalid.saturating_add(stats.invalid);
        self.codec_truncated = self.codec_truncated.saturating_add(stats.truncated);
        self.codec_unsupported = self.codec_unsupported.saturating_add(stats.unsupported);
        self.codec_overflow = self.codec_overflow.saturating_add(stats.overflow);
        self.codec_fallbacks = self.codec_fallbacks.saturating_add(stats.total_failures());
        self.codec_pipeline_candidates = self
            .codec_pipeline_candidates
            .saturating_add(stats.pipeline_candidates);
        self.codec_pipeline_stages = self
            .codec_pipeline_stages
            .saturating_add(stats.pipeline_stages);
        self.codec_pipeline_hardware_candidates = self
            .codec_pipeline_hardware_candidates
            .saturating_add(stats.pipeline_hardware_candidates);
        self.codec_pipeline_fallbacks = self
            .codec_pipeline_fallbacks
            .saturating_add(stats.pipeline_fallbacks);
        self.codec_pipeline_unsupported = self
            .codec_pipeline_unsupported
            .saturating_add(stats.pipeline_unsupported);
        self.codec_pipeline_overflows = self
            .codec_pipeline_overflows
            .saturating_add(stats.pipeline_overflows);
    }

    pub fn mark_codec_pipeline(&mut self, stats: CodecPipelineStats) {
        self.codec_pipeline_candidates = self
            .codec_pipeline_candidates
            .saturating_add(stats.candidates);
        self.codec_pipeline_stages = self.codec_pipeline_stages.saturating_add(stats.stages);
        self.codec_pipeline_hardware_candidates = self
            .codec_pipeline_hardware_candidates
            .saturating_add(stats.hardware_candidates);
        self.codec_pipeline_fallbacks = self
            .codec_pipeline_fallbacks
            .saturating_add(stats.fallbacks);
        self.codec_pipeline_unsupported = self
            .codec_pipeline_unsupported
            .saturating_add(stats.unsupported);
        self.codec_pipeline_overflows = self
            .codec_pipeline_overflows
            .saturating_add(stats.overflows);
    }

    pub fn mark_progressive_jpeg(&mut self) {
        self.progressive_jpeg_decodes = self.progressive_jpeg_decodes.saturating_add(1);
    }

    pub fn mark_progressive_jpeg_scan_fallback(&mut self) {
        self.progressive_jpeg_scan_fallbacks =
            self.progressive_jpeg_scan_fallbacks.saturating_add(1);
    }

    pub fn mark_cff_raster_glyph(&mut self) {
        self.cff_raster_glyphs = self.cff_raster_glyphs.saturating_add(1);
    }

    pub fn mark_cff_fallback_glyph(&mut self) {
        self.cff_fallback_glyphs = self.cff_fallback_glyphs.saturating_add(1);
    }

    pub fn mark_opentype_shaping_run(&mut self) {
        self.opentype_shaping_runs = self.opentype_shaping_runs.saturating_add(1);
    }

    pub fn mark_opentype_gsub_hit(&mut self) {
        self.opentype_gsub_hits = self.opentype_gsub_hits.saturating_add(1);
    }

    pub fn mark_opentype_gpos_hit(&mut self) {
        self.opentype_gpos_hits = self.opentype_gpos_hits.saturating_add(1);
    }

    pub fn mark_opentype_kern_hit(&mut self) {
        self.opentype_kern_hits = self.opentype_kern_hits.saturating_add(1);
    }

    pub fn mark_svg_clip_path(&mut self) {
        self.svg_clip_paths = self.svg_clip_paths.saturating_add(1);
    }

    pub fn mark_svg_mask(&mut self) {
        self.svg_masks = self.svg_masks.saturating_add(1);
    }

    pub fn mark_svg_filter(&mut self) {
        self.svg_filters = self.svg_filters.saturating_add(1);
    }

    pub fn mark_svg_gradient(&mut self) {
        self.svg_gradients = self.svg_gradients.saturating_add(1);
    }

    pub fn mark_svg_real_clip_path(&mut self) {
        self.svg_real_clip_paths = self.svg_real_clip_paths.saturating_add(1);
    }

    pub fn mark_svg_real_mask(&mut self) {
        self.svg_real_masks = self.svg_real_masks.saturating_add(1);
    }

    pub fn mark_svg_filter_fallback(&mut self) {
        self.svg_filter_fallbacks = self.svg_filter_fallbacks.saturating_add(1);
    }

    pub fn mark_svg_gradient_fallback(&mut self) {
        self.svg_gradient_fallbacks = self.svg_gradient_fallbacks.saturating_add(1);
    }

    pub fn mark_vector_mask_raster(&mut self) {
        self.vector_mask_rasters = self.vector_mask_rasters.saturating_add(1);
    }

    pub fn mark_vector_mask_scratch_overflow(&mut self) {
        self.vector_mask_scratch_overflows = self.vector_mask_scratch_overflows.saturating_add(1);
    }

    pub fn mark_text_layout_overflow(&mut self) {
        self.text_layout_overflows = self.text_layout_overflows.saturating_add(1);
    }

    pub fn mark_text_shaped_layout(&mut self) {
        self.text_shaped_layouts = self.text_shaped_layouts.saturating_add(1);
    }

    pub fn mark_text_shaping_fallback(&mut self) {
        self.text_shaping_fallbacks = self.text_shaping_fallbacks.saturating_add(1);
    }

    pub fn mark_glyph_id_draw_hit(&mut self) {
        self.glyph_id_draw_hits = self.glyph_id_draw_hits.saturating_add(1);
    }

    pub fn mark_codepoint_fallback(&mut self) {
        self.codepoint_fallbacks = self.codepoint_fallbacks.saturating_add(1);
    }

    pub fn mark_selection_fallback(&mut self) {
        self.selection_fallbacks = self.selection_fallbacks.saturating_add(1);
    }

    pub fn mark_svg_filter_budget_exceeded(&mut self) {
        self.svg_filter_budget_exceeded = self.svg_filter_budget_exceeded.saturating_add(1);
    }

    pub fn dispatch_hits_for(&self, kind: DrawTaskKind) -> u32 {
        self.task_software_hits
            .get(kind)
            .saturating_add(self.task_accelerated_hits.get(kind))
            .saturating_add(self.task_fallback_hits.get(kind))
    }

    pub fn fallback_hits_for(&self, kind: DrawTaskKind) -> u32 {
        self.task_fallback_hits.get(kind)
    }

    pub fn top_dispatch_task(&self) -> Option<(DrawTaskKind, u32)> {
        let mut best_kind = None;
        let mut best_count = 0u32;
        let mut index = 0usize;
        while index < DrawTaskKind::COUNT {
            if let Some(kind) = DrawTaskKind::from_index(index) {
                let count = self.dispatch_hits_for(kind);
                if count > best_count {
                    best_count = count;
                    best_kind = Some(kind);
                }
            }
            index += 1;
        }
        best_kind.map(|kind| (kind, best_count))
    }

    pub fn top_fallback_task(&self) -> Option<(DrawTaskKind, u32)> {
        let mut best_kind = None;
        let mut best_count = 0u32;
        let mut index = 0usize;
        while index < DrawTaskKind::COUNT {
            if let Some(kind) = DrawTaskKind::from_index(index) {
                let count = self.fallback_hits_for(kind);
                if count > best_count {
                    best_count = count;
                    best_kind = Some(kind);
                }
            }
            index += 1;
        }
        best_kind.map(|kind| (kind, best_count))
    }

    pub fn top_chain_task(&self) -> Option<(DrawTaskKind, u32)> {
        let mut best_kind = None;
        let mut best_count = 0u32;
        let mut index = 0usize;
        while index < DrawTaskKind::COUNT {
            if let Some(kind) = DrawTaskKind::from_index(index) {
                let count = self.task_draw_chain_hits.get(kind);
                if count > best_count {
                    best_count = count;
                    best_kind = Some(kind);
                }
            }
            index += 1;
        }
        best_kind.map(|kind| (kind, best_count))
    }

    pub fn benchmark_summary(&self) -> RenderBenchmarkSummary {
        let top_dispatch = self.top_dispatch_task();
        let top_fallback = self.top_fallback_task();
        RenderBenchmarkSummary {
            draw_us: self.draw_us,
            present_us: self.present_us,
            dirty_passes: self.dirty_passes,
            dirty_copy_bytes: self.dirty_copy_bytes,
            present_bytes: self.present_bytes,
            top_dispatch_task: top_dispatch.map(|(kind, _)| kind),
            top_dispatch_hits: top_dispatch.map(|(_, count)| count).unwrap_or(0),
            top_fallback_task: top_fallback.map(|(kind, _)| kind),
            top_fallback_hits: top_fallback.map(|(_, count)| count).unwrap_or(0),
        }
    }

    pub fn merge_draw_stats(&mut self, other: Self) {
        self.commands_seen = self.commands_seen.saturating_add(other.commands_seen);
        self.commands_drawn = self.commands_drawn.saturating_add(other.commands_drawn);
        self.commands_clipped = self.commands_clipped.saturating_add(other.commands_clipped);
        self.blend_commands = self.blend_commands.saturating_add(other.blend_commands);
        self.mask_commands = self.mask_commands.saturating_add(other.mask_commands);
        self.layer_commands = self.layer_commands.saturating_add(other.layer_commands);
        self.blur_commands = self.blur_commands.saturating_add(other.blur_commands);
        self.vector_commands = self.vector_commands.saturating_add(other.vector_commands);
        self.text_commands = self.text_commands.saturating_add(other.text_commands);
        self.image_commands = self.image_commands.saturating_add(other.image_commands);
        self.three_d_commands = self.three_d_commands.saturating_add(other.three_d_commands);
        self.fill_commands = self.fill_commands.saturating_add(other.fill_commands);
        self.border_commands = self.border_commands.saturating_add(other.border_commands);
        self.shadow_commands = self.shadow_commands.saturating_add(other.shadow_commands);
        self.line_commands = self.line_commands.saturating_add(other.line_commands);
        self.arc_commands = self.arc_commands.saturating_add(other.arc_commands);
        self.triangle_commands = self
            .triangle_commands
            .saturating_add(other.triangle_commands);
        self.mask_rect_commands = self
            .mask_rect_commands
            .saturating_add(other.mask_rect_commands);
        self.mask_bitmap_commands = self
            .mask_bitmap_commands
            .saturating_add(other.mask_bitmap_commands);
        self.letter_commands = self.letter_commands.saturating_add(other.letter_commands);
        self.label_commands = self.label_commands.saturating_add(other.label_commands);
        self.fallback_count = self.fallback_count.saturating_add(other.fallback_count);
        self.scratch_bytes = self.scratch_bytes.saturating_add(other.scratch_bytes);
        self.mask_bytes = self.mask_bytes.saturating_add(other.mask_bytes);
        self.layer_bytes = self.layer_bytes.saturating_add(other.layer_bytes);
        self.scratch_peak_bytes = self.scratch_peak_bytes.max(other.scratch_peak_bytes);
        self.fast_path_hits = self.fast_path_hits.saturating_add(other.fast_path_hits);
        self.layer_alloc_failures = self
            .layer_alloc_failures
            .saturating_add(other.layer_alloc_failures);
        self.mask_stack_overflows = self
            .mask_stack_overflows
            .saturating_add(other.mask_stack_overflows);
        self.blur_pixels = self.blur_pixels.saturating_add(other.blur_pixels);
        self.shadow_pixels = self.shadow_pixels.saturating_add(other.shadow_pixels);
        self.layer_fallbacks = self.layer_fallbacks.saturating_add(other.layer_fallbacks);
        self.codec_fallbacks = self.codec_fallbacks.saturating_add(other.codec_fallbacks);
        self.codec_missing_resources = self
            .codec_missing_resources
            .saturating_add(other.codec_missing_resources);
        self.codec_invalid = self.codec_invalid.saturating_add(other.codec_invalid);
        self.codec_truncated = self.codec_truncated.saturating_add(other.codec_truncated);
        self.codec_unsupported = self
            .codec_unsupported
            .saturating_add(other.codec_unsupported);
        self.codec_overflow = self.codec_overflow.saturating_add(other.codec_overflow);
        self.progressive_jpeg_decodes = self
            .progressive_jpeg_decodes
            .saturating_add(other.progressive_jpeg_decodes);
        self.progressive_jpeg_scan_fallbacks = self
            .progressive_jpeg_scan_fallbacks
            .saturating_add(other.progressive_jpeg_scan_fallbacks);
        self.cff_raster_glyphs = self
            .cff_raster_glyphs
            .saturating_add(other.cff_raster_glyphs);
        self.cff_fallback_glyphs = self
            .cff_fallback_glyphs
            .saturating_add(other.cff_fallback_glyphs);
        self.opentype_shaping_runs = self
            .opentype_shaping_runs
            .saturating_add(other.opentype_shaping_runs);
        self.opentype_gsub_hits = self
            .opentype_gsub_hits
            .saturating_add(other.opentype_gsub_hits);
        self.opentype_gpos_hits = self
            .opentype_gpos_hits
            .saturating_add(other.opentype_gpos_hits);
        self.opentype_kern_hits = self
            .opentype_kern_hits
            .saturating_add(other.opentype_kern_hits);
        self.svg_clip_paths = self.svg_clip_paths.saturating_add(other.svg_clip_paths);
        self.svg_masks = self.svg_masks.saturating_add(other.svg_masks);
        self.svg_filters = self.svg_filters.saturating_add(other.svg_filters);
        self.svg_gradients = self.svg_gradients.saturating_add(other.svg_gradients);
        self.svg_real_clip_paths = self
            .svg_real_clip_paths
            .saturating_add(other.svg_real_clip_paths);
        self.svg_real_masks = self.svg_real_masks.saturating_add(other.svg_real_masks);
        self.svg_filter_fallbacks = self
            .svg_filter_fallbacks
            .saturating_add(other.svg_filter_fallbacks);
        self.svg_gradient_fallbacks = self
            .svg_gradient_fallbacks
            .saturating_add(other.svg_gradient_fallbacks);
        self.vector_mask_rasters = self
            .vector_mask_rasters
            .saturating_add(other.vector_mask_rasters);
        self.vector_mask_scratch_overflows = self
            .vector_mask_scratch_overflows
            .saturating_add(other.vector_mask_scratch_overflows);
        self.text_layout_overflows = self
            .text_layout_overflows
            .saturating_add(other.text_layout_overflows);
        self.text_shaped_layouts = self
            .text_shaped_layouts
            .saturating_add(other.text_shaped_layouts);
        self.text_shaping_fallbacks = self
            .text_shaping_fallbacks
            .saturating_add(other.text_shaping_fallbacks);
        self.glyph_run_cache_hits = self
            .glyph_run_cache_hits
            .saturating_add(other.glyph_run_cache_hits);
        self.glyph_run_cache_misses = self
            .glyph_run_cache_misses
            .saturating_add(other.glyph_run_cache_misses);
        self.glyph_run_cache_inserts = self
            .glyph_run_cache_inserts
            .saturating_add(other.glyph_run_cache_inserts);
        self.glyph_run_cache_evictions = self
            .glyph_run_cache_evictions
            .saturating_add(other.glyph_run_cache_evictions);
        self.glyph_run_cache_overflows = self
            .glyph_run_cache_overflows
            .saturating_add(other.glyph_run_cache_overflows);
        self.glyph_run_cache_slots = self.glyph_run_cache_slots.max(other.glyph_run_cache_slots);
        self.svg_doc_cache_hits = self
            .svg_doc_cache_hits
            .saturating_add(other.svg_doc_cache_hits);
        self.svg_doc_cache_misses = self
            .svg_doc_cache_misses
            .saturating_add(other.svg_doc_cache_misses);
        self.svg_doc_cache_loads = self
            .svg_doc_cache_loads
            .saturating_add(other.svg_doc_cache_loads);
        self.svg_doc_cache_fallbacks = self
            .svg_doc_cache_fallbacks
            .saturating_add(other.svg_doc_cache_fallbacks);
        self.svg_doc_cache_slots = self.svg_doc_cache_slots.max(other.svg_doc_cache_slots);
        self.glyph_id_draw_hits = self
            .glyph_id_draw_hits
            .saturating_add(other.glyph_id_draw_hits);
        self.codepoint_fallbacks = self
            .codepoint_fallbacks
            .saturating_add(other.codepoint_fallbacks);
        self.selection_fallbacks = self
            .selection_fallbacks
            .saturating_add(other.selection_fallbacks);
        self.svg_filter_budget_exceeded = self
            .svg_filter_budget_exceeded
            .saturating_add(other.svg_filter_budget_exceeded);
        self.cache_decode_invalid = self
            .cache_decode_invalid
            .saturating_add(other.cache_decode_invalid);
        self.cache_decode_truncated = self
            .cache_decode_truncated
            .saturating_add(other.cache_decode_truncated);
        self.cache_decode_unsupported = self
            .cache_decode_unsupported
            .saturating_add(other.cache_decode_unsupported);
        self.cache_decode_overflow = self
            .cache_decode_overflow
            .saturating_add(other.cache_decode_overflow);
        self.cache_decode_placeholders = self
            .cache_decode_placeholders
            .saturating_add(other.cache_decode_placeholders);
        self.cache_decode_placeholder_missing = self
            .cache_decode_placeholder_missing
            .saturating_add(other.cache_decode_placeholder_missing);
        self.cache_decode_placeholder_invalid = self
            .cache_decode_placeholder_invalid
            .saturating_add(other.cache_decode_placeholder_invalid);
        self.cache_decode_placeholder_truncated = self
            .cache_decode_placeholder_truncated
            .saturating_add(other.cache_decode_placeholder_truncated);
        self.cache_decode_placeholder_unsupported = self
            .cache_decode_placeholder_unsupported
            .saturating_add(other.cache_decode_placeholder_unsupported);
        self.cache_decode_placeholder_overflow = self
            .cache_decode_placeholder_overflow
            .saturating_add(other.cache_decode_placeholder_overflow);
        self.draw_task_fallbacks = self
            .draw_task_fallbacks
            .saturating_add(other.draw_task_fallbacks);
        self.software_path_hits = self
            .software_path_hits
            .saturating_add(other.software_path_hits);
        self.accelerated_path_hits = self
            .accelerated_path_hits
            .saturating_add(other.accelerated_path_hits);
        self.fallback_path_hits = self
            .fallback_path_hits
            .saturating_add(other.fallback_path_hits);
        self.draw_chain_candidates = self
            .draw_chain_candidates
            .saturating_add(other.draw_chain_candidates);
        self.draw_chain_ops = self.draw_chain_ops.saturating_add(other.draw_chain_ops);
        self.draw_chain_submitted = self
            .draw_chain_submitted
            .saturating_add(other.draw_chain_submitted);
        self.draw_chain_fallbacks = self
            .draw_chain_fallbacks
            .saturating_add(other.draw_chain_fallbacks);
        self.draw_chain_unsupported = self
            .draw_chain_unsupported
            .saturating_add(other.draw_chain_unsupported);
        self.draw_chain_overflows = self
            .draw_chain_overflows
            .saturating_add(other.draw_chain_overflows);
        self.draw_chain_runs = self.draw_chain_runs.saturating_add(other.draw_chain_runs);
        self.draw_chain_hw_runs = self
            .draw_chain_hw_runs
            .saturating_add(other.draw_chain_hw_runs);
        self.draw_chain_sw_runs = self
            .draw_chain_sw_runs
            .saturating_add(other.draw_chain_sw_runs);
        self.draw_chain_splits = self
            .draw_chain_splits
            .saturating_add(other.draw_chain_splits);
        self.draw_chain_parallel_hints = self
            .draw_chain_parallel_hints
            .saturating_add(other.draw_chain_parallel_hints);
        self.draw_chain_parallel_queued = self
            .draw_chain_parallel_queued
            .saturating_add(other.draw_chain_parallel_queued);
        self.draw_chain_parallel_completed = self
            .draw_chain_parallel_completed
            .saturating_add(other.draw_chain_parallel_completed);
        self.draw_chain_parallel_barriers = self
            .draw_chain_parallel_barriers
            .saturating_add(other.draw_chain_parallel_barriers);
        self.draw_chain_parallel_software_runs = self
            .draw_chain_parallel_software_runs
            .saturating_add(other.draw_chain_parallel_software_runs);
        self.draw_chain_parallel_fallbacks = self
            .draw_chain_parallel_fallbacks
            .saturating_add(other.draw_chain_parallel_fallbacks);
        self.accel2d_ring_submissions = self
            .accel2d_ring_submissions
            .saturating_add(other.accel2d_ring_submissions);
        self.accel2d_ring_flushes = self
            .accel2d_ring_flushes
            .saturating_add(other.accel2d_ring_flushes);
        self.accel2d_fences_issued = self
            .accel2d_fences_issued
            .saturating_add(other.accel2d_fences_issued);
        self.accel2d_fences_completed = self
            .accel2d_fences_completed
            .saturating_add(other.accel2d_fences_completed);
        self.accel2d_ring_overflows = self
            .accel2d_ring_overflows
            .saturating_add(other.accel2d_ring_overflows);
        self.codec_prewarm_parallel_hints = self
            .codec_prewarm_parallel_hints
            .saturating_add(other.codec_prewarm_parallel_hints);
        self.task_draw_chain_hits.merge(other.task_draw_chain_hits);
        self.codec_pipeline_candidates = self
            .codec_pipeline_candidates
            .saturating_add(other.codec_pipeline_candidates);
        self.codec_pipeline_stages = self
            .codec_pipeline_stages
            .saturating_add(other.codec_pipeline_stages);
        self.codec_pipeline_hardware_candidates = self
            .codec_pipeline_hardware_candidates
            .saturating_add(other.codec_pipeline_hardware_candidates);
        self.codec_pipeline_fallbacks = self
            .codec_pipeline_fallbacks
            .saturating_add(other.codec_pipeline_fallbacks);
        self.codec_pipeline_unsupported = self
            .codec_pipeline_unsupported
            .saturating_add(other.codec_pipeline_unsupported);
        self.codec_pipeline_overflows = self
            .codec_pipeline_overflows
            .saturating_add(other.codec_pipeline_overflows);
        self.task_software_hits.merge(other.task_software_hits);
        self.task_accelerated_hits
            .merge(other.task_accelerated_hits);
        self.task_fallback_hits.merge(other.task_fallback_hits);
        self.clip_changes = self.clip_changes.saturating_add(other.clip_changes);
        self.effective_clip_changes = self
            .effective_clip_changes
            .saturating_add(other.effective_clip_changes);
        self.dirty_rects = self.dirty_rects.max(other.dirty_rects);
        self.dirty_passes = self.dirty_passes.saturating_add(other.dirty_passes);
        self.dirty_copy_bytes = self.dirty_copy_bytes.saturating_add(other.dirty_copy_bytes);
        self.pixels_estimate = self.pixels_estimate.saturating_add(other.pixels_estimate);
        self.overflowed |= other.overflowed;
    }

    pub fn command_seen(&mut self) {
        self.commands_seen = self.commands_seen.saturating_add(1);
    }

    pub fn mark_command_kind(&mut self, cmd: DrawCommand) {
        match cmd {
            DrawCommand::FillRect { .. }
            | DrawCommand::FillRoundRect { .. }
            | DrawCommand::FillGradient { .. }
            | DrawCommand::FillCircle { .. } => {
                self.fill_commands = self.fill_commands.saturating_add(1);
                self.fast_path_hits = self.fast_path_hits.saturating_add(1);
            }
            DrawCommand::FillStyled { style, .. } => {
                self.fill_commands = self.fill_commands.saturating_add(1);
                if style.blend != crate::BlendMode::Normal {
                    self.blend_commands = self.blend_commands.saturating_add(1);
                }
                if style.radius == 0 && style.gradient == crate::GradientStyle::None {
                    self.fast_path_hits = self.fast_path_hits.saturating_add(1);
                }
            }
            DrawCommand::DrawMask { rect, .. } => {
                self.mask_commands = self.mask_commands.saturating_add(1);
                self.mask_rect_commands = self.mask_rect_commands.saturating_add(1);
                self.mask_bytes = self.mask_bytes.saturating_add(rect_area(rect));
            }
            DrawCommand::DrawLayer { rect, .. } => {
                self.layer_commands = self.layer_commands.saturating_add(1);
                self.mark_layer_bytes(rect_area(rect).saturating_mul(4));
                self.mark_layer_fallback();
            }
            DrawCommand::BeginLayer { rect, .. } => {
                self.layer_commands = self.layer_commands.saturating_add(1);
                self.mark_layer_bytes(rect_area(rect).saturating_mul(4));
            }
            DrawCommand::DrawBlur { rect, .. } => {
                self.blur_commands = self.blur_commands.saturating_add(1);
                self.mark_scratch_bytes(rect_area(rect).saturating_mul(4));
                self.blur_pixels = self.blur_pixels.saturating_add(rect_area(rect));
                self.mark_layer_fallback();
            }
            DrawCommand::DrawShadow { rect, .. } => {
                self.shadow_commands = self.shadow_commands.saturating_add(1);
                self.mark_shadow_pixels(rect_area(rect));
            }
            DrawCommand::DrawBorder { .. } => {
                self.border_commands = self.border_commands.saturating_add(1);
            }
            DrawCommand::StrokeLine { .. } | DrawCommand::StrokeStyledLine { .. } => {
                self.line_commands = self.line_commands.saturating_add(1);
            }
            DrawCommand::DrawArc { .. } => {
                self.arc_commands = self.arc_commands.saturating_add(1);
            }
            DrawCommand::PushMask { .. } => {
                self.mask_commands = self.mask_commands.saturating_add(1);
                self.mask_rect_commands = self.mask_rect_commands.saturating_add(1);
            }
            DrawCommand::PushBitmapMask { .. } => {
                self.mask_commands = self.mask_commands.saturating_add(1);
                self.mask_bitmap_commands = self.mask_bitmap_commands.saturating_add(1);
            }
            DrawCommand::DrawSvgIcon { .. } | DrawCommand::DrawSvgDocument { .. } => {
                self.vector_commands = self.vector_commands.saturating_add(1);
            }
            DrawCommand::DrawText { .. } => {
                self.text_commands = self.text_commands.saturating_add(1);
                self.letter_commands = self.letter_commands.saturating_add(1);
            }
            DrawCommand::DrawLabel { .. } => {
                self.text_commands = self.text_commands.saturating_add(1);
                self.label_commands = self.label_commands.saturating_add(1);
            }
            DrawCommand::DrawImage { .. } | DrawCommand::DrawImageFit { .. } => {
                self.image_commands = self.image_commands.saturating_add(1);
                self.fast_path_hits = self.fast_path_hits.saturating_add(1);
            }
            DrawCommand::DrawImageTint { .. } => {
                self.image_commands = self.image_commands.saturating_add(1);
            }
            DrawCommand::DrawImageStyled { style, .. } => {
                self.image_commands = self.image_commands.saturating_add(1);
                if style.blend != crate::BlendMode::Normal {
                    self.blend_commands = self.blend_commands.saturating_add(1);
                }
                if style.blend == crate::BlendMode::Normal
                    && style.tint.is_none()
                    && style.clip_radius == 0
                    && !style.tile
                {
                    self.fast_path_hits = self.fast_path_hits.saturating_add(1);
                }
            }
            DrawCommand::DrawTriangle { .. } | DrawCommand::DrawGradientTriangle { .. } => {
                self.triangle_commands = self.triangle_commands.saturating_add(1);
                self.three_d_commands = self.three_d_commands.saturating_add(1);
            }
            DrawCommand::DrawTexturedTriangle { .. } => {
                self.triangle_commands = self.triangle_commands.saturating_add(1);
                self.three_d_commands = self.three_d_commands.saturating_add(1);
            }
            _ => {}
        }
    }

    pub fn mark_scratch_bytes(&mut self, bytes: u32) {
        self.scratch_bytes = self.scratch_bytes.saturating_add(bytes);
        self.scratch_peak_bytes = self.scratch_peak_bytes.max(bytes);
    }

    pub fn mark_layer_bytes(&mut self, bytes: u32) {
        self.layer_bytes = self.layer_bytes.saturating_add(bytes);
        self.scratch_peak_bytes = self.scratch_peak_bytes.max(bytes);
    }

    pub fn mark_layer_alloc_failure(&mut self) {
        self.layer_alloc_failures = self.layer_alloc_failures.saturating_add(1);
        self.mark_layer_fallback();
    }

    pub fn mark_mask_stack_overflow(&mut self) {
        self.mask_stack_overflows = self.mask_stack_overflows.saturating_add(1);
        self.overflowed = true;
    }

    pub fn mark_layer_fallback(&mut self) {
        self.layer_fallbacks = self.layer_fallbacks.saturating_add(1);
        self.fallback_count = self.fallback_count.saturating_add(1);
        self.draw_task_fallbacks = self.draw_task_fallbacks.saturating_add(1);
        self.fallback_path_hits = self.fallback_path_hits.saturating_add(1);
    }

    pub fn mark_draw_chain_stats(&mut self, chain: DrawChainStats) {
        self.draw_chain_candidates = self.draw_chain_candidates.saturating_add(chain.candidates);
        self.draw_chain_ops = self.draw_chain_ops.saturating_add(chain.ops);
        self.draw_chain_submitted = self.draw_chain_submitted.saturating_add(chain.submitted);
        self.draw_chain_fallbacks = self.draw_chain_fallbacks.saturating_add(chain.fallbacks);
        self.draw_chain_unsupported = self
            .draw_chain_unsupported
            .saturating_add(chain.unsupported);
        self.draw_chain_overflows = self.draw_chain_overflows.saturating_add(chain.overflows);
        self.task_draw_chain_hits.merge(chain.task_hits);
    }

    pub fn mark_draw_chain_submit(
        &mut self,
        mut chain: DrawChainStats,
        result: DrawChainSubmitResult,
    ) {
        match result {
            DrawChainSubmitResult::Submitted => {
                chain.record_submitted();
            }
            DrawChainSubmitResult::Unsupported => {
                chain.record_unsupported();
            }
            DrawChainSubmitResult::Fallback => {
                chain.fallbacks = chain.fallbacks.saturating_add(1);
            }
        }
        self.mark_draw_chain_stats(chain);
    }

    pub fn mark_draw_chain_run_result(&mut self, result: DrawChainSubmitResult) {
        self.draw_chain_runs = self.draw_chain_runs.saturating_add(1);
        match result {
            DrawChainSubmitResult::Submitted => {
                self.draw_chain_hw_runs = self.draw_chain_hw_runs.saturating_add(1);
            }
            DrawChainSubmitResult::Unsupported | DrawChainSubmitResult::Fallback => {
                self.draw_chain_sw_runs = self.draw_chain_sw_runs.saturating_add(1);
            }
        }
    }

    pub fn mark_draw_chain_split(&mut self) {
        self.draw_chain_splits = self.draw_chain_splits.saturating_add(1);
    }

    pub fn mark_draw_chain_parallel_hint(&mut self) {
        self.draw_chain_parallel_hints = self.draw_chain_parallel_hints.saturating_add(1);
    }

    pub fn mark_draw_chain_parallel_queued(&mut self) {
        self.draw_chain_parallel_queued = self.draw_chain_parallel_queued.saturating_add(1);
    }

    pub fn mark_draw_chain_parallel_completed(&mut self) {
        self.draw_chain_parallel_completed = self.draw_chain_parallel_completed.saturating_add(1);
    }

    pub fn mark_draw_chain_parallel_barrier(&mut self) {
        self.draw_chain_parallel_barriers = self.draw_chain_parallel_barriers.saturating_add(1);
    }

    pub fn mark_draw_chain_parallel_software_run(&mut self) {
        self.draw_chain_parallel_software_runs =
            self.draw_chain_parallel_software_runs.saturating_add(1);
    }

    pub fn mark_draw_chain_parallel_fallback(&mut self) {
        self.draw_chain_parallel_fallbacks = self.draw_chain_parallel_fallbacks.saturating_add(1);
    }

    pub fn mark_accel2d_ring_submission(&mut self) {
        self.accel2d_ring_submissions = self.accel2d_ring_submissions.saturating_add(1);
    }

    pub fn mark_accel2d_ring_flush(&mut self) {
        self.accel2d_ring_flushes = self.accel2d_ring_flushes.saturating_add(1);
    }

    pub fn mark_accel2d_fence_issued(&mut self) {
        self.accel2d_fences_issued = self.accel2d_fences_issued.saturating_add(1);
    }

    pub fn mark_accel2d_fence_completed(&mut self) {
        self.accel2d_fences_completed = self.accel2d_fences_completed.saturating_add(1);
    }

    pub fn mark_accel2d_ring_overflow(&mut self) {
        self.accel2d_ring_overflows = self.accel2d_ring_overflows.saturating_add(1);
        self.draw_chain_overflows = self.draw_chain_overflows.saturating_add(1);
        self.overflowed = true;
    }

    pub fn mark_codec_prewarm_parallel_hint(&mut self) {
        self.codec_prewarm_parallel_hints = self.codec_prewarm_parallel_hints.saturating_add(1);
    }

    pub fn mark_draw_dispatch(&mut self, path: DrawPathKind) {
        self.mark_draw_dispatch_kind(path, None);
    }

    pub fn mark_draw_dispatch_for(&mut self, kind: DrawTaskKind, path: DrawPathKind) {
        self.mark_draw_dispatch_kind(path, Some(kind));
    }

    fn mark_draw_dispatch_kind(&mut self, path: DrawPathKind, kind: Option<DrawTaskKind>) {
        match path {
            DrawPathKind::Software => {
                self.software_path_hits = self.software_path_hits.saturating_add(1);
                if let Some(kind) = kind {
                    self.task_software_hits.increment(kind);
                }
            }
            DrawPathKind::Accelerated => {
                self.accelerated_path_hits = self.accelerated_path_hits.saturating_add(1);
                if let Some(kind) = kind {
                    self.task_accelerated_hits.increment(kind);
                }
            }
            DrawPathKind::Fallback => {
                self.fallback_path_hits = self.fallback_path_hits.saturating_add(1);
                self.draw_task_fallbacks = self.draw_task_fallbacks.saturating_add(1);
                self.fallback_count = self.fallback_count.saturating_add(1);
                if let Some(kind) = kind {
                    self.task_fallback_hits.increment(kind);
                }
            }
        }
    }

    pub fn mark_blur_pixels(&mut self, pixels: u32) {
        self.blur_pixels = self.blur_pixels.saturating_add(pixels);
    }

    pub fn mark_shadow_pixels(&mut self, pixels: u32) {
        self.shadow_pixels = self.shadow_pixels.saturating_add(pixels);
    }

    pub fn command_clipped(&mut self) {
        self.commands_clipped = self.commands_clipped.saturating_add(1);
    }

    pub fn clip_changed(&mut self) {
        self.clip_changes = self.clip_changes.saturating_add(1);
    }

    pub fn effective_clip_changed(&mut self) {
        self.effective_clip_changes = self.effective_clip_changes.saturating_add(1);
        self.clip_changed();
    }

    pub fn command_drawn(&mut self, bounds: Option<Rect>) {
        self.commands_drawn = self.commands_drawn.saturating_add(1);
        if let Some(bounds) = bounds {
            self.pixels_estimate = self.pixels_estimate.saturating_add(rect_area(bounds));
        }
    }
}

impl BackendCapabilities {
    pub const SOFTWARE_UNKNOWN: Self = Self {
        pixel_format: PixelFormat::Unknown(0),
        software: true,
        accelerated_2d: false,
        accelerated_3d: false,
        clip_rect: true,
        alpha_blend: true,
        blend_modes: true,
        gradients: true,
        mask: true,
        layers: true,
        blur: true,
        arc: true,
        vector_icons: true,
        vector_fill: true,
        image_transform: true,
        text_layout: true,
        triangles: true,
        draw_fill: true,
        draw_border: true,
        draw_box_shadow: true,
        draw_letter: true,
        draw_label: true,
        draw_image: true,
        draw_layer: true,
        draw_line: true,
        draw_arc: true,
        draw_triangle: true,
        draw_mask_rect: true,
        draw_mask_bitmap: true,
        draw_blur: true,
        draw_vector: true,
        draw_3d: true,
        software_draw_features: DrawFeatureFlags::ALL_SOFTWARE,
        accelerated_draw_features: DrawFeatureFlags::NONE,
        draw_chain: false,
        chain_continuous_submit: false,
        chain_run_fence: false,
        chain_mix_alpha: false,
        chain_mix_clip: false,
        chain_mix_mask: false,
        chain_mix_layer: false,
        max_chain_ops: 0,
        chain_draw_features: DrawFeatureFlags::NONE,
    };

    pub const fn software(pixel_format: PixelFormat) -> Self {
        Self {
            pixel_format,
            software: true,
            accelerated_2d: false,
            accelerated_3d: false,
            clip_rect: true,
            alpha_blend: true,
            blend_modes: true,
            gradients: true,
            mask: true,
            layers: true,
            blur: true,
            arc: true,
            vector_icons: true,
            vector_fill: true,
            image_transform: true,
            text_layout: true,
            triangles: true,
            draw_fill: true,
            draw_border: true,
            draw_box_shadow: true,
            draw_letter: true,
            draw_label: true,
            draw_image: true,
            draw_layer: true,
            draw_line: true,
            draw_arc: true,
            draw_triangle: true,
            draw_mask_rect: true,
            draw_mask_bitmap: true,
            draw_blur: true,
            draw_vector: true,
            draw_3d: true,
            software_draw_features: DrawFeatureFlags::ALL_SOFTWARE,
            accelerated_draw_features: DrawFeatureFlags::NONE,
            draw_chain: false,
            chain_continuous_submit: false,
            chain_run_fence: false,
            chain_mix_alpha: false,
            chain_mix_clip: false,
            chain_mix_mask: false,
            chain_mix_layer: false,
            max_chain_ops: 0,
            chain_draw_features: DrawFeatureFlags::NONE,
        }
    }

    pub const fn supports_draw_task(&self, kind: DrawTaskKind) -> bool {
        self.software_draw_features.supports(kind) || self.accelerated_draw_features.supports(kind)
    }
}

pub trait RenderBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::SOFTWARE_UNKNOWN
    }

    fn submit_draw_chain<const OPS: usize>(
        &mut self,
        chain: &DrawChain<OPS>,
        stats: &mut RenderStats,
    ) -> DrawChainSubmitResult {
        if chain.is_empty() {
            return DrawChainSubmitResult::Unsupported;
        }
        stats.mark_draw_chain_submit(chain.stats(), DrawChainSubmitResult::Unsupported);
        DrawChainSubmitResult::Unsupported
    }

    fn submit_draw_chain_with_contract<const OPS: usize>(
        &mut self,
        chain: &DrawChain<OPS>,
        _contract: &DrawChainRunContract<OPS>,
        stats: &mut RenderStats,
    ) -> DrawChainSubmitResult {
        self.submit_draw_chain(chain, stats)
    }

    fn set_clip(&mut self, _clip: Option<Rect>) {}
    fn draw_command(&mut self, cmd: DrawCommand);

    fn draw_dispatched_command(
        &mut self,
        cmd: DrawCommand,
        path: DrawPathKind,
        stats: &mut RenderStats,
    ) {
        if let Some(kind) = cmd.task_kind() {
            stats.mark_draw_dispatch_for(kind, path);
        } else {
            stats.mark_draw_dispatch(path);
        }
        if let DrawCommand::DrawLabel { style, .. } = cmd {
            if style.font.0 != 0 {
                stats.mark_glyph_id_draw_hit();
            } else {
                stats.mark_codepoint_fallback();
            }
            if style.selection.is_some() && style.font.0 == 0 {
                stats.mark_selection_fallback();
            }
        }
        self.draw_command(cmd);
    }

    fn push_mask(&mut self, _spec: MaskSpec) -> bool {
        true
    }

    fn pop_mask(&mut self) -> bool {
        true
    }

    fn clear_masks(&mut self) {}

    fn draw_layer_commands(
        &mut self,
        rect: Rect,
        spec: LayerSpec,
        _cmds: &[DrawCommand],
        _clips: &[Option<Rect>],
        stats: &mut RenderStats,
    ) -> bool {
        stats.mark_layer_fallback();
        self.draw_command(DrawCommand::DrawLayer {
            rect,
            depth: 0,
            spec,
        });
        false
    }

    fn draw_dispatched_layer_commands(
        &mut self,
        rect: Rect,
        spec: LayerSpec,
        cmds: &[DrawCommand],
        clips: &[Option<Rect>],
        path: DrawPathKind,
        stats: &mut RenderStats,
    ) -> bool {
        stats.mark_draw_dispatch_for(DrawTaskKind::Layer, path);
        self.draw_layer_commands(rect, spec, cmds, clips, stats)
    }
}

pub trait ParallelRenderBackend: RenderBackend {
    fn queue_draw_chain_with_contract<const OPS: usize>(
        &mut self,
        _chain: &DrawChain<OPS>,
        _contract: &DrawChainRunContract<OPS>,
        _stats: &mut RenderStats,
    ) -> ParallelDrawChainSubmitResult {
        ParallelDrawChainSubmitResult::Unsupported
    }

    fn pending_draw_chain_bounds(&self) -> Option<Rect> {
        None
    }

    fn flush_pending_draw_chain(&mut self, _stats: &mut RenderStats) -> DrawChainSubmitResult {
        DrawChainSubmitResult::Unsupported
    }
}

fn rect_area(rect: Rect) -> u32 {
    (rect.w as u32).saturating_mul(rect.h as u32)
}
