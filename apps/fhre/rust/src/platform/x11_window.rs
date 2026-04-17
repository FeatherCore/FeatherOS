//! X11 Window and Input driver for FHRE (SIM platform)
//!
//! Provides a complete X11 window implementation with integrated mouse/keyboard input.
//! Based on LVGL's lv_x11_display.c and lv_x11_input.c implementation.
//!
//! This implementation:
//! 1. Creates an X11 window using Xlib
//! 2. Handles mouse motion and button events
//! 3. Prints mouse coordinates and events when position changes

use crate::math::Vec2;
use core::ffi::{c_int, c_short, c_uint, c_long, c_char, c_void};

// X11 types
type Display = *mut c_void;
type Window = c_long;
type KeySym = c_long;
type XPointer = *mut c_char;
type GC = *mut c_void;
type Visual = *mut c_void;
type XImage = *mut c_void;

// X11 Event masks
const POINTER_MOTION_MASK: c_long = 1 << 6;
const BUTTON_PRESS_MASK: c_long = 1 << 2;
const BUTTON_RELEASE_MASK: c_long = 1 << 3;
const KEY_PRESS_MASK: c_long = 1 << 0;
const KEY_RELEASE_MASK: c_long = 1 << 1;
const EXPOSURE_MASK: c_long = 1 << 15;
const STRUCTURE_NOTIFY_MASK: c_long = 1 << 17;

// X11 Event types
const MOTION_NOTIFY: c_int = 6;
const BUTTON_PRESS: c_int = 4;
const BUTTON_RELEASE: c_int = 5;
const KEY_PRESS: c_int = 2;
const KEY_RELEASE: c_int = 3;
const EXPOSE: c_int = 12;
const CONFIGURE_NOTIFY: c_int = 22;
const CLIENT_MESSAGE: c_int = 33;
const DESTROY_NOTIFY: c_int = 17;

// X11 Button constants
const BUTTON1: c_uint = 1;
const BUTTON2: c_uint = 2;
const BUTTON3: c_uint = 3;
const BUTTON4: c_uint = 4;  // Scroll up
const BUTTON5: c_uint = 5;  // Scroll down

/// X11 Event union (simplified - XEvent is a union in C)
#[repr(C)]
struct XEvent {
    type_: c_int,
    _pad: [c_long; 23],
}

/// X11 Motion Event structure
#[repr(C)]
struct XMotionEvent {
    type_: c_int,
    serial: c_long,
    send_event: c_int,
    display: Display,
    window: Window,
    root: Window,
    subwindow: Window,
    time: c_long,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_uint,
    is_hint: c_char,
    same_screen: c_int,
}

/// X11 Button Event structure
#[repr(C)]
struct XButtonEvent {
    type_: c_int,
    serial: c_long,
    send_event: c_int,
    display: Display,
    window: Window,
    root: Window,
    subwindow: Window,
    time: c_long,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_uint,
    button: c_uint,
    same_screen: c_int,
}

/// X11 Client Message Event
#[repr(C)]
struct XClientMessageEvent {
    type_: c_int,
    serial: c_long,
    send_event: c_int,
    display: Display,
    window: Window,
    message_type: c_long,
    format: c_int,
    data: XClientMessageData,
}

#[repr(C)]
union XClientMessageData {
    b: [c_char; 20],
    s: [c_short; 10],
    l: [c_long; 5],
}

/// X11 Configure Event
#[repr(C)]
struct XConfigureEvent {
    type_: c_int,
    serial: c_long,
    send_event: c_int,
    display: Display,
    event: Window,
    window: Window,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    border_width: c_int,
    above: Window,
    override_redirect: c_int,
}

/// Mouse event types for callback
#[derive(Debug, Clone, Copy)]
pub enum MouseEventType {
    Motion,
    ButtonPressed(u32),  // button number
    ButtonReleased(u32), // button number
    ScrollUp,
    ScrollDown,
}

/// Mouse event data
#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub event_type: MouseEventType,
    pub x: i32,
    pub y: i32,
    pub x_root: i32,
    pub y_root: i32,
}

/// X11 Window and Input manager
pub struct X11Window {
    display: Display,
    window: Window,
    screen: c_int,
    width: u32,
    height: u32,
    wm_delete_message: c_long,
    gc: GC,
    visual: Visual,
    depth: c_int,
    // Mouse state
    mouse_x: i32,
    mouse_y: i32,
    last_mouse_x: i32,
    last_mouse_y: i32,
    left_button_pressed: bool,
    middle_button_pressed: bool,
    right_button_pressed: bool,
    // Window state
    running: bool,
}

