//! Render World - Full ECS Implementation (Bevy-aligned)
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
//! Entity + Cube           →     Entity + ExtractedMesh
//! Entity + SoccerBall     →     Entity + ExtractedMesh
//! ```
//!
//! # Render Pipeline
//!
//! 1. Extract Phase: Copy components from Main World to Render World
//! 2. Queue Phase: Generate PhaseItems into RenderPhases
//! 3. Sort Phase: Sort items within each phase
//! 4. Render Phase: Execute render commands in sorted order
//!
//! This matches Bevy's architecture where Render World has its own ECS.

use super::command::RenderCommand;
use super::object::RenderObject;
use super::phase::{RenderPhases, RenderPhaseType, PhaseItem};
use super::view::ViewBundle;
use crate::{Entity, Component};
use crate::sync::MainEntity;
use crate::pipeline::{SoftwareBackend, Texture};
use crate::math::{Color, Rect};
use crate::resources::Resource;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::any::{TypeId, Any};

/// Render World - Full ECS container for rendering data
///
/// Contains:
/// - ECS structure (entities, components) - aligned with Bevy
/// - Entity mapping with Main World (via MainEntity component)
/// - Render phases (for automatic sorting)
/// - Views
pub struct RenderWorld {
    // === ECS Core ===
    next_entity_id: u64,
    entities: Vec<Entity>,
    components: BTreeMap<TypeId, BTreeMap<u64, Box<dyn Any>>>,
    
    // === Resources ===
    resources: BTreeMap<TypeId, Box<dyn Any>>,
    
    // === Entity Mapping ===
    /// Map from Main World entity ID to Render World entity ID
    main_to_render: BTreeMap<u64, Entity>,
    
    // === Render Data ===
    width: u32,
    height: u32,
    objects: Vec<RenderObject>,
    clear_color: Color,
    viewport: Rect,
    phases: RenderPhases,
    views: Vec<ViewBundle>,
    current_view: Option<usize>,
    backend: SoftwareBackend,
}

