//! Gradient System
//!
//! Provides gradient fill support for rendering.
//! Supports linear, radial, and sweep gradients.

use crate::math::{Color, Vec2};
use alloc::vec::Vec;
use libm::{floorf, sqrtf, atan2f, cosf, sinf};

fn hypotf(x: f32, y: f32) -> f32 {
    sqrtf(x * x + y * y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GradientDirection {
    #[default]
    Horizontal,
    Vertical,
    Linear,
    Radial,
    Sweep,
}

#[derive(Debug, Clone, Copy)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color,
}

impl GradientStop {
    pub const fn new(position: f32, color: Color) -> Self {
        Self { position, color }
    }
}

#[derive(Debug, Clone)]
pub struct Gradient {
    pub stops: Vec<GradientStop>,
    pub direction: GradientDirection,
    pub start: Vec2,
    pub end: Vec2,
    pub angle: f32,
}

impl Gradient {
    pub fn new(stops: Vec<GradientStop>, direction: GradientDirection) -> Self {
        Self {
            stops,
            direction,
            start: Vec2::ZERO,
            end: Vec2::new(1.0, 1.0),
            angle: 0.0,
        }
    }

    pub fn horizontal(colors: &[Color]) -> Self {
        let stops = Self::create_stops(colors);
        Self::new(stops, GradientDirection::Horizontal)
    }

    pub fn vertical(colors: &[Color]) -> Self {
        let stops = Self::create_stops(colors);
        Self::new(stops, GradientDirection::Vertical)
    }

    pub fn linear(colors: &[Color], angle: f32) -> Self {
        let stops = Self::create_stops(colors);
        let mut grad = Self::new(stops, GradientDirection::Linear);
        grad.angle = angle;
        grad
    }

    pub fn radial(colors: &[Color], center: Vec2, radius: f32) -> Self {
        let stops = Self::create_stops(colors);
        let mut grad = Self::new(stops, GradientDirection::Radial);
        grad.start = center;
        grad.end = Vec2::new(center.x + radius, center.y);
        grad
    }

    pub fn sweep(colors: &[Color], center: Vec2, start_angle: f32, end_angle: f32) -> Self {
        let stops = Self::create_stops(colors);
        let mut grad = Self::new(stops, GradientDirection::Sweep);
        grad.start = center;
        grad.angle = start_angle;
        grad.end = Vec2::new(end_angle, 0.0);
        grad
    }

    fn create_stops(colors: &[Color]) -> Vec<GradientStop> {
        if colors.is_empty() {
            return alloc::vec![GradientStop::new(0.0, Color::BLACK)];
        }

        if colors.len() == 1 {
            return alloc::vec![
                GradientStop::new(0.0, colors[0]),
                GradientStop::new(1.0, colors[0])
            ];
        }

        colors
            .iter()
            .enumerate()
            .map(|(i, &color)| {
                let position = i as f32 / (colors.len() - 1) as f32;
                GradientStop::new(position, color)
            })
            .collect()
    }

    pub fn with_bounds(mut self, start: Vec2, end: Vec2) -> Self {
        self.start = start;
        self.end = end;
        self
    }

    pub fn sample(&self, x: f32, y: f32) -> Color {
        let t = self.calculate_t(x, y);
        self.sample_at(t)
    }

    fn calculate_t(&self, x: f32, y: f32) -> f32 {
        match self.direction {
            GradientDirection::Horizontal => {
                let width = self.end.x - self.start.x;
                if width.abs() < 0.0001 {
                    return 0.0;
                }
                ((x - self.start.x) / width).clamp(0.0, 1.0)
            }
            GradientDirection::Vertical => {
                let height = self.end.y - self.start.y;
                if height.abs() < 0.0001 {
                    return 0.0;
                }
                ((y - self.start.y) / height).clamp(0.0, 1.0)
            }
            GradientDirection::Linear => {
                let dx = x - self.start.x;
                let dy = y - self.start.y;
                let angle_rad = self.angle.to_radians();
                let cos_a = cosf(angle_rad);
                let sin_a = sinf(angle_rad);
                let proj = dx * cos_a + dy * sin_a;
                let length = hypotf(self.end.x - self.start.x, self.end.y - self.start.y);
                if length.abs() < 0.0001 {
                    return 0.0;
                }
                (proj / length).clamp(0.0, 1.0)
            }
            GradientDirection::Radial => {
                let dx = x - self.start.x;
                let dy = y - self.start.y;
                let radius = hypotf(self.end.x - self.start.x, self.end.y - self.start.y);
                if radius.abs() < 0.0001 {
                    return 0.0;
                }
                let dist = sqrtf(dx * dx + dy * dy);
                (dist / radius).clamp(0.0, 1.0)
            }
            GradientDirection::Sweep => {
                let dx = x - self.start.x;
                let dy = y - self.start.y;
                let angle = atan2f(dy, dx);
                let angle_deg = angle.to_degrees();
                let normalized = if angle_deg < 0.0 {
                    angle_deg + 360.0
                } else {
                    angle_deg
                };
                let start_angle = self.angle;
                let end_angle = self.end.x;
                let range = end_angle - start_angle;
                if range.abs() < 0.0001 {
                    return 0.0;
                }
                let t = ((normalized - start_angle) / range).clamp(0.0, 1.0);
                t
            }
        }
    }

    fn sample_at(&self, t: f32) -> Color {
        if self.stops.is_empty() {
            return Color::BLACK;
        }

        if self.stops.len() == 1 {
            return self.stops[0].color;
        }

        for i in 0..self.stops.len() - 1 {
            let stop0 = &self.stops[i];
            let stop1 = &self.stops[i + 1];

            if t >= stop0.position && t <= stop1.position {
                let range = stop1.position - stop0.position;
                if range.abs() < 0.0001 {
                    return stop0.color;
                }
                let local_t = (t - stop0.position) / range;
                return Color::lerp(stop0.color, stop1.color, local_t);
            }
        }

        if t <= self.stops[0].position {
            return self.stops[0].color;
        }

        self.stops[self.stops.len() - 1].color
    }

    pub fn precompute(&self, width: u32) -> PrecomputedGradient {
        let mut colors = Vec::with_capacity(width as usize);
        for x in 0..width {
            let t = x as f32 / (width - 1).max(1) as f32;
            colors.push(self.sample_at(t));
        }
        PrecomputedGradient { colors }
    }

    pub fn sample_rect(&self, rect: &crate::math::Rect) -> [Color; 4] {
        let self_with_bounds = self.clone().with_bounds(
            Vec2::new(rect.x, rect.y),
            Vec2::new(rect.x + rect.width, rect.y + rect.height)
        );
        [
            self_with_bounds.sample(rect.x, rect.y),
            self_with_bounds.sample(rect.x + rect.width, rect.y),
            self_with_bounds.sample(rect.x + rect.width, rect.y + rect.height),
            self_with_bounds.sample(rect.x, rect.y + rect.height),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct PrecomputedGradient {
    pub colors: Vec<Color>,
}

impl PrecomputedGradient {
    pub fn sample(&self, t: f32) -> Color {
        if self.colors.is_empty() {
            return Color::BLACK;
        }

        let index = (t * (self.colors.len() - 1) as f32).clamp(0.0, self.colors.len() as f32 - 0.001) as usize;
        self.colors[index.min(self.colors.len() - 1)]
    }

    pub fn sample_linear(&self, t: f32) -> Color {
        if self.colors.is_empty() {
            return Color::BLACK;
        }

        if self.colors.len() == 1 {
            return self.colors[0];
        }

        let fidx = t * (self.colors.len() - 1) as f32;
        let idx0 = (floorf(fidx) as usize).min(self.colors.len() - 2);
        let idx1 = idx0 + 1;
        let frac = fidx - idx0 as f32;

        Color::lerp(self.colors[idx0], self.colors[idx1], frac)
    }
}
