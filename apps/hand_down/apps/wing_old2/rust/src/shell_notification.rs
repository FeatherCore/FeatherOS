use crate::core::FixedList;

pub const NOTIFICATION_CAPACITY: usize = 8;

pub const NOTIFICATION_ID_MESSAGE: u16 = 1;
pub const NOTIFICATION_ID_MAIL: u16 = 2;
pub const NOTIFICATION_ID_WEATHER: u16 = 3;
pub const NOTIFICATION_ID_SYSTEM: u16 = 4;
pub const NOTIFICATION_ID_APP: u16 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationIcon {
    Chat,
    Mail,
    Cloud,
    Alert,
    Settings,
    Play,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationPriority {
    Normal,
    High,
}

impl NotificationPriority {
    pub const fn is_high(self) -> bool {
        matches!(self, Self::High)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotificationEntry {
    pub id: u16,
    pub icon: NotificationIcon,
    pub title: &'static str,
    pub body: &'static str,
    pub time: &'static str,
    pub priority: NotificationPriority,
}

impl NotificationEntry {
    pub const fn new(
        id: u16,
        icon: NotificationIcon,
        title: &'static str,
        body: &'static str,
        time: &'static str,
        priority: NotificationPriority,
    ) -> Self {
        Self {
            id,
            icon,
            title,
            body,
            time,
            priority,
        }
    }
}

pub struct NotificationStore {
    entries: FixedList<NotificationEntry, NOTIFICATION_CAPACITY>,
    revision: u32,
}

impl Default for NotificationStore {
    fn default() -> Self {
        Self {
            entries: FixedList::new(),
            revision: 0,
        }
    }
}

impl NotificationStore {
    pub fn with_shell_defaults() -> Self {
        let mut store = Self::default();
        store.upsert(NotificationEntry::new(
            NOTIFICATION_ID_MESSAGE,
            NotificationIcon::Chat,
            "MESSAGE",
            "AT 14:00",
            "NOW",
            NotificationPriority::High,
        ));
        store.upsert(NotificationEntry::new(
            NOTIFICATION_ID_MAIL,
            NotificationIcon::Mail,
            "MAIL",
            "BUILD PASSED",
            "2M",
            NotificationPriority::Normal,
        ));
        store.upsert(NotificationEntry::new(
            NOTIFICATION_ID_WEATHER,
            NotificationIcon::Cloud,
            "WEATHER",
            "CLEAR 24C",
            "8M",
            NotificationPriority::Normal,
        ));
        store.upsert(NotificationEntry::new(
            NOTIFICATION_ID_SYSTEM,
            NotificationIcon::Alert,
            "SYSTEM",
            "SURFACE READY",
            "12M",
            NotificationPriority::Normal,
        ));
        store
    }

    pub fn entries(&self) -> &[NotificationEntry] {
        self.entries.as_slice()
    }

    pub fn revision(&self) -> u32 {
        self.revision
    }

    pub fn upsert(&mut self, entry: NotificationEntry) -> bool {
        for existing in self.entries.as_mut_slice() {
            if existing.id == entry.id {
                if *existing != entry {
                    *existing = entry;
                    self.bump_revision();
                    return true;
                }
                return false;
            }
        }

        if self.entries.push(entry) {
            self.bump_revision();
            true
        } else {
            false
        }
    }

    pub fn notify_app_started(&mut self) -> bool {
        self.upsert(NotificationEntry::new(
            NOTIFICATION_ID_APP,
            NotificationIcon::Play,
            "APP",
            "TASK STARTED",
            "NOW",
            NotificationPriority::High,
        ))
    }

    pub fn notify_app_failed(&mut self, reason: &'static str) -> bool {
        self.upsert(NotificationEntry::new(
            NOTIFICATION_ID_APP,
            NotificationIcon::Alert,
            "APP",
            reason,
            "NOW",
            NotificationPriority::High,
        ))
    }

    pub fn notify_task_exit(&mut self) -> bool {
        self.upsert(NotificationEntry::new(
            NOTIFICATION_ID_APP,
            NotificationIcon::Settings,
            "APP",
            "TASK EXITED",
            "NOW",
            NotificationPriority::Normal,
        ))
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}
