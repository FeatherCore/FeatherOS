/// Transitional marker for the desktop root entity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DesktopRoot;

impl fhre::Component for DesktopRoot {
    fn type_name() -> &'static str {
        "DesktopRoot"
    }
}

/// Transitional wallpaper component for future ECS-managed desktop presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DesktopWallpaper {
    pub visible: bool,
}

impl DesktopWallpaper {
    pub const fn visible() -> Self {
        Self { visible: true }
    }
}

impl fhre::Component for DesktopWallpaper {
    fn type_name() -> &'static str {
        "DesktopWallpaper"
    }
}

/// Desktop icon semantic component for future ECS-managed icon entities.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DesktopIconComponent {
    pub selected: bool,
}

impl DesktopIconComponent {
    pub const fn selected() -> Self {
        Self { selected: true }
    }

    pub const fn idle() -> Self {
        Self { selected: false }
    }
}

impl fhre::Component for DesktopIconComponent {
    fn type_name() -> &'static str {
        "DesktopIcon"
    }
}
