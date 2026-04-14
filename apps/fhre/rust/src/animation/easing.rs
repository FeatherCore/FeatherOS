//! Easing Functions
//!
//! Comprehensive easing functions inspired by LVGL's animation paths.
//! Provides smooth interpolation for animations.

use crate::math::clamp;
use libm::{powf, cosf, sinf, sqrtf};

/// Easing function type alias
pub type EasingFn = fn(f32) -> f32;

/// Linear interpolation (no easing)
pub fn linear(t: f32) -> f32 {
    t
}

/// Ease in - quadratic
pub fn ease_in_quad(t: f32) -> f32 {
    t * t
}

/// Ease out - quadratic
pub fn ease_out_quad(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Ease in-out - quadratic
pub fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - powf(-2.0 * t + 2.0, 2.0) / 2.0
    }
}

/// Ease in - cubic
pub fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

/// Ease out - cubic
pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - powf(1.0 - t, 3.0)
}

/// Ease in-out - cubic
pub fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - powf(-2.0 * t + 2.0, 3.0) / 2.0
    }
}

/// Ease in - quartic
pub fn ease_in_quart(t: f32) -> f32 {
    t * t * t * t
}

/// Ease out - quartic
pub fn ease_out_quart(t: f32) -> f32 {
    1.0 - powf(1.0 - t, 4.0)
}

/// Ease in-out - quartic
pub fn ease_in_out_quart(t: f32) -> f32 {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        1.0 - powf(-2.0 * t + 2.0, 4.0) / 2.0
    }
}

/// Ease in - sine
pub fn ease_in_sine(t: f32) -> f32 {
    1.0 - cosf(t * core::f32::consts::PI / 2.0)
}

/// Ease out - sine
pub fn ease_out_sine(t: f32) -> f32 {
    sinf(t * core::f32::consts::PI / 2.0)
}

/// Ease in-out - sine
pub fn ease_in_out_sine(t: f32) -> f32 {
    -(cosf(core::f32::consts::PI * t) / 2.0) + 0.5
}

/// Ease in - exponential
pub fn ease_in_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        powf(2.0, 10.0 * (t - 1.0))
    }
}

/// Ease out - exponential
pub fn ease_out_expo(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - powf(2.0, -10.0 * t)
    }
}

/// Ease in-out - exponential
pub fn ease_in_out_expo(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        powf(2.0, 20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - powf(2.0, -20.0 * t + 10.0)) / 2.0
    }
}

/// Ease in - circular
pub fn ease_in_circ(t: f32) -> f32 {
    1.0 - sqrtf(1.0 - t * t)
}

/// Ease out - circular
pub fn ease_out_circ(t: f32) -> f32 {
    sqrtf(1.0 - powf(t - 1.0, 2.0))
}

/// Ease in-out - circular
pub fn ease_in_out_circ(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - sqrtf(1.0 - 4.0 * t * t)) / 2.0
    } else {
        (sqrtf(1.0 - powf(-2.0 * t + 2.0, 2.0)) + 1.0) / 2.0
    }
}

/// Ease in - back (overshoot)
pub fn ease_in_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    C3 * t * t * t - C1 * t * t
}

/// Ease out - back (overshoot)
pub fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    1.0 + C3 * powf(t - 1.0, 3.0) + C1 * powf(t - 1.0, 2.0)
}

/// Ease in-out - back (overshoot)
pub fn ease_in_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C2: f32 = C1 * 1.525;
    if t < 0.5 {
        (powf(2.0 * t, 2.0) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
    } else {
        (powf(2.0 * t - 2.0, 2.0) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) + 2.0) / 2.0
    }
}

/// Ease out - bounce
pub fn ease_out_bounce(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    
    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

/// Ease in - bounce
pub fn ease_in_bounce(t: f32) -> f32 {
    1.0 - ease_out_bounce(1.0 - t)
}

/// Ease in-out - bounce
pub fn ease_in_out_bounce(t: f32) -> f32 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

/// Ease out - elastic
pub fn ease_out_elastic(t: f32) -> f32 {
    const C4: f32 = (2.0 * core::f32::consts::PI) / 3.0;
    
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        powf(2.0, -10.0 * t) * sinf((t * 10.0 - 0.75) * C4) + 1.0
    }
}

/// Ease in - elastic
pub fn ease_in_elastic(t: f32) -> f32 {
    const C4: f32 = (2.0 * core::f32::consts::PI) / 3.0;
    
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        -(powf(2.0, 10.0 * t - 10.0)) * sinf((t * 10.0 - 10.75) * C4)
    }
}

/// Step function (no interpolation)
pub fn step(t: f32) -> f32 {
    if t >= 1.0 {
        1.0
    } else {
        0.0
    }
}

