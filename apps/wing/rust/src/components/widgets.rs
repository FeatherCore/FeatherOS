/// Transitional marker for ECS-managed widget entities.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WidgetNodeComponent;

impl fhre::Component for WidgetNodeComponent {
    fn type_name() -> &'static str {
        "WidgetNodeComponent"
    }
}

/// Widget layout component for future layout systems.
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

/// Transitional button component for future ECS-managed button state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonWidget {
    pub pressed: bool,
}

impl ButtonWidget {
    pub const fn pressed() -> Self {
        Self { pressed: true }
    }

    pub const fn idle() -> Self {
        Self { pressed: false }
    }
}

impl fhre::Component for ButtonWidget {
    fn type_name() -> &'static str {
        "ButtonWidget"
    }
}

/// Transitional text field component for future ECS-managed edit state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextFieldWidget {
    pub focused: bool,
}

impl TextFieldWidget {
    pub const fn focused() -> Self {
        Self { focused: true }
    }

    pub const fn blurred() -> Self {
        Self { focused: false }
    }
}

impl fhre::Component for TextFieldWidget {
    fn type_name() -> &'static str {
        "TextFieldWidget"
    }
}

/// Transitional scroll-area component for future ECS-managed scrolling state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollAreaWidget {
    pub scrollable: bool,
}

impl ScrollAreaWidget {
    pub const fn enabled() -> Self {
        Self { scrollable: true }
    }

    pub const fn disabled() -> Self {
        Self { scrollable: false }
    }
}

impl fhre::Component for ScrollAreaWidget {
    fn type_name() -> &'static str {
        "ScrollAreaWidget"
    }
}

/// Read-only text presentation component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Label;

impl fhre::Component for Label {
    fn type_name() -> &'static str {
        "Label"
    }
}

/// Icon presentation component for future extracted glyph rendering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IconGlyph;

impl fhre::Component for IconGlyph {
    fn type_name() -> &'static str {
        "IconGlyph"
    }
}
