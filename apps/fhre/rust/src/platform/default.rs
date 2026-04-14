//! Default platform implementation for FHRE
//!
//! Provides fallback implementations when no specific platform is selected.

/// Create a default render target
/// Returns framebuffer dimensions
pub fn create_render_target(width: u32, height: u32) -> (u32, u32) {
    (width, height)
}
