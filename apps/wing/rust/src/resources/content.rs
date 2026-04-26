use alloc::vec::Vec;

use crate::types::SurfaceId;

/// Surface state in the shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceState {
    Running,
    Paused,
    Background,
    Closed,
}

impl Default for SurfaceState {
    fn default() -> Self { Self::Background }
}

/// Entry for a surface (app) in the shell.
/// 
/// A surface represents a running application or screen in the shell.
/// The shell manages multiple surfaces via card-based navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellSurfaceEntry {
    pub id: SurfaceId,
    pub label: &'static str,
    pub preview_title: &'static str,
    pub state: SurfaceState,
    pub icon_hint: &'static str,
    pub last_active_time: u64,
}

impl ShellSurfaceEntry {
    pub const fn new(id: SurfaceId, label: &'static str, preview_title: &'static str) -> Self {
        Self {
            id,
            label,
            preview_title,
            state: SurfaceState::Background,
            icon_hint: "",
            last_active_time: 0,
        }
    }

    pub const fn with_state(self, state: SurfaceState) -> Self {
        Self { state, ..self }
    }

    pub const fn with_icon(self, icon_hint: &'static str) -> Self {
        Self { icon_hint, ..self }
    }

    pub const fn home() -> Self {
        Self::new(1, "Home", "Home Screen")
    }
}

/// Notification category for grouping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NotificationCategory {
    #[default]
    Other,
    System,
    Message,
    Email,
    Social,
    Alarm,
    Reminder,
}

/// Notification priority level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NotificationPriority {
    #[default]
    Normal,
    High,
    Low,
}

/// Entry for a notification in the shell.
/// 
/// Android-style notification data model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellNotificationEntry {
    pub id: u32,
    pub title: &'static str,
    pub summary: &'static str,
    pub app_name: &'static str,
    pub priority: NotificationPriority,
    pub category: NotificationCategory,
    pub timestamp: u64,
}

impl ShellNotificationEntry {
    pub const fn new(id: u32, title: &'static str, summary: &'static str) -> Self {
        Self {
            id,
            title,
            summary,
            app_name: "",
            priority: NotificationPriority::Normal,
            category: NotificationCategory::Other,
            timestamp: 0,
        }
    }
    
    pub const fn with_app(self, app_name: &'static str) -> Self {
        Self { app_name, ..self }
    }
    
    pub const fn with_priority(self, priority: NotificationPriority) -> Self {
        Self { priority, ..self }
    }
    
    pub const fn with_category(self, category: NotificationCategory) -> Self {
        Self { category, ..self }
    }
    
    pub const fn with_timestamp(self, timestamp: u64) -> Self {
        Self { timestamp, ..self }
    }
}

/// Quick control state for the notification panel.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct QuickControlState {
    pub wifi_enabled: bool,
    pub wifi_connected: bool,
    pub bluetooth_enabled: bool,
    pub bluetooth_connected: bool,
    pub airplane_mode: bool,
    pub flashlight_on: bool,
    pub dnd_mode: bool,
    pub auto_rotate: bool,
    pub battery_saver: bool,
    pub brightness: f32,  // 0.0 to 1.0
}

impl QuickControlState {
    pub const fn default_state() -> Self {
        Self {
            wifi_enabled: true,
            wifi_connected: true,
            bluetooth_enabled: false,
            bluetooth_connected: false,
            airplane_mode: false,
            flashlight_on: false,
            dnd_mode: false,
            auto_rotate: true,
            battery_saver: false,
            brightness: 0.5,
        }
    }
}

/// Shell content manager.
/// 
/// Manages the list of surfaces (apps) and notifications in the shell.
/// 
/// # Future
/// - Will support page/surface switching via swipe gestures
/// - Will manage surface lifecycle (launch/pause/close)
#[derive(Clone, Debug, PartialEq)]
pub struct ShellContent {
    /// List of surfaces currently in the shell
    pub surfaces: Vec<ShellSurfaceEntry>,
    /// Maximum number of surfaces supported
    pub max_surfaces: usize,
    /// List of notifications (Android-style notification panel)
    pub notifications: Vec<ShellNotificationEntry>,
    /// Maximum number of notifications
    pub max_notifications: usize,
    /// Quick control state
    pub quick_controls: QuickControlState,
}

