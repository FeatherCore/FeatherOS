//! X11 Window and Input driver for FHRE Demo (SIM platform)
//!
//! Provides a complete X11 window implementation with integrated mouse/keyboard input.
//! This is a platform-specific implementation that should be provided by the application.

use fhre::{Vec2};
use core::ffi::{c_int, c_short, c_uint, c_long, c_char, c_void};

// X11 types
type Display = *mut c_void;
type X11WindowHandle = c_long;
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
const BUTTON4: c_uint = 4;
const BUTTON5: c_uint = 5;

/// Input events collected from window (X11 specific)
#[derive(Debug, Clone)]
pub struct X11InputEvents {
    pub mouse_button_events: alloc::vec::Vec<X11MouseButtonEvent>,
    pub mouse_motion_events: alloc::vec::Vec<X11MouseMotionEvent>,
    pub mouse_wheel_events: alloc::vec::Vec<X11MouseWheelEvent>,
    pub keyboard_events: alloc::vec::Vec<X11KeyboardEvent>,
}

/// X11 Mouse button event
#[derive(Debug, Clone, Copy)]
pub struct X11MouseButtonEvent {
    pub button: u32,
    pub pressed: bool,
    pub x: i32,
    pub y: i32,
}

/// X11 Mouse motion event
#[derive(Debug, Clone, Copy)]
pub struct X11MouseMotionEvent {
    pub x: i32,
    pub y: i32,
    pub delta_x: i32,
    pub delta_y: i32,
}

/// X11 Mouse wheel event
#[derive(Debug, Clone, Copy)]
pub struct X11MouseWheelEvent {
    pub direction: i32,
    pub x: i32,
    pub y: i32,
}

/// X11 Keyboard event
#[derive(Debug, Clone, Copy)]
pub struct X11KeyboardEvent {
    pub keycode: u32,
    pub pressed: bool,
}

/// X11 Event union
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
    window: X11WindowHandle,
    root: X11WindowHandle,
    subwindow: X11WindowHandle,
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
    window: X11WindowHandle,
    root: X11WindowHandle,
    subwindow: X11WindowHandle,
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
    window: X11WindowHandle,
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
    event: X11WindowHandle,
    window: X11WindowHandle,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
    border_width: c_int,
    above: X11WindowHandle,
    override_redirect: c_int,
}

/// X11 Key Event
#[repr(C)]
struct XKeyEvent {
    type_: c_int,
    serial: c_long,
    send_event: c_int,
    display: Display,
    window: X11WindowHandle,
    root: X11WindowHandle,
    subwindow: X11WindowHandle,
    time: c_long,
    x: c_int,
    y: c_int,
    x_root: c_int,
    y_root: c_int,
    state: c_uint,
    keycode: c_uint,
    same_screen: c_int,
}

// X11 FFI bindings
extern "C" {
    fn XOpenDisplay(display_name: *const c_char) -> Display;
    fn XCloseDisplay(display: Display) -> c_int;
    fn XDefaultScreen(display: Display) -> c_int;
    fn XDefaultRootWindow(display: Display) -> X11WindowHandle;
    fn XCreateSimpleWindow(
        display: Display,
        parent: X11WindowHandle,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        border_width: c_uint,
        border: c_long,
        background: c_long,
    ) -> X11WindowHandle;
    fn XMapRaised(display: Display, w: X11WindowHandle);
    fn XSelectInput(display: Display, w: X11WindowHandle, mask: c_long);
    fn XSetStandardProperties(
        display: Display,
        w: X11WindowHandle,
        window_name: *const c_char,
        icon_name: *const c_char,
        icon_pixmap: c_long,
        argv: *mut *mut c_char,
        argc: c_int,
        hints: *mut c_void,
    );
    fn XInternAtom(display: Display, atom_name: *const c_char, only_if_exists: c_int) -> c_long;
    fn XSetWMProtocols(display: Display, w: X11WindowHandle, protocols: *mut c_long, count: c_int);
    fn XCheckIfEvent(
        display: Display,
        event: *mut XEvent,
        predicate: Option<unsafe extern "C" fn(Display, *mut XEvent, XPointer) -> c_int>,
        arg: XPointer,
    ) -> c_int;
    fn XDestroyWindow(display: Display, w: X11WindowHandle);
    fn XFlush(display: Display);
    fn XDefaultDepth(display: Display, screen: c_int) -> c_int;
    fn XDefaultVisual(display: Display, screen: c_int) -> Visual;
    fn XCreateGC(display: Display, d: X11WindowHandle, valuemask: c_long, values: *mut c_void) -> GC;
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
    fn XPutImage(
        display: Display,
        d: X11WindowHandle,
        gc: GC,
        image: XImage,
        src_x: c_int,
        src_y: c_int,
        dest_x: c_int,
        dest_y: c_int,
        width: c_uint,
        height: c_uint,
    );
    fn printf(format: *const u8, ...) -> i32;
}

