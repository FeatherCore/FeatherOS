//! Animation Player
//!
//! Component that plays animation clips on entities.

use super::clip::{AnimationClip, AnimationClipHandle};
use super::AnimationResources;
use crate::main_world::Component;
use crate::math::{Vec2, Vec3};

/// Repeat behavior for animations
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RepeatAnimation {
    /// Play once and stop
    Never,
    /// Repeat forever
    Forever,
    /// Repeat N times
    Count(u32),
}

impl Default for RepeatAnimation {
    fn default() -> Self {
        RepeatAnimation::Never
    }
}

/// State of an active animation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationState {
    /// Animation is playing
    Playing,
    /// Animation is paused
    Paused,
    /// Animation has finished
    Finished,
}

impl Default for AnimationState {
    fn default() -> Self {
        AnimationState::Playing
    }
}

/// An active animation instance
#[derive(Clone, Debug)]
pub struct ActiveAnimation {
    /// Handle to the clip
    pub clip_handle: AnimationClipHandle,
    /// Current playback time
    pub time: f32,
    /// Playback speed (1.0 = normal)
    pub speed: f32,
    /// Weight for blending (0.0 - 1.0)
    pub weight: f32,
    /// Repeat mode
    pub repeat: RepeatAnimation,
    /// Current state
    pub state: AnimationState,
    /// Number of times completed
    pub completions: u32,
}

impl ActiveAnimation {
    /// Create a new active animation
    pub fn new(clip_handle: AnimationClipHandle) -> Self {
        Self {
            clip_handle,
            time: 0.0,
            speed: 1.0,
            weight: 1.0,
            repeat: RepeatAnimation::Never,
            state: AnimationState::Playing,
            completions: 0,
        }
    }
    
    /// Set playback speed
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
    
    /// Set weight
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }
    
    /// Set repeat mode
    pub fn with_repeat(mut self, repeat: RepeatAnimation) -> Self {
        self.repeat = repeat;
        self
    }
    
    /// Check if animation is finished
    pub fn is_finished(&self) -> bool {
        matches!(self.state, AnimationState::Finished)
    }
    
    /// Check if animation is playing
    pub fn is_playing(&self) -> bool {
        matches!(self.state, AnimationState::Playing)
    }
    
    /// Update the animation
    pub fn update(&mut self, delta_time: f32, clip_duration: f32) {
        if !self.is_playing() {
            return;
        }
        
        self.time += delta_time * self.speed;
        
        // Check if animation reached end
        if self.time >= clip_duration {
            self.completions += 1;
            
            match self.repeat {
                RepeatAnimation::Never => {
                    self.time = clip_duration;
                    self.state = AnimationState::Finished;
                }
                RepeatAnimation::Forever => {
                    self.time = 0.0;
                }
                RepeatAnimation::Count(n) => {
                    if self.completions >= n {
                        self.time = clip_duration;
                        self.state = AnimationState::Finished;
                    } else {
                        self.time = 0.0;
                    }
                }
            }
        }
    }
    
    /// Pause the animation
    pub fn pause(&mut self) {
        self.state = AnimationState::Paused;
    }
    
    /// Resume the animation
    pub fn resume(&mut self) {
        if matches!(self.state, AnimationState::Paused) {
            self.state = AnimationState::Playing;
        }
    }
    
    /// Stop and reset the animation
    pub fn stop(&mut self) {
        self.time = 0.0;
        self.state = AnimationState::Finished;
    }
    
    /// Seek to a specific time
    pub fn seek_to(&mut self, time: f32) {
        self.time = time.max(0.0);
    }
}

impl Default for ActiveAnimation {
    fn default() -> Self {
        Self::new(AnimationClipHandle::null())
    }
}

/// Animation player component - attached to entities that play animations
#[derive(Clone, Debug, Default)]
pub struct AnimationPlayer {
    /// Currently playing animations
    animations: alloc::vec::Vec<ActiveAnimation>,
    /// Whether the player is paused
    paused: bool,
}

impl AnimationPlayer {
    /// Create a new animation player
    pub fn new() -> Self {
        Self {
            animations: alloc::vec::Vec::new(),
            paused: false,
        }
    }
    
    /// Play an animation clip
    pub fn play(&mut self, clip_handle: AnimationClipHandle) -> &mut ActiveAnimation {
        let animation = ActiveAnimation::new(clip_handle);
        self.animations.push(animation);
        self.animations.last_mut().unwrap()
    }
    
    /// Play with specific settings
    pub fn play_with<F>(&mut self, clip_handle: AnimationClipHandle, f: F) -> &mut ActiveAnimation
    where
        F: FnOnce(ActiveAnimation) -> ActiveAnimation,
    {
        let animation = f(ActiveAnimation::new(clip_handle));
        self.animations.push(animation);
        self.animations.last_mut().unwrap()
    }
    
    /// Stop all animations
    pub fn stop_all(&mut self) {
        self.animations.clear();
    }
    
    /// Stop a specific animation by index
    pub fn stop(&mut self, index: usize) {
        if index < self.animations.len() {
            self.animations.remove(index);
        }
    }
    
    /// Pause all animations
    pub fn pause_all(&mut self) {
        self.paused = true;
        for anim in &mut self.animations {
            anim.pause();
        }
    }
    
    /// Resume all animations
    pub fn resume_all(&mut self) {
        self.paused = false;
        for anim in &mut self.animations {
            anim.resume();
        }
    }
    
    /// Get an animation by index
    pub fn animation(&self, index: usize) -> Option<&ActiveAnimation> {
        self.animations.get(index)
    }
    
    /// Get a mutable animation by index
    pub fn animation_mut(&mut self, index: usize) -> Option<&mut ActiveAnimation> {
        self.animations.get_mut(index)
    }
    
    /// Get all animations
    pub fn animations(&self) -> &[ActiveAnimation] {
        &self.animations
    }
    
    /// Get mutable animations
    pub fn animations_mut(&mut self) -> &mut [ActiveAnimation] {
        &mut self.animations
    }
    
    /// Update all animations
    pub fn update(&mut self, delta_time: f32, resources: &AnimationResources) {
        if self.paused {
            return;
        }
        
        // Remove finished animations that don't repeat
        self.animations.retain(|anim| {
            if anim.is_finished() {
                matches!(anim.repeat, RepeatAnimation::Forever | RepeatAnimation::Count(_))
            } else {
                true
            }
        });
        
        // Update remaining animations
        for anim in &mut self.animations {
            if let Some(clip) = resources.get_clip(&anim.clip_handle) {
                anim.update(delta_time, clip.duration());
            }
        }
    }
    
    /// Check if any animation is playing
    pub fn is_playing(&self) -> bool {
        self.animations.iter().any(|a| a.is_playing())
    }
    
    /// Get the number of active animations
    pub fn len(&self) -> usize {
        self.animations.len()
    }
    
    /// Check if player has no animations
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
}

impl Component for AnimationPlayer {
    fn type_name() -> &'static str {
        "AnimationPlayer"
    }
}
