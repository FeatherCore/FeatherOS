use crate::{
    backend::{
        BackendCapabilities, DrawBackendDispatch, DrawChain, DrawChainOp, DrawChainOpKind,
        DrawChainSubmitResult, DrawTaskKind, RenderBackend, RenderStats,
        DEFAULT_DRAW_CHAIN_OPS,
    },
    dirty::DirtyRegion,
    surface::Surface,
    Color, Fixed16, Point, Rect,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FontId(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageId(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SvgId(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageFit {
    Stretch,
    Contain,
    Cover,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlendMode {
    Normal,
    Additive,
    Subtractive,
    Multiply,
    Difference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientStyle {
    None,
    Vertical {
        start: Color,
        end: Color,
    },
    Horizontal {
        start: Color,
        end: Color,
    },
    Linear {
        start: Point,
        end: Point,
        start_color: Color,
        end_color: Color,
    },
    Radial {
        center: Point,
        radius: u16,
        inner: Color,
        outer: Color,
    },
    Conical {
        center: Point,
        start: Color,
        end: Color,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FillStyle {
    pub color: Color,
    pub gradient: GradientStyle,
    pub radius: u16,
    pub blend: BlendMode,
}

impl FillStyle {
    pub const fn solid(color: Color) -> Self {
        Self {
            color,
            gradient: GradientStyle::None,
            radius: 0,
            blend: BlendMode::Normal,
        }
    }

    pub const fn rounded(color: Color, radius: u16) -> Self {
        Self {
            color,
            gradient: GradientStyle::None,
            radius,
            blend: BlendMode::Normal,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BorderSides(pub u8);

impl BorderSides {
    pub const NONE: Self = Self(0);
    pub const BOTTOM: Self = Self(1 << 0);
    pub const TOP: Self = Self(1 << 1);
    pub const LEFT: Self = Self(1 << 2);
    pub const RIGHT: Self = Self(1 << 3);
    pub const FULL: Self = Self(Self::BOTTOM.0 | Self::TOP.0 | Self::LEFT.0 | Self::RIGHT.0);

    pub const fn contains(self, side: Self) -> bool {
        (self.0 & side.0) != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BorderAlign {
    Inside,
    Center,
    Outside,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BorderStyle {
    pub color: Color,
    pub width: u16,
    pub radius: u16,
    pub sides: BorderSides,
    pub align: BorderAlign,
}

impl BorderStyle {
    pub const fn full(color: Color, width: u16, radius: u16) -> Self {
        Self {
            color,
            width,
            radius,
            sides: BorderSides::FULL,
            align: BorderAlign::Inside,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShadowStyle {
    pub color: Color,
    pub width: u16,
    pub spread: i16,
    pub offset: Point,
    pub radius: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineCap {
    Butt,
    Square,
    Round,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineJoin {
    Miter,
    Bevel,
    Round,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineStyle {
    pub color: Color,
    pub width: u16,
    pub dash_width: u16,
    pub dash_gap: u16,
    pub round_start: bool,
    pub round_end: bool,
    pub cap_start: LineCap,
    pub cap_end: LineCap,
    pub join: LineJoin,
    pub blend: BlendMode,
}

impl LineStyle {
    pub const fn solid(color: Color, width: u16) -> Self {
        Self {
            color,
            width,
            dash_width: 0,
            dash_gap: 0,
            round_start: false,
            round_end: false,
            cap_start: LineCap::Butt,
            cap_end: LineCap::Butt,
            join: LineJoin::Miter,
            blend: BlendMode::Normal,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcStyle {
    pub color: Color,
    pub width: u16,
    pub start_angle: i16,
    pub end_angle: i16,
    pub rounded: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextDecor(pub u8);

impl TextDecor {
    pub const NONE: Self = Self(0);
    pub const UNDERLINE: Self = Self(1 << 0);
    pub const STRIKETHROUGH: Self = Self(1 << 1);

    pub const fn contains(self, decor: Self) -> bool {
        (self.0 & decor.0) != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextStyle {
    pub font: FontId,
    pub color: Color,
    pub scale: u16,
    pub align: TextAlign,
    pub letter_spacing: i16,
    pub line_spacing: i16,
    pub kerning: bool,
    pub decor: TextDecor,
    pub selection: Option<(u16, u16, Color, Color)>,
}

impl TextStyle {
    pub const fn debug(color: Color, scale: u16) -> Self {
        Self {
            font: FontId(0),
            color,
            scale,
            align: TextAlign::Left,
            letter_spacing: 0,
            line_spacing: 0,
            kerning: false,
            decor: TextDecor::NONE,
            selection: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageDrawStyle {
    pub opacity: u8,
    pub fit: ImageFit,
    pub tint: Option<Color>,
    pub clip_radius: u16,
    pub tile: bool,
    pub blend: BlendMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayerBudget {
    pub max_bytes: usize,
}

impl LayerBudget {
    pub const DEFAULT: Self = Self {
        max_bytes: 256 * 1024,
    };

    pub const DEMO: Self = Self {
        max_bytes: 2 * 1024 * 1024,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaskKind {
    Rect,
    RoundedRect,
    Bitmap { image: ImageId },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaskSpec {
    pub area: Rect,
    pub radius: u16,
    pub kind: MaskKind,
    pub inverted: bool,
    pub opacity: u8,
}

impl MaskSpec {
    pub const fn rect(area: Rect) -> Self {
        Self {
            area,
            radius: 0,
            kind: MaskKind::Rect,
            inverted: false,
            opacity: 255,
        }
    }

    pub const fn rounded(area: Rect, radius: u16) -> Self {
        Self {
            area,
            radius,
            kind: MaskKind::RoundedRect,
            inverted: false,
            opacity: 255,
        }
    }

    pub const fn bitmap(area: Rect, image: ImageId) -> Self {
        Self {
            area,
            radius: 0,
            kind: MaskKind::Bitmap { image },
            inverted: false,
            opacity: 255,
        }
    }

    pub const fn inverted(mut self, inverted: bool) -> Self {
        self.inverted = inverted;
        self
    }

    pub const fn opacity(mut self, opacity: u8) -> Self {
        self.opacity = opacity;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayerSpec {
    pub opacity: u8,
    pub recolor: Option<Color>,
    pub blur_radius: u16,
    pub mask: Option<MaskSpec>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaskStack<const N: usize> {
    specs: [Option<MaskSpec>; N],
    len: usize,
    overflowed: bool,
}

impl<const N: usize> MaskStack<N> {
    pub const fn new() -> Self {
        Self {
            specs: [None; N],
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

    pub fn push(&mut self, spec: MaskSpec) -> bool {
        if self.len >= N {
            self.overflowed = true;
            return false;
        }
        self.specs[self.len] = Some(spec);
        self.len += 1;
        true
    }

    pub fn pop(&mut self) -> Option<MaskSpec> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let spec = self.specs[self.len];
        self.specs[self.len] = None;
        spec
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.overflowed = false;
        let mut i = 0;
        while i < N {
            self.specs[i] = None;
            i += 1;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DepthSpan {
    pub a: Fixed16,
    pub b: Fixed16,
    pub c: Fixed16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TexCoord {
    pub u: u16,
    pub v: u16,
}

impl TexCoord {
    pub const fn new(u: u16, v: u16) -> Self {
        Self { u, v }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TriangleStyle {
    pub colors: [Color; 3],
    pub blend: BlendMode,
}

impl TriangleStyle {
    pub const fn solid(color: Color) -> Self {
        Self {
            colors: [color; 3],
            blend: BlendMode::Normal,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawCommand {
    Noop,
    Clear(Color),
    SetClip(Rect),
    FillRect {
        rect: Rect,
        depth: Fixed16,
        color: Color,
    },
    FillRoundRect {
        rect: Rect,
        depth: Fixed16,
        radius: u16,
        color: Color,
    },
    FillGradient {
        rect: Rect,
        depth: Fixed16,
        top: Color,
        bottom: Color,
    },
    FillStyled {
        rect: Rect,
        depth: Fixed16,
        style: FillStyle,
    },
    FillCircle {
        center: Point,
        depth: Fixed16,
        radius: u16,
        color: Color,
    },
    StrokeLine {
        from: Point,
        to: Point,
        depth: Fixed16,
        width: u16,
        color: Color,
    },
    StrokeStyledLine {
        from: Point,
        to: Point,
        depth: Fixed16,
        style: LineStyle,
    },
    DrawBorder {
        rect: Rect,
        depth: Fixed16,
        style: BorderStyle,
    },
    DrawShadow {
        rect: Rect,
        depth: Fixed16,
        style: ShadowStyle,
    },
    DrawArc {
        center: Point,
        depth: Fixed16,
        radius: u16,
        style: ArcStyle,
    },
    DrawText {
        pos: Point,
        depth: Fixed16,
        text: &'static str,
        font: FontId,
        color: Color,
        scale: u16,
    },
    DrawLabel {
        rect: Rect,
        depth: Fixed16,
        text: &'static str,
        style: TextStyle,
    },
    DrawImage {
        rect: Rect,
        depth: Fixed16,
        image: ImageId,
        opacity: u8,
    },
    DrawImageFit {
        rect: Rect,
        depth: Fixed16,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
    },
    DrawImageTint {
        rect: Rect,
        depth: Fixed16,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
        tint: Color,
    },
    DrawImageStyled {
        rect: Rect,
        depth: Fixed16,
        image: ImageId,
        style: ImageDrawStyle,
    },
    DrawSvgIcon {
        rect: Rect,
        depth: Fixed16,
        icon: SvgId,
        color: Color,
        opacity: u8,
    },
    DrawSvgDocument {
        rect: Rect,
        depth: Fixed16,
        document: SvgId,
        color: Color,
        opacity: u8,
    },
    DrawMask {
        rect: Rect,
        depth: Fixed16,
        spec: MaskSpec,
    },
    DrawBlur {
        rect: Rect,
        depth: Fixed16,
        radius: u16,
        opacity: u8,
    },
    DrawLayer {
        rect: Rect,
        depth: Fixed16,
        spec: LayerSpec,
    },
    BeginLayer {
        rect: Rect,
        depth: Fixed16,
        spec: LayerSpec,
    },
    EndLayer,
    PushMask {
        depth: Fixed16,
        spec: MaskSpec,
    },
    PushBitmapMask {
        rect: Rect,
        depth: Fixed16,
        image: ImageId,
        inverted: bool,
        opacity: u8,
    },
    PopMask,
    DrawTriangle {
        p0: Point,
        p1: Point,
        p2: Point,
        depth: DepthSpan,
        color: Color,
    },
    DrawGradientTriangle {
        p0: Point,
        p1: Point,
        p2: Point,
        depth: DepthSpan,
        style: TriangleStyle,
    },
    DrawTexturedTriangle {
        p0: Point,
        p1: Point,
        p2: Point,
        uv0: TexCoord,
        uv1: TexCoord,
        uv2: TexCoord,
        depth: DepthSpan,
        image: ImageId,
        opacity: u8,
    },
}

impl DrawCommand {
    pub const fn depth(self) -> Fixed16 {
        match self {
            Self::Noop | Self::Clear(_) | Self::SetClip(_) => 0,
            Self::FillRect { depth, .. }
            | Self::FillRoundRect { depth, .. }
            | Self::FillGradient { depth, .. }
            | Self::FillStyled { depth, .. }
            | Self::FillCircle { depth, .. }
            | Self::StrokeLine { depth, .. }
            | Self::StrokeStyledLine { depth, .. }
            | Self::DrawBorder { depth, .. }
            | Self::DrawShadow { depth, .. }
            | Self::DrawArc { depth, .. }
            | Self::DrawText { depth, .. }
            | Self::DrawLabel { depth, .. }
            | Self::DrawImage { depth, .. }
            | Self::DrawImageFit { depth, .. }
            | Self::DrawImageTint { depth, .. }
            | Self::DrawImageStyled { depth, .. }
            | Self::DrawSvgIcon { depth, .. }
            | Self::DrawSvgDocument { depth, .. }
            | Self::DrawMask { depth, .. }
            | Self::DrawBlur { depth, .. }
            | Self::DrawLayer { depth, .. }
            | Self::BeginLayer { depth, .. }
            | Self::PushMask { depth, .. }
            | Self::PushBitmapMask { depth, .. } => depth,
            Self::EndLayer | Self::PopMask => 0,
            Self::DrawTriangle { depth, .. }
            | Self::DrawGradientTriangle { depth, .. }
            | Self::DrawTexturedTriangle { depth, .. } => (depth.a + depth.b + depth.c) / 3,
        }
    }

    pub const fn task_kind(self) -> Option<DrawTaskKind> {
        match self {
            Self::Noop | Self::Clear(_) | Self::SetClip(_) | Self::EndLayer | Self::PopMask => None,
            Self::FillRect { .. }
            | Self::FillRoundRect { .. }
            | Self::FillGradient { .. }
            | Self::FillStyled { .. }
            | Self::FillCircle { .. } => Some(DrawTaskKind::Fill),
            Self::DrawBorder { .. } => Some(DrawTaskKind::Border),
            Self::DrawShadow { .. } => Some(DrawTaskKind::BoxShadow),
            Self::DrawText { .. } => Some(DrawTaskKind::Letter),
            Self::DrawLabel { .. } => Some(DrawTaskKind::Label),
            Self::DrawImage { .. }
            | Self::DrawImageFit { .. }
            | Self::DrawImageTint { .. }
            | Self::DrawImageStyled { .. } => Some(DrawTaskKind::Image),
            Self::DrawLayer { .. } | Self::BeginLayer { .. } => Some(DrawTaskKind::Layer),
            Self::StrokeLine { .. } | Self::StrokeStyledLine { .. } => Some(DrawTaskKind::Line),
            Self::DrawArc { .. } => Some(DrawTaskKind::Arc),
            Self::DrawTriangle { .. }
            | Self::DrawGradientTriangle { .. }
            | Self::DrawTexturedTriangle { .. } => Some(DrawTaskKind::Triangle),
            Self::DrawMask { .. } | Self::PushMask { .. } => Some(DrawTaskKind::MaskRect),
            Self::PushBitmapMask { .. } => Some(DrawTaskKind::MaskBitmap),
            Self::DrawBlur { .. } => Some(DrawTaskKind::Blur),
            Self::DrawSvgIcon { .. } | Self::DrawSvgDocument { .. } => Some(DrawTaskKind::Vector),
        }
    }

    pub fn execute(self, surface: &mut Surface) {
        match self {
            Self::Noop => {}
            Self::SetClip(rect) => surface.set_clip(Some(rect)),
            Self::Clear(color) => surface.clear(color),
            Self::FillRect { rect, color, .. } => surface.fill_rect(rect, color),
            Self::FillRoundRect {
                rect,
                radius,
                color,
                ..
            } => surface.fill_round_rect(rect, radius, color),
            Self::FillGradient {
                rect, top, bottom, ..
            } => surface.fill_vgradient(rect, top, bottom),
            Self::FillStyled { rect, style, .. } => surface.fill_style(rect, style),
            Self::FillCircle {
                center,
                radius,
                color,
                ..
            } => surface.fill_circle(center.x, center.y, radius as i32, color),
            Self::StrokeLine {
                from,
                to,
                width,
                color,
                ..
            } => surface.draw_wide_line(from, to, width, color),
            Self::StrokeStyledLine {
                from, to, style, ..
            } => surface.draw_styled_line(from, to, style),
            Self::DrawBorder { rect, style, .. } => surface.draw_border(rect, style),
            Self::DrawShadow { rect, style, .. } => surface.draw_shadow(rect, style),
            Self::DrawArc {
                center,
                radius,
                style,
                ..
            } => surface.draw_arc(center, radius, style),
            Self::DrawText {
                pos,
                text,
                font,
                color,
                scale,
                ..
            } => surface.draw_text_font(pos.x, pos.y, text, font, color, scale),
            Self::DrawLabel {
                rect, text, style, ..
            } => surface.draw_label(rect, text, style),
            Self::DrawImage {
                rect,
                image,
                opacity,
                ..
            } => surface.draw_image_id(rect, image, opacity),
            Self::DrawImageFit {
                rect,
                image,
                opacity,
                fit,
                ..
            } => surface.draw_image_id_fit(rect, image, opacity, fit),
            Self::DrawImageTint {
                rect,
                image,
                opacity,
                fit,
                tint,
                ..
            } => surface.draw_image_id_tint(rect, image, opacity, fit, tint),
            Self::DrawImageStyled {
                rect, image, style, ..
            } => surface.draw_image_id_styled(rect, image, style),
            Self::DrawSvgIcon {
                rect,
                icon,
                color,
                opacity,
                ..
            } => surface.draw_svg_icon(rect, icon, Color::rgba(color.r, color.g, color.b, opacity)),
            Self::DrawSvgDocument {
                rect,
                document,
                color,
                opacity,
                ..
            } => surface.draw_svg_document_id(
                rect,
                document,
                Color::rgba(color.r, color.g, color.b, opacity),
            ),
            Self::DrawMask { rect, spec, .. } => surface.draw_mask_rect(rect, spec),
            Self::DrawBlur {
                rect,
                radius,
                opacity,
                ..
            } => surface.draw_blur_fallback(rect, radius, opacity),
            Self::DrawLayer { rect, spec, .. } => surface.draw_layer_fallback(rect, spec),
            Self::BeginLayer { rect, spec, .. } => surface.draw_layer_fallback(rect, spec),
            Self::EndLayer => {}
            Self::PushMask { spec, .. } => {
                let _ = surface.push_mask(spec);
            }
            Self::PushBitmapMask {
                rect,
                image,
                inverted,
                opacity,
                ..
            } => {
                let _ = surface.push_mask(
                    MaskSpec::bitmap(rect, image)
                        .inverted(inverted)
                        .opacity(opacity),
                );
            }
            Self::PopMask => {
                let _ = surface.pop_mask();
            }
            Self::DrawTriangle {
                p0, p1, p2, color, ..
            } => surface.fill_triangle(p0, p1, p2, color),
            Self::DrawGradientTriangle {
                p0, p1, p2, style, ..
            } => surface.fill_gradient_triangle(p0, p1, p2, style),
            Self::DrawTexturedTriangle {
                p0,
                p1,
                p2,
                uv0,
                uv1,
                uv2,
                image,
                opacity,
                ..
            } => surface.fill_textured_triangle(p0, p1, p2, uv0, uv1, uv2, image, opacity),
        }
    }

    pub fn bounds(self) -> Option<Rect> {
        match self {
            Self::Noop | Self::SetClip(_) => None,
            Self::Clear(_) => None,
            Self::FillRect { rect, .. }
            | Self::FillRoundRect { rect, .. }
            | Self::FillGradient { rect, .. }
            | Self::FillStyled { rect, .. }
            | Self::DrawImage { rect, .. }
            | Self::DrawImageFit { rect, .. }
            | Self::DrawImageTint { rect, .. }
            | Self::DrawImageStyled { rect, .. }
            | Self::DrawSvgIcon { rect, .. }
            | Self::DrawSvgDocument { rect, .. }
            | Self::DrawLabel { rect, .. }
            | Self::DrawMask { rect, .. }
            | Self::DrawBlur { rect, .. }
            | Self::DrawLayer { rect, .. }
            | Self::BeginLayer { rect, .. } => Some(rect),
            Self::PushMask { spec, .. } => Some(spec.area),
            Self::PushBitmapMask { rect, .. } => Some(rect),
            Self::EndLayer | Self::PopMask => None,
            Self::FillCircle { center, radius, .. } => {
                let radius = radius as i32;
                Some(Rect::from_edges(
                    center.x - radius,
                    center.y - radius,
                    center.x + radius + 1,
                    center.y + radius + 1,
                ))
            }
            Self::StrokeLine {
                from, to, width, ..
            } => {
                let radius = (width.max(1) as i32 + 1) / 2;
                Some(Rect::from_edges(
                    from.x.min(to.x) - radius,
                    from.y.min(to.y) - radius,
                    from.x.max(to.x) + radius + 1,
                    from.y.max(to.y) + radius + 1,
                ))
            }
            Self::StrokeStyledLine {
                from,
                to,
                style:
                    LineStyle {
                        width,
                        cap_start,
                        cap_end,
                        ..
                    },
                ..
            } => {
                let cap_extra = if cap_start == LineCap::Square || cap_end == LineCap::Square {
                    width.max(1) as i32
                } else {
                    0
                };
                let radius = (width.max(1) as i32 + 1) / 2 + cap_extra;
                Some(Rect::from_edges(
                    from.x.min(to.x) - radius,
                    from.y.min(to.y) - radius,
                    from.x.max(to.x) + radius + 1,
                    from.y.max(to.y) + radius + 1,
                ))
            }
            Self::DrawBorder { rect, style, .. } => {
                let width = style.width as i32;
                Some(Rect::from_edges(
                    rect.x - width,
                    rect.y - width,
                    rect.right() + width,
                    rect.bottom() + width,
                ))
            }
            Self::DrawShadow { rect, style, .. } => {
                let spread = style.spread as i32;
                let blur = style.width as i32;
                Some(Rect::from_edges(
                    rect.x + style.offset.x - spread - blur,
                    rect.y + style.offset.y - spread - blur,
                    rect.right() + style.offset.x + spread + blur,
                    rect.bottom() + style.offset.y + spread + blur,
                ))
            }
            Self::DrawArc {
                center,
                radius,
                style,
                ..
            } => {
                let r = radius as i32 + style.width as i32 + 1;
                Some(Rect::from_edges(
                    center.x - r,
                    center.y - r,
                    center.x + r,
                    center.y + r,
                ))
            }
            Self::DrawText {
                pos, text, scale, ..
            } => {
                let scale = scale.max(1);
                let w = (text.len() as u32)
                    .saturating_mul(6)
                    .saturating_mul(scale as u32)
                    .min(u16::MAX as u32) as u16;
                let h = (7u32.saturating_mul(scale as u32)).min(u16::MAX as u32) as u16;
                Some(Rect::new(pos.x, pos.y, w, h))
            }
            Self::DrawTriangle { p0, p1, p2, .. }
            | Self::DrawGradientTriangle { p0, p1, p2, .. }
            | Self::DrawTexturedTriangle { p0, p1, p2, .. } => Some(Rect::from_edges(
                min3_i32(p0.x, p1.x, p2.x),
                min3_i32(p0.y, p1.y, p2.y),
                max3_i32(p0.x, p1.x, p2.x) + 1,
                max3_i32(p0.y, p1.y, p2.y) + 1,
            )),
        }
    }

    pub fn clipped_bounds(self, clip: Option<Rect>) -> Option<Rect> {
        let bounds = self.bounds()?;
        match clip {
            Some(clip) => {
                let clipped = bounds.clipped_to(clip);
                if clipped.is_empty() {
                    None
                } else {
                    Some(clipped)
                }
            }
            None => Some(bounds),
        }
    }

    pub const fn is_stateful(self) -> bool {
        match self {
            Self::BeginLayer { .. }
            | Self::EndLayer
            | Self::PushMask { .. }
            | Self::PushBitmapMask { .. }
            | Self::PopMask => true,
            _ => false,
        }
    }

    pub fn translated(self, dx: i32, dy: i32) -> Self {
        match self {
            Self::SetClip(rect) => Self::SetClip(translate_rect(rect, dx, dy)),
            Self::FillRect { rect, depth, color } => Self::FillRect {
                rect: translate_rect(rect, dx, dy),
                depth,
                color,
            },
            Self::FillRoundRect {
                rect,
                depth,
                radius,
                color,
            } => Self::FillRoundRect {
                rect: translate_rect(rect, dx, dy),
                depth,
                radius,
                color,
            },
            Self::FillGradient {
                rect,
                depth,
                top,
                bottom,
            } => Self::FillGradient {
                rect: translate_rect(rect, dx, dy),
                depth,
                top,
                bottom,
            },
            Self::FillStyled { rect, depth, style } => Self::FillStyled {
                rect: translate_rect(rect, dx, dy),
                depth,
                style: translate_fill_style(style, dx, dy),
            },
            Self::FillCircle {
                center,
                depth,
                radius,
                color,
            } => Self::FillCircle {
                center: translate_point(center, dx, dy),
                depth,
                radius,
                color,
            },
            Self::StrokeLine {
                from,
                to,
                depth,
                width,
                color,
            } => Self::StrokeLine {
                from: translate_point(from, dx, dy),
                to: translate_point(to, dx, dy),
                depth,
                width,
                color,
            },
            Self::StrokeStyledLine {
                from,
                to,
                depth,
                style,
            } => Self::StrokeStyledLine {
                from: translate_point(from, dx, dy),
                to: translate_point(to, dx, dy),
                depth,
                style,
            },
            Self::DrawBorder { rect, depth, style } => Self::DrawBorder {
                rect: translate_rect(rect, dx, dy),
                depth,
                style,
            },
            Self::DrawShadow { rect, depth, style } => Self::DrawShadow {
                rect: translate_rect(rect, dx, dy),
                depth,
                style,
            },
            Self::DrawArc {
                center,
                depth,
                radius,
                style,
            } => Self::DrawArc {
                center: translate_point(center, dx, dy),
                depth,
                radius,
                style,
            },
            Self::DrawText {
                pos,
                depth,
                text,
                font,
                color,
                scale,
            } => Self::DrawText {
                pos: translate_point(pos, dx, dy),
                depth,
                text,
                font,
                color,
                scale,
            },
            Self::DrawLabel {
                rect,
                depth,
                text,
                style,
            } => Self::DrawLabel {
                rect: translate_rect(rect, dx, dy),
                depth,
                text,
                style,
            },
            Self::DrawImage {
                rect,
                depth,
                image,
                opacity,
            } => Self::DrawImage {
                rect: translate_rect(rect, dx, dy),
                depth,
                image,
                opacity,
            },
            Self::DrawImageFit {
                rect,
                depth,
                image,
                opacity,
                fit,
            } => Self::DrawImageFit {
                rect: translate_rect(rect, dx, dy),
                depth,
                image,
                opacity,
                fit,
            },
            Self::DrawImageTint {
                rect,
                depth,
                image,
                opacity,
                fit,
                tint,
            } => Self::DrawImageTint {
                rect: translate_rect(rect, dx, dy),
                depth,
                image,
                opacity,
                fit,
                tint,
            },
            Self::DrawImageStyled {
                rect,
                depth,
                image,
                style,
            } => Self::DrawImageStyled {
                rect: translate_rect(rect, dx, dy),
                depth,
                image,
                style,
            },
            Self::DrawSvgIcon {
                rect,
                depth,
                icon,
                color,
                opacity,
            } => Self::DrawSvgIcon {
                rect: translate_rect(rect, dx, dy),
                depth,
                icon,
                color,
                opacity,
            },
            Self::DrawSvgDocument {
                rect,
                depth,
                document,
                color,
                opacity,
            } => Self::DrawSvgDocument {
                rect: translate_rect(rect, dx, dy),
                depth,
                document,
                color,
                opacity,
            },
            Self::DrawMask { rect, depth, spec } => Self::DrawMask {
                rect: translate_rect(rect, dx, dy),
                depth,
                spec: translate_mask_spec(spec, dx, dy),
            },
            Self::DrawBlur {
                rect,
                depth,
                radius,
                opacity,
            } => Self::DrawBlur {
                rect: translate_rect(rect, dx, dy),
                depth,
                radius,
                opacity,
            },
            Self::DrawLayer { rect, depth, spec } => Self::DrawLayer {
                rect: translate_rect(rect, dx, dy),
                depth,
                spec: translate_layer_spec(spec, dx, dy),
            },
            Self::BeginLayer { rect, depth, spec } => Self::BeginLayer {
                rect: translate_rect(rect, dx, dy),
                depth,
                spec: translate_layer_spec(spec, dx, dy),
            },
            Self::PushMask { depth, spec } => Self::PushMask {
                depth,
                spec: translate_mask_spec(spec, dx, dy),
            },
            Self::PushBitmapMask {
                rect,
                depth,
                image,
                inverted,
                opacity,
            } => Self::PushBitmapMask {
                rect: translate_rect(rect, dx, dy),
                depth,
                image,
                inverted,
                opacity,
            },
            Self::DrawTriangle {
                p0,
                p1,
                p2,
                depth,
                color,
            } => Self::DrawTriangle {
                p0: translate_point(p0, dx, dy),
                p1: translate_point(p1, dx, dy),
                p2: translate_point(p2, dx, dy),
                depth,
                color,
            },
            Self::DrawGradientTriangle {
                p0,
                p1,
                p2,
                depth,
                style,
            } => Self::DrawGradientTriangle {
                p0: translate_point(p0, dx, dy),
                p1: translate_point(p1, dx, dy),
                p2: translate_point(p2, dx, dy),
                depth,
                style,
            },
            Self::DrawTexturedTriangle {
                p0,
                p1,
                p2,
                uv0,
                uv1,
                uv2,
                depth,
                image,
                opacity,
            } => Self::DrawTexturedTriangle {
                p0: translate_point(p0, dx, dy),
                p1: translate_point(p1, dx, dy),
                p2: translate_point(p2, dx, dy),
                uv0,
                uv1,
                uv2,
                depth,
                image,
                opacity,
            },
            Self::Noop | Self::Clear(_) | Self::EndLayer | Self::PopMask => self,
        }
    }
}

pub struct DrawList<const N: usize> {
    cmds: [DrawCommand; N],
    clips: [Option<Rect>; N],
    len: usize,
    overflowed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DrawChainRun {
    op_start: u16,
    op_end: u16,
    command_start: u16,
    command_end: u16,
    candidate: bool,
    bounds: Rect,
}

impl DrawChainRun {
    const EMPTY: Self = Self {
        op_start: 0,
        op_end: 0,
        command_start: 0,
        command_end: 0,
        candidate: false,
        bounds: Rect::EMPTY,
    };

    const fn len_ops(self) -> usize {
        self.op_end as usize - self.op_start as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DrawChainRunPlan<const OPS: usize> {
    runs: [DrawChainRun; OPS],
    len: usize,
    splits: u32,
    parallel_hints: u32,
    chain_ops: u32,
    candidate_ops: u32,
}

impl<const OPS: usize> DrawChainRunPlan<OPS> {
    const fn new() -> Self {
        Self {
            runs: [DrawChainRun::EMPTY; OPS],
            len: 0,
            splits: 0,
            parallel_hints: 0,
            chain_ops: 0,
            candidate_ops: 0,
        }
    }

    fn push(&mut self, run: DrawChainRun) {
        if self.len >= OPS {
            return;
        }
        self.runs[self.len] = run;
        self.len = self.len.saturating_add(1);
        self.chain_ops = self.chain_ops.saturating_add(run.len_ops() as u32);
        if run.candidate {
            self.candidate_ops = self.candidate_ops.saturating_add(run.len_ops() as u32);
        }
    }
}

impl<const N: usize> DrawList<N> {
    pub const fn new() -> Self {
        Self {
            cmds: [DrawCommand::Noop; N],
            clips: [None; N],
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

    pub fn clear(&mut self) {
        self.len = 0;
        self.overflowed = false;
    }

    pub fn push(&mut self, cmd: DrawCommand) -> bool {
        self.push_clipped(cmd, None)
    }

    pub fn push_clipped(&mut self, cmd: DrawCommand, clip: Option<Rect>) -> bool {
        if self.len >= N {
            self.overflowed = true;
            return false;
        }

        self.cmds[self.len] = cmd;
        self.clips[self.len] = clip;
        self.len += 1;
        true
    }

    pub fn sort_by_depth(&mut self) {
        let mut scan = 0;
        while scan < self.len {
            if self.cmds[scan].is_stateful() {
                return;
            }
            scan += 1;
        }

        let mut i = 1;
        while i < self.len {
            let item = self.cmds[i];
            let item_clip = self.clips[i];
            let mut j = i;
            while j > 0 && self.cmds[j - 1].depth() > item.depth() {
                self.cmds[j] = self.cmds[j - 1];
                self.clips[j] = self.clips[j - 1];
                j -= 1;
            }
            self.cmds[j] = item;
            self.clips[j] = item_clip;
            i += 1;
        }
    }

    pub fn compile_draw_chain<const OPS: usize>(
        &self,
        dirty_clip: Option<Rect>,
    ) -> DrawChain<OPS> {
        let mut chain = DrawChain::new();
        let mut active_clip = None;
        let mut i = 0usize;
        while i < self.len {
            let cmd = self.cmds[i];
            let clip = effective_chain_clip(self.clips[i], dirty_clip);
            let Some(op_kind) = chain_op_kind(cmd) else {
                i += 1;
                continue;
            };

            let bounds = chain_op_bounds(cmd, clip);
            if chain_op_culls_with_clip(op_kind) {
                let Some(command_bounds) = cmd.bounds() else {
                    i += 1;
                    continue;
                };
                let clipped = match clip {
                    Some(clip) => command_bounds.clipped_to(clip),
                    None => command_bounds,
                };
                if clipped.is_empty() {
                    i += 1;
                    continue;
                }
            } else if let Some(clip) = clip {
                if clip.is_empty() {
                    i += 1;
                    continue;
                }
            }

            if clip != active_clip {
                let clip_bounds = clip.unwrap_or(Rect::EMPTY);
                let _ = chain.push(DrawChainOp::new(
                    DrawChainOpKind::Clip,
                    None,
                    clip_bounds,
                    clip,
                    saturating_u16(i),
                    0,
                ));
                active_clip = clip;
            }

            let _ = chain.push(DrawChainOp::new(
                op_kind,
                cmd.task_kind(),
                bounds,
                clip,
                saturating_u16(i),
                1,
            ));
            i += 1;
        }
        chain
    }

    pub fn execute(&self, surface: &mut Surface) {
        self.execute_on(surface);
    }

    pub fn execute_on<B: RenderBackend>(&self, backend: &mut B) {
        let mut stats = RenderStats::new();
        self.execute_tracked_on(backend, &mut stats);
    }

    pub fn execute_tracked_on<B: RenderBackend>(&self, backend: &mut B, stats: &mut RenderStats) {
        stats.clear();
        stats.mark_overflowed(self.overflowed);
        let dispatch = DrawBackendDispatch::from_capabilities(backend.capabilities());
        self.execute_run_planner::<B, DEFAULT_DRAW_CHAIN_OPS>(backend, None, dispatch, stats);
    }

    pub fn execute_dirty_on<B: RenderBackend, const D: usize>(
        &self,
        backend: &mut B,
        dirty: &DirtyRegion<D>,
    ) {
        let mut stats = RenderStats::new();
        self.execute_dirty_tracked_on(backend, dirty, &mut stats);
    }

    pub fn execute_dirty_tracked_on<B: RenderBackend, const D: usize>(
        &self,
        backend: &mut B,
        dirty: &DirtyRegion<D>,
        stats: &mut RenderStats,
    ) {
        stats.clear();
        stats.mark_dirty_rects(dirty.len());
        stats.mark_overflowed(self.overflowed || dirty.overflowed());
        let dispatch = DrawBackendDispatch::from_capabilities(backend.capabilities());
        let mut dirty_index = 0;
        while dirty_index < dirty.len() {
            let Some(dirty_rect) = dirty.rect(dirty_index) else {
                dirty_index += 1;
                continue;
            };
            if dirty_rect.is_empty() {
                dirty_index += 1;
                continue;
            }

            stats.dirty_pass_started();
            self.execute_run_planner::<B, DEFAULT_DRAW_CHAIN_OPS>(
                backend,
                Some(dirty_rect),
                dispatch,
                stats,
            );
            dirty_index += 1;
        }
    }

    fn plan_draw_chain_runs<const OPS: usize>(
        &self,
        chain: &DrawChain<OPS>,
        capabilities: BackendCapabilities,
    ) -> DrawChainRunPlan<OPS> {
        let mut plan = DrawChainRunPlan::new();
        let ops = chain.ops();
        if ops.is_empty() {
            return plan;
        }

        let supports_chain = capabilities.draw_chain && capabilities.max_chain_ops > 0;
        let mut max_ops = capabilities.max_chain_ops.max(1).min(ops.len().max(1));
        if !supports_chain {
            max_ops = ops.len().max(1);
        }

        let mut current = DrawChainRun::EMPTY;
        let mut started = false;
        let mut run_len = 0usize;
        let mut run_has_render = false;
        let mut run_candidate = false;
        let mut run_index = 0usize;

        while run_index < ops.len() {
            let op = ops[run_index];
            let op_candidate = supports_chain && op.kind.supports(capabilities);
            let split_by_state = started && run_has_render && op.kind.is_stateful();
            let split_by_candidate = started && run_candidate != op_candidate;
            let split_by_capacity = supports_chain && started && run_len >= max_ops;
            let split = split_by_state || split_by_candidate || split_by_capacity;

            if split {
                plan.push(current);
                plan.splits = plan.splits.saturating_add(1);
                current = DrawChainRun::EMPTY;
                started = false;
                run_len = 0;
                run_has_render = false;
                run_candidate = false;
            }

            if !started {
                current = DrawChainRun {
                    op_start: saturating_u16(run_index),
                    op_end: saturating_u16(run_index.saturating_add(1)),
                    command_start: op.command_start,
                    command_end: op.command_start.saturating_add(op.command_len),
                    candidate: op_candidate,
                    bounds: op.bounds,
                };
                run_candidate = op_candidate;
                run_has_render = op.command_len > 0;
                run_len = 1;
                started = true;
            } else {
                current.op_end = saturating_u16(run_index.saturating_add(1));
                current.candidate = current.candidate && op_candidate;
                if op.command_len > 0 {
                    run_has_render = true;
                    let command_start = op.command_start as usize;
                    let command_end = command_start
                        .saturating_add(op.command_len as usize);
                    if command_start < current.command_start as usize {
                        current.command_start = saturating_u16(command_start);
                    }
                    if command_end > current.command_end as usize {
                        current.command_end = saturating_u16(command_end);
                    }
                }
                current.bounds = current.bounds.union(op.bounds);
                run_len = run_len.saturating_add(1);
            }

            run_index += 1;
        }

        if started {
            plan.push(current);
        }

        let mut index = 1usize;
        while index < plan.len {
            let previous = plan.runs[index - 1];
            let current = plan.runs[index];
            if previous.candidate && current.candidate && !previous.bounds.intersects(current.bounds) {
                plan.parallel_hints = plan.parallel_hints.saturating_add(1);
            }
            index += 1;
        }
        plan
    }

    fn execute_run_planner<B: RenderBackend, const OPS: usize>(
        &self,
        backend: &mut B,
        dirty_clip: Option<Rect>,
        dispatch: DrawBackendDispatch,
        stats: &mut RenderStats,
    ) {
        let capabilities = backend.capabilities();
        let chain = self.compile_draw_chain::<OPS>(dirty_clip);
        let run_plan = self.plan_draw_chain_runs::<OPS>(&chain, capabilities);
        let mut split_hints = run_plan.splits;
        while split_hints > 0 {
            stats.mark_draw_chain_split();
            split_hints -= 1;
        }
        let mut parallel_hints = run_plan.parallel_hints;
        while parallel_hints > 0 {
            stats.mark_draw_chain_parallel_hint();
            parallel_hints -= 1;
        }

        let mut active_clip = None;
        let mut run_index = 0usize;
        while run_index < run_plan.len {
            let run = run_plan.runs[run_index];
            let mut run_chain: DrawChain<OPS> = DrawChain::new();
            let mut op_index = run.op_start as usize;
            while op_index < run.op_end as usize {
                let _ = run_chain.push(chain.ops()[op_index]);
                op_index += 1;
            }
            if run_chain.is_empty() {
                run_index += 1;
                continue;
            }

            let result = if run.candidate {
                backend.submit_draw_chain(&run_chain, stats)
            } else {
                DrawChainSubmitResult::Fallback
            };
            if run.candidate {
                stats.mark_draw_chain_submit(run_chain.stats(), result);
            } else {
                stats.mark_draw_chain_submit(run_chain.stats(), DrawChainSubmitResult::Fallback);
            }
            stats.mark_draw_chain_run_result(result);
            if result != DrawChainSubmitResult::Submitted {
                self.execute_command_range(
                    backend,
                    run.command_start as usize,
                    run.command_end as usize,
                    dirty_clip,
                    dispatch,
                    &mut active_clip,
                    stats,
                );
                if !run.candidate {
                    active_clip = None;
                }
            } else {
                active_clip = None;
            }
            run_index += 1;
        }

        if active_clip.is_some() {
            backend.set_clip(None);
            stats.clip_changed();
        }
        backend.clear_masks();
    }

    fn execute_command_range<B: RenderBackend>(
        &self,
        backend: &mut B,
        start: usize,
        end: usize,
        dirty_clip: Option<Rect>,
        dispatch: DrawBackendDispatch,
        active_clip: &mut Option<Rect>,
        stats: &mut RenderStats,
    ) {
        if start >= end {
            return;
        }

        let mut i = start;
        while i < end {
            let cmd = self.cmds[i];
            let command_clip = self.clips[i];
            let effective_clip = match dirty_clip {
                Some(dirty) => match command_clip {
                    Some(clip) => Some(clip.clipped_to(dirty)),
                    None => Some(dirty),
                },
                None => command_clip,
            };
            stats.command_seen();

            match cmd {
                DrawCommand::BeginLayer { rect, spec, .. } => {
                    let end = self.find_layer_end(i + 1);
                    let Some(layer_bounds) = cmd.bounds() else {
                        i = end.saturating_add(1);
                        continue;
                    };
                    if let Some(dirty) = dirty_clip {
                        let Some(effective_clip) = effective_clip else {
                            i = end.saturating_add(1);
                            continue;
                        };
                        if effective_clip.is_empty() || !layer_bounds.intersects(dirty) {
                            stats.command_clipped();
                            i = end.saturating_add(1);
                            continue;
                        }
                        let layer_clip = Some(effective_clip);
                        if layer_clip != *active_clip {
                            backend.set_clip(layer_clip);
                            stats.effective_clip_changed();
                            *active_clip = layer_clip;
                        }
                        if let Some(kind) = cmd.task_kind() {
                            let _ = backend.draw_dispatched_layer_commands(
                                rect,
                                spec,
                                &self.cmds[i + 1..end],
                                &self.clips[i + 1..end],
                                dispatch.classify(kind),
                                stats,
                            );
                        } else {
                            let _ = backend.draw_layer_commands(
                                rect,
                                spec,
                                &self.cmds[i + 1..end],
                                &self.clips[i + 1..end],
                                stats,
                            );
                        }
                        stats.mark_command_kind(cmd);
                        stats.command_drawn(Some(layer_bounds.clipped_to(effective_clip)));
                        i = end.saturating_add(1);
                        continue;
                    }
                    let layer_clip = command_clip;
                    if layer_clip != *active_clip {
                        backend.set_clip(layer_clip);
                        stats.clip_changed();
                        *active_clip = layer_clip;
                    }
                    if let Some(kind) = cmd.task_kind() {
                        let _ = backend.draw_dispatched_layer_commands(
                            rect,
                            spec,
                            &self.cmds[i + 1..end],
                            &self.clips[i + 1..end],
                            dispatch.classify(kind),
                            stats,
                        );
                    } else {
                        let _ = backend.draw_layer_commands(
                            rect,
                            spec,
                            &self.cmds[i + 1..end],
                            &self.clips[i + 1..end],
                            stats,
                        );
                    }
                    stats.mark_command_kind(cmd);
                    stats.command_drawn(cmd.clipped_bounds(layer_clip));
                    i = end.saturating_add(1);
                    continue;
                }
                DrawCommand::EndLayer => {
                    i += 1;
                    continue;
                }
                DrawCommand::PushMask { spec, .. } => {
                    if !backend.push_mask(spec) {
                        stats.mark_mask_stack_overflow();
                    }
                    if let Some(kind) = cmd.task_kind() {
                        if dirty_clip.is_some() {
                            stats.effective_clip_changed();
                        }
                        stats.mark_draw_dispatch_for(kind, dispatch.classify(kind));
                    }
                    stats.mark_command_kind(cmd);
                    i += 1;
                    continue;
                }
                DrawCommand::PushBitmapMask {
                    rect,
                    image,
                    inverted,
                    opacity,
                    ..
                } => {
                    let spec = MaskSpec::bitmap(rect, image)
                        .inverted(inverted)
                        .opacity(opacity);
                    if !backend.push_mask(spec) {
                        stats.mark_mask_stack_overflow();
                    }
                    if let Some(kind) = cmd.task_kind() {
                        if dirty_clip.is_some() {
                            stats.effective_clip_changed();
                        }
                        stats.mark_draw_dispatch_for(kind, dispatch.classify(kind));
                    }
                    stats.mark_command_kind(cmd);
                    i += 1;
                    continue;
                }
                DrawCommand::PopMask => {
                    let _ = backend.pop_mask();
                    i += 1;
                    continue;
                }
                _ => {}
            }

            if dirty_clip.is_some() && effective_clip.is_none_or(|clip| clip.is_empty()) {
                stats.command_clipped();
                i += 1;
                continue;
            }

            let bounds = cmd.bounds();
            if let Some(dirty) = dirty_clip {
                if let Some(bounds) = bounds {
                    if !bounds.intersects(dirty) {
                        stats.command_clipped();
                        i += 1;
                        continue;
                    }
                }
            } else if let Some(bounds) = bounds {
                let _ = bounds;
            }

            let clip = if dirty_clip.is_some() {
                effective_clip
            } else {
                command_clip
            };
            if clip != *active_clip {
                backend.set_clip(clip);
                if dirty_clip.is_some() {
                    stats.effective_clip_changed();
                } else {
                    stats.clip_changed();
                }
                *active_clip = clip;
            }

            if let Some(kind) = cmd.task_kind() {
                backend.draw_dispatched_command(cmd, dispatch.classify(kind), stats);
            } else {
                backend.draw_command(cmd);
            }
            stats.mark_command_kind(cmd);
            stats.command_drawn(if dirty_clip.is_some() {
                match (bounds, effective_clip) {
                    (Some(bounds), Some(effective_clip)) => Some(bounds.clipped_to(effective_clip)),
                    (None, Some(effective_clip)) => Some(effective_clip),
                    _ => None,
                }
            } else {
                cmd.clipped_bounds(command_clip)
            });
            i += 1;
        }
    }

    fn find_layer_end(&self, start: usize) -> usize {
        let mut depth = 0usize;
        let mut i = start;
        while i < self.len {
            match self.cmds[i] {
                DrawCommand::BeginLayer { .. } => depth = depth.saturating_add(1),
                DrawCommand::EndLayer => {
                    if depth == 0 {
                        return i;
                    }
                    depth -= 1;
                }
                _ => {}
            }
            i += 1;
        }
        self.len
    }
}

fn effective_chain_clip(command_clip: Option<Rect>, dirty_clip: Option<Rect>) -> Option<Rect> {
    match dirty_clip {
        Some(dirty) => match command_clip {
            Some(command) => Some(command.clipped_to(dirty)),
            None => Some(dirty),
        },
        None => command_clip,
    }
}

fn chain_op_kind(cmd: DrawCommand) -> Option<DrawChainOpKind> {
    match cmd {
        DrawCommand::Noop => None,
        DrawCommand::SetClip(_) => Some(DrawChainOpKind::Clip),
        DrawCommand::FillRect { color, .. } => {
            if color.a == 255 {
                Some(DrawChainOpKind::SolidFill)
            } else {
                Some(DrawChainOpKind::AlphaFill)
            }
        }
        DrawCommand::FillStyled { style, .. }
            if style.radius == 0
                && style.gradient == GradientStyle::None
                && style.blend == BlendMode::Normal =>
        {
            if style.color.a == 255 {
                Some(DrawChainOpKind::SolidFill)
            } else {
                Some(DrawChainOpKind::AlphaFill)
            }
        }
        DrawCommand::DrawImage { opacity, .. } | DrawCommand::DrawImageFit { opacity, .. } => {
            if opacity == 255 {
                Some(DrawChainOpKind::ImageBlit)
            } else {
                Some(DrawChainOpKind::ImageBlend)
            }
        }
        DrawCommand::DrawImageTint { .. } => Some(DrawChainOpKind::ImageBlend),
        DrawCommand::DrawImageStyled { style, .. }
            if style.blend == BlendMode::Normal
                && style.tint.is_none()
                && style.clip_radius == 0
                && !style.tile =>
        {
            if style.opacity == 255 {
                Some(DrawChainOpKind::ImageBlit)
            } else {
                Some(DrawChainOpKind::ImageBlend)
            }
        }
        DrawCommand::DrawMask { .. }
        | DrawCommand::PushMask { .. }
        | DrawCommand::PushBitmapMask { .. } => Some(DrawChainOpKind::MaskEnter),
        DrawCommand::PopMask => Some(DrawChainOpKind::MaskExit),
        DrawCommand::BeginLayer { .. } => Some(DrawChainOpKind::LayerEnter),
        DrawCommand::EndLayer => Some(DrawChainOpKind::LayerExit),
        DrawCommand::Clear(_)
        | DrawCommand::FillRoundRect { .. }
        | DrawCommand::FillGradient { .. }
        | DrawCommand::FillStyled { .. }
        | DrawCommand::FillCircle { .. }
        | DrawCommand::StrokeLine { .. }
        | DrawCommand::StrokeStyledLine { .. }
        | DrawCommand::DrawBorder { .. }
        | DrawCommand::DrawShadow { .. }
        | DrawCommand::DrawArc { .. }
        | DrawCommand::DrawText { .. }
        | DrawCommand::DrawLabel { .. }
        | DrawCommand::DrawImageStyled { .. }
        | DrawCommand::DrawSvgIcon { .. }
        | DrawCommand::DrawSvgDocument { .. }
        | DrawCommand::DrawBlur { .. }
        | DrawCommand::DrawLayer { .. }
        | DrawCommand::DrawTriangle { .. }
        | DrawCommand::DrawGradientTriangle { .. }
        | DrawCommand::DrawTexturedTriangle { .. } => Some(DrawChainOpKind::FallbackRange),
    }
}

fn chain_op_bounds(cmd: DrawCommand, clip: Option<Rect>) -> Rect {
    match cmd {
        DrawCommand::SetClip(rect) => rect,
        _ => match (cmd.bounds(), clip) {
            (Some(bounds), Some(clip)) => bounds.clipped_to(clip),
            (Some(bounds), None) => bounds,
            (None, Some(clip)) => clip,
            (None, None) => Rect::EMPTY,
        },
    }
}

fn chain_op_culls_with_clip(op: DrawChainOpKind) -> bool {
    matches!(
        op,
        DrawChainOpKind::SolidFill
            | DrawChainOpKind::AlphaFill
            | DrawChainOpKind::ImageBlit
            | DrawChainOpKind::ImageBlend
            | DrawChainOpKind::FallbackRange
    )
}

fn saturating_u16(value: usize) -> u16 {
    value.min(u16::MAX as usize) as u16
}

fn min3_i32(a: i32, b: i32, c: i32) -> i32 {
    a.min(b).min(c)
}

fn max3_i32(a: i32, b: i32, c: i32) -> i32 {
    a.max(b).max(c)
}

fn translate_point(point: Point, dx: i32, dy: i32) -> Point {
    Point::new(point.x.saturating_add(dx), point.y.saturating_add(dy))
}

fn translate_rect(rect: Rect, dx: i32, dy: i32) -> Rect {
    Rect::new(
        rect.x.saturating_add(dx),
        rect.y.saturating_add(dy),
        rect.w,
        rect.h,
    )
}

fn translate_mask_spec(mut spec: MaskSpec, dx: i32, dy: i32) -> MaskSpec {
    spec.area = translate_rect(spec.area, dx, dy);
    spec
}

fn translate_layer_spec(mut spec: LayerSpec, dx: i32, dy: i32) -> LayerSpec {
    if let Some(mask) = spec.mask {
        spec.mask = Some(translate_mask_spec(mask, dx, dy));
    }
    spec
}

fn translate_fill_style(mut style: FillStyle, dx: i32, dy: i32) -> FillStyle {
    style.gradient = match style.gradient {
        GradientStyle::Linear {
            start,
            end,
            start_color,
            end_color,
        } => GradientStyle::Linear {
            start: translate_point(start, dx, dy),
            end: translate_point(end, dx, dy),
            start_color,
            end_color,
        },
        GradientStyle::Radial {
            center,
            radius,
            inner,
            outer,
        } => GradientStyle::Radial {
            center: translate_point(center, dx, dy),
            radius,
            inner,
            outer,
        },
        GradientStyle::Conical { center, start, end } => GradientStyle::Conical {
            center: translate_point(center, dx, dy),
            start,
            end,
        },
        other => other,
    };
    style
}
