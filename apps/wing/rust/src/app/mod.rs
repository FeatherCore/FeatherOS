use crate::action::{
    ActionId, ACTION_APP_GROOVE, ACTION_APP_MESSAGING, ACTION_APP_NEWS, ACTION_APP_PEOPLE,
    ACTION_APP_PHONE, ACTION_APP_PHOTOS, ACTION_APP_STARS, ACTION_APP_THERMAL, ACTION_DIRTY,
    ACTION_DRAW, ACTION_ECS, ACTION_FONT, ACTION_INPUT, ACTION_LOCK_SCREEN, ACTION_SETTINGS,
    ACTION_SVG, ICON_DIRTY, ICON_DRAW, ICON_ECS, ICON_FONT, ICON_INPUT, ICON_SETTINGS, ICON_SVG,
};
use crate::asset::{
    CAMERA_ICON, EDGE_ICON, FILE_ICON, GROOVE_ICON, MESSAGE_ICON, NEWS_ICON, OUTLOOK_ICON,
    PADLOCK_ICON, PEOPLE_ICON, PHONE_ICON, PHOTOS_ICON, SETTINGS_ICON, STARS_ICON, TIPS_ICON,
    WEATHER_ICON, WingAssetId,
};
use fhre::SvgId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppId(pub u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppSurfaceState {
    Stopped,
    Running,
    Focused,
    Preview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppEntry {
    pub id: AppId,
    pub action: ActionId,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub icon: SvgId,
    pub icon_asset: WingAssetId,
    pub shell_owned: bool,
}

impl AppEntry {
    pub const EMPTY: Self = Self {
        id: AppId(0),
        action: ActionId(0),
        title: "",
        subtitle: "",
        icon: SvgId(0),
        icon_asset: WingAssetId(0),
        shell_owned: true,
    };
}

pub struct AppRegistry<const N: usize> {
    entries: [AppEntry; N],
    len: usize,
}

impl<const N: usize> AppRegistry<N> {
    pub const fn new() -> Self {
        Self {
            entries: [AppEntry::EMPTY; N],
            len: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, entry: AppEntry) -> bool {
        if self.len >= N {
            return false;
        }
        self.entries[self.len] = entry;
        self.len += 1;
        true
    }

    pub fn get(&self, index: usize) -> Option<AppEntry> {
        if index < self.len {
            Some(self.entries[index])
        } else {
            None
        }
    }

    pub fn by_action(&self, action: ActionId) -> Option<AppEntry> {
        let mut i = 0;
        while i < self.len {
            let entry = self.entries[i];
            if entry.action == action {
                return Some(entry);
            }
            i += 1;
        }
        None
    }

    pub fn by_id(&self, id: AppId) -> Option<AppEntry> {
        let mut i = 0;
        while i < self.len {
            let entry = self.entries[i];
            if entry.id == id {
                return Some(entry);
            }
            i += 1;
        }
        None
    }
}

pub const APP_FHRE_SAMPLE: AppId = AppId(1);
pub const APP_INPUT: AppId = AppId(2);
pub const APP_DIRTY: AppId = AppId(3);
pub const APP_FONT: AppId = AppId(4);
pub const APP_SVG: AppId = AppId(5);
pub const APP_ECS: AppId = AppId(6);
pub const APP_SETTINGS: AppId = AppId(7);
pub const APP_PHONE: AppId = AppId(8);
pub const APP_PEOPLE: AppId = AppId(9);
pub const APP_MESSAGING: AppId = AppId(10);
pub const APP_GROOVE: AppId = AppId(11);
pub const APP_NEWS: AppId = AppId(12);
pub const APP_PHOTOS: AppId = AppId(13);
pub const APP_STARS: AppId = AppId(14);
pub const APP_THERMAL: AppId = AppId(15);
pub const APP_LOCK_SCREEN: AppId = AppId(16);

pub fn builtin_app_registry() -> AppRegistry<20> {
    let mut registry = AppRegistry::new();
    registry.push(AppEntry {
        id: APP_FHRE_SAMPLE,
        action: ACTION_DRAW,
        title: "FHRE SAMPLE",
        subtitle: "GRAPHICS APP",
        icon: ICON_DRAW,
        icon_asset: WEATHER_ICON,
        shell_owned: false,
    });
    registry.push(AppEntry {
        id: APP_SETTINGS,
        action: ACTION_SETTINGS,
        title: "SETTINGS",
        subtitle: "SYSTEM APP",
        icon: ICON_SETTINGS,
        icon_asset: SETTINGS_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_LOCK_SCREEN,
        action: ACTION_LOCK_SCREEN,
        title: "LOCK SCREEN",
        subtitle: "W10M COVER",
        icon: ICON_SETTINGS,
        icon_asset: PADLOCK_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_PHONE,
        action: ACTION_APP_PHONE,
        title: "PHONE",
        subtitle: "DIALER",
        icon: ICON_INPUT,
        icon_asset: PHONE_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_PEOPLE,
        action: ACTION_APP_PEOPLE,
        title: "PEOPLE",
        subtitle: "CONTACTS",
        icon: ICON_ECS,
        icon_asset: PEOPLE_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_MESSAGING,
        action: ACTION_APP_MESSAGING,
        title: "MESSAGING",
        subtitle: "CHAT",
        icon: ICON_INPUT,
        icon_asset: MESSAGE_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_GROOVE,
        action: ACTION_APP_GROOVE,
        title: "GROOVE MUSIC",
        subtitle: "AUDIO",
        icon: ICON_DRAW,
        icon_asset: GROOVE_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_NEWS,
        action: ACTION_APP_NEWS,
        title: "NEWS",
        subtitle: "LIVE TILE",
        icon: ICON_SVG,
        icon_asset: NEWS_ICON,
        shell_owned: false,
    });
    registry.push(AppEntry {
        id: APP_STARS,
        action: ACTION_APP_STARS,
        title: "STARS",
        subtitle: "SKY DEMO",
        icon: ICON_SVG,
        icon_asset: STARS_ICON,
        shell_owned: false,
    });
    registry.push(AppEntry {
        id: APP_THERMAL,
        action: ACTION_APP_THERMAL,
        title: "THERMAL",
        subtitle: "SENSOR APP",
        icon: ICON_DIRTY,
        icon_asset: WEATHER_ICON,
        shell_owned: false,
    });
    registry.push(AppEntry {
        id: APP_PHOTOS,
        action: ACTION_APP_PHOTOS,
        title: "PHOTOS",
        subtitle: "GALLERY",
        icon: ICON_DRAW,
        icon_asset: PHOTOS_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: AppId(17),
        action: ACTION_DIRTY,
        title: "CAMERA",
        subtitle: "VIEWFINDER",
        icon: ICON_DRAW,
        icon_asset: CAMERA_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_INPUT,
        action: ACTION_INPUT,
        title: "INPUT",
        subtitle: "TOUCH + KEYS",
        icon: ICON_INPUT,
        icon_asset: PHONE_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_DIRTY,
        action: ACTION_DIRTY,
        title: "DIRTY RECT",
        subtitle: "CLIP + STATS",
        icon: ICON_DIRTY,
        icon_asset: FILE_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_FONT,
        action: ACTION_FONT,
        title: "FONT",
        subtitle: "A8 GLYPH PATH",
        icon: ICON_FONT,
        icon_asset: OUTLOOK_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_SVG,
        action: ACTION_SVG,
        title: "SVG ICONS",
        subtitle: "PATH CACHE",
        icon: ICON_SVG,
        icon_asset: EDGE_ICON,
        shell_owned: true,
    });
    registry.push(AppEntry {
        id: APP_ECS,
        action: ACTION_ECS,
        title: "WING ECS",
        subtitle: "DECLARATIVE UI",
        icon: ICON_ECS,
        icon_asset: TIPS_ICON,
        shell_owned: true,
    });
    registry
}
