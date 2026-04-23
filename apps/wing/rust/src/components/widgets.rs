/// Widget layout component for shell layout systems.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WidgetLayoutNode {
    pub width: f32,
    pub height: f32,
}

impl fhre::Component for WidgetLayoutNode {
    fn type_name() -> &'static str {
        "WidgetLayoutNode"
    }
}

/// Minimal pointer-driven button state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonWidget {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

impl ButtonWidget {
    pub const fn pressed() -> Self {
        Self {
            hovered: false,
            pressed: true,
            clicked: false,
        }
    }

    pub const fn idle() -> Self {
        Self {
            hovered: false,
            pressed: false,
            clicked: false,
        }
    }

    pub const fn hovered() -> Self {
        Self {
            hovered: true,
            pressed: false,
            clicked: false,
        }
    }
}

impl fhre::Component for ButtonWidget {
    fn type_name() -> &'static str {
        "ButtonWidget"
    }
}
