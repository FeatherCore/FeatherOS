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

mod components;
mod extract;
pub mod input;
mod plugin;
mod resources;
mod systems;
pub mod theme;
pub mod types;

// Re-exports for public API
pub use components::{
    AppSurface, BottomBar, ButtonWidget, CardStackRoot, GestureZone, HomeSurface,
    OverlayLayer, QuickSettingsPanel, ShellRoot, StatusBar, SurfaceCardSubtitle,
    SurfaceCardTitle, SurfacePreviewCard, SurfaceStackRoot, SurfaceText, WidgetLayoutNode,
    // Android-style notification components
    NotificationPanel, QuickControlTile, QuickControlType,
    BrightnessControl, NotificationCard, NotificationCardTitle, NotificationCardContent,
};
pub use extract::{extract_view, extract_wing_shell, queue_wing_primitives, ExtractedShellText};
pub use input::{ButtonInput, KeyCode, MouseButton, MouseWheel};
pub use plugin::WingShellPlugin;
pub use resources::{
    ShellContent, ShellMetrics, ShellOverlayAnimation, ShellState, SurfaceState,
    ThemeState,
};
pub use systems::{
    setup_wing_shell, wing_minimal_button_interaction_system, wing_picking_system,
    wing_shell_interaction_system, wing_shell_layout_system, wing_shell_overlay_layout_system,
    wing_shell_stack_layout_system, wing_shell_overlay_card_layout_system,
};
pub use theme::{shell_palette, ThemePalette, WingTheme};
pub use types::{SurfaceId, WidgetId};
