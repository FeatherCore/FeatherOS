use crate::types::{WindowId, WindowState};

/// Transitional window component carrying stable window identity and state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowFrame {
    pub id: WindowId,
    pub state: WindowState,
}

impl WindowFrame {
    pub const fn new(id: WindowId, state: WindowState) -> Self {
        Self { id, state }
    }

    pub const fn is_visible(&self) -> bool {
        !matches!(self.state, WindowState::Minimized | WindowState::Closed)
    }
}

impl fhre::Component for WindowFrame {
    fn type_name() -> &'static str {
        "WindowFrame"
    }
}

/// Transitional chrome metadata for title bars and window controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowChrome {
    pub draggable: bool,
}

impl WindowChrome {
    pub const fn draggable() -> Self {
        Self { draggable: true }
    }
}

impl fhre::Component for WindowChrome {
    fn type_name() -> &'static str {
        "WindowChrome"
    }
}

/// Title bar entity linked to a window frame in the 3D desktop scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowTitleBar {
    pub window_id: WindowId,
}

impl WindowTitleBar {
    pub const fn new(window_id: WindowId) -> Self {
        Self { window_id }
    }
}

impl fhre::Component for WindowTitleBar {
    fn type_name() -> &'static str {
        "WindowTitleBar"
    }
}

/// Title text entity linked to a window frame in the 3D desktop scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowTitleText {
    pub window_id: WindowId,
    pub text: &'static str,
}

impl WindowTitleText {
    pub const fn new(window_id: WindowId, text: &'static str) -> Self {
        Self { window_id, text }
    }
}

impl fhre::Component for WindowTitleText {
    fn type_name() -> &'static str {
        "WindowTitleText"
    }
}

/// Title icon entity linked to a window frame in the 3D desktop scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowTitleIcon {
    pub window_id: WindowId,
}

impl WindowTitleIcon {
    pub const fn new(window_id: WindowId) -> Self {
        Self { window_id }
    }
}

impl fhre::Component for WindowTitleIcon {
    fn type_name() -> &'static str {
        "WindowTitleIcon"
    }
}

/// Content placeholder text entity linked to a window frame in the 3D desktop scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowContentText {
    pub window_id: WindowId,
    pub text: &'static str,
}

impl WindowContentText {
    pub const fn new(window_id: WindowId, text: &'static str) -> Self {
        Self { window_id, text }
    }
}

impl fhre::Component for WindowContentText {
    fn type_name() -> &'static str {
        "WindowContentText"
    }
}

/// Content root entity linked to a window frame in the 3D desktop scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowContentRoot {
    pub window_id: WindowId,
}

impl WindowContentRoot {
    pub const fn new(window_id: WindowId) -> Self {
        Self { window_id }
    }
}

impl fhre::Component for WindowContentRoot {
    fn type_name() -> &'static str {
        "WindowContentRoot"
    }
}

/// Which desktop window control button an entity represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowControlKind {
    Minimize,
    Maximize,
    Close,
}

/// Control button entity linked to a window frame in the 3D desktop scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowControlButton {
    pub window_id: WindowId,
    pub kind: WindowControlKind,
}

impl WindowControlButton {
    pub const fn new(window_id: WindowId, kind: WindowControlKind) -> Self {
        Self { window_id, kind }
    }
}

impl fhre::Component for WindowControlButton {
    fn type_name() -> &'static str {
        "WindowControlButton"
    }
}

/// Restorable placement for a desktop window in the 3D scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowPlacement {
    pub normal_center_x: f32,
    pub normal_center_y: f32,
    pub normal_width: f32,
    pub normal_height: f32,
}

impl WindowPlacement {
    pub const fn new(center_x: f32, center_y: f32, width: f32, height: f32) -> Self {
        Self {
            normal_center_x: center_x,
            normal_center_y: center_y,
            normal_width: width,
            normal_height: height,
        }
    }
}

impl fhre::Component for WindowPlacement {
    fn type_name() -> &'static str {
        "WindowPlacement"
    }
}

/// Transitional focus component for future ECS-managed window activation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowFocus {
    pub focused: bool,
}

impl WindowFocus {
    pub const fn focused() -> Self {
        Self { focused: true }
    }

    pub const fn blurred() -> Self {
        Self { focused: false }
    }
}

impl fhre::Component for WindowFocus {
    fn type_name() -> &'static str {
        "WindowFocus"
    }
}
