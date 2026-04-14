//! System Set Implementation
//!
//! System sets group related systems together.

use super::label::SystemLabel;
use alloc::vec::Vec;

/// SystemSet - A group of related systems
///
/// Systems can be organized into sets for better structure.
/// Sets can have dependencies on other sets.
#[derive(Clone, Debug)]
pub struct SystemSet {
    /// Name of the set
    name: &'static str,
    /// Labels of systems in this set
    systems: Vec<SystemLabel>,
    /// Sets that must run before this set
    run_after: Vec<&'static str>,
    /// Sets that must run after this set
    run_before: Vec<&'static str>,
}

impl SystemSet {
    /// Create a new system set
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            systems: Vec::new(),
            run_after: Vec::new(),
            run_before: Vec::new(),
        }
    }

    /// Add a system to this set
    pub fn with_system(mut self, label: SystemLabel) -> Self {
        self.systems.push(label);
        self
    }

    /// Specify that this set runs after another set
    pub fn after(mut self, set_name: &'static str) -> Self {
        self.run_after.push(set_name);
        self
    }

    /// Specify that this set runs before another set
    pub fn before(mut self, set_name: &'static str) -> Self {
        self.run_before.push(set_name);
        self
    }

    /// Get the set name
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Get systems in this set
    pub fn systems(&self) -> &[SystemLabel] {
        &self.systems
    }

    /// Get sets that must run before this set
    pub fn after_sets(&self) -> &[&'static str] {
        &self.run_after
    }

    /// Get sets that must run after this set
    pub fn before_sets(&self) -> &[&'static str] {
        &self.run_before
    }

    /// Check if this set contains a system
    pub fn contains(&self, label: &SystemLabel) -> bool {
        self.systems.contains(label)
    }
}

impl Default for SystemSet {
    fn default() -> Self {
        Self::new("default")
    }
}

/// SystemSetConfig - Configuration for system sets
///
/// Used to configure system sets in the app builder.
#[derive(Clone, Debug)]
pub struct SystemSetConfig {
    /// The set being configured
    pub set: SystemSet,
    /// Whether the set is active
    pub active: bool,
}

impl SystemSetConfig {
    /// Create a new config for a set
    pub fn new(set: SystemSet) -> Self {
        Self { set, active: true }
    }

    /// Disable the set
    pub fn disabled(mut self) -> Self {
        self.active = false;
        self
    }

    /// Enable the set
    pub fn enabled(mut self) -> Self {
        self.active = true;
        self
    }
}
