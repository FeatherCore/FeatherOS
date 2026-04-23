//! Resources for Wing shell and legacy migration code.

mod shell;
mod theme;

use fhre::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DesktopMetrics {
    pub screen_size: Vec2,
    pub taskbar_height: f32,
    pub icon_margin: f32,
}

impl DesktopMetrics {
    pub const fn new(screen_size: Vec2, taskbar_height: f32, icon_margin: f32) -> Self {
        Self {
            screen_size,
            taskbar_height,
            icon_margin,
        }
    }
}

impl Default for DesktopMetrics {
    fn default() -> Self {
        Self::new(Vec2::ZERO, 48.0, 16.0)
    }
}

impl fhre::resources::Resource for DesktopMetrics {}

pub use shell::ShellState;
pub use theme::ThemeState;
