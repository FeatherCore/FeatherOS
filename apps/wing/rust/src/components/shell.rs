//! Shell UI components.

use crate::types::SurfaceId;

// === Marker components (unit structs) ===

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShellRoot;

impl fhre::Component for ShellRoot {
    fn type_name() -> &'static str { "ShellRoot" }
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GestureZone;

impl fhre::Component for GestureZone {
    fn type_name() -> &'static str { "GestureZone" }
}

// === State components ===

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomeSurface {
    pub active: bool,
}

impl HomeSurface {
    pub const fn active() -> Self { Self { active: true } }
}

impl Default for HomeSurface {
    fn default() -> Self { Self { active: false } }
}

impl fhre::Component for HomeSurface {
    fn type_name() -> &'static str { "HomeSurface" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayLayer {
    pub visible: bool,
}

impl OverlayLayer {
    pub const fn hidden() -> Self { Self { visible: false } }
}

impl Default for OverlayLayer {
    fn default() -> Self { Self { visible: false } }
}

impl fhre::Component for OverlayLayer {
    fn type_name() -> &'static str { "OverlayLayer" }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuickSettingsPanel {
    pub open: bool,
}

impl QuickSettingsPanel {
    pub const fn closed() -> Self { Self { open: false } }
}

impl Default for QuickSettingsPanel {
    fn default() -> Self { Self { open: false } }
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

// === Card components ===

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

// === Text components ===

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

// === Android-style Notification Panel Components ===

/// Notification panel container (swipe down from status bar).
/// Contains QuickSettingsPanel at top and notification list below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NotificationPanel {
    pub open: bool,
    /// Scroll offset for notification list
    pub scroll_offset: f32,
}

impl NotificationPanel {
    pub const fn closed() -> Self { Self { open: false, scroll_offset: 0.0 } }
}

impl Default for NotificationPanel {
    fn default() -> Self { Self { open: false, scroll_offset: 0.0 } }
}

impl fhre::Component for NotificationPanel {
    fn type_name() -> &'static str { "NotificationPanel" }
}

/// Quick control tile (WiFi, Bluetooth, etc.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuickControlTile {
    pub tile_type: QuickControlType,
    pub enabled: bool,
    pub active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuickControlType {
    WiFi,
    Bluetooth,
    AirplaneMode,
    Flashlight,
    Dnd,
    AutoRotate,
    BatterySaver,
}

impl QuickControlTile {
    pub const fn new(tile_type: QuickControlType) -> Self {
        Self { tile_type, enabled: false, active: false }
    }
    
    pub const fn with_enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }
    
    pub const fn with_active(self, active: bool) -> Self {
        Self { active, ..self }
    }
    
    pub fn icon(&self) -> &'static str {
        match self.tile_type {
            QuickControlType::WiFi => if self.active { "wifi_on" } else { "wifi_off" },
            QuickControlType::Bluetooth => if self.active { "bt_on" } else { "bt_off" },
            QuickControlType::AirplaneMode => "airplane",
            QuickControlType::Flashlight => if self.active { "flash_on" } else { "flash_off" },
            QuickControlType::Dnd => "dnd",
            QuickControlType::AutoRotate => if self.active { "rotate_on" } else { "rotate_off" },
            QuickControlType::BatterySaver => "battery_saver",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self.tile_type {
            QuickControlType::WiFi => "WiFi",
            QuickControlType::Bluetooth => "Bluetooth",
            QuickControlType::AirplaneMode => "Airplane",
            QuickControlType::Flashlight => "Flashlight",
            QuickControlType::Dnd => "Do Not Disturb",
            QuickControlType::AutoRotate => "Auto-rotate",
            QuickControlType::BatterySaver => "Battery Saver",
        }
    }
}

impl fhre::Component for QuickControlTile {
    fn type_name() -> &'static str { "QuickControlTile" }
}

/// Brightness slider control.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrightnessControl {
    pub brightness: f32,  // 0.0 to 1.0
    pub visible: bool,
}

impl BrightnessControl {
    pub const fn hidden() -> Self { Self { brightness: 0.5, visible: false } }
}

impl Default for BrightnessControl {
    fn default() -> Self { Self { brightness: 0.5, visible: false } }
}

impl fhre::Component for BrightnessControl {
    fn type_name() -> &'static str { "BrightnessControl" }
}

/// Notification card for app notifications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationCard {
    pub notification_id: u32,
    pub visible: bool,
    pub stack_index: u8,
    pub priority: NotificationPriority,
    pub expanded: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NotificationPriority {
    #[default]
    Normal,
    High,
    Low,
}

impl NotificationCard {
    pub const fn hidden(notification_id: u32, stack_index: u8) -> Self {
        Self {
            notification_id,
            visible: false,
            stack_index,
            priority: NotificationPriority::Normal,
            expanded: false,
        }
    }
    
    pub const fn with_priority(self, priority: NotificationPriority) -> Self {
        Self { priority, ..self }
    }
}

impl fhre::Component for NotificationCard {
    fn type_name() -> &'static str { "NotificationCard" }
}

/// Notification card title text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationCardTitle {
    pub notification_id: u32,
}

impl NotificationCardTitle {
    pub const fn for_notification(notification_id: u32) -> Self {
        Self { notification_id }
    }
}

impl fhre::Component for NotificationCardTitle {
    fn type_name() -> &'static str { "NotificationCardTitle" }
}

/// Notification card content text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationCardContent {
    pub notification_id: u32,
}

impl NotificationCardContent {
    pub const fn for_notification(notification_id: u32) -> Self {
        Self { notification_id }
    }
}

impl fhre::Component for NotificationCardContent {
    fn type_name() -> &'static str { "NotificationCardContent" }
}
