//! DefaultPlugins for FHRE
//!
//! A collection of commonly used plugins that provide a good starting point for most applications.
//!
//! # Included Plugins
//!
//! - **EventPlugin**: Event system with standard input events
//! - **AnimationPlugin**: Animation clip and player system (time + sampling)
//! - **CameraPlugin**: Single 3D perspective camera (projects onto 2D screen canvas)
//! - **UiAnimatablePlugin**: Binds animation output to built-in UI components (Cube, SoccerBall)
//!
//! # Usage
//!
//! ```rust
//! App::new(640, 480)
//!     .add_plugins(DefaultPlugins)
//!     .run();
//! ```

use crate::plugin::{Plugin, PluginGroup};
use crate::plugin::plugin_group::PluginGroupBuilder;
use crate::event::EventPlugin;
use crate::animation::AnimationPlugin;
use crate::camera::CameraPlugin;
use super::ui_animatable::UiAnimatablePlugin;

/// Default plugins for FHRE applications
///
/// This plugin group includes:
/// - `EventPlugin` - Event system (Events resource, EventReader/EventWriter)
/// - `AnimationPlugin` - Animation system (AnimationResources, AnimationPlayer, time + sampling)
/// - `CameraPlugin` - Single 3D perspective camera (projects 3D scene onto 2D screen canvas)
/// - `UiAnimatablePlugin` - Applies sampled animations to built-in UI components (Cube, SoccerBall)
///
/// Note: Window is NOT included by default because it requires platform-specific
/// implementation. The application should provide its own window implementation
/// using the `fhre::window::Window` trait.
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        unsafe {
            extern "C" { fn printf(format: *const u8, ...) -> i32; }
            printf(b"[DEFAULT_PLUGINS] Building default plugins...\n\0".as_ptr());
        }

        PluginGroupBuilder::start::<Self>()
            .add(EventPlugin)
            .add(AnimationPlugin)
            .add(CameraPlugin)
            .add(UiAnimatablePlugin)
    }
}

impl Default for DefaultPlugins {
    fn default() -> Self {
        Self
    }
}
