//! Wing Shell Environment Library
//!
//! A mobile/watch-style shell built on FHRE's declarative ECS architecture.
//! Features:
//! - Card-based UI (similar to Android/Symbian smartwatch systems)
//! - Surface/app preview cards in stack layout
//! - Quick settings panel
//! - Gesture-based navigation (swipe to switch surfaces)
//!
//! Architecture follows FHRE's core design:
//! - Camera + Canvas system (z=0 plane for UI elements)
//! - Presentation Window + Input system (platform-specific)
//! - Declarative ECS with automatic synchronization

#![no_std]

extern crate alloc;

/// Default Wing screen width in portrait orientation.
pub const DEFAULT_SCREEN_WIDTH: u32 = 480;
/// Default Wing screen height in portrait orientation.
pub const DEFAULT_SCREEN_HEIGHT: u32 = 640;

mod components;
mod extract;
pub mod input;
mod platform;
mod plugin;
mod resources;
mod systems;
pub mod theme;
pub mod types;

// Re-exports for public API
pub use components::{
    AppSurface, BottomBar, ButtonWidget, CardStackRoot, GestureZone, HomeSurface,
    LauncherIcon, LauncherIconLabel, OverlayLayer, PreviewSoccerBall, QuickSettingsPanel,
    SettingsAction, SettingsPanel, SettingsRow, SettingsRowLabel, SettingsRowValue,
    ShellRoot, StatusBar, SurfaceCardSubtitle, SurfaceCardTitle, SurfacePreviewCard,
    SurfaceStackRoot, SurfaceText, SystemNavAction, SystemNavButton, SystemNavButtonLabel,
    ThemeBackdrop, WidgetLayoutNode,
    // Android-style notification components
    NotificationPanel, QuickControlTile, QuickControlType,
    BrightnessControl, NotificationCard, NotificationCardTitle, NotificationCardContent,
};
pub use extract::{
    extract_view, extract_wing_shell, queue_wing_primitives, ExtractedShellImage,
    ExtractedShellText,
};
pub use input::{ButtonInput, KeyCode, MouseButton, MouseWheel};
pub use plugin::WingShellPlugin;
pub use resources::{
    PreviewEffect, ShellContent, ShellMetrics, ShellOverlayAnimation, ShellState, SurfaceState,
    ThemeState, WingAppEntry, WingAppLaunchKind, WingImageResources,
};
pub use systems::{
    setup_wing_image_resources, setup_wing_shell, wing_app_lifecycle_system,
    wing_launcher_layout_system, wing_minimal_button_interaction_system, wing_picking_system,
    wing_preview_effect_animation_system, wing_settings_interaction_system,
    wing_settings_layout_system, wing_shell_interaction_system, wing_shell_layout_system,
    wing_shell_overlay_card_layout_system, wing_shell_overlay_layout_system,
    wing_shell_stack_layout_system, wing_theme_backdrop_layout_system,
};
pub use theme::{shell_palette, ThemePalette, WingTheme};
pub use types::{SurfaceId, WidgetId};
