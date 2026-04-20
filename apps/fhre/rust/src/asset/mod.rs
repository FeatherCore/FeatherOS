//! Asset System - Bevy-aligned asset management
//!
//! This module provides a simplified asset system inspired by Bevy's bevy_asset crate.
//!
//! # Architecture
//!
//! - `Asset` trait - Marker for asset types
//! - `AssetId<A>` - Unique identifier for an asset
//! - `Handle<A>` - Reference-counted handle to an asset
//! - `Assets<A>` - Collection storing asset instances
//! - `AssetEvent<A>` - Events for asset lifecycle
//!
//! # Example
//!
//! ```ignore
//! use fhre::asset::{Asset, Assets, Handle};
//!
//! #[derive(Clone, Asset)]
//! struct Texture {
//!     width: u32,
//!     height: u32,
//!     data: Vec<u8>,
//! }
//!
//! fn setup(mut textures: ResMut<Assets<Texture>>) {
//!     let handle: Handle<Texture> = textures.add(Texture { ... });
//! }
//! ```

mod id;
mod handle;
mod assets;
mod event;
mod render_asset;
mod extract_plugin;

pub use id::*;
pub use handle::*;
pub use assets::*;
pub use event::*;
pub use render_asset::*;
pub use extract_plugin::*;

use crate::Component;

/// Marker trait for asset types.
///
/// Assets are resources that can be loaded, stored, and referenced by handles.
/// They are typically large data like textures, meshes, sounds, etc.
pub trait Asset: Component + Clone + Send + Sync + 'static {}

/// Implement Asset for any type that meets the requirements.
/// For custom derive, use `#[derive(Asset)]`.
impl<T: Component + Clone + Send + Sync + 'static> Asset for T {}
