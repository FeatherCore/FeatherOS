//! Plugin trait and related types

use crate::app::App;
use alloc::boxed::Box;
use alloc::string::String;
use core::any::Any;

/// A collection of FHRE app logic and configuration.
///
/// Plugins configure an [`App`]. When an [`App`] registers a plugin,
/// the plugin's [`Plugin::build`] function is run.
pub trait Plugin: Any + Send + Sync {
    /// Configures the [`App`] to which this plugin is added.
    fn build(&self, app: &mut App);

    /// Configures a name for the [`Plugin`] which is primarily used for debugging.
    fn name(&self) -> &str {
        core::any::type_name::<Self>()
    }

    /// If the plugin can be meaningfully instantiated several times in an [`App`],
    /// override this method to return `false`.
    fn is_unique(&self) -> bool {
        true
    }
}

/// Plugins state in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginsState {
    /// Plugins are being added.
    Adding,
    /// All plugins are built.
    Built,
}

impl Default for PluginsState {
    fn default() -> Self {
        PluginsState::Adding
    }
}

/// Entry for a registered plugin
pub struct PluginEntry {
    pub plugin: Box<dyn Plugin>,
    pub enabled: bool,
}

impl PluginEntry {
    pub fn new(plugin: Box<dyn Plugin>) -> Self {
        Self {
            plugin,
            enabled: true,
        }
    }
}

// Allow function pointers to be used as plugins
impl<F: Fn(&mut App) + Send + Sync + 'static> Plugin for F {
    fn build(&self, app: &mut App) {
        self(app);
    }
}
