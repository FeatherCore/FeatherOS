#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsKey {
    Theme,
    PreviewEffect,
    Brightness,
    Haptic,
    ReduceMotion,
}

pub const SETTINGS_THEME_AURORA: u8 = 0;
pub const SETTINGS_THEME_DUSK: u8 = 1;
pub const SETTINGS_PREVIEW_CARDS: u8 = 0;
pub const SETTINGS_PREVIEW_SOCCER: u8 = 1;
pub const SETTINGS_PREVIEW_CUBE: u8 = 2;
pub const SETTINGS_BLOCK_SIZE: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsStorageSource {
    DataFile,
    TempFile,
    Memory,
}

const SETTINGS_BLOCK_MAGIC: [u8; 4] = [b'W', b'C', b'F', b'G'];
const SETTINGS_BLOCK_VERSION: u8 = 1;
const SETTINGS_FLAG_HAPTIC: u8 = 1 << 0;
const SETTINGS_FLAG_REDUCE_MOTION: u8 = 1 << 1;
const SETTINGS_CHECKSUM_OFFSET: usize = 12;
const SETTINGS_CHECKSUM_SEED: u32 = 0x811c_9dc5;
const SETTINGS_CHECKSUM_PRIME: u32 = 0x0100_0193;

impl SettingsKey {
    pub const fn raw(self) -> u16 {
        match self {
            Self::Theme => 1,
            Self::PreviewEffect => 2,
            Self::Brightness => 3,
            Self::Haptic => 4,
            Self::ReduceMotion => 5,
        }
    }

