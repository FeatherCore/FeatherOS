//! Framebuffer platform implementation for NuttX SIM.

use alloc::vec::Vec;
use core::ffi::c_int;
use fhre::window::{KeyboardEvent, MouseButtonEvent, MouseMotionEvent, Window as WindowTrait, WindowInputEvents};

use super::input::{KeyCode, MouseButton};
use super::runner::InputBridge;

extern "C" {
    fn open(path: *const u8, flags: c_int) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut u8, count: usize) -> isize;
    fn ioctl(fd: c_int, request: c_int, ...) -> c_int;
    fn sim_x11events();
}

const O_RDWR: c_int = 3;
const O_NONBLOCK: c_int = 0x40;
const FBIOGET_VIDEOINFO: c_int = 0x2801;
const FBIOGET_PLANEINFO: c_int = 0x2802;
const X11_KEY_SPACE: u32 = 0x0020;
const X11_KEY_ESCAPE: u32 = 0xff1b;
const X11_KEY_BACKSPACE: u32 = 0xff08;
const X11_KEY_DELETE: u32 = 0xffff;
const X11_KEY_ENTER: u32 = 0xff0d;
const X11_KEY_LEFT: u32 = 0xff51;
const X11_KEY_UP: u32 = 0xff52;
const X11_KEY_RIGHT: u32 = 0xff53;
const X11_KEY_DOWN: u32 = 0xff54;
const X11_KEY_HOME: u32 = 0xff50;
const X11_KEY_END: u32 = 0xff57;

#[repr(C)]
struct VideoInfo {
    fmt: u8,
    xres: u16,
    yres: u16,
    nplanes: u8,
}

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

#[repr(C)]
struct TouchSample {
    npoints: i32,
    point: TouchPoint,
}

#[repr(C)]
struct KeyboardEventNuttX {
    keycode: u16,
    pressed: u8,
}

pub struct Window {
    width: u32,
    height: u32,
    fb_fd: c_int,
    fb_ptr: *mut u32,
    input_fd: c_int,
    kbd_fd: c_int,
    touch_pressed: bool,
    last_touch_x: i32,
    last_touch_y: i32,
}

