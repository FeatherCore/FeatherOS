//! DefaultPlugins for FHRE
//!
//! A collection of commonly used plugins that provide a good starting point for most applications.
//!
//! # Included Plugins
//!
//! - **EventPlugin**: Event system with standard input events
//! - **AnimationPlugin**: Animation clip and player system
//!
//! # Usage
//!
//! ```rust
//! App::new(640, 480)
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(WindowPlugin::new(640, 480, "My App"))
//!     .run();
//! ```

use crate::plugin::{Plugin, PluginGroup};
use crate::plugin::plugin_group::PluginGroupBuilder;
use crate::event::EventPlugin;
use crate::animation::AnimationPlugin;

/// Default plugins for FHRE applications
///
/// This plugin group includes:
/// - `EventPlugin` - Event system (Events resource, EventReader/EventWriter)
/// - `AnimationPlugin` - Animation system (AnimationResources, AnimationPlayer)
///
/// Note: `WindowPlugin` is NOT included by default because it requires platform-specific
/// configuration (window size, title). Add it separately.
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        unsafe {
            extern "C" { fn printf(format: *const u8, ...) -> i32; }
            printf(b"[DEFAULT_PLUGINS] Building default plugins...\n\0".as_ptr());
        }

        PluginGroupBuilder::start::<Self>()
            .add(EventPlugin)   // Event system
            .add(AnimationPlugin) // Animation system
    }
}

impl Default for DefaultPlugins {
    fn default() -> Self {
        Self
    }
}
