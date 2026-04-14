//! Schedule Implementation
//!
//! Schedules manage the execution order of systems.
//! They provide a way to organize systems into phases.

use super::label::ScheduleLabel;
use super::set::SystemSet;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;

/// Schedule - A collection of systems with execution order
///
/// Schedules define when systems run relative to each other.
pub struct Schedule {
    /// Name of the schedule
    name: &'static str,
    /// Systems in this schedule
    systems: Vec<Box<dyn FnMut()>>,
    /// System sets for organization
    sets: Vec<SystemSet>,
    /// Labels for systems
    labels: BTreeMap<&'static str, usize>,
}

impl Schedule {
    /// Create a new schedule
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
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
    pub fn add_system_with_label<F>(&mut self, system: F, label: &'static str) -> &mut Self
    where
        F: FnMut() + 'static,
    {
        let index = self.systems.len();
        self.systems.push(Box::new(system));
        self.labels.insert(label, index);
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
    pub fn name(&self) -> &'static str {
        self.name
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
    schedules: BTreeMap<ScheduleLabel, Schedule>,
    /// Default schedule order
    order: Vec<ScheduleLabel>,
}

impl Schedules {
    /// Create a new schedules container with default schedules
    pub fn new() -> Self {
        let mut schedules = BTreeMap::new();
        
        // Create default schedules
        schedules.insert(ScheduleLabel::PreUpdate, Schedule::new("PreUpdate"));
        schedules.insert(ScheduleLabel::Update, Schedule::new("Update"));
        schedules.insert(ScheduleLabel::PostUpdate, Schedule::new("PostUpdate"));
        schedules.insert(ScheduleLabel::Extract, Schedule::new("Extract"));
        schedules.insert(ScheduleLabel::Render, Schedule::new("Render"));
        
        let order = alloc::vec![
            ScheduleLabel::PreUpdate,
            ScheduleLabel::Update,
            ScheduleLabel::PostUpdate,
            ScheduleLabel::Extract,
            ScheduleLabel::Render,
        ];
        
        Self { schedules, order }
    }

    /// Get a schedule by label
    pub fn get(&self, label: ScheduleLabel) -> Option<&Schedule> {
        self.schedules.get(&label)
    }

    /// Get a mutable schedule by label
    pub fn get_mut(&mut self, label: ScheduleLabel) -> Option<&mut Schedule> {
        self.schedules.get_mut(&label)
    }

    /// Add a custom schedule
    pub fn add(&mut self, label: ScheduleLabel, schedule: Schedule) -> &mut Self {
        self.schedules.insert(label, schedule);
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
    pub fn run(&mut self, label: ScheduleLabel) {
        if let Some(schedule) = self.schedules.get_mut(&label) {
            schedule.run();
        }
    }

    /// Set the execution order
    pub fn set_order(&mut self, order: Vec<ScheduleLabel>) {
        self.order = order;
    }
}

impl Default for Schedules {
    fn default() -> Self {
        Self::new()
    }
}
