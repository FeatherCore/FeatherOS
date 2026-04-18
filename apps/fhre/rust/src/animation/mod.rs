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
pub use property::{AnimationProperty, AnimationTargetId, AnimatedField, AnimationReceiver};

use crate::main_world::{Component, Entity, Res, ResMut, Query, system2, system3};
use crate::resources::Time;
use crate::resources::Resource;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Animation plugin - Registers animation systems as a Plugin
pub struct AnimationPlugin;

impl AnimationPlugin {
    /// Create a new animation plugin
    pub fn new() -> Self {
        Self
    }
    
    /// Initialize animation resources
    pub fn init() -> AnimationResources {
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

// SAFETY: AnimationPlugin is stateless and safe to share
unsafe impl Send for AnimationPlugin {}
unsafe impl Sync for AnimationPlugin {}

use crate::plugin::Plugin;
use crate::app::{App, Update};

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Self::init())
            .add_systems(Update, system3::<Res<Time>, Res<AnimationResources>, Query<AnimationPlayer>, _>(advance_animations))
            .add_systems(Update, system2::<Res<AnimationResources>, Query<AnimationPlayer>, _>(animate_targets));
    }
}

/// System: Advance all active animations (update playback time)
///
/// Equivalent to Bevy's `advance_animations`. Updates AnimationPlayer time
/// for all entities that have an AnimationPlayer component. Handles looping,
/// pausing, and speed.
///
/// Registered automatically by `AnimationPlugin`. Users should NOT need to
/// call this manually.
pub fn advance_animations(time: Res<Time>, anim_resources: Res<AnimationResources>, mut players: Query<AnimationPlayer>) {
    let delta = time.delta();
    for mut player in players.iter_mut() {
        player.update_time(delta, &anim_resources);
    }
}

/// System: Sample animation curves and store values in ActiveAnimation.sampled_properties
///
/// Equivalent to Bevy's `animate_targets` (sampling phase). For each entity with
/// an active AnimationPlayer:
/// 1. Gets the playing animation clip from AnimationResources
/// 2. Samples each curve at the current playback time
/// 3. Stores sampled values in `ActiveAnimation.sampled_properties`
///
/// Component-specific apply happens separately via `apply_animation_to<T>` or
/// by the component implementing `AnimationTarget`.
///
/// Registered automatically by `AnimationPlugin`.
pub fn animate_targets(
    anim_resources: Res<AnimationResources>,
    mut players: Query<AnimationPlayer>,
) {
    for mut player in players.iter_mut() {
        if let Some(anim) = player.animation_mut(0) {
            if anim.clip_handle.is_null() { continue; }

            anim.sampled_properties.clear();

            if let Some(clip) = anim_resources.get_clip(&anim.clip_handle) {
                if let Some(curves) = clip.curves_for_target(anim.target_id) {
                    for property_curve in curves.iter() {
                        if let Some(value) = property_curve.curve.sample(anim.time) {
                            anim.sampled_properties.insert(property_curve.property, value);
                        }
                    }
                }
            }
        }
    }
}

/// Apply sampled animation values from AnimationPlayer to a component implementing AnimationReceiver
///
/// Internal helper used by `apply_animations<T>`.
fn apply_sampled_to<T: AnimationReceiver>(player: &AnimationPlayer, target: &mut T) {
    if let Some(anim) = player.animation(0) {
        for (&property, &value) in anim.sampled_properties.iter() {
            target.apply_animation(property, value);
        }
    }
}

/// Generic Phase 3 system: Apply sampled animation values to any `AnimationReceiver` component (Entity-matched)
///
/// This is a **template function** — instantiate it for each animatable component type:
///
/// ```ignore
/// app.add_systems(Update, system2::<Query<AnimationPlayer>, Query<Cube>, _>(apply_animations::<Cube>));
/// app.add_systems(Update, system2::<Query<AnimationPlayer>, Query<SoccerBall>, _>(apply_animations::<SoccerBall>));
/// ```
///
/// # Design (aligned with Bevy's `animate_targets`)
///
/// Bevy merges sampling + applying into a single `animate_targets` system that uses
/// `AnimationCurveEvaluator` trait objects for type-erased property application.
/// FHRE achieves the same goal without reflection by providing this **generic template**:
/// - `AnimationPlugin` registers only Phase 1 (time) + Phase 2 (sampling) — zero business dependency
/// - Phase 3 is registered per-component-type via `apply_animations::<T>`
/// - Entity matching uses `iter_mut_with_entities()` + `get_mut(entity)` — correct association
///
/// # Why not hardcode in AnimationPlugin?
///
/// Hardcoding `Cube`/`SoccerBall` in AnimationPlugin creates a "god plugin" that must be
/// modified for every new animatable component. This pattern separates concerns:
/// - **AnimationPlugin** = pure animation infrastructure (time + curves + sampling)
/// - **Component owner** (UI plugin, game plugin) = registers which types receive animation
pub fn apply_animations<T: AnimationReceiver + Component>(
    mut players: Query<AnimationPlayer>,
    mut targets: Query<T>,
) {
    for (entity, player) in players.iter_with_entities() {
        if let Some(anim) = player.animation(0) {
            if !anim.sampled_properties.is_empty() {
                if let Some(mut target) = targets.get_mut(entity) {
                    apply_sampled_to(&player, &mut *target);
                }
            }
        }
    }
}

// SAFETY: AnimationResources is only accessed on main thread in SIM platform
unsafe impl Send for AnimationResources {}
unsafe impl Sync for AnimationResources {}

impl Resource for AnimationResources {}

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

/// System that advances all active animations (ECS style - for use with MainWorld systems)
pub fn animate_system(
    time: &Time,
    mut resources: core::cell::RefMut<AnimationResources>,
    players: &mut [(Entity, &mut AnimationPlayer)],
) {
    for (_entity, player) in players.iter_mut() {
        player.update(time.delta(), &*resources);
    }
}

/// System that handles animation transitions (ECS style)
pub fn transition_system(
    time: &Time,
    transitions: &mut [(Entity, &mut AnimationTransitions, &mut AnimationPlayer)],
) {
    for (_entity, transition, player) in transitions.iter_mut() {
        transition.update(time.delta(), player);
    }
}
