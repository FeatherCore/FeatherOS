#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeResourceId {
    AuroraLandscape,
    AuroraPortrait,
    DuskLandscape,
    DuskPortrait,
    DefaultFont,
    ShellGlyphWorkset,
}

impl RuntimeResourceId {
    pub const fn path(self) -> &'static [u8] {
        match self {
            Self::AuroraLandscape => {
                b"/etc/wing/resource/images/shell/wallpaper_aurora_landscape.png\0"
            }
            Self::AuroraPortrait => {
                b"/etc/wing/resource/images/shell/wallpaper_aurora_portrait.png\0"
            }
            Self::DuskLandscape => {
                b"/etc/wing/resource/images/shell/wallpaper_dusk_landscape.png\0"
            }
            Self::DuskPortrait => b"/etc/wing/resource/images/shell/wallpaper_dusk_portrait.png\0",
            Self::DefaultFont => b"/etc/wing/resource/fonts/simhei.ttf\0",
            Self::ShellGlyphWorkset => b"/etc/wing/resource/fonts/shell_workset.txt\0",
        }
    }

    pub const fn is_shell_wallpaper(self) -> bool {
        matches!(
            self,
            Self::AuroraLandscape | Self::AuroraPortrait | Self::DuskLandscape | Self::DuskPortrait
        )
    }

    pub const fn is_default_font(self) -> bool {
        matches!(self, Self::DefaultFont)
    }

    pub const fn is_shell_glyph_workset(self) -> bool {
        matches!(self, Self::ShellGlyphWorkset)
    }
}

