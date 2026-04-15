//! Animation Transitions
//!
//! Smooth transitions between animations with fade in/out effects.

use super::player::{ActiveAnimation, AnimationPlayer, RepeatAnimation};
use super::graph::AnimationNodeIndex;
use crate::main_world::Component;

/// An animation transition (fade out)
#[derive(Clone, Debug)]
pub struct AnimationTransition {
    /// Current weight (starts at 1.0, goes to 0.0)
    pub current_weight: f32,
    /// How much to decrease weight per second
    pub weight_decline_per_sec: f32,
    /// The animation being faded out
    pub animation_index: usize,
}

impl AnimationTransition {
    /// Create a new transition
    pub fn new(animation_index: usize, transition_duration: f32, start_weight: f32) -> Self {
        Self {
            current_weight: start_weight,
            weight_decline_per_sec: if transition_duration > 0.0 {
                1.0 / transition_duration
            } else {
                f32::MAX
            },
            animation_index,
        }
    }
    
    /// Update the transition
    pub fn update(&mut self, delta_time: f32) -> bool {
        self.current_weight -= self.weight_decline_per_sec * delta_time;
        self.current_weight > 0.0
    }
}

/// Component for managing animation transitions
#[derive(Clone, Debug, Default)]
pub struct AnimationTransitions {
    /// Current main animation index
    main_animation: Option<usize>,
    /// Active transitions (fading out)
    transitions: alloc::vec::Vec<AnimationTransition>,
}

impl AnimationTransitions {
    /// Create a new transitions manager
    pub fn new() -> Self {
        Self {
            main_animation: None,
            transitions: alloc::vec::Vec::new(),
        }
    }
    
    /// Play a new animation with transition
    pub fn play(
        &mut self,
        player: &mut AnimationPlayer,
        animation_index: usize,
        transition_duration: f32,
    ) {
        // If there's a current main animation, start fading it out
        if let Some(old_index) = self.main_animation {
            if let Some(old_anim) = player.animation(old_index) {
                let start_weight = old_anim.weight;
                self.transitions.push(AnimationTransition::new(
                    old_index,
                    transition_duration,
                    start_weight,
                ));
            }
        }
        
        // Cancel any existing transition to this animation
        self.transitions.retain(|t| t.animation_index != animation_index);
        
        // Set new main animation
        self.main_animation = Some(animation_index);
        
        // Ensure the new animation has full weight initially
        if let Some(anim) = player.animation_mut(animation_index) {
            anim.weight = 1.0;
        }
    }
    
    /// Get the main animation index
    pub fn main_animation(&self) -> Option<usize> {
        self.main_animation
    }
    
    /// Update all transitions
    pub fn update(&mut self, delta_time: f32, player: &mut AnimationPlayer) {
        let mut remaining_weight = 1.0;
        
        // Process transitions from newest to oldest (greedy layer system)
        for transition in self.transitions.iter_mut().rev() {
            let still_active = transition.update(delta_time);
            
            if let Some(anim) = player.animation_mut(transition.animation_index) {
                if still_active {
                    anim.weight = transition.current_weight * remaining_weight;
                    remaining_weight -= anim.weight;
                } else {
                    anim.weight = 0.0;
                }
            }
        }
        
        // Remove completed transitions
        self.transitions.retain(|t| t.current_weight > 0.0);
        
        // Main animation gets remaining weight
        if let Some(main_index) = self.main_animation {
            if let Some(anim) = player.animation_mut(main_index) {
                anim.weight = remaining_weight.max(0.0);
            }
        }
    }
    
    /// Check if currently transitioning
    pub fn is_transitioning(&self) -> bool {
        !self.transitions.is_empty()
    }
    
    /// Get number of active transitions
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
    
    /// Stop all transitions
    pub fn stop_all(&mut self) {
        self.transitions.clear();
        self.main_animation = None;
    }
}

impl Component for AnimationTransitions {
    fn type_name() -> &'static str {
        "AnimationTransitions"
    }
}
