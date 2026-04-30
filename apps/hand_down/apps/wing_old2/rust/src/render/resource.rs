use super::texture::{ImageId, PixelFormat};

const RESOURCE_BLOB_MAGIC: &[u8; 4] = b"WRS1";
const RESOURCE_BLOB_HEADER_LEN: usize = 16;
const RESOURCE_BLOB_RECORD_LEN: usize = 20;
const RESOURCE_MANIFEST_MAGIC: &[u8; 4] = b"WRM1";
const RESOURCE_MANIFEST_HEADER_LEN: usize = 16;
const RESOURCE_MANIFEST_RECORD_LEN: usize = RESOURCE_BLOB_RECORD_LEN;
const RESOURCE_FORMAT_RGB565: u8 = 1;
const RESOURCE_FORMAT_A8: u8 = 2;

#[derive(Clone, Copy, Debug)]
pub struct ResourceEntry {
    pub id: ImageId,
    pub width: u16,
    pub height: u16,
    pub format: PixelFormat,
    pub data: &'static [u8],
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceManifest {
    pub textures: &'static [ResourceEntry],
}

impl ResourceManifest {
    pub const fn new(textures: &'static [ResourceEntry]) -> Self {
        Self { textures }
    }
}

pub const BUILTIN_RESOURCE_BLOB: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wing_assets.bin"));
pub const BUILTIN_RESOURCE_MANIFEST: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wing_manifest.bin"));
pub const BUILTIN_RESOURCE_PAYLOAD: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/wing_payload.bin"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourcePartitionSource {
    Builtin,
    External,
}

#[derive(Clone, Copy, Debug)]
pub struct ResourcePartition {
    manifest: &'static [u8],
    payload: &'static [u8],
    source: ResourcePartitionSource,
}

impl ResourcePartition {
    pub const fn new(manifest: &'static [u8], payload: &'static [u8]) -> Self {
        Self::external(manifest, payload)
    }

    pub const fn external(manifest: &'static [u8], payload: &'static [u8]) -> Self {
        Self {
            manifest,
            payload,
            source: ResourcePartitionSource::External,
        }
    }

    pub const fn builtin() -> Self {
        Self {
            manifest: BUILTIN_RESOURCE_MANIFEST,
            payload: BUILTIN_RESOURCE_PAYLOAD,
            source: ResourcePartitionSource::Builtin,
        }
    }

    pub const fn manifest(self) -> &'static [u8] {
        self.manifest
    }

    pub const fn payload(self) -> &'static [u8] {
        self.payload
    }

    pub const fn source(self) -> ResourcePartitionSource {
        self.source
    }
}

pub const BUILTIN_RESOURCE_PARTITION: ResourcePartition = ResourcePartition::builtin();

pub fn visit_resource_partition<F>(partition: ResourcePartition, visit: F) -> bool
where
    F: FnMut(ResourceEntry) -> bool,
{
    visit_resource_manifest_blob(partition.manifest(), partition.payload(), visit)
}

pub fn visit_resource_manifest_blob<F>(
    manifest: &'static [u8],
    payload: &'static [u8],
    mut visit: F,
) -> bool
where
    F: FnMut(ResourceEntry) -> bool,
{
    let Some(header) = ResourceManifestBlobHeader::parse(manifest, payload) else {
        return false;
    };

    let mut ok = true;
    let mut index = 0usize;
    while index < header.count as usize {
        let Some(record) = ResourceManifestBlobRecord::parse(manifest, header, index) else {
            return false;
        };
        let Some(format) = pixel_format(record.format) else {
            return false;
        };
        if record.len != resource_data_len(record.width, record.height, format) {
            return false;
        }
        let Some(data) = record.data(payload, header) else {
            return false;
        };

        ok &= visit(ResourceEntry {
            id: ImageId(record.id),
            width: record.width,
            height: record.height,
            format,
            data,
        });
        index += 1;
    }

    ok
}

