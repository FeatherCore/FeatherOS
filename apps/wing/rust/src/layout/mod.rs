use fhre::{Point, Rect, Size};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutSpace {
    Screen,
    Parent,
    Content,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContentInset {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

impl ContentInset {
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    pub const fn new(left: u16, top: u16, right: u16, bottom: u16) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn uniform(value: u16) -> Self {
        Self::new(value, value, value, value)
    }

    pub fn content_rect(self, rect: Rect) -> Rect {
        Rect::new(
            rect.x.saturating_add(self.left as i32),
            rect.y.saturating_add(self.top as i32),
            rect.w.saturating_sub(self.left.saturating_add(self.right)),
            rect.h.saturating_sub(self.top.saturating_add(self.bottom)),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GridLayout {
    pub origin: Point,
    pub cell: Size,
    pub gap: Size,
    pub columns: u8,
}

impl GridLayout {
    pub const fn new(origin: Point, cell: Size, gap: Size, columns: u8) -> Self {
        Self {
            origin,
            cell,
            gap,
            columns,
        }
    }

    pub fn cell_rect(self, index: u8) -> Rect {
        let columns = self.columns.max(1);
        let col = (index % columns) as i32;
        let row = (index / columns) as i32;
        Rect::new(
            self.origin.x + col * (self.cell.w as i32 + self.gap.w as i32),
            self.origin.y + row * (self.cell.h as i32 + self.gap.h as i32),
            self.cell.w,
            self.cell.h,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StackLayout {
    pub origin: Point,
    pub item: Size,
    pub step: Point,
}

impl StackLayout {
    pub const fn new(origin: Point, item: Size, step: Point) -> Self {
        Self { origin, item, step }
    }

    pub const fn column(origin: Point, item: Size, gap: u16) -> Self {
        Self {
            origin,
            item,
            step: Point::new(0, item.h as i32 + gap as i32),
        }
    }

    pub const fn row(origin: Point, item: Size, gap: u16) -> Self {
        Self {
            origin,
            item,
            step: Point::new(item.w as i32 + gap as i32, 0),
        }
    }

    pub fn item_rect(self, index: u8) -> Rect {
        Rect::new(
            self.origin.x + self.step.x * index as i32,
            self.origin.y + self.step.y * index as i32,
            self.item.w,
            self.item.h,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutRule {
    None,
    GridCell { grid: GridLayout, index: u8 },
    StackItem { stack: StackLayout, index: u8 },
}
