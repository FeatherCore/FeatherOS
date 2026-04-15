//! Color structure

/// Color structure (RGBA)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create from RGB (alpha = 255)
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Convert to RGBA32 format (ARGB with Alpha in highest byte)
    pub const fn to_rgba32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Convert to XRGB32 format for NuttX framebuffer (no Alpha, X in highest byte)
    pub const fn to_xrgb32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Convert to BGRA32 format for X11 (Blue, Green, Red, Alpha - little endian)
    pub const fn to_bgra32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Convert to u32 format for framebuffer (BGRA32 for X11)
    pub const fn to_u32(self) -> u32 {
        self.to_bgra32()
    }

    /// Create from u32 RGBA value
    pub const fn from_u32(value: u32) -> Self {
        Self {
            a: ((value >> 24) & 0xFF) as u8,
            r: ((value >> 16) & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: (value & 0xFF) as u8,
        }
    }

    /// Predefined colors
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    pub const CYAN: Self = Self::rgb(0, 255, 255);
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const DARK_GRAY: Self = Self::rgb(64, 64, 64);

    /// Create a color with modified alpha
    pub const fn with_alpha(self, alpha: u8) -> Self {
        Self::new(self.r, self.g, self.b, alpha)
    }
}
