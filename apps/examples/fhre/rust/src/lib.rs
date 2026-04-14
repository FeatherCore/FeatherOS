#![no_std]
#![no_main]

//! FHRE Rust Demo
//!
//! Demonstrates the declarative dual-world architecture:
//! - Main World: Game logic, entity spawning, component updates
//! - Render World: Extracted data, draw commands, framebuffer output
//! - Extract Phase: Syncs data from Main World to Render World
//!
//! This version uses mmap to directly write to the framebuffer memory.
//! Includes usleep() to allow NuttX idle thread to run sim_x11loop().

extern crate alloc;

// Import libc functions
extern "C" {
    fn printf(format: *const u8, ...) -> i32;
    fn open(path: *const u8, flags: i32) -> i32;
    fn close(fd: i32) -> i32;
    fn ioctl(fd: i32, request: u32, ...) -> i32;
    fn mmap(
        addr: *mut core::ffi::c_void,
        length: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: isize,
    ) -> *mut core::ffi::c_void;
    fn munmap(addr: *mut core::ffi::c_void, length: usize) -> i32;
    fn usleep(usec: u32) -> i32;
    fn getpid() -> i32;
    fn __errno() -> *mut i32;
}

const O_RDWR: i32 = 2;
const PROT_READ: i32 = 1;
const PROT_WRITE: i32 = 2;
const MAP_SHARED: i32 = 1;

// Import FHRE modules
use fhre::{
    App, FHRE_VERSION,
    main_world::{Transform, Sprite, Velocity},
    math::{Color, Vec2},
};

/// Framebuffer video info structure
/// Matches NuttX struct fb_videoinfo_s
#[repr(C)]
struct FbVideoInfo {
    fmt: u8,        /* see FB_FMT_* */
    xres: u16,      /* Horizontal resolution in pixel columns */
    yres: u16,      /* Vertical resolution in pixel rows */
    nplanes: u8,    /* Number of color planes supported */
}

/// Framebuffer plane info structure
/// Matches NuttX struct fb_planeinfo_s
#[repr(C)]
struct FbPlaneInfo {
    fbmem: *mut u8,     /* Start of frame buffer memory */
    fblen: usize,       /* Length of frame buffer memory in bytes */
    stride: u16,        /* Length of a line in bytes */
    display: u8,        /* Display number */
    bpp: u8,            /* Bits per pixel */
    xres_virtual: u32,  /* Virtual Horizontal resolution */
    yres_virtual: u32,  /* Virtual Vertical resolution */
    xoffset: u32,       /* Offset from virtual to visible */
    yoffset: u32,       /* Offset from virtual to visible */
}

/// IOCTL commands
/// NuttX uses _FBIOCBASE (0x2800) as the base for framebuffer ioctls
/// FBIOGET_VIDEOINFO = _FBIOC(0x0001) = 0x2800 | 0x0001 = 0x2801
/// FBIOGET_PLANEINFO = _FBIOC(0x0002) = 0x2800 | 0x0002 = 0x2802
const FBIOGET_VIDEOINFO: u32 = 0x2801;
const FBIOGET_PLANEINFO: u32 = 0x2802;

/// Error codes from NuttX
const ENOENT: i32 = 2;    // No such file or directory
const ENOTTY: i32 = 25;   // Not a typewriter (inappropriate ioctl)

/// Demo application state
struct DemoApp {
    app: App,
    fb_fd: i32,
    fb_mem: *mut u32,
    fb_size: usize,
    width: u32,
    height: u32,
    frame_count: u32,
}

