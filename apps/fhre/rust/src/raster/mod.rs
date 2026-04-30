use crate::Color;

pub(crate) fn clamp_i32(value: i32, low: i32, high: i32) -> i32 {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

pub(crate) fn rgb565(color: Color) -> u16 {
    let r = (color.r as u16 >> 3) & 0x1f;
    let g = (color.g as u16 >> 2) & 0x3f;
    let b = (color.b as u16 >> 3) & 0x1f;
    (r << 11) | (g << 5) | b
}

pub(crate) fn min3_i32(a: i32, b: i32, c: i32) -> i32 {
    a.min(b).min(c)
}

pub(crate) fn max3_i32(a: i32, b: i32, c: i32) -> i32 {
    a.max(b).max(c)
}

pub(crate) fn abs_i32(value: i32) -> i32 {
    if value < 0 {
        value.saturating_neg()
    } else {
        value
    }
}
