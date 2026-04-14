//! Pipeline Module
//!
//! Manages GPU/CPU hybrid rendering and batch processing.
//!
//! This module implements the rendering pipeline:
//! 1. Collect render commands from RenderWorld
//! 2. Build batches (merge compatible draw calls)
//! 3. Submit to GPU or execute on CPU
//!
//! Architecture:
//! ```
//! RenderCommand → BatchBuilder → RenderBatch → GPU/CPU Execution
//! ```

pub mod batch;

pub use batch::{GpuTaskCollector, HybridScheduler, RenderBatch, BatchStats, TaskType};