impl Window {
    pub fn new() -> Option<Self> {
        unsafe {
            let fb_fd = open(b"/dev/fb0\0".as_ptr(), O_RDWR);
            if fb_fd < 0 {
                return None;
            }

            let mut vinfo: VideoInfo = core::mem::zeroed();
            if ioctl(fb_fd, FBIOGET_VIDEOINFO, &mut vinfo as *mut VideoInfo) < 0 {
                close(fb_fd);
                return None;
            }

            let mut pinfo: PlaneInfo = core::mem::zeroed();
            if ioctl(fb_fd, FBIOGET_PLANEINFO, &mut pinfo as *mut PlaneInfo) < 0 {
                close(fb_fd);
                return None;
            }

            let input_fd = open(b"/dev/input0\0".as_ptr(), O_RDWR | O_NONBLOCK);
            let kbd_fd = open(b"/dev/kbd\0".as_ptr(), O_RDWR | O_NONBLOCK);

            Some(Self {
                width: vinfo.xres as u32,
                height: vinfo.yres as u32,
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

    fn read_touch_events(&mut self, events: &mut WindowInputEvents) {
        if self.input_fd < 0 {
            return;
        }

        unsafe {
            sim_x11events();

            let mut sample: TouchSample = core::mem::zeroed();
            let n = read(
                self.input_fd,
                &mut sample as *mut _ as *mut u8,
                core::mem::size_of::<TouchSample>(),
            );

            if n > 0 && sample.npoints > 0 {
                let x = sample.point.x as i32;
                let y = sample.point.y as i32;
                let pressure = sample.point.pressure;

                events.mouse_motion_events.push(MouseMotionEvent {
                    x,
                    y,
                    delta_x: 0,
                    delta_y: 0,
                });

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

    fn read_keyboard_events(&mut self, events: &mut WindowInputEvents) {
        if self.kbd_fd < 0 {
            return;
        }

        unsafe {
            sim_x11events();
            let mut event: KeyboardEventNuttX = core::mem::zeroed();
            let n = read(
                self.kbd_fd,
                &mut event as *mut _ as *mut u8,
                core::mem::size_of::<KeyboardEventNuttX>(),
            );
            if n > 0 {
                events.keyboard_events.push(KeyboardEvent {
                    keycode: event.keycode as u32,
                    pressed: event.pressed != 0,
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
        unsafe {
            core::ptr::copy_nonoverlapping(
                framebuffer.as_ptr(),
                self.fb_ptr,
                ((self.width * self.height) as usize).min(framebuffer.len()),
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

pub struct InputAdapter;

impl InputBridge for InputAdapter {
    fn map_keycode(&self, code: u32) -> Option<KeyCode> {
        match code {
            X11_KEY_SPACE => Some(KeyCode::Space),
            X11_KEY_ESCAPE => Some(KeyCode::Escape),
            X11_KEY_BACKSPACE => Some(KeyCode::Backspace),
            X11_KEY_DELETE => Some(KeyCode::Delete),
            X11_KEY_ENTER => Some(KeyCode::Enter),
            X11_KEY_LEFT => Some(KeyCode::ArrowLeft),
            X11_KEY_UP => Some(KeyCode::ArrowUp),
            X11_KEY_RIGHT => Some(KeyCode::ArrowRight),
            X11_KEY_DOWN => Some(KeyCode::ArrowDown),
            X11_KEY_HOME => Some(KeyCode::Home),
            X11_KEY_END => Some(KeyCode::End),
            0x0061 | 0x0041 => Some(KeyCode::KeyA),
            0x0062 | 0x0042 => Some(KeyCode::KeyB),
            0x0063 | 0x0043 => Some(KeyCode::KeyC),
            0x0064 | 0x0044 => Some(KeyCode::KeyD),
            0x0065 | 0x0045 => Some(KeyCode::KeyE),
            0x0066 | 0x0046 => Some(KeyCode::KeyF),
            0x0067 | 0x0047 => Some(KeyCode::KeyG),
            0x0068 | 0x0048 => Some(KeyCode::KeyH),
            0x0069 | 0x0049 => Some(KeyCode::KeyI),
            0x006a | 0x004a => Some(KeyCode::KeyJ),
            0x006b | 0x004b => Some(KeyCode::KeyK),
            0x006c | 0x004c => Some(KeyCode::KeyL),
            0x006d | 0x004d => Some(KeyCode::KeyM),
            0x006e | 0x004e => Some(KeyCode::KeyN),
            0x006f | 0x004f => Some(KeyCode::KeyO),
            0x0070 | 0x0050 => Some(KeyCode::KeyP),
            0x0071 | 0x0051 => Some(KeyCode::KeyQ),
            0x0072 | 0x0052 => Some(KeyCode::KeyR),
            0x0073 | 0x0053 => Some(KeyCode::KeyS),
            0x0074 | 0x0054 => Some(KeyCode::KeyT),
            0x0075 | 0x0055 => Some(KeyCode::KeyU),
            0x0076 | 0x0056 => Some(KeyCode::KeyV),
            0x0077 | 0x0057 => Some(KeyCode::KeyW),
            0x0078 | 0x0058 => Some(KeyCode::KeyX),
            0x0079 | 0x0059 => Some(KeyCode::KeyY),
            0x007a | 0x005a => Some(KeyCode::KeyZ),
            _ => None,
        }
    }

    fn map_mouse_button(&self, button: u32) -> Option<MouseButton> {
        match button {
            1 => Some(MouseButton::Left),
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::Right),
            _ => None,
        }
    }
}

impl Default for InputAdapter {
    fn default() -> Self {
        Self
    }
}
