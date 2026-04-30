//! DefaultPlugins for FHRE
//!
//! A collection of commonly used plugins that provide a good starting point for most applications.
//!
//! # Included Plugins
//!
//! - **EventPlugin**: Event system with standard input events
//! - **AnimationPlugin**: Animation clip and player system (time + sampling)
//! - **CameraPlugin**: Single 3D perspective camera (projects onto 2D screen canvas)
//!
//! # Usage
//!
//! ```rust
//! App::new(640, 480)
//!     .add_plugins(DefaultPlugins)
//!     .run();
//! ```

use crate::plugin::PluginGroup;
use crate::plugin::plugin_group::PluginGroupBuilder;
use crate::event::EventPlugin;
use crate::animation::AnimationPlugin;
use crate::camera::CameraPlugin;
use crate::picking::PickingPlugin;

/// Default plugins for FHRE applications
///
/// This plugin group includes:
/// - `EventPlugin` - Event system (Events resource, EventReader/EventWriter)
/// - `AnimationPlugin` - Animation system (AnimationResources, AnimationPlayer, time + sampling)
/// - `CameraPlugin` - Single 3D perspective camera (projects 3D scene onto 2D screen canvas)
///
/// Note: Window is NOT included by default because it requires platform-specific
/// implementation. The application should provide its own window implementation
/// using the `fhre::window::Window` trait.
///
/// For custom animatable components, users should register their own apply systems:
/// ```ignore
/// app.add_systems(Update, system2::<Query<&AnimationPlayer>, Query<&mut MyComponent>, _>(
///     fhre::animation::apply_animations::<MyComponent>
/// ));
/// ```
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(EventPlugin)
            .add(AnimationPlugin)
            .add(CameraPlugin)
            .add(PickingPlugin)
    }
}

impl Default for DefaultPlugins {
    fn default() -> Self {
        Self
    }
}
