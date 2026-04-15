//! Render World Implementation
//!
//! The Render World contains rendering data, draw commands, view configuration,
//! and manages the rendering backend execution.
//!
//! Architecture:
//! - Render Phases: Organizes draw commands into phases for efficient rendering
//! - View Management: Supports multiple views with different projections
//! - Command Queue: Stores render commands for execution
//! - Backend Management: Encapsulates the rendering backend (Software/GPU)
//!
//! Note: RenderWorld is the facade for all rendering operations.
//! The actual backend implementation is in the Pipeline module.

use super::command::RenderCommand;
use super::object::RenderObject;
use super::phase::{RenderPhases, RenderPhaseType, PhaseItem, PhaseBatch, BatchBuilder};
use super::view::ViewBundle;
use crate::pipeline::SoftwareBackend;
use crate::math::{Color, Rect};
use alloc::vec::Vec;

/// Render World - Container for rendering data and backend execution
///
/// The Render World is populated during the Extract phase
/// and executes rendering during the Render phase.
///
/// Architecture inspired by Bevy's render world:
/// - Multiple render phases (Background, Opaque2d, Opaque3d, AlphaMask, Transparent, UI)
/// - View management for multiple cameras/viewports
/// - Command queue for rendering
/// - Backend abstraction for Software/GPU rendering
pub struct RenderWorld {
    /// Width of the render target
    width: u32,
    /// Height of the render target
    height: u32,
    /// Render objects to draw
    objects: Vec<RenderObject>,
    /// Render commands queue
    commands: Vec<RenderCommand>,
    /// Clear color
    clear_color: Color,
    /// Viewport rectangle
    viewport: Rect,
    /// Render phases - organize draw commands into phases
    phases: RenderPhases,
    /// Active views (cameras)
    views: Vec<ViewBundle>,
    /// Current view index
    current_view: Option<usize>,
    /// Whether to use phase-based rendering
    use_phases: bool,
    /// Rendering backend (Software for now, GPU in future)
    backend: SoftwareBackend,
}

