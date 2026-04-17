//! Platform abstraction module
//!
//! Provides platform-specific implementations for different targets.

// X11 Window for SIM platform (input and display)
#[cfg(feature = "sim")]
pub mod x11_window;

// Re-export X11 window for SIM platform
#[cfg(feature = "sim")]
pub use x11_window::*;
