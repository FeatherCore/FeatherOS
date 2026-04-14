//! NuttX platform support for FHRE
//!
//! Provides integration with NuttX RTOS framebuffer device.

/// NuttX framebuffer device path
pub const FB_DEVICE_PATH: &str = "/dev/fb0";

/// Create a render target for NuttX platform
/// Returns framebuffer dimensions
pub fn create_render_target(width: u32, height: u32) -> (u32, u32) {
    // For now, just return the requested dimensions
    // In a full implementation, this would query the actual framebuffer
    (width, height)
}
