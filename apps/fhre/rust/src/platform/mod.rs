//! Platform abstraction module
//!
//! Provides platform-specific implementations for different targets.

// Framebuffer abstraction (common across all platforms)
pub mod framebuffer;

// Platform-specific modules
#[cfg(feature = "sim")]
pub mod sim;

#[cfg(feature = "nuttx")]
pub mod nuttx;

pub mod default;

// Re-export framebuffer types
pub use framebuffer::{Framebuffer, SimpleFramebuffer};

// Re-export platform types based on feature flags
#[cfg(feature = "sim")]
pub use sim::{SimDisplay, create_display, refresh_loop, FB_DEVICE_PATH};

// When no specific platform is selected, use default implementations
#[cfg(not(any(feature = "sim", feature = "nuttx")))]
pub use default::*;
