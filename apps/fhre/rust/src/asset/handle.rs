//! Handle - Reference-counted handle to an asset

use crate::asset::{Asset, AssetId, AssetIndex};
use alloc::sync::Arc;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, Ordering};

/// Internal strong handle data.
#[derive(Debug)]
pub(crate) struct StrongHandleData {
    pub(crate) index: AssetIndex,
    pub(crate) ref_count: AtomicU32,
}

/// A strong reference to an asset.
///
/// When all strong handles to an asset are dropped, the asset is removed.
#[derive(Debug)]
pub struct StrongHandle {
    pub(crate) data: Arc<StrongHandleData>,
}

impl StrongHandle {
    pub(crate) fn new(index: AssetIndex) -> Self {
        Self {
            data: Arc::new(StrongHandleData {
                index,
                ref_count: AtomicU32::new(1),
            }),
        }
    }
    
    pub fn index(&self) -> AssetIndex {
        self.data.index
    }
}

impl Clone for StrongHandle {
    fn clone(&self) -> Self {
        self.data.ref_count.fetch_add(1, Ordering::Relaxed);
        Self {
            data: self.data.clone(),
        }
    }
}

impl Drop for StrongHandle {
    fn drop(&mut self) {
        self.data.ref_count.fetch_sub(1, Ordering::Relaxed);
    }
}

/// A handle to an asset of type `A`.
///
/// Handles are cheap to clone and pass around. They act as references
/// to assets stored in `Assets<A>`.
///
/// # Variants
///
/// - `Strong`: Keeps the asset alive until all handles are dropped
/// - `Weak`: References the asset but doesn't prevent removal
/// - `Uuid`: A stable identifier that doesn't track lifetime
#[derive(Debug)]
pub enum Handle<A: Asset> {
    Strong(StrongHandle),
    Weak(AssetIndex, PhantomData<A>),
    Uuid(u128, PhantomData<A>),
}

impl<A: Asset> Handle<A> {
    pub fn id(&self) -> AssetId<A> {
        match self {
            Handle::Strong(strong) => AssetId::from(strong.data.index),
            Handle::Weak(index, _) => AssetId::from(*index),
            Handle::Uuid(uuid, _) => AssetId::from(*uuid),
        }
    }
    
    pub fn is_strong(&self) -> bool {
        matches!(self, Handle::Strong(_))
    }
    
    pub fn is_weak(&self) -> bool {
        matches!(self, Handle::Weak(_, _))
    }
    
    pub fn is_uuid(&self) -> bool {
        matches!(self, Handle::Uuid(_, _))
    }
    
    pub fn index(&self) -> Option<AssetIndex> {
        match self {
            Handle::Strong(strong) => Some(strong.data.index),
            Handle::Weak(index, _) => Some(*index),
            Handle::Uuid(_, _) => None,
        }
    }
    
    pub fn downgrade(&self) -> Self {
        match self {
            Handle::Strong(strong) => Handle::Weak(strong.data.index, PhantomData),
            Handle::Weak(index, _) => Handle::Weak(*index, PhantomData),
            Handle::Uuid(uuid, _) => Handle::Uuid(*uuid, PhantomData),
        }
    }
}

impl<A: Asset> Clone for Handle<A> {
    fn clone(&self) -> Self {
        match self {
            Handle::Strong(strong) => Handle::Strong(strong.clone()),
            Handle::Weak(index, _) => Handle::Weak(*index, PhantomData),
            Handle::Uuid(uuid, _) => Handle::Uuid(*uuid, PhantomData),
        }
    }
}

impl<A: Asset> Default for Handle<A> {
    fn default() -> Self {
        Handle::Uuid(0, PhantomData)
    }
}

impl<A: Asset> PartialEq for Handle<A> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<A: Asset> Eq for Handle<A> {}

impl<A: Asset> core::hash::Hash for Handle<A> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl<A: Asset> From<AssetId<A>> for Handle<A> {
    fn from(id: AssetId<A>) -> Self {
        match id {
            AssetId::Index { index, .. } => Handle::Weak(index, PhantomData),
            AssetId::Uuid { uuid } => Handle::Uuid(uuid, PhantomData),
        }
    }
}
