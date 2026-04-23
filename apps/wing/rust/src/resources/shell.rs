use crate::types::SurfaceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShellState {
    pub active_surface: Option<SurfaceId>,
    pub quick_settings_open: bool,
    pub overlay_visible: bool,
    pub notifications_visible: bool,
    pub app_switcher_open: bool,
}

impl fhre::resources::Resource for ShellState {}
