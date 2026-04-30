//! 3D Vector

use core::ops::{Add, Sub, Mul};

/// 3D Vector
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// Create a new 3D vector
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Create a vector with all components set to the same value
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    /// Zero vector
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    
    /// One vector
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };

    /// Unit X vector
    pub const X: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    
    /// Unit Y vector
    pub const Y: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    
    /// Unit Z vector
    pub const Z: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    
    /// Negative Z vector (forward in right-handed system)
    pub const NEG_Z: Self = Self { x: 0.0, y: 0.0, z: -1.0 };

    /// Zero vector (deprecated, use ZERO)
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }

    /// One vector (deprecated, use ONE)
    pub const fn one() -> Self {
        Self { x: 1.0, y: 1.0, z: 1.0 }
    }

    /// Add two vectors
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    /// Subtract two vectors
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    /// Multiply by scalar
    pub fn mul_scalar(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    /// Get the length of the vector
    pub fn length(self) -> f32 {
        libm::sqrtf(self.x * self.x + self.y * self.y + self.z * self.z)
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
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Linear interpolation between two vectors
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self::new(
            self.x + (other.x - self.x) * t,
            self.y + (other.y - self.y) * t,
            self.z + (other.z - self.z) * t,
        )
    }

    /// Extend to Vec4 with w component
    pub fn extend(self, w: f32) -> super::Vec4 {
        super::Vec4::new(self.x, self.y, self.z, w)
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.add(other)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.sub(other)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        self.mul_scalar(scalar)
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;
    fn mul(self, vec: Vec3) -> Vec3 {
        vec.mul_scalar(self)
    }
}

// Vec4 type for homogeneous coordinates
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    /// Create a new 4D vector
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Truncate to Vec3 (drop w component)
    pub fn truncate(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}
