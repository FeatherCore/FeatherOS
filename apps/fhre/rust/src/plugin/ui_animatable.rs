//! UI Animatable Plugin
//!
//! Registers Phase 3 animation apply systems for FHRE's built-in UI components.
//!
//! This plugin is included in `DefaultPlugins`. It is separated from `AnimationPlugin`
//! because:
//! - **AnimationPlugin** = pure animation infrastructure (time + curves + sampling), zero business dependency
//! - **UiAnimatablePlugin** = binds animation output to concrete UI component types (Cube, SoccerBall)
//!
//! For custom animatable components, users register their own:
//! ```ignore
//! app.add_systems(Update, system2::<Query<AnimationPlayer>, Query<MyComponent>, _>(
//!     fhre::animation::apply_animations::<MyComponent>
//! ));
//! ```

use crate::plugin::Plugin;
use crate::app::{App, Update};
use crate::main_world::{system2, Query};
use crate::animation::{AnimationPlayer, apply_animations};
use crate::ui::{Cube, SoccerBall};

/// Plugin that registers animation apply systems for built-in UI components.
///
/// Included in `DefaultPlugins`. Registers `apply_animations::<Cube>` and
/// `apply_animations::<SoccerBall>` so that sampled animation values are
/// automatically written to these components each frame.
///
/// # Architecture note
///
/// This follows Bevy's pattern where `AnimationPlugin` handles sampling
/// (type-erased via `AnimationCurveEvaluator`) and property application
/// is handled by evaluator implementations. In FHRE's simpler model:
/// - `AnimationPlugin` → advance time + sample curves into `sampled_properties`
/// - `UiAnimatablePlugin` → read `sampled_properties` → write to Cube/SoccerBall via `AnimationReceiver`
pub struct UiAnimatablePlugin;

impl Plugin for UiAnimatablePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            system2::<Query<AnimationPlayer>, Query<Cube>, _>(apply_animations::<Cube>),
        )
        .add_systems(
            Update,
            system2::<Query<AnimationPlayer>, Query<SoccerBall>, _>(apply_animations::<SoccerBall>),
        );
    }
}

impl Default for UiAnimatablePlugin {
    fn default() -> Self {
        Self
    }
}
