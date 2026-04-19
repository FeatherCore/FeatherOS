//! Animation Curves
//!
//! Defines animation curves and interpolation for animating properties over time.
//! Inspired by Bevy's curve system but simplified for embedded systems.

use crate::math::{Vec2, Vec3, Color};
use crate::animation::easing::Easing;
use core::f32;

/// A single keyframe with time, value, and easing
#[derive(Clone, Copy, Debug)]
pub struct Keyframe {
    /// Time in seconds
    pub time: f32,
    /// Value at this keyframe
    pub value: f32,
    /// Easing function to next keyframe
    pub easing: Easing,
}

impl Keyframe {
    /// Create a new keyframe
    pub fn new(time: f32, value: f32, easing: Easing) -> Self {
        Self { time, value, easing }
    }
    
    /// Create a linear keyframe
    pub fn linear(time: f32, value: f32) -> Self {
        Self::new(time, value, Easing::Linear)
    }
    
    /// Create an ease-in keyframe
    pub fn ease_in(time: f32, value: f32) -> Self {
        Self::new(time, value, Easing::EaseIn)
    }
    
    /// Create an ease-out keyframe
    pub fn ease_out(time: f32, value: f32) -> Self {
        Self::new(time, value, Easing::EaseOut)
    }
    
    /// Create an ease-in-out keyframe
    pub fn ease_in_out(time: f32, value: f32) -> Self {
        Self::new(time, value, Easing::EaseInOut)
    }
}

/// Trait for types that can be animated
pub trait Animatable: Clone + Copy + 'static {
    /// Interpolate between two values
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self;
    
    /// Blend multiple values with weights
    fn blend(inputs: &[(Self, f32)]) -> Self;
}

impl Animatable for f32 {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        a + (b - a) * t
    }
    
    fn blend(inputs: &[(Self, f32)]) -> Self {
        let mut sum = 0.0;
        let mut weight_sum = 0.0;
        for (value, weight) in inputs {
            sum += value * weight;
            weight_sum += weight;
        }
        if weight_sum > 0.0 {
            sum / weight_sum
        } else {
            0.0
        }
    }
}

impl Animatable for Vec2 {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Vec2::new(
            f32::interpolate(&a.x, &b.x, t),
            f32::interpolate(&a.y, &b.y, t),
        )
    }
    
    fn blend(inputs: &[(Self, f32)]) -> Self {
        let mut sum = Vec2::new(0.0, 0.0);
        let mut weight_sum = 0.0;
        for (value, weight) in inputs {
            sum.x += value.x * weight;
            sum.y += value.y * weight;
            weight_sum += weight;
        }
        if weight_sum > 0.0 {
            Vec2::new(sum.x / weight_sum, sum.y / weight_sum)
        } else {
            Vec2::new(0.0, 0.0)
        }
    }
}

impl Animatable for Vec3 {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Vec3::new(
            f32::interpolate(&a.x, &b.x, t),
            f32::interpolate(&a.y, &b.y, t),
            f32::interpolate(&a.z, &b.z, t),
        )
    }
    
    fn blend(inputs: &[(Self, f32)]) -> Self {
        let mut sum = Vec3::new(0.0, 0.0, 0.0);
        let mut weight_sum = 0.0;
        for (value, weight) in inputs {
            sum.x += value.x * weight;
            sum.y += value.y * weight;
            sum.z += value.z * weight;
            weight_sum += weight;
        }
        if weight_sum > 0.0 {
            Vec3::new(sum.x / weight_sum, sum.y / weight_sum, sum.z / weight_sum)
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        }
    }
}

impl Animatable for Color {
    fn interpolate(a: &Self, b: &Self, t: f32) -> Self {
        Color::new(
            f32::interpolate(&(a.r as f32), &(b.r as f32), t) as u8,
            f32::interpolate(&(a.g as f32), &(b.g as f32), t) as u8,
            f32::interpolate(&(a.b as f32), &(b.b as f32), t) as u8,
            f32::interpolate(&(a.a as f32), &(b.a as f32), t) as u8,
        )
    }
    
    fn blend(inputs: &[(Self, f32)]) -> Self {
        let mut r = 0.0f32;
        let mut g = 0.0f32;
        let mut b = 0.0f32;
        let mut a = 0.0f32;
        let mut weight_sum = 0.0f32;
        
        for (color, weight) in inputs {
            r += (color.r as f32) * weight;
            g += (color.g as f32) * weight;
            b += (color.b as f32) * weight;
            a += (color.a as f32) * weight;
            weight_sum += weight;
        }
        
        if weight_sum > 0.0 {
            Color::new(
                (r / weight_sum) as u8,
                (g / weight_sum) as u8,
                (b / weight_sum) as u8,
                (a / weight_sum) as u8,
            )
        } else {
            Color::WHITE
        }
    }
}

