#![no_std]

mod action;
mod animation;
mod app;
mod asset;
mod builder;
mod demo;
mod demo_all_apps;
mod demo_detail;
mod demo_home;
mod demo_launcher;
mod demo_lock;
mod demo_cortana;
mod demo_notifications;
mod demo_overlay;
mod demo_sample_app;
mod demo_settings;
mod demo_status;
mod demo_switcher;
mod demo_wallpaper;
mod key;
mod layout;
mod node;
mod shell;
mod spec;
pub mod prelude;
mod theme;
mod tree;
mod tree_layout;
mod tree_render;

pub use action::{
    action_label, ActionId, ACTION_ALL_APPS, ACTION_APP_GROOVE, ACTION_APP_MESSAGING,
    ACTION_APP_NEWS, ACTION_APP_PEOPLE, ACTION_APP_PHONE, ACTION_APP_PHOTOS, ACTION_APP_STARS,
    ACTION_APP_THERMAL, ACTION_CORTANA, ACTION_DIRTY, ACTION_DRAW, ACTION_ECS, ACTION_FONT,
    ACTION_HOME, ACTION_INPUT, ACTION_LOCK_SCREEN, ACTION_SETTINGS, ACTION_SETTINGS_ABOUT,
    ACTION_SETTINGS_ACCOUNT, ACTION_SETTINGS_APPS, ACTION_SETTINGS_DEVICES, ACTION_SETTINGS_NETWORK,
    ACTION_SETTINGS_PERSONALIZATION, ACTION_SETTINGS_PRIVACY, ACTION_SETTINGS_SYSTEM,
    ACTION_SETTINGS_TIME, ACTION_SVG, ICON_DIRTY, ICON_DRAW, ICON_ECS, ICON_FONT, ICON_INPUT,
    ICON_SETTINGS, ICON_SVG,
};
pub use animation::Animation;
pub use app::{
    builtin_app_registry, AppEntry, AppId, AppRegistry, AppSurfaceState, APP_FHRE_SAMPLE, APP_NEWS,
    APP_STARS, APP_THERMAL,
};
pub use asset::{
    WingAssetId, WingAssetManifest, WINDOWS10_MOBILE_ASSETS, WINDOWS10_MOBILE_PREWARM_ASSETS,
};
pub use builder::UiBuilder;
pub use demo::{draw_demo, draw_demo_with_state, WingDemoState};
pub use key::UiKey;
pub use layout::{ContentInset, GridLayout, LayoutRule, LayoutSpace, StackLayout};
pub use node::{
    Button, Children, Clip, IconNode, ImageNode, LayoutBox, LayoutResult, Opacity, Parent, ResolvedClip,
    TextNode, UiFrameStats, UiHit, UiNode, Visual,
};
pub use shell::{ScrollState, SettingsRoute, ShellAction, ShellActionQueue, ShellMode, ShellState};
pub use spec::{UiKind, UiSpec};
pub use tree::{apply_ui_frame, UiTree};

pub const WING_VERSION: &str = "wing-rust 0.1";
