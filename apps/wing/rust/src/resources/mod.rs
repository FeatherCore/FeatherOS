//! Transitional resources for the Wing desktop plugin.

mod desktop;
mod input;
mod runtime;
mod theme;
mod window;

pub use desktop::{DesktopMetrics, LayoutInvalidation, LauncherState, TaskbarState, WingDesktopState};
pub use input::{FocusState, SelectionState, TextInputState};
pub use runtime::WingRuntime;
pub use theme::ThemeState;
pub use window::{DragTransaction, WindowManagerState};
