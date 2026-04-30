//! Resource management for FHRE
//!
//! Resources are global data that can be accessed by systems.
//! Similar to Bevy's Resource system but simplified for embedded use.
//!
//! Note: Res and ResMut are defined in main_world::system_param to avoid
//! circular dependencies, and are re-exported here for convenience.

mod resources;
mod time;
mod config;
mod screen;
mod camera;

pub use resources::{Resources, Resource};
pub use time::Time;
pub use config::{RenderConfig, WindowConfig};
pub use screen::{PrimaryScreen, ScreenCoordinate, ViewportConfig};
pub use camera::{Camera, ProjectionType};

// Re-export Res/ResMut from main_world::system_param
pub use crate::main_world::{Res, ResMut};
