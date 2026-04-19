//! Schedule Label Implementation
//!
//! Labels are used to identify schedules and systems.
//! Fully aligned with Bevy's ScheduleLabel system.

use alloc::vec::Vec;
use alloc::string::String;
use core::fmt::Debug;

/// A type that can be used to identify a schedule.
///
/// Schedule labels are used to identify and reference schedules.
/// They are implemented as types that implement this trait.
///
/// # Example
///
/// ```
/// use fhre::schedule::ScheduleLabel;
///
/// #[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
/// struct MySchedule;
/// ```
pub trait ScheduleLabel: 'static + Send + Sync + Clone + Debug + PartialEq + Eq {
    /// Returns the name of the schedule.
    fn name(&self) -> String;
}

/// Interned schedule label for efficient storage and comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InternedScheduleLabel(pub(crate) &'static str);

impl InternedScheduleLabel {
    /// Create a new interned label.
    pub fn new(label: &'static str) -> Self {
        Self(label)
    }

    /// Get the label string.
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl From<&'static str> for InternedScheduleLabel {
    fn from(s: &'static str) -> Self {
        Self::new(s)
    }
}

/// Macro to derive ScheduleLabel for a type.
#[macro_export]
macro_rules! impl_schedule_label {
    ($type:ty) => {
        impl $crate::schedule::ScheduleLabel for $type {
            fn name(&self) -> alloc::string::String {
                alloc::string::String::from(core::any::type_name::<Self>())
            }
        }
    };
}

/// Standard schedule labels used by FHRE.
pub mod common {
    use super::*;

    /// Runs once at application startup.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    pub struct Startup;

    /// Runs before Update. Ideal for input handling.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    pub struct PreUpdate;

    /// The main update loop. Most game logic goes here.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    pub struct Update;

    /// Runs after Update. Ideal for transform propagation.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    pub struct PostUpdate;

    /// Runs after PostUpdate. Ideal for rendering preparation.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
    pub struct Last;

    impl ScheduleLabel for Startup {
        fn name(&self) -> String {
            String::from("Startup")
        }
    }

    impl ScheduleLabel for PreUpdate {
        fn name(&self) -> String {
            String::from("PreUpdate")
        }
    }

    impl ScheduleLabel for Update {
        fn name(&self) -> String {
            String::from("Update")
        }
    }

    impl ScheduleLabel for PostUpdate {
        fn name(&self) -> String {
            String::from("PostUpdate")
        }
    }

    impl ScheduleLabel for Last {
        fn name(&self) -> String {
            String::from("Last")
        }
    }
}

/// Label trait - For labeling systems and sets (legacy, kept for compatibility)
pub trait Label: Clone + PartialEq + Debug {
    /// Get the label name
    fn name(&self) -> &'static str;
}

/// System label - For identifying individual systems
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SystemLabel {
    /// Named system
    Name(&'static str),
    /// Anonymous system (auto-generated)
    Anonymous(u64),
}

impl SystemLabel {
    /// Create a named label
    pub fn new(name: &'static str) -> Self {
        SystemLabel::Name(name)
    }

    /// Get the label name
    pub fn name(&self) -> &'static str {
        match self {
            SystemLabel::Name(name) => name,
            SystemLabel::Anonymous(_) => "<anonymous>",
        }
    }
}

impl Label for SystemLabel {
    fn name(&self) -> &'static str {
        self.name()
    }
}

/// Label ordering - Define before/after relationships
#[derive(Clone, Debug)]
pub enum LabelOrder {
    /// Run before this label
    Before(&'static str),
    /// Run after this label
    After(&'static str),
}

/// Helper function to create a label array
pub fn labels(names: &[&'static str]) -> Vec<SystemLabel> {
    names.iter().map(|&name| SystemLabel::new(name)).collect()
}
