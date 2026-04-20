//! Asset ID - Unique identifier for assets

use core::marker::PhantomData;

/// A generational index for identifying assets at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetIndex {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

impl AssetIndex {
    pub fn to_bits(self) -> u64 {
        ((self.generation as u64) << 32) | self.index as u64
    }

    pub fn from_bits(bits: u64) -> Self {
        Self {
            index: (bits & 0xFFFFFFFF) as u32,
            generation: (bits >> 32) as u32,
        }
    }
}

/// Unique identifier for an asset of type `A`.
#[derive(Debug)]
pub enum AssetId<A> {
    Index { index: AssetIndex, marker: PhantomData<A> },
    Uuid { uuid: u128 },
}

impl<A> Clone for AssetId<A> {
    fn clone(&self) -> Self {
        match self {
            AssetId::Index { index, marker } => AssetId::Index { index: *index, marker: *marker },
            AssetId::Uuid { uuid } => AssetId::Uuid { uuid: *uuid },
        }
    }
}

impl<A> Copy for AssetId<A> {}

impl<A> PartialEq for AssetId<A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AssetId::Index { index: a, .. }, AssetId::Index { index: b, .. }) => a == b,
            (AssetId::Uuid { uuid: a }, AssetId::Uuid { uuid: b }) => a == b,
            _ => false,
        }
    }
}

impl<A> Eq for AssetId<A> {}

impl<A> core::hash::Hash for AssetId<A> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            AssetId::Index { index, .. } => {
                0u8.hash(state);
                index.hash(state);
            }
            AssetId::Uuid { uuid } => {
                1u8.hash(state);
                uuid.hash(state);
            }
        }
    }
}

impl<A> PartialOrd for AssetId<A> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A> Ord for AssetId<A> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match (self, other) {
            (AssetId::Index { index: a, .. }, AssetId::Index { index: b, .. }) => a.cmp(b),
            (AssetId::Uuid { uuid: a }, AssetId::Uuid { uuid: b }) => a.cmp(b),
            (AssetId::Index { .. }, AssetId::Uuid { .. }) => core::cmp::Ordering::Less,
            (AssetId::Uuid { .. }, AssetId::Index { .. }) => core::cmp::Ordering::Greater,
        }
    }
}

impl<A> AssetId<A> {
    pub const DEFAULT_UUID: u128 = 0;
    
    pub fn new(index: u32, generation: u32) -> Self {
        Self::Index {
            index: AssetIndex { index, generation },
            marker: PhantomData,
        }
    }
    
    pub fn from_uuid(uuid: u128) -> Self {
        Self::Uuid { uuid }
    }
    
    pub fn index(&self) -> Option<AssetIndex> {
        match self {
            AssetId::Index { index, .. } => Some(*index),
            AssetId::Uuid { .. } => None,
        }
    }
    
    pub fn uuid(&self) -> Option<u128> {
        match self {
            AssetId::Index { .. } => None,
            AssetId::Uuid { uuid } => Some(*uuid),
        }
    }
}

impl<A> Default for AssetId<A> {
    fn default() -> Self {
        Self::Uuid { uuid: Self::DEFAULT_UUID }
    }
}

impl<A> From<AssetIndex> for AssetId<A> {
    fn from(index: AssetIndex) -> Self {
        Self::Index {
            index,
            marker: PhantomData,
        }
    }
}

impl<A> From<u128> for AssetId<A> {
    fn from(uuid: u128) -> Self {
        Self::Uuid { uuid }
    }
}
