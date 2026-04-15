//! Extract system for FHRE
//!
//! The Extract phase syncs data from Main World to Render World.
//! This happens after the Main World update and before rendering.

mod extract;

pub use extract::{Extract, ExtractSchedule, default_extract_schedule, extract_sprites, extract_buttons, extract_cubes, extract_soccer_balls, extract_time};
