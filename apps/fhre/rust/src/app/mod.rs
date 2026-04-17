//! Application module for FHRE
//!
//! Provides the main App struct and lifecycle management.
//! Inspired by Bevy's App and schedule runner.

mod app;

pub use app::{
    App, AppBuilder, FHRE_VERSION, DefaultUiCamera, DefaultGameCamera,
    AppRunner, AppExit, Startup, PreUpdate, Update, PostUpdate
};