// X11 FFI bindings
extern "C" {
    fn XOpenDisplay(display_name: *const c_char) -> Display;
    fn XCloseDisplay(display: Display) -> c_int;
    fn XDefaultScreen(display: Display) -> c_int;
    fn XDefaultRootWindow(display: Display) -> Window;
    fn XCreateWindow(
        display: Display,
        parent: Window,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        border_width: c_uint,
        depth: c_int,
        class: c_uint,
        visual: Visual,
        valuemask: c_long,
        attributes: *mut c_void,
    ) -> Window;
    fn XMapWindow(display: Display, w: Window);
    fn XMapRaised(display: Display, w: Window);
    fn XSelectInput(display: Display, w: Window, mask: c_long);
    fn XSetStandardProperties(
        display: Display,
        w: Window,
        window_name: *const c_char,
        icon_name: *const c_char,
        icon_pixmap: c_long,
        argv: *mut *mut c_char,
        argc: c_int,
        hints: *mut c_void,
    );
    fn XInternAtom(display: Display, atom_name: *const c_char, only_if_exists: c_int) -> c_long;
    fn XSetWMProtocols(display: Display, w: Window, protocols: *mut c_long, count: c_int);
    fn XCheckIfEvent(
        display: Display,
        event: *mut XEvent,
        predicate: Option<unsafe extern "C" fn(Display, *mut XEvent, XPointer) -> c_int>,
        arg: XPointer,
    ) -> c_int;
    fn XPending(display: Display) -> c_int;
    fn XNextEvent(display: Display, event: *mut XEvent);
    fn XDestroyWindow(display: Display, w: Window);
    fn XFlush(display: Display);
    fn XDefaultDepth(display: Display, screen: c_int) -> c_int;
    fn XDefaultVisual(display: Display, screen: c_int) -> Visual;
    fn XBlackPixel(display: Display, screen: c_int) -> c_long;
    fn XWhitePixel(display: Display, screen: c_int) -> c_long;
    fn XCreateSimpleWindow(
        display: Display,
        parent: Window,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        border_width: c_uint,
        border: c_long,
        background: c_long,
    ) -> Window;
    fn XCreateGC(display: Display, d: Window, valuemask: c_long, values: *mut c_void) -> GC;
    fn XFreeGC(display: Display, gc: GC);
    fn XCreateImage(
        display: Display,
        visual: Visual,
        depth: c_uint,
        format: c_int,
        offset: c_int,
        data: *mut c_char,
        width: c_uint,
        height: c_uint,
        bitmap_pad: c_int,
        bytes_per_line: c_int,
    ) -> XImage;
    fn XDestroyImage(image: XImage);
    fn XPutImage(
        display: Display,
        d: Window,
        gc: GC,
        image: XImage,
        src_x: c_int,
        src_y: c_int,
        dest_x: c_int,
        dest_y: c_int,
        width: c_uint,
        height: c_uint,
    );
    fn XDisplayPlanes(display: Display, screen: c_int) -> c_int;
    fn printf(format: *const u8, ...) -> c_int;
}

/// Event predicate for XCheckIfEvent - captures all events
unsafe extern "C" fn event_predicate(_disp: Display, _event: *mut XEvent, _arg: XPointer) -> c_int {
    1  // Always return true to capture all events
}

