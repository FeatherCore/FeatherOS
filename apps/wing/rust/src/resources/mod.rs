//! Resources for the Wing shell.

mod animation;
mod content;
mod gesture;
mod shell;
mod theme;

use fhre::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellMetrics {
    pub screen_size: Vec2,
}

impl ShellMetrics {
    pub const fn new(screen_size: Vec2) -> Self {
        Self { screen_size }
    }
}

impl Default for ShellMetrics {
    fn default() -> Self {
        Self::new(Vec2::ZERO)
    }
}

impl fhre::resources::Resource for ShellMetrics {}

pub use animation::{ShellOverlayAnimation, ThemeAnimation};
pub use content::{NotificationCategory, NotificationPriority, ShellContent, ShellNotificationEntry, ShellSurfaceEntry, SurfaceState};
pub use gesture::{GesturePhase, GestureState, SwipeDirection};
pub use shell::{ShellOverlayMode, ShellState};
pub use theme::{ThemeState, ThemeVariant};
