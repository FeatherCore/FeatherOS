use crate::types::SurfaceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShellRoot;

impl fhre::Component for ShellRoot {
    fn type_name() -> &'static str { "ShellRoot" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomeSurface {
    pub active: bool,
}

impl HomeSurface {
    pub const fn active() -> Self { Self { active: true } }
}

impl fhre::Component for HomeSurface {
    fn type_name() -> &'static str { "HomeSurface" }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StatusBar;

impl fhre::Component for StatusBar {
    fn type_name() -> &'static str { "StatusBar" }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BottomBar;

impl fhre::Component for BottomBar {
    fn type_name() -> &'static str { "BottomBar" }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceStackRoot;

impl fhre::Component for SurfaceStackRoot {
    fn type_name() -> &'static str { "SurfaceStackRoot" }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CardStackRoot;

impl fhre::Component for CardStackRoot {
    fn type_name() -> &'static str { "CardStackRoot" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayLayer {
    pub visible: bool,
}

impl OverlayLayer {
    pub const fn hidden() -> Self { Self { visible: false } }
}

impl fhre::Component for OverlayLayer {
    fn type_name() -> &'static str { "OverlayLayer" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationLayer {
    pub visible: bool,
}

impl NotificationLayer {
    pub const fn hidden() -> Self { Self { visible: false } }
}

impl fhre::Component for NotificationLayer {
    fn type_name() -> &'static str { "NotificationLayer" }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NotificationStackRoot;

impl fhre::Component for NotificationStackRoot {
    fn type_name() -> &'static str { "NotificationStackRoot" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationCard {
    pub notification_id: u32,
    pub visible: bool,
    pub stack_index: u8,
    pub priority: crate::resources::NotificationPriority,
    pub category: crate::resources::NotificationCategory,
}

impl NotificationCard {
    pub const fn hidden(notification_id: u32, stack_index: u8) -> Self {
        Self {
            notification_id,
            visible: false,
            stack_index,
            priority: crate::resources::NotificationPriority::Normal,
            category: crate::resources::NotificationCategory::Other,
        }
    }

    pub const fn with_priority(self, priority: crate::resources::NotificationPriority) -> Self {
        Self { priority, ..self }
    }

    pub const fn with_category(self, category: crate::resources::NotificationCategory) -> Self {
        Self { category, ..self }
    }
}

impl fhre::Component for NotificationCard {
    fn type_name() -> &'static str { "NotificationCard" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationTextRole {
    Title,
    Summary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationText {
    pub stack_index: u8,
    pub role: NotificationTextRole,
    pub text: &'static str,
}

impl NotificationText {
    pub const fn title(stack_index: u8, text: &'static str) -> Self {
        Self { stack_index, role: NotificationTextRole::Title, text }
    }

    pub const fn summary(stack_index: u8, text: &'static str) -> Self {
        Self { stack_index, role: NotificationTextRole::Summary, text }
    }
}

impl fhre::Component for NotificationText {
    fn type_name() -> &'static str { "NotificationText" }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GestureZone;

impl fhre::Component for GestureZone {
    fn type_name() -> &'static str { "GestureZone" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuickSettingsPanel {
    pub open: bool,
}

impl QuickSettingsPanel {
    pub const fn closed() -> Self { Self { open: false } }
}

impl fhre::Component for QuickSettingsPanel {
    fn type_name() -> &'static str { "QuickSettingsPanel" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppSurface {
    pub id: SurfaceId,
    pub active: bool,
}

impl AppSurface {
    pub const fn new(id: SurfaceId) -> Self {
        Self { id, active: false }
    }
}

impl fhre::Component for AppSurface {
    fn type_name() -> &'static str { "AppSurface" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfacePreviewCard {
    pub surface_id: SurfaceId,
    pub visible: bool,
    pub stack_index: u8,
    pub state: crate::resources::SurfaceState,
    pub icon_hint: &'static str,
}

impl SurfacePreviewCard {
    pub const fn hidden(surface_id: SurfaceId, stack_index: u8) -> Self {
        Self {
            surface_id,
            visible: false,
            stack_index,
            state: crate::resources::SurfaceState::Background,
            icon_hint: "",
        }
    }

    pub const fn with_state(self, state: crate::resources::SurfaceState) -> Self {
        Self { state, ..self }
    }

    pub const fn with_icon(self, icon_hint: &'static str) -> Self {
        Self { icon_hint, ..self }
    }
}

impl fhre::Component for SurfacePreviewCard {
    fn type_name() -> &'static str { "SurfacePreviewCard" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceCardTitle {
    pub surface_id: SurfaceId,
    pub stack_index: u8,
}

impl SurfaceCardTitle {
    pub const fn for_card(surface_id: SurfaceId, stack_index: u8) -> Self {
        Self { surface_id, stack_index }
    }
}

impl fhre::Component for SurfaceCardTitle {
    fn type_name() -> &'static str { "SurfaceCardTitle" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceCardSubtitle {
    pub surface_id: SurfaceId,
    pub stack_index: u8,
}

impl SurfaceCardSubtitle {
    pub const fn for_card(surface_id: SurfaceId, stack_index: u8) -> Self {
        Self { surface_id, stack_index }
    }
}

impl fhre::Component for SurfaceCardSubtitle {
    fn type_name() -> &'static str { "SurfaceCardSubtitle" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceText {
    pub surface_id: Option<SurfaceId>,
    pub text: &'static str,
}

impl SurfaceText {
    pub const fn shell(text: &'static str) -> Self {
        Self { surface_id: None, text }
    }

    pub const fn for_surface(surface_id: SurfaceId, text: &'static str) -> Self {
        Self { surface_id: Some(surface_id), text }
    }
}

impl fhre::Component for SurfaceText {
    fn type_name() -> &'static str { "SurfaceText" }
}
