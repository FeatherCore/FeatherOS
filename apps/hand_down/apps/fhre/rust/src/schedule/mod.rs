//! Schedule system for FHRE
//!
//! Provides ordered execution of systems with labels and sets.
//! Fully aligned with Bevy's schedule system.

mod schedule;
pub mod label;
mod set;
mod condition;

pub use schedule::{Schedule, Schedules};
pub use label::{
    ScheduleLabel, InternedScheduleLabel,
    Label, SystemLabel, labels, LabelOrder,
};
pub use label::common::{Startup, PreUpdate, Update, PostUpdate, Last};
pub use set::{SystemSet};
pub use condition::{Condition, common_conditions, and, or, not};
