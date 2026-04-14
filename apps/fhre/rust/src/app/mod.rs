//! Application module for FHRE
//!
//! Provides the main App struct and lifecycle management.
//! Inspired by Bevy's App and schedule runner.

mod app;
mod config;

pub use app::{App, AppBuilder, FHRE_VERSION};
pub use config::{AppConfig, RunMode};
