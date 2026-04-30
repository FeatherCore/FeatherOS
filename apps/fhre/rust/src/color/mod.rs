#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PixelFormat {
    Rgb565,
    Rgb888,
    Rgba8888,
    Unknown(u8),
}

impl PixelFormat {
    pub const fn from_nuttx(fmt: u8, bpp: u8) -> Self {
        match (fmt, bpp) {
            (11, 16) => Self::Rgb565,
            (_, 16) => Self::Rgb565,
            (_, 24) => Self::Rgb888,
            (_, 32) => Self::Rgba8888,
            _ => Self::Unknown(fmt),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn mix(self, other: Self, amount: u8) -> Self {
        let amount = amount as u16;
        let inv = 255u16.saturating_sub(amount);
        Self::rgba(
            (((self.r as u16 * inv) + (other.r as u16 * amount)) / 255) as u8,
            (((self.g as u16 * inv) + (other.g as u16 * amount)) / 255) as u8,
            (((self.b as u16 * inv) + (other.b as u16 * amount)) / 255) as u8,
            (((self.a as u16 * inv) + (other.a as u16 * amount)) / 255) as u8,
        )
    }

    pub fn over(self, dst: Self) -> Self {
        if self.a == 255 {
            return self;
        }
        if self.a == 0 {
            return dst;
        }

        let a = self.a as u16;
        let inv = 255u16.saturating_sub(a);
        Self::rgb(
            (((self.r as u16 * a) + (dst.r as u16 * inv)) / 255) as u8,
            (((self.g as u16 * a) + (dst.g as u16 * inv)) / 255) as u8,
            (((self.b as u16 * a) + (dst.b as u16 * inv)) / 255) as u8,
        )
    }
}
