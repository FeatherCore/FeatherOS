//! Hover Map

use alloc::collections::BTreeMap;
use crate::Entity;
use super::HitData;

pub type PointerId = u64;

#[derive(Debug, Clone, Default)]
pub struct HoverMap(pub BTreeMap<PointerId, (Entity, HitData)>);

#[derive(Debug, Clone, Default)]
pub struct PreviousHoverMap(pub BTreeMap<PointerId, (Entity, HitData)>);

impl HoverMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
    
    pub fn get(&self, id: &PointerId) -> Option<&(Entity, HitData)> {
        self.0.get(id)
    }
    
    pub fn insert(&mut self, id: PointerId, value: (Entity, HitData)) {
        self.0.insert(id, value);
    }
}

impl PreviousHoverMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl core::ops::Deref for HoverMap {
    type Target = BTreeMap<PointerId, (Entity, HitData)>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for HoverMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
