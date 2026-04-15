//! Render World Module
//!
//! The Render World contains GPU resources, draw commands, and framebuffer data.
//! It is separate from Main World to allow parallel processing.
//!
//! Inspired by Bevy's render world architecture, but simplified for embedded systems.
//! Key features:
//! - Render Phases: Organizes draw commands into phases (Background, Opaque2d, Opaque3d, AlphaMask, Transparent, UI)
//! - View Management: Supports multiple views (cameras) with different projections
//! - Batch Processing: Groups similar draw commands for efficient rendering

// Sub-modules
mod world;
mod command;
mod object;
mod phase;
mod view;

// Re-exports
pub use world::RenderWorld;
pub use command::{RenderCommand, DrawCall, PrimitiveType, Vertex};
pub use object::{RenderObject, ExtractedTransform, ExtractedSprite};
pub use phase::{
    RenderPhaseType,
    PhaseItem,
    RenderPhase,
    RenderPhases,
    PhaseBatch,
    BatchBuilder,
};
pub use view::{
    View,
    ViewTarget,
    ClearConfig,
    ViewBundle,
};
