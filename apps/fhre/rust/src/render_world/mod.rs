//! Render World Module
//!
//! The Render World contains GPU resources, draw commands, and framebuffer data.
//! It is separate from Main World to allow parallel processing.

// Sub-modules
mod world;
mod command;
mod object;

// Re-exports
pub use world::RenderWorld;
pub use command::{RenderCommand, DrawCall, PrimitiveType, Vertex};
pub use object::{RenderObject, ExtractedTransform, ExtractedSprite};
