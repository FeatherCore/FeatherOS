//! Framebuffer Implementation
//!
//! Software framebuffer for CPU rendering.

use crate::math::{Color, Rect};
use alloc::vec::Vec;

/// Framebuffer - Software rendering target
///
/// A simple CPU-side framebuffer with basic drawing operations.
pub struct Framebuffer {
    /// Width in pixels
    width: u32,
    /// Height in pixels
    height: u32,
    /// Pixel data (RGBA format)
    data: Vec<u32>,
}

impl Framebuffer {
    /// Create a new framebuffer with specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height) as usize;
        Self {
            width,
            height,
            data: alloc::vec![0; pixel_count],
        }
    }

    /// Get framebuffer width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get framebuffer height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get raw pixel data
    pub fn data(&self) -> &[u32] {
        &self.data
    }

    /// Get mutable raw pixel data
    pub fn data_mut(&mut self) -> &mut [u32] {
        &mut self.data
    }

    /// Clear the framebuffer with a color
    pub fn clear(&mut self, color: Color) {
        let value = color.to_u32();
        for pixel in self.data.iter_mut() {
            *pixel = value;
        }
    }

    /// Clear the framebuffer with a raw color value
    pub fn clear_raw(&mut self, color: u32) {
        for pixel in self.data.iter_mut() {
            *pixel = color;
        }
    }

    /// Draw a single pixel
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.data[index] = color.to_u32();
        }
    }

    /// Draw a single pixel with raw color value
    pub fn draw_pixel_raw(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            self.data[index] = color;
        }
    }

    /// Get pixel color at position
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            Some(Color::from_u32(self.data[index]))
        } else {
            None
        }
    }

    /// Get raw pixel value at position
    pub fn get_pixel_raw(&self, x: u32, y: u32) -> Option<u32> {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            Some(self.data[index])
        } else {
            None
        }
    }

    /// Draw a filled rectangle
    pub fn draw_rect(&mut self, rect: Rect, color: Color) {
        let x0 = rect.x.max(0.0) as u32;
        let y0 = rect.y.max(0.0) as u32;
        let x1 = (rect.x + rect.width).min(self.width as f32) as u32;
        let y1 = (rect.y + rect.height).min(self.height as f32) as u32;

        let color_value = color.to_u32();
        for y in y0..y1 {
            for x in x0..x1 {
                let index = (y * self.width + x) as usize;
                self.data[index] = color_value;
            }
        }
    }

    /// Draw a rectangle outline
    pub fn draw_rect_outline(&mut self, rect: Rect, color: Color, thickness: u32) {
        let x0 = rect.x.max(0.0) as u32;
        let y0 = rect.y.max(0.0) as u32;
        let x1 = (rect.x + rect.width).min(self.width as f32) as u32;
        let y1 = (rect.y + rect.height).min(self.height as f32) as u32;

        let color_value = color.to_u32();

        // Top and bottom edges
        for t in 0..thickness {
            if y0 + t < self.height {
                for x in x0..x1 {
                    let index = ((y0 + t) * self.width + x) as usize;
                    self.data[index] = color_value;
                }
            }
            if y1 > t && y1 - t - 1 < self.height {
                for x in x0..x1 {
                    let index = ((y1 - t - 1) * self.width + x) as usize;
                    self.data[index] = color_value;
                }
            }
        }

        // Left and right edges
        for t in 0..thickness {
            if x0 + t < self.width {
                for y in y0..y1 {
                    let index = (y * self.width + x0 + t) as usize;
                    self.data[index] = color_value;
                }
            }
            if x1 > t && x1 - t - 1 < self.width {
                for y in y0..y1 {
                    let index = (y * self.width + x1 - t - 1) as usize;
                    self.data[index] = color_value;
                }
            }
        }
    }

    /// Draw a horizontal line
    pub fn draw_hline(&mut self, x: i32, y: i32, width: i32, color: Color) {
        if y < 0 || y >= self.height as i32 {
            return;
        }

        let x0 = x.max(0);
        let x1 = (x + width).min(self.width as i32);

        let color_value = color.to_u32();
        let y_offset = (y as u32 * self.width) as usize;

        for xi in x0..x1 {
            self.data[y_offset + xi as usize] = color_value;
        }
    }

    /// Draw a vertical line
    pub fn draw_vline(&mut self, x: i32, y: i32, height: i32, color: Color) {
        if x < 0 || x >= self.width as i32 {
            return;
        }

        let y0 = y.max(0);
        let y1 = (y + height).min(self.height as i32);

        let color_value = color.to_u32();
        let x_offset = x as usize;

        for yi in y0..y1 {
            let index = (yi as u32 * self.width) as usize + x_offset;
            self.data[index] = color_value;
        }
    }

    /// Resize the framebuffer
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let pixel_count = (width * height) as usize;
        self.data.resize(pixel_count, 0);
    }

    /// Copy from another framebuffer
    pub fn copy_from(&mut self, other: &Framebuffer) {
        if self.width == other.width && self.height == other.height {
            self.data.copy_from_slice(&other.data);
        }
    }

    /// Blit another framebuffer onto this one at position
    pub fn blit(&mut self, other: &Framebuffer, x: i32, y: i32) {
        let src_width = other.width as i32;
        let src_height = other.height as i32;
        let dst_width = self.width as i32;
        let dst_height = self.height as i32;

        for sy in 0..src_height {
            let dy = y + sy;
            if dy < 0 || dy >= dst_height {
                continue;
            }

            for sx in 0..src_width {
                let dx = x + sx;
                if dx < 0 || dx >= dst_width {
                    continue;
                }

                let src_index = (sy * src_width + sx) as usize;
                let dst_index = (dy * dst_width + dx) as usize;
                self.data[dst_index] = other.data[src_index];
            }
        }
    }
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new(800, 600)
    }
}
