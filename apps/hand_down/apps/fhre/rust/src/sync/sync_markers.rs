//! Sync Markers - Entity synchronization markers between Main World and Render World
//!
//! Inspired by Bevy's sync_world module:
//! - SyncToRenderWorld: Marks entities that need to be synced to Render World
//! - RenderEntity: Stores the corresponding Render World entity ID
//! - MainEntity: Stores the corresponding Main World entity ID

use crate::{Entity, Component};

/// Marker component that indicates an entity needs to be synchronized to the Render World.
///
/// This component is automatically added as a required component when using
/// ExtractComponentPlugin or SyncComponentPlugin.
///
/// # Example
/// ```
/// commands.spawn()
///     .insert(Player { name: "Player1".into() })
///     .insert(SyncToRenderWorld); // This entity will be synced to Render World
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SyncToRenderWorld;

impl Component for SyncToRenderWorld {
    fn type_name() -> &'static str {
        "SyncToRenderWorld"
    }
}

/// Component added to Main World entities to track the corresponding Render World entity.
///
/// This is automatically inserted by the sync system when an entity with
/// `SyncToRenderWorld` is spawned.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderEntity(pub Entity);

impl RenderEntity {
    /// Get the Render World entity ID
    pub fn id(&self) -> Entity {
        self.0
    }
}

impl Component for RenderEntity {
    fn type_name() -> &'static str {
        "RenderEntity"
    }
}

impl From<Entity> for RenderEntity {
    fn from(entity: Entity) -> Self {
        RenderEntity(entity)
    }
}

/// Component added to Render World entities to track the corresponding Main World entity.
///
/// This is automatically inserted when the sync system creates an entity in Render World.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MainEntity(pub Entity);

impl MainEntity {
    /// Get the Main World entity ID
    pub fn id(&self) -> Entity {
        self.0
    }
}

impl Component for MainEntity {
    fn type_name() -> &'static str {
        "MainEntity"
    }
}

impl From<Entity> for MainEntity {
    fn from(entity: Entity) -> Self {
        MainEntity(entity)
    }
}
