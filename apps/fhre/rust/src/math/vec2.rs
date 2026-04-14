//! 2D Vector

use core::ops::{Add, Sub, Mul};

/// 2D Vector
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// Create a new 2D vector
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero vector
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    
    /// One vector
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    /// Zero vector (deprecated, use ZERO)
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// One vector (deprecated, use ONE)
    pub const fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    /// Add two vectors
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    /// Subtract two vectors
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    /// Multiply by scalar
    pub fn mul_scalar(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.add(other)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.sub(other)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        self.mul_scalar(scalar)
    }
}
