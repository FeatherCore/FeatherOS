//! Sync Component Plugin - Automatically syncs components to Render World
//!
//! Inspired by Bevy's SyncComponentPlugin and ExtractComponentPlugin.
//! This plugin enables automatic component extraction to Render World.

use crate::app::App;
use crate::Component;
use crate::sync::sync_markers::SyncToRenderWorld;
use crate::sync::pending_sync::PendingSyncEntity;
use crate::ExtractSchedule;
use crate::ExtractComponent;
use crate::extract::extract_components;
use crate::plugin::Plugin;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Plugin that enables automatic component extraction to Render World
///
/// When added to an App, this plugin will:
/// 1. Register the component type for automatic extraction
/// 2. Add an extractor to the ExtractSchedule
///
/// # Example
/// ```rust
/// use fhre::prelude::*;
/// use fhre::plugin::SyncComponentPlugin;
///
/// #[derive(Component, Clone)]
/// struct Transform3D { position: Vec3 }
///
/// impl ExtractComponent for Transform3D {
///     type QueryData = Transform3D;
///     type QueryFilter = ();
///     type Out = Transform3D;
///
///     fn extract_component(item: &Self::QueryData) -> Option<Self::Out> {
///         Some(item.clone())
///     }
/// }
///
/// fn main() {
///     App::new(640, 480)
///         .add_plugin(SyncComponentPlugin::<Transform3D>::default())
///         .run();
/// }
/// ```
pub struct SyncComponentPlugin<C: ExtractComponent>(core::marker::PhantomData<C>);

impl<C: ExtractComponent> Default for SyncComponentPlugin<C> {
    fn default() -> Self {
        Self(core::marker::PhantomData)
    }
}

impl<C: ExtractComponent + 'static> Plugin for SyncComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        // Ensure PendingSyncEntity and ExtractSchedule resources exist
        if app.main_world.resources().get::<PendingSyncEntity>().is_none() {
            app.insert_resource(PendingSyncEntity::default());
        }

        if app.main_world.resources().get::<ExtractSchedule>().is_none() {
            app.insert_resource(ExtractSchedule::default());
        }

        // Add extract system to schedule
        // We use a closure that captures the type C
        let extract_fn = move |main_world: &crate::main_world::MainWorld,
                               render_world: &mut crate::render_world::RenderWorld| {
            extract_components::<C>(main_world, render_world);
        };

        // Register in ExtractSchedule
        if let Some(schedule) = app.main_world.resources_mut().get_mut::<ExtractSchedule>() {
            schedule.add_extractor(extract_fn);
        }
    }
}

/// Plugin group for syncing multiple components
///
/// # Example
/// ```rust
/// use fhre::plugin::{SyncComponents, SyncComponentPlugin};
///
/// app.add_plugins(SyncComponents::new()
///     .add::<Transform3D>()
///     .add::<Mesh>()
///     .add::<Material>()
/// );
/// ```
pub struct SyncComponents {
    plugins: Vec<Box<dyn Fn(&mut App) + Send + Sync>>,
}

impl SyncComponents {
    /// Create a new empty SyncComponents group
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Add a component type to sync
    pub fn add<C: ExtractComponent + 'static>(mut self) -> Self {
        self.plugins.push(Box::new(|app: &mut App| {
            app.add_plugin(SyncComponentPlugin::<C>::default());
        }) as Box<dyn Fn(&mut App) + Send + Sync>);
        self
    }
}

impl Plugin for SyncComponents {
    fn build(&self, app: &mut App) {
        for plugin_fn in &self.plugins {
            plugin_fn(app);
        }
    }
}

impl Default for SyncComponents {
    fn default() -> Self {
        Self::new()
    }
}
