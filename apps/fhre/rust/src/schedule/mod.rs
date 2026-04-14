//! Schedule system for FHRE
//!
//! Provides ordered execution of systems with labels and sets.
//! Simplified version of Bevy's schedule system.

mod schedule;
mod label;
mod set;

pub use schedule::{Schedule, Schedules};
pub use label::{ScheduleLabel, Label, labels};
pub use set::{SystemSet};
