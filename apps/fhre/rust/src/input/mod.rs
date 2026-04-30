use crate::{event::EventQueue, Point};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointerId(pub u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointerEvent {
    pub id: PointerId,
    pub phase: PointerPhase,
    pub position: Point,
    pub delta: Point,
    pub timestamp_us: u64,
}

impl PointerEvent {
    pub const fn new(
        id: PointerId,
        phase: PointerPhase,
        position: Point,
        timestamp_us: u64,
    ) -> Self {
        Self {
            id,
            phase,
            position,
            delta: Point::new(0, 0),
            timestamp_us,
        }
    }

    pub const fn with_delta(mut self, delta: Point) -> Self {
        self.delta = delta;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyCode {
    Back,
    Home,
    Select,
    Up,
    Down,
    Left,
    Right,
    Escape,
    Enter,
    Char(u32),
    Unknown(u16),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub pressed: bool,
    pub repeat: bool,
    pub timestamp_us: u64,
}

impl KeyEvent {
    pub const fn new(code: KeyCode, pressed: bool, timestamp_us: u64) -> Self {
        Self {
            code,
            pressed,
            repeat: false,
            timestamp_us,
        }
    }

    pub const fn repeated(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputEvent {
    Pointer(PointerEvent),
    Key(KeyEvent),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GestureDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GestureKind {
    Tap,
    Swipe(GestureDirection),
    DragStart,
    Drag,
    DragEnd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GestureEvent {
    pub id: PointerId,
    pub kind: GestureKind,
    pub start: Point,
    pub current: Point,
    pub delta: Point,
    pub duration_us: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GestureConfig {
    pub tap_slop: u16,
    pub swipe_threshold: u16,
    pub drag_threshold: u16,
    pub max_tap_us: u32,
}

impl GestureConfig {
    pub const DEFAULT: Self = Self {
        tap_slop: 12,
        swipe_threshold: 48,
        drag_threshold: 16,
        max_tap_us: 350_000,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GestureRecognizer {
    pub config: GestureConfig,
    active_id: Option<PointerId>,
    start: Point,
    last: Point,
    start_us: u64,
    dragging: bool,
}

impl GestureRecognizer {
    pub const fn new() -> Self {
        Self::with_config(GestureConfig::DEFAULT)
    }

    pub const fn with_config(config: GestureConfig) -> Self {
        Self {
            config,
            active_id: None,
            start: Point::new(0, 0),
            last: Point::new(0, 0),
            start_us: 0,
            dragging: false,
        }
    }

    pub fn reset(&mut self) {
        self.active_id = None;
        self.start = Point::new(0, 0);
        self.last = Point::new(0, 0);
        self.start_us = 0;
        self.dragging = false;
    }

    pub fn update(&mut self, event: PointerEvent) -> Option<GestureEvent> {
        match event.phase {
            PointerPhase::Down => {
                self.active_id = Some(event.id);
                self.start = event.position;
                self.last = event.position;
                self.start_us = event.timestamp_us;
                self.dragging = false;
                None
            }
            PointerPhase::Move => {
                if self.active_id != Some(event.id) {
                    return None;
                }

                self.last = event.position;
                let delta = point_delta(self.start, event.position);
                let max_axis = abs_i32(delta.x).max(abs_i32(delta.y));
                if !self.dragging && max_axis >= self.config.drag_threshold as i32 {
                    self.dragging = true;
                    return Some(self.event(event, GestureKind::DragStart));
                }

                if self.dragging {
                    Some(self.event(event, GestureKind::Drag))
                } else {
                    None
                }
            }
            PointerPhase::Up => {
                if self.active_id != Some(event.id) {
                    self.reset();
                    return None;
                }

                let delta = point_delta(self.start, event.position);
                let duration = event.timestamp_us.saturating_sub(self.start_us);
                let kind = classify_gesture(delta, duration, self.dragging, self.config);
                let result = Some(self.event(event, kind));
                self.reset();
                result
            }
            PointerPhase::Cancel => {
                self.reset();
                None
            }
        }
    }

    fn event(&self, pointer: PointerEvent, kind: GestureKind) -> GestureEvent {
        GestureEvent {
            id: pointer.id,
            kind,
            start: self.start,
            current: pointer.position,
            delta: point_delta(self.start, pointer.position),
            duration_us: pointer.timestamp_us.saturating_sub(self.start_us),
        }
    }
}

pub type InputQueue<const N: usize> = EventQueue<InputEvent, N>;

fn point_delta(start: Point, current: Point) -> Point {
    Point::new(
        current.x.saturating_sub(start.x),
        current.y.saturating_sub(start.y),
    )
}

fn classify_gesture(
    delta: Point,
    duration_us: u64,
    dragging: bool,
    config: GestureConfig,
) -> GestureKind {
    let ax = abs_i32(delta.x);
    let ay = abs_i32(delta.y);
    let max_axis = ax.max(ay);

    if !dragging && max_axis <= config.tap_slop as i32 && duration_us <= config.max_tap_us as u64 {
        return GestureKind::Tap;
    }

    if max_axis >= config.swipe_threshold as i32 {
        if ax >= ay {
            if delta.x < 0 {
                GestureKind::Swipe(GestureDirection::Left)
            } else {
                GestureKind::Swipe(GestureDirection::Right)
            }
        } else if delta.y < 0 {
            GestureKind::Swipe(GestureDirection::Up)
        } else {
            GestureKind::Swipe(GestureDirection::Down)
        }
    } else {
        GestureKind::DragEnd
    }
}

fn abs_i32(value: i32) -> i32 {
    if value < 0 {
        value.saturating_neg()
    } else {
        value
    }
}
