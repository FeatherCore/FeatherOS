//! Button Input Resource
//!
//! Generic button state management inspired by Bevy's ButtonInput<T>.
//! Tracks pressed, just_pressed, and just_released states.

use alloc::collections::BTreeSet;
use core::hash::Hash;

/// ButtonInput - Tracks the state of buttons/keys
///
/// Generic over the button type (MouseButton, KeyCode, etc.)
#[derive(Clone, Debug)]
pub struct ButtonInput<T: Copy + Eq + Ord + Hash + 'static> {
    /// Currently pressed buttons
    pressed: BTreeSet<T>,
    /// Buttons pressed this frame
    just_pressed: BTreeSet<T>,
    /// Buttons released this frame
    just_released: BTreeSet<T>,
}

impl<T: Copy + Eq + Ord + Hash + 'static> ButtonInput<T> {
    /// Create a new empty ButtonInput
    pub fn new() -> Self {
        Self {
            pressed: BTreeSet::new(),
            just_pressed: BTreeSet::new(),
            just_released: BTreeSet::new(),
        }
    }

    /// Register a button as pressed
    pub fn press(&mut self, input: T) {
        if self.pressed.insert(input) {
            // Only add to just_pressed if it wasn't already pressed
            self.just_pressed.insert(input);
        }
    }

    /// Register a button as released
    pub fn release(&mut self, input: T) {
        if self.pressed.remove(&input) {
            // Only add to just_released if it was pressed
            self.just_released.insert(input);
        }
    }

    /// Check if a button is currently pressed
    pub fn pressed(&self, input: T) -> bool {
        self.pressed.contains(&input)
    }

    /// Check if a button was pressed this frame
    pub fn just_pressed(&self, input: T) -> bool {
        self.just_pressed.contains(&input)
    }

    /// Check if a button was released this frame
    pub fn just_released(&self, input: T) -> bool {
        self.just_released.contains(&input)
    }

    /// Check if any of the given buttons are pressed
    pub fn any_pressed(&self, inputs: impl IntoIterator<Item = T>) -> bool {
        inputs.into_iter().any(|input| self.pressed(input))
    }

    /// Check if all of the given buttons are pressed
    pub fn all_pressed(&self, inputs: impl IntoIterator<Item = T>) -> bool {
        inputs.into_iter().all(|input| self.pressed(input))
    }

    /// Get all currently pressed buttons
    pub fn get_pressed(&self) -> impl Iterator<Item = &T> {
        self.pressed.iter()
    }

    /// Get all buttons pressed this frame
    pub fn get_just_pressed(&self) -> impl Iterator<Item = &T> {
        self.just_pressed.iter()
    }

    /// Get all buttons released this frame
    pub fn get_just_released(&self) -> impl Iterator<Item = &T> {
        self.just_released.iter()
    }

    /// Clear just_pressed and just_released (call at start of frame)
    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Release all buttons (e.g., when window loses focus)
    pub fn release_all(&mut self) {
        // Move all pressed buttons to just_released
        for input in self.pressed.iter().copied() {
            self.just_released.insert(input);
        }
        self.pressed.clear();
    }

    /// Reset all state
    pub fn reset_all(&mut self) {
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Check if any buttons are pressed
    pub fn any_pressed_button(&self) -> bool {
        !self.pressed.is_empty()
    }

    /// Get the number of pressed buttons
    pub fn pressed_count(&self) -> usize {
        self.pressed.len()
    }
}

impl<T: Copy + Eq + Ord + Hash + 'static> Default for ButtonInput<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Resource trait for ButtonInput with specific types
use crate::resources::Resource;
use super::mouse::MouseButton;
use super::keyboard::{KeyCode, Key};

impl Resource for ButtonInput<MouseButton> {}
impl Resource for ButtonInput<KeyCode> {}
impl Resource for ButtonInput<Key> {}
