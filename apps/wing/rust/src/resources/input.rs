use crate::WidgetId;

/// Focus ownership tracked independently from the transitional `Wing` object.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusState {
    pub window_id: Option<crate::WindowId>,
    pub widget_id: Option<WidgetId>,
}

impl fhre::resources::Resource for FocusState {}

/// Selection ownership for list or text oriented widgets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectionState {
    pub window_id: Option<crate::WindowId>,
    pub widget_id: Option<WidgetId>,
}

impl fhre::resources::Resource for SelectionState {}

/// Text editing context for future ECS text input systems.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextInputState {
    pub active_window_id: Option<crate::WindowId>,
    pub active_widget_id: Option<WidgetId>,
}

impl fhre::resources::Resource for TextInputState {}
