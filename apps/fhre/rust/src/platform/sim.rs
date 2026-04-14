//! Sim platform support for FHRE
//!
//! Provides integration with NuttX SIM platform (X11 framebuffer).
//! Similar to LVGL's lv_nuttx_fbdev approach.

use crate::math::Color;
use crate::render_world::RenderWorld;

/// NuttX framebuffer device path
pub const FB_DEVICE_PATH: &str = "/dev/fb0";

/// Framebuffer video info structure
#[repr(C)]
pub struct FbVideoInfo {
    pub fmt: u32,
    pub xres: u32,
    pub yres: u32,
    pub nplanes: u8,
}

/// Framebuffer plane info structure
#[repr(C)]
pub struct FbPlaneInfo {
    pub fbmem: *mut u8,
    pub fblen: usize,
    pub stride: u32,
    pub display: u8,
    pub bpp: u8,
    pub xoffset: u32,
    pub yoffset: u32,
}

/// Framebuffer area for partial update
#[repr(C)]
pub struct FbArea {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// IOCTL commands
pub const FBIOGET_VIDEOINFO: u32 = 0x4600;
pub const FBIOGET_PLANEINFO: u32 = 0x4601;
pub const FBIO_UPDATE: u32 = 0x4602;
pub const FBIOPAN_DISPLAY: u32 = 0x4603;

/// Sim platform display driver
pub struct SimDisplay {
    fd: i32,
    width: u32,
    height: u32,
    bpp: u8,
    stride: u32,
    framebuffer: *mut u32,
    backbuffer: alloc::vec::Vec<u32>,
}

impl SimDisplay {
    /// Create a new sim display
    pub fn new() -> Option<Self> {
        unsafe {
            // Open framebuffer device
            let fd = libc::open(FB_DEVICE_PATH.as_ptr() as *const i8, libc::O_RDWR);
            if fd < 0 {
                return None;
            }

            // Get video info
            let mut vinfo: FbVideoInfo = core::mem::zeroed();
            if libc::ioctl(fd, FBIOGET_VIDEOINFO as _, &mut vinfo as *mut _) < 0 {
                libc::close(fd);
                return None;
            }

            // Get plane info
            let mut pinfo: FbPlaneInfo = core::mem::zeroed();
            if libc::ioctl(fd, FBIOGET_PLANEINFO as _, &mut pinfo as *mut _) < 0 {
                libc::close(fd);
                return None;
            }

            let size = (vinfo.xres * vinfo.yres) as usize;
            let backbuffer = alloc::vec![0u32; size];

            Some(Self {
                fd,
                width: vinfo.xres,
                height: vinfo.yres,
                bpp: pinfo.bpp,
                stride: pinfo.stride,
                framebuffer: pinfo.fbmem as *mut u32,
                backbuffer,
            })
        }
    }

    /// Get display dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get backbuffer slice
    pub fn backbuffer(&mut self) -> &mut [u32] {
        &mut self.backbuffer
    }

    /// Present render world to display
    pub fn present(&mut self, render_world: &RenderWorld) {
        // Copy render world framebuffer to backbuffer
        let fb = render_world.get_framebuffer();
        self.backbuffer.copy_from_slice(fb);

        // Copy backbuffer to hardware framebuffer
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.backbuffer.as_ptr(),
                self.framebuffer,
                self.backbuffer.len()
            );
        }

        // Trigger display update via ioctl
        self.flush();
    }

    /// Flush framebuffer to display (trigger update)
    fn flush(&mut self) {
        unsafe {
            // Update entire framebuffer
            let area = FbArea {
                x: 0,
                y: 0,
                w: self.width,
                h: self.height,
            };
            libc::ioctl(self.fd, FBIO_UPDATE as _, &area as *const _);
        }
    }

    /// Partial update (for dirty rectangle optimization)
    pub fn update_area(&mut self, x: u32, y: u32, width: u32, height: u32) {
        unsafe {
            let area = FbArea { x, y, w: width, h: height };
            libc::ioctl(self.fd, FBIO_UPDATE as _, &area as *const _);
        }
    }
}

impl Drop for SimDisplay {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

/// Create a sim display instance
pub fn create_display() -> Option<SimDisplay> {
    SimDisplay::new()
}

/// Sim platform refresh loop
/// Similar to LVGL's display_refr_timer_cb
pub fn refresh_loop<F>(mut render_fn: F)
where
    F: FnMut(),
{
    use core::time::Duration;
    
    let frame_duration = Duration::from_millis(16); // ~60 FPS
    
    loop {
        let start = unsafe { libc::clock() };
        
        // Execute render function
        render_fn();
        
        // Calculate sleep time to maintain frame rate
        let elapsed = unsafe { libc::clock() } - start;
        let elapsed_ms = (elapsed * 1000 / libc::CLOCKS_PER_SEC as i64) as u64;
        
        if elapsed_ms < 16 {
            unsafe {
                libc::usleep(((16 - elapsed_ms) * 1000) as u32);
            }
        }
    }
}
