//! Framebuffer Platform Implementation for NuttX SIM
//!
//! Provides window and input handling using NuttX's X11 simulation.
//!
//! # Device Paths
//! - Framebuffer: `/dev/fb0` - X11 framebuffer driver
//! - Touch/Mouse: `/dev/input0` - Touchscreen driver (maps X11 mouse events)
//! - Keyboard: `/dev/kbd` - Keyboard driver
//!
//! # X11 Integration
//! Opening `/dev/fb0` triggers `sim_x11openwindow()` which calls `XMapWindow()`
//! to display the X11 window.

use fhre::{
    window::{Window as WindowTrait, WindowInputEvents, WindowInputAdapter, 
             MouseButtonEvent, MouseMotionEvent, KeyboardEvent},
    input::{KeyCode, MouseButton},
};
use core::ffi::{c_int, c_short};
use alloc::vec::Vec;

extern "C" {
    fn open(path: *const u8, flags: c_int) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut u8, count: usize) -> isize;
    fn ioctl(fd: c_int, request: c_int, ...) -> c_int;
    fn printf(format: *const u8, ...) -> i32;
    fn sim_x11events();
    fn XPending(display: *mut core::ffi::c_void) -> c_int;
    static mut g_display: *mut core::ffi::c_void;
}

// ============================================================
// File Constants
// ============================================================

/// Open for reading and writing
const O_RDWR: c_int = 3;

/// Non-blocking open mode
const O_NONBLOCK: c_int = 0x40;

// ============================================================
// IOCTL Request Codes
// ============================================================

/// Get video info (FBIOGET_VIDEOINFO)
const FBIOGET_VIDEOINFO: c_int = 0x2801;

/// Get plane info (FBIOGET_PLANEINFO)
const FBIOGET_PLANEINFO: c_int = 0x2802;

// ============================================================
// X11 Keycode Mappings
// ============================================================

/// X11 keycode for Space key
const X11_KEY_SPACE: u32 = 0x0020;

/// X11 keycode for R key
const X11_KEY_R: u32 = 0x0072;

/// X11 keycode for Escape key
const X11_KEY_ESCAPE: u32 = 0xff1b;

// ============================================================
// NuttX Structures
// ============================================================

/// Video information from framebuffer ioctl
#[repr(C)]
struct VideoInfo {
    fmt: u8,
    xres: u16,
    yres: u16,
    nplanes: u8,
}

/// Plane information from framebuffer ioctl
#[repr(C)]
struct PlaneInfo {
    fbmem: *mut u8,
    fblen: usize,
    stride: u16,
    display: u8,
    bpp: u8,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
}

/// Single touch point data
#[repr(C)]
struct TouchPoint {
    id: u8,
    flags: u8,
    x: i16,
    y: i16,
    h: i16,
    w: i16,
    gesture: u16,
    pressure: u16,
    timestamp: u64,
}

/// Touch sample containing touch point(s)
#[repr(C)]
struct TouchSample {
    npoints: i32,
    point: TouchPoint,
}

/// Keyboard event from NuttX
#[repr(C)]
struct KeyboardEventNuttX {
    keycode: u16,
    pressed: u8,
}

// ============================================================
// Window Implementation
// ============================================================

/// Platform window wrapping NuttX framebuffer and input devices
pub struct Window {
    /// Screen width in pixels
    width: u32,
    /// Screen height in pixels
    height: u32,
    /// Framebuffer file descriptor
    fb_fd: c_int,
    /// Pointer to framebuffer memory (mapped as u32 ARGB)
    fb_ptr: *mut u32,
    /// Touch/mouse input file descriptor
    input_fd: c_int,
    /// Keyboard input file descriptor
    kbd_fd: c_int,
    /// Current touch pressed state (for tracking release)
    touch_pressed: bool,
    /// Last touch X coordinate (for release event position)
    last_touch_x: i32,
    /// Last touch Y coordinate (for release event position)
    last_touch_y: i32,
}

impl Window {
    /// Create new window by opening framebuffer and input devices
    pub fn new() -> Option<Self> {
        unsafe {
            // Open framebuffer device
            let fb_path = b"/dev/fb0\0";
            let fb_fd = open(fb_path.as_ptr(), O_RDWR);
            if fb_fd < 0 {
                return None;
            }
            
            // Get video info for dimensions
            let mut vinfo: VideoInfo = core::mem::zeroed();
            let ret = ioctl(fb_fd, FBIOGET_VIDEOINFO, &mut vinfo as *mut _ as core::ffi::c_ulong);
            if ret < 0 {
                close(fb_fd);
                return None;
            }
            
            let width = vinfo.xres as u32;
            let height = vinfo.yres as u32;
            
            // Get plane info for framebuffer pointer
            let mut pinfo: PlaneInfo = core::mem::zeroed();
            let ret = ioctl(fb_fd, FBIOGET_PLANEINFO, &mut pinfo as *mut _ as core::ffi::c_ulong);
            if ret < 0 {
                close(fb_fd);
                return None;
            }
            
            // Open input devices (non-blocking)
            let input_path = b"/dev/input0\0";
            let input_fd = open(input_path.as_ptr(), O_RDWR | O_NONBLOCK);
            
            let kbd_path = b"/dev/kbd\0";
            let kbd_fd = open(kbd_path.as_ptr(), O_RDWR | O_NONBLOCK);
            
            Some(Self {
                width,
                height,
                fb_fd,
                fb_ptr: pinfo.fbmem as *mut u32,
                input_fd,
                kbd_fd,
                touch_pressed: false,
                last_touch_x: 0,
                last_touch_y: 0,
            })
        }
    }
    
