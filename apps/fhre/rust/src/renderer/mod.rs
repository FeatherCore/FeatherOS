//! Renderer for FHRE
//!
//! Handles framebuffer output and display presentation.

mod renderer;
mod framebuffer;
mod target;

pub use renderer::Renderer;
pub use framebuffer::Framebuffer;
pub use target::RenderTarget;
