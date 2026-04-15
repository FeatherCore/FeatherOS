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

    /// Create a vector with all components set to the same value
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
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

    /// Linear interpolation between two vectors
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
        )
    }

    /// Get the length of the vector
    pub fn length(self) -> f32 {
        libm::sqrtf(self.x * self.x + self.y * self.y)
    }

    /// Normalize the vector
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 {
            self.mul_scalar(1.0 / len)
        } else {
            Self::ZERO
        }
    }

    /// Dot product
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Get the minimum of each component
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    /// Get the maximum of each component
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
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

impl Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, vec: Vec2) -> Vec2 {
        vec.mul_scalar(self)
    }
}
