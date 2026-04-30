pub type Fixed16 = i32;

pub const FIXED_ONE: Fixed16 = 1 << 16;

pub const fn fixed_from_i32(value: i32) -> Fixed16 {
    value.saturating_mul(FIXED_ONE)
}

pub const fn fixed_to_i32(value: Fixed16) -> i32 {
    value / FIXED_ONE
}

pub fn fixed_mul(a: Fixed16, b: Fixed16) -> Fixed16 {
    ((a as i64 * b as i64) / FIXED_ONE as i64) as Fixed16
}

pub fn fixed_div(a: Fixed16, b: Fixed16) -> Fixed16 {
    if b == 0 {
        0
    } else {
        (((a as i64) * FIXED_ONE as i64) / b as i64) as Fixed16
    }
}

pub fn fixed_lerp(a: Fixed16, b: Fixed16, t: Fixed16) -> Fixed16 {
    a.saturating_add(fixed_mul(b.saturating_sub(a), t))
}
