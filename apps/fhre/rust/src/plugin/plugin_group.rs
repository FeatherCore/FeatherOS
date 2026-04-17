//! PluginGroup for organizing multiple plugins

use crate::app::App;
use crate::plugin::{Plugin, PluginEntry};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::any::TypeId;

/// Combines multiple [`Plugin`]s into a single unit.
pub trait PluginGroup: Sized {
    /// Configures the [`Plugin`]s that are to be added.
    fn build(self) -> PluginGroupBuilder;

    /// Configures a name for the [`PluginGroup`] which is primarily used for debugging.
    fn name() -> String {
        core::any::type_name::<Self>().to_string()
    }
}

/// Facilitates the creation and configuration of a [`PluginGroup`].
pub struct PluginGroupBuilder {
    group_name: String,
    plugins: BTreeMap<TypeId, PluginEntry>,
    order: Vec<TypeId>,
}

impl PluginGroupBuilder {
    /// Start a new builder for the [`PluginGroup`].
    pub fn start<PG: PluginGroup>() -> Self {
        Self {
            group_name: PG::name(),
            plugins: BTreeMap::new(),
            order: Vec::new(),
        }
    }

    /// Checks if the [`PluginGroupBuilder`] contains the given [`Plugin`].
    pub fn contains<T: Plugin>(&self) -> bool {
        self.plugins.contains_key(&TypeId::of::<T>())
    }

    /// Returns `true` if the [`PluginGroupBuilder`] contains the given [`Plugin`] and it's enabled.
    pub fn enabled<T: Plugin>(&self) -> bool {
        self.plugins
            .get(&TypeId::of::<T>())
            .map(|e| e.enabled)
            .unwrap_or(false)
    }

    /// Adds the plugin [`Plugin`] at the end of this [`PluginGroupBuilder`].
    pub fn add<T: Plugin>(mut self, plugin: T) -> Self {
        let type_id = TypeId::of::<T>();
        self.order.push(type_id);
        self.plugins.insert(type_id, PluginEntry::new(Box::new(plugin)));
        self
    }

    /// Adds a [`PluginGroup`] at the end of this [`PluginGroupBuilder`].
    pub fn add_group(mut self, group: impl PluginGroup) -> Self {
        let PluginGroupBuilder {
            mut plugins, order, ..
        } = group.build();

        for plugin_id in order {
            if let Some(entry) = plugins.remove(&plugin_id) {
                self.order.push(plugin_id);
                self.plugins.insert(plugin_id, entry);
            }
        }

        self
    }

    /// Disables a [`Plugin`], preventing it from being added to the [`App`].
    pub fn disable<T: Plugin>(mut self) -> Self {
        if let Some(entry) = self.plugins.get_mut(&TypeId::of::<T>()) {
            entry.enabled = false;
        }
        self
    }

    /// Enables a [`Plugin`].
    pub fn enable<T: Plugin>(mut self) -> Self {
        if let Some(entry) = self.plugins.get_mut(&TypeId::of::<T>()) {
            entry.enabled = true;
        }
        self
    }

    /// Consumes the [`PluginGroupBuilder`] and builds the contained [`Plugin`]s.
    pub fn finish(mut self, app: &mut App) {
        for ty in &self.order {
            if let Some(entry) = self.plugins.remove(ty) {
                if entry.enabled {
                    app.add_boxed_plugin(entry.plugin);
                }
            }
        }
    }
}

impl PluginGroup for PluginGroupBuilder {
    fn build(self) -> PluginGroupBuilder {
        self
    }
}

/// A plugin group which doesn't do anything.
pub struct NoopPluginGroup;

impl Default for NoopPluginGroup {
    fn default() -> Self {
        Self
    }
}

impl PluginGroup for NoopPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
    }
}
