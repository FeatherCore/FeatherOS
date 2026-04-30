#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    pub w: u16,
    pub h: u16,
}

impl Size {
    pub const fn new(w: u16, h: u16) -> Self {
        Self { w, h }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    pub fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.w as i32
            && point.y < self.y + self.h as i32
    }

    pub fn intersect(self, other: Self) -> Option<Self> {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = self
            .x
            .saturating_add(self.w as i32)
            .min(other.x.saturating_add(other.w as i32));
        let y1 = self
            .y
            .saturating_add(self.h as i32)
            .min(other.y.saturating_add(other.h as i32));

        if x0 >= x1 || y0 >= y1 {
            None
        } else {
            Some(Self::new(
                x0,
                y0,
                (x1 - x0).clamp(0, u16::MAX as i32) as u16,
                (y1 - y0).clamp(0, u16::MAX as i32) as u16,
            ))
        }
    }

    pub fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }

        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = (self.x + self.w as i32).max(other.x + other.w as i32);
        let y1 = (self.y + self.h as i32).max(other.y + other.h as i32);

        Self::new(
            x0,
            y0,
            (x1 - x0).clamp(0, u16::MAX as i32) as u16,
            (y1 - y0).clamp(0, u16::MAX as i32) as u16,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn argb8888(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | self.b as u32
    }

    pub fn with_alpha(self, alpha: u8) -> Self {
        Self { a: alpha, ..self }
    }

    pub fn lerp(self, other: Self, t: u8) -> Self {
        let inv = 255u16.saturating_sub(t as u16);
        let t = t as u16;
        Self {
            r: ((self.r as u16 * inv + other.r as u16 * t) / 255) as u8,
            g: ((self.g as u16 * inv + other.g as u16 * t) / 255) as u8,
            b: ((self.b as u16 * inv + other.b as u16 * t) / 255) as u8,
            a: ((self.a as u16 * inv + other.a as u16 * t) / 255) as u8,
        }
    }
}
