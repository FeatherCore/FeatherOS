use fhre::ImageId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WingAssetId(pub u16);

impl WingAssetId {
    pub const fn image(self) -> ImageId {
        ImageId(1000 + self.0)
    }

    pub const fn path(self) -> &'static [u8] {
        match self.0 {
            1 => asset_path(b"airplane_ic.png\0"),
            2 => asset_path(b"battery_ic.png\0"),
            3 => asset_path(b"bluetooth_ic.png\0"),
            4 => asset_path(b"brightness_ic.png\0"),
            5 => asset_path(b"camera_ic.png\0"),
            6 => asset_path(b"cellular_ic.png\0"),
            7 => asset_path(b"edge_ic.png\0"),
            8 => asset_path(b"embedded_tile.png\0"),
            9 => asset_path(b"file_ic.png\0"),
            10 => asset_path(b"groove_ic.png\0"),
            11 => asset_path(b"hotspot_ic.png\0"),
            12 => asset_path(b"img0.png\0"),
            13 => asset_path(b"img1.png\0"),
            14 => asset_path(b"img2.png\0"),
            15 => asset_path(b"img3.png\0"),
            16 => asset_path(b"img4.png\0"),
            17 => asset_path(b"img5.png\0"),
            18 => asset_path(b"img6.png\0"),
            19 => asset_path(b"img7.png\0"),
            20 => asset_path(b"img8.png\0"),
            21 => asset_path(b"img9.png\0"),
            22 => asset_path(b"location_ic.png\0"),
            23 => asset_path(b"message_ic.png\0"),
            24 => asset_path(b"microsoft_ic.png\0"),
            25 => asset_path(b"news_ic.png\0"),
            26 => asset_path(b"news_image.png\0"),
            27 => asset_path(b"news_tile.png\0"),
            28 => asset_path(b"outlook_ic.png\0"),
            29 => asset_path(b"people_ic.png\0"),
            30 => asset_path(b"phone_ic.png\0"),
            31 => asset_path(b"photo_tile.png\0"),
            32 => asset_path(b"photos_ic.png\0"),
            33 => asset_path(b"settings_ic.png\0"),
            34 => asset_path(b"sky_bg.png\0"),
            35 => asset_path(b"tips_ic.png\0"),
            36 => asset_path(b"vpn_ic.png\0"),
            37 => asset_path(b"weather_ic.png\0"),
            38 => asset_path(b"wifi_ic.png\0"),
            39 => asset_path(b"windows_logo.png\0"),
            40 => asset_path(b"wp_back.png\0"),
            41 => asset_path(b"wp_logo.png\0"),
            42 => asset_path(b"wp_next.png\0"),
            43 => asset_path(b"wp_search.png\0"),
            44 => asset_path(b"wp_settings.png\0"),
            45 => asset_path(b"wp_system.png\0"),
            46 => b"/etc/wing/resource/windows10_mobile/raw/padlock.fraw\0",
            47 => b"/etc/wing/resource/windows10_mobile/raw/settings_back.fraw\0",
            48 => b"/etc/wing/resource/windows10_mobile/raw/stars_ic.fraw\0",
            49 => b"/etc/wing/resource/windows10_mobile/raw/wp_about.fraw\0",
            50 => b"/etc/wing/resource/windows10_mobile/raw/wp_account.fraw\0",
            51 => b"/etc/wing/resource/windows10_mobile/raw/wp_apps.fraw\0",
            52 => b"/etc/wing/resource/windows10_mobile/raw/wp_devices.fraw\0",
            53 => b"/etc/wing/resource/windows10_mobile/raw/wp_network.fraw\0",
            54 => b"/etc/wing/resource/windows10_mobile/raw/wp_personalization.fraw\0",
            55 => b"/etc/wing/resource/windows10_mobile/raw/wp_privacy.fraw\0",
            56 => b"/etc/wing/resource/windows10_mobile/raw/wp_time.fraw\0",
            57 => b"/etc/wing/resource/windows10_mobile/raw/wifi_icon.fraw\0",
            _ => b"\0",
        }
    }
}

