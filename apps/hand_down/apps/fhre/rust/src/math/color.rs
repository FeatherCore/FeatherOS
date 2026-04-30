//! Color structure with alpha blending support

/// Blend mode for rendering operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Additive,
    Multiply,
    Screen,
}

/// Color structure (RGBA) with linear color space operations
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub const fn from_rgba32(value: u32) -> Self {
        Self {
            a: ((value >> 24) & 0xFF) as u8,
            r: ((value >> 16) & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: (value & 0xFF) as u8,
        }
    }

    pub const fn to_rgba32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn to_xrgb32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn to_bgra32(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn to_u32(self) -> u32 {
        self.to_bgra32()
    }

    pub const fn from_u32(value: u32) -> Self {
        Self::from_rgba32(value)
    }

    pub const fn with_alpha(self, alpha: u8) -> Self {
        Self::new(self.r, self.g, self.b, alpha)
    }

    pub const fn lerp(a: Color, b: Color, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let one_minus_t = 1.0 - t;
        Self::new(
            (a.r as f32 * one_minus_t + b.r as f32 * t) as u8,
            (a.g as f32 * one_minus_t + b.g as f32 * t) as u8,
            (a.b as f32 * one_minus_t + b.b as f32 * t) as u8,
            (a.a as f32 * one_minus_t + b.a as f32 * t) as u8,
        )
    }

    pub fn blend(self, bg: Color, mode: BlendMode) -> Color {
        if self.a == 0 {
            return bg;
        }
        if self.a == 255 && mode == BlendMode::Normal {
            return self;
        }

        match mode {
            BlendMode::Normal => self.blend_normal(bg),
            BlendMode::Additive => self.blend_additive(bg),
            BlendMode::Multiply => self.blend_multiply(bg),
            BlendMode::Screen => self.blend_screen(bg),
        }
    }

    fn blend_normal(self, bg: Color) -> Color {
        let fg_alpha = self.a as u32;
        let bg_alpha = bg.a as u32;
        let out_alpha = fg_alpha + ((255 - fg_alpha) * bg_alpha + 127) / 255;

        if out_alpha == 0 {
            return Color::TRANSPARENT;
        }

        let blend_channel = |fg: u8, bg: u8| -> u8 {
            let fg_val = fg as u32;
            let bg_val = bg as u32;
            ((fg_val * 255 * fg_alpha + bg_val * bg_alpha * (255 - fg_alpha) + 127 * 255) / (out_alpha * 255)) as u8
        };

        Color::new(
            blend_channel(self.r, bg.r),
            blend_channel(self.g, bg.g),
            blend_channel(self.b, bg.b),
            out_alpha as u8,
        )
    }

    fn blend_additive(self, bg: Color) -> Color {
        Color::new(
            self.r.saturating_add((bg.r as u16 * (255 - self.a as u16) / 255) as u8),
            self.g.saturating_add((bg.g as u16 * (255 - self.a as u16) / 255) as u8),
            self.b.saturating_add((bg.b as u16 * (255 - self.a as u16) / 255) as u8),
            self.a.max(bg.a),
        )
    }

    fn blend_multiply(self, bg: Color) -> Color {
        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;

        Color::new(
            ((self.r as u32 * bg.r as u32 * alpha + bg.r as u32 * inv_alpha * 255 + 32640) / 65025) as u8,
            ((self.g as u32 * bg.g as u32 * alpha + bg.g as u32 * inv_alpha * 255 + 32640) / 65025) as u8,
            ((self.b as u32 * bg.b as u32 * alpha + bg.b as u32 * inv_alpha * 255 + 32640) / 65025) as u8,
            self.a,
        )
    }

    fn blend_screen(self, bg: Color) -> Color {
        let alpha = self.a as u32;
        let inv_alpha = 255 - alpha;

        Color::new(
            (((255 - (255 - self.r as u32) * (255 - bg.r as u32) / 255) * alpha + bg.r as u32 * inv_alpha + 127) / 255) as u8,
            (((255 - (255 - self.g as u32) * (255 - bg.g as u32) / 255) * alpha + bg.g as u32 * inv_alpha + 127) / 255) as u8,
            (((255 - (255 - self.b as u32) * (255 - bg.b as u32) / 255) * alpha + bg.b as u32 * inv_alpha + 127) / 255) as u8,
            self.a,
        )
    }

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
    pub const LIGHT_GRAY: Self = Self::rgb(192, 192, 192);
    pub const GRAY: Self = Self::rgb(128, 128, 128);
    pub const ORANGE: Self = Self::rgb(255, 165, 0);
    pub const PURPLE: Self = Self::rgb(128, 0, 128);
    pub const PINK: Self = Self::rgb(255, 192, 203);
}
