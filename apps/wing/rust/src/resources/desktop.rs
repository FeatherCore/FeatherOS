use fhre::Vec2;

/// Shared desktop shell state while Wing migrates behavior into ECS systems.
#[derive(Clone, Debug, Default)]
pub struct WingDesktopState {
    pub initialized: bool,
}

impl fhre::resources::Resource for WingDesktopState {}

/// Launcher panel visibility state separated from the central runtime object.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LauncherState {
    pub open: bool,
}

impl fhre::resources::Resource for LauncherState {}

/// Taskbar shell state for future ECS-managed synchronization.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskbarState {
    pub launcher_open: bool,
}

impl fhre::resources::Resource for TaskbarState {}

/// Layout invalidation flag shared by future layout systems.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutInvalidation {
    pub pending: bool,
}

impl fhre::resources::Resource for LayoutInvalidation {}

/// Desktop layout metrics for screen-relative shell placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopMetrics {
    pub screen_size: Vec2,
    pub taskbar_height: f32,
    pub icon_margin: f32,
}

impl DesktopMetrics {
    pub const fn new(screen_size: Vec2, taskbar_height: f32, icon_margin: f32) -> Self {
        Self {
            screen_size,
            taskbar_height,
            icon_margin,
        }
    }
}

impl Default for DesktopMetrics {
    fn default() -> Self {
        Self::new(Vec2::ZERO, 48.0, 16.0)
    }
}

impl fhre::resources::Resource for DesktopMetrics {}
