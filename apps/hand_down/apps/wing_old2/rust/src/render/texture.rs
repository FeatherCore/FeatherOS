use crate::core::FixedList;

use super::resource::{
    visit_resource_blob, visit_resource_manifest_blob, visit_resource_partition, ResourceEntry,
    ResourceManifest, ResourcePartition, ResourcePartitionSource, BUILTIN_RESOURCE_PARTITION,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageId(pub u16);

pub const TEXTURE_STORE_CAPACITY: usize = 8;

pub const IMAGE_WALLPAPER_AURORA: ImageId = IMAGE_WALLPAPER_AURORA_PORTRAIT;
pub const IMAGE_WALLPAPER_DUSK: ImageId = IMAGE_WALLPAPER_DUSK_PORTRAIT;
pub const IMAGE_WALLPAPER_AURORA_PORTRAIT: ImageId = ImageId(1);
pub const IMAGE_WALLPAPER_DUSK_PORTRAIT: ImageId = ImageId(2);
pub const IMAGE_WALLPAPER_AURORA_LANDSCAPE: ImageId = ImageId(3);
pub const IMAGE_WALLPAPER_DUSK_LANDSCAPE: ImageId = ImageId(4);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb565,
    A8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureResourceSource {
    Builtin,
    External,
    RuntimeFilesystem,
    BuiltinFallback,
}

impl Default for TextureResourceSource {
    fn default() -> Self {
        Self::Builtin
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Texture {
    pub id: ImageId,
    pub width: u16,
    pub height: u16,
    pub format: PixelFormat,
    pub data: &'static [u8],
}

#[derive(Default)]
pub struct TextureStore {
    textures: FixedList<Texture, TEXTURE_STORE_CAPACITY>,
    resource_source: TextureResourceSource,
}

impl TextureStore {
    pub fn with_builtin_textures() -> Self {
        let mut store = Self::default();
        store.resource_source = TextureResourceSource::Builtin;
        store.register_resource_partition(BUILTIN_RESOURCE_PARTITION);
        store
    }

    pub fn with_resource_partition(partition: ResourcePartition) -> Self {
        let mut store = Self::default();
        store.resource_source = texture_resource_source(partition.source());
        if !store.register_resource_partition(partition) {
            store.textures.clear();
            store.resource_source = TextureResourceSource::BuiltinFallback;
            store.register_resource_partition(BUILTIN_RESOURCE_PARTITION);
        }
        store
    }

    pub fn register(&mut self, texture: Texture) -> bool {
        for item in self.textures.as_mut_slice() {
            if item.id == texture.id {
                *item = texture;
                return true;
            }
        }

        self.textures.push(texture)
    }

    pub fn register_runtime(&mut self, texture: Texture) -> bool {
        let ok = self.register(texture);
        if ok {
            self.resource_source = TextureResourceSource::RuntimeFilesystem;
        }
        ok
    }

    pub fn register_manifest(&mut self, manifest: ResourceManifest) -> bool {
        let mut ok = true;
        for entry in manifest.textures {
            ok &= self.register(Texture::from_resource(*entry));
        }
        ok
    }

    pub fn register_resource_blob(&mut self, blob: &'static [u8]) -> bool {
        visit_resource_blob(blob, |entry| self.register(Texture::from_resource(entry)))
    }

    pub fn register_resource_manifest_blob(
        &mut self,
        manifest: &'static [u8],
        payload: &'static [u8],
    ) -> bool {
        visit_resource_manifest_blob(manifest, payload, |entry| {
            self.register(Texture::from_resource(entry))
        })
    }

    pub fn register_resource_partition(&mut self, partition: ResourcePartition) -> bool {
        visit_resource_partition(partition, |entry| {
            self.register(Texture::from_resource(entry))
        })
    }

    pub fn get(&self, id: ImageId) -> Option<&Texture> {
        self.textures
            .as_slice()
            .iter()
            .find(|texture| texture.id == id)
    }

    pub fn contains(&self, id: ImageId) -> bool {
        self.get(id).is_some()
    }

    pub fn shell_wallpapers_ready(&self) -> bool {
        self.contains(IMAGE_WALLPAPER_AURORA_PORTRAIT)
            && self.contains(IMAGE_WALLPAPER_DUSK_PORTRAIT)
            && self.contains(IMAGE_WALLPAPER_AURORA_LANDSCAPE)
            && self.contains(IMAGE_WALLPAPER_DUSK_LANDSCAPE)
    }

    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn capacity(&self) -> usize {
        self.textures.capacity()
    }

    pub fn overflowed(&self) -> bool {
        self.textures.overflowed()
    }

    pub fn resource_source(&self) -> TextureResourceSource {
        self.resource_source
    }
}

impl Texture {
    pub const fn from_resource(entry: ResourceEntry) -> Self {
        Self {
            id: entry.id,
            width: entry.width,
            height: entry.height,
            format: entry.format,
            data: entry.data,
        }
    }
}

fn texture_resource_source(source: ResourcePartitionSource) -> TextureResourceSource {
    match source {
        ResourcePartitionSource::Builtin => TextureResourceSource::Builtin,
        ResourcePartitionSource::External => TextureResourceSource::External,
    }
}