    pub const fn from_raw(raw: u16) -> Option<Self> {
        match raw {
            1 => Some(Self::Theme),
            2 => Some(Self::PreviewEffect),
            3 => Some(Self::Brightness),
            4 => Some(Self::Haptic),
            5 => Some(Self::ReduceMotion),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsEvent {
    pub key: SettingsKey,
    pub value: u32,
}

impl SettingsEvent {
    pub const fn new(key: SettingsKey, value: u32) -> Self {
        Self { key, value }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsSnapshot {
    pub theme: u8,
    pub preview_effect: u8,
    pub brightness: u8,
    pub haptic_enabled: bool,
    pub reduce_motion: bool,
}

impl SettingsSnapshot {
    pub const fn new(
        theme: u8,
        preview_effect: u8,
        brightness: u8,
        haptic_enabled: bool,
        reduce_motion: bool,
    ) -> Self {
        Self {
            theme,
            preview_effect,
            brightness,
            haptic_enabled,
            reduce_motion,
        }
    }

    pub fn sanitized(self) -> Self {
        let defaults = Self::default();
        Self {
            theme: if self.theme <= SETTINGS_THEME_DUSK {
                self.theme
            } else {
                defaults.theme
            },
            preview_effect: if self.preview_effect <= SETTINGS_PREVIEW_CUBE {
                self.preview_effect
            } else {
                defaults.preview_effect
            },
            brightness: self.brightness,
            haptic_enabled: self.haptic_enabled,
            reduce_motion: self.reduce_motion,
        }
    }
}

impl Default for SettingsSnapshot {
    fn default() -> Self {
        Self {
            theme: 0,
            preview_effect: 0,
            brightness: 192,
            haptic_enabled: true,
            reduce_motion: false,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SettingsMessage {
    pub key: u16,
    pub reserved: u16,
    pub value: u32,
}

impl SettingsMessage {
    pub const fn new(key: SettingsKey, value: u32) -> Self {
        Self {
            key: key.raw(),
            reserved: 0,
            value,
        }
    }

    pub const fn event(self) -> Option<SettingsEvent> {
        match SettingsKey::from_raw(self.key) {
            Some(key) => Some(SettingsEvent::new(key, self.value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsBlockError {
    BadMagic,
    UnsupportedVersion,
    BadChecksum,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsBlock {
    bytes: [u8; SETTINGS_BLOCK_SIZE],
}

impl SettingsBlock {
    pub fn from_snapshot(snapshot: SettingsSnapshot) -> Self {
        let snapshot = snapshot.sanitized();
        let mut bytes = [0u8; SETTINGS_BLOCK_SIZE];
        bytes[0] = SETTINGS_BLOCK_MAGIC[0];
        bytes[1] = SETTINGS_BLOCK_MAGIC[1];
        bytes[2] = SETTINGS_BLOCK_MAGIC[2];
        bytes[3] = SETTINGS_BLOCK_MAGIC[3];
        bytes[4] = SETTINGS_BLOCK_VERSION;
        bytes[5] = settings_flags(snapshot);
        bytes[6] = snapshot.theme;
        bytes[7] = snapshot.preview_effect;
        bytes[8] = snapshot.brightness;

        let checksum = settings_checksum(&bytes[..SETTINGS_CHECKSUM_OFFSET]);
        bytes[SETTINGS_CHECKSUM_OFFSET..SETTINGS_CHECKSUM_OFFSET + 4]
            .copy_from_slice(&checksum.to_le_bytes());

        Self { bytes }
    }

    pub const fn from_bytes(bytes: [u8; SETTINGS_BLOCK_SIZE]) -> Self {
        Self { bytes }
    }

    pub const fn bytes(&self) -> &[u8; SETTINGS_BLOCK_SIZE] {
        &self.bytes
    }

    pub const fn into_bytes(self) -> [u8; SETTINGS_BLOCK_SIZE] {
        self.bytes
    }

    pub fn decode(self) -> Result<SettingsSnapshot, SettingsBlockError> {
        if self.bytes[0] != SETTINGS_BLOCK_MAGIC[0]
            || self.bytes[1] != SETTINGS_BLOCK_MAGIC[1]
            || self.bytes[2] != SETTINGS_BLOCK_MAGIC[2]
            || self.bytes[3] != SETTINGS_BLOCK_MAGIC[3]
        {
            return Err(SettingsBlockError::BadMagic);
        }

        if self.bytes[4] != SETTINGS_BLOCK_VERSION {
            return Err(SettingsBlockError::UnsupportedVersion);
        }

        let expected = u32::from_le_bytes([
            self.bytes[SETTINGS_CHECKSUM_OFFSET],
            self.bytes[SETTINGS_CHECKSUM_OFFSET + 1],
            self.bytes[SETTINGS_CHECKSUM_OFFSET + 2],
            self.bytes[SETTINGS_CHECKSUM_OFFSET + 3],
        ]);
        let actual = settings_checksum(&self.bytes[..SETTINGS_CHECKSUM_OFFSET]);
        if actual != expected {
            return Err(SettingsBlockError::BadChecksum);
        }

        Ok(SettingsSnapshot::new(
            self.bytes[6],
            self.bytes[7],
            self.bytes[8],
            (self.bytes[5] & SETTINGS_FLAG_HAPTIC) != 0,
            (self.bytes[5] & SETTINGS_FLAG_REDUCE_MOTION) != 0,
        )
        .sanitized())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettingsMemoryStore {
    block: Option<SettingsBlock>,
}

impl SettingsMemoryStore {
    pub const fn empty() -> Self {
        Self { block: None }
    }

    pub fn load(&self) -> Option<SettingsSnapshot> {
        match self.block {
            Some(block) => block.decode().ok(),
            None => None,
        }
    }

    pub fn save(&mut self, snapshot: SettingsSnapshot) -> bool {
        self.block = Some(SettingsBlock::from_snapshot(snapshot));
        true
    }

    pub const fn block(&self) -> Option<SettingsBlock> {
        self.block
    }
}

impl Default for SettingsMemoryStore {
    fn default() -> Self {
        Self::empty()
    }
}

fn settings_flags(snapshot: SettingsSnapshot) -> u8 {
    let mut flags = 0;
    if snapshot.haptic_enabled {
        flags |= SETTINGS_FLAG_HAPTIC;
    }
    if snapshot.reduce_motion {
        flags |= SETTINGS_FLAG_REDUCE_MOTION;
    }
    flags
}

fn settings_checksum(bytes: &[u8]) -> u32 {
    let mut hash = SETTINGS_CHECKSUM_SEED;
    for byte in bytes {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(SETTINGS_CHECKSUM_PRIME);
    }
    hash
}
