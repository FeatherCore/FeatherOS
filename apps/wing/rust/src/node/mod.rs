use crate::{
    action::ActionId,
    key::UiKey,
    layout::{ContentInset, LayoutRule, LayoutSpace},
    spec::{UiKind, UiSpec},
};
use fhre::{
    fixed_to_i32, Camera, Color, Entity, Fixed16, FontId, ImageFit, ImageId, Point, Rect,
    RenderNode, Size, SvgId, Transform3D,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiFrameStats {
    pub node_count: usize,
    pub created: usize,
    pub updated: usize,
    pub removed: usize,
    pub dirty_count: usize,
    pub overflowed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHit {
    pub key: UiKey,
    pub action: ActionId,
    pub rect: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNode {
    pub key: UiKey,
    pub kind: UiKind,
    pub seen: bool,
}

impl UiNode {
    pub const EMPTY: Self = Self {
        key: UiKey(0),
        kind: UiKind::Panel,
        seen: false,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Parent {
    pub key: UiKey,
    pub entity: Option<Entity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Children {
    pub first_child: Option<Entity>,
    pub next_sibling: Option<Entity>,
}

impl Children {
    pub const EMPTY: Self = Self {
        first_child: None,
        next_sibling: None,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutBox {
    pub space: LayoutSpace,
    pub content_inset: ContentInset,
    pub rule: LayoutRule,
    pub transform: Transform3D,
    pub size: Size,
    pub line_to: Point,
}

impl LayoutBox {
    pub const EMPTY: Self = Self {
        space: LayoutSpace::Screen,
        content_inset: ContentInset::ZERO,
        rule: LayoutRule::None,
        transform: Transform3D::IDENTITY,
        size: Size::new(0, 0),
        line_to: Point::new(0, 0),
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutResult {
    pub rect: Rect,
    pub depth: Fixed16,
    pub line_to: Point,
}

impl LayoutResult {
    pub const EMPTY: Self = Self {
        rect: Rect::EMPTY,
        depth: 0,
        line_to: Point::new(0, 0),
    };

    pub fn project(spec: UiSpec, camera: &Camera) -> Self {
        let projected = RenderNode::new(spec.transform, spec.size).project(camera);
        let line_to = if matches!(spec.kind, UiKind::Line) {
            let z = fixed_to_i32(spec.transform.position.z);
            let end = RenderNode::new(
                Transform3D::screen(spec.line_to.x, spec.line_to.y, z),
                Size::new(1, 1),
            )
            .project(camera);
            Point::new(end.rect.x, end.rect.y)
        } else {
            Point::new(0, 0)
        };

        Self {
            rect: projected.rect,
            depth: projected.depth,
            line_to,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Visual {
    pub color: Color,
    pub radius: u16,
    pub scale: u16,
}

impl Visual {
    pub const EMPTY: Self = Self {
        color: Color::TRANSPARENT,
        radius: 0,
        scale: 1,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Opacity {
    pub value: u8,
}

impl Opacity {
    pub const OPAQUE: Self = Self { value: 255 };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Clip {
    pub rect: Rect,
    pub projected: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedClip {
    pub rect: Rect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextNode {
    pub text: &'static str,
    pub font: FontId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IconNode {
    pub icon: SvgId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageNode {
    pub image: ImageId,
    pub fit: ImageFit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Button {
    pub action: ActionId,
}
