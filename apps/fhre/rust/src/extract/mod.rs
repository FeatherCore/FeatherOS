//! Extract system for FHRE
//!
//! The Extract phase syncs data from Main World to Render World.
//! This happens after the Main World update and before rendering.
//!
//! Aligned with Bevy's extract system:
//! - ExtractSchedule: Configurable extract phase with registered extractors
//! - ExtractComponent: Trait for components that can be extracted to Render World
//! - extract_components: System to extract components using ExtractComponent trait

mod extract;
mod extract_component;
mod extract_schedule;

pub use extract::{Extract, default_extract_schedule, extract_sprites, extract_buttons, extract_cubes, extract_time, extract_renderable_components};
pub use extract_component::{ExtractComponent, extract_components};
// Macros are exported at crate root, re-export from there
pub use crate::{impl_extract_clone, impl_extract_identity};
pub use extract_schedule::ExtractSchedule;