    /// Get screen dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Read touch/mouse events from input device
    fn read_touch_events(&mut self, events: &mut WindowInputEvents) {
        if self.input_fd < 0 {
            return;
        }
        
        unsafe {
            // Process X11 events first
            sim_x11events();
            
            let mut sample: TouchSample = core::mem::zeroed();
            let n = read(self.input_fd, &mut sample as *mut _ as *mut u8, core::mem::size_of::<TouchSample>());
            
            if n > 0 && sample.npoints > 0 {
                let x = sample.point.x as i32;
                let y = sample.point.y as i32;
                let pressure = sample.point.pressure;
                
                // Always report motion
                events.mouse_motion_events.push(MouseMotionEvent {
                    x,
                    y,
                    delta_x: 0,
                    delta_y: 0,
                });
                
                // Track touch state for proper press/release events
                if pressure > 0 {
                    if !self.touch_pressed {
                        self.touch_pressed = true;
                        events.mouse_button_events.push(MouseButtonEvent {
                            button: 1,
                            pressed: true,
                            x,
                            y,
                        });
                    }
                    self.last_touch_x = x;
                    self.last_touch_y = y;
                } else if self.touch_pressed {
                    self.touch_pressed = false;
                    events.mouse_button_events.push(MouseButtonEvent {
                        button: 1,
                        pressed: false,
                        x: self.last_touch_x,
                        y: self.last_touch_y,
                    });
                }
            }
        }
    }
    
    /// Read keyboard events from keyboard device
    fn read_keyboard_events(&mut self, events: &mut WindowInputEvents) {
        if self.kbd_fd < 0 {
            return;
        }
        
        unsafe {
            // Process X11 events first
            sim_x11events();
            
            let mut kbd_event: KeyboardEventNuttX = core::mem::zeroed();
            let n = read(self.kbd_fd, &mut kbd_event as *mut _ as *mut u8, core::mem::size_of::<KeyboardEventNuttX>());
            
            if n > 0 {
                events.keyboard_events.push(KeyboardEvent {
                    keycode: kbd_event.keycode as u32,
                    pressed: kbd_event.pressed != 0,
                });
            }
        }
    }
}

impl WindowTrait for Window {
    fn is_running(&self) -> bool {
        true
    }

    fn collect_input_events(&mut self) -> WindowInputEvents {
        let mut events = WindowInputEvents {
            mouse_button_events: Vec::new(),
            mouse_motion_events: Vec::new(),
            mouse_wheel_events: Vec::new(),
            keyboard_events: Vec::new(),
        };
        
        self.read_touch_events(&mut events);
        self.read_keyboard_events(&mut events);
        
        events
    }

    fn present(&mut self, framebuffer: &[u32]) {
        let size = (self.width * self.height) as usize;
        unsafe {
            core::ptr::copy_nonoverlapping(
                framebuffer.as_ptr(),
                self.fb_ptr,
                size.min(framebuffer.len()),
            );
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if self.input_fd >= 0 {
                close(self.input_fd);
            }
            if self.kbd_fd >= 0 {
                close(self.kbd_fd);
            }
            if self.fb_fd >= 0 {
                close(self.fb_fd);
            }
        }
    }
}

// ============================================================
// Input Adapter
// ============================================================

/// Maps X11 keycodes and mouse buttons to FHRE input types
pub struct InputAdapter;

impl WindowInputAdapter for InputAdapter {
    /// Map X11 keycode to FHRE KeyCode
    fn map_keycode(&self, code: u32) -> Option<KeyCode> {
        match code {
            X11_KEY_SPACE => Some(KeyCode::Space),
            X11_KEY_R => Some(KeyCode::KeyR),
            X11_KEY_ESCAPE => Some(KeyCode::Escape),
            _ => None,
        }
    }
    
    /// Map button number to FHRE MouseButton
    fn map_mouse_button(&self, btn: u32) -> Option<MouseButton> {
        match btn {
            1 => Some(MouseButton::Left),
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::Right),
            _ => None,
        }
    }
}

impl Default for InputAdapter {
    fn default() -> Self { Self }
}

extern crate alloc;
