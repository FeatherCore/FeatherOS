use fhre::Vec2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GesturePhase {
    #[default]
    None,
    Started,
    Updated,
    Ended,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GestureState {
    pub phase: GesturePhase,
    pub start_position: Vec2,
    pub current_position: Vec2,
    pub delta: Vec2,
    pub total_delta: Vec2,
    pub current_velocity: Vec2,
    pub average_velocity: Vec2,
    pub peak_velocity: Vec2,
    pub hold_time: f32,
    pub is_long_press: bool,
    velocity_history: [Vec2; 5],
    velocity_index: usize,
}

impl GestureState {
    pub const SWIPE_THRESHOLD: f32 = 50.0;
    pub const VELOCITY_THRESHOLD: f32 = 200.0;
    pub const MIN_SWIPE_FOR_VELOCITY: f32 = 20.0;
    pub const LONG_PRESS_THRESHOLD: f32 = 0.5;
    pub const LONG_PRESS_MOVE_THRESHOLD: f32 = 15.0;
    const VELOCITY_HISTORY_SIZE: usize = 5;

    pub fn start(&mut self, position: Vec2) {
        self.phase = GesturePhase::Started;
        self.start_position = position;
        self.current_position = position;
        self.delta = Vec2::ZERO;
        self.total_delta = Vec2::ZERO;
        self.current_velocity = Vec2::ZERO;
        self.average_velocity = Vec2::ZERO;
        self.peak_velocity = Vec2::ZERO;
        self.hold_time = 0.0;
        self.is_long_press = false;
        self.velocity_history = [Vec2::ZERO; Self::VELOCITY_HISTORY_SIZE];
        self.velocity_index = 0;
    }

    pub fn update(&mut self, position: Vec2) {
        if self.phase == GesturePhase::Started || self.phase == GesturePhase::Updated {
            let previous = self.current_position;
            self.current_position = position;
            self.delta = position - previous;
            self.total_delta = position - self.start_position;
            self.phase = GesturePhase::Updated;
        }
    }

    pub fn update_velocity(&mut self, delta_seconds: f32) {
        if delta_seconds <= 0.0 {
            return;
        }
        let instant_velocity = Vec2::new(
            self.delta.x / delta_seconds,
            self.delta.y / delta_seconds,
        );
        self.velocity_history[self.velocity_index] = instant_velocity;
        self.velocity_index = (self.velocity_index + 1) % Self::VELOCITY_HISTORY_SIZE;
        let mut count = 0;
        let mut sum_x = 0.0f32;
        let mut sum_y = 0.0f32;
        for vel in &self.velocity_history {
            if vel.x != 0.0 || vel.y != 0.0 {
                sum_x += vel.x;
                sum_y += vel.y;
                count += 1;
            }
        }
        if count > 0 {
            self.average_velocity = Vec2::new(sum_x / count as f32, sum_y / count as f32);
        }
        let speed = self.average_velocity.length();
        if speed > self.peak_velocity.length() {
            self.peak_velocity = self.average_velocity;
        }
        self.current_velocity = instant_velocity;
    }

    pub fn update_hold_time(&mut self, delta_seconds: f32) {
        if self.phase == GesturePhase::Started || self.phase == GesturePhase::Updated {
            self.hold_time += delta_seconds;
            if self.hold_time >= Self::LONG_PRESS_THRESHOLD
                && self.total_delta.length() < Self::LONG_PRESS_MOVE_THRESHOLD
            {
                self.is_long_press = true;
            }
        }
    }

    pub fn end(&mut self) {
        if self.phase == GesturePhase::Started || self.phase == GesturePhase::Updated {
            self.phase = GesturePhase::Ended;
        }
    }

    pub fn is_long_press_triggered(&self) -> bool {
        self.is_long_press && !self.is_swipe()
    }

    pub fn is_swipe(&self) -> bool {
        self.total_delta.length() > Self::LONG_PRESS_MOVE_THRESHOLD * 2.0
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn is_active(&self) -> bool {
        matches!(self.phase, GesturePhase::Started | GesturePhase::Updated)
    }

    pub fn is_horizontal_swipe(&self) -> bool {
        self.total_delta.x.abs() > Self::SWIPE_THRESHOLD
            && self.total_delta.x.abs() > self.total_delta.y.abs() * 2.0
    }

    pub fn is_vertical_swipe(&self) -> bool {
        self.total_delta.y.abs() > Self::SWIPE_THRESHOLD
            && self.total_delta.y.abs() > self.total_delta.x.abs() * 2.0
    }

    pub fn is_fast_swipe(&self, direction: SwipeDirection) -> bool {
        let velocity_threshold = Self::VELOCITY_THRESHOLD;
        let displacement_threshold = Self::MIN_SWIPE_FOR_VELOCITY;
        match direction {
            SwipeDirection::Up => {
                self.average_velocity.y < -velocity_threshold
                    && self.total_delta.y.abs() > displacement_threshold
            }
            SwipeDirection::Down => {
                self.average_velocity.y > velocity_threshold
                    && self.total_delta.y.abs() > displacement_threshold
            }
            SwipeDirection::Left => {
                self.average_velocity.x < -velocity_threshold
                    && self.total_delta.x.abs() > displacement_threshold
            }
            SwipeDirection::Right => {
                self.average_velocity.x > velocity_threshold
                    && self.total_delta.x.abs() > displacement_threshold
            }
        }
    }

    pub fn swipe_direction(&self) -> Option<SwipeDirection> {
        if self.is_horizontal_swipe() {
            if self.total_delta.x > 0.0 {
                Some(SwipeDirection::Right)
            } else {
                Some(SwipeDirection::Left)
            }
        } else if self.is_vertical_swipe() {
            if self.total_delta.y > 0.0 {
                Some(SwipeDirection::Down)
            } else {
                Some(SwipeDirection::Up)
            }
        } else {
            None
        }
    }

    pub fn swipe_direction_with_velocity(&self) -> Option<SwipeDirection> {
        if self.is_horizontal_swipe() || self.is_vertical_swipe() {
            let direction = self.swipe_direction()?;
            if self.is_fast_swipe(direction) || self.meets_displacement_threshold() {
                return Some(direction);
            }
        }
        None
    }

    fn meets_displacement_threshold(&self) -> bool {
        self.total_delta.y.abs() > Self::SWIPE_THRESHOLD * 0.6
            || self.total_delta.x.abs() > Self::SWIPE_THRESHOLD * 0.6
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

impl fhre::resources::Resource for GestureState {}
