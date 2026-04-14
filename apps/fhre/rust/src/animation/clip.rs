//! Animation Clip
//!
//! Animation clips contain the actual animation data - curves that define how properties change over time.

use super::curve::{AnimationCurve, KeyframeCurve};
use super::property::{AnimationProperty, AnimationTargetId};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Handle to an animation clip
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationClipHandle {
    pub(crate) id: u64,
}

impl AnimationClipHandle {
    /// Create a null handle
    pub fn null() -> Self {
        Self { id: 0 }
    }
    
    /// Check if handle is null
    pub fn is_null(&self) -> bool {
        self.id == 0
    }
}

impl Default for AnimationClipHandle {
    fn default() -> Self {
        Self::null()
    }
}

/// A curve with its target property
#[derive(Clone, Debug)]
pub struct PropertyCurve {
    /// Target property
    pub property: AnimationProperty,
    /// The animation curve
    pub curve: KeyframeCurve,
}

/// Animation clip - contains animation curves for one or more targets
#[derive(Clone, Debug, Default)]
pub struct AnimationClip {
    /// Duration in seconds
    duration: f32,
    /// Curves indexed by target ID
    curves: BTreeMap<AnimationTargetId, Vec<PropertyCurve>>,
    /// Whether the clip is valid
    valid: bool,
}

impl AnimationClip {
    /// Create a new empty animation clip
    pub fn new() -> Self {
        Self {
            duration: 0.0,
            curves: BTreeMap::new(),
            valid: true,
        }
    }
    
    /// Create a clip with a specific duration
    pub fn with_duration(duration: f32) -> Self {
        Self {
            duration,
            curves: BTreeMap::new(),
            valid: true,
        }
    }
    
    /// Get the clip duration
    pub fn duration(&self) -> f32 {
        self.duration
    }
    
    /// Set the clip duration
    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
    }
    
    /// Add a curve for a specific target and property
    pub fn add_curve_to_target(
        &mut self,
        target_id: AnimationTargetId,
        property: AnimationProperty,
        curve: KeyframeCurve,
    ) {
        // Update duration if curve extends beyond current duration
        let curve_duration = curve.duration();
        if curve_duration > self.duration {
            self.duration = curve_duration;
        }
        
        let property_curve = PropertyCurve { property, curve };
        
        self.curves
            .entry(target_id)
            .or_insert_with(Vec::new)
            .push(property_curve);
    }
    
    /// Get curves for a specific target
    pub fn curves_for_target(&self, target_id: AnimationTargetId) -> Option<&Vec<PropertyCurve>> {
        self.curves.get(&target_id)
    }
    
    /// Get all curves
    pub fn curves(&self) -> &BTreeMap<AnimationTargetId, Vec<PropertyCurve>> {
        &self.curves
    }
    
    /// Check if clip has any curves
    pub fn is_empty(&self) -> bool {
        self.curves.is_empty()
    }
    
    /// Mark clip as invalid
    pub fn invalidate(&mut self) {
        self.valid = false;
    }
    
    /// Check if clip is valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }
    
    /// Sample the clip at a given time for a specific target and property
    pub fn sample(
        &self,
        target_id: AnimationTargetId,
        property: AnimationProperty,
        time: f32,
    ) -> Option<f32> {
        let curves = self.curves.get(&target_id)?;
        
        for property_curve in curves {
            if property_curve.property == property {
                return property_curve.curve.sample(time);
            }
        }
        
        None
    }
    
    /// Sample all properties for a target at a given time
    pub fn sample_target(
        &self,
        target_id: AnimationTargetId,
        time: f32,
    ) -> BTreeMap<AnimationProperty, f32> {
        let mut result = BTreeMap::new();
        
        if let Some(curves) = self.curves.get(&target_id) {
            for property_curve in curves {
                if let Some(value) = property_curve.curve.sample(time) {
                    result.insert(property_curve.property, value);
                }
            }
        }
        
        result
    }
    
    /// Get all target IDs in this clip
    pub fn target_ids(&self) -> Vec<AnimationTargetId> {
        self.curves.keys().copied().collect()
    }
    
    /// Merge another clip into this one
    pub fn merge(&mut self, other: &AnimationClip) {
        for (target_id, curves) in &other.curves {
            for property_curve in curves {
                self.add_curve_to_target(
                    *target_id,
                    property_curve.property,
                    property_curve.curve.clone(),
                );
            }
        }
        
        if other.duration > self.duration {
            self.duration = other.duration;
        }
    }
}

/// Builder for creating animation clips
pub struct AnimationClipBuilder {
    clip: AnimationClip,
}

impl AnimationClipBuilder {
    /// Create a new clip builder
    pub fn new() -> Self {
        Self {
            clip: AnimationClip::new(),
        }
    }
    
    /// Add a curve
    pub fn add_curve(
        mut self,
        target_id: AnimationTargetId,
        property: AnimationProperty,
        curve: KeyframeCurve,
    ) -> Self {
        self.clip.add_curve_to_target(target_id, property, curve);
        self
    }
    
    /// Set duration
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.clip.set_duration(duration);
        self
    }
    
    /// Build the clip
    pub fn build(self) -> AnimationClip {
        self.clip
    }
}

impl Default for AnimationClipBuilder {
    fn default() -> Self {
        Self::new()
    }
}
