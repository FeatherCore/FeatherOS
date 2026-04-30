//! Application configuration for FHRE

/// Application run mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunMode {
    /// Run once and exit
    Once,
    /// Run in a loop
    Loop,
}

impl Default for RunMode {
    fn default() -> Self {
        RunMode::Loop
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Run mode
    pub run_mode: RunMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: 640,
            height: 480,
            run_mode: RunMode::default(),
        }
    }
}
