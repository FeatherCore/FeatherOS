//! Extract Component Plugin
//!
//! Provides automatic component extraction from Main World to Render World.
//! This is inspired by Bevy's `ExtractComponent` trait and plugin system.
//!
//! # Usage
//!
//! ```rust
//! use fhre::extract::{ExtractComponent, ExtractComponentPlugin};
//!
//! // Define a component that can be extracted
//! #[derive(Clone)]
//! struct Transform {
//!     position: Vec3,
//! }
//!
//! // Implement ExtractComponent
//! impl ExtractComponent for Transform {
//!     type QueryData = &'static Self;
//!     type QueryFilter = ();
//!     type Out = Self;
//!
//!     fn extract_component(component: &Self) -> Option<Self> {
//!         Some(component.clone())
//!     }
//! }
//!
//! // Register the plugin
//! app.add_plugin(ExtractComponentPlugin::<Transform>::default());
//! ```

use crate::main_world::{Component, Entity, MainWorld};
use crate::render_world::RenderWorld;
use crate::sync::{RenderEntity, SyncToRenderWorld};
use crate::plugin::Plugin;
use crate::app::App;
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Trait for components that can be extracted to the Render World.
///
/// Implement this trait to enable automatic extraction of components
/// from the Main World to the Render World during the extract phase.
pub trait ExtractComponent: Component + Sized {
    /// The query data type used to fetch the component.
    /// Typically `&Self` for immutable access or `&mut Self` for mutable.
    type QueryData;

    /// Additional query filter.
    /// Use `()` for no filter, or `Changed<Self>` for change detection.
    type QueryFilter;

    /// The output type inserted into the Render World.
    /// Usually `Self` for a direct copy, or a transformed type.
    type Out: Component;

    /// Extract the component data.
    ///
    /// Return `Some(output)` to insert into Render World,
    /// or `None` to skip this entity.
    fn extract_component(component: &Self) -> Option<Self::Out>;

    /// Extract from a query item (override for custom query data).
    fn extract_from_query(item: &Self) -> Option<Self::Out> {
        Self::extract_component(item)
    }
}

/// Plugin that automatically extracts a component type.
///
/// This plugin:
/// 1. Registers the extraction system
/// 2. Handles entity synchronization
/// 3. Inserts extracted components into the Render World
pub struct ExtractComponentPlugin<C: ExtractComponent> {
    _marker: PhantomData<C>,
}

impl<C: ExtractComponent> Default for ExtractComponentPlugin<C> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C: ExtractComponent> Plugin for ExtractComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_extractor(extract_component::<C>);
    }
}

impl<C: ExtractComponent> ExtractComponentPlugin<C> {
    /// Create a new instance
    pub fn new() -> Self {
        Self::default()
    }
}

/// Extract components from Main World to Render World.
///
/// This is the core extraction function that can be used as an extractor.
///
/// # Type Parameters
///
/// * `C` - The component type to extract
///
/// # Example
///
/// ```rust
/// app.add_extractor(extract_component::<Transform>);
/// ```
pub fn extract_component<C: ExtractComponent>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    // Get all entities with the component
    let entities: Vec<Entity> = main_world.entities()
        .iter()
        .filter(|entity| {
            main_world.get_component::<C>(**entity).is_some()
        })
        .copied()
        .collect();

    for entity in entities {
        // Get the component from main world
        if let Some(component) = main_world.get_component::<C>(entity) {
            // Extract the output
            if let Some(extracted) = C::extract_component(component) {
                // Get or create render entity
                let render_entity = render_world.get_or_spawn_synced(entity);

                // Insert the extracted component
                render_world.insert_component(render_entity, extracted);
            }
        }
    }
}

/// Extract components with their Transform.
///
/// This is useful for 3D objects that need position data.
pub fn extract_component_with_transform<C: ExtractComponent>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) where
    C: ExtractComponent,
{
    use crate::node::Transform;

    let entities: Vec<Entity> = main_world.entities()
        .iter()
        .filter(|entity| {
            main_world.get_component::<C>(**entity).is_some()
                && main_world.get_component::<Transform>(**entity).is_some()
        })
        .copied()
        .collect();

    for entity in entities {
        if let (Some(component), Some(transform)) = (
            main_world.get_component::<C>(entity),
            main_world.get_component::<Transform>(entity),
        ) {
            if let Some(extracted) = C::extract_component(component) {
                let render_entity = render_world.get_or_spawn_synced(entity);
                render_world.insert_component(render_entity, extracted);
                render_world.insert_component(render_entity, *transform);
            }
        }
    }
}

/// Trait for resources that can be extracted to the Render World.
pub trait ExtractResource: crate::resources::Resource + Clone {
    /// Extract the resource data.
    fn extract_resource(resource: &Self) -> Self {
        resource.clone()
    }
}

/// Plugin that automatically extracts a resource type.
pub struct ExtractResourcePlugin<R: ExtractResource> {
    _marker: PhantomData<R>,
}

impl<R: ExtractResource> Default for ExtractResourcePlugin<R> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<R: ExtractResource> Plugin for ExtractResourcePlugin<R> {
    fn build(&self, app: &mut App) {
        app.add_extractor(extract_resource::<R>);
    }
}

/// Plugin that automatically extracts a component with its Transform.
pub struct ExtractComponentWithTransformPlugin<C: ExtractComponent> {
    _marker: PhantomData<C>,
}

impl<C: ExtractComponent> Default for ExtractComponentWithTransformPlugin<C> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<C: ExtractComponent> Plugin for ExtractComponentWithTransformPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_extractor(extract_component_with_transform::<C>);
    }
}

/// Extract a resource from Main World to Render World.
pub fn extract_resource<R: ExtractResource>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    if let Some(resource) = main_world.resources().get::<R>() {
        let extracted = R::extract_resource(resource);
        render_world.insert_resource(extracted);
    }
}

/// Builder for creating custom extractors.
pub struct ExtractorBuilder {
    extractors: Vec<fn(&MainWorld, &mut RenderWorld)>,
}

impl ExtractorBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    /// Add a component extractor
    pub fn add_component<C: ExtractComponent>(mut self) -> Self {
        self.extractors.push(extract_component::<C>);
        self
    }

    /// Add a component extractor with transform
    pub fn add_component_with_transform<C: ExtractComponent>(mut self) -> Self {
        self.extractors.push(extract_component_with_transform::<C>);
        self
    }

    /// Add a resource extractor
    pub fn add_resource<R: ExtractResource>(mut self) -> Self {
        self.extractors.push(extract_resource::<R>);
        self
    }

    /// Add a custom extractor function
    pub fn add(mut self, extractor: fn(&MainWorld, &mut RenderWorld)) -> Self {
        self.extractors.push(extractor);
        self
    }

    /// Build the extractor list
    pub fn build(self) -> Vec<fn(&MainWorld, &mut RenderWorld)> {
        self.extractors
    }
}

impl Default for ExtractorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestComponent {
        value: u32,
    }

    impl Component for TestComponent {
        fn type_name() -> &'static str {
            "TestComponent"
        }
    }

    impl ExtractComponent for TestComponent {
        type QueryData = &'static Self;
        type QueryFilter = ();
        type Out = Self;

        fn extract_component(component: &Self) -> Option<Self> {
            Some(component.clone())
        }
    }

    #[test]
    fn test_extractor_builder() {
        let builder = ExtractorBuilder::new()
            .add_component::<TestComponent>();

        assert_eq!(builder.extractors.len(), 1);
    }
}