const fn asset_path(name: &'static [u8]) -> &'static [u8] {
    name
}

pub const AIRPLANE_ICON: WingAssetId = WingAssetId(1);
pub const BATTERY_ICON: WingAssetId = WingAssetId(2);
pub const BLUETOOTH_ICON: WingAssetId = WingAssetId(3);
pub const BRIGHTNESS_ICON: WingAssetId = WingAssetId(4);
pub const CAMERA_ICON: WingAssetId = WingAssetId(5);
pub const CELLULAR_ICON: WingAssetId = WingAssetId(6);
pub const EDGE_ICON: WingAssetId = WingAssetId(7);
pub const EMBEDDED_TILE: WingAssetId = WingAssetId(8);
pub const FILE_ICON: WingAssetId = WingAssetId(9);
pub const GROOVE_ICON: WingAssetId = WingAssetId(10);
pub const HOTSPOT_ICON: WingAssetId = WingAssetId(11);
pub const WALLPAPER_0: WingAssetId = WingAssetId(12);
pub const WALLPAPER_1: WingAssetId = WingAssetId(13);
pub const WALLPAPER_2: WingAssetId = WingAssetId(14);
pub const WALLPAPER_3: WingAssetId = WingAssetId(15);
pub const WALLPAPER_4: WingAssetId = WingAssetId(16);
pub const WALLPAPER_5: WingAssetId = WingAssetId(17);
pub const WALLPAPER_6: WingAssetId = WingAssetId(18);
pub const WALLPAPER_7: WingAssetId = WingAssetId(19);
pub const WALLPAPER_8: WingAssetId = WingAssetId(20);
pub const WALLPAPER_9: WingAssetId = WingAssetId(21);
pub const LOCATION_ICON: WingAssetId = WingAssetId(22);
pub const MESSAGE_ICON: WingAssetId = WingAssetId(23);
pub const MICROSOFT_ICON: WingAssetId = WingAssetId(24);
pub const NEWS_ICON: WingAssetId = WingAssetId(25);
pub const NEWS_IMAGE: WingAssetId = WingAssetId(26);
pub const NEWS_TILE: WingAssetId = WingAssetId(27);
pub const OUTLOOK_ICON: WingAssetId = WingAssetId(28);
pub const PEOPLE_ICON: WingAssetId = WingAssetId(29);
pub const PHONE_ICON: WingAssetId = WingAssetId(30);
pub const PHOTO_TILE: WingAssetId = WingAssetId(31);
pub const PHOTOS_ICON: WingAssetId = WingAssetId(32);
pub const SETTINGS_ICON: WingAssetId = WingAssetId(33);
pub const SKY_BG: WingAssetId = WingAssetId(34);
pub const TIPS_ICON: WingAssetId = WingAssetId(35);
pub const VPN_ICON: WingAssetId = WingAssetId(36);
pub const WEATHER_ICON: WingAssetId = WingAssetId(37);
pub const WIFI_ICON: WingAssetId = WingAssetId(38);
pub const WINDOWS_LOGO: WingAssetId = WingAssetId(39);
pub const WP_BACK: WingAssetId = WingAssetId(40);
pub const WP_LOGO: WingAssetId = WingAssetId(41);
pub const WP_NEXT: WingAssetId = WingAssetId(42);
pub const WP_SEARCH: WingAssetId = WingAssetId(43);
pub const WP_SETTINGS: WingAssetId = WingAssetId(44);
pub const WP_SYSTEM: WingAssetId = WingAssetId(45);
pub const PADLOCK_ICON: WingAssetId = WingAssetId(46);
pub const SETTINGS_BACK_ICON: WingAssetId = WingAssetId(47);
pub const STARS_ICON: WingAssetId = WingAssetId(48);
pub const WP_ABOUT: WingAssetId = WingAssetId(49);
pub const WP_ACCOUNT: WingAssetId = WingAssetId(50);
pub const WP_APPS: WingAssetId = WingAssetId(51);
pub const WP_DEVICES: WingAssetId = WingAssetId(52);
pub const WP_NETWORK: WingAssetId = WingAssetId(53);
pub const WP_PERSONALIZATION: WingAssetId = WingAssetId(54);
pub const WP_PRIVACY: WingAssetId = WingAssetId(55);
pub const WP_TIME: WingAssetId = WingAssetId(56);
pub const WIFI_SIGNAL_ICON: WingAssetId = WingAssetId(57);

pub const WINDOWS10_MOBILE_ASSETS: [WingAssetId; 57] = [
    AIRPLANE_ICON,
    BATTERY_ICON,
    BLUETOOTH_ICON,
    BRIGHTNESS_ICON,
    CAMERA_ICON,
    CELLULAR_ICON,
    EDGE_ICON,
    EMBEDDED_TILE,
    FILE_ICON,
    GROOVE_ICON,
    HOTSPOT_ICON,
    WALLPAPER_0,
    WALLPAPER_1,
    WALLPAPER_2,
    WALLPAPER_3,
    WALLPAPER_4,
    WALLPAPER_5,
    WALLPAPER_6,
    WALLPAPER_7,
    WALLPAPER_8,
    WALLPAPER_9,
    LOCATION_ICON,
    MESSAGE_ICON,
    MICROSOFT_ICON,
    NEWS_ICON,
    NEWS_IMAGE,
    NEWS_TILE,
    OUTLOOK_ICON,
    PEOPLE_ICON,
    PHONE_ICON,
    PHOTO_TILE,
    PHOTOS_ICON,
    SETTINGS_ICON,
    SKY_BG,
    TIPS_ICON,
    VPN_ICON,
    WEATHER_ICON,
    WIFI_ICON,
    WINDOWS_LOGO,
    WP_BACK,
    WP_LOGO,
    WP_NEXT,
    WP_SEARCH,
    WP_SETTINGS,
    WP_SYSTEM,
    PADLOCK_ICON,
    SETTINGS_BACK_ICON,
    STARS_ICON,
    WP_ABOUT,
    WP_ACCOUNT,
    WP_APPS,
    WP_DEVICES,
    WP_NETWORK,
    WP_PERSONALIZATION,
    WP_PRIVACY,
    WP_TIME,
    WIFI_SIGNAL_ICON,
];

pub const WINDOWS10_MOBILE_PREWARM_ASSETS: [WingAssetId; 16] = [
    BATTERY_ICON,
    BLUETOOTH_ICON,
    BRIGHTNESS_ICON,
    CELLULAR_ICON,
    NEWS_TILE,
    PEOPLE_ICON,
    PHONE_ICON,
    SETTINGS_ICON,
    WIFI_ICON,
    WP_BACK,
    WP_LOGO,
    WP_NEXT,
    WP_SEARCH,
    WP_SETTINGS,
    WP_SYSTEM,
    SETTINGS_BACK_ICON,
];

pub struct WingAssetManifest;

impl WingAssetManifest {
    pub const fn path_for_image(image: ImageId) -> Option<&'static [u8]> {
        let raw = image.0;
        if raw < 1001 || raw > 1057 {
            return None;
        }
        Some(WingAssetId(raw - 1000).path())
    }
}
