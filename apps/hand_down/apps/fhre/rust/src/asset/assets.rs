//! Assets - Collection for storing and managing asset instances

use crate::asset::{Asset, AssetId, AssetIndex, AssetEvent, Handle, StrongHandle};
use crate::resources::Resource;
use crate::event::Events;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU32, Ordering};

/// Allocates generational `AssetIndex` values.
pub(crate) struct AssetIndexAllocator {
    next_index: AtomicU32,
    free_indices: core::cell::RefCell<Vec<u32>>,
}

impl AssetIndexAllocator {
    pub const fn new() -> Self {
        Self {
            next_index: AtomicU32::new(0),
            free_indices: core::cell::RefCell::new(Vec::new()),
        }
    }
    
    pub fn allocate(&self) -> AssetIndex {
        let mut free = self.free_indices.borrow_mut();
        if let Some(index) = free.pop() {
            AssetIndex { index, generation: 0 }
        } else {
            let index = self.next_index.fetch_add(1, Ordering::Relaxed);
            AssetIndex { index, generation: 0 }
        }
    }
    
    pub fn deallocate(&self, index: u32) {
        self.free_indices.borrow_mut().push(index);
    }
}

/// Entry in the asset storage.
#[derive(Clone)]
enum Entry<A: Asset> {
    None,
    Some { value: Option<A>, generation: u32 },
}

/// Stores asset instances of type `A`.
///
/// This is the main collection for managing assets. Assets are identified
/// by `AssetId` and accessed via `Handle`.
///
/// # Example
///
/// ```ignore
/// let mut textures: Assets<Texture> = Assets::new();
/// let handle = textures.add(Texture { ... });
/// 
/// if let Some(texture) = textures.get(&handle) {
///     // use texture
/// }
/// ```
pub struct Assets<A: Asset> {
    storage: Vec<Entry<A>>,
    allocator: AssetIndexAllocator,
    uuid_map: BTreeMap<u128, usize>,
}

impl<A: Asset> Resource for Assets<A> {}

