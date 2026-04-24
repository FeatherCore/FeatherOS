use crate::types::SurfaceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShellOverlayMode {
    #[default]
    None,
    QuickSettings,
    AppSwitcher,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShellState {
    pub active_surface: Option<SurfaceId>,
    pub overlay_mode: ShellOverlayMode,
}

impl ShellState {
    pub const fn notifications_visible(&self) -> bool {
        matches!(self.overlay_mode, ShellOverlayMode::QuickSettings)
    }

    pub const fn quick_settings_open(&self) -> bool {
        matches!(self.overlay_mode, ShellOverlayMode::QuickSettings)
    }

    pub const fn overlay_visible(&self) -> bool {
        !matches!(self.overlay_mode, ShellOverlayMode::None)
    }

    pub const fn app_switcher_open(&self) -> bool {
        matches!(self.overlay_mode, ShellOverlayMode::AppSwitcher)
    }
}

impl fhre::resources::Resource for ShellState {}
