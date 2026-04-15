//! Render Phase Module
//!
//! Inspired by Bevy's render phase system, but simplified for embedded systems.
//! Manages draw commands in phases for efficient rendering.

use alloc::vec::Vec;
use crate::math::Color;

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
#[derive(Debug, Clone)]
pub struct PhaseItem {
    /// Sort key for ordering within phase
    pub sort_key: i32,
    /// Z-depth for sorting (higher = further back)
    pub z_depth: f32,
    /// Entity ID (optional)
    pub entity_id: Option<u32>,
    /// Draw command index
    pub draw_command_index: usize,
    /// Whether this item can be batched
    pub batchable: bool,
    /// Batch key (for batching similar items)
    pub batch_key: u64,
}

impl PhaseItem {
    /// Create a new phase item
    pub fn new(draw_command_index: usize) -> Self {
        Self {
            sort_key: 0,
            z_depth: 0.0,
            entity_id: None,
            draw_command_index,
            batchable: true,
            batch_key: 0,
        }
    }

    /// Set sort key
    pub fn with_sort_key(mut self, key: i32) -> Self {
        self.sort_key = key;
        self
    }

    /// Set Z depth
    pub fn with_z_depth(mut self, depth: f32) -> Self {
        self.z_depth = depth;
        self
    }

    /// Set entity ID
    pub fn with_entity(mut self, entity_id: u32) -> Self {
        self.entity_id = Some(entity_id);
        self
    }

    /// Set batchable
    pub fn with_batchable(mut self, batchable: bool) -> Self {
        self.batchable = batchable;
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
    pub fn add_item(&mut self, item: PhaseItem) {
        self.items.push(item);
        self.sorted = false;
    }

    /// Sort items in this phase
    pub fn sort(&mut self) {
        if self.phase_type.requires_sorting() {
            // Sort by Z depth (back to front for transparent)
            self.items.sort_by(|a, b| {
                b.z_depth.partial_cmp(&a.z_depth)
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
    pub fn items(&mut self) -> &[PhaseItem] {
        if !self.sorted && self.phase_type.requires_sorting() {
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
    pub fn count(&self) -> usize {
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
    /// All phases
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
    pub fn add_item(&mut self, phase_type: RenderPhaseType, item: PhaseItem) {
        self.phases[phase_type as usize].add_item(item);
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
        self.phases.iter().map(|p| p.count()).sum()
    }

    /// Iterate over all phases in order
    pub fn iter(&self) -> impl Iterator<Item = &RenderPhase> {
        self.phases.iter()
    }

    /// Iterate over all phases mutably
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RenderPhase> {
        self.phases.iter_mut()
    }
}

impl Default for RenderPhases {
    fn default() -> Self {
        Self::new()
    }
}

/// Phase batch - groups items that can be rendered together
#[derive(Debug, Clone)]
pub struct PhaseBatch {
    /// Start index in phase items
    pub start_index: usize,
    /// Number of items in batch
    pub count: usize,
    /// Batch key (identifies similar items)
    pub batch_key: u64,
    /// Phase type
    pub phase_type: RenderPhaseType,
}

impl PhaseBatch {
    /// Create a new batch
    pub fn new(start_index: usize, batch_key: u64, phase_type: RenderPhaseType) -> Self {
        Self {
            start_index,
            count: 1,
            batch_key,
            phase_type,
        }
    }

    /// Add item to batch
    pub fn add_item(&mut self) {
        self.count += 1;
    }

    /// Check if item can be added to this batch
    pub fn can_batch(&self, batch_key: u64) -> bool {
        self.batch_key == batch_key
    }
}

/// Batch builder - creates batches from phase items
pub struct BatchBuilder;

impl BatchBuilder {
    /// Build batches from phase items
    pub fn build_batches(phase: &RenderPhase) -> Vec<PhaseBatch> {
        let mut batches = Vec::new();
        
        if phase.items.is_empty() {
            return batches;
        }

        // Only batch phases that support it
        if !phase.phase_type.supports_batching() {
            // Create individual batches for each item
            for (i, _) in phase.items.iter().enumerate() {
                batches.push(PhaseBatch::new(
                    i,
                    i as u64,
                    phase.phase_type,
                ));
            }
            return batches;
        }

        // Build batches from batchable items
        let mut current_batch: Option<PhaseBatch> = None;

        for (i, item) in phase.items.iter().enumerate() {
            if !item.batchable {
                // Finish current batch if any
                if let Some(batch) = current_batch.take() {
                    batches.push(batch);
                }
                // Add non-batchable item as individual batch
                batches.push(PhaseBatch::new(
                    i,
                    item.batch_key,
                    phase.phase_type,
                ));
            } else if let Some(ref mut batch) = current_batch {
                if batch.can_batch(item.batch_key) {
                    batch.add_item();
                } else {
                    batches.push(current_batch.take().unwrap());
                    current_batch = Some(PhaseBatch::new(
                        i,
                        item.batch_key,
                        phase.phase_type,
                    ));
                }
            } else {
                current_batch = Some(PhaseBatch::new(
                    i,
                    item.batch_key,
                    phase.phase_type,
                ));
            }
        }

        // Don't forget the last batch
        if let Some(batch) = current_batch {
            batches.push(batch);
        }

        batches
    }
}
