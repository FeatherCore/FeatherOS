//! Plugin System for FHRE
//!
//! Inspired by Bevy's plugin system, allows modular app configuration.

mod plugin;
mod plugin_group;
mod winit_plugin;
mod default_plugins;
mod ui_animatable;
mod sync_component_plugin;

pub use plugin::{Plugin, PluginEntry, PluginsState};
pub use plugin_group::{PluginGroup, PluginGroupBuilder, NoopPluginGroup};
pub use winit_plugin::WinitPlugin;
pub use default_plugins::DefaultPlugins;
pub use ui_animatable::UiAnimatablePlugin;
pub use sync_component_plugin::{SyncComponentPlugin, SyncComponents};

// Re-export for convenience
pub use winit_plugin::DefaultPlugins as WinitDefaultPlugins;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::{Any, TypeId};

/// Resource that tracks registered plugins
#[derive(Default)]
pub struct Plugins {
    pub(crate) plugins: BTreeMap<TypeId, PluginEntry>,
    pub(crate) order: Vec<TypeId>,
    pub(crate) state: PluginsState,
}

impl crate::resources::Resource for Plugins {}

impl Plugins {
    pub fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
            order: Vec::new(),
            state: PluginsState::Adding,
        }
    }

    /// Check if a plugin of type T has been added
    pub fn contains<T: Plugin>(&self) -> bool {
        self.plugins.contains_key(&TypeId::of::<T>())
    }

    /// Get the number of registered plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}
