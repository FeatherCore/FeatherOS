//! Render World Module
//!
//! The Render World is a complete ECS that mirrors Main World entities
//! and stores render-specific components.
//!
//! # Architecture (aligned with Bevy)
//!
//! ```text
//! Main World                    Render World
//! -----------                   ------------
//! Entity + Transform3D    →     Entity + MainEntity + ExtractedTransform
//! Entity + Cube           →     Entity + MainEntity + ExtractedMesh
//! Entity + SoccerBall     →     Entity + MainEntity + ExtractedMesh
//! ```
//!
//! # Render Pipeline
//!
//! 1. Extract Phase: Copy components from Main World to Render World
//! 2. Queue Phase: Generate PhaseItems into RenderPhases
//! 3. Sort Phase: Sort items within each phase
//! 4. Render Phase: Execute render commands in sorted order

mod world;
mod command;
mod object;
mod phase;
mod view;
mod extracted;

pub use world::RenderWorld;
pub use command::{RenderCommand, DrawCall, PrimitiveType, Vertex};
pub use object::RenderObject;
pub use phase::{
    RenderPhaseType,
    PhaseItem,
    RenderPhase,
    RenderPhases,
};
pub use view::{
    View,
    ViewTarget,
    ClearConfig,
    ViewBundle,
};
pub use extracted::{ExtractedTransform, ExtractedMesh, ExtractedUI, ExtractedView};

use alloc::vec::Vec;

/// Render Component Trait - Components that can generate render commands
pub trait RenderComponent {
    /// Generate render commands for this component
    fn generate_render_commands(&self, transform: &crate::node::Transform3D, view: &View) -> Vec<RenderCommand>;
}
