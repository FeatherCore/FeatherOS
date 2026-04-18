//! Pending Sync Entity - Records entities pending synchronization
//!
//! Tracks which entities need to be synced between Main World and Render World.

use crate::Entity;
use super::sync_markers::RenderEntity;
use crate::resources::Resource;
use alloc::vec::Vec;

/// Records of entity changes pending synchronization
#[derive(Debug, Clone)]
pub enum EntityRecord {
    /// Entity added SyncToRenderWorld in Main World
    /// Contains: Main World entity ID
    Added(Entity),
    
    /// Entity removed SyncToRenderWorld or was despawned in Main World
    /// Contains: Render World entity ID
    Removed(RenderEntity),
}

/// Resource storing pending entity sync records
pub struct PendingSyncEntity {
    pub(crate) records: Vec<EntityRecord>,
}

impl Default for PendingSyncEntity {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for PendingSyncEntity {}

impl PendingSyncEntity {
    /// Create a new empty PendingSyncEntity
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    
    /// Push a new record
    pub fn push(&mut self, record: EntityRecord) {
        self.records.push(record);
    }
    
    /// Drain all records
    pub fn drain(&mut self) -> impl Iterator<Item = EntityRecord> + '_ {
        self.records.drain(..)
    }
    
    /// Check if there are pending records
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    
    /// Clear all records
    pub fn clear(&mut self) {
        self.records.clear();
    }
    
    /// Get the number of pending records
    pub fn len(&self) -> usize {
        self.records.len()
    }
}
