//! Render Target Implementation
//!
//! Abstraction for render targets (framebuffer, texture, etc.)

use crate::math::{Color, Rect};
use alloc::vec::Vec;

/// RenderTarget - Abstraction for rendering destinations
///
/// This trait allows the renderer to work with different
/// types of render targets (framebuffer, texture, etc.)
pub trait RenderTarget {
    /// Get target width
    fn width(&self) -> u32;

    /// Get target height
    fn height(&self) -> u32;

    /// Clear the target
    fn clear(&mut self, color: Color);

    /// Draw a pixel
    fn draw_pixel(&mut self, x: u32, y: u32, color: Color);

    /// Get pixel color
    fn get_pixel(&self, x: u32, y: u32) -> Option<Color>;

    /// Get raw pixel data
    fn data(&self) -> &[u32];

    /// Get mutable raw pixel data
    fn data_mut(&mut self) -> &mut [u32];
}

/// WindowRenderTarget - Render target for window/display
///
/// This is the primary render target that outputs to the screen.
pub struct WindowRenderTarget {
    width: u32,
    height: u32,
    data: Vec<u32>,
}

impl WindowRenderTarget {
    /// Create a new window render target
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            width,
            height,
            data: alloc::vec![0; pixel_count],
        }
    }

    /// Resize the target
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let pixel_count = (width * height) as usize;
        self.data.resize(pixel_count, 0);
    }
}

impl RenderTarget for WindowRenderTarget {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn clear(&mut self, color: Color) {
        let value = color.to_u32();
        for pixel in self.data.iter_mut() {
            *pixel = value;
        }
    }

    fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.data[index] = color.to_u32();
        }
    }

    fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            Some(Color::from_u32(self.data[index]))
        } else {
            None
        }
    }

    fn data(&self) -> &[u32] {
        &self.data
    }

    fn data_mut(&mut self) -> &mut [u32] {
        &mut self.data
    }
}

/// TextureRenderTarget - Render target backed by a texture
///
/// Useful for off-screen rendering and post-processing.
pub struct TextureRenderTarget {
    width: u32,
    height: u32,
    data: Vec<u32>,
}

impl TextureRenderTarget {
    /// Create a new texture render target
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            width,
            height,
            data: alloc::vec![0; pixel_count],
        }
    }

    /// Resize the texture
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let pixel_count = (width * height) as usize;
        self.data.resize(pixel_count, 0);
    }
}

impl RenderTarget for TextureRenderTarget {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn clear(&mut self, color: Color) {
        let value = color.to_u32();
        for pixel in self.data.iter_mut() {
            *pixel = value;
        }
    }

    fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.data[index] = color.to_u32();
        }
    }

    fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            Some(Color::from_u32(self.data[index]))
        } else {
            None
        }
    }

    fn data(&self) -> &[u32] {
        &self.data
    }

    fn data_mut(&mut self) -> &mut [u32] {
        &mut self.data
    }
}
