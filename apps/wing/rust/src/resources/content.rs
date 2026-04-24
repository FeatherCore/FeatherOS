use alloc::vec::Vec;

use crate::types::SurfaceId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Urgent,
}

impl Default for NotificationPriority {
    fn default() -> Self { Self::Normal }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationCategory {
    System,
    Message,
    Email,
    Social,
    Alarm,
    Reminder,
    Other,
}

impl Default for NotificationCategory {
    fn default() -> Self { Self::Other }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellNotificationEntry {
    pub id: u32,
    pub title: &'static str,
    pub summary: &'static str,
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
            priority: NotificationPriority::Normal,
            category: NotificationCategory::Other,
            timestamp: 0,
        }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellContent {
    pub surfaces: Vec<ShellSurfaceEntry>,
    pub notifications: Vec<ShellNotificationEntry>,
    pub max_surfaces: usize,
    pub max_notifications: usize,
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
            notifications: Vec::from([
                ShellNotificationEntry::new(1, "System Update", "Shell card stack ready")
                    .with_priority(NotificationPriority::High)
                    .with_category(NotificationCategory::System)
                    .with_timestamp(1000),
                ShellNotificationEntry::new(2, "Watch Shell", "Notifications stack online")
                    .with_priority(NotificationPriority::Normal)
                    .with_category(NotificationCategory::System)
                    .with_timestamp(2000),
                ShellNotificationEntry::new(3, "Weather Alert", "Rain expected in 2 hours")
                    .with_priority(NotificationPriority::Urgent)
                    .with_category(NotificationCategory::Alarm)
                    .with_timestamp(3000),
            ]),
            max_surfaces: 8,
            max_notifications: 16,
        }
    }
}

impl ShellContent {
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

    pub fn remove_surface(&mut self, surface_id: SurfaceId) -> bool {
        let index = self.surfaces.iter().position(|s| s.id == surface_id);
        if let Some(i) = index {
            self.surfaces.remove(i);
            true
        } else {
            false
        }
    }

    pub fn get_surface(&self, surface_id: SurfaceId) -> Option<&ShellSurfaceEntry> {
        self.surfaces.iter().find(|s| s.id == surface_id)
    }

    pub fn update_surface_state(&mut self, surface_id: SurfaceId, state: SurfaceState) -> bool {
        if let Some(surface) = self.surfaces.iter_mut().find(|s| s.id == surface_id) {
            surface.state = state;
            true
        } else {
            false
        }
    }

    pub fn add_notification(&mut self, notification: ShellNotificationEntry) -> bool {
        if self.notifications.len() >= self.max_notifications {
            self.notifications.pop();
        }
        if self.notifications.iter().any(|n| n.id == notification.id) {
            return false;
        }
        self.notifications.insert(0, notification);
        true
    }

    pub fn remove_notification(&mut self, notification_id: u32) -> bool {
        let index = self.notifications.iter().position(|n| n.id == notification_id);
        if let Some(i) = index {
            self.notifications.remove(i);
            true
        } else {
            false
        }
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    pub fn notification_count(&self) -> usize {
        self.notifications.len()
    }

    pub fn surface_count(&self) -> usize {
        self.surfaces.len()
    }
}

impl fhre::resources::Resource for ShellContent {}
