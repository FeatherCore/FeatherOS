use fhre::{Handle, Image};

use super::ThemeVariant;

#[derive(Clone, Debug)]
pub struct WingImageResources {
    pub aurora_background: Handle<Image>,
    pub dusk_background: Handle<Image>,
    pub settings_icon: Handle<Image>,
    pub terminal_icon: Handle<Image>,
    pub wifi_icon: Handle<Image>,
    pub bluetooth_icon: Handle<Image>,
    pub airplane_icon: Handle<Image>,
    pub flashlight_icon: Handle<Image>,
    pub dnd_icon: Handle<Image>,
    pub rotate_icon: Handle<Image>,
    pub brightness_icon: Handle<Image>,
    pub theme_icon: Handle<Image>,
    pub preview_icon: Handle<Image>,
    pub home_icon: Handle<Image>,
    pub wing_icon: Handle<Image>,
    pub loaded: bool,
}

impl WingImageResources {
    pub fn theme_background(&self, variant: ThemeVariant) -> &Handle<Image> {
        match variant {
            ThemeVariant::Aurora => &self.aurora_background,
            ThemeVariant::Dusk => &self.dusk_background,
        }
    }

    pub fn icon_for_hint(&self, hint: &str) -> Option<&Handle<Image>> {
        match hint {
            "gear" | "settings" => Some(&self.settings_icon),
            "terminal" | "nsh" | "shell" => Some(&self.terminal_icon),
            "wifi" | "wifi_on" | "wifi_off" => Some(&self.wifi_icon),
            "bluetooth" | "bt_on" | "bt_off" => Some(&self.bluetooth_icon),
            "airplane" | "airplane_mode" => Some(&self.airplane_icon),
            "flashlight" | "flash_on" | "flash_off" => Some(&self.flashlight_icon),
            "dnd" | "do_not_disturb" => Some(&self.dnd_icon),
            "rotate" | "rotate_on" | "rotate_off" => Some(&self.rotate_icon),
            "brightness" => Some(&self.brightness_icon),
            "theme" => Some(&self.theme_icon),
            "preview" | "preview_card" | "preview_effect" => Some(&self.preview_icon),
            "home" => Some(&self.home_icon),
            "wing" => Some(&self.wing_icon),
            _ => None,
        }
    }
}

impl Default for WingImageResources {
    fn default() -> Self {
        Self {
            aurora_background: Handle::default(),
            dusk_background: Handle::default(),
            settings_icon: Handle::default(),
            terminal_icon: Handle::default(),
            wifi_icon: Handle::default(),
            bluetooth_icon: Handle::default(),
            airplane_icon: Handle::default(),
            flashlight_icon: Handle::default(),
            dnd_icon: Handle::default(),
            rotate_icon: Handle::default(),
            brightness_icon: Handle::default(),
            theme_icon: Handle::default(),
            preview_icon: Handle::default(),
            home_icon: Handle::default(),
            wing_icon: Handle::default(),
            loaded: false,
        }
    }
}

impl fhre::resources::Resource for WingImageResources {}
