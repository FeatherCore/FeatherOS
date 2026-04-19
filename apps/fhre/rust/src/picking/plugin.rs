//! Picking Plugin

use alloc::vec::Vec;
use crate::plugin::Plugin;
use crate::app::App;
use crate::resources::Resource;
use super::{HoverMap, PreviousHoverMap, PointerHits};

pub struct PointerHitsBuffer {
    hits: Vec<PointerHits>,
}

impl PointerHitsBuffer {
    pub fn new() -> Self {
        Self { hits: Vec::new() }
    }
    
    pub fn push(&mut self, hit: PointerHits) {
        self.hits.push(hit);
    }
    
    pub fn clear(&mut self) {
        self.hits.clear();
    }
    
    pub fn hits(&self) -> &[PointerHits] {
        &self.hits
    }
}

impl Resource for HoverMap {}
impl Resource for PreviousHoverMap {}
impl Resource for PointerHitsBuffer {}

pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HoverMap::new())
            .insert_resource(PreviousHoverMap::new())
            .insert_resource(PointerHitsBuffer::new());
    }
    
    fn name(&self) -> &'static str {
        "fhre::picking::PickingPlugin"
    }
}
