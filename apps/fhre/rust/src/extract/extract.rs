//! Extract System Implementation
//!
//! The Extract phase syncs data from Main World to Render World.
//! This is a critical phase that bridges the game logic and rendering.

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::render_world::RenderObject;
use crate::main_world::{Transform, Sprite};
use crate::resources::Time;
use crate::math::Vec2;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Extract trait - Defines extract operations
///
/// Types that implement this trait can extract data
/// from Main World to Render World.
pub trait Extract {
    /// Extract data from Main World to Render World
    fn extract(&self, main_world: &MainWorld, render_world: &mut RenderWorld);
}

/// ExtractSchedule - Collection of extract operations
///
/// Manages multiple extract operations that run during the Extract phase.
pub struct ExtractSchedule {
    /// Extract functions
    extractors: Vec<Box<dyn Fn(&MainWorld, &mut RenderWorld)>>,
}

impl ExtractSchedule {
    /// Create a new extract schedule
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    /// Add an extract function
    pub fn add_extractor<F>(&mut self, extractor: F)
    where
        F: Fn(&MainWorld, &mut RenderWorld) + 'static,
    {
        self.extractors.push(Box::new(extractor));
    }

    /// Run all extract operations
    pub fn run(&self, main_world: &MainWorld, render_world: &mut RenderWorld) {
        // Clear previous frame's render objects
        render_world.clear_objects();
        
        // Run all extractors
        for extractor in &self.extractors {
            extractor(main_world, render_world);
        }
    }
}

impl Default for ExtractSchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// ExtractResource - Trait for resources that need extraction
///
/// Resources that need to be synced to the render world
/// can implement this trait.
pub trait ExtractResource {
    /// Extract this resource to the render world
    fn extract_resource(&self, render_world: &mut RenderWorld);
}

/// ExtractFn - Type alias for extract functions
pub type ExtractFn = fn(&MainWorld, &mut RenderWorld);

/// Default extract function for sprites
///
/// Extracts all entities with Transform and Sprite components
/// and creates RenderObjects in the Render World.
pub fn extract_sprites(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // Query all entities with Transform
    let transforms: Vec<_> = main_world.query::<Transform>().collect();
    
    // Match entities that have both Transform and Sprite components
    for (transform_entity, transform) in transforms {
        // Check if this entity also has a Sprite component
        if let Some(sprite) = main_world.get_component::<Sprite>(transform_entity) {
            if sprite.visible {
                let render_object = RenderObject {
                    position: transform.position,
                    rotation: transform.rotation,
                    scale: transform.scale,
                    color: sprite.color,
                    size: Vec2::new(sprite.width, sprite.height),
                    visible: sprite.visible,
                    z_order: 0,
                };
                render_world.add_object(render_object);
            }
        }
    }
}

/// Extract transforms only
///
/// Use this if you need to extract transforms for other purposes.
pub fn extract_transforms(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // This is a placeholder for custom transform extraction
    // The sprite extractor already handles transforms
    let _ = main_world;
    let _ = render_world;
}

/// Extract time resource
///
/// Copies time information to render world if needed.
pub fn extract_time(main_world: &MainWorld, _render_world: &mut RenderWorld) {
    // Time is typically accessed directly from resources
    // but this shows how you would extract it if needed
    if let Some(time) = main_world.resources().get::<Time>() {
        // Could store time in render world if needed
        let _elapsed = time.elapsed();
    }
}

/// Create default extract schedule
///
/// This sets up the standard extraction pipeline.
pub fn default_extract_schedule() -> ExtractSchedule {
    let mut schedule = ExtractSchedule::new();
    
    // Add default extractors
    schedule.add_extractor(extract_sprites);
    schedule.add_extractor(extract_time);
    
    schedule
}

/// Extract parameters - Helper struct for extraction
pub struct ExtractParams<'a> {
    pub main_world: &'a MainWorld,
    pub render_world: &'a mut RenderWorld,
}

impl<'a> ExtractParams<'a> {
    /// Create new extract parameters
    pub fn new(main_world: &'a MainWorld, render_world: &'a mut RenderWorld) -> Self {
        Self { main_world, render_world }
    }
}

/// Extract system - Runs all extraction
///
/// This is the main entry point for the extract phase.
pub fn extract_system(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // Clear render world for new frame
    render_world.reset();
    
    // Extract sprites (this is the main extraction)
    extract_sprites(main_world, render_world);
}

/// Plugin trait for extract systems
///
/// Allows custom extraction logic to be added.
pub trait ExtractPlugin {
    /// Register extract functions
    fn register(&self, schedule: &mut ExtractSchedule);
}

/// Built-in sprite extractor plugin
pub struct SpriteExtractor;

impl ExtractPlugin for SpriteExtractor {
    fn register(&self, schedule: &mut ExtractSchedule) {
        schedule.add_extractor(extract_sprites);
    }
}
