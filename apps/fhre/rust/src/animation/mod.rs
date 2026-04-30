use crate::{fixed_from_i32, fixed_lerp, fixed_mul, Fixed16, FIXED_ONE};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Easing {
    pub fn apply(self, t: Fixed16) -> Fixed16 {
        let t = clamp_fixed01(t);
        match self {
            Self::Linear => t,
            Self::EaseIn => fixed_mul(t, t),
            Self::EaseOut => {
                let inv = FIXED_ONE.saturating_sub(t);
                FIXED_ONE.saturating_sub(fixed_mul(inv, inv))
            }
            Self::EaseInOut => {
                if t < FIXED_ONE / 2 {
                    fixed_mul(t, t).saturating_mul(2)
                } else {
                    let inv = FIXED_ONE.saturating_sub(t);
                    FIXED_ONE.saturating_sub(fixed_mul(inv, inv).saturating_mul(2))
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tween {
    pub start: Fixed16,
    pub end: Fixed16,
    pub duration_us: u32,
    pub delay_us: u32,
    pub repeat: bool,
    pub ping_pong: bool,
    pub easing: Easing,
}

impl Tween {
    pub const fn new(start: Fixed16, end: Fixed16, duration_us: u32) -> Self {
        Self {
            start,
            end,
            duration_us,
            delay_us: 0,
            repeat: false,
            ping_pong: false,
            easing: Easing::Linear,
        }
    }

    pub const fn from_i32(start: i32, end: i32, duration_us: u32) -> Self {
        Self::new(fixed_from_i32(start), fixed_from_i32(end), duration_us)
    }

    pub const fn with_delay(mut self, delay_us: u32) -> Self {
        self.delay_us = delay_us;
        self
    }

    pub const fn with_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    pub const fn with_ping_pong(mut self, ping_pong: bool) -> Self {
        self.ping_pong = ping_pong;
        self
    }

    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn sample(self, elapsed_us: u64) -> Fixed16 {
        if elapsed_us <= self.delay_us as u64 {
            return self.start;
        }
        if self.duration_us == 0 {
            return self.end;
        }

        let active = elapsed_us.saturating_sub(self.delay_us as u64);
        let duration = self.duration_us as u64;
        let mut cycle = active / duration;
        let mut in_cycle = active % duration;

        if !self.repeat && active >= duration {
            cycle = 0;
            in_cycle = duration;
        }

        let mut t = ((in_cycle as u64).saturating_mul(FIXED_ONE as u64) / duration) as Fixed16;
        if self.ping_pong && (cycle & 1) == 1 {
            t = FIXED_ONE.saturating_sub(t);
        }

        fixed_lerp(self.start, self.end, self.easing.apply(t))
    }
}

fn clamp_fixed01(value: Fixed16) -> Fixed16 {
    if value < 0 {
        0
    } else if value > FIXED_ONE {
        FIXED_ONE
    } else {
        value
    }
}
