//! ECS components for Wing shell.

mod shell;
mod preview;
mod widgets;

pub use preview::{
    app_switcher_ball_center, app_switcher_ball_size, point_in_polygon,
    surface_index_for_face, PreviewSoccerBall,
};
pub use shell::{
    AppSurface, BottomBar, CardStackRoot, GestureZone, HomeSurface,
    LauncherIcon, LauncherIconLabel, OverlayLayer, QuickSettingsPanel,
    SettingsAction, SettingsPanel, SettingsRow, SettingsRowLabel, SettingsRowValue,
    ShellRoot, StatusBar, SurfaceCardSubtitle, SurfaceCardTitle, SurfacePreviewCard,
    SurfaceStackRoot, SurfaceText, SystemNavAction, SystemNavButton, SystemNavButtonLabel,
    ThemeBackdrop,
    // Android-style notification components
    NotificationPanel, QuickControlTile, QuickControlType,
    BrightnessControl, NotificationCard, NotificationPriority,
    NotificationCardTitle, NotificationCardContent,
};
pub use widgets::{ButtonWidget, WidgetLayoutNode};