impl RenderWorld {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            next_entity_id: 0,
            entities: Vec::new(),
            components: BTreeMap::new(),
            resources: BTreeMap::new(),
            main_to_render: BTreeMap::new(),
            width,
            height,
            objects: Vec::new(),
            clear_color: Color::BLACK,
            viewport: Rect::new(0.0, 0.0, width as f32, height as f32),
            phases: RenderPhases::new(),
            views: Vec::new(),
            current_view: None,
            backend: SoftwareBackend::new(width, height),
        }
    }

    // =========================================================================
    // Resource Operations
    // =========================================================================

    /// Initialize a resource with its default value.
    pub fn init_resource<R: Resource + Default>(&mut self) {
        self.resources.insert(TypeId::of::<R>(), Box::new(R::default()));
    }

    /// Insert a resource.
    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }

    /// Get a reference to a resource.
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.resources.get(&TypeId::of::<R>())?.downcast_ref::<R>()
    }

    /// Get a mutable reference to a resource.
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.resources.get_mut(&TypeId::of::<R>())?.downcast_mut::<R>()
    }

    /// Remove a resource.
    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        self.resources.remove(&TypeId::of::<R>())?.downcast::<R>().ok().map(|b| *b)
    }

    /// Check if a resource exists.
    pub fn contains_resource<R: Resource>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }

    // =========================================================================
    // ECS Operations
    // =========================================================================

    /// Spawn a new entity
    pub fn spawn(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }

    /// Spawn an entity linked to a Main World entity
    pub fn spawn_synced(&mut self, main_entity: Entity) -> Entity {
        let render_entity = self.spawn();
        self.insert_component(render_entity, MainEntity(main_entity));
        self.main_to_render.insert(main_entity.id(), render_entity);
        render_entity
    }

    /// Get or spawn a synced entity
    pub fn get_or_spawn_synced(&mut self, main_entity: Entity) -> Entity {
        if let Some(&render_entity) = self.main_to_render.get(&main_entity.id()) {
            if self.entities.iter().any(|e| e.id() == render_entity.id()) {
                return render_entity;
            }
        }
        self.spawn_synced(main_entity)
    }

    /// Despawn an entity
    pub fn despawn(&mut self, entity: Entity) {
        if let Some(pos) = self.entities.iter().position(|e| e.id() == entity.id()) {
            self.entities.swap_remove(pos);
        }
        for (_, storage) in self.components.iter_mut() {
            storage.remove(&entity.id());
        }
        // Remove from mapping
        self.main_to_render.retain(|_, v| v.id() != entity.id());
    }

    /// Insert a component
    pub fn insert_component<T: Component>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        let storage = self.components.entry(type_id).or_insert_with(BTreeMap::new);
        storage.insert(entity.id(), Box::new(component));
    }

    /// Get a component
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get(&type_id)
            .and_then(|storage| storage.get(&entity.id()))
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Get a mutable component
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|storage| storage.get_mut(&entity.id()))
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Remove a component
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.components
            .get_mut(&type_id)
            .and_then(|storage| storage.remove(&entity.id()))
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    /// Query entities with a component
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

    /// Query entities with two components
    pub fn query2<A: Component, B: Component>(&self) -> impl Iterator<Item = (Entity, &A, &B)> {
        let entities = &self.entities;
        let type_a = TypeId::of::<A>();
        let type_b = TypeId::of::<B>();
        
        let storage_a = self.components.get(&type_a);
        let storage_b = self.components.get(&type_b);
        
        entities.iter().filter_map(move |entity| {
            let a = storage_a.as_ref()?.get(&entity.id())?.downcast_ref::<A>()?;
            let b = storage_b.as_ref()?.get(&entity.id())?.downcast_ref::<B>()?;
            Some((*entity, a, b))
        })
    }

    /// Get entity by Main World entity
    pub fn get_entity_by_main(&self, main_entity: Entity) -> Option<Entity> {
        self.main_to_render.get(&main_entity.id()).copied()
    }

    /// Get all entities
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
    
    /// Get the main-to-render entity mapping
    pub fn main_to_render(&self) -> &BTreeMap<u64, Entity> {
        &self.main_to_render
    }

    /// Clear all entities and components
    pub fn clear_entities(&mut self) {
        self.entities.clear();
        self.components.clear();
        self.main_to_render.clear();
    }

    // =========================================================================
    // Render Phases (Bevy-aligned)
    // =========================================================================

    /// Add a phase item to a specific phase
    pub fn add_phase_item(&mut self, phase_type: RenderPhaseType, item: PhaseItem) {
        self.phases.add(phase_type, item);
    }

    /// Get mutable access to render phases
    pub fn phases_mut(&mut self) -> &mut RenderPhases {
        &mut self.phases
    }

    /// Get render phases
    pub fn phases(&self) -> &RenderPhases {
        &self.phases
    }

    /// Sort all phases
    pub fn sort_phases(&mut self) {
        self.phases.sort_all();
    }

    /// Clear all phases
    pub fn clear_phases(&mut self) {
        self.phases.clear_all();
    }

    // =========================================================================
    // Legacy Render Commands (deprecated, use phases instead)
    // =========================================================================

    /// Add a render command directly (legacy, prefer add_phase_item)
    #[deprecated(note = "Use add_phase_item for automatic sorting")]
    pub fn add_command(&mut self, command: RenderCommand) {
        let item = PhaseItem::new(command);
        self.phases.add(RenderPhaseType::Opaque3d, item);
    }

    /// Get all render commands from phases (sorted)
    pub fn commands(&self) -> Vec<RenderCommand> {
        self.phases.clone().collect_commands()
    }

    /// Clear all render data
    pub fn clear_commands(&mut self) {
        self.phases.clear_all();
    }

    // =========================================================================
    // Views
    // =========================================================================

    pub fn add_view(&mut self, view: ViewBundle) -> usize {
        let index = self.views.len();
        self.views.push(view);
        index
    }

    pub fn current_view(&self) -> Option<&ViewBundle> {
        self.current_view.and_then(|idx| self.views.get(idx))
    }

    pub fn set_current_view(&mut self, index: Option<usize>) {
        self.current_view = index;
    }

    pub fn clear_views(&mut self) {
        self.views.clear();
        self.current_view = None;
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    /// Execute render - runs the render pipeline
    /// 
    /// This sorts all phases and executes commands in order.
    pub fn execute_render(&mut self) {
        self.backend.clear(self.clear_color);
        
        let commands = self.phases.collect_commands();
        self.backend.execute_commands(&commands);
    }

    /// Get the framebuffer
    pub fn framebuffer(&self) -> &[u32] {
        self.backend.framebuffer()
    }

    /// Get dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn upload_texture(&mut self, id: u32, texture: Texture) {
        self.backend.upload_texture(id, texture);
    }

    pub fn remove_texture(&mut self, id: u32) -> Option<Texture> {
        self.backend.remove_texture(id)
    }

    pub fn get_texture(&self, id: u32) -> Option<&Texture> {
        self.backend.get_texture(id)
    }
}
