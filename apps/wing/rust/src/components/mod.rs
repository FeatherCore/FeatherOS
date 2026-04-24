//! ECS components for Wing shell and legacy migration code.

mod shell;
mod widgets;

pub use shell::{AppSurface, BottomBar, CardStackRoot, GestureZone, HomeSurface, NotificationCard, NotificationLayer, NotificationStackRoot, NotificationText, NotificationTextRole, OverlayLayer, QuickSettingsPanel, ShellRoot, StatusBar, SurfaceCardSubtitle, SurfaceCardTitle, SurfacePreviewCard, SurfaceStackRoot, SurfaceText};
pub use widgets::{ButtonWidget, WidgetLayoutNode};