pub const RUNTIME_RESOURCE_IDS: [RuntimeResourceId; 6] = [
    RuntimeResourceId::AuroraLandscape,
    RuntimeResourceId::AuroraPortrait,
    RuntimeResourceId::DuskLandscape,
    RuntimeResourceId::DuskPortrait,
    RuntimeResourceId::DefaultFont,
    RuntimeResourceId::ShellGlyphWorkset,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeResourceKind {
    Missing,
    Unknown,
    Png,
    Jpeg,
    TrueType,
    OpenType,
    TrueTypeCollection,
    Svg,
    Text,
}

impl RuntimeResourceKind {
    pub const fn is_image(self) -> bool {
        matches!(self, Self::Png | Self::Jpeg)
    }

    pub const fn is_vector_font(self) -> bool {
        matches!(
            self,
            Self::TrueType | Self::OpenType | Self::TrueTypeCollection
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeResourceInfo {
    pub id: RuntimeResourceId,
    pub kind: RuntimeResourceKind,
    pub width: u16,
    pub height: u16,
    pub tables: u16,
}

impl RuntimeResourceInfo {
    pub const fn missing(id: RuntimeResourceId) -> Self {
        Self {
            id,
            kind: RuntimeResourceKind::Missing,
            width: 0,
            height: 0,
            tables: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeResourceSummary {
    pub files: u8,
    pub images: u8,
    pub shell_wallpapers: u8,
    pub png: u8,
    pub jpeg: u8,
    pub vector_fonts: u8,
    pub default_font: bool,
    pub default_font_tables: u16,
    pub font_worksets: u8,
    pub svg: u8,
    pub shell_svg_icons: u8,
    pub unknown: u8,
    pub missing: u8,
}

impl RuntimeResourceSummary {
    pub fn include(&mut self, info: RuntimeResourceInfo) {
        match info.kind {
            RuntimeResourceKind::Missing => self.missing = self.missing.saturating_add(1),
            RuntimeResourceKind::Unknown => {
                self.files = self.files.saturating_add(1);
                self.unknown = self.unknown.saturating_add(1);
            }
            RuntimeResourceKind::Png => {
                self.files = self.files.saturating_add(1);
                self.images = self.images.saturating_add(1);
                if info.id.is_shell_wallpaper() {
                    self.shell_wallpapers = self.shell_wallpapers.saturating_add(1);
                }
                self.png = self.png.saturating_add(1);
            }
            RuntimeResourceKind::Jpeg => {
                self.files = self.files.saturating_add(1);
                self.images = self.images.saturating_add(1);
                if info.id.is_shell_wallpaper() {
                    self.shell_wallpapers = self.shell_wallpapers.saturating_add(1);
                }
                self.jpeg = self.jpeg.saturating_add(1);
            }
            RuntimeResourceKind::TrueType
            | RuntimeResourceKind::OpenType
            | RuntimeResourceKind::TrueTypeCollection => {
                self.files = self.files.saturating_add(1);
                self.vector_fonts = self.vector_fonts.saturating_add(1);
                if info.id.is_default_font() {
                    self.default_font = true;
                    self.default_font_tables = info.tables;
                }
            }
            RuntimeResourceKind::Text => {
                self.files = self.files.saturating_add(1);
                if info.id.is_shell_glyph_workset() {
                    self.font_worksets = self.font_worksets.saturating_add(1);
                }
            }
            RuntimeResourceKind::Svg => {
                self.files = self.files.saturating_add(1);
                self.svg = self.svg.saturating_add(1);
            }
        }
    }

    pub fn include_shell_svg_icons(&mut self, count: usize) {
        let count = count.min(u8::MAX as usize) as u8;
        self.files = self.files.saturating_add(count);
        self.svg = self.svg.saturating_add(count);
        self.shell_svg_icons = self.shell_svg_icons.saturating_add(count);
    }

    pub const fn all_expected_present(self) -> bool {
        self.missing == 0 && self.unknown == 0
    }

    pub const fn shell_wallpaper_files_ready(self) -> bool {
        self.shell_wallpapers >= 4
    }

    pub const fn default_font_ready(self) -> bool {
        self.default_font
    }
}

pub fn inspect_resource_header(id: RuntimeResourceId, bytes: &[u8]) -> RuntimeResourceInfo {
    if id.is_shell_glyph_workset() {
        return RuntimeResourceInfo {
            id,
            kind: RuntimeResourceKind::Text,
            width: bytes.len().min(u16::MAX as usize) as u16,
            height: 0,
            tables: 0,
        };
    }

    if let Some(info) = inspect_png(id, bytes) {
        return info;
    }
    if let Some(info) = inspect_jpeg(id, bytes) {
        return info;
    }
    if let Some(info) = inspect_font(id, bytes) {
        return info;
    }
    if let Some(info) = inspect_svg(id, bytes) {
        return info;
    }

    RuntimeResourceInfo {
        id,
        kind: RuntimeResourceKind::Unknown,
        width: 0,
        height: 0,
        tables: 0,
    }
}

fn inspect_png(id: RuntimeResourceId, bytes: &[u8]) -> Option<RuntimeResourceInfo> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 33 || bytes.get(..8)? != PNG_SIGNATURE {
        return None;
    }
    if bytes.get(12..16)? != b"IHDR" {
        return None;
    }

    Some(RuntimeResourceInfo {
        id,
        kind: RuntimeResourceKind::Png,
        width: read_be_u32(bytes, 16)?.min(u16::MAX as u32) as u16,
        height: read_be_u32(bytes, 20)?.min(u16::MAX as u32) as u16,
        tables: 0,
    })
}

fn inspect_jpeg(id: RuntimeResourceId, bytes: &[u8]) -> Option<RuntimeResourceInfo> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }

    let mut offset = 2usize;
    while offset + 9 < bytes.len() {
        while offset < bytes.len() && bytes[offset] == 0xff {
            offset += 1;
        }
        let marker = *bytes.get(offset)?;
        offset += 1;
        if marker == 0xd9 || marker == 0xda {
            break;
        }

        let len = read_be_u16(bytes, offset)? as usize;
        if len < 2 || offset + len > bytes.len() {
            break;
        }

        if is_jpeg_sof(marker) && len >= 7 {
            let height = read_be_u16(bytes, offset + 3)?;
            let width = read_be_u16(bytes, offset + 5)?;
            return Some(RuntimeResourceInfo {
                id,
                kind: RuntimeResourceKind::Jpeg,
                width,
                height,
                tables: 0,
            });
        }

        offset += len;
    }

    Some(RuntimeResourceInfo {
        id,
        kind: RuntimeResourceKind::Jpeg,
        width: 0,
        height: 0,
        tables: 0,
    })
}

fn inspect_font(id: RuntimeResourceId, bytes: &[u8]) -> Option<RuntimeResourceInfo> {
    if bytes.len() < 12 {
        return None;
    }

    let tag = bytes.get(0..4)?;
    match tag {
        b"\x00\x01\x00\x00" | b"true" => Some(RuntimeResourceInfo {
            id,
            kind: RuntimeResourceKind::TrueType,
            width: 0,
            height: 0,
            tables: read_be_u16(bytes, 4)?,
        }),
        b"OTTO" => Some(RuntimeResourceInfo {
            id,
            kind: RuntimeResourceKind::OpenType,
            width: 0,
            height: 0,
            tables: read_be_u16(bytes, 4)?,
        }),
        b"ttcf" => Some(RuntimeResourceInfo {
            id,
            kind: RuntimeResourceKind::TrueTypeCollection,
            width: 0,
            height: 0,
            tables: read_be_u32(bytes, 8)?.min(u16::MAX as u32) as u16,
        }),
        _ => None,
    }
}

fn inspect_svg(id: RuntimeResourceId, bytes: &[u8]) -> Option<RuntimeResourceInfo> {
    let text = core::str::from_utf8(bytes.get(..bytes.len().min(512))?).ok()?;
    let trimmed = text.trim_start();
    if !trimmed.starts_with("<svg") && !trimmed.starts_with("<?xml") {
        return None;
    }
    if !trimmed.contains("<svg") {
        return None;
    }

    Some(RuntimeResourceInfo {
        id,
        kind: RuntimeResourceKind::Svg,
        width: 0,
        height: 0,
        tables: 0,
    })
}

fn is_jpeg_sof(marker: u8) -> bool {
    matches!(
        marker,
        0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
    )
}

fn read_be_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let slice = bytes.get(offset..offset + 2)?;
    Some(u16::from_be_bytes([slice[0], slice[1]]))
}

fn read_be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset + 4)?;
    Some(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
}
