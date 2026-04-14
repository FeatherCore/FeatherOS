//! Framebuffer abstraction
//!
//! Provides a common framebuffer interface for all platforms.
//! This replaces the old renderer::Framebuffer to avoid duplication.

use crate::math::Color;
use alloc::vec::Vec;
use alloc::vec;

/// Framebuffer trait - implemented by platform-specific displays
pub trait Framebuffer {
    /// Get framebuffer width
    fn width(&self) -> u32;
    
    /// Get framebuffer height
    fn height(&self) -> u32;
    
    /// Get raw pixel data
    fn data(&self) -> &[u32];
    
    /// Get mutable pixel data
    fn data_mut(&mut self) -> &mut [u32];
    
    /// Clear framebuffer with color
    fn clear(&mut self, color: Color) {
        let rgba = color.to_rgba32();
        for pixel in self.data_mut().iter_mut() {
            *pixel = rgba;
        }
    }
    
    /// Clear with raw u32 color
    fn clear_raw(&mut self, color: u32) {
        for pixel in self.data_mut().iter_mut() {
            *pixel = color;
        }
    }
    
    /// Set pixel at (x, y)
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width() && y < self.height() {
            let index = (y * self.width() + x) as usize;
            if index < self.data_mut().len() {
                self.data_mut()[index] = color.to_rgba32();
            }
        }
    }
    
    /// Set pixel with raw u32 color
    fn set_pixel_raw(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width() && y < self.height() {
            let index = (y * self.width() + x) as usize;
            if index < self.data_mut().len() {
                self.data_mut()[index] = color;
            }
        }
    }
    
    /// Get pixel at (x, y)
    fn get_pixel(&self, x: u32, y: u32) -> Option<u32> {
        if x < self.width() && y < self.height() {
            let index = (y * self.width() + x) as usize;
            self.data().get(index).copied()
        } else {
            None
        }
    }
}

/// Simple in-memory framebuffer for testing and fallback
pub struct SimpleFramebuffer {
    width: u32,
    height: u32,
    data: Vec<u32>,
}

impl SimpleFramebuffer {
    /// Create a new framebuffer
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            data: vec![0u32; size],
        }
    }
    
    /// Create with initial color
    pub fn with_color(width: u32, height: u32, color: Color) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            data: vec![color.to_rgba32(); size],
        }
    }
}

impl Framebuffer for SimpleFramebuffer {
    fn width(&self) -> u32 {
        self.width
    }
    
    fn height(&self) -> u32 {
        self.height
    }
    
    fn data(&self) -> &[u32] {
        &self.data
    }
    
    fn data_mut(&mut self) -> &mut [u32] {
        &mut self.data
    }
}
