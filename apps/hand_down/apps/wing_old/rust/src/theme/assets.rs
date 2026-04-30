//! Runtime raster assets for the Wing shell.
//!
//! Design preview PNGs live in `apps/wing/preview`. The files included here
//! are generated runtime resources under `apps/wing/resource`, already in
//! formats FHRE can upload without a PNG decoder on target.

pub const THEME_BACKGROUND_WIDTH: u32 = 480;
pub const THEME_BACKGROUND_HEIGHT: u32 = 640;

pub const ICON_SIZE_LAUNCHER: u32 = 64;
pub const ICON_SIZE_QUICK: u32 = 48;
pub const ICON_SIZE_WING: u32 = 96;

pub const AURORA_BACKGROUND_RGB565: &[u8] =
    include_bytes!("../../../resource/theme_aurora_background_480x640.rgb565");
pub const DUSK_BACKGROUND_RGB565: &[u8] =
    include_bytes!("../../../resource/theme_dusk_background_480x640.rgb565");

pub const ICON_SETTINGS_A8: &[u8] = include_bytes!("../../../resource/icon_settings_64.a8");
pub const ICON_TERMINAL_A8: &[u8] = include_bytes!("../../../resource/icon_terminal_64.a8");
pub const ICON_WIFI_A8: &[u8] = include_bytes!("../../../resource/icon_wifi_48.a8");
pub const ICON_BLUETOOTH_A8: &[u8] = include_bytes!("../../../resource/icon_bluetooth_48.a8");
pub const ICON_AIRPLANE_A8: &[u8] = include_bytes!("../../../resource/icon_airplane_48.a8");
pub const ICON_FLASHLIGHT_A8: &[u8] = include_bytes!("../../../resource/icon_flashlight_48.a8");
pub const ICON_DND_A8: &[u8] = include_bytes!("../../../resource/icon_dnd_48.a8");
pub const ICON_ROTATE_A8: &[u8] = include_bytes!("../../../resource/icon_rotate_48.a8");
pub const ICON_BRIGHTNESS_A8: &[u8] = include_bytes!("../../../resource/icon_brightness_48.a8");
pub const ICON_THEME_A8: &[u8] = include_bytes!("../../../resource/icon_theme_48.a8");
pub const ICON_PREVIEW_A8: &[u8] = include_bytes!("../../../resource/icon_preview_48.a8");
pub const ICON_HOME_A8: &[u8] = include_bytes!("../../../resource/icon_home_48.a8");
pub const ICON_WING_A8: &[u8] = include_bytes!("../../../resource/icon_wing_96.a8");
