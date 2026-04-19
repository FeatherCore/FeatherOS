//! Platform Implementations
//!
//! NuttX Framebuffer + Input 实现

pub mod framebuffer;
pub mod input;
pub mod runner;

pub use runner::{InputBridge, DefaultInputBridge, WindowRunner};
