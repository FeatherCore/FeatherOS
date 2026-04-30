//! Render Phase Module
//!
//! Inspired by Bevy's render phase system, but simplified for embedded systems.
//! Manages draw commands in phases for efficient rendering.
//!
//! # Architecture
//!
//! ```text
//! Extract Phase → Queue Phase → RenderPhases → Sort → Execute
//!                     ↓
//!              PhaseItem { sort_key, z_depth, RenderCommand }
//! ```
//!
//! # Usage
//!
//! ```ignore
//! // In queue phase, add items to phases
//! render_world.add_phase_item(RenderPhaseType::Transparent, phase_item);
//!
//! // Before render, sort all phases
//! render_world.sort_phases();
//!
//! // Execute renders commands in sorted order
//! render_world.execute_render();
//! ```

use super::command::RenderCommand;
use crate::math::Vec2;
use alloc::vec::Vec;

/// Render phase type - determines rendering order and behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderPhaseType {
    /// Background/clear phase (first)
    Background = 0,
    /// Opaque 2D objects
    Opaque2d = 1,
    /// Opaque 3D objects
    Opaque3d = 2,
    /// Alpha-masked objects (cutout transparency)
    AlphaMask = 3,
    /// Transparent objects (sorted back-to-front)
    Transparent = 4,
    /// UI overlay (last)
    Ui = 5,
}

impl RenderPhaseType {
    /// Get all phases in rendering order
    pub fn all() -> [RenderPhaseType; 6] {
        [
            RenderPhaseType::Background,
            RenderPhaseType::Opaque2d,
            RenderPhaseType::Opaque3d,
            RenderPhaseType::AlphaMask,
            RenderPhaseType::Transparent,
            RenderPhaseType::Ui,
        ]
    }

    /// Check if this phase requires sorting
    pub fn requires_sorting(&self) -> bool {
        matches!(self, RenderPhaseType::Transparent)
    }

    /// Check if this phase supports batching
    pub fn supports_batching(&self) -> bool {
        !matches!(self, RenderPhaseType::Transparent | RenderPhaseType::Ui)
    }
}

/// Phase item - a single drawable item in a render phase
///
/// Each item contains a render command and sorting information.
/// Items are sorted within their phase before rendering.
#[derive(Debug, Clone)]
pub struct PhaseItem {
    /// Sort key for ordering within phase (lower = rendered first)
    pub sort_key: i32,
    /// Z-depth for sorting transparent objects (higher = further back)
    pub z_depth: f32,
    /// Entity ID (optional, for debugging)
    pub entity_id: Option<u32>,
    /// The render command to execute
    pub command: RenderCommand,
    /// Batch key for batching similar items (0 = no batching)
    pub batch_key: u64,
}

impl PhaseItem {
    /// Create a new phase item with a render command
    pub fn new(command: RenderCommand) -> Self {
        Self {
            sort_key: 0,
            z_depth: 0.0,
            entity_id: None,
            command,
            batch_key: 0,
        }
    }

    /// Create from opaque 3D mesh
    pub fn opaque_3d(command: RenderCommand, sort_key: i32) -> Self {
        Self {
            sort_key,
            z_depth: 0.0,
            entity_id: None,
            command,
            batch_key: 0,
        }
    }

    /// Create from transparent 3D mesh (sorted by z_depth)
    pub fn transparent(command: RenderCommand, z_depth: f32) -> Self {
        Self {
            sort_key: 0,
            z_depth,
            entity_id: None,
            command,
            batch_key: 0,
        }
    }

    /// Create from UI element
    pub fn ui(command: RenderCommand, sort_key: i32) -> Self {
        Self {
            sort_key,
            z_depth: 0.0,
            entity_id: None,
            command,
            batch_key: 0,
        }
    }

    /// Set sort key
    pub fn with_sort_key(mut self, key: i32) -> Self {
        self.sort_key = key;
        self
    }

    /// Set Z depth (for transparent sorting)
    pub fn with_z_depth(mut self, depth: f32) -> Self {
        self.z_depth = depth;
        self
    }

    /// Set entity ID
    pub fn with_entity(mut self, entity_id: u32) -> Self {
        self.entity_id = Some(entity_id);
        self
    }

    /// Set batch key
    pub fn with_batch_key(mut self, key: u64) -> Self {
        self.batch_key = key;
        self
    }
}

