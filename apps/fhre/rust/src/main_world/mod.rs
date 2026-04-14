//! Main World Module
//!
//! The Main World is the primary ECS world containing game entities,
//! components, and systems. This is where the game logic runs.

// Sub-modules
mod world;
mod entity;
mod component;
mod system;

// Re-exports
pub use world::MainWorld;
pub use entity::Entity;
pub use component::{Component, Transform, Sprite, Velocity};
pub use system::{System, IntoSystem};
