//! Plugin System for FHRE
//!
//! Inspired by Bevy's plugin system, allows modular app configuration.

mod plugin;
mod plugin_group;
mod default_plugins;
mod sync_component_plugin;

pub use plugin::{Plugin, PluginEntry, PluginsState};
pub use plugin_group::{PluginGroup, PluginGroupBuilder, NoopPluginGroup};
pub use default_plugins::DefaultPlugins;
pub use sync_component_plugin::{SyncComponentPlugin, SyncComponents};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::any::TypeId;

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

    pub fn contains<T: Plugin>(&self) -> bool {
        self.plugins.contains_key(&TypeId::of::<T>())
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}
