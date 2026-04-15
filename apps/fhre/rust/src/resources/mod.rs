//! Resource management for FHRE
//!
//! Resources are global data that can be accessed by systems.
//! Similar to Bevy's Resource system but simplified for embedded use.

mod resources;
mod time;
mod config;
mod screen;

pub use resources::{Resources, Resource, Res, ResMut};
pub use time::Time;
pub use config::{RenderConfig, WindowConfig};
pub use screen::{PrimaryScreen, ScreenCoordinate, ViewportConfig};
