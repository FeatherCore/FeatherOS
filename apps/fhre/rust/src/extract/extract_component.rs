//! Extract Component - Trait for extracting components from Main World to Render World
//!
//! Inspired by Bevy's extract_component module.
//! Components implementing this trait can be automatically extracted during the Extract phase.

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::Component;
use crate::sync::sync_markers::RenderEntity;
use alloc::vec::Vec;

/// Trait for components that can be extracted to the Render World
///
/// # Example
/// ```rust
/// use fhre::extract::ExtractComponent;
/// use fhre::main_world::component::Component;
///
/// #[derive(Component, Clone)]
/// struct Position { x: f32, y: f32 }
///
/// #[derive(Component)]
/// struct ExtractedPosition { x: f32, y: f32 }
///
/// impl ExtractComponent for Position {
///     type QueryData = Position;
///     type QueryFilter = ();
///     type Out = ExtractedPosition;
///
///     fn extract_component(item: &Self::QueryData) -> Option<Self::Out> {
///         Some(ExtractedPosition {
///             x: item.x,
///             y: item.y,
///         })
///     }
/// }
/// ```
pub trait ExtractComponent: Component + Clone {
    /// The data queried from Main World
    type QueryData: Component;

    /// Optional filter for the query
    type QueryFilter: Default;

    /// The output component type in Render World
    type Out: Component;

    /// Extract the component from Main World to Render World
    ///
    /// Return None to remove the component from Render World
    fn extract_component(item: &Self::QueryData) -> Option<Self::Out>;
}

/// Extract all components of type C from Main World to Render World
///
/// This system runs in the Extract schedule
pub fn extract_components<C: ExtractComponent>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    // Query all synced entities with the component
    let extracts: Vec<_> = main_world.query::<C::QueryData>()
        .filter_map(|(main_entity, component)| {
            // Get the corresponding Render World entity
            main_world.get_component::<RenderEntity>(main_entity)
                .map(|render_entity| (*render_entity, component))
        })
        .filter_map(|(render_entity, query_data)| {
            C::extract_component(query_data)
                .map(|extracted| (render_entity.id(), extracted))
        })
        .collect();

    // Insert extracted components into Render World
    for (render_entity, extracted) in extracts {
        render_world.insert_component(render_entity, extracted);
    }
}

/// Macro to implement ExtractComponent for simple clone cases
#[macro_export]
macro_rules! impl_extract_clone {
    ($component:ty, $extracted:ty) => {
        impl $crate::extract::ExtractComponent for $component {
            type QueryData = $component;
            type QueryFilter = ();
            type Out = $extracted;

            fn extract_component(item: &Self::QueryData) -> Option<Self::Out> {
                Some(item.clone().into())
            }
        }
    };
}

/// Macro to implement ExtractComponent for identity extraction (same type)
#[macro_export]
macro_rules! impl_extract_identity {
    ($component:ty) => {
        impl $crate::extract::ExtractComponent for $component {
            type QueryData = $component;
            type QueryFilter = ();
            type Out = $component;

            fn extract_component(item: &Self::QueryData) -> Option<Self::Out> {
                Some(item.clone())
            }
        }
    };
}
