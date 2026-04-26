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
    /// Default long press threshold in seconds
    pub const LONG_PRESS_THRESHOLD: f32 = 0.5;
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
        }
    }

    pub fn end(&mut self) {
        if self.phase == GesturePhase::Started || self.phase == GesturePhase::Updated {
            self.phase = GesturePhase::Ended;
        }
    }

    /// Check if long press is triggered with given move threshold
    pub fn is_long_press_triggered(&self, move_threshold: f32) -> bool {
        self.is_long_press 
            && self.hold_time >= Self::LONG_PRESS_THRESHOLD
            && self.total_delta.length() < move_threshold
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn is_active(&self) -> bool {
        matches!(self.phase, GesturePhase::Started | GesturePhase::Updated)
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
