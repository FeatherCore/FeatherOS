//! Extract Plugin - Bevy-aligned extraction system
//!
//! This module implements the extraction phase that syncs data from Main World to Render World.
//!
//! # Architecture (aligned with Bevy)
//!
//! 1. **Extractors** - Collection of extraction functions
//! 2. **Extract<P>** - A wrapper for accessing MainWorld data during extraction
//! 3. **ExtractComponent** - Trait for components that can be extracted
//! 4. **ExtractComponentPlugin** - Plugin that auto-registers extraction
//!
//! # Usage
//!
//! ```ignore
//! // 1. Define your component
//! #[derive(Clone)]
//! struct MyComponent {
//!     value: f32,
//! }
//!
//! // 2. Implement ExtractComponent
//! impl ExtractComponent for MyComponent {
//!     type Out = ExtractedMyComponent;
//!     
//!     fn extract_component(&self) -> Option<Self::Out> {
//!         Some(ExtractedMyComponent { value: self.value })
//!     }
//! }
//!
//! // 3. Register plugin (auto-extracts)
//! app.add_plugin(ExtractComponentPlugin::<MyComponent>::default());
//! ```

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::resources::Resource;
use crate::plugin::Plugin;
use crate::app::App;
use crate::Component;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Schedule label for extraction systems
/// 
/// Similar to Bevy's ExtractSchedule, this runs after Main World systems
/// and before rendering.
pub struct ExtractSchedule;

/// Extract system param - allows accessing MainWorld data during ExtractSchedule.
/// 
/// This wraps a system parameter type `P` that would normally access Render World,
/// but instead provides access to Main World data.
/// 
/// # Example
/// 
/// ```ignore
/// fn extract_cubes(
///     mut commands: Commands,
///     cubes: Extract<Query<&Cube>>,
/// ) {
///     for cube in &cubes {
///         // Extract cube data to render world
///     }
/// }
/// ```
pub struct Extract<P> {
    item: P,
}

impl<P> Extract<P> {
    pub fn new(item: P) -> Self {
        Self { item }
    }
    
    pub fn into_inner(self) -> P {
        self.item
    }
}

impl<P> core::ops::Deref for Extract<P> {
    type Target = P;
    
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<P> core::ops::DerefMut for Extract<P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

/// Trait for components that can be extracted to Render World.
/// 
/// Similar to Bevy's ExtractComponent trait. Components implement this
/// to define how they should be extracted.
/// 
/// # Example
/// 
/// ```ignore
/// impl ExtractComponent for Cube {
///     type Out = ExtractedMesh;
///     
///     fn extract_component(&self) -> Option<ExtractedMesh> {
///         Some(ExtractedMesh {
///             vertices: self.get_vertices().to_vec(),
///             // ...
///         })
///     }
/// }
/// ```
pub trait ExtractComponent: Component + Clone + 'static {
    /// The output component(s) inserted into Render World
    type Out: Component;
    
    /// Extract the component from Main World to Render World
    /// 
    /// Returns None if the component should not be extracted.
    fn extract_component(&self) -> Option<Self::Out>;
}

/// Trait for components that need Transform during extraction.
/// 
/// This is a convenience trait for the common case of extracting
/// components that depend on their Transform.
pub trait ExtractComponentWithTransform: Component + Clone + 'static {
    /// The output component(s) inserted into Render World
    type Out: Component;
    
    /// Extract the component with its transform
    fn extract_component(&self, transform: &crate::Transform) -> Option<Self::Out>;
}

/// Plugin that automatically extracts a component type.
/// 
/// This plugin registers an extraction function that:
/// 1. Queries all entities with the component
/// 2. Calls `extract_component` on each
/// 3. Inserts the result into Render World
/// 
/// # Example
/// 
/// ```ignore
/// app.add_plugin(ExtractComponentPlugin::<MyComponent>::default());
/// ```
pub struct ExtractComponentPlugin<C: ExtractComponent> {
    _marker: core::marker::PhantomData<C>,
}

impl<C: ExtractComponent> Default for ExtractComponentPlugin<C> {
    fn default() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<C: ExtractComponent> Plugin for ExtractComponentPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_extractor(extract_component_fn::<C>);
    }
}

/// Extraction function for ExtractComponent
fn extract_component_fn<C: ExtractComponent>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    for (entity, component) in main_world.query::<C>() {
        if let Some(extracted) = component.extract_component() {
            let render_entity = render_world.get_or_spawn_synced(entity);
            render_world.insert_component(render_entity, extracted);
        }
    }
}

/// Plugin for components that need Transform during extraction.
pub struct ExtractComponentWithTransformPlugin<C: ExtractComponentWithTransform> {
    _marker: core::marker::PhantomData<C>,
}

impl<C: ExtractComponentWithTransform> Default for ExtractComponentWithTransformPlugin<C> {
    fn default() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<C: ExtractComponentWithTransform> Plugin for ExtractComponentWithTransformPlugin<C> {
    fn build(&self, app: &mut App) {
        app.add_extractor(extract_component_with_transform_fn::<C>);
    }
}

/// Extraction function for ExtractComponentWithTransform
fn extract_component_with_transform_fn<C: ExtractComponentWithTransform>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    for (entity, component) in main_world.query::<C>() {
        if let Some(transform) = main_world.get_component::<crate::Transform>(entity) {
            if let Some(extracted) = component.extract_component(&transform) {
                let render_entity = render_world.get_or_spawn_synced(entity);
                render_world.insert_component(render_entity, extracted);
            }
        }
    }
}

/// Plugin that sets up the extraction system.
pub struct ExtractPlugin;

impl Plugin for ExtractPlugin {
    fn build(&self, _app: &mut App) {
    }
}

/// Run the extraction phase.
/// 
/// This is called by App::update_and_render() to sync Main World to Render World.
pub fn run_extraction(
    main_world: &mut MainWorld,
    render_world: &mut RenderWorld,
    extract_systems: &[Box<dyn Fn(&MainWorld, &mut RenderWorld)>],
) {
    crate::sync::entity_sync_system(main_world, render_world);
    
    render_world.clear_views();
    
    for system in extract_systems {
        system(main_world, render_world);
    }
}

/// Type-erased extractor function
pub type ExtractorFn = Box<dyn Fn(&MainWorld, &mut RenderWorld)>;

/// Collection of extractors.
/// 
/// This is the main API for registering extraction functions.
/// 
/// # Example
/// 
/// ```ignore
/// let mut extractors = Extractors::new();
/// extractors.add(extract_view);
/// extractors.add(extract_3d_components);
/// extractors.run(&main_world, &mut render_world);
/// ```
pub struct Extractors {
    extractors: Vec<ExtractorFn>,
}

impl Extractors {
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }
    
    pub fn add<F>(&mut self, extractor: F)
    where
        F: Fn(&MainWorld, &mut RenderWorld) + 'static,
    {
        self.extractors.push(Box::new(extractor));
    }
    
    pub fn run(&self, main_world: &MainWorld, render_world: &mut RenderWorld) {
        for extractor in &self.extractors {
            extractor(main_world, render_world);
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.extractors.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.extractors.len()
    }
}

impl Default for Extractors {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for Extractors {}
