//! Image Asset - Main World representation of textures
//!
//! This module provides the `Image` asset type for the Main World,
//! following Bevy's `bevy_render::texture::Image` pattern.
//!
//! # Automatic Texture Upload
//!
//! Using `TextureAssetPlugin`, textures are automatically uploaded to the GPU:
//!
//! ```ignore
//! // In your app setup
//! app.add_plugin(TextureAssetPlugin);
//!
//! // Add images to the asset collection
//! let mut images: ResMut<Assets<Image>> = ...;
//! let handle = images.add(Image::from_rgba32(64, 64, pixel_data));
//!
//! // Use the handle in components
//! // The texture will be automatically uploaded when extracted
//! ```

use crate::asset::{Asset, AssetId, RenderAsset, RenderAssets, ExtractedAssets, RenderAssetPlugin, Assets};
use crate::pipeline::{Texture, TextureFormat, Sampler};
use crate::render_world::RenderWorld;
use crate::plugin::Plugin;
use crate::app::App;
use crate::Component;
use alloc::vec::Vec;

/// Image asset stored in Main World.
///
/// This is the source asset that gets extracted and prepared for GPU usage.
/// Similar to Bevy's `bevy_render::texture::Image`.
#[derive(Clone, Debug)]
pub struct Image {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub sampler: Sampler,
}

impl Image {
    pub fn new(width: u32, height: u32, format: TextureFormat) -> Self {
        let size = (width * height * format.bytes_per_pixel()) as usize;
        Self {
            data: alloc::vec![0; size],
            width,
            height,
            format,
            sampler: Sampler::default(),
        }
    }

    pub fn from_rgba32(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            data,
            width,
            height,
            format: TextureFormat::Rgba32,
            sampler: Sampler::default(),
        }
    }

    pub fn from_rgb24(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            data,
            width,
            height,
            format: TextureFormat::Rgb24,
            sampler: Sampler::default(),
        }
    }

    pub fn with_sampler(mut self, sampler: Sampler) -> Self {
        self.sampler = sampler;
        self
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn as_texture(&self) -> Texture {
        Texture {
            data: self.data.clone(),
            width: self.width,
            height: self.height,
            format: self.format,
            sampler: self.sampler,
        }
    }
}

impl Component for Image {
    fn type_name() -> &'static str {
        "Image"
    }
}

/// GPU texture stored in Render World.
///
/// This is the prepared GPU representation of an `Image` asset.
/// The texture ID is used to reference the texture in render commands.
#[derive(Clone, Debug)]
pub struct GpuTexture {
    pub id: u32,
    pub width: u32,
    pub height: u32,
}

impl GpuTexture {
    pub fn new(id: u32, width: u32, height: u32) -> Self {
        Self { id, width, height }
    }
}

/// Counter for generating unique texture IDs.
static TEXTURE_ID_COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1);

fn next_texture_id() -> u32 {
    TEXTURE_ID_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
}

impl RenderAsset for GpuTexture {
    type SourceAsset = Image;

    fn prepare_asset(source: &Image, render_world: &mut RenderWorld) -> Option<Self> {
        let id = next_texture_id();
        let texture = source.as_texture();
        render_world.upload_texture(id, texture);
        Some(GpuTexture::new(id, source.width, source.height))
    }

    fn unload_asset(_id: AssetId<Image>, render_world: &mut RenderWorld) {
        // Note: We don't have the GpuTexture's texture_id here directly,
        // but in practice, the GpuTexture would be removed from RenderAssets
        // and we'd need to clean up the texture from the backend.
        // For now, textures are kept in the backend for simplicity.
    }
}

/// Type alias for the render assets storage of GPU textures.
pub type GpuTextures = RenderAssets<GpuTexture>;

/// Type alias for extracted image assets.
pub type ExtractedImages = ExtractedAssets<GpuTexture>;

/// Plugin that sets up automatic texture upload.
///
/// This plugin registers:
/// - `Assets<Image>` in Main World
/// - `ExtractedImages` and `GpuTextures` in Render World
/// - Extract and prepare extractors for automatic GPU upload
///
/// # Example
///
/// ```ignore
/// app.add_plugin(TextureAssetPlugin);
///
/// // Later, add textures
/// let mut images = app.main_world.resources().get_mut::<Assets<Image>>().unwrap();
/// let handle = images.add(Image::from_rgba32(64, 64, pixels));
///
/// // The texture will be automatically uploaded to GPU on next frame
/// ```
pub struct TextureAssetPlugin;

impl Plugin for TextureAssetPlugin {
    fn build(&self, app: &mut App) {
        // Register Assets<Image> in Main World
        app.main_world.resources_mut().insert(Assets::<Image>::new());
        app.add_plugin(RenderAssetPlugin::<GpuTexture>::default());
    }
}
