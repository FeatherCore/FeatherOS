//! Button Input Resource
//!
//! Generic button state management.

use alloc::collections::BTreeSet;
use core::hash::Hash;

#[derive(Clone, Debug)]
pub struct ButtonInput<T: Copy + Eq + Ord + Hash + 'static> {
    pressed: BTreeSet<T>,
    just_pressed: BTreeSet<T>,
    just_released: BTreeSet<T>,
}

impl<T: Copy + Eq + Ord + Hash + 'static> ButtonInput<T> {
    pub fn new() -> Self {
        Self {
            pressed: BTreeSet::new(),
            just_pressed: BTreeSet::new(),
            just_released: BTreeSet::new(),
        }
    }

    pub fn press(&mut self, input: T) {
        if self.pressed.insert(input) {
            self.just_pressed.insert(input);
        }
    }

    pub fn release(&mut self, input: T) {
        if self.pressed.remove(&input) {
            self.just_released.insert(input);
        }
    }

    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }

    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn release_all(&mut self) {
        for input in self.pressed.iter().copied() {
            self.just_released.insert(input);
        }
        self.pressed.clear();
    }
}

impl<T: Copy + Eq + Ord + Hash + 'static> Default for ButtonInput<T> {
    fn default() -> Self {
        Self::new()
    }
}

use fhre::resources::Resource;
use super::MouseButton;
use super::{KeyCode, Key};

impl Resource for ButtonInput<MouseButton> {}
impl Resource for ButtonInput<KeyCode> {}
impl Resource for ButtonInput<Key> {}