impl RenderWorld {
    /// Create a new Render World with specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            objects: Vec::new(),
            commands: Vec::new(),
            clear_color: Color::BLACK,
            viewport: Rect::new(0.0, 0.0, width as f32, height as f32),
            phases: RenderPhases::new(),
            views: Vec::new(),
            current_view: None,
            use_phases: true,
            backend: SoftwareBackend::new(width, height),
        }
    }

    /// Get framebuffer width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get framebuffer height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get framebuffer reference
    pub fn framebuffer(&self) -> &[u32] {
        self.backend.framebuffer()
    }

    /// Get framebuffer reference (alias for framebuffer)
    pub fn get_framebuffer(&self) -> &[u32] {
        self.backend.framebuffer()
    }

    /// Set clear color
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    /// Get clear color
    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    /// Set viewport
    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.viewport = Rect::new(x, y, width, height);
    }

    /// Get viewport
    pub fn viewport(&self) -> Rect {
        self.viewport
    }

    // =========================================================================
    // Render Objects
    // =========================================================================

    /// Add a render object
    pub fn add_object(&mut self, object: RenderObject) {
        self.objects.push(object);
    }

    /// Get all render objects
    pub fn objects(&self) -> &[RenderObject] {
        &self.objects
    }

    /// Get mutable render objects
    pub fn objects_mut(&mut self) -> &mut Vec<RenderObject> {
        &mut self.objects
    }

    /// Clear all render objects
    pub fn clear_objects(&mut self) {
        self.objects.clear();
    }

    // =========================================================================
    // Render Commands
    // =========================================================================

    /// Add a render command
    pub fn add_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }

    /// Get all render commands
    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }

    /// Get mutable render commands
    pub fn commands_mut(&mut self) -> &mut Vec<RenderCommand> {
        &mut self.commands
    }

    /// Clear all render commands
    pub fn clear_commands(&mut self) {
        self.commands.clear();
    }

    // =========================================================================
    // Render Phases (Bevy-inspired)
    // =========================================================================

    /// Get render phases
    pub fn phases(&self) -> &RenderPhases {
        &self.phases
    }

    /// Get mutable render phases
    pub fn phases_mut(&mut self) -> &mut RenderPhases {
        &mut self.phases
    }

    /// Add a phase item to a specific render phase
    pub fn add_phase_item(&mut self, phase_type: RenderPhaseType, item: PhaseItem) {
        self.phases.add_item(phase_type, item);
    }

    /// Sort all render phases
    pub fn sort_phases(&mut self) {
        self.phases.sort_all();
    }

    /// Clear all render phases
    pub fn clear_phases(&mut self) {
        self.phases.clear_all();
    }

    /// Build batches for a specific phase
    pub fn build_batches(&self, phase_type: RenderPhaseType) -> Vec<PhaseBatch> {
        let phase = self.phases.get(phase_type);
        BatchBuilder::build_batches(phase)
    }

    /// Enable/disable phase-based rendering
    pub fn set_use_phases(&mut self, use_phases: bool) {
        self.use_phases = use_phases;
    }

    /// Check if phase-based rendering is enabled
    pub fn use_phases(&self) -> bool {
        self.use_phases
    }

    // =========================================================================
    // View Management (Bevy-inspired)
    // =========================================================================

    /// Add a view
    pub fn add_view(&mut self, view: ViewBundle) -> usize {
        let index = self.views.len();
        self.views.push(view);
        index
    }

    /// Get a view by index
    pub fn get_view(&self, index: usize) -> Option<&ViewBundle> {
        self.views.get(index)
    }

    /// Get mutable view by index
    pub fn get_view_mut(&mut self, index: usize) -> Option<&mut ViewBundle> {
        self.views.get_mut(index)
    }

    /// Get all views
    pub fn views(&self) -> &[ViewBundle] {
        &self.views
    }

    /// Get mutable views
    pub fn views_mut(&mut self) -> &mut Vec<ViewBundle> {
        &mut self.views
    }

    /// Remove a view
    pub fn remove_view(&mut self, index: usize) -> Option<ViewBundle> {
        if index < self.views.len() {
            Some(self.views.remove(index))
        } else {
            None
        }
    }

    /// Clear all views
    pub fn clear_views(&mut self) {
        self.views.clear();
        self.current_view = None;
    }

    /// Set current view
    pub fn set_current_view(&mut self, index: Option<usize>) {
        self.current_view = index;
    }

    /// Get current view index
    pub fn current_view_index(&self) -> Option<usize> {
        self.current_view
    }

    /// Get current view
    pub fn current_view(&self) -> Option<&ViewBundle> {
        self.current_view.and_then(|idx| self.views.get(idx))
    }

    /// Get mutable current view
    pub fn current_view_mut(&mut self) -> Option<&mut ViewBundle> {
        self.current_view.and_then(|idx| self.views.get_mut(idx))
    }

    /// Create default screen view
    pub fn create_default_view(&mut self) -> usize {
        let view = ViewBundle::new_screen(self.width as f32, self.height as f32)
            .with_clear_color(self.clear_color);
        self.add_view(view)
    }

    // =========================================================================
    // Rendering Execution
    // =========================================================================

    /// Execute all pending render commands
    ///
    /// This is the main entry point for rendering execution.
    /// It submits all queued commands to the rendering backend.
    pub fn execute_render(&mut self) {
        // Debug: print number of commands
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[RENDER] Executing %d commands, %d views\n\0".as_ptr(), 
                self.commands.len() as i32,
                self.views.len() as i32,
            );
        }
        
        // Execute all commands through the backend
        self.backend.execute_commands(&self.commands);
        
        // Debug: check framebuffer after render
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            let fb = self.backend.framebuffer();
            let mut non_zero = 0;
            // Check the entire framebuffer
            for i in 0..fb.len() {
                if fb[i] != 0 {
                    non_zero += 1;
                }
            }
            printf(b"[RENDER] Framebuffer non-zero pixels: %d/%d\n\0".as_ptr(),
                non_zero,
                fb.len() as i32,
            );
            // Sample a few pixels from the expected book area (around 125,150 to 275,350)
            let sample_y = 200;
            let sample_x = 200;
            let idx = (sample_y * self.width + sample_x) as usize;
            if idx < fb.len() {
                printf(b"[RENDER] Sample pixel at (200,200): 0x%x\n\0".as_ptr(), fb[idx]);
            }
        }
    }

    /// Execute render commands for a specific phase
    pub fn execute_phase(&mut self, phase_type: RenderPhaseType) {
        // Build batches and execute for this phase
        let batches = self.build_batches(phase_type);
        
        for batch in batches {
            self.execute_batch(batch);
        }
    }

    /// Execute a batch of draw commands
    fn execute_batch(&mut self, batch: PhaseBatch) {
        let phase = self.phases.get(batch.phase_type);
        
        for i in batch.start_index..batch.start_index + batch.count {
            if let Some(item) = phase.items.get(i) {
                if let Some(command) = self.commands.get(item.draw_command_index) {
                    self.backend.execute_command(command);
                }
            }
        }
    }

    /// Render all phases for all views
    ///
    /// This is the complete rendering pipeline.
    /// It follows Bevy's approach of rendering each view's phases in order.
    pub fn render_all(&mut self) {
        // If no views, create default view
        if self.views.is_empty() {
            self.create_default_view();
            self.current_view = Some(0);
        }

        // Clear the backend framebuffer
        self.backend.reset();

        // Render each view
        for view_idx in 0..self.views.len() {
            self.render_view(view_idx);
        }
    }

    /// Render a specific view
    fn render_view(&mut self, view_idx: usize) {
        // Get view configuration
        let viewport = match self.views.get(view_idx) {
            Some(v) => v.view.viewport,
            None => return,
        };

        // Set viewport on backend
        self.backend.set_viewport(viewport);

        // Render all phases in order
        use RenderPhaseType::*;
        let phase_order = [Background, Opaque2d, Opaque3d, AlphaMask, Transparent, Ui];

        for phase_type in phase_order.iter() {
            self.execute_phase(*phase_type);
        }
    }

    // =========================================================================
    // Frame Management
    // =========================================================================

    /// Reset the render world for a new frame
    ///
    /// This clears all data and resets the backend for the next frame.
    pub fn reset(&mut self) {
        self.clear_objects();
        self.clear_commands();
        self.clear_phases();
        self.clear_views();  // Also clear views to prevent accumulation
        self.backend.reset();
    }

    /// Full reset including views
    pub fn full_reset(&mut self) {
        self.reset();
        self.clear_views();
    }

    /// Resize the render world dimensions
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.viewport = Rect::new(0.0, 0.0, width as f32, height as f32);
        self.backend.resize(width, height);
    }
}

impl Default for RenderWorld {
    fn default() -> Self {
        Self::new(800, 600)
    }
}