impl DemoApp {
    /// Create a new demo application
    fn new() -> Option<Self> {
        unsafe {
            printf(b"[DEBUG] Starting DemoApp::new()\n\0".as_ptr());
            printf(b"[DEBUG] PID=%d\n\0".as_ptr(), getpid());
        }

        // Open framebuffer device
        let fb_fd = unsafe { open(b"/dev/fb0\0".as_ptr(), O_RDWR) };
        if fb_fd < 0 {
            unsafe {
                printf(b"[ERROR] Failed to open /dev/fb0, fd=%d\n\0".as_ptr(), fb_fd);
            }
            return None;
        }
        unsafe {
            printf(b"[DEBUG] Opened /dev/fb0, fd=%d\n\0".as_ptr(), fb_fd);
        }

        // Get video info
        unsafe {
            printf(b"[DEBUG] FBIOGET_VIDEOINFO = 0x%x\n\0".as_ptr(), FBIOGET_VIDEOINFO);
        }
        let mut vinfo: FbVideoInfo = unsafe { core::mem::zeroed() };
        let ret = unsafe { ioctl(fb_fd, FBIOGET_VIDEOINFO, &mut vinfo as *mut _) };
        if ret < 0 {
            unsafe {
                let errno = *__errno();
                printf(b"[ERROR] Failed to get video info, ret=%d, errno=%d\n\0".as_ptr(), ret, errno);
                if errno == ENOTTY {
                    printf(b"[ERROR] ENOTTY: Driver doesn't support this ioctl\n\0".as_ptr());
                }
                close(fb_fd);
            }
            return None;
        }

        // Get plane info
        let mut pinfo: FbPlaneInfo = unsafe { core::mem::zeroed() };
        let ret = unsafe { ioctl(fb_fd, FBIOGET_PLANEINFO, &mut pinfo as *mut _) };
        if ret < 0 {
            unsafe {
                let errno = *__errno();
                printf(b"[ERROR] Failed to get plane info, ret=%d, errno=%d\n\0".as_ptr(), ret, errno);
                close(fb_fd);
            }
            return None;
        }

        unsafe {
            printf(
                b"[DEBUG] Video info: %dx%d, fmt=%d, nplanes=%d\n\0".as_ptr(),
                vinfo.xres as u32,
                vinfo.yres as u32,
                vinfo.fmt as u32,
                vinfo.nplanes as u32,
            );
            printf(
                b"[DEBUG] Plane info: fbmem=%p, fblen=%d, stride=%d, bpp=%d\n\0".as_ptr(),
                pinfo.fbmem,
                pinfo.fblen,
                pinfo.stride as u32,
                pinfo.bpp as u32,
            );
        }

        // Use framebuffer memory directly from plane info
        // Note: In NuttX SIM platform, mmap() doesn't work as expected because
        // the system calls are passed through to the host Linux kernel.
        // Instead, we use the fbmem address directly provided by the driver.
        let width = vinfo.xres as u32;
        let height = vinfo.yres as u32;
        let fb_size = (width * height * 4) as usize; // 4 bytes per pixel (RGBA32)
        
        unsafe {
            printf(b"[DEBUG] Using driver-provided framebuffer memory\n\0".as_ptr());
            printf(b"[DEBUG] fbmem=%p, fblen=%d, requested size=%d\n\0".as_ptr(),
                pinfo.fbmem, pinfo.fblen, fb_size);
        }

        // Use the framebuffer memory address directly from the driver
        let fb_mem = pinfo.fbmem as *mut u32;

        if fb_mem.is_null() {
            unsafe {
                printf(b"[ERROR] Driver returned null framebuffer address\n\0".as_ptr());
                close(fb_fd);
            }
            return None;
        }

        unsafe {
            printf(
                b"[DEBUG] Using framebuffer at: %p\n\0".as_ptr(),
                fb_mem as *const u8,
            );
        }

        let mut app = App::new();

        // Spawn some entities in the Main World
        Self::spawn_entities(&mut app);

        Some(Self {
            app,
            fb_fd,
            fb_mem,
            fb_size,
            width,
            height,
            frame_count: 0,
        })
    }

    /// Spawn demo entities
    fn spawn_entities(app: &mut App) {
        unsafe {
            printf(b"[DEBUG] Spawning entities...\n\0".as_ptr());
        }

        // Entity 1: Red square that moves
        let entity1 = app.main_world.spawn();
        app.main_world.insert_component(entity1, Transform::from_position(100.0, 100.0));
        app.main_world.insert_component(entity1, Sprite::new_with_color(50.0, 50.0, Color::RED));
        app.main_world.insert_component(entity1, Velocity::from_xy(2.0, 1.5));

        // Entity 2: Green rectangle
        let entity2 = app.main_world.spawn();
        app.main_world.insert_component(entity2, Transform::from_position(300.0, 200.0));
        app.main_world.insert_component(entity2, Sprite::new_with_color(100.0, 60.0, Color::GREEN));

        // Entity 3: Blue square
        let entity3 = app.main_world.spawn();
        app.main_world.insert_component(entity3, Transform::from_position(500.0, 300.0));
        app.main_world.insert_component(entity3, Sprite::new_with_color(80.0, 80.0, Color::BLUE));

        // Entity 4: Yellow circle (represented as square for now)
        let entity4 = app.main_world.spawn();
        app.main_world.insert_component(entity4, Transform::from_position(200.0, 350.0));
        app.main_world.insert_component(entity4, Sprite::new_with_color(60.0, 60.0, Color::YELLOW));

        unsafe {
            printf(b"[DEBUG] Spawned 4 entities\n\0".as_ptr());
        }
    }

