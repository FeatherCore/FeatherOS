//! Texture System
//!
//! Provides texture/image support for rendering.
//! Supports multiple color formats and sampling modes.

use crate::math::{Color, Vec2};
use alloc::vec::Vec;
use libm::floorf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureFormat {
    #[default]
    Rgba32,
    Rgb24,
    Argb32,
    A8,
    L8,
    La16,
    Rgb565,
    Rgba4444,
}

impl TextureFormat {
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            TextureFormat::Rgba32 => 4,
            TextureFormat::Rgb24 => 3,
            TextureFormat::Argb32 => 4,
            TextureFormat::A8 => 1,
            TextureFormat::L8 => 1,
            TextureFormat::La16 => 2,
            TextureFormat::Rgb565 => 2,
            TextureFormat::Rgba4444 => 2,
        }
    }

    pub fn has_alpha(&self) -> bool {
        matches!(
            self,
            TextureFormat::Rgba32
                | TextureFormat::Argb32
                | TextureFormat::A8
                | TextureFormat::La16
                | TextureFormat::Rgba4444
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SamplerFilter {
    #[default]
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SamplerAddress {
    #[default]
    ClampToEdge,
    Repeat,
    Mirror,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Sampler {
    pub min_filter: SamplerFilter,
    pub mag_filter: SamplerFilter,
    pub address_u: SamplerAddress,
    pub address_v: SamplerAddress,
}

impl Sampler {
    pub const NEAREST: Self = Self {
        min_filter: SamplerFilter::Nearest,
        mag_filter: SamplerFilter::Nearest,
        address_u: SamplerAddress::ClampToEdge,
        address_v: SamplerAddress::ClampToEdge,
    };

    pub const LINEAR: Self = Self {
        min_filter: SamplerFilter::Linear,
        mag_filter: SamplerFilter::Linear,
        address_u: SamplerAddress::ClampToEdge,
        address_v: SamplerAddress::ClampToEdge,
    };

    pub const REPEAT: Self = Self {
        min_filter: SamplerFilter::Nearest,
        mag_filter: SamplerFilter::Nearest,
        address_u: SamplerAddress::Repeat,
        address_v: SamplerAddress::Repeat,
    };
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub sampler: Sampler,
}

impl Texture {
    pub fn new(width: u32, height: u32, format: TextureFormat) -> Self {
        let size = (width * height * format.bytes_per_pixel()) as usize;
        Self {
            data: alloc::vec![0; size],
            width,
            height,
            format,
            sampler: Sampler::default(),
        }
    }

    pub fn from_rgba32(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            data,
            width,
            height,
            format: TextureFormat::Rgba32,
            sampler: Sampler::default(),
        }
    }

    pub fn from_rgb24(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            data,
            width,
            height,
            format: TextureFormat::Rgb24,
            sampler: Sampler::default(),
        }
    }

    pub fn with_sampler(mut self, sampler: Sampler) -> Self {
        self.sampler = sampler;
        self
    }

    pub fn sample(&self, uv: Vec2) -> Color {
        self.sample_with_sampler(uv, &self.sampler)
    }

    pub fn sample_with_sampler(&self, uv: Vec2, sampler: &Sampler) -> Color {
        let u = self.wrap_coord(uv.x, sampler.address_u);
        let v = self.wrap_coord(uv.y, sampler.address_v);

        match sampler.mag_filter {
            SamplerFilter::Nearest => self.sample_nearest(u, v),
            SamplerFilter::Linear => self.sample_linear(u, v),
        }
    }

    fn wrap_coord(&self, coord: f32, mode: SamplerAddress) -> f32 {
        match mode {
            SamplerAddress::ClampToEdge => coord.clamp(0.0, 1.0),
            SamplerAddress::Repeat => coord - floorf(coord),
            SamplerAddress::Mirror => {
                let wrapped = coord - floorf(coord);
                if (floorf(coord) as i32) % 2 == 0 {
                    wrapped
                } else {
                    1.0 - wrapped
                }
            }
        }
    }

    fn sample_nearest(&self, u: f32, v: f32) -> Color {
        let x = ((u * self.width as f32).min(self.width as f32 - 0.001) as u32).min(self.width - 1);
        let y = ((v * self.height as f32).min(self.height as f32 - 0.001) as u32).min(self.height - 1);
        self.get_pixel(x, y)
    }

    fn sample_linear(&self, u: f32, v: f32) -> Color {
        let fx = u * self.width as f32 - 0.5;
        let fy = v * self.height as f32 - 0.5;

        let x0 = (floorf(fx) as i32).max(0) as u32;
        let y0 = (floorf(fy) as i32).max(0) as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let dx = (fx - x0 as f32).clamp(0.0, 1.0);
        let dy = (fy - y0 as f32).clamp(0.0, 1.0);

        let c00 = self.get_pixel(x0, y0);
        let c10 = self.get_pixel(x1, y0);
        let c01 = self.get_pixel(x0, y1);
        let c11 = self.get_pixel(x1, y1);

        let c0 = Color::lerp(c00, c10, dx);
        let c1 = Color::lerp(c01, c11, dx);
        Color::lerp(c0, c1, dy)
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x >= self.width || y >= self.height {
            return Color::TRANSPARENT;
        }

        let bpp = self.format.bytes_per_pixel() as usize;
        let index = (y * self.width + x) as usize * bpp;

        if index + bpp > self.data.len() {
            return Color::TRANSPARENT;
        }

        match self.format {
            TextureFormat::Rgba32 => Color::new(
                self.data[index],
                self.data[index + 1],
                self.data[index + 2],
                self.data[index + 3],
            ),
            TextureFormat::Argb32 => Color::new(
                self.data[index + 1],
                self.data[index + 2],
                self.data[index + 3],
                self.data[index],
            ),
            TextureFormat::Rgb24 => Color::new(
                self.data[index],
                self.data[index + 1],
                self.data[index + 2],
                255,
            ),
            TextureFormat::A8 => Color::new(255, 255, 255, self.data[index]),
            TextureFormat::L8 => {
                let l = self.data[index];
                Color::new(l, l, l, 255)
            }
            TextureFormat::La16 => Color::new(
                self.data[index],
                self.data[index],
                self.data[index],
                self.data[index + 1],
            ),
            TextureFormat::Rgb565 => {
                let packed = self.data[index] as u16 | ((self.data[index + 1] as u16) << 8);
                let r = ((packed >> 11) & 0x1F) as u8;
                let g = ((packed >> 5) & 0x3F) as u8;
                let b = (packed & 0x1F) as u8;
                Color::new(
                    (r << 3) | (r >> 2),
                    (g << 2) | (g >> 4),
                    (b << 3) | (b >> 2),
                    255,
                )
            }
            TextureFormat::Rgba4444 => {
                let packed = self.data[index] as u16 | ((self.data[index + 1] as u16) << 8);
                let r = ((packed >> 12) & 0xF) as u8;
                let g = ((packed >> 8) & 0xF) as u8;
                let b = ((packed >> 4) & 0xF) as u8;
                let a = (packed & 0xF) as u8;
                Color::new(
                    (r << 4) | r,
                    (g << 4) | g,
                    (b << 4) | b,
                    (a << 4) | a,
                )
            }
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let bpp = self.format.bytes_per_pixel() as usize;
        let index = (y * self.width + x) as usize * bpp;

        if index + bpp > self.data.len() {
            return;
        }

        match self.format {
            TextureFormat::Rgba32 => {
                self.data[index] = color.r;
                self.data[index + 1] = color.g;
                self.data[index + 2] = color.b;
                self.data[index + 3] = color.a;
            }
            TextureFormat::Argb32 => {
                self.data[index] = color.a;
                self.data[index + 1] = color.r;
                self.data[index + 2] = color.g;
                self.data[index + 3] = color.b;
            }
            TextureFormat::Rgb24 => {
                self.data[index] = color.r;
                self.data[index + 1] = color.g;
                self.data[index + 2] = color.b;
            }
            TextureFormat::A8 => {
                self.data[index] = color.a;
            }
            TextureFormat::L8 => {
                self.data[index] = ((color.r as u32 + color.g as u32 + color.b as u32) / 3) as u8;
            }
            TextureFormat::La16 => {
                let l = ((color.r as u32 + color.g as u32 + color.b as u32) / 3) as u8;
                self.data[index] = l;
                self.data[index + 1] = color.a;
            }
            TextureFormat::Rgb565 => {
                let r = (color.r >> 3) as u16;
                let g = (color.g >> 2) as u16;
                let b = (color.b >> 3) as u16;
                let packed = (r << 11) | (g << 5) | b;
                self.data[index] = (packed & 0xFF) as u8;
                self.data[index + 1] = ((packed >> 8) & 0xFF) as u8;
            }
            TextureFormat::Rgba4444 => {
                let r = (color.r >> 4) as u16;
                let g = (color.g >> 4) as u16;
                let b = (color.b >> 4) as u16;
                let a = (color.a >> 4) as u16;
                let packed = (r << 12) | (g << 8) | (b << 4) | a;
                self.data[index] = (packed & 0xFF) as u8;
                self.data[index + 1] = ((packed >> 8) & 0xFF) as u8;
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TextureRegion {
    pub texture_id: u32,
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
}

impl TextureRegion {
    pub fn full(texture_id: u32) -> Self {
        Self {
            texture_id,
            u0: 0.0,
            v0: 0.0,
            u1: 1.0,
            v1: 1.0,
        }
    }

    pub fn from_rect(texture_id: u32, x: u32, y: u32, width: u32, height: u32, tex_width: u32, tex_height: u32) -> Self {
        Self {
            texture_id,
            u0: x as f32 / tex_width as f32,
            v0: y as f32 / tex_height as f32,
            u1: (x + width) as f32 / tex_width as f32,
            v1: (y + height) as f32 / tex_height as f32,
        }
    }

    pub fn sample_uv(&self, u: f32, v: f32) -> Vec2 {
        Vec2::new(
            self.u0 + u * (self.u1 - self.u0),
            self.v0 + v * (self.v1 - self.v0),
        )
    }
}
