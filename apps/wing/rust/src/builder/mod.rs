use crate::{
    action::ActionId,
    key::UiKey,
    layout::{ContentInset, GridLayout, StackLayout},
    spec::{UiKind, UiSpec},
};
use fhre::{Camera, Color, DrawList, ImageFit, ImageId, Point, Rect, Surface, SvgId};

pub struct UiBuilder<const N: usize> {
    pub(crate) specs: [UiSpec; N],
    pub(crate) len: usize,
    overflowed: bool,
}

impl<const N: usize> UiBuilder<N> {
    pub const fn new() -> Self {
        Self {
            specs: [UiSpec::EMPTY; N],
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

    pub fn push(&mut self, spec: UiSpec) -> bool {
        if self.len >= N {
            self.overflowed = true;
            return false;
        }
        self.specs[self.len] = spec;
        self.len += 1;
        true
    }

    pub fn push_clipped(&mut self, spec: UiSpec, clip: Rect) -> bool {
        self.push(spec.with_clip(clip))
    }

    pub fn panel(&mut self, key: UiKey, rect: Rect, z: i32, color: Color, radius: u16) -> bool {
        self.push(UiSpec::rect(key, UiKind::Panel, rect, z, color, radius))
    }

    pub fn panel_with_content(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i32,
        color: Color,
        radius: u16,
        inset: ContentInset,
    ) -> bool {
        self.push(UiSpec::rect(key, UiKind::Panel, rect, z, color, radius).with_content_inset(inset))
    }

    pub fn tile(&mut self, key: UiKey, rect: Rect, z: i32, color: Color, radius: u16) -> bool {
        self.push(UiSpec::rect(key, UiKind::Tile, rect, z, color, radius))
    }

    pub fn tile_action(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i32,
        color: Color,
        radius: u16,
        action: ActionId,
    ) -> bool {
        self.push(UiSpec::rect(key, UiKind::Tile, rect, z, color, radius).with_action(action))
    }

    pub fn bar(&mut self, key: UiKey, rect: Rect, z: i32, color: Color, radius: u16) -> bool {
        self.push(UiSpec::rect(key, UiKind::Bar, rect, z, color, radius))
    }

    pub fn circle(&mut self, key: UiKey, rect: Rect, z: i32, color: Color) -> bool {
        self.push(UiSpec::rect(key, UiKind::Circle, rect, z, color, rect.w.min(rect.h) / 2))
    }

    pub fn line(&mut self, key: UiKey, from: Point, to: Point, z: i32, width: u16, color: Color) -> bool {
        self.push(UiSpec::line(key, from, to, z, width, color))
    }

    pub fn text(
        &mut self,
        key: UiKey,
        x: i32,
        y: i32,
        z: i32,
        text: &'static str,
        color: Color,
        scale: u16,
    ) -> bool {
        self.push(UiSpec::text(key, x, y, z, text, color, scale))
    }

    pub fn icon(&mut self, key: UiKey, rect: Rect, z: i32, icon: SvgId, color: Color) -> bool {
        self.push(UiSpec::icon(key, rect, z, icon, color))
    }

    pub fn image(&mut self, key: UiKey, rect: Rect, z: i32, image: ImageId, opacity: u8) -> bool {
        self.push(UiSpec::image(key, rect, z, image, opacity))
    }

    pub fn image_fit(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
    ) -> bool {
        self.push(UiSpec::image_fit(key, rect, z, image, opacity, fit))
    }

    pub fn image_tint(
        &mut self,
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
        tint: Color,
    ) -> bool {
        self.push(UiSpec::image_tint(key, rect, z, image, opacity, fit, tint))
    }

    pub fn child_image(
        &mut self,
        parent: UiKey,
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
    ) -> bool {
        self.push(UiSpec::image(key, rect, z, image, opacity).with_local_parent(parent))
    }

    pub fn child_image_fit(
        &mut self,
        parent: UiKey,
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
    ) -> bool {
        self.push(UiSpec::image_fit(key, rect, z, image, opacity, fit).with_local_parent(parent))
    }

    pub fn child_image_tint(
        &mut self,
        parent: UiKey,
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
        tint: Color,
    ) -> bool {
        self.push(UiSpec::image_tint(key, rect, z, image, opacity, ImageFit::Stretch, tint).with_local_parent(parent))
    }

    pub fn child_image_tint_fit(
        &mut self,
        parent: UiKey,
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
        tint: Color,
    ) -> bool {
        self.push(UiSpec::image_tint(key, rect, z, image, opacity, fit, tint).with_local_parent(parent))
    }

    pub fn content_image(
        &mut self,
        parent: UiKey,
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
    ) -> bool {
        self.push(UiSpec::image(key, rect, z, image, opacity).with_content_parent(parent))
    }

    pub fn content_image_fit(
        &mut self,
        parent: UiKey,
        key: UiKey,
        rect: Rect,
        z: i32,
        image: ImageId,
        opacity: u8,
        fit: ImageFit,
    ) -> bool {
        self.push(UiSpec::image_fit(key, rect, z, image, opacity, fit).with_content_parent(parent))
    }

    pub fn push_content_scrolled(
        &mut self,
        mut spec: UiSpec,
        parent: UiKey,
        scroll_y: i32,
        clip: Rect,
    ) -> bool {
        spec.transform.position.y = spec.transform.position.y.saturating_sub(fhre::fixed_from_i32(scroll_y));
        self.push(spec.with_content_parent(parent).with_clip(clip))
    }

    pub fn child_text(
        &mut self,
        parent: UiKey,
        key: UiKey,
        x: i32,
        y: i32,
        z: i32,
        text: &'static str,
        color: Color,
        scale: u16,
    ) -> bool {
        self.push(UiSpec::text(key, x, y, z, text, color, scale).with_local_parent(parent))
    }

    pub fn content_text(
        &mut self,
        parent: UiKey,
        key: UiKey,
        x: i32,
        y: i32,
        z: i32,
        text: &'static str,
        color: Color,
        scale: u16,
    ) -> bool {
        self.push(UiSpec::text(key, x, y, z, text, color, scale).with_content_parent(parent))
    }

    pub fn content_bar(
        &mut self,
        parent: UiKey,
        key: UiKey,
        rect: Rect,
        z: i32,
        color: Color,
        radius: u16,
    ) -> bool {
        self.push(UiSpec::rect(key, UiKind::Bar, rect, z, color, radius).with_content_parent(parent))
    }

    pub fn child_icon(
        &mut self,
        parent: UiKey,
        key: UiKey,
        rect: Rect,
        z: i32,
        icon: SvgId,
        color: Color,
    ) -> bool {
        self.push(UiSpec::icon(key, rect, z, icon, color).with_local_parent(parent))
    }

    pub fn grid_tile_action(
        &mut self,
        parent: UiKey,
        key: UiKey,
        grid: GridLayout,
        index: u8,
        z: i32,
        color: Color,
        radius: u16,
        action: ActionId,
    ) -> bool {
        self.push(
            UiSpec::rect(key, UiKind::Tile, Rect::new(0, 0, grid.cell.w, grid.cell.h), z, color, radius)
                .with_local_parent(parent)
                .with_grid_cell(grid, index)
                .with_action(action),
        )
    }

    pub fn content_grid_tile_action(
        &mut self,
        parent: UiKey,
        key: UiKey,
        grid: GridLayout,
        index: u8,
        z: i32,
        color: Color,
        radius: u16,
        action: ActionId,
    ) -> bool {
        self.push(
            UiSpec::rect(key, UiKind::Tile, Rect::new(0, 0, grid.cell.w, grid.cell.h), z, color, radius)
                .with_content_parent(parent)
                .with_grid_cell(grid, index)
                .with_action(action),
        )
    }

    pub fn stack_tile(
        &mut self,
        parent: UiKey,
        key: UiKey,
        stack: StackLayout,
        index: u8,
        z: i32,
        color: Color,
        radius: u16,
    ) -> bool {
        self.push(
            UiSpec::rect(key, UiKind::Tile, Rect::new(0, 0, stack.item.w, stack.item.h), z, color, radius)
                .with_local_parent(parent)
                .with_stack_item(stack, index),
        )
    }

    pub fn render(&self, surface: &mut Surface, camera: &Camera) {
        let mut list: DrawList<N> = DrawList::new();
        let mut index = 0;
        while index < self.len {
            let spec = self.specs[index];
            if let Some(command) = spec.command(camera) {
                list.push_clipped(command, spec.projected_clip(camera));
            }
            index += 1;
        }
        list.sort_by_depth();
        list.execute(surface);
    }
}
