//! Main World Module
//!
//! The Main World is the primary ECS world containing game entities,
//! components, and systems. This is where the game logic runs.

mod tuples;
mod world;
mod entity;
mod component;
mod system;
mod system_param;
mod commands;
pub mod query_filter;
mod change_detection;
mod filtered_query;
mod query_data;

pub use world::MainWorld;
pub use entity::Entity;
pub use component::{Component, Sprite, Velocity};
pub use system::{System, IntoSystem, system1, system2, system3, system4, system5, system6, system7, system8, system9, system10};
pub use system_param::{SystemParam, Res, ResMut, Local, FromWorld, EntityRef, EntityMut};
pub use commands::{Commands, Command, CommandsState, EntityCommands};
pub use query_filter::{QueryFilter, With, Without, Or, And, Added, Changed};
pub use change_detection::{ChangeDetection, ChangeTicks, Mut, Ref};
pub use filtered_query::{Query, QueryState, FilteredQuery};
pub use query_data::{QueryData, MultiQuery, MultiQueryState};
