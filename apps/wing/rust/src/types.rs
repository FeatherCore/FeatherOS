//! Minimal shared types for the ECS-first Wing desktop.

/// Stable window identifier for ECS-managed desktop windows.
pub type WindowId = u32;

/// Stable widget identifier for ECS-managed desktop controls.
pub type WidgetId = u32;

/// Stable surface identifier for mobile/watch shell surfaces.
pub type SurfaceId = u32;

/// Minimal window lifecycle state used by the empty desktop shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Closed,
}
