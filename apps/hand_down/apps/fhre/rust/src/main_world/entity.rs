//! Entity Implementation
//!
//! Entities are lightweight identifiers for game objects.
//! They contain no data themselves - only an ID.

/// Entity - A lightweight identifier for a game object
/// 
/// Entities are the "E" in ECS (Entity-Component-System).
/// They are simple IDs that components are attached to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Entity {
    id: u64,
}

impl Entity {
    /// Create a new entity with the given ID
    pub fn new(id: u64) -> Self {
        Self { id }
    }

    /// Get the entity's unique ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Create an invalid entity (used for null checks)
    pub const fn null() -> Self {
        Self { id: u64::MAX }
    }

    /// Check if this is a valid entity
    pub fn is_valid(&self) -> bool {
        self.id != u64::MAX
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self::null()
    }
}
