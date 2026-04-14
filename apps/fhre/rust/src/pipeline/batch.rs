//! Batch Rendering System
//!
//! Implements GPU/CPU hybrid batch rendering for maximum efficiency.
//!
//! Core Principles (from GPU_CPU_Rendering_Optimization.md):
//! - Rule 1: If it can maintain batch continuity → give to GPU regardless of size
//! - Rule 2: Only isolated primitives that can't join any batch go to CPU
//! - Rule 3: All GPU tasks in same group merge into one batch submission
//! - Rule 4: GPU tasks should be continuous, many, and long
//!
//! This is part of the render_graph module, called during the Render schedule phase.

use crate::render_world::{DrawCall, PrimitiveType, RenderCommand, Vertex, RenderWorld};
use crate::math::{Color, Rect, Vec2};
use alloc::vec::Vec;
use libm::sqrtf;
use alloc::boxed::Box;

/// Maximum vertices per batch (prevents buffer overflow)
const MAX_BATCH_VERTICES: usize = 65536;
/// Maximum indices per batch
const MAX_BATCH_INDICES: usize = 65536;
/// Default batch size threshold
const DEFAULT_BATCH_THRESHOLD: usize = 1024;
/// Maximum wait time before forced submission (ms)
const MAX_WAIT_TIME_MS: u32 = 16;

/// GPU Task Collector - Collects and batches GPU-compatible render commands
///
/// This is the core of the hybrid rendering strategy. It maintains batch
/// continuity by only breaking batches when absolutely necessary.
pub struct GpuTaskCollector {
    /// Current batch being built
    current_batch: Option<RenderBatch>,
    /// Completed batches ready for submission
    completed_batches: Vec<RenderBatch>,
    /// Batch size threshold before forced submission
    batch_threshold: usize,
    /// Last submission timestamp
    last_submit_time: u32,
    /// Statistics for debugging
    stats: BatchStats,
}

/// A batch of render commands that can be submitted to GPU in one draw call
#[derive(Clone, Debug)]
pub struct RenderBatch {
    /// Primitive type for this batch
    pub primitive: PrimitiveType,
    /// Merged vertex data
    pub vertices: Vec<Vertex>,
    /// Merged index data
    pub indices: Vec<u16>,
    /// Number of draw calls merged into this batch
    pub draw_call_count: u32,
    /// Batch color (if uniform)
    pub uniform_color: Option<Color>,
    /// Bounding box of all geometry
    pub bounds: Rect,
}

/// Statistics for batching performance monitoring
#[derive(Clone, Copy, Debug, Default)]
pub struct BatchStats {
    /// Total batches created
    pub total_batches: u32,
    /// Total draw calls processed
    pub total_draw_calls: u32,
    /// Average vertices per batch
    pub avg_vertices_per_batch: f32,
    /// Number of batches broken due to state change
    pub state_change_breaks: u32,
    /// Number of batches broken due to size limit
    pub size_limit_breaks: u32,
}

/// Task type for hybrid scheduling decision
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskType {
    /// Task should go to GPU (can be batched)
    Gpu,
    /// Task should go to CPU (isolated, small, or special)
    Cpu,
}

/// Hybrid Scheduler - Decides GPU vs CPU and manages batching
pub struct HybridScheduler {
    /// GPU task collector
    gpu_collector: GpuTaskCollector,
    /// CPU task queue (for isolated/small tasks)
    cpu_queue: Vec<RenderCommand>,
    /// Whether scheduler is initialized
    initialized: bool,
}

impl GpuTaskCollector {
    /// Create a new GPU task collector
    pub fn new() -> Self {
        Self {
            current_batch: None,
            completed_batches: Vec::new(),
            batch_threshold: DEFAULT_BATCH_THRESHOLD,
            last_submit_time: 0,
            stats: BatchStats::default(),
        }
    }

