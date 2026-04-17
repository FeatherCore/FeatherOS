//! Schedule Implementation
//!
//! Schedules manage the execution order of systems.
//! Fully aligned with Bevy's schedule system.

use super::label::{ScheduleLabel, InternedScheduleLabel};
use super::set::SystemSet;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;

/// Schedule - A collection of systems with execution order
///
/// Schedules define when systems run relative to each other.
pub struct Schedule {
    /// Name of the schedule
    name: String,
    /// Systems in this schedule
    systems: Vec<Box<dyn FnMut()>>,
    /// System sets for organization
    sets: Vec<SystemSet>,
    /// Labels for systems
    labels: BTreeMap<String, usize>,
}

impl Schedule {
    /// Create a new schedule
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
            sets: Vec::new(),
            labels: BTreeMap::new(),
        }
    }

    /// Add a system to the schedule
    pub fn add_system<F>(&mut self, system: F) -> &mut Self
    where
        F: FnMut() + 'static,
    {
        self.systems.push(Box::new(system));
        self
    }

    /// Add a system with a label
    pub fn add_system_with_label<F>(&mut self, system: F, label: impl Into<String>) -> &mut Self
    where
        F: FnMut() + 'static,
    {
        let index = self.systems.len();
        self.systems.push(Box::new(system));
        self.labels.insert(label.into(), index);
        self
    }

    /// Add a system set
    pub fn add_set(&mut self, set: SystemSet) -> &mut Self {
        self.sets.push(set);
        self
    }

    /// Run all systems in the schedule
    pub fn run(&mut self) {
        for system in self.systems.iter_mut() {
            system();
        }
    }

    /// Get the schedule name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Clear all systems
    pub fn clear(&mut self) {
        self.systems.clear();
        self.labels.clear();
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Schedules - Container for multiple schedules
///
/// Manages multiple schedules like PreUpdate, Update, PostUpdate, etc.
pub struct Schedules {
    /// Map of schedule labels to schedules
    schedules: BTreeMap<String, Schedule>,
    /// Default schedule order
    order: Vec<String>,
}

impl Schedules {
    /// Create a new schedules container with default schedules
    pub fn new() -> Self {
        let mut schedules = BTreeMap::new();

        // Create default schedules
        schedules.insert(String::from("PreUpdate"), Schedule::new("PreUpdate"));
        schedules.insert(String::from("Update"), Schedule::new("Update"));
        schedules.insert(String::from("PostUpdate"), Schedule::new("PostUpdate"));
        schedules.insert(String::from("Extract"), Schedule::new("Extract"));
        schedules.insert(String::from("Render"), Schedule::new("Render"));

        let order = alloc::vec![
            String::from("PreUpdate"),
            String::from("Update"),
            String::from("PostUpdate"),
            String::from("Extract"),
            String::from("Render"),
        ];

        Self { schedules, order }
    }

    /// Get a schedule by label
    pub fn get(&self, label: impl ScheduleLabel) -> Option<&Schedule> {
        self.schedules.get(&label.name())
    }

    /// Get a mutable schedule by label
    pub fn get_mut(&mut self, label: impl ScheduleLabel) -> Option<&mut Schedule> {
        self.schedules.get_mut(&label.name())
    }

    /// Add a custom schedule
    pub fn add(&mut self, label: impl ScheduleLabel, schedule: Schedule) -> &mut Self {
        self.schedules.insert(label.name(), schedule);
        self
    }

    /// Run all schedules in order
    pub fn run_all(&mut self) {
        for label in &self.order {
            if let Some(schedule) = self.schedules.get_mut(label) {
                schedule.run();
            }
        }
    }

    /// Run a specific schedule
    pub fn run(&mut self, label: impl ScheduleLabel) {
        let label_name = label.name();
        if let Some(schedule) = self.schedules.get_mut(&label_name) {
            schedule.run();
        }
    }

    /// Set the execution order
    pub fn set_order(&mut self, order: Vec<String>) {
        self.order = order;
    }
}

impl Default for Schedules {
    fn default() -> Self {
        Self::new()
    }
}
