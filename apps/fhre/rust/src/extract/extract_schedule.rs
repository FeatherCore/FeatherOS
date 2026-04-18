//! Extract Schedule - Configurable extract phase
//!
//! Allows registering custom extract systems, similar to Bevy's ExtractSchedule

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::resources::Resource;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// A function that extracts data from Main World to Render World
pub type ExtractFn = Box<dyn Fn(&MainWorld, &mut RenderWorld) + Send + Sync>;

/// Extract Schedule - Collection of extract operations
///
/// Similar to Bevy's ExtractSchedule, this allows registering
/// custom extract systems that run during the Extract phase.
///
/// # Example
/// ```rust
/// use fhre::extract::ExtractSchedule;
/// use fhre::main_world::MainWorld;
/// use fhre::render_world::RenderWorld;
///
/// fn my_extractor(main_world: &MainWorld, render_world: &mut RenderWorld) {
///     // Custom extraction logic
/// }
///
/// let mut schedule = ExtractSchedule::new();
/// schedule.add_extractor(my_extractor);
/// ```
#[derive(Default)]
pub struct ExtractSchedule {
    extractors: Vec<ExtractFn>,
}

impl Clone for ExtractSchedule {
    fn clone(&self) -> Self {
        // Note: We can't clone the function pointers, so we create an empty schedule
        // This is a limitation - extractors need to be re-registered after cloning
        Self::new()
    }
}

impl ExtractSchedule {
    /// Create a new empty ExtractSchedule
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    /// Add an extract function to the schedule
    ///
    /// # Example
    /// ```rust
    /// schedule.add_extractor(|main_world, render_world| {
    ///     // Extract custom data
    /// });
    /// ```
    pub fn add_extractor<F>(&mut self, extractor: F)
    where
        F: Fn(&MainWorld, &mut RenderWorld) + Send + Sync + 'static,
    {
        self.extractors.push(Box::new(extractor));
    }

    /// Run all extractors
    ///
    /// This is called during the Extract phase to execute all registered extractors.
    pub fn run(&self, main_world: &MainWorld, render_world: &mut RenderWorld) {
        for extractor in &self.extractors {
            extractor(main_world, render_world);
        }
    }

    /// Check if there are any extractors registered
    pub fn is_empty(&self) -> bool {
        self.extractors.is_empty()
    }

    /// Get the number of registered extractors
    pub fn len(&self) -> usize {
        self.extractors.len()
    }

    /// Clear all extractors
    pub fn clear(&mut self) {
        self.extractors.clear();
    }
}

impl Resource for ExtractSchedule {}
