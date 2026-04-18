//! Application module for FHRE
//!
//! Provides the main App struct and lifecycle management.
//! Inspired by Bevy's App and schedule runner.

mod app;
pub mod runner;

pub use app::{
    App, AppBuilder, FHRE_VERSION,
    AppRunner, AppExit, Startup, PreUpdate, Update, PostUpdate
};
pub use runner::{RunOnceRunner, FixedTimestepRunner};
