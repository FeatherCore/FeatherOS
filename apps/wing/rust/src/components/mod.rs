//! ECS components for Wing shell.

mod shell;
mod widgets;

pub use shell::{
    AppSurface, BottomBar, CardStackRoot, GestureZone, HomeSurface, 
    OverlayLayer, QuickSettingsPanel, ShellRoot, StatusBar, 
    SurfaceCardSubtitle, SurfaceCardTitle, SurfacePreviewCard, 
    SurfaceStackRoot, SurfaceText,
    // Android-style notification components
    NotificationPanel, QuickControlTile, QuickControlType,
    BrightnessControl, NotificationCard, NotificationPriority,
    NotificationCardTitle, NotificationCardContent,
};
pub use widgets::{ButtonWidget, WidgetLayoutNode};