impl<A: Asset> Assets<A> {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            allocator: AssetIndexAllocator::new(),
            uuid_map: BTreeMap::new(),
        }
    }
    
    /// Add a new asset and return a strong handle to it.
    pub fn add(&mut self, asset: A) -> Handle<A> {
        let index = self.allocator.allocate();
        let idx = index.index as usize;
        
        if idx >= self.storage.len() {
            self.storage.resize(idx + 1, Entry::None);
        }
        
        self.storage[idx] = Entry::Some {
            value: Some(asset),
            generation: index.generation,
        };
        
        Handle::Strong(StrongHandle::new(index))
    }
    
    /// Add an asset and send event to Events resource.
    pub fn add_with_event(&mut self, asset: A, events: &mut Events) -> Handle<A> {
        let handle = self.add(asset);
        events.send(AssetEvent::Added { id: handle.id() });
        handle
    }
    
    /// Insert an asset with a specific UUID.
    pub fn insert_with_uuid(&mut self, uuid: u128, asset: A, events: Option<&mut Events>) {
        if let Some(&idx) = self.uuid_map.get(&uuid) {
            if let Entry::Some { value, .. } = &mut self.storage[idx] {
                *value = Some(asset);
                if let Some(events) = events {
                    events.send(AssetEvent::Modified { 
                        id: AssetId::<A>::from_uuid(uuid) 
                    });
                }
            }
        } else {
            let index = self.allocator.allocate();
            let idx = index.index as usize;
            
            if idx >= self.storage.len() {
                self.storage.resize(idx + 1, Entry::None);
            }
            
            self.storage[idx] = Entry::Some {
                value: Some(asset),
                generation: index.generation,
            };
            
            self.uuid_map.insert(uuid, idx);
            if let Some(events) = events {
                events.send(AssetEvent::Added { 
                    id: AssetId::<A>::from_uuid(uuid) 
                });
            }
        }
    }
    
    /// Get a reference to an asset by its ID.
    pub fn get(&self, id: impl Into<AssetId<A>>) -> Option<&A> {
        match id.into() {
            AssetId::Index { index, .. } => {
                let entry = self.storage.get(index.index as usize)?;
                match entry {
                    Entry::Some { value, generation } if *generation == index.generation => {
                        value.as_ref()
                    }
                    _ => None,
                }
            }
            AssetId::Uuid { uuid } => {
                let idx = self.uuid_map.get(&uuid)?;
                match &self.storage[*idx] {
                    Entry::Some { value, .. } => value.as_ref(),
                    Entry::None => None,
                }
            }
        }
    }
    
    /// Get a mutable reference to an asset by its ID.
    pub fn get_mut(&mut self, id: impl Into<AssetId<A>>) -> Option<&mut A> {
        let id = id.into();
        match id {
            AssetId::Index { index, .. } => {
                let entry = self.storage.get_mut(index.index as usize)?;
                match entry {
                    Entry::Some { value, generation } if *generation == index.generation => {
                        value.as_mut()
                    }
                    _ => None,
                }
            }
            AssetId::Uuid { uuid } => {
                let idx = self.uuid_map.get(&uuid).copied()?;
                match &mut self.storage[idx] {
                    Entry::Some { value, .. } => value.as_mut(),
                    Entry::None => None,
                }
            }
        }
    }
    
    /// Get a mutable reference and send Modified event.
    pub fn get_mut_with_event(&mut self, id: impl Into<AssetId<A>>, events: &mut Events) -> Option<&mut A> {
        let id = id.into();
        let result = self.get_mut(id);
        if result.is_some() {
            events.send(AssetEvent::Modified { id });
        }
        result
    }
    
    /// Check if an asset exists.
    pub fn contains(&self, id: impl Into<AssetId<A>>) -> bool {
        self.get(id).is_some()
    }
    
    /// Remove an asset by its ID.
    pub fn remove(&mut self, id: impl Into<AssetId<A>>) -> Option<A> {
        let id = id.into();
        match id {
            AssetId::Index { index, .. } => {
                let entry = self.storage.get_mut(index.index as usize)?;
                match entry {
                    Entry::Some { value, generation } if *generation == index.generation => {
                        *generation += 1;
                        value.take()
                    }
                    _ => None,
                }
            }
            AssetId::Uuid { uuid } => {
                let idx = self.uuid_map.remove(&uuid)?;
                match &mut self.storage[idx] {
                    Entry::Some { value, .. } => value.take(),
                    Entry::None => None,
                }
            }
        }
    }
    
    /// Remove an asset and send Removed event.
    pub fn remove_with_event(&mut self, id: impl Into<AssetId<A>>, events: &mut Events) -> Option<A> {
        let id = id.into();
        let result = self.remove(id);
        if result.is_some() {
            events.send(AssetEvent::Removed { id });
        }
        result
    }
    
    /// Get the number of assets.
    pub fn len(&self) -> usize {
        self.storage.iter()
            .filter(|e| matches!(e, Entry::Some { value: Some(_), .. }))
            .count()
    }
    
    /// Check if there are no assets.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Iterate over all asset IDs.
    pub fn ids(&self) -> impl Iterator<Item = AssetId<A>> + '_ {
        self.storage.iter().enumerate()
            .filter_map(|(i, entry)| {
                match entry {
                    Entry::Some { value: Some(_), generation } => {
                        Some(AssetId::new(i as u32, *generation))
                    }
                    _ => None,
                }
            })
    }
    
    /// Iterate over all assets.
    pub fn iter(&self) -> impl Iterator<Item = (AssetId<A>, &A)> + '_ {
        self.storage.iter().enumerate()
            .filter_map(|(i, entry)| {
                match entry {
                    Entry::Some { value: Some(v), generation } => {
                        Some((AssetId::new(i as u32, *generation), v))
                    }
                    _ => None,
                }
            })
    }
    
    /// Reserve a handle without inserting an asset.
    pub fn reserve_handle(&self) -> Handle<A> {
        let index = self.allocator.allocate();
        Handle::Strong(StrongHandle::new(index))
    }
}

impl<A: Asset> Default for Assets<A> {
    fn default() -> Self {
        Self::new()
    }
}
