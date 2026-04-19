//! Input Plugin for FHRE
//!
//! Automatically registers input systems and resources.

use crate::app::App;
use crate::plugin::Plugin;
use crate::input::{
    ButtonInput, MouseButton,
    MouseInput, AccumulatedMouseMotion, AccumulatedMouseScroll,
    KeyCode, Key,
};
use crate::event::Events;

/// Plugin that adds input handling to the app.
///
/// This plugin registers:
/// - Input event types (Events)
/// - ButtonInput resources for mouse and keyboard
/// - Mouse motion and scroll resources
/// - Input processing systems (automatically called in App::update)
///
/// # Example
///
/// ```
/// use fhre::{App, InputPlugin};
///
/// let mut app = App::new();
/// app.add_plugin(InputPlugin);
/// ```
#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // Register event queue
        app.init_resource::<Events>();

        // Register mouse button state
        app.init_resource::<ButtonInput<MouseButton>>();

        // Register keyboard state
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<Key>>();

        // Register mouse position and motion
        app.init_resource::<MouseInput>();
        app.init_resource::<AccumulatedMouseMotion>();
        app.init_resource::<AccumulatedMouseScroll>();

        // Note: Input systems are automatically called in App::update()
        // through input_systems_update()
    }

    fn name(&self) -> &str {
        "fhre::input::InputPlugin"
    }
}

/// Extension trait for App to add input-related methods
pub trait InputAppExt {
    /// Initialize a resource with its Default value
    fn init_resource<R: crate::resources::Resource + Default>(&mut self) -> &mut Self;
}

impl InputAppExt for App {
    fn init_resource<R: crate::resources::Resource + Default>(&mut self) -> &mut Self {
        if !self.main_world.resources().contains::<R>() {
            self.main_world.resources_mut().insert(R::default());
        }
        self
    }
}