    /// Create with custom batch threshold
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            current_batch: None,
            completed_batches: Vec::new(),
            batch_threshold: threshold,
            last_submit_time: 0,
            stats: BatchStats::default(),
        }
    }

    /// Check if a draw call can be merged into current batch
    ///
    /// This is the key function that maintains batch continuity.
    /// We only return false if we absolutely cannot merge.
    fn can_merge(&self, draw_call: &DrawCall) -> bool {
        let Some(ref batch) = self.current_batch else {
            return true; // No current batch, can always start new one
        };

        // Check primitive type compatibility
        if batch.primitive != draw_call.primitive {
            return false; // Different primitive types can't batch
        }

        // Check size limits - prevent buffer overflow
        let new_vertex_count = batch.vertices.len() + draw_call.vertices.len();
        let new_index_count = batch.indices.len() + draw_call.indices.len();

        if new_vertex_count > MAX_BATCH_VERTICES || new_index_count > MAX_BATCH_INDICES {
            return false; // Would exceed buffer limits
        }

        true
    }

    /// Merge a draw call into the current batch
    fn merge_draw_call(&mut self, draw_call: &DrawCall) {
        let base_vertex = self.current_batch.as_ref().map(|b| b.vertices.len() as u16).unwrap_or(0);

        if self.current_batch.is_none() {
            self.current_batch = Some(RenderBatch {
                primitive: draw_call.primitive,
                vertices: Vec::new(),
                indices: Vec::new(),
                draw_call_count: 0,
                uniform_color: Some(draw_call.color),
                bounds: Rect::EMPTY,
            });
        }

        let batch = self.current_batch.as_mut().unwrap();

        // Add vertices
        batch.vertices.extend_from_slice(&draw_call.vertices);

        // Add indices with offset
        for &index in &draw_call.indices {
            batch.indices.push(index + base_vertex);
        }

        // Update draw call count
        batch.draw_call_count += 1;

        // Update uniform color check
        if batch.uniform_color != Some(draw_call.color) {
            batch.uniform_color = None; // Not uniform anymore
        }

        // Update bounds
        for vertex in &draw_call.vertices {
            batch.bounds = batch.bounds.expand_to_include(vertex.position);
        }
    }

    /// Add a draw call to the collector
    ///
    /// This implements Rule 1: "If it can maintain batch continuity → give to GPU"
    /// We only break the batch if absolutely necessary.
    pub fn add_draw_call(&mut self, draw_call: &DrawCall) {
        self.stats.total_draw_calls += 1;

        if self.can_merge(draw_call) {
            // Rule 1: Can merge → add to current batch
            self.merge_draw_call(draw_call);
        } else {
            // Must break batch → submit current and start new
            self.stats.state_change_breaks += 1;
            self.submit_current_batch();
            self.merge_draw_call(draw_call);
        }

        // Check if we should force submission due to size
        if let Some(ref batch) = self.current_batch {
            if batch.vertices.len() >= self.batch_threshold {
                self.stats.size_limit_breaks += 1;
                self.submit_current_batch();
            }
        }
    }

    /// Submit current batch and start a new one
    pub fn submit_current_batch(&mut self) {
        if let Some(batch) = self.current_batch.take() {
            if !batch.vertices.is_empty() {
                self.completed_batches.push(batch);
                self.stats.total_batches += 1;
            }
        }
    }

    /// Get all completed batches
    pub fn take_completed_batches(&mut self) -> Vec<RenderBatch> {
        self.submit_current_batch();
        core::mem::take(&mut self.completed_batches)
    }

    /// Get current statistics
    pub fn stats(&self) -> BatchStats {
        self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = BatchStats::default();
    }
}

impl HybridScheduler {
    /// Create a new hybrid scheduler
    pub fn new() -> Self {
        Self {
            gpu_collector: GpuTaskCollector::new(),
            cpu_queue: Vec::new(),
            initialized: true,
        }
    }

    /// Decide if a render command should go to GPU or CPU
    ///
    /// This implements the core decision logic from the optimization document:
    /// - Rule 2: Only isolated primitives that can't join any batch go to CPU
    fn classify_task(command: &RenderCommand) -> TaskType {
        match command {
            // Clear and scissor commands are special - they break batches
            RenderCommand::Clear { .. } => TaskType::Cpu,
            RenderCommand::SetScissor { .. } => TaskType::Cpu,
            RenderCommand::DisableScissor => TaskType::Cpu,

            // Text rendering is currently CPU-only (simplified implementation)
            RenderCommand::DrawText { .. } => TaskType::Cpu,

            // All geometric primitives can be batched for GPU
            RenderCommand::DrawRect { .. } => TaskType::Gpu,
            RenderCommand::DrawLine { .. } => TaskType::Gpu,
            RenderCommand::DrawTriangle { .. } => TaskType::Gpu,
        }
    }

