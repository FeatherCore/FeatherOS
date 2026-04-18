//! Render World Implementation - ECS Version
//!
//! The Render World now contains a full ECS structure to align with Bevy's architecture.
//! It stores extracted components and maintains entity mapping with Main World.
//!
//! Architecture:
//! - ECS Core: Entities, components storage (NEW - aligned with Bevy)
//! - Entity Mapping: MainEntity component links to Main World entities
//! - Render Phases: Organizes draw commands into phases for efficient rendering
//! - View Management: Supports multiple views with different projections
//! - Command Queue: Stores render commands for execution
//! - Backend Management: Encapsulates the rendering backend (Software/GPU)

use super::command::RenderCommand;
use super::object::RenderObject;
use super::phase::{RenderPhases, RenderPhaseType, PhaseItem, PhaseBatch, BatchBuilder};
use super::view::ViewBundle;
use crate::{Entity, Component};
use crate::sync::sync_markers::MainEntity;
use crate::pipeline::SoftwareBackend;
use crate::math::{Color, Rect};
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::any::{TypeId, Any};

// Debug output control - disabled by default
// Enable with --features debug
#[cfg(feature = "debug")]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf($($arg)*);
        }
    };
}

#[cfg(not(feature = "debug"))]
macro_rules! debug_print {
    ($($arg:tt)*) => {};
}

/// Render World - Full ECS container for rendering data
///
/// Now contains:
/// - ECS structure (entities, components) - NEW: aligned with Bevy
/// - Entity mapping with Main World (via MainEntity component)
/// - Render commands (generated from components)
/// - Views and phases
///
/// The Render World is populated during the Extract phase
/// and executes rendering during the Render phase.
pub struct RenderWorld {
    // === ECS Core (NEW - aligned with Bevy) ===
    /// Next entity ID to allocate
    next_entity_id: u64,
    /// Active entities
    entities: Vec<Entity>,
    /// Component storage: TypeId -> EntityId -> Component
    components: BTreeMap<TypeId, BTreeMap<u64, Box<dyn Any>>>,

    // === Render Data ===
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
            // ECS Core (NEW)
            next_entity_id: 0,
            entities: Vec::new(),
            components: BTreeMap::new(),

            // Render Data
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

    // =========================================================================
    // ECS Operations (NEW - aligned with Bevy)
    // =========================================================================

    /// Spawn a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }

    /// Spawn an entity with MainEntity component (linked to Main World)
    pub fn spawn_synced(&mut self, main_entity: Entity) -> Entity {
        let render_entity = self.spawn();
        self.insert_component(render_entity, MainEntity(main_entity));
        render_entity
    }

    /// Despawn an entity and all its components
    pub fn despawn(&mut self, entity: Entity) {
        if let Some(pos) = self.entities.iter().position(|e| e.id() == entity.id()) {
            self.entities.swap_remove(pos);
        }

        for (_, storage) in self.components.iter_mut() {
            storage.remove(&entity.id());
        }
    }

    /// Insert a component for an entity
    pub fn insert_component<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self.components
            .entry(type_id)
            .or_insert_with(BTreeMap::new);
        storage.insert(entity.id(), Box::new(component));
    }

    /// Get a component reference
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .and_then(|storage| storage.get(&entity.id()))
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable component reference
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|storage| storage.get_mut(&entity.id()))
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove a component from an entity
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|storage| storage.remove(&entity.id()))
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Query all entities with a specific component type
    pub fn query<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        let type_id = TypeId::of::<T>();
        let entities = &self.entities;

        self.components
            .get(&type_id)
            .map(move |storage| {
                storage.iter()
                    .filter_map(move |(entity_id, boxed)| {
                        entities.iter()
                            .find(|e| e.id() == *entity_id)
                            .zip(boxed.downcast_ref::<T>())
                            .map(|(e, c)| (*e, c))
                    })
            })
            .into_iter()
            .flatten()
    }

    /// Get entity by MainEntity component
    pub fn get_entity_by_main(&self, main_entity: Entity) -> Option<Entity> {
        self.query::<MainEntity>()
            .find(|(_, main)| main.id() == main_entity)
            .map(|(entity, _)| entity)
    }

    /// Check if an entity exists
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entities.iter().any(|e| e.id() == entity.id())
    }

    /// Get all entities
    pub fn entities(&self) -> &[Entity] {
        &self.entities
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

    /// Create default 3D perspective view
    pub fn create_default_view(&mut self) -> usize {
        let viewport = Rect::new(0.0, 0.0, self.width as f32, self.height as f32);
        let projection = crate::math::Mat4::perspective_rh(
            45.0_f32.to_radians(),
            self.width as f32 / self.height as f32,
            0.1,
            1000.0,
        );
        let camera_pos = crate::math::Vec3::new(
            self.width as f32 / 2.0,
            self.height as f32 / 2.0,
            600.0,
        );
        let target_pos = crate::math::Vec3::new(
            self.width as f32 / 2.0,
            self.height as f32 / 2.0,
            0.0,
        );
        let view = crate::math::Mat4::look_at_rh(
            camera_pos,
            target_pos,
            crate::math::Vec3::new(0.0, 1.0, 0.0),
        );
        let vp_matrix = projection * view;

        let view_bundle = ViewBundle {
            view: super::view::View {
                projection,
                view,
                view_projection: vp_matrix,
                camera_position: camera_pos,
                near: 0.1,
                far: 1000.0,
                orthographic: false,
                viewport,
            },
            target: super::view::ViewTarget::Screen,
            clear: super::view::ClearConfig::color(self.clear_color),
        };
        self.add_view(view_bundle)
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
        debug_print!(b"[RENDER] Executing %d commands, %d views\n\0".as_ptr(),
            self.commands.len() as i32,
            self.views.len() as i32,
        );

        // Clear the framebuffer with clear_color before rendering
        self.backend.clear(self.clear_color);

        // Execute all commands through the backend
        self.backend.execute_commands(&self.commands);

        // Debug: check framebuffer after render
        #[cfg(feature = "debug")]
        {
            let fb = self.backend.framebuffer();
            let mut non_zero = 0;
            // Check the entire framebuffer
            for i in 0..fb.len() {
                if fb[i] != 0 {
                    non_zero += 1;
                }
            }
            debug_print!(b"[RENDER] Framebuffer non-zero pixels: %d/%d\n\0".as_ptr(),
                non_zero,
                fb.len() as i32,
            );
            // Sample a few pixels from the expected book area (around 125,150 to 275,350)
            let sample_y = 200;
            let sample_x = 200;
            let idx = (sample_y * self.width + sample_x) as usize;
            if idx < fb.len() {
                debug_print!(b"[RENDER] Sample pixel at (200,200): 0x%x\n\0".as_ptr(), fb[idx]);
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

    // =========================================================================
    // Getters
    // =========================================================================

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
}

impl Default for RenderWorld {
    fn default() -> Self {
        Self::new(800, 600)
    }
}
