use crate::types::SurfaceId;

/// Shell overlay modes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShellOverlayMode {
    #[default]
    None,
    /// Android-style notification panel (swipe down from status bar)
    /// Contains quick settings at top + notification list below
    NotificationPanel,
    /// App switcher showing surface preview cards
    AppSwitcher,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShellState {
    pub active_surface: Option<SurfaceId>,
    pub overlay_mode: ShellOverlayMode,
}

impl ShellState {
    /// Check if notification panel is visible
    pub const fn notification_panel_open(&self) -> bool {
        matches!(self.overlay_mode, ShellOverlayMode::NotificationPanel)
    }

    pub const fn overlay_visible(&self) -> bool {
        !matches!(self.overlay_mode, ShellOverlayMode::None)
    }

    pub const fn app_switcher_open(&self) -> bool {
        matches!(self.overlay_mode, ShellOverlayMode::AppSwitcher)
    }
}

impl fhre::resources::Resource for ShellState {}
