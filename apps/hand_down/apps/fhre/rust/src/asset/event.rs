//! Asset Events - Lifecycle events for assets

use crate::asset::{Asset, AssetId};
use crate::event::Event;

/// Events that occur in the lifecycle of an asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetEvent<A: Asset> {
    /// A new asset was added.
    Added { id: AssetId<A> },
    /// An existing asset was modified.
    Modified { id: AssetId<A> },
    /// An asset was removed.
    Removed { id: AssetId<A> },
    /// An asset's last strong handle was dropped.
    Unused { id: AssetId<A> },
    /// An asset finished loading with all dependencies.
    LoadedWithDependencies { id: AssetId<A> },
}

impl<A: Asset> AssetEvent<A> {
    pub fn id(&self) -> AssetId<A> {
        match self {
            AssetEvent::Added { id } => *id,
            AssetEvent::Modified { id } => *id,
            AssetEvent::Removed { id } => *id,
            AssetEvent::Unused { id } => *id,
            AssetEvent::LoadedWithDependencies { id } => *id,
        }
    }
    
    pub fn is_added(&self) -> bool {
        matches!(self, AssetEvent::Added { .. })
    }
    
    pub fn is_modified(&self) -> bool {
        matches!(self, AssetEvent::Modified { .. })
    }
    
    pub fn is_removed(&self) -> bool {
        matches!(self, AssetEvent::Removed { .. })
    }
    
    pub fn is_unused(&self) -> bool {
        matches!(self, AssetEvent::Unused { .. })
    }
}

impl<A: Asset> Event for AssetEvent<A> {}
