/// Transitional marker for the taskbar root entity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskbarRoot;

impl fhre::Component for TaskbarRoot {
    fn type_name() -> &'static str {
        "TaskbarRoot"
    }
}

/// Transitional taskbar item component for future ECS-managed window/app slots.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskbarItem {
    pub active: bool,
}

impl TaskbarItem {
    pub const fn active() -> Self {
        Self { active: true }
    }

    pub const fn inactive() -> Self {
        Self { active: false }
    }
}

impl fhre::Component for TaskbarItem {
    fn type_name() -> &'static str {
        "TaskbarItem"
    }
}
