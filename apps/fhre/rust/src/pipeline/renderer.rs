//! Renderer Trait
//!
//! Abstract interface for different rendering backends.
//! Allows switching between Software (CPU), GPU, or hybrid rendering.

use crate::render_world::{RenderCommand, RenderPhaseType};
use crate::math::Rect;

/// Renderer trait - implemented by all rendering backends
pub trait Renderer {
    /// Initialize the renderer with framebuffer dimensions
    fn new(width: u32, height: u32) -> Self where Self: Sized;
    
    /// Execute a batch of render commands
    fn execute_commands(&mut self, commands: &[RenderCommand]);
    
    /// Execute commands for a specific phase
    fn execute_phase(&mut self, phase: RenderPhaseType, commands: &[RenderCommand]);
    
    /// Get the rendered framebuffer
    fn framebuffer(&self) -> &[u32];
    
    /// Get mutable framebuffer reference
    fn framebuffer_mut(&mut self) -> &mut [u32];
    
    /// Resize the framebuffer
    fn resize(&mut self, width: u32, height: u32);
    
    /// Set viewport
    fn set_viewport(&mut self, rect: Rect);
    
    /// Reset/clear the renderer state
    fn reset(&mut self);
}

/// Renderer type selection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererType {
    /// Pure software CPU rendering
    Software,
    /// GPU accelerated rendering (if available)
    Gpu,
    /// Hybrid: GPU for batches, CPU for small tasks
    Hybrid,
}