pub fn visit_resource_blob<F>(blob: &'static [u8], mut visit: F) -> bool
where
    F: FnMut(ResourceEntry) -> bool,
{
    let Some(header) = ResourceBlobHeader::parse(blob) else {
        return false;
    };

    let mut ok = true;
    let mut index = 0usize;
    while index < header.count as usize {
        let Some(record) = ResourceBlobRecord::parse(blob, header, index) else {
            return false;
        };
        let Some(format) = pixel_format(record.format) else {
            return false;
        };
        if record.len != resource_data_len(record.width, record.height, format) {
            return false;
        }
        let Some(data) = record.data(blob, header) else {
            return false;
        };

        ok &= visit(ResourceEntry {
            id: ImageId(record.id),
            width: record.width,
            height: record.height,
            format,
            data,
        });
        index += 1;
    }

    ok
}

#[derive(Clone, Copy)]
struct ResourceManifestBlobHeader {
    count: u16,
    record_len: u16,
    payload_len: usize,
    manifest_len: usize,
}

impl ResourceManifestBlobHeader {
    fn parse(manifest: &[u8], payload: &[u8]) -> Option<Self> {
        if manifest.len() < RESOURCE_MANIFEST_HEADER_LEN
            || &manifest[..4] != RESOURCE_MANIFEST_MAGIC
        {
            return None;
        }

        let count = read_u16(manifest, 4)?;
        let record_len = read_u16(manifest, 6)?;
        let payload_len = read_u32(manifest, 8)? as usize;
        let manifest_len = read_u32(manifest, 12)? as usize;
        let table_len = (count as usize).checked_mul(record_len as usize)?;
        let expected_manifest_len = RESOURCE_MANIFEST_HEADER_LEN.checked_add(table_len)?;

        if record_len as usize != RESOURCE_MANIFEST_RECORD_LEN
            || manifest_len != expected_manifest_len
            || manifest_len > manifest.len()
            || payload_len > payload.len()
        {
            return None;
        }

        Some(Self {
            count,
            record_len,
            payload_len,
            manifest_len,
        })
    }
}

#[derive(Clone, Copy)]
struct ResourceManifestBlobRecord {
    id: u16,
    width: u16,
    height: u16,
    format: u8,
    offset: usize,
    len: usize,
}

impl ResourceManifestBlobRecord {
    fn parse(manifest: &[u8], header: ResourceManifestBlobHeader, index: usize) -> Option<Self> {
        let base = RESOURCE_MANIFEST_HEADER_LEN
            .checked_add(index.checked_mul(header.record_len as usize)?)?;
        let end = base.checked_add(RESOURCE_MANIFEST_RECORD_LEN)?;
        if end > header.manifest_len {
            return None;
        }

        Some(Self {
            id: read_u16(manifest, base)?,
            width: read_u16(manifest, base + 2)?,
            height: read_u16(manifest, base + 4)?,
            format: *manifest.get(base + 6)?,
            offset: read_u32(manifest, base + 10)? as usize,
            len: read_u32(manifest, base + 14)? as usize,
        })
    }

    fn data(
        self,
        payload: &'static [u8],
        header: ResourceManifestBlobHeader,
    ) -> Option<&'static [u8]> {
        let start = self.offset;
        let end = start.checked_add(self.len)?;
        if end > header.payload_len {
            return None;
        }

        payload.get(start..end)
    }
}

#[derive(Clone, Copy)]
struct ResourceBlobHeader {
    count: u16,
    record_len: u16,
    data_offset: usize,
    total_len: usize,
}

impl ResourceBlobHeader {
    fn parse(blob: &[u8]) -> Option<Self> {
        if blob.len() < RESOURCE_BLOB_HEADER_LEN || &blob[..4] != RESOURCE_BLOB_MAGIC {
            return None;
        }

        let count = read_u16(blob, 4)?;
        let record_len = read_u16(blob, 6)?;
        let data_offset = read_u32(blob, 8)? as usize;
        let total_len = read_u32(blob, 12)? as usize;
        let table_len = (count as usize).checked_mul(record_len as usize)?;
        let min_data_offset = RESOURCE_BLOB_HEADER_LEN.checked_add(table_len)?;

        if record_len as usize != RESOURCE_BLOB_RECORD_LEN
            || data_offset != min_data_offset
            || total_len > blob.len()
            || data_offset > total_len
        {
            return None;
        }

        Some(Self {
            count,
            record_len,
            data_offset,
            total_len,
        })
    }
}

#[derive(Clone, Copy)]
struct ResourceBlobRecord {
    id: u16,
    width: u16,
    height: u16,
    format: u8,
    offset: usize,
    len: usize,
}

impl ResourceBlobRecord {
    fn parse(blob: &[u8], header: ResourceBlobHeader, index: usize) -> Option<Self> {
        let base = RESOURCE_BLOB_HEADER_LEN
            .checked_add(index.checked_mul(header.record_len as usize)?)?;
        let end = base.checked_add(RESOURCE_BLOB_RECORD_LEN)?;
        if end > header.data_offset {
            return None;
        }

        Some(Self {
            id: read_u16(blob, base)?,
            width: read_u16(blob, base + 2)?,
            height: read_u16(blob, base + 4)?,
            format: *blob.get(base + 6)?,
            offset: read_u32(blob, base + 10)? as usize,
            len: read_u32(blob, base + 14)? as usize,
        })
    }

    fn data(self, blob: &'static [u8], header: ResourceBlobHeader) -> Option<&'static [u8]> {
        let start = header.data_offset.checked_add(self.offset)?;
        let end = start.checked_add(self.len)?;
        if end > header.total_len {
            return None;
        }

        blob.get(start..end)
    }
}

fn pixel_format(format: u8) -> Option<PixelFormat> {
    match format {
        RESOURCE_FORMAT_RGB565 => Some(PixelFormat::Rgb565),
        RESOURCE_FORMAT_A8 => Some(PixelFormat::A8),
        _ => None,
    }
}

fn resource_data_len(width: u16, height: u16, format: PixelFormat) -> usize {
    let bytes_per_pixel = match format {
        PixelFormat::Rgb565 => 2usize,
        PixelFormat::A8 => 1usize,
    };

    width as usize * height as usize * bytes_per_pixel
}

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    let bytes = data.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}
