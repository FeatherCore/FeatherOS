//! Extract Plugin - Bevy-aligned extraction system
//!
//! This module implements the extraction phase that syncs data from Main World to Render World.
//!
//! # Architecture (aligned with Bevy)
//!
//! 1. **Extractors** - Collection of extraction functions
//! 2. **Extract<P>** - A wrapper for accessing MainWorld data during extraction
//! 3. **ExtractComponent** - Trait for components that can be extracted

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::resources::Resource;
use crate::plugin::Plugin;
use crate::app::App;
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
/// Similar to Bevy's ExtractComponent trait.
pub trait ExtractComponent: crate::Component + Clone + 'static {
    /// The query data to fetch from Main World
    type QueryData: crate::Component;
    /// The query filter
    type QueryFilter: Default;
    /// The output component(s) inserted into Render World
    type Out: crate::Component;
    
    /// Extract the component from the query item
    fn extract_component(item: &Self::QueryData) -> Option<Self::Out>;
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

/// Extract a component from Main World to Render World.
/// 
/// This is a helper function for common extraction patterns.
pub fn extract_component<C: ExtractComponent>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    for (entity, component) in main_world.query::<C::QueryData>() {
        if let Some(extracted) = C::extract_component(component) {
            if let Some(&render_entity) = render_world.main_to_render().get(&entity.id()) {
                render_world.insert_component(render_entity, extracted);
            }
        }
    }
}
