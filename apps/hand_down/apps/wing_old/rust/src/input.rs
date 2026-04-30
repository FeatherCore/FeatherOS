//! Input resource types shared by the Wing shell and demo platform code.

extern crate alloc;

use alloc::collections::BTreeSet;
use core::hash::Hash;

use fhre::resources::Resource;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCode {
    Escape,
    Space,
    Backspace,
    Delete,
    Enter,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

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
}

impl<T: Copy + Eq + Ord + Hash + 'static> Default for ButtonInput<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MouseWheel {
    pub delta: i32,
}

impl MouseWheel {
    pub fn clear(&mut self) {
        self.delta = 0;
    }
}

impl Resource for ButtonInput<MouseButton> {}
impl Resource for ButtonInput<KeyCode> {}
impl Resource for MouseWheel {}
