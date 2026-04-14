//! Math utilities for FHRE
//!
//! Provides basic vector and color types for 2D/3D rendering

mod vec2;
mod vec3;
mod color;
mod rect;

pub use vec2::Vec2;
pub use vec3::Vec3;
pub use color::Color;
pub use rect::Rect;

/// Clamp a value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
