//! Keyboard Input Types
//!
//! Keyboard key codes and events.

use crate::resources::Resource;

/// Physical key code - represents the physical position of a key
/// Independent of keyboard layout
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyCode {
    // Letters
    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ,
    KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT,
    KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,

    // Numbers (top row)
    Digit0, Digit1, Digit2, Digit3, Digit4,
    Digit5, Digit6, Digit7, Digit8, Digit9,

    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide,
    NumpadEnter, NumpadDecimal, NumpadEqual,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10,
    F11, F12, F13, F14, F15, F16, F17, F18, F19, F20,
    F21, F22, F23, F24, F25, F26, F27, F28, F29, F30,
    F31, F32, F33, F34, F35,

    // Modifiers
    ShiftLeft, ShiftRight,
    ControlLeft, ControlRight,
    AltLeft, AltRight,
    SuperLeft, SuperRight,

    // Special keys
    Escape,
    Space,
    Enter,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home, End,
    PageUp, PageDown,

    // Arrow keys
    ArrowLeft, ArrowRight, ArrowUp, ArrowDown,

    // Punctuation
    Minus, Equal,
    BracketLeft, BracketRight,
    Semicolon, Quote,
    Comma, Period, Slash, Backslash,
    Backquote,

    // Lock keys
    CapsLock,
    NumLock,
    ScrollLock,

    // Media keys
    Pause,
    PrintScreen,

    // Unidentified
    Unidentified,
}

/// Logical key - represents the character produced by a key
/// Dependent on keyboard layout
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    /// Character key (stores single char)
    Character(char),
    /// Enter/Return key
    Enter,
    /// Tab key
    Tab,
    /// Space key
    Space,
    /// Backspace key
    Backspace,
    /// Escape key
    Escape,
    /// Delete key
    Delete,
    /// Arrow keys
    ArrowLeft, ArrowRight, ArrowUp, ArrowDown,
    /// Home/End
    Home, End,
    /// Page Up/Down
    PageUp, PageDown,
    /// Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10,
    F11, F12, F13, F14, F15, F16, F17, F18, F19, F20,
    F21, F22, F23, F24, F25, F26, F27, F28, F29, F30,
    F31, F32, F33, F34, F35,
    /// Modifier keys
    Shift, Control, Alt, Super,
    /// Unidentified key
    Unidentified,
}

impl Key {
    /// Create a character key from a char
    pub fn from_char(c: char) -> Self {
        Key::Character(c)
    }

    /// Check if this is a character key
    pub fn is_character(&self) -> bool {
        matches!(self, Key::Character(_))
    }

    /// Get the character if this is a character key
    pub fn as_char(&self) -> Option<char> {
        match self {
            Key::Character(c) => Some(*c),
            _ => None,
        }
    }
}

/// Button state for keyboard
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Keyboard input event
#[derive(Clone, Debug)]
pub struct KeyboardInput {
    pub key_code: KeyCode,
    pub logical_key: Key,
    pub state: KeyState,
    pub repeat: bool,
}

impl crate::event::Event for KeyboardInput {}
unsafe impl Send for KeyboardInput {}
unsafe impl Sync for KeyboardInput {}

/// Event sent when window loses keyboard focus
#[derive(Clone, Debug)]
pub struct KeyboardFocusLost;

impl Resource for KeyboardFocusLost {}
