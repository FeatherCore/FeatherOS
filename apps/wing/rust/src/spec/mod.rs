use crate::{
    action::ActionId,
    animation::Animation,
    key::UiKey,
    layout::{ContentInset, GridLayout, LayoutRule, LayoutSpace, StackLayout},
};
use fhre::{
    fixed_to_i32, Camera, Color, DrawCommand, FontId, ImageFit, ImageId, Point, Rect, RenderNode,
    Size, SvgId, Transform3D,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiKind {
    Panel,
    Tile,
    Text,
    Bar,
    Circle,
    Line,
    Icon,
    Image,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiSpec {
    pub key: UiKey,
    pub parent: Option<UiKey>,
    pub space: LayoutSpace,
    pub content_inset: ContentInset,
    pub layout: LayoutRule,
    pub kind: UiKind,
    pub transform: Transform3D,
    pub size: Size,
    pub color: Color,
    pub opacity: u8,
    pub clip: Option<Rect>,
    pub radius: u16,
    pub scale: u16,
    pub line_to: Point,
    pub text: Option<&'static str>,
    pub icon: Option<SvgId>,
    pub image: Option<ImageId>,
    pub image_fit: ImageFit,
    pub animation: Option<Animation>,
    pub action: Option<ActionId>,
}

impl UiSpec {
    pub const EMPTY: Self = Self {
        key: UiKey(0),
        parent: None,
        space: LayoutSpace::Screen,
        content_inset: ContentInset::ZERO,
        layout: LayoutRule::None,
        kind: UiKind::Panel,
        transform: Transform3D::IDENTITY,
        size: Size::new(0, 0),
        color: Color::TRANSPARENT,
        opacity: 255,
        clip: None,
        radius: 0,
        scale: 1,
        line_to: Point::new(0, 0),
        text: None,
            icon: None,
            image: None,
            image_fit: ImageFit::Stretch,
            animation: None,
            action: None,
        };

    pub const fn rect(
        key: UiKey,
        kind: UiKind,
        rect: Rect,
        z: i32,
        color: Color,
        radius: u16,
    ) -> Self {
        Self {
            key,
            parent: None,
            space: LayoutSpace::Screen,
            content_inset: ContentInset::ZERO,
            layout: LayoutRule::None,
            kind,
            transform: Transform3D::screen(rect.x, rect.y, z),
            size: Size::new(rect.w, rect.h),
            color,
            opacity: 255,
            clip: None,
            radius,
            scale: 1,
            line_to: Point::new(0, 0),
            text: None,
            icon: None,
            image: None,
            image_fit: ImageFit::Stretch,
            animation: None,
            action: None,
        }
    }

    pub const fn line(key: UiKey, from: Point, to: Point, z: i32, width: u16, color: Color) -> Self {
        Self {
            key,
            parent: None,
            space: LayoutSpace::Screen,
            content_inset: ContentInset::ZERO,
            layout: LayoutRule::None,
            kind: UiKind::Line,
            transform: Transform3D::screen(from.x, from.y, z),
            size: Size::new(1, 1),
            color,
            opacity: 255,
            clip: None,
            radius: width,
            scale: 1,
            line_to: to,
            text: None,
            icon: None,
            image: None,
            image_fit: ImageFit::Stretch,
            animation: None,
            action: None,
        }
    }

    pub fn text(key: UiKey, x: i32, y: i32, z: i32, text: &'static str, color: Color, scale: u16) -> Self {
        let scale = scale.max(1);
        let w = text.len().saturating_mul(6).saturating_mul(scale as usize).min(u16::MAX as usize);
        let h = 8usize.saturating_mul(scale as usize).min(u16::MAX as usize);
        Self {
            key,
            parent: None,
            space: LayoutSpace::Screen,
            content_inset: ContentInset::ZERO,
            layout: LayoutRule::None,
            kind: UiKind::Text,
            transform: Transform3D::screen(x, y, z),
            size: Size::new(w as u16, h as u16),
            color,
            opacity: 255,
            clip: None,
            radius: 0,
            scale,
            line_to: Point::new(0, 0),
            text: Some(text),
            icon: None,
            image: None,
            image_fit: ImageFit::Stretch,
            animation: None,
            action: None,
        }
    }

    pub const fn icon(key: UiKey, rect: Rect, z: i32, icon: SvgId, color: Color) -> Self {
        Self {
            key,
            parent: None,
            space: LayoutSpace::Screen,
            content_inset: ContentInset::ZERO,
            layout: LayoutRule::None,
            kind: UiKind::Icon,
            transform: Transform3D::screen(rect.x, rect.y, z),
            size: Size::new(rect.w, rect.h),
            color,
            opacity: 255,
            clip: None,
            radius: 0,
            scale: 1,
            line_to: Point::new(0, 0),
            text: None,
            icon: Some(icon),
            image: None,
            image_fit: ImageFit::Stretch,
            animation: None,
            action: None,
        }
    }

    pub const fn image(key: UiKey, rect: Rect, z: i32, image: ImageId, opacity: u8) -> Self {
        Self {
            key,
            parent: None,
            space: LayoutSpace::Screen,
            content_inset: ContentInset::ZERO,
            layout: LayoutRule::None,
            kind: UiKind::Image,
            transform: Transform3D::screen(rect.x, rect.y, z),
            size: Size::new(rect.w, rect.h),
            color: Color::WHITE,
            opacity,
            clip: None,
            radius: 0,
            scale: 1,
            line_to: Point::new(0, 0),
            text: None,
            icon: None,
            image: Some(image),
            image_fit: ImageFit::Stretch,
            animation: None,
            action: None,
        }
    }

    pub const fn image_fit(
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
    ) -> Self {
        let mut spec = Self::image(key, rect, z, image, opacity);
        spec.image_fit = fit;
        spec
    }

    pub const fn image_tint(
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
        tint: Color,
    ) -> Self {
        let mut spec = Self::image_fit(key, rect, z, image, opacity, fit);
        spec.color = tint;
        spec
    }

    pub const fn with_action(mut self, action: ActionId) -> Self {
        self.action = Some(action);
        self
    }

    pub const fn with_parent(mut self, parent: UiKey) -> Self {
        self.parent = Some(parent);
        self
    }

    pub const fn with_local_parent(mut self, parent: UiKey) -> Self {
        self.parent = Some(parent);
        self.space = LayoutSpace::Parent;
        self
    }

    pub const fn with_content_parent(mut self, parent: UiKey) -> Self {
        self.parent = Some(parent);
        self.space = LayoutSpace::Content;
        self
    }

    pub const fn with_content_inset(mut self, inset: ContentInset) -> Self {
        self.content_inset = inset;
        self
    }

    pub const fn with_grid_cell(mut self, grid: GridLayout, index: u8) -> Self {
        self.layout = LayoutRule::GridCell { grid, index };
        self
    }

    pub const fn with_stack_item(mut self, stack: StackLayout, index: u8) -> Self {
        self.layout = LayoutRule::StackItem { stack, index };
        self
    }

    pub const fn with_opacity(mut self, opacity: u8) -> Self {
        self.opacity = opacity;
        self
    }

    pub const fn with_clip(mut self, clip: Rect) -> Self {
        self.clip = Some(clip);
        self
    }

    pub const fn with_animation(mut self, animation: Animation) -> Self {
        self.animation = Some(animation);
        self
    }

    pub fn bounds(self, camera: &Camera) -> Option<Rect> {
        self.command(camera)
            .and_then(|command| command.clipped_bounds(self.projected_clip(camera)))
    }

    pub(crate) fn projected_clip(self, camera: &Camera) -> Option<Rect> {
        self.clip.map(|clip| project_clip_rect(clip, camera))
    }

    pub(crate) fn command(self, camera: &Camera) -> Option<DrawCommand> {
        let projected = RenderNode::new(self.transform, self.size).project(camera);
        let color = apply_opacity(self.color, self.opacity);
        match self.kind {
            UiKind::Panel | UiKind::Tile => Some(DrawCommand::FillRoundRect {
                rect: projected.rect,
                depth: projected.depth,
                radius: self.radius,
                color,
            }),
            UiKind::Bar => {
                if self.radius == 0 {
                    Some(DrawCommand::FillRect {
                        rect: projected.rect,
                        depth: projected.depth,
                        color,
                    })
                } else {
                    Some(DrawCommand::FillRoundRect {
                        rect: projected.rect,
                        depth: projected.depth,
                        radius: self.radius,
                        color,
                    })
                }
            }
            UiKind::Circle => {
                let radius = projected.rect.w.min(projected.rect.h) / 2;
                Some(DrawCommand::FillCircle {
                    center: Point::new(
                        projected.rect.x + projected.rect.w as i32 / 2,
                        projected.rect.y + projected.rect.h as i32 / 2,
                    ),
                    depth: projected.depth,
                    radius,
                    color,
                })
            }
            UiKind::Line => {
                let z = fixed_to_i32(self.transform.position.z);
                let end = RenderNode::new(
                    Transform3D::screen(self.line_to.x, self.line_to.y, z),
                    Size::new(1, 1),
                )
                .project(camera);
                Some(DrawCommand::StrokeLine {
                    from: Point::new(projected.rect.x, projected.rect.y),
                    to: Point::new(end.rect.x, end.rect.y),
                    depth: projected.depth,
                    width: self.radius.max(1),
                    color,
                })
            }
            UiKind::Text => self.text.map(|text| DrawCommand::DrawText {
                pos: Point::new(projected.rect.x, projected.rect.y),
                depth: projected.depth,
                text,
                font: FontId(0),
                color,
                scale: self.scale,
            }),
            UiKind::Icon => self.icon.map(|icon| DrawCommand::DrawSvgIcon {
                rect: projected.rect,
                depth: projected.depth,
                icon,
                color,
                opacity: color.a,
            }),
            UiKind::Image => self.image.map(|image| {
                if self.color != Color::WHITE {
                    DrawCommand::DrawImageTint {
                        rect: projected.rect,
                        depth: projected.depth,
                        image,
                        opacity: self.opacity,
                        fit: self.image_fit,
                        tint: self.color,
                    }
                } else if self.image_fit != ImageFit::Stretch {
                    DrawCommand::DrawImageFit {
                        rect: projected.rect,
                        depth: projected.depth,
                        image,
                        opacity: self.opacity,
                        fit: self.image_fit,
                    }
                } else {
                    DrawCommand::DrawImage {
                        rect: projected.rect,
                        depth: projected.depth,
                        image,
                        opacity: self.opacity,
                    }
                }
            }),
        }
    }
}


fn apply_opacity(color: Color, opacity: u8) -> Color {
    Color::rgba(
        color.r,
        color.g,
        color.b,
        ((color.a as u16 * opacity as u16) / 255) as u8,
    )
}

fn project_clip_rect(clip: Rect, camera: &Camera) -> Rect {
    RenderNode::new(Transform3D::screen(clip.x, clip.y, 0), Size::new(clip.w, clip.h))
        .project(camera)
        .rect
}
