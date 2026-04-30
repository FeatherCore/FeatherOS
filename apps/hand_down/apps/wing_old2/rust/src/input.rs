use crate::math::Point;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Escape,
    Enter,
    Backspace,
    Delete,
    Space,
    Home,
    End,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Middle,
    Secondary,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonState {
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Swipe {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub struct InputState {
    pub pointer: Point,
    pub pointer_start: Point,
    pub pointer_delta: Point,
    pub primary: ButtonState,
    pub last_key_pressed: Option<KeyCode>,
    pub last_key_released: Option<KeyCode>,
    pub wheel_delta: i8,
    pub swipe: Swipe,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            pointer: Point::default(),
            pointer_start: Point::default(),
            pointer_delta: Point::default(),
            primary: ButtonState::default(),
            last_key_pressed: None,
            last_key_released: None,
            wheel_delta: 0,
            swipe: Swipe::None,
        }
    }
}

impl InputState {
    pub fn begin_frame(&mut self) {
        self.primary.pressed = false;
        self.primary.released = false;
        self.pointer_delta = Point::default();
        self.last_key_pressed = None;
        self.last_key_released = None;
        self.wheel_delta = 0;
        self.swipe = Swipe::None;
    }

    pub fn move_pointer(&mut self, x: i32, y: i32) {
        self.pointer_delta.x += x - self.pointer.x;
        self.pointer_delta.y += y - self.pointer.y;
        self.pointer = Point::new(x, y);
    }

    pub fn set_primary(&mut self, down: bool, x: i32, y: i32) {
        self.move_pointer(x, y);
        if down && !self.primary.down {
            self.primary.down = true;
            self.primary.pressed = true;
            self.pointer_start = self.pointer;
        } else if !down && self.primary.down {
            self.primary.down = false;
            self.primary.released = true;
            self.classify_swipe();
        }
    }

    pub fn press_key(&mut self, key: KeyCode) {
        self.last_key_pressed = Some(key);
    }

    pub fn release_key(&mut self, key: KeyCode) {
        self.last_key_released = Some(key);
    }

    pub fn has_activity(&self) -> bool {
        self.primary.pressed
            || self.primary.released
            || self.pointer_delta.x != 0
            || self.pointer_delta.y != 0
            || self.last_key_pressed.is_some()
            || self.last_key_released.is_some()
            || self.wheel_delta != 0
            || !matches!(self.swipe, Swipe::None)
    }

    fn classify_swipe(&mut self) {
        let dx = self.pointer.x - self.pointer_start.x;
        let dy = self.pointer.y - self.pointer_start.y;
        let abs_x = dx.abs();
        let abs_y = dy.abs();
        let threshold = 64;

        self.swipe = if abs_y >= threshold && abs_y > abs_x {
            if dy < 0 {
                Swipe::Up
            } else {
                Swipe::Down
            }
        } else if abs_x >= threshold && abs_x > abs_y {
            if dx < 0 {
                Swipe::Left
            } else {
                Swipe::Right
            }
        } else {
            Swipe::None
        };
    }
}