    /// Submit a render command to the scheduler
    pub fn submit_command(&mut self, command: RenderCommand) {
        match Self::classify_task(&command) {
            TaskType::Gpu => {
                // Convert command to draw calls and add to GPU collector
                let draw_calls = command_to_draw_calls(&command);
                for draw_call in draw_calls {
                    self.gpu_collector.add_draw_call(&draw_call);
                }
            }
            TaskType::Cpu => {
                // Add to CPU queue
                self.cpu_queue.push(command);
            }
        }
    }

    /// Execute all pending tasks
    /// Returns (gpu_batches, cpu_commands)
    pub fn execute(&mut self) -> (Vec<RenderBatch>, Vec<RenderCommand>) {
        let gpu_batches = self.gpu_collector.take_completed_batches();
        let cpu_commands = core::mem::take(&mut self.cpu_queue);
        (gpu_batches, cpu_commands)
    }

    /// Get current statistics
    pub fn stats(&self) -> BatchStats {
        self.gpu_collector.stats()
    }
}

/// Convert a render command to draw calls
fn command_to_draw_calls(command: &RenderCommand) -> Vec<DrawCall> {
    let mut draw_calls = Vec::new();

    match command {
        RenderCommand::DrawRect { rect, color } => {
            let mut draw_call = DrawCall::new(PrimitiveType::Triangles);

            let tl = Vertex::with_color(Vec2::new(rect.x, rect.y), *color);
            let tr = Vertex::with_color(Vec2::new(rect.x + rect.width, rect.y), *color);
            let br = Vertex::with_color(Vec2::new(rect.x + rect.width, rect.y + rect.height), *color);
            let bl = Vertex::with_color(Vec2::new(rect.x, rect.y + rect.height), *color);

            draw_call.add_quad(tl, tr, br, bl);
            draw_calls.push(draw_call);
        }

        RenderCommand::DrawLine { start, end, color, thickness } => {
            let mut draw_call = DrawCall::new(PrimitiveType::Triangles);

            // Create a thick line using two triangles
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let len = sqrtf(dx * dx + dy * dy);

            if len > 0.0 {
                let nx = -dy / len * (*thickness * 0.5);
                let ny = dx / len * (*thickness * 0.5);

                let p0 = Vertex::with_color(Vec2::new(start.x + nx, start.y + ny), *color);
                let p1 = Vertex::with_color(Vec2::new(end.x + nx, end.y + ny), *color);
                let p2 = Vertex::with_color(Vec2::new(end.x - nx, end.y - ny), *color);
                let p3 = Vertex::with_color(Vec2::new(start.x - nx, start.y - ny), *color);

                draw_call.add_quad(p0, p1, p2, p3);
                draw_calls.push(draw_call);
            }
        }

        RenderCommand::DrawTriangle { p0, p1, p2, color } => {
            let mut draw_call = DrawCall::new(PrimitiveType::Triangles);

            let v0 = Vertex::with_color(*p0, *color);
            let v1 = Vertex::with_color(*p1, *color);
            let v2 = Vertex::with_color(*p2, *color);

            draw_call.add_triangle(v0, v1, v2);
            draw_calls.push(draw_call);
        }

        _ => {
            // Other commands don't produce draw calls (handled by CPU)
        }
    }

    draw_calls
}

impl RenderBatch {
    /// Create an empty render batch
    pub fn empty() -> Self {
        Self {
            primitive: PrimitiveType::Triangles,
            vertices: Vec::new(),
            indices: Vec::new(),
            draw_call_count: 0,
            uniform_color: None,
            bounds: Rect::EMPTY,
        }
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get index count
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}

impl Default for GpuTaskCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HybridScheduler {
    fn default() -> Self {
        Self::new()
    }
}
