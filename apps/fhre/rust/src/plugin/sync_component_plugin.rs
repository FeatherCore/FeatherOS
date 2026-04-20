//! Sync Component Plugin - Automatically syncs components to Render World
//!
//! Inspired by Bevy's SyncComponentPlugin and ExtractComponentPlugin.
//! This plugin enables automatic component extraction to Render World.

use crate::app::App;
use crate::sync::RenderEntity;
use crate::extract::ExtractComponent;
use crate::plugin::Plugin;
use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Extract components implementing ExtractComponent trait
pub fn extract_components<C: ExtractComponent>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    let extracts: Vec<_> = main_world.query::<C>()
        .filter_map(|(main_entity, component)| {
            main_world.get_component::<RenderEntity>(main_entity)
                .map(|render_entity| (*render_entity, component))
        })
        .filter_map(|(render_entity, query_data)| {
            query_data.extract_component()
                .map(|extracted| (render_entity.id(), extracted))
        })
        .collect();

    for (render_entity, extracted) in extracts {
        render_world.insert_component(render_entity, extracted);
    }
}

/// Plugin that enables automatic component extraction to Render World
///
/// When added to an App, this plugin will:
/// 1. Register the component type for automatic extraction
/// 2. Add an extractor to the extract phase
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
///     type Out = Transform3D;
///
///     fn extract_component(&self) -> Option<Self::Out> {
///         Some(self.clone())
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
        // Add extract system to app
        app.add_extractor(move |main_world, render_world| {
            extract_components::<C>(main_world, render_world);
        });
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
    plugins: Vec<Box<dyn Fn(&mut App)>>,
}

impl SyncComponents {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn add<C: ExtractComponent + 'static>(mut self) -> Self {
        self.plugins.push(Box::new(|app: &mut App| {
            app.add_plugin(SyncComponentPlugin::<C>::default());
        }));
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