impl Default for ShellContent {
    fn default() -> Self {
        Self {
            surfaces: Vec::from([
                ShellSurfaceEntry::home(),
                ShellSurfaceEntry::new(2, "Settings", "Settings App")
                    .with_state(SurfaceState::Running)
                    .with_icon("gear"),
                ShellSurfaceEntry::new(3, "Weather", "Weather App")
                    .with_state(SurfaceState::Background)
                    .with_icon("cloud"),
            ]),
            max_surfaces: 8,
            notifications: Vec::from([
                ShellNotificationEntry::new(1, "New Message", "Hello from Alice")
                    .with_app("Messages")
                    .with_priority(NotificationPriority::High)
                    .with_category(NotificationCategory::Message),
                ShellNotificationEntry::new(2, "Update Available", "System update ready")
                    .with_app("System")
                    .with_priority(NotificationPriority::Normal)
                    .with_category(NotificationCategory::System),
            ]),
            max_notifications: 16,
            quick_controls: QuickControlState::default_state(),
        }
    }
}

impl ShellContent {
    /// Add a surface to the shell.
    pub fn add_surface(&mut self, surface: ShellSurfaceEntry) -> bool {
        if self.surfaces.len() >= self.max_surfaces {
            return false;
        }
        if self.surfaces.iter().any(|s| s.id == surface.id) {
            return false;
        }
        self.surfaces.push(surface);
        true
    }

    /// Remove a surface from the shell.
    pub fn remove_surface(&mut self, surface_id: SurfaceId) -> bool {
        let index = self.surfaces.iter().position(|s| s.id == surface_id);
        if let Some(i) = index {
            self.surfaces.remove(i);
            true
        } else {
            false
        }
    }

    /// Get a surface by ID.
    pub fn get_surface(&self, surface_id: SurfaceId) -> Option<&ShellSurfaceEntry> {
        self.surfaces.iter().find(|s| s.id == surface_id)
    }

    /// Update surface state.
    pub fn update_surface_state(&mut self, surface_id: SurfaceId, state: SurfaceState) -> bool {
        if let Some(surface) = self.surfaces.iter_mut().find(|s| s.id == surface_id) {
            surface.state = state;
            true
        } else {
            false
        }
    }

    /// Get the number of surfaces.
    pub fn surface_count(&self) -> usize {
        self.surfaces.len()
    }

    // === Notification management ===

    /// Add a notification.
    pub fn add_notification(&mut self, notification: ShellNotificationEntry) -> bool {
        if self.notifications.len() >= self.max_notifications {
            return false;
        }
        if self.notifications.iter().any(|n| n.id == notification.id) {
            return false;
        }
        self.notifications.push(notification);
        true
    }

    /// Remove a notification.
    pub fn remove_notification(&mut self, notification_id: u32) -> bool {
        let index = self.notifications.iter().position(|n| n.id == notification_id);
        if let Some(i) = index {
            self.notifications.remove(i);
            true
        } else {
            false
        }
    }

    /// Get a notification by ID.
    pub fn get_notification(&self, notification_id: u32) -> Option<&ShellNotificationEntry> {
        self.notifications.iter().find(|n| n.id == notification_id)
    }

    /// Clear all notifications.
    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    /// Get the number of notifications.
    pub fn notification_count(&self) -> usize {
        self.notifications.len()
    }

    // === Quick control management ===

    /// Toggle WiFi.
    pub fn toggle_wifi(&mut self) {
        self.quick_controls.wifi_enabled = !self.quick_controls.wifi_enabled;
        if !self.quick_controls.wifi_enabled {
            self.quick_controls.wifi_connected = false;
        }
    }

    /// Toggle Bluetooth.
    pub fn toggle_bluetooth(&mut self) {
        self.quick_controls.bluetooth_enabled = !self.quick_controls.bluetooth_enabled;
        if !self.quick_controls.bluetooth_enabled {
            self.quick_controls.bluetooth_connected = false;
        }
    }

    /// Toggle Airplane mode.
    pub fn toggle_airplane_mode(&mut self) {
        self.quick_controls.airplane_mode = !self.quick_controls.airplane_mode;
        if self.quick_controls.airplane_mode {
            self.quick_controls.wifi_enabled = false;
            self.quick_controls.bluetooth_enabled = false;
        }
    }

    /// Toggle Flashlight.
    pub fn toggle_flashlight(&mut self) {
        self.quick_controls.flashlight_on = !self.quick_controls.flashlight_on;
    }

    /// Set brightness.
    pub fn set_brightness(&mut self, brightness: f32) {
        self.quick_controls.brightness = brightness.clamp(0.0, 1.0);
    }
}

impl fhre::resources::Resource for ShellContent {}
