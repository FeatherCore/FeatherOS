//! Extract system for FHRE
//!
//! The Extract phase syncs data from Main World to Render World.
//! This happens after the Main World update and before rendering.

mod extract;

pub use extract::{Extract, ExtractSchedule, ExtractResource, ExtractFn, extract_system};
