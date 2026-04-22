/// Transitional marker for the launcher panel entity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LauncherPanel;

impl fhre::Component for LauncherPanel {
    fn type_name() -> &'static str {
        "LauncherPanel"
    }
}

/// Transitional launcher entry component for future ECS-managed app items.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LauncherEntry {
    pub selected: bool,
}

impl LauncherEntry {
    pub const fn selected() -> Self {
        Self { selected: true }
    }

    pub const fn idle() -> Self {
        Self { selected: false }
    }
}

impl fhre::Component for LauncherEntry {
    fn type_name() -> &'static str {
        "LauncherEntry"
    }
}