/// Render phase - contains items to render in a specific order
#[derive(Debug, Clone)]
pub struct RenderPhase {
    /// Phase type
    pub phase_type: RenderPhaseType,
    /// Items in this phase
    pub items: Vec<PhaseItem>,
    /// Whether items are sorted
    pub sorted: bool,
}

impl RenderPhase {
    /// Create a new render phase
    pub fn new(phase_type: RenderPhaseType) -> Self {
        Self {
            phase_type,
            items: Vec::new(),
            sorted: false,
        }
    }

    /// Add an item to this phase
    pub fn add(&mut self, item: PhaseItem) {
        self.items.push(item);
        self.sorted = false;
    }

    /// Sort items in this phase
    pub fn sort(&mut self) {
        if self.sorted {
            return;
        }
        
        if self.phase_type.requires_sorting() {
            // Sort by Z depth (back to front for painter's algorithm)
            // Smaller Z = further from camera, render first
            // Larger Z = closer to camera, render last (on top)
            self.items.sort_by(|a, b| {
                a.z_depth.partial_cmp(&b.z_depth)
                    .unwrap_or(core::cmp::Ordering::Equal)
                    .then_with(|| a.sort_key.cmp(&b.sort_key))
            });
        } else {
            // Sort by sort key only
            self.items.sort_by_key(|item| item.sort_key);
        }
        self.sorted = true;
    }

    /// Get items (sorted if needed)
    pub fn items_mut(&mut self) -> &mut [PhaseItem] {
        if !self.sorted {
            self.sort();
        }
        &mut self.items
    }

    /// Get items (sorted if needed)
    pub fn items(&mut self) -> &[PhaseItem] {
        if !self.sorted {
            self.sort();
        }
        &self.items
    }

    /// Clear all items
    pub fn clear(&mut self) {
        self.items.clear();
        self.sorted = false;
    }

    /// Get item count
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Render phase collection - manages all render phases
#[derive(Debug, Clone)]
pub struct RenderPhases {
    phases: [RenderPhase; 6],
}

impl RenderPhases {
    /// Create new render phases
    pub fn new() -> Self {
        Self {
            phases: [
                RenderPhase::new(RenderPhaseType::Background),
                RenderPhase::new(RenderPhaseType::Opaque2d),
                RenderPhase::new(RenderPhaseType::Opaque3d),
                RenderPhase::new(RenderPhaseType::AlphaMask),
                RenderPhase::new(RenderPhaseType::Transparent),
                RenderPhase::new(RenderPhaseType::Ui),
            ],
        }
    }

    /// Get phase by type
    pub fn get(&self, phase_type: RenderPhaseType) -> &RenderPhase {
        &self.phases[phase_type as usize]
    }

    /// Get mutable phase by type
    pub fn get_mut(&mut self, phase_type: RenderPhaseType) -> &mut RenderPhase {
        &mut self.phases[phase_type as usize]
    }

    /// Add item to a specific phase
    pub fn add(&mut self, phase_type: RenderPhaseType, item: PhaseItem) {
        self.phases[phase_type as usize].add(item);
    }

    /// Sort all phases
    pub fn sort_all(&mut self) {
        for phase in &mut self.phases {
            phase.sort();
        }
    }

    /// Clear all phases
    pub fn clear_all(&mut self) {
        for phase in &mut self.phases {
            phase.clear();
        }
    }

    /// Get total item count across all phases
    pub fn total_count(&self) -> usize {
        self.phases.iter().map(|p| p.len()).sum()
    }

    /// Check if all phases are empty
    pub fn is_empty(&self) -> bool {
        self.phases.iter().all(|p| p.is_empty())
    }

    /// Iterate over all phases in order
    pub fn iter(&self) -> impl Iterator<Item = &RenderPhase> {
        self.phases.iter()
    }

    /// Iterate over all phases mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RenderPhase> {
        self.phases.iter_mut()
    }

    /// Collect all render commands in order (sorted)
    pub fn collect_commands(&mut self) -> Vec<RenderCommand> {
        self.sort_all();
        self.phases.iter_mut()
            .flat_map(|phase| phase.items.iter())
            .map(|item| item.command.clone())
            .collect()
    }
}

impl Default for RenderPhases {
    fn default() -> Self {
        Self::new()
    }
}
