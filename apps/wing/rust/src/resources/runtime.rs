use fhre::Vec2;

use crate::Wing;

/// Transitional runtime resource while Wing migrates to full ECS systems.
pub struct WingRuntime {
    pub wing: Wing,
    pub pointer_pos: Vec2,
}

impl WingRuntime {
    pub fn new(width: f32, height: f32) -> Self {
        let mut wing = Wing::new(width, height);
        wing.init();
        Self {
            wing,
            pointer_pos: Vec2::ZERO,
        }
    }
}

impl fhre::resources::Resource for WingRuntime {}
