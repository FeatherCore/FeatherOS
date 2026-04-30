#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub const EMPTY: Self = Self::new(0, 0, 0, 0);

    pub const fn new(x: i32, y: i32, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(self) -> i32 {
        self.x.saturating_add(self.w as i32)
    }

    pub fn bottom(self) -> i32 {
        self.y.saturating_add(self.h as i32)
    }

    pub const fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub fn intersects(self, other: Self) -> bool {
        !self.is_empty()
            && !other.is_empty()
            && self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    pub fn contains_point(self, point: Point) -> bool {
        point.x >= self.x && point.x < self.right() && point.y >= self.y && point.y < self.bottom()
    }

    pub fn clipped_to(self, bounds: Self) -> Self {
        let x0 = self.x.max(bounds.x);
        let y0 = self.y.max(bounds.y);
        let x1 = self.right().min(bounds.right());
        let y1 = self.bottom().min(bounds.bottom());
        Self::from_edges(x0, y0, x1, y1)
    }

    pub fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }

        Self::from_edges(
            self.x.min(other.x),
            self.y.min(other.y),
            self.right().max(other.right()),
            self.bottom().max(other.bottom()),
        )
    }

    pub fn from_edges(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        if x1 <= x0 || y1 <= y0 {
            return Self::EMPTY;
        }

        Self {
            x: x0,
            y: y0,
            w: (x1 - x0).min(u16::MAX as i32) as u16,
            h: (y1 - y0).min(u16::MAX as i32) as u16,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size {
    pub w: u16,
    pub h: u16,
}

impl Size {
    pub const fn new(w: u16, h: u16) -> Self {
        Self { w, h }
    }
}
