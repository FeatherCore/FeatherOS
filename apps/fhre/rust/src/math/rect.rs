//! Rectangle

use super::Vec2;

/// Rectangle
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Get position
    pub fn pos(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Get size
    pub fn size(self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    /// Get right edge
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Get bottom edge
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Check if point is inside rectangle
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x && point.x <= self.right() &&
        point.y >= self.y && point.y <= self.bottom()
    }

    /// Intersect with another rectangle
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = self.right().min(other.right());
        let y2 = self.bottom().min(other.bottom());

        if x2 > x1 && y2 > y1 {
            Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }

    /// Check if rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right() &&
        self.right() > other.x &&
        self.y < other.bottom() &&
        self.bottom() > other.y
    }
}
