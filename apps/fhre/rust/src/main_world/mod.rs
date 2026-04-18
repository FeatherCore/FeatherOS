//! Main World Module
//!
//! The Main World is the primary ECS world containing game entities,
//! components, and systems. This is where the game logic runs.

// Sub-modules
mod world;
mod entity;
mod component;
mod system;
mod system_param;
mod commands;
mod query_filter;
mod change_detection;
mod filtered_query;
mod query_data;

// Re-exports
pub use world::MainWorld;
pub use entity::Entity;
pub use component::{Component, Transform, Sprite, Velocity};
pub use system::{System, IntoSystem, system1, system2, system3, system4};
pub use system_param::{SystemParam, Res, ResMut, Local, FromWorld};
pub use commands::{Commands, Command, CommandsState, EntityCommands};
pub use query_filter::{QueryFilter, With, Without, Or, And};
pub use change_detection::{ChangeDetection, ChangeTicks, Mut, Ref};
pub use filtered_query::{FilteredQuery, FilteredQueryIter, FilteredQueryIterMut};
pub use query_data::{QueryData, MultiQuery, MultiQueryState};

// Type aliases for Bevy-style Query syntax
/// Single component query with optional filter
pub type Query<'w, 's, T, F = ()> = FilteredQuery<'w, 's, T, F>;

/// Multi-component query using QueryData trait
pub type MultiCompQuery<'w, 's, D, F = ()> = MultiQuery<'w, 's, D, F>;