    /// Update game logic in Main World
    fn update(&mut self) {
        self.frame_count += 1;

        // Collect entity IDs and velocities first to avoid borrow issues
        let mut updates: alloc::vec::Vec<(u64, f32, f32)> = alloc::vec::Vec::new();

        for (entity, velocity) in self.app.main_world.query::<Velocity>() {
            updates.push((entity.id(), velocity.linear.x, velocity.linear.y));
        }

        // Now apply updates
        for (entity_id, vx, vy) in updates {
            let entity = fhre::main_world::Entity::new(entity_id);
            if let Some(transform) = self.app.main_world.get_component_mut::<Transform>(entity) {
                transform.position.x += vx;
                transform.position.y += vy;

                // Simple boundary bounce - modify velocity through component
                let mut new_vx = vx;
                let mut new_vy = vy;

                if transform.position.x > 600.0 || transform.position.x < 0.0 {
                    new_vx = -vx;
                }
                if transform.position.y > 440.0 || transform.position.y < 0.0 {
                    new_vy = -vy;
                }

                // Update velocity if changed
                if new_vx != vx || new_vy != vy {
                    if let Some(velocity) = self.app.main_world.get_component_mut::<Velocity>(entity)
                    {
                        velocity.linear.x = new_vx;
                        velocity.linear.y = new_vy;
                    }
                }
            }
        }

        // Animate the yellow entity (rotation effect)
        let yellow_entity = fhre::main_world::Entity::new(3);
        if let Some(transform) = self.app.main_world.get_component_mut::<Transform>(yellow_entity)
        {
            transform.rotation += 0.05;
            // Add some vertical oscillation
            transform.position.y = 350.0 + libm::sinf(self.frame_count as f32 * 0.05) * 50.0;
        }
    }

    /// Render one frame and present to display
    fn render(&mut self) {
        // First run our game logic
        self.update();

        // Then let the app handle the rendering pipeline
        self.app.update();

        // Copy render world framebuffer to hardware framebuffer
        let fb = self.app.get_framebuffer();
        let size = fb.len().min(self.fb_size / 4);

        unsafe {
            // Print debug info every 30 frames
            if self.frame_count % 30 == 0 {
                printf(
                    b"[DEBUG] Frame %d: fb=%p, fb_mem=%p, size=%d, first_pixel=0x%x\n\0".as_ptr(),
                    self.frame_count,
                    fb.as_ptr(),
                    self.fb_mem,
                    size,
                    *fb.as_ptr(),
                );
            }

            // Copy to mmap'd framebuffer
            core::ptr::copy_nonoverlapping(fb.as_ptr(), self.fb_mem, size);

            // Verify write
            if self.frame_count % 30 == 0 {
                let first_pixel = *self.fb_mem;
                printf(
                    b"[DEBUG] After copy: first_pixel=0x%x\n\0".as_ptr(),
                    first_pixel,
                );
            }
        }
    }

    /// Get the framebuffer for output
    fn get_framebuffer(&self) -> &[u32] {
        self.app.get_framebuffer()
    }
}

impl Drop for DemoApp {
    fn drop(&mut self) {
        unsafe {
            printf(b"[DEBUG] Cleaning up DemoApp\n\0".as_ptr());
            // Note: We don't munmap because we didn't mmap - we used driver's fbmem directly
            close(self.fb_fd);
        }
    }
}

/// Main entry point
#[no_mangle]
pub extern "C" fn fhre_rust_main(_argc: i32, _argv: *const *const u8) -> i32 {
    // Print version info
    unsafe {
        printf(b"\n========================================\n\0".as_ptr());
        printf(b"FHRE Rust Demo - Dual World Architecture\n\0".as_ptr());
        printf(b"Version: %s\n\0".as_ptr(), FHRE_VERSION.as_ptr());
        printf(b"========================================\n\0".as_ptr());
        printf(b"\nArchitecture:\n\0".as_ptr());
        printf(b"  - Main World: Game logic, entities, components\n\0".as_ptr());
        printf(b"  - Render World: Extracted data, draw commands\n\0".as_ptr());
        printf(b"  - Extract Phase: Sync Main World -> Render World\n\0".as_ptr());
        printf(b"  - mmap: Direct framebuffer memory access\n\0".as_ptr());
        printf(b"  - usleep: Yield CPU to allow X11 refresh\n\n\0".as_ptr());
    }

    // Create demo application
    let mut demo = match DemoApp::new() {
        Some(demo) => demo,
        None => {
            unsafe {
                printf(b"[ERROR] Failed to create demo application\n\0".as_ptr());
            }
            return -1;
        }
    };

    // Run infinite loop - demo never exits unless killed
    unsafe {
        printf(b"[DEBUG] Starting main loop (infinite)\n\0".as_ptr());
    }

    let mut frame: u32 = 0;
    loop {
        demo.render();

        // Print every 60 frames (about once per second)
        if frame % 60 == 0 {
            unsafe {
                printf(b"[DEBUG] Rendered frame %d\n\0".as_ptr(), frame);
            }
        }
        frame += 1;

        // IMPORTANT: Yield CPU time to allow NuttX idle thread to run sim_x11loop()
        unsafe {
            usleep(16000); // ~16ms for 60 FPS
        }
    }
}