unsafe extern "C" fn event_predicate(_disp: Display, _event: *mut XEvent, _arg: XPointer) -> c_int {
    1
}

/// X11 Window implementation for FHRE Demo
pub struct X11Window {
    display: Display,
    window: X11WindowHandle,
    screen: c_int,
    width: u32,
    height: u32,
    wm_delete_message: c_long,
    gc: GC,
    visual: Visual,
    depth: c_int,
    mouse_x: i32,
    mouse_y: i32,
    running: bool,
}

impl X11Window {
    pub fn new(width: u32, height: u32, title: &str) -> Option<Self> {
        unsafe {
            let display = XOpenDisplay(core::ptr::null());
            if display.is_null() {
                printf(b"[X11_WINDOW] ERROR: Failed to open X11 display\n\0".as_ptr());
                return None;
            }

            let screen = XDefaultScreen(display);
            let root_window = XDefaultRootWindow(display);

            let border_width = 0u32;
            let window = XCreateSimpleWindow(
                display,
                root_window,
                0, 0,
                width as c_uint,
                height as c_uint,
                border_width as c_uint,
                0,
                0xFFFFFF,
            );

            if window == 0 {
                printf(b"[X11_WINDOW] ERROR: Failed to create window\n\0".as_ptr());
                XCloseDisplay(display);
                return None;
            }

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

            let wm_delete_message = XInternAtom(display, b"WM_DELETE_WINDOW\0".as_ptr() as *const c_char, 0);
            XSetWMProtocols(display, window, &wm_delete_message as *const _ as *mut c_long, 1);

            let gc = XCreateGC(display, window, 0, core::ptr::null_mut());
            if gc.is_null() {
                printf(b"[X11_WINDOW] ERROR: Failed to create GC\n\0".as_ptr());
                XDestroyWindow(display, window);
                XCloseDisplay(display);
                return None;
            }

            let visual = XDefaultVisual(display, screen);
            let depth = XDefaultDepth(display, screen);

            XMapRaised(display, window);
            XFlush(display);

            printf(b"[X11_WINDOW] Window created successfully\n\0".as_ptr());

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
                running: true,
            })
        }
    }

    pub fn collect_input_events(&mut self) -> X11InputEvents {
        use alloc::vec::Vec;

        let mut events = X11InputEvents {
            mouse_button_events: Vec::new(),
            mouse_motion_events: Vec::new(),
            mouse_wheel_events: Vec::new(),
            keyboard_events: Vec::new(),
        };

        unsafe {
            let mut event: XEvent = core::mem::zeroed();

            while XCheckIfEvent(self.display, &mut event, Some(event_predicate), core::ptr::null_mut()) != 0 {
                match event.type_ {
                    MOTION_NOTIFY => {
                        let motion = &*(core::ptr::addr_of!(event) as *const XMotionEvent);
                        let delta_x = motion.x - self.mouse_x;
                        let delta_y = motion.y - self.mouse_y;
                        
                        self.mouse_x = motion.x;
                        self.mouse_y = motion.y;

                        if delta_x != 0 || delta_y != 0 {
                            events.mouse_motion_events.push(X11MouseMotionEvent {
                                x: motion.x,
                                y: motion.y,
                                delta_x,
                                delta_y,
                            });
                        }
                    }
                    BUTTON_PRESS => {
                        let button = &*(core::ptr::addr_of!(event) as *const XButtonEvent);
                        self.mouse_x = button.x;
                        self.mouse_y = button.y;

                        match button.button {
                            BUTTON1 => {
                                events.mouse_button_events.push(X11MouseButtonEvent {
                                    button: 1,
                                    pressed: true,
                                    x: button.x,
                                    y: button.y,
                                });
                                printf(
                                    b"[X11_INPUT] Left Button PRESSED at (%d, %d)\n\0".as_ptr(),
                                    button.x, button.y
                                );
                            }
                            BUTTON2 => {
                                events.mouse_button_events.push(X11MouseButtonEvent {
                                    button: 2,
                                    pressed: true,
                                    x: button.x,
                                    y: button.y,
                                });
                            }
                            BUTTON3 => {
                                events.mouse_button_events.push(X11MouseButtonEvent {
                                    button: 3,
                                    pressed: true,
                                    x: button.x,
                                    y: button.y,
                                });
                                printf(
                                    b"[X11_INPUT] Right Button PRESSED at (%d, %d)\n\0".as_ptr(),
                                    button.x, button.y
                                );
                            }
                            BUTTON4 => {
                                events.mouse_wheel_events.push(X11MouseWheelEvent {
                                    direction: 1,
                                    x: button.x,
                                    y: button.y,
                                });
                            }
                            BUTTON5 => {
                                events.mouse_wheel_events.push(X11MouseWheelEvent {
                                    direction: -1,
                                    x: button.x,
                                    y: button.y,
                                });
                            }
                            _ => {}
                        }
                    }
                    BUTTON_RELEASE => {
                        let button = &*(core::ptr::addr_of!(event) as *const XButtonEvent);
                        self.mouse_x = button.x;
                        self.mouse_y = button.y;

                        match button.button {
                            BUTTON1 => {
                                events.mouse_button_events.push(X11MouseButtonEvent {
                                    button: 1,
                                    pressed: false,
                                    x: button.x,
                                    y: button.y,
                                });
                                printf(
                                    b"[X11_INPUT] Left Button RELEASED at (%d, %d)\n\0".as_ptr(),
                                    button.x, button.y
                                );
                            }
                            BUTTON2 => {
                                events.mouse_button_events.push(X11MouseButtonEvent {
                                    button: 2,
                                    pressed: false,
                                    x: button.x,
                                    y: button.y,
                                });
                            }
                            BUTTON3 => {
                                events.mouse_button_events.push(X11MouseButtonEvent {
                                    button: 3,
                                    pressed: false,
                                    x: button.x,
                                    y: button.y,
                                });
                                printf(
                                    b"[X11_INPUT] Right Button RELEASED at (%d, %d)\n\0".as_ptr(),
                                    button.x, button.y
                                );
                            }
                            _ => {}
                        }
                    }
                    KEY_PRESS => {
                        let key = &*(core::ptr::addr_of!(event) as *const XKeyEvent);
                        events.keyboard_events.push(X11KeyboardEvent {
                            keycode: key.keycode as u32,
                            pressed: true,
                        });
                    }
                    KEY_RELEASE => {
                        let key = &*(core::ptr::addr_of!(event) as *const XKeyEvent);
                        events.keyboard_events.push(X11KeyboardEvent {
                            keycode: key.keycode as u32,
                            pressed: false,
                        });
                    }
                    CONFIGURE_NOTIFY => {
                        let configure = &*(core::ptr::addr_of!(event) as *const XConfigureEvent);
                        if configure.width as u32 != self.width || configure.height as u32 != self.height {
                            self.width = configure.width as u32;
                            self.height = configure.height as u32;
                        }
                    }
                    CLIENT_MESSAGE => {
                        let client = &*(core::ptr::addr_of!(event) as *const XClientMessageEvent);
                        if client.data.l[0] == self.wm_delete_message {
                            self.running = false;
                        }
                    }
                    DESTROY_NOTIFY => {
                        self.running = false;
                    }
                    _ => {}
                }
            }
        }

        events
    }

    pub fn present(&self, framebuffer: &[u32]) {
        unsafe {
            let image = XCreateImage(
                self.display,
                self.visual,
                self.depth as c_uint,
                2,
                0,
                framebuffer.as_ptr() as *mut c_char,
                self.width as c_uint,
                self.height as c_uint,
                32,
                0,
            );

            if image.is_null() {
                return;
            }

            XPutImage(
                self.display,
                self.window,
                self.gc,
                image,
                0, 0,
                0, 0,
                self.width as c_uint,
                self.height as c_uint,
            );

            XFlush(self.display);
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn mouse_position(&self) -> Vec2 {
        Vec2::new(self.mouse_x as f32, self.mouse_y as f32)
    }
}

impl Drop for X11Window {
    fn drop(&mut self) {
        unsafe {
            if !self.display.is_null() {
                XFreeGC(self.display, self.gc);
                XDestroyWindow(self.display, self.window);
                XFlush(self.display);
                XCloseDisplay(self.display);
            }
        }
    }
}

extern crate alloc;