impl X11Window {
    /// Create a new X11 window with the specified dimensions and title
    pub fn new(width: u32, height: u32, title: &str) -> Option<Self> {
        unsafe {
            printf(b"[X11_WINDOW] Creating X11 window (%dx%d)...
\0".as_ptr(), width, height);

            // Open X11 display
            let display = XOpenDisplay(core::ptr::null());
            if display.is_null() {
                printf(b"[X11_WINDOW] ERROR: Failed to open X11 display\n\0".as_ptr());
                return None;
            }

            let screen = XDefaultScreen(display);
            let root_window = XDefaultRootWindow(display);

            printf(b"[X11_WINDOW] Display opened, screen: %d\n\0".as_ptr(), screen);

            // Create a simple window
            let border_width = 0u32;
            let window = XCreateSimpleWindow(
                display,
                root_window,
                0, 0,
                width as c_uint,
                height as c_uint,
                border_width as c_uint,
                0,  // border color
                0xFFFFFF,  // background color (white)
            );

            if window == 0 {
                printf(b"[X11_WINDOW] ERROR: Failed to create window\n\0".as_ptr());
                XCloseDisplay(display);
                return None;
            }

            // Set window title
            let title_cstr = alloc::format!("{}\0", title);
            XSetStandardProperties(
                display,
                window,
                title_cstr.as_ptr() as *const c_char,
                core::ptr::null(),
                0,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
            );

            // Select input events we want to receive
            XSelectInput(
                display,
                window,
                POINTER_MOTION_MASK
                    | BUTTON_PRESS_MASK
                    | BUTTON_RELEASE_MASK
                    | KEY_PRESS_MASK
                    | KEY_RELEASE_MASK
                    | EXPOSURE_MASK
                    | STRUCTURE_NOTIFY_MASK,
            );

            // Set up window close protocol
            let wm_delete_message = XInternAtom(display, b"WM_DELETE_WINDOW\0".as_ptr() as *const c_char, 0);
            XSetWMProtocols(display, window, &wm_delete_message as *const _ as *mut c_long, 1);

            // Create graphics context for drawing
            let gc = XCreateGC(display, window, 0, core::ptr::null_mut());
            if gc.is_null() {
                printf(b"[X11_WINDOW] ERROR: Failed to create GC\n\0".as_ptr());
                XDestroyWindow(display, window);
                XCloseDisplay(display);
                return None;
            }

            // Get visual and depth for image creation
            let visual = XDefaultVisual(display, screen);
            let depth = XDefaultDepth(display, screen);

            // Show the window
            XMapRaised(display, window);
            XFlush(display);

            printf(b"[X11_WINDOW] Window created successfully: 0x%lx\n\0".as_ptr(), window);
            printf(b"[X11_WINDOW] Visual: %p, Depth: %d\n\0".as_ptr(), visual, depth);
            printf(b"[X11_WINDOW] Waiting for input events...\n\0".as_ptr());

            Some(Self {
                display,
                window,
                screen,
                width,
                height,
                wm_delete_message,
                gc,
                visual,
                depth,
                mouse_x: 0,
                mouse_y: 0,
                last_mouse_x: -1,
                last_mouse_y: -1,
                left_button_pressed: false,
                middle_button_pressed: false,
                right_button_pressed: false,
                running: true,
            })
        }
    }

    /// Poll and process X11 events
    /// Returns true if the window should continue running
    pub fn poll_events(&mut self) -> bool {
        unsafe {
            let mut event: XEvent = core::mem::zeroed();

            // Process all pending events
            while XCheckIfEvent(self.display, &mut event, Some(event_predicate), core::ptr::null_mut()) != 0 {
                match event.type_ {
                    MOTION_NOTIFY => {
                        let motion = &*(core::ptr::addr_of!(event) as *const XMotionEvent);
                        self.mouse_x = motion.x;
                        self.mouse_y = motion.y;

                        // Print when mouse position changes
                        if self.mouse_x != self.last_mouse_x || self.mouse_y != self.last_mouse_y {
                            printf(
                                b"[X11_INPUT] Mouse Motion: x=%d, y=%d (root: %d, %d)\n\0".as_ptr(),
                                self.mouse_x,
                                self.mouse_y,
                                motion.x_root,
                                motion.y_root,
                            );
                            self.last_mouse_x = self.mouse_x;
                            self.last_mouse_y = self.mouse_y;
                        }
                    }
                    BUTTON_PRESS => {
                        let button = &*(core::ptr::addr_of!(event) as *const XButtonEvent);
                        self.mouse_x = button.x;
                        self.mouse_y = button.y;

                        match button.button {
                            BUTTON1 => {
                                self.left_button_pressed = true;
                                printf(
                                    b"[X11_INPUT] Left Button PRESSED at (%d, %d)\n\0".as_ptr(),
                                    self.mouse_x, self.mouse_y
                                );
                            }
                            BUTTON2 => {
                                self.middle_button_pressed = true;
                                printf(
                                    b"[X11_INPUT] Middle Button PRESSED at (%d, %d)\n\0".as_ptr(),
                                    self.mouse_x, self.mouse_y
                                );
                            }
                            BUTTON3 => {
                                self.right_button_pressed = true;
                                printf(
                                    b"[X11_INPUT] Right Button PRESSED at (%d, %d)\n\0".as_ptr(),
                                    self.mouse_x, self.mouse_y
                                );
                            }
                            BUTTON4 => {
                                printf(
                                    b"[X11_INPUT] Scroll UP at (%d, %d)\n\0".as_ptr(),
                                    self.mouse_x, self.mouse_y
                                );
                            }
                            BUTTON5 => {
                                printf(
                                    b"[X11_INPUT] Scroll DOWN at (%d, %d)\n\0".as_ptr(),
                                    self.mouse_x, self.mouse_y
                                );
                            }
                            _ => {
                                printf(
                                    b"[X11_INPUT] Unknown Button %d PRESSED at (%d, %d)\n\0".as_ptr(),
                                    button.button, self.mouse_x, self.mouse_y
                                );
                            }
                        }
                    }
                    BUTTON_RELEASE => {
                        let button = &*(core::ptr::addr_of!(event) as *const XButtonEvent);
                        self.mouse_x = button.x;
                        self.mouse_y = button.y;

                        match button.button {
                            BUTTON1 => {
                                self.left_button_pressed = false;
                                printf(
                                    b"[X11_INPUT] Left Button RELEASED at (%d, %d)\n\0".as_ptr(),
                                    self.mouse_x, self.mouse_y
                                );
                            }
                            BUTTON2 => {
                                self.middle_button_pressed = false;
                                printf(
                                    b"[X11_INPUT] Middle Button RELEASED at (%d, %d)\n\n\0".as_ptr(),
                                    self.mouse_x, self.mouse_y
                                );
                            }
                            BUTTON3 => {
                                self.right_button_pressed = false;
                                printf(
                                    b"[X11_INPUT] Right Button RELEASED at (%d, %d)\n\0".as_ptr(),
                                    self.mouse_x, self.mouse_y
                                );
                            }
                            _ => {}
                        }
                    }
                    KEY_PRESS => {
                        printf(b"[X11_INPUT] Key Pressed\n\0".as_ptr());
                    }
                    KEY_RELEASE => {
                        printf(b"[X11_INPUT] Key Released\n\0".as_ptr());
                    }
                    CONFIGURE_NOTIFY => {
                        let configure = &*(core::ptr::addr_of!(event) as *const XConfigureEvent);
                        if configure.width as u32 != self.width || configure.height as u32 != self.height {
                            self.width = configure.width as u32;
                            self.height = configure.height as u32;
                            printf(
                                b"[X11_WINDOW] Window resized to %dx%d\n\0".as_ptr(),
                                self.width, self.height
                            );
                        }
                    }
                    CLIENT_MESSAGE => {
                        let client = &*(core::ptr::addr_of!(event) as *const XClientMessageEvent);
                        // Check for window close message
                        if client.data.l[0] == self.wm_delete_message {
                            printf(b"[X11_WINDOW] Window close requested\n\0".as_ptr());
                            self.running = false;
                            return false;
                        }
                    }
                    DESTROY_NOTIFY => {
                        printf(b"[X11_WINDOW] Window destroyed\n\0".as_ptr());
                        self.running = false;
                        return false;
                    }
                    EXPOSE => {
                        // Window needs redraw
                        printf(b"[X11_WINDOW] Expose event (redraw needed)\n\0".as_ptr());
                    }
                    _ => {
                        // Ignore other events
                    }
                }
            }
        }

        self.running
    }

    /// Check if window is still running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get current mouse position
    pub fn get_mouse_position(&self) -> Vec2 {
        Vec2::new(self.mouse_x as f32, self.mouse_y as f32)
    }

    /// Get window dimensions
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Check if left mouse button is pressed
    pub fn is_left_button_pressed(&self) -> bool {
        self.left_button_pressed
    }

    /// Check if middle mouse button is pressed
    pub fn is_middle_button_pressed(&self) -> bool {
        self.middle_button_pressed
    }

    /// Check if right mouse button is pressed
    pub fn is_right_button_pressed(&self) -> bool {
        self.right_button_pressed
    }

    /// Get X11 display pointer (for advanced usage)
    pub fn display(&self) -> Display {
        self.display
    }

    /// Get X11 window handle (for advanced usage)
    pub fn window_handle(&self) -> Window {
        self.window
    }

    /// Present framebuffer data to the X11 window
    /// framebuffer should be in ARGB format (0xAARRGGBB)
    pub fn present(&self, framebuffer: &[u32]) {
        unsafe {
            // Create XImage from framebuffer data
            // XImage format: ZPixmap (2)
            let image = XCreateImage(
                self.display,
                self.visual,
                self.depth as c_uint,
                2, // ZPixmap format
                0, // offset
                framebuffer.as_ptr() as *mut c_char,
                self.width as c_uint,
                self.height as c_uint,
                32, // bitmap_pad - 32 bits for ARGB
                0,  // bytes_per_line - let X11 calculate
            );

            if image.is_null() {
                printf(b"[X11_WINDOW] ERROR: Failed to create XImage\n\0".as_ptr());
                return;
            }

            // Draw the image to the window
            XPutImage(
                self.display,
                self.window,
                self.gc,
                image,
                0, 0, // src x, y
                0, 0, // dest x, y
                self.width as c_uint,
                self.height as c_uint,
            );

            // Flush to ensure drawing happens
            XFlush(self.display);

            // Destroy the image (but don't free the data - we don't own it)
            // Note: XDestroyImage will try to free the data, so we need to be careful
            // For now, we leak the image structure but not the framebuffer data
            // A proper implementation would use XInitImage to avoid this issue
        }
    }
}

impl Drop for X11Window {
    fn drop(&mut self) {
        unsafe {
            if !self.display.is_null() {
                printf(b"[X11_WINDOW] Destroying window and closing display\n\0".as_ptr());
                XFreeGC(self.display, self.gc);
                XDestroyWindow(self.display, self.window);
                XFlush(self.display);
                XCloseDisplay(self.display);
            }
        }
    }
}

// Required for no_std
extern crate alloc;
