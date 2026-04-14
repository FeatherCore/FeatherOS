//! Resource management for FHRE
//!
//! Resources are global data that can be accessed by systems.
//! Similar to Bevy's Resource system but simplified for embedded use.

mod resources;
mod time;
mod config;

pub use resources::{Resources, Resource};
pub use time::Time;
pub use config::{RenderConfig, WindowConfig};
