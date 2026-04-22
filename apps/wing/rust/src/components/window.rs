use crate::{WindowId, WindowState};

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

/// Marker for a window's content root entity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowContentRoot;

impl fhre::Component for WindowContentRoot {
    fn type_name() -> &'static str {
        "WindowContentRoot"
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
