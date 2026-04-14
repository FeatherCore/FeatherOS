//! Animation Module
//!
//! A lightweight animation system inspired by Bevy's animation engine.
//! Provides animation clips, curves, blending, and transitions for embedded systems.
//!
//! ## Architecture
//!
//! ```text
//! AnimationClip (animation data)
//!     ↓
//! AnimationPlayer (component on entity)
//!     ↓
//! AnimationGraph (optional blending)
//!     ↓
//! Animated properties (Transform, Sprite, etc.)
//! ```
//!
//! ## Usage Example
//!
//! ```rust
//! // Create an animation clip
//! let mut clip = AnimationClip::new(1.0);
//! clip.add_curve_to_target(
//!     target_id,
//!     AnimationProperty::TranslationX,
//!     KeyframeCurve::new(vec![
//!         Keyframe::new(0.0, 0.0, Easing::Linear),
//!         Keyframe::new(1.0, 100.0, Easing::EaseOut),
//!     ]),
//! );
//!
//! // Play on an entity
//! world.add_component(entity, AnimationPlayer::new(clip_handle));
//! ```

pub mod clip;
pub mod curve;
pub mod easing;
pub mod player;
pub mod graph;
pub mod transition;
pub mod property;

pub use clip::{AnimationClip, AnimationClipHandle};
pub use curve::{AnimationCurve, KeyframeCurve, Keyframe, Animatable};
pub use easing::{Easing, EasingFn};
pub use player::{AnimationPlayer, ActiveAnimation, RepeatAnimation, AnimationState};
pub use graph::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex, BlendNode, ClipNode};
pub use transition::{AnimationTransitions, AnimationTransition};
pub use property::{AnimationProperty, AnimationTargetId, AnimatedField};

use crate::main_world::{Component, Entity};
use crate::resources::Time;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Animation plugin - Registers animation systems
pub struct AnimationPlugin;

impl AnimationPlugin {
    /// Create a new animation plugin
    pub fn new() -> Self {
        Self
    }
    
    /// Initialize animation resources
    pub fn init(&self) -> AnimationResources {
        AnimationResources {
            clips: BTreeMap::new(),
            graphs: BTreeMap::new(),
            next_clip_id: 1,
            next_graph_id: 1,
        }
    }
}

impl Default for AnimationPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Global animation resources
pub struct AnimationResources {
    /// Animation clip storage
    pub clips: BTreeMap<u64, AnimationClip>,
    /// Animation graph storage
    pub graphs: BTreeMap<u64, AnimationGraph>,
    /// Next clip ID
    next_clip_id: u64,
    /// Next graph ID
    next_graph_id: u64,
}

impl AnimationResources {
    /// Insert a new animation clip and return its handle
    pub fn insert_clip(&mut self, clip: AnimationClip) -> AnimationClipHandle {
        let id = self.next_clip_id;
        self.next_clip_id += 1;
        self.clips.insert(id, clip);
        AnimationClipHandle { id }
    }
    
    /// Get a clip by handle
    pub fn get_clip(&self, handle: &AnimationClipHandle) -> Option<&AnimationClip> {
        self.clips.get(&handle.id)
    }
    
    /// Get a mutable clip by handle
    pub fn get_clip_mut(&mut self, handle: &AnimationClipHandle) -> Option<&mut AnimationClip> {
        self.clips.get_mut(&handle.id)
    }
    
    /// Insert a new animation graph and return its handle
    pub fn insert_graph(&mut self, graph: AnimationGraph) -> AnimationGraphHandle {
        let id = self.next_graph_id;
        self.next_graph_id += 1;
        self.graphs.insert(id, graph);
        AnimationGraphHandle { id }
    }
    
    /// Get a graph by handle
    pub fn get_graph(&self, handle: &AnimationGraphHandle) -> Option<&AnimationGraph> {
        self.graphs.get(&handle.id)
    }
    
    /// Get a mutable graph by handle
    pub fn get_graph_mut(&mut self, handle: &AnimationGraphHandle) -> Option<&mut AnimationGraph> {
        self.graphs.get_mut(&handle.id)
    }
}

/// System that advances all active animations
pub fn animate_system(
    time: &Time,
    resources: &AnimationResources,
    players: &mut [(Entity, &mut AnimationPlayer)],
) {
    for (_entity, player) in players.iter_mut() {
        player.update(time.delta(), resources);
    }
}

/// System that handles animation transitions
pub fn transition_system(
    time: &Time,
    transitions: &mut [(Entity, &mut AnimationTransitions, &mut AnimationPlayer)],
) {
    for (_entity, transition, player) in transitions.iter_mut() {
        transition.update(time.delta(), player);
    }
}
