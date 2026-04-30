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
//!
//! Directory structure:
//! ```
//! pipeline/
//! ├── mod.rs           # This file
//! ├── renderer.rs      # Renderer trait
//! ├── batch.rs         # Batch processing system
//! ├── texture.rs       # Texture system
//! ├── gradient.rs      # Gradient system
//! ├── software/        # CPU software rendering
//! │   ├── mod.rs
//! │   └── backend.rs
//! └── gpu/             # GPU rendering (future)
//!     └── mod.rs
//! ```

pub mod renderer;
pub mod batch;
pub mod texture;
pub mod gradient;
pub mod software;
pub mod gpu;

pub use renderer::{Renderer, RendererType};
pub use batch::{GpuTaskCollector, HybridScheduler, RenderBatch, BatchStats, TaskType};
pub use texture::{Texture, TextureFormat, Sampler, SamplerFilter, SamplerAddress, TextureRegion};
pub use gradient::{Gradient, GradientDirection, GradientStop, PrecomputedGradient};
pub use software::SoftwareBackend;
