//! Sync Module - Entity synchronization between Main World and Render World
//!
//! This module provides the infrastructure for synchronizing entities between
//! Main World and Render World, inspired by Bevy's sync_world module.
//!
//! # Core Components
//!
//! - [`SyncToRenderWorld`]: Marker component to indicate an entity should be synced
//! - [`RenderEntity`]: Component in Main World storing the corresponding Render World entity ID
//! - [`MainEntity`]: Component in Render World storing the corresponding Main World entity ID
//! - [`PendingSyncEntity`]: Resource tracking pending synchronization operations
//!
//! # Observer Mechanism
//!
//! Entity sync is triggered automatically via Observer pattern in MainWorld:
//! - `insert_component<SyncToRenderWorld>` → adds `EntityRecord::Added`
//! - `remove_component<SyncToRenderWorld>` → adds `EntityRecord::Removed`
//! - `despawn` entity with `RenderEntity` → adds `EntityRecord::Removed`
//!
//! # Architecture
//!
//! ```text
//! Main World                              Render World
//! ----------                              ------------
//! Entity + SyncToRenderWorld              Entity + MainEntity
//!    ↓                                         ↑
//! RenderEntity (stores Render World ID) ──────┘
//! ```

pub mod sync_markers;
pub mod pending_sync;
pub mod sync_system;

pub use sync_markers::{SyncToRenderWorld, RenderEntity, MainEntity};
pub use pending_sync::{PendingSyncEntity, EntityRecord};
pub use sync_system::entity_sync_system;
