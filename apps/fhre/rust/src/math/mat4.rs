//! 4x4 Matrix for 3D transformations
//!
//! Provides basic matrix operations for 3D graphics.

use super::Vec3;

/// 4x4 Matrix (column-major order for OpenGL compatibility)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    /// Matrix data in column-major order
    pub data: [f32; 16],
}

impl Mat4 {
    /// Identity matrix
    pub const IDENTITY: Self = Self {
        data: [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
    };

    /// Create a new matrix from raw data
    pub fn new(data: [f32; 16]) -> Self {
        Self { data }
    }

    /// Create an identity matrix
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    /// Get element at row, column
    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.data[col * 4 + row]
    }

    /// Set element at row, column
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        self.data[col * 4 + row] = value;
    }

    /// Matrix multiplication
    pub fn mul(&self, other: &Self) -> Self {
        let mut result = Self::identity();
        
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.get(i, k) * other.get(k, j);
                }
                result.set(i, j, sum);
            }
        }
        
        result
    }

    /// Multiply by a Vec3 (treating it as Vec4 with w=1)
    pub fn mul_vec3(&self, v: Vec3) -> Vec3 {
        let x = self.get(0, 0) * v.x + self.get(0, 1) * v.y + self.get(0, 2) * v.z + self.get(0, 3);
        let y = self.get(1, 0) * v.x + self.get(1, 1) * v.y + self.get(1, 2) * v.z + self.get(1, 3);
        let z = self.get(2, 0) * v.x + self.get(2, 1) * v.y + self.get(2, 2) * v.z + self.get(2, 3);
        let w = self.get(3, 0) * v.x + self.get(3, 1) * v.y + self.get(3, 2) * v.z + self.get(3, 3);
        
        if w != 0.0 && w != 1.0 {
            Vec3::new(x / w, y / w, z / w)
        } else {
            Vec3::new(x, y, z)
        }
    }

    /// Create a translation matrix
    pub fn from_translation(translation: Vec3) -> Self {
        let mut m = Self::identity();
        m.set(0, 3, translation.x);
        m.set(1, 3, translation.y);
        m.set(2, 3, translation.z);
        m
    }

    /// Create a scale matrix
    pub fn from_scale(scale: Vec3) -> Self {
        let mut m = Self::identity();
        m.set(0, 0, scale.x);
        m.set(1, 1, scale.y);
        m.set(2, 2, scale.z);
        m
    }

    /// Create a rotation matrix around X axis (angle in radians)
    pub fn from_rotation_x(angle: f32) -> Self {
        let cos = libm::cosf(angle);
        let sin = libm::sinf(angle);
        
        let mut m = Self::identity();
        m.set(1, 1, cos);
        m.set(1, 2, -sin);
        m.set(2, 1, sin);
        m.set(2, 2, cos);
        m
    }

    /// Create a rotation matrix around Y axis (angle in radians)
    pub fn from_rotation_y(angle: f32) -> Self {
        let cos = libm::cosf(angle);
        let sin = libm::sinf(angle);
        
        let mut m = Self::identity();
        m.set(0, 0, cos);
        m.set(0, 2, sin);
        m.set(2, 0, -sin);
        m.set(2, 2, cos);
        m
    }

    /// Create a rotation matrix around Z axis (angle in radians)
    pub fn from_rotation_z(angle: f32) -> Self {
        let cos = libm::cosf(angle);
        let sin = libm::sinf(angle);
        
        let mut m = Self::identity();
        m.set(0, 0, cos);
        m.set(0, 1, -sin);
        m.set(1, 0, sin);
        m.set(1, 1, cos);
        m
    }

    /// Create a perspective projection matrix (right-handed)
    pub fn perspective_rh(fov_y_radians: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / libm::tanf(fov_y_radians / 2.0);
        let nf = 1.0 / (near - far);
        
        let mut m = Self::identity();
        m.set(0, 0, f / aspect_ratio);
        m.set(1, 1, f);
        m.set(2, 2, (far + near) * nf);
        m.set(2, 3, 2.0 * far * near * nf);
        m.set(3, 2, -1.0);
        m.set(3, 3, 0.0);
        
        m
    }

    /// Create an orthographic projection matrix (right-handed)
    pub fn orthographic_rh(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let mut m = Self::identity();
        m.set(0, 0, 2.0 / (right - left));
        m.set(1, 1, 2.0 / (top - bottom));
        m.set(2, 2, -2.0 / (far - near));
        m.set(0, 3, -(right + left) / (right - left));
        m.set(1, 3, -(top + bottom) / (top - bottom));
        m.set(2, 3, -(far + near) / (far - near));
        m
    }

    /// Create a look-at view matrix (right-handed)
    pub fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);
        
        let mut m = Self::identity();
        m.set(0, 0, s.x);
        m.set(0, 1, s.y);
        m.set(0, 2, s.z);
        m.set(1, 0, u.x);
        m.set(1, 1, u.y);
        m.set(1, 2, u.z);
        m.set(2, 0, -f.x);
        m.set(2, 1, -f.y);
        m.set(2, 2, -f.z);
        m.set(0, 3, -s.dot(eye));
        m.set(1, 3, -u.dot(eye));
        m.set(2, 3, f.dot(eye));
        
        m
    }

    /// Calculate matrix inverse (for view-projection matrices)
    pub fn inverse(&self) -> Option<Self> {
        // For now, return identity as placeholder
        // Full implementation would use Gaussian elimination
        Some(Self::identity())
    }

    /// Transpose the matrix
    pub fn transpose(&self) -> Self {
        let mut result = Self::identity();
        for i in 0..4 {
            for j in 0..4 {
                result.set(j, i, self.get(i, j));
            }
        }
        result
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

// Operator overloads
impl core::ops::Mul for Mat4 {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self {
        let mut result = Self::identity();
        
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.get(i, k) * rhs.get(k, j);
                }
                result.set(i, j, sum);
            }
        }
        
        result
    }
}

impl core::ops::Mul<Vec3> for Mat4 {
    type Output = Vec3;
    
    fn mul(self, rhs: Vec3) -> Vec3 {
        self.mul_vec3(rhs)
    }
}

impl core::ops::Mul<super::Vec4> for Mat4 {
    type Output = super::Vec4;
    
    fn mul(self, rhs: super::Vec4) -> super::Vec4 {
        let x = self.get(0, 0) * rhs.x + self.get(0, 1) * rhs.y + self.get(0, 2) * rhs.z + self.get(0, 3) * rhs.w;
        let y = self.get(1, 0) * rhs.x + self.get(1, 1) * rhs.y + self.get(1, 2) * rhs.z + self.get(1, 3) * rhs.w;
        let z = self.get(2, 0) * rhs.x + self.get(2, 1) * rhs.y + self.get(2, 2) * rhs.z + self.get(2, 3) * rhs.w;
        let w = self.get(3, 0) * rhs.x + self.get(3, 1) * rhs.y + self.get(3, 2) * rhs.z + self.get(3, 3) * rhs.w;
        super::Vec4::new(x, y, z, w)
    }
}
