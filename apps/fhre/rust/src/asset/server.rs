//! AssetServer - Simplified asset loading interface
//!
//! This provides a unified interface for loading assets, inspired by Bevy's AssetServer.
//! For embedded systems, this is synchronous (no async loading).
//!
//! # Architecture
//!
//! ```text
//! AssetServer (Main World)
//!     │
//!     │ load() → Assets<A>
//!     │
//!     ▼
//! RenderAssetPlugin<A>
//!     │
//!     │ Extract + Prepare
//!     │
//!     ▼
//! RenderAssets<A> (Render World)
//! ```
//!
//! # Usage
//!
//! ```ignore
//! // Register asset types
//! app.init_asset::<Image>()
//!    .init_asset::<Mesh>();
//!
//! // Load assets
//! let server = app.main_world.resources().get::<AssetServer>().unwrap();
//! let texture: Handle<Image> = server.add(Image { ... });
//! ```

use crate::asset::{Asset, Assets, Handle, AssetId};
use crate::resources::Resource;
use crate::app::App;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::{TypeId, Any};

/// A trait for asset loaders.
///
/// Loaders convert raw data into asset instances.
pub trait AssetLoader: Send + Sync + 'static {
    /// The asset type this loader produces.
    type Asset: Asset;
    
    /// Load an asset from raw data.
    fn load(&self, data: &[u8]) -> Option<Self::Asset>;
    
    /// Get file extensions this loader handles.
    fn extensions(&self) -> &'static [&'static str] {
        &[]
    }
}

/// Asset registry - stores asset collections by type.
pub struct AssetRegistry {
    storages: BTreeMap<TypeId, Box<dyn Any>>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            storages: BTreeMap::new(),
        }
    }
    
    /// Register an asset type.
    pub fn register<A: Asset>(&mut self) {
        let type_id = TypeId::of::<A>();
        if !self.storages.contains_key(&type_id) {
            self.storages.insert(type_id, Box::new(Assets::<A>::new()));
        }
    }
    
    /// Get the asset storage for a type.
    pub fn get_assets<A: Asset>(&self) -> Option<&Assets<A>> {
        self.storages
            .get(&TypeId::of::<A>())
            .and_then(|boxed| boxed.downcast_ref::<Assets<A>>())
    }
    
    /// Get mutable access to asset storage.
    pub fn get_assets_mut<A: Asset>(&mut self) -> Option<&mut Assets<A>> {
        self.storages
            .get_mut(&TypeId::of::<A>())
            .and_then(|boxed| boxed.downcast_mut::<Assets<A>>())
    }
    
    /// Add an asset and return a handle.
    pub fn add<A: Asset>(&mut self, asset: A) -> Handle<A> {
        let assets = self.get_assets_mut::<A>().expect("Asset type not registered");
        assets.add(asset)
    }
    
    /// Get an asset by handle.
    pub fn get<A: Asset>(&self, handle: &Handle<A>) -> Option<&A> {
        self.get_assets::<A>()?.get(handle.id())
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for AssetRegistry {}

/// AssetServer - unified interface for asset management.
///
/// This is a simplified version of Bevy's AssetServer for embedded systems.
/// It provides synchronous asset loading (no async).
pub struct AssetServer {
    registry: AssetRegistry,
    loaders: BTreeMap<TypeId, Box<dyn Any>>,
}

impl AssetServer {
    pub fn new() -> Self {
        Self {
            registry: AssetRegistry::new(),
            loaders: BTreeMap::new(),
        }
    }
    
    /// Register an asset type.
    pub fn register_asset<A: Asset>(&mut self) {
        self.registry.register::<A>();
    }
    
    /// Register an asset loader.
    pub fn register_loader<L: AssetLoader>(&mut self, loader: L) {
        let type_id = TypeId::of::<L::Asset>();
        self.loaders.insert(type_id, Box::new(loader));
    }
    
    /// Add an asset directly and return a handle.
    pub fn add<A: Asset>(&mut self, asset: A) -> Handle<A> {
        self.registry.add(asset)
    }
    
    /// Load an asset from raw data using a registered loader.
    pub fn load<A: Asset>(&mut self, data: &[u8]) -> Option<Handle<A>> {
        let loader = self.loaders
            .get(&TypeId::of::<A>())?
            .downcast_ref::<Box<dyn AssetLoader<Asset = A>>>()?;
        
        let asset = loader.load(data)?;
        Some(self.registry.add(asset))
    }
    
    /// Get an asset by handle.
    pub fn get<A: Asset>(&self, handle: &Handle<A>) -> Option<&A> {
        self.registry.get(handle)
    }
    
    /// Get mutable access to asset storage.
    pub fn get_assets_mut<A: Asset>(&mut self) -> Option<&mut Assets<A>> {
        self.registry.get_assets_mut::<A>()
    }
    
    /// Get the asset registry.
    pub fn registry(&self) -> &AssetRegistry {
        &self.registry
    }
    
    /// Get mutable access to the asset registry.
    pub fn registry_mut(&mut self) -> &mut AssetRegistry {
        &mut self.registry
    }
}

impl Default for AssetServer {
    fn default() -> Self {
        Self::new()
    }
}

impl Resource for AssetServer {}

/// Plugin for initializing an asset type.
pub struct AssetPlugin<A: Asset> {
    _marker: core::marker::PhantomData<A>,
}

impl<A: Asset> Default for AssetPlugin<A> {
    fn default() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<A: Asset> crate::plugin::Plugin for AssetPlugin<A> {
    fn build(&self, app: &mut App) {
        // Initialize AssetServer if not exists
        if !app.main_world.resources().contains::<AssetServer>() {
            app.main_world.resources_mut().insert(AssetServer::new());
        }
        
        // Register asset type
        if let Some(server) = app.main_world.resources_mut().get_mut::<AssetServer>() {
            server.register_asset::<A>();
        }
    }
}

/// Extension trait for App to add asset support.
pub trait AppAssetExt {
    /// Initialize an asset type.
    fn init_asset<A: Asset>(&mut self) -> &mut Self;
}

impl AppAssetExt for App {
    fn init_asset<A: Asset>(&mut self) -> &mut Self {
        self.add_plugin(AssetPlugin::<A>::default());
        self
    }
}