/// Cubic bezier interpolation
/// Control points: (0, 0), (x1, y1), (x2, y2), (1, 1)
pub fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // Use Newton-Raphson method to find x for given t
    // Then calculate y
    
    let mut x = t;
    for _ in 0..8 {
        let t2 = x * x;
        let t3 = t2 * x;
        let nt = 1.0 - x;
        let nt2 = nt * nt;
        let nt3 = nt2 * nt;
        
        let current_x = 3.0 * nt2 * x * x1 + 3.0 * nt * t2 * x2 + t3;
        let dx = 3.0 * nt2 * x1 + 6.0 * nt * x * (x2 - x1) + 3.0 * t2 * (1.0 - x2);
        
        if dx.abs() < 0.0001 {
            break;
        }
        
        x = x - (current_x - t) / dx;
        x = clamp(x, 0.0, 1.0);
    }
    
    let t2 = x * x;
    let t3 = t2 * x;
    let nt = 1.0 - x;
    let nt2 = nt * nt;
    let nt3 = nt2 * nt;
    
    3.0 * nt2 * x * y1 + 3.0 * nt * t2 * y2 + t3
}

/// Easing function enumeration
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
    EaseInElastic,
    EaseOutElastic,
    Step,
    Custom(fn(f32) -> f32),
}

impl Default for Easing {
    fn default() -> Self {
        Easing::Linear
    }
}

impl Easing {
    /// Apply easing to a value t in [0, 1]
    pub fn apply(&self, t: f32) -> f32 {
        let t = clamp(t, 0.0, 1.0);
        
        match self {
            Easing::Linear => linear(t),
            Easing::EaseIn => ease_in_quad(t),
            Easing::EaseOut => ease_out_quad(t),
            Easing::EaseInOut => ease_in_out_quad(t),
            Easing::EaseInQuad => ease_in_quad(t),
            Easing::EaseOutQuad => ease_out_quad(t),
            Easing::EaseInOutQuad => ease_in_out_quad(t),
            Easing::EaseInCubic => ease_in_cubic(t),
            Easing::EaseOutCubic => ease_out_cubic(t),
            Easing::EaseInOutCubic => ease_in_out_cubic(t),
            Easing::EaseInQuart => ease_in_quart(t),
            Easing::EaseOutQuart => ease_out_quart(t),
            Easing::EaseInOutQuart => ease_in_out_quart(t),
            Easing::EaseInSine => ease_in_sine(t),
            Easing::EaseOutSine => ease_out_sine(t),
            Easing::EaseInOutSine => ease_in_out_sine(t),
            Easing::EaseInExpo => ease_in_expo(t),
            Easing::EaseOutExpo => ease_out_expo(t),
            Easing::EaseInOutExpo => ease_in_out_expo(t),
            Easing::EaseInCirc => ease_in_circ(t),
            Easing::EaseOutCirc => ease_out_circ(t),
            Easing::EaseInOutCirc => ease_in_out_circ(t),
            Easing::EaseInBack => ease_in_back(t),
            Easing::EaseOutBack => ease_out_back(t),
            Easing::EaseInOutBack => ease_in_out_back(t),
            Easing::EaseInBounce => ease_in_bounce(t),
            Easing::EaseOutBounce => ease_out_bounce(t),
            Easing::EaseInOutBounce => ease_in_out_bounce(t),
            Easing::EaseInElastic => ease_in_elastic(t),
            Easing::EaseOutElastic => ease_out_elastic(t),
            Easing::Step => step(t),
            Easing::Custom(f) => f(t),
        }
    }
    
    /// Get the function pointer for this easing
    pub fn as_fn(&self) -> EasingFn {
        match self {
            Easing::Linear => linear,
            Easing::EaseIn | Easing::EaseInQuad => ease_in_quad,
            Easing::EaseOut | Easing::EaseOutQuad => ease_out_quad,
            Easing::EaseInOut | Easing::EaseInOutQuad => ease_in_out_quad,
            Easing::EaseInCubic => ease_in_cubic,
            Easing::EaseOutCubic => ease_out_cubic,
            Easing::EaseInOutCubic => ease_in_out_cubic,
            Easing::EaseInQuart => ease_in_quart,
            Easing::EaseOutQuart => ease_out_quart,
            Easing::EaseInOutQuart => ease_in_out_quart,
            Easing::EaseInSine => ease_in_sine,
            Easing::EaseOutSine => ease_out_sine,
            Easing::EaseInOutSine => ease_in_out_sine,
            Easing::EaseInExpo => ease_in_expo,
            Easing::EaseOutExpo => ease_out_expo,
            Easing::EaseInOutExpo => ease_in_out_expo,
            Easing::EaseInCirc => ease_in_circ,
            Easing::EaseOutCirc => ease_out_circ,
            Easing::EaseInOutCirc => ease_in_out_circ,
            Easing::EaseInBack => ease_in_back,
            Easing::EaseOutBack => ease_out_back,
            Easing::EaseInOutBack => ease_in_out_back,
            Easing::EaseInBounce => ease_in_bounce,
            Easing::EaseOutBounce => ease_out_bounce,
            Easing::EaseInOutBounce => ease_in_out_bounce,
            Easing::EaseInElastic => ease_in_elastic,
            Easing::EaseOutElastic => ease_out_elastic,
            Easing::Step => step,
            Easing::Custom(f) => *f,
        }
    }
}
