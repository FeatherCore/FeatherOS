pub const DEFAULT_TRANSITION_FRAMES: u8 = 6;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameTrack {
    pub frame: u8,
    pub duration: u8,
}

impl FrameTrack {
    pub const fn idle() -> Self {
        Self {
            frame: 0,
            duration: 0,
        }
    }

    pub const fn start(duration: u8) -> Self {
        Self { frame: 0, duration }
    }

    pub fn active(self) -> bool {
        self.duration != 0
    }

    pub fn advance(&mut self) {
        if !self.active() {
            return;
        }

        self.frame = self.frame.saturating_add(1);
        if self.frame >= self.duration {
            *self = Self::idle();
        }
    }

    pub fn remaining(self) -> u8 {
        if self.active() {
            self.duration.saturating_sub(self.frame)
        } else {
            0
        }
    }

    pub fn progress_255(self) -> u8 {
        if !self.active() {
            return 255;
        }

        ((self.frame as u16 * 255) / self.duration.max(1) as u16) as u8
    }

    pub fn scale_remaining_i32(self, value: i32) -> i32 {
        if !self.active() {
            return 0;
        }

        value * self.remaining() as i32 / self.duration.max(1) as i32
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlideDirection {
    #[default]
    None,
    FromTop,
    FromBottom,
    FromLeft,
    FromRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlideTransition {
    pub direction: SlideDirection,
    pub track: FrameTrack,
}

impl SlideTransition {
    pub const fn none() -> Self {
        Self {
            direction: SlideDirection::None,
            track: FrameTrack::idle(),
        }
    }

    pub const fn enter(direction: SlideDirection) -> Self {
        match direction {
            SlideDirection::None => Self::none(),
            _ => Self {
                direction,
                track: FrameTrack::start(DEFAULT_TRANSITION_FRAMES),
            },
        }
    }

    pub fn active(self) -> bool {
        !matches!(self.direction, SlideDirection::None) && self.track.active()
    }

    pub fn advance(&mut self) {
        if !self.active() {
            return;
        }

        self.track.advance();
        if !self.track.active() {
            *self = Self::none();
        }
    }

    pub fn remaining(self) -> u8 {
        if !self.active() {
            0
        } else {
            self.track.remaining()
        }
    }

    pub fn offset(self, width: u16, height: u16) -> (i32, i32) {
        if !self.active() {
            return (0, 0);
        }

        let dx = self.track.scale_remaining_i32(width as i32);
        let dy = self.track.scale_remaining_i32(height as i32);

        match self.direction {
            SlideDirection::None => (0, 0),
            SlideDirection::FromTop => (0, -dy),
            SlideDirection::FromBottom => (0, dy),
            SlideDirection::FromLeft => (-dx, 0),
            SlideDirection::FromRight => (dx, 0),
        }
    }

    pub fn alpha(self) -> u8 {
        if self.active() {
            self.track.progress_255()
        } else {
            255
        }
    }
}

impl Default for SlideTransition {
    fn default() -> Self {
        Self::none()
    }
}
