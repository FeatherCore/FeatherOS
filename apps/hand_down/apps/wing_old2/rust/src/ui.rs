use crate::app::AppId;
use crate::core::{Entity, FixedList};
use crate::math::{Color, Rect};
use crate::render::{font_text_bounds, EffectKind, EffectParams, ImageId, VectorIcon};

pub const UI_SPEC_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Node {
    pub parent: Option<Entity>,
    pub first_child: Option<Entity>,
    pub next_sibling: Option<Entity>,
}

impl Node {
    pub const fn root() -> Self {
        Self {
            parent: None,
            first_child: None,
            next_sibling: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Layout {
    pub rect: Rect,
}

impl Layout {
    pub const fn new(x: i32, y: i32, w: u16, h: u16) -> Self {
        Self {
            rect: Rect::new(x, y, w, h),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transform {
    pub z: i16,
    pub scale: u8,
    pub rotation: i16,
}

impl Transform {
    pub const fn new(z: i16) -> Self {
        Self {
            z,
            scale: 255,
            rotation: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Visibility {
    pub visible: bool,
    pub alpha: u8,
}

impl Default for Visibility {
    fn default() -> Self {
        Self {
            visible: true,
            alpha: 255,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visual {
    Rect { color: Color },
    RoundRect { color: Color, radius: u8 },
    RoundRectGradient {
        top: Color,
        bottom: Color,
        radius: u8,
    },
    ShadowRoundRect {
        color: Color,
        radius: u8,
        offset_x: i16,
        offset_y: i16,
        blur: u8,
        spread: u8,
    },
    Circle { color: Color },
    Image { image: ImageId, tint: Color },
    Icon { icon: VectorIcon, color: Color },
    Text { text: &'static str, color: Color, scale: u8 },
    Effect { effect: EffectKind, params: EffectParams },
}

impl Visual {
    pub fn bounds(self, rect: Rect) -> Rect {
        match self {
            Self::ShadowRoundRect {
                offset_x,
                offset_y,
                blur,
                spread,
                ..
            } => shadow_bounds(rect, offset_x, offset_y, blur, spread),
            Self::Text { text, scale, .. } => {
                font_text_bounds(rect.x, rect.y, text, scale.max(1))
            }
            _ => rect,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitBox {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonAction {
    OpenSettings,
    OpenNotification,
    OpenAppSwitcher,
    LaunchTerminal,
    LaunchApp(AppId),
    BackHome,
    ToggleTheme,
    TogglePreviewEffect,
    ToggleWifi,
    ToggleBluetooth,
    ToggleAirplane,
    ToggleDnd,
    ToggleLight,
    ToggleSync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Button {
    pub action: ButtonAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Always,
    Home,
    Notification,
    AppSwitcher,
    ExternalApp,
    Settings,
    SystemInfo,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiKey(pub u16);

impl UiKey {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn offset(self, delta: u16) -> Self {
        if self.0 > u16::MAX - delta {
            Self(u16::MAX)
        } else {
            Self(self.0 + delta)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiMark {
    pub revision: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiSpec {
    pub key: UiKey,
    pub layer: Layer,
    pub rect: Rect,
    pub clip: Option<Rect>,
    pub z: i16,
    pub alpha: u8,
    pub visual: Visual,
    pub action: Option<ButtonAction>,
}

impl UiSpec {
    pub fn visual_bounds(self) -> Rect {
        let bounds = self.visual.bounds(self.rect);
        if let Some(clip) = self.clip {
            bounds.intersect(clip).unwrap_or(Rect::new(0, 0, 0, 0))
        } else {
            bounds
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShadowStyle {
    pub color: Color,
    pub offset_x: i16,
    pub offset_y: i16,
    pub blur: u8,
    pub spread: u8,
}

impl ShadowStyle {
    pub const fn new(
        color: Color,
        offset_x: i16,
        offset_y: i16,
        blur: u8,
        spread: u8,
    ) -> Self {
        Self {
            color,
            offset_x,
            offset_y,
            blur,
            spread,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorderStyle {
    pub color: Color,
    pub width: u8,
}

impl BorderStyle {
    pub const fn new(color: Color, width: u8) -> Self {
        Self { color, width }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GradientStyle {
    pub top: Color,
    pub bottom: Color,
}

impl GradientStyle {
    pub const fn vertical(top: Color, bottom: Color) -> Self {
        Self { top, bottom }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawStyle {
    pub fill: Color,
    pub radius: u8,
    pub gradient: Option<GradientStyle>,
    pub shadow: Option<ShadowStyle>,
    pub border: Option<BorderStyle>,
}

impl DrawStyle {
    pub const fn round_rect(fill: Color, radius: u8) -> Self {
        Self {
            fill,
            radius,
            gradient: None,
            shadow: None,
            border: None,
        }
    }

    pub const fn with_vertical_gradient(mut self, top: Color, bottom: Color) -> Self {
        self.gradient = Some(GradientStyle::vertical(top, bottom));
        self
    }

    pub const fn with_shadow(mut self, shadow: ShadowStyle) -> Self {
        self.shadow = Some(shadow);
        self
    }

    pub const fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = Some(border);
        self
    }
}

#[derive(Default)]
pub struct UiFrame {
    specs: FixedList<UiSpec, UI_SPEC_CAPACITY>,
}

impl UiFrame {
    pub fn clear(&mut self) {
        self.specs.clear();
    }

    pub fn builder(&mut self) -> UiBuilder<'_> {
        UiBuilder::new(self)
    }

    pub fn visual(
        &mut self,
        key: UiKey,
        layer: Layer,
        rect: Rect,
        z: i16,
        visual: Visual,
    ) {
        self.visual_clipped(key, layer, rect, None, z, visual);
    }

    pub fn visual_clipped(
        &mut self,
        key: UiKey,
        layer: Layer,
        rect: Rect,
        clip: Option<Rect>,
        z: i16,
        visual: Visual,
    ) {
        let _ = self.specs.push(UiSpec {
            key,
            layer,
            rect,
            clip,
            z,
            alpha: 255,
            visual,
            action: None,
        });
    }

    pub fn button(
        &mut self,
        key: UiKey,
        layer: Layer,
        rect: Rect,
        z: i16,
        action: ButtonAction,
        visual: Visual,
    ) {
        self.button_clipped(key, layer, rect, None, z, action, visual);
    }

    pub fn button_clipped(
        &mut self,
        key: UiKey,
        layer: Layer,
        rect: Rect,
        clip: Option<Rect>,
        z: i16,
        action: ButtonAction,
        visual: Visual,
    ) {
        let _ = self.specs.push(UiSpec {
            key,
            layer,
            rect,
            clip,
            z,
            alpha: 255,
            visual,
            action: Some(action),
        });
    }

    pub fn text(
        &mut self,
        key: UiKey,
        layer: Layer,
        x: i32,
        y: i32,
        z: i16,
        text: &'static str,
        color: Color,
        scale: u8,
    ) {
        self.visual(
            key,
            layer,
            Rect::new(x, y, 1, 1),
            z,
            Visual::Text { text, color, scale },
        );
    }

    pub fn specs(&self) -> &[UiSpec] {
        self.specs.as_slice()
    }

    pub fn overflowed(&self) -> bool {
        self.specs.overflowed()
    }

    pub fn translate_layer(&mut self, layer: Layer, dx: i32, dy: i32) {
        if dx == 0 && dy == 0 {
            return;
        }

        for spec in self.specs.as_mut_slice() {
            if spec.layer == layer {
                spec.rect.x = spec.rect.x.saturating_add(dx);
                spec.rect.y = spec.rect.y.saturating_add(dy);
                if let Some(mut clip) = spec.clip {
                    clip.x = clip.x.saturating_add(dx);
                    clip.y = clip.y.saturating_add(dy);
                    spec.clip = Some(clip);
                }
            }
        }
    }

    pub fn fade_layer(&mut self, layer: Layer, alpha: u8) {
        if alpha == 255 {
            return;
        }

        for spec in self.specs.as_mut_slice() {
            if spec.layer == layer {
                spec.alpha = multiply_alpha(spec.alpha, alpha);
            }
        }
    }
}

pub struct UiBuilder<'a> {
    frame: &'a mut UiFrame,
}

impl<'a> UiBuilder<'a> {
    pub fn new(frame: &'a mut UiFrame) -> Self {
        Self { frame }
    }

    pub fn layer<F>(&mut self, layer: Layer, build: F)
    where
        F: FnOnce(&mut UiLayer<'_>),
    {
        let mut ui = UiLayer {
            frame: &mut *self.frame,
            layer,
            clip: None,
        };
        build(&mut ui);
    }
}

pub struct UiLayer<'a> {
    frame: &'a mut UiFrame,
    layer: Layer,
    clip: Option<Rect>,
}

impl<'a> UiLayer<'a> {
    pub fn visual(&mut self, key: UiKey, rect: Rect, z: i16, visual: Visual) {
        self.frame
            .visual_clipped(key, self.layer, rect, self.clip, z, visual);
    }

    pub fn button(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i16,
        action: ButtonAction,
        visual: Visual,
    ) {
        self.frame
            .button_clipped(key, self.layer, rect, self.clip, z, action, visual);
    }

    pub fn with_clip<F>(&mut self, rect: Rect, build: F)
    where
        F: FnOnce(&mut UiLayer<'_>),
    {
        let previous = self.clip;
        self.clip = combine_clip(previous, rect);
        build(self);
        self.clip = previous;
    }

    pub fn rect(&mut self, key: UiKey, rect: Rect, z: i16, color: Color) {
        self.visual(key, rect, z, Visual::Rect { color });
    }

    pub fn round_rect(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i16,
        radius: u8,
        color: Color,
    ) {
        self.visual(key, rect, z, Visual::RoundRect { color, radius });
    }

    pub fn round_rect_gradient(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i16,
        radius: u8,
        top: Color,
        bottom: Color,
    ) {
        self.visual(
            key,
            rect,
            z,
            Visual::RoundRectGradient {
                top,
                bottom,
                radius,
            },
        );
    }

    pub fn shadow_round_rect(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i16,
        radius: u8,
        color: Color,
        offset_x: i16,
        offset_y: i16,
        blur: u8,
        spread: u8,
    ) {
        self.visual(
            key,
            rect,
            z,
            Visual::ShadowRoundRect {
                color,
                radius,
                offset_x,
                offset_y,
                blur,
                spread,
            },
        );
    }

    pub fn styled_round_rect(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i16,
        style: DrawStyle,
    ) -> UiKey {
        let face_key = self.draw_style_decorations(key, rect, z, style);
        self.draw_style_face(face_key, rect, z, style);
        face_key.offset(1)
    }

    pub fn circle(&mut self, key: UiKey, rect: Rect, z: i16, color: Color) {
        self.visual(key, rect, z, Visual::Circle { color });
    }

    pub fn image(&mut self, key: UiKey, rect: Rect, z: i16, image: ImageId, tint: Color) {
        self.visual(key, rect, z, Visual::Image { image, tint });
    }

    pub fn icon(&mut self, key: UiKey, rect: Rect, z: i16, icon: VectorIcon, color: Color) {
        self.visual(key, rect, z, Visual::Icon { icon, color });
    }

    pub fn effect(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i16,
        effect: EffectKind,
        params: EffectParams,
    ) {
        self.visual(key, rect, z, Visual::Effect { effect, params });
    }

    pub fn text(
        &mut self,
        key: UiKey,
        x: i32,
        y: i32,
        z: i16,
        text: &'static str,
        color: Color,
        scale: u8,
    ) {
        self.visual(
            key,
            Rect::new(x, y, 1, 1),
            z,
            Visual::Text { text, color, scale },
        );
    }

    pub fn button_round_rect(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i16,
        action: ButtonAction,
        radius: u8,
        color: Color,
    ) {
        self.button(key, rect, z, action, Visual::RoundRect { color, radius });
    }

    pub fn button_styled_round_rect(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i16,
        action: ButtonAction,
        style: DrawStyle,
    ) -> UiKey {
        let face_key = self.draw_style_decorations(key, rect, z, style);
        self.draw_style_face(face_key, rect, z, style);
        let hit_key = face_key.offset(1);
        self.button(
            hit_key,
            rect,
            style_face_z(z, style),
            action,
            Visual::Rect {
                color: Color::TRANSPARENT,
            },
        );
        hit_key.offset(1)
    }

    fn draw_style_decorations(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i16,
        style: DrawStyle,
    ) -> UiKey {
        let mut next_key = key;
        if let Some(shadow) = style.shadow {
            self.shadow_round_rect(
                next_key,
                rect,
                z,
                style.radius,
                shadow.color,
                shadow.offset_x,
                shadow.offset_y,
                shadow.blur,
                shadow.spread,
            );
            next_key = next_key.offset(1);
        }

        if let Some(border) = effective_border(style) {
            self.round_rect(
                next_key,
                rect,
                style_border_z(z, style),
                style.radius,
                border.color,
            );
            next_key = next_key.offset(1);
        }

        next_key
    }

    fn draw_style_face(&mut self, key: UiKey, rect: Rect, z: i16, style: DrawStyle) {
        let rect = style_face_rect(rect, style);
        let z = style_face_z(z, style);
        let radius = style_face_radius(style);
        if let Some(gradient) = style.gradient {
            self.round_rect_gradient(key, rect, z, radius, gradient.top, gradient.bottom);
        } else {
            self.round_rect(key, rect, z, radius, style.fill);
        }
    }

    pub fn vstack(&self, rect: Rect, padding: UiPadding, gap: u16) -> UiStack {
        UiStack::new(rect, UiAxis::Vertical, padding, gap)
    }

    pub fn hstack(&self, rect: Rect, padding: UiPadding, gap: u16) -> UiStack {
        UiStack::new(rect, UiAxis::Horizontal, padding, gap)
    }

    pub fn grid(
        &self,
        rect: Rect,
        columns: u8,
        rows: u8,
        padding: UiPadding,
        gap: u16,
    ) -> UiGrid {
        UiGrid::new(rect, columns, rows, padding, gap)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiPadding {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

impl UiPadding {
    pub const ZERO: Self = Self::all(0);

    pub const fn all(value: u16) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    pub const fn vh(vertical: u16, horizontal: u16) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAxis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiStack {
    rect: Rect,
    axis: UiAxis,
    padding: UiPadding,
    gap: u16,
    cursor: u16,
    index: u16,
}

impl UiStack {
    pub const fn new(
        rect: Rect,
        axis: UiAxis,
        padding: UiPadding,
        gap: u16,
    ) -> Self {
        Self {
            rect,
            axis,
            padding,
            gap,
            cursor: 0,
            index: 0,
        }
    }

    pub fn next(&mut self, extent: u16) -> Rect {
        let gap = if self.index == 0 { 0 } else { self.gap };
        self.cursor = self.cursor.saturating_add(gap);

        let rect = match self.axis {
            UiAxis::Vertical => Rect::new(
                self.rect.x.saturating_add(self.padding.left as i32),
                self.rect
                    .y
                    .saturating_add(self.padding.top as i32)
                    .saturating_add(self.cursor as i32),
                inner_width(self.rect, self.padding),
                extent.min(self.remaining_main()),
            ),
            UiAxis::Horizontal => Rect::new(
                self.rect
                    .x
                    .saturating_add(self.padding.left as i32)
                    .saturating_add(self.cursor as i32),
                self.rect.y.saturating_add(self.padding.top as i32),
                extent.min(self.remaining_main()),
                inner_height(self.rect, self.padding),
            ),
        };

        self.cursor = self.cursor.saturating_add(extent);
        self.index = self.index.saturating_add(1);
        rect
    }

    pub fn remaining(&self) -> Rect {
        match self.axis {
            UiAxis::Vertical => Rect::new(
                self.rect.x.saturating_add(self.padding.left as i32),
                self.rect
                    .y
                    .saturating_add(self.padding.top as i32)
                    .saturating_add(self.cursor as i32),
                inner_width(self.rect, self.padding),
                self.remaining_main(),
            ),
            UiAxis::Horizontal => Rect::new(
                self.rect
                    .x
                    .saturating_add(self.padding.left as i32)
                    .saturating_add(self.cursor as i32),
                self.rect.y.saturating_add(self.padding.top as i32),
                self.remaining_main(),
                inner_height(self.rect, self.padding),
            ),
        }
    }

    fn remaining_main(self) -> u16 {
        let inner = match self.axis {
            UiAxis::Vertical => inner_height(self.rect, self.padding),
            UiAxis::Horizontal => inner_width(self.rect, self.padding),
        };
        inner.saturating_sub(self.cursor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiGrid {
    rect: Rect,
    columns: u8,
    rows: u8,
    padding: UiPadding,
    gap: u16,
}

impl UiGrid {
    pub const fn new(
        rect: Rect,
        columns: u8,
        rows: u8,
        padding: UiPadding,
        gap: u16,
    ) -> Self {
        Self {
            rect,
            columns,
            rows,
            padding,
            gap,
        }
    }

    pub fn cell(&self, index: usize) -> Rect {
        let columns = self.columns.max(1) as usize;
        let rows = self.rows.max(1) as usize;
        let column = (index % columns) as u16;
        let row = ((index / columns).min(rows.saturating_sub(1))) as u16;
        let gap_x = self.gap.saturating_mul(columns.saturating_sub(1) as u16);
        let gap_y = self.gap.saturating_mul(rows.saturating_sub(1) as u16);
        let cell_w =
            inner_width(self.rect, self.padding).saturating_sub(gap_x) / columns as u16;
        let cell_h =
            inner_height(self.rect, self.padding).saturating_sub(gap_y) / rows as u16;
        let x = self
            .rect
            .x
            .saturating_add(self.padding.left as i32)
            .saturating_add(column.saturating_mul(cell_w.saturating_add(self.gap)) as i32);
        let y = self
            .rect
            .y
            .saturating_add(self.padding.top as i32)
            .saturating_add(row.saturating_mul(cell_h.saturating_add(self.gap)) as i32);

        Rect::new(x, y, cell_w, cell_h)
    }
}

fn inner_width(rect: Rect, padding: UiPadding) -> u16 {
    rect.w
        .saturating_sub(padding.left.saturating_add(padding.right))
}

fn inner_height(rect: Rect, padding: UiPadding) -> u16 {
    rect.h
        .saturating_sub(padding.top.saturating_add(padding.bottom))
}

fn multiply_alpha(lhs: u8, rhs: u8) -> u8 {
    ((lhs as u16 * rhs as u16) / 255) as u8
}

fn combine_clip(current: Option<Rect>, next: Rect) -> Option<Rect> {
    match current {
        Some(current) => current.intersect(next).or(Some(Rect::new(0, 0, 0, 0))),
        None => Some(next),
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

fn effective_border(style: DrawStyle) -> Option<BorderStyle> {
    match style.border {
        Some(border) if border.width != 0 && border.color.a != 0 => Some(border),
        _ => None,
    }
}

fn style_border_z(z: i16, style: DrawStyle) -> i16 {
    if style.shadow.is_some() {
        z.saturating_add(1)
    } else {
        z
    }
}

fn style_face_z(z: i16, style: DrawStyle) -> i16 {
    let mut layer = z;
    if style.shadow.is_some() {
        layer = layer.saturating_add(1);
    }
    if effective_border(style).is_some() {
        layer = layer.saturating_add(1);
    }
    layer
}

fn style_face_rect(rect: Rect, style: DrawStyle) -> Rect {
    if let Some(border) = effective_border(style) {
        inset_rect(rect, border.width as u16)
    } else {
        rect
    }
}

fn style_face_radius(style: DrawStyle) -> u8 {
    if let Some(border) = effective_border(style) {
        style.radius.saturating_sub(border.width)
    } else {
        style.radius
    }
}

fn inset_rect(rect: Rect, amount: u16) -> Rect {
    let dx = amount.min(rect.w / 2);
    let dy = amount.min(rect.h / 2);
    Rect::new(
        rect.x.saturating_add(dx as i32),
        rect.y.saturating_add(dy as i32),
        rect.w.saturating_sub(dx.saturating_mul(2)),
        rect.h.saturating_sub(dy.saturating_mul(2)),
    )
}
