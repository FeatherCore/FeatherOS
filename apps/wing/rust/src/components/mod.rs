//! Transitional ECS components for the Wing desktop plugin.

mod desktop;
mod launcher;
mod taskbar;
mod widgets;
mod window;

pub use desktop::{DesktopIconComponent, DesktopRoot, DesktopWallpaper};
pub use launcher::{LauncherEntry, LauncherPanel};
pub use taskbar::{TaskbarItem, TaskbarRoot};
pub use widgets::{ButtonWidget, IconGlyph, Label, ScrollAreaWidget, TextFieldWidget, WidgetLayoutNode, WidgetNodeComponent};
pub use window::{WindowChrome, WindowContentRoot, WindowFocus, WindowFrame};
