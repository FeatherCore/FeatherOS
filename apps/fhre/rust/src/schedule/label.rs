//! Schedule Label Implementation
//!
//! Labels are used to identify schedules and systems.

use alloc::vec::Vec;

/// ScheduleLabel - Identifies a schedule
///
/// Used to reference specific schedules like Update, Render, etc.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScheduleLabel {
    /// Pre-update phase (input handling, etc.)
    PreUpdate,
    /// Main update phase (game logic)
    Update,
    /// Post-update phase (cleanup, etc.)
    PostUpdate,
    /// Extract phase (sync to render world)
    Extract,
    /// Render phase (draw to screen)
    Render,
    /// Custom label
    Custom(&'static str),
}

impl ScheduleLabel {
    /// Get the name of the label
    pub fn name(&self) -> &'static str {
        match self {
            ScheduleLabel::PreUpdate => "PreUpdate",
            ScheduleLabel::Update => "Update",
            ScheduleLabel::PostUpdate => "PostUpdate",
            ScheduleLabel::Extract => "Extract",
            ScheduleLabel::Render => "Render",
            ScheduleLabel::Custom(name) => name,
        }
    }
}

/// Label trait - For labeling systems and sets
///
/// Allows systems and sets to be referenced by name.
pub trait Label: Clone + PartialEq + core::fmt::Debug {
    /// Get the label name
    fn name(&self) -> &'static str;
}

impl Label for ScheduleLabel {
    fn name(&self) -> &'static str {
        self.name()
    }
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
