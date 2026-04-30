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
    pub overlay_origin_surface: Option<SurfaceId>,
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

    pub fn open_overlay(&mut self, mode: ShellOverlayMode) {
        if !self.overlay_visible() {
            self.overlay_origin_surface = self.active_surface;
        }
        self.overlay_mode = mode;
    }

    pub fn close_overlay_to_origin(&mut self) {
        self.overlay_mode = ShellOverlayMode::None;
        if let Some(surface_id) = self.overlay_origin_surface {
            self.active_surface = Some(surface_id);
        }
        self.overlay_origin_surface = None;
    }

    pub fn dismiss_overlay(&mut self) {
        self.overlay_mode = ShellOverlayMode::None;
        self.overlay_origin_surface = None;
    }
}

impl fhre::resources::Resource for ShellState {}
