//! Winit-style Plugin for FHRE
//!
//! This plugin provides window creation and event loop management,
//! similar to Bevy's WinitPlugin.
//!
//! When added, this plugin sets the app's runner to handle the window event loop.

use crate::app::{App, AppRunner, AppExit};
use crate::plugin::Plugin;
use alloc::boxed::Box;

/// Plugin that provides window and event loop management
///
/// This plugin sets up a window and controls the application's main loop.
/// It's similar to Bevy's WinitPlugin.
///
/// # Example
///
/// ```rust
/// fn main() {
///     App::new(640, 480)
///         .add_plugin(WinitPlugin::new("My App"))
///         .add_systems(Startup, setup)
///         .add_systems(Update, update)
///         .run();
/// }
/// ```
pub struct WinitPlugin {
    title: &'static str,
    width: u32,
    height: u32,
}

impl WinitPlugin {
    /// Create a new WinitPlugin with the specified window title
    pub fn new(title: &'static str) -> Self {
        Self {
            title,
            width: 640,
            height: 480,
        }
    }

    /// Set the window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

impl Plugin for WinitPlugin {
    fn build(&self, app: &mut App) {
        // Set the runner that will handle the window event loop
        let runner = WinitRunner::new(self.title, self.width, self.height);
        app.set_runner(runner);

        // Initialize window-related resources
        // (Window creation happens in the runner)
    }

    fn name(&self) -> &str {
        "fhre::WinitPlugin"
    }
}

/// Runner that manages the window event loop
struct WinitRunner {
    title: &'static str,
    width: u32,
    height: u32,
}

impl WinitRunner {
    fn new(title: &'static str, width: u32, height: u32) -> Self {
        Self { title, width, height }
    }
}

impl AppRunner for WinitRunner {
    fn run(self: Box<Self>, mut app: App) -> AppExit {
        // Simple runner - just run the app once
        // In a full implementation, this would:
        // 1. Create the window
        // 2. Set up the event loop
        // 3. Handle window events
        // 4. Call app.update() each frame
        
        loop {
            app.update();
            // For now, just run once and exit
            break;
        }
        
        AppExit::Success
    }
}

/// Plugin group that provides default plugins
///
/// This is similar to Bevy's DefaultPlugins.
pub struct DefaultPlugins {
    window_title: &'static str,
    width: u32,
    height: u32,
}

impl DefaultPlugins {
    /// Create default plugins with the specified window configuration
    pub fn new(title: &'static str, width: u32, height: u32) -> Self {
        Self {
            window_title: title,
            width,
            height,
        }
    }
}

impl crate::plugin::PluginGroup for DefaultPlugins {
    fn build(self) -> crate::plugin::PluginGroupBuilder {
        let mut builder = crate::plugin::PluginGroupBuilder::start::<Self>();

        // Add window plugin
        builder = builder.add(WinitPlugin::new(self.window_title)
            .with_size(self.width, self.height));

        // Add input plugin
        builder = builder.add(crate::input::InputPlugin);

        builder
    }
}

impl Default for DefaultPlugins {
    fn default() -> Self {
        Self::new("FHRE App", 640, 480)
    }
}
