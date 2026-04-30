//! Shared theme primitives for Wing shell and widgets.

use fhre::Color;

mod assets;
pub(crate) use assets::{
    AURORA_BACKGROUND_RGB565, DUSK_BACKGROUND_RGB565, ICON_AIRPLANE_A8, ICON_BLUETOOTH_A8,
    ICON_BRIGHTNESS_A8, ICON_DND_A8, ICON_FLASHLIGHT_A8, ICON_HOME_A8, ICON_PREVIEW_A8,
    ICON_ROTATE_A8, ICON_SETTINGS_A8, ICON_SIZE_LAUNCHER, ICON_SIZE_QUICK, ICON_SIZE_WING,
    ICON_TERMINAL_A8, ICON_THEME_A8, ICON_WIFI_A8, ICON_WING_A8, THEME_BACKGROUND_HEIGHT,
    THEME_BACKGROUND_WIDTH,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemePalette {
    pub background: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub border: Color,
    pub overlay: Color,
}

impl ThemePalette {
    pub const fn aurora(accent: Color) -> Self {
        Self {
            background: Color::rgb(20, 28, 46),
            surface: Color::rgb(248, 250, 252),
            surface_alt: Color::rgb(236, 242, 248),
            text: Color::rgb(58, 68, 88),
            text_muted: Color::rgb(104, 114, 132),
            accent,
            accent_hover: Color::rgb(102, 156, 255),
            accent_pressed: Color::rgb(66, 104, 180),
            success: Color::rgb(92, 214, 127),
            warning: Color::rgb(255, 191, 71),
            danger: Color::rgb(255, 107, 107),
            border: Color::rgb(220, 227, 236),
            overlay: Color::new(255, 255, 255, 120),
        }
    }
}

pub const fn shell_palette() -> ThemePalette {
    ThemePalette::aurora(Color::rgb(88, 142, 255))
}

impl ThemePalette {
    pub const fn with_accent(self, accent: Color) -> Self {
        Self::aurora(accent)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WingTheme {
    pub shell: ThemePalette,
}

impl WingTheme {
    pub const fn aurora() -> Self {
        Self { shell: shell_palette() }
    }

    pub const fn dusk() -> Self {
        Self {
            shell: ThemePalette {
                background: Color::rgb(18, 18, 26),
                surface: Color::rgb(236, 232, 244),
                surface_alt: Color::rgb(214, 210, 226),
                text: Color::rgb(56, 48, 72),
                text_muted: Color::rgb(104, 96, 124),
                accent: Color::rgb(180, 136, 255),
                accent_hover: Color::rgb(200, 160, 255),
                accent_pressed: Color::rgb(132, 94, 214),
                success: Color::rgb(94, 204, 147),
                warning: Color::rgb(245, 196, 87),
                danger: Color::rgb(240, 110, 124),
                border: Color::rgb(186, 178, 204),
                overlay: Color::new(255, 255, 255, 110),
            },
        }
    }
}

impl Default for WingTheme {
    fn default() -> Self {
        Self::aurora()
    }
}
