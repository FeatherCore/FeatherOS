//! Render Asset - GPU representation of assets
//!
//! This module provides the `RenderAsset` trait for converting Main World assets
//! to Render World GPU representations, following Bevy's architecture.

use crate::asset::{Asset, AssetId, Assets, Handle};
use crate::render_world::RenderWorld;
use crate::resources::Resource;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Describes how an asset gets extracted and prepared for rendering.
///
/// Similar to Bevy's `RenderAsset` trait. In the Extract phase,
/// the source asset is transferred from Main World to Render World.
/// Then in Prepare phase, it's converted to GPU representation.
pub trait RenderAsset: Send + Sync + 'static + Sized + Clone {
    /// The source asset type in Main World.
    type SourceAsset: Asset;
    
    /// Prepare the source asset for GPU usage.
    ///
    /// This is called during the Prepare phase to create the GPU representation.
    fn prepare_asset(
        source: &Self::SourceAsset,
        render_world: &mut RenderWorld,
    ) -> Option<Self>;
    
    /// Called when the asset is unloaded.
    fn unload_asset(_id: AssetId<Self::SourceAsset>, _render_world: &mut RenderWorld) {}
}

/// Stores GPU representations of assets in Render World.
#[derive(Clone)]
pub struct RenderAssets<A: RenderAsset> {
    assets: BTreeMap<AssetId<A::SourceAsset>, A>,
}

impl<A: RenderAsset> RenderAssets<A> {
    pub fn new() -> Self {
        Self {
            assets: BTreeMap::new(),
        }
    }
    
    pub fn get(&self, id: impl Into<AssetId<A::SourceAsset>>) -> Option<&A> {
        self.assets.get(&id.into())
    }
    
    pub fn get_mut(&mut self, id: impl Into<AssetId<A::SourceAsset>>) -> Option<&mut A> {
        self.assets.get_mut(&id.into())
    }
    
    pub fn insert(&mut self, id: impl Into<AssetId<A::SourceAsset>>, asset: A) -> Option<A> {
        self.assets.insert(id.into(), asset)
    }
    
    pub fn remove(&mut self, id: impl Into<AssetId<A::SourceAsset>>) -> Option<A> {
        self.assets.remove(&id.into())
    }
    
    pub fn contains(&self, id: impl Into<AssetId<A::SourceAsset>>) -> bool {
        self.assets.contains_key(&id.into())
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (AssetId<A::SourceAsset>, &A)> {
        self.assets.iter().map(|(k, v)| (*k, v))
    }
    
    pub fn len(&self) -> usize {
        self.assets.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

impl<A: RenderAsset> Default for RenderAssets<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: RenderAsset> Resource for RenderAssets<A> {}

/// Extracted assets waiting to be prepared.
pub struct ExtractedAssets<A: RenderAsset> {
    pub extracted: Vec<(AssetId<A::SourceAsset>, A::SourceAsset)>,
    pub removed: Vec<AssetId<A::SourceAsset>>,
}

impl<A: RenderAsset> ExtractedAssets<A> {
    pub fn new() -> Self {
        Self {
            extracted: Vec::new(),
            removed: Vec::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.extracted.clear();
        self.removed.clear();
    }
}

impl<A: RenderAsset> Default for ExtractedAssets<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: RenderAsset> Resource for ExtractedAssets<A> {}
