//! Pipeline Module
//!
//! Manages rendering backends and execution.
//!
//! This module implements the rendering pipeline:
//! 1. Collect render commands from RenderWorld
//! 2. Route to appropriate backend (Software/GPU/Hybrid)
//! 3. Execute and output to framebuffer
//!
//! Architecture:
//! ```
//! RenderCommand → Pipeline → Backend (Software/GPU/Hybrid) → Framebuffer
//! ```

pub mod backend;
pub mod batch;
pub mod renderer;

pub use backend::SoftwareBackend;
pub use renderer::{Renderer, RendererType};
pub use batch::{GpuTaskCollector, HybridScheduler, RenderBatch, BatchStats, TaskType};