/// Animation curve trait - Defines how to sample values over time
pub trait AnimationCurve: Clone + 'static {
    /// The value type this curve produces
    type Value: Animatable;
    
    /// Sample the curve at a given time
    fn sample(&self, time: f32) -> Option<Self::Value>;
    
    /// Get the duration of the curve
    fn duration(&self) -> f32;
    
    /// Check if the curve is empty
    fn is_empty(&self) -> bool;
}

/// A curve defined by keyframes
#[derive(Clone, Debug)]
pub struct KeyframeCurve {
    /// Keyframes sorted by time
    keyframes: alloc::vec::Vec<Keyframe>,
    /// Cached duration
    duration: f32,
}

impl KeyframeCurve {
    /// Create a new keyframe curve
    pub fn new(keyframes: alloc::vec::Vec<Keyframe>) -> Self {
        let duration = keyframes.last().map(|k| k.time).unwrap_or(0.0);
        Self { keyframes, duration }
    }
    
    /// Create a curve from keyframe values with linear easing
    pub fn from_values(values: &[(f32, f32)]) -> Self {
        let keyframes: alloc::vec::Vec<Keyframe> = values
            .iter()
            .map(|(time, value)| Keyframe::linear(*time, *value))
            .collect();
        Self::new(keyframes)
    }
    
    /// Add a keyframe
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframes.push(keyframe);
        // Keep sorted by time
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        self.duration = self.keyframes.last().map(|k| k.time).unwrap_or(0.0);
    }
    
    /// Get the keyframes
    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }
    
    /// Find the keyframe index at or before the given time
    fn find_keyframe_index(&self, time: f32) -> Option<usize> {
        if self.keyframes.is_empty() {
            return None;
        }
        
        // Binary search for the keyframe at or before time
        let mut low = 0;
        let mut high = self.keyframes.len() - 1;
        
        while low < high {
            let mid = (low + high + 1) / 2;
            if self.keyframes[mid].time <= time {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        
        Some(low)
    }
}

impl AnimationCurve for KeyframeCurve {
    type Value = f32;
    
    fn sample(&self, time: f32) -> Option<f32> {
        if self.keyframes.is_empty() {
            return None;
        }
        
        // Clamp time to curve bounds
        let time = time.max(0.0).min(self.duration);
        
        // Find the keyframe at or before this time
        let index = self.find_keyframe_index(time)?;
        let current = &self.keyframes[index];
        
        // If we're at the last keyframe, return its value
        if index >= self.keyframes.len() - 1 {
            return Some(current.value);
        }
        
        // Interpolate to next keyframe
        let next = &self.keyframes[index + 1];
        let duration = next.time - current.time;
        
        if duration <= 0.0 {
            return Some(current.value);
        }
        
        let t = (time - current.time) / duration;
        let eased_t = current.easing.apply(t);
        
        Some(f32::interpolate(&current.value, &next.value, eased_t))
    }
    
    fn duration(&self) -> f32 {
        self.duration
    }
    
    fn is_empty(&self) -> bool {
        self.keyframes.is_empty()
    }
}

/// A constant value curve (holds a single value)
#[derive(Clone, Copy, Debug)]
pub struct ConstantCurve<T: Animatable> {
    value: T,
    duration: f32,
}

impl<T: Animatable> ConstantCurve<T> {
    /// Create a new constant curve
    pub fn new(value: T, duration: f32) -> Self {
        Self { value, duration }
    }
}

impl<T: Animatable> AnimationCurve for ConstantCurve<T> {
    type Value = T;
    
    fn sample(&self, _time: f32) -> Option<T> {
        Some(self.value)
    }
    
    fn duration(&self) -> f32 {
        self.duration
    }
    
    fn is_empty(&self) -> bool {
        false
    }
}

/// A curve that follows a function
#[derive(Clone)]
pub struct FunctionCurve<F: Fn(f32) -> f32 + Clone + 'static> {
    function: F,
    duration: f32,
}

impl<F: Fn(f32) -> f32 + Clone + 'static> FunctionCurve<F> {
    /// Create a new function curve
    pub fn new(function: F, duration: f32) -> Self {
        Self { function, duration }
    }
}

impl<F: Fn(f32) -> f32 + Clone + 'static> AnimationCurve for FunctionCurve<F> {
    type Value = f32;
    
    fn sample(&self, time: f32) -> Option<f32> {
        Some((self.function)(time))
    }
    
    fn duration(&self) -> f32 {
        self.duration
    }
    
    fn is_empty(&self) -> bool {
        false
    }
}

/// Blend multiple curves together
pub fn blend_curves<T: Animatable>(
    curves: &[(impl AnimationCurve<Value = T>, f32)],
    time: f32,
) -> Option<T> {
    let mut values = alloc::vec::Vec::new();
    
    for (curve, weight) in curves {
        if let Some(value) = curve.sample(time) {
            values.push((value, *weight));
        }
    }
    
    if values.is_empty() {
        None
    } else {
        Some(T::blend(&values))
    }
}
