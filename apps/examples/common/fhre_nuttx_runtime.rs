#![allow(dead_code)]

use alloc::vec::Vec;
use core::ffi::c_int;
use fhre::{
    DirtyRegion, FrameStats, ImageView, InputQueue, KeyCode, KeyEvent, LayerBudget, Point,
    PointerEvent, PointerId, PointerPhase, PresentStats, ResourceLoader, Surface,
};

const O_RDONLY: c_int = 1;
const O_RDWR: c_int = 3;
const O_NONBLOCK: c_int = 1 << 6;
const FBIOGET_VIDEOINFO: c_int = 0x2801;
const FBIOGET_PLANEINFO: c_int = 0x2802;
const FBIOPAN_DISPLAY: c_int = 0x2818;
const CLOCK_MONOTONIC: c_int = 1;
const POLLOUT: u32 = 0x04;
const TOUCH_DOWN: u8 = 1 << 0;
const TOUCH_MOVE: u8 = 1 << 1;
const TOUCH_UP: u8 = 1 << 2;
const KEYBOARD_PRESS: u32 = 0;
const XK_BACKSPACE: u32 = 0xff08;
const XK_RETURN: u32 = 0xff0d;
const XK_ESCAPE: u32 = 0xff1b;
const XK_HOME: u32 = 0xff50;
const XK_LEFT: u32 = 0xff51;
const XK_UP: u32 = 0xff52;
const XK_RIGHT: u32 = 0xff53;
const XK_DOWN: u32 = 0xff54;
const BACKBUFFER_BYTES: usize = 640 * 480 * 4;

static mut BACKBUFFER: [u8; BACKBUFFER_BYTES] = [0; BACKBUFFER_BYTES];

#[repr(C)]
struct VideoInfo {
    fmt: u8,
    xres: u16,
    yres: u16,
    nplanes: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
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
struct PollFd {
    fd: c_int,
    events: u32,
    revents: u32,
    arg: *mut u8,
    cb: *mut u8,
    priv_data: *mut u8,
}

#[repr(C)]
struct Timespec {
    tv_sec: u32,
    tv_nsec: isize,
}

#[repr(C)]
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
struct TouchSample {
    npoints: c_int,
    point: [TouchPoint; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KeyboardEvent {
    event_type: u32,
    code: u32,
}

extern "C" {
    fn open(path: *const u8, flags: c_int, ...) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn ioctl(fd: c_int, request: c_int, ...) -> c_int;
    fn poll(fds: *mut PollFd, nfds: usize, timeout: c_int) -> c_int;
    fn read(fd: c_int, buffer: *mut u8, count: usize) -> isize;
    fn usleep(usec: u32) -> c_int;
    fn clock_gettime(clockid: c_int, tp: *mut Timespec) -> c_int;
    fn sim_x11events();
}

pub struct NuttxFramebuffer {
    fd: c_int,
    fbmem: *mut u8,
    frame_len: usize,
    plane: PlaneInfo,
    width: u16,
    height: u16,
    fmt: u8,
    pub surface: Surface,
    pan_enabled: bool,
    software_buffered: bool,
    draw_yoffset: u32,
}

impl NuttxFramebuffer {
    pub fn open() -> Option<Self> {
        unsafe {
            let fd = open(b"/dev/fb0\0".as_ptr(), O_RDWR);
            if fd < 0 {
                return None;
            }

            let mut vinfo: VideoInfo = core::mem::zeroed();
            if ioctl(fd, FBIOGET_VIDEOINFO, &mut vinfo as *mut VideoInfo) < 0 {
                close(fd);
                return None;
            }

            let mut pinfo: PlaneInfo = core::mem::zeroed();
            if ioctl(fd, FBIOGET_PLANEINFO, &mut pinfo as *mut PlaneInfo) < 0 {
                close(fd);
                return None;
            }

            if pinfo.fbmem.is_null() || pinfo.stride == 0 {
                close(fd);
                return None;
            }

            let frame_len = (pinfo.stride as usize).saturating_mul(vinfo.yres as usize);
            let has_pan_buffer = pinfo.yres_virtual >= (vinfo.yres as u32).saturating_mul(2)
                && pinfo.fblen >= frame_len.saturating_mul(2);
            let draw_yoffset = if has_pan_buffer && pinfo.yoffset == 0 {
                vinfo.yres as u32
            } else {
                0
            };
            let (pixels, software_buffered) = if has_pan_buffer {
                (
                    pinfo.fbmem.add(draw_yoffset as usize * pinfo.stride as usize),
                    false,
                )
            } else if frame_len <= BACKBUFFER_BYTES {
                (core::ptr::addr_of_mut!(BACKBUFFER).cast::<u8>(), true)
            } else {
                (pinfo.fbmem, false)
            };

            let mut surface = Surface::from_raw(
                pixels,
                vinfo.xres,
                vinfo.yres,
                pinfo.stride as usize,
                pinfo.bpp,
                vinfo.fmt,
            );
            surface.set_layer_budget(LayerBudget::DEMO);

            Some(Self {
                fd,
                fbmem: pinfo.fbmem,
                frame_len,
                plane: pinfo,
                width: vinfo.xres,
                height: vinfo.yres,
                fmt: vinfo.fmt,
                surface,
                pan_enabled: has_pan_buffer,
                software_buffered,
                draw_yoffset,
            })
        }
    }

    pub const fn width(&self) -> u16 {
        self.width
    }

    pub const fn height(&self) -> u16 {
        self.height
    }

    pub fn prepare_dirty_frame<const N: usize>(&mut self, dirty: &DirtyRegion<N>) -> u32 {
        if !self.pan_enabled || self.fbmem.is_null() || self.frame_len == 0 || dirty.is_empty() {
            return 0;
        }

        if dirty.overflowed() {
            return 0;
        }

        self.copy_visible_to_draw_page()
    }

    pub fn present(&mut self) -> PresentStats {
        if self.pan_enabled {
            if !self.can_pan() {
                return PresentStats::skipped();
            }

            self.plane.yoffset = self.draw_yoffset;
            let ret = unsafe {
                ioctl(
                    self.fd,
                    FBIOPAN_DISPLAY,
                    &mut self.plane as *mut PlaneInfo,
                )
            };
            if ret >= 0 {
                self.draw_yoffset = if self.draw_yoffset == 0 {
                    self.height as u32
                } else {
                    0
                };
                self.reset_draw_surface();
                return PresentStats::pan();
            }
            return PresentStats::skipped();
        }

        if !self.software_buffered || self.fbmem.is_null() || self.frame_len == 0 {
            return PresentStats::skipped();
        }

        let bytes = self.frame_len.min(BACKBUFFER_BYTES);
        unsafe {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!(BACKBUFFER).cast::<u8>(),
                self.fbmem,
                bytes,
            );
        }
        PresentStats::copied(bytes)
    }

    fn copy_visible_to_draw_page(&mut self) -> u32 {
        let visible_yoffset = self.visible_yoffset();
        unsafe {
            let src = self
                .fbmem
                .add(visible_yoffset as usize * self.plane.stride as usize);
            let dst = self
                .fbmem
                .add(self.draw_yoffset as usize * self.plane.stride as usize);
            core::ptr::copy_nonoverlapping(src, dst, self.frame_len);
        }
        self.frame_len.min(u32::MAX as usize) as u32
    }

    fn visible_yoffset(&self) -> u32 {
        if self.draw_yoffset == 0 {
            self.height as u32
        } else {
            0
        }
    }

    fn can_pan(&self) -> bool {
        let mut pfd = PollFd {
            fd: self.fd,
            events: POLLOUT,
            revents: 0,
            arg: core::ptr::null_mut(),
            cb: core::ptr::null_mut(),
            priv_data: core::ptr::null_mut(),
        };
        unsafe { poll(&mut pfd as *mut PollFd, 1, 0) > 0 && (pfd.revents & POLLOUT) != 0 }
    }

    fn reset_draw_surface(&mut self) {
        unsafe {
            let ptr = self
                .fbmem
                .add(self.draw_yoffset as usize * self.plane.stride as usize);
            self.surface = Surface::from_raw(
                ptr,
                self.width,
                self.height,
                self.plane.stride as usize,
                self.plane.bpp,
                self.fmt,
            );
            self.surface.set_layer_budget(LayerBudget::DEMO);
        }
    }
}

impl Drop for NuttxFramebuffer {
    fn drop(&mut self) {
        unsafe {
            let _ = close(self.fd);
        }
    }
}

pub struct NuttxInput {
    touch_fd: c_int,
    key_fd: c_int,
    last_touch: Option<Point>,
}

impl NuttxInput {
    pub fn open() -> Self {
        unsafe {
            Self {
                touch_fd: open(b"/dev/input0\0".as_ptr(), O_RDONLY | O_NONBLOCK),
                key_fd: open(b"/dev/kbd\0".as_ptr(), O_RDONLY | O_NONBLOCK),
                last_touch: None,
            }
        }
    }

    pub fn pump<const N: usize>(&mut self, queue: &mut InputQueue<N>, timestamp_us: u64) -> usize {
        self.pump_touch(queue, timestamp_us) + self.pump_keyboard(queue, timestamp_us)
    }

    fn pump_touch<const N: usize>(&mut self, queue: &mut InputQueue<N>, timestamp_us: u64) -> usize {
        if self.touch_fd < 0 {
            return 0;
        }

        let mut pushed = 0;
        let mut pending_move = None;
        for _ in 0..16 {
            let mut sample = TouchSample {
                npoints: 0,
                point: [TouchPoint {
                    id: 0,
                    flags: 0,
                    x: 0,
                    y: 0,
                    h: 0,
                    w: 0,
                    gesture: 0,
                    pressure: 0,
                    timestamp: 0,
                }],
            };
            let read_len = unsafe {
                read(
                    self.touch_fd,
                    &mut sample as *mut TouchSample as *mut u8,
                    core::mem::size_of::<TouchSample>(),
                )
            };
            if read_len < core::mem::size_of::<TouchSample>() as isize || sample.npoints <= 0 {
                break;
            }

            if let Some(event) = self.touch_to_event(sample.point[0], timestamp_us) {
                match event {
                    fhre::InputEvent::Pointer(pointer) if pointer.phase == PointerPhase::Move => {
                        pending_move = Some(event);
                    }
                    _ => {
                        if let Some(move_event) = pending_move.take() {
                            if queue.push(move_event) {
                                pushed += 1;
                            }
                        }
                        if queue.push(event) {
                            pushed += 1;
                        }
                    }
                }
            }
        }
        if let Some(move_event) = pending_move {
            if queue.push(move_event) {
                pushed += 1;
            }
        }
        pushed
    }

    fn pump_keyboard<const N: usize>(&mut self, queue: &mut InputQueue<N>, timestamp_us: u64) -> usize {
        if self.key_fd < 0 {
            return 0;
        }

        let mut pushed = 0;
        for _ in 0..16 {
            let mut event = KeyboardEvent {
                event_type: 0,
                code: 0,
            };
            let read_len = unsafe {
                read(
                    self.key_fd,
                    &mut event as *mut KeyboardEvent as *mut u8,
                    core::mem::size_of::<KeyboardEvent>(),
                )
            };
            if read_len < core::mem::size_of::<KeyboardEvent>() as isize {
                break;
            }

            let key = KeyEvent::new(
                map_key_code(event.code),
                event.event_type == KEYBOARD_PRESS,
                timestamp_us,
            );
            if queue.push(fhre::InputEvent::Key(key)) {
                pushed += 1;
            }
        }
        pushed
    }

    fn touch_to_event(&mut self, point: TouchPoint, timestamp_us: u64) -> Option<fhre::InputEvent> {
        let phase = if point.flags & TOUCH_DOWN != 0 {
            PointerPhase::Down
        } else if point.flags & TOUCH_UP != 0 {
            PointerPhase::Up
        } else if point.flags & TOUCH_MOVE != 0 {
            PointerPhase::Move
        } else {
            return None;
        };

        let position = Point::new(point.x as i32, point.y as i32);
        let delta = if let Some(previous) = self.last_touch {
            Point::new(position.x - previous.x, position.y - previous.y)
        } else {
            Point::new(0, 0)
        };
        match phase {
            PointerPhase::Down | PointerPhase::Move => self.last_touch = Some(position),
            PointerPhase::Up | PointerPhase::Cancel => self.last_touch = None,
        }

        let timestamp = if point.timestamp == 0 {
            timestamp_us
        } else {
            point.timestamp
        };
        Some(fhre::InputEvent::Pointer(
            PointerEvent::new(PointerId(point.id), phase, position, timestamp).with_delta(delta),
        ))
    }
}

impl Drop for NuttxInput {
    fn drop(&mut self) {
        unsafe {
            if self.touch_fd >= 0 {
                let _ = close(self.touch_fd);
            }
            if self.key_fd >= 0 {
                let _ = close(self.key_fd);
            }
        }
    }
}

pub struct NuttxResourceLoader {
    prefix: &'static [u8],
}

impl NuttxResourceLoader {
    pub const fn new(prefix: &'static [u8]) -> Self {
        Self { prefix }
    }
}

impl ResourceLoader for NuttxResourceLoader {
    fn load(&mut self, path: &[u8], out: &mut Vec<u8>) -> bool {
        out.clear();
        let mut full_path = Vec::new();
        if path.first().copied() == Some(b'/') {
            full_path.extend_from_slice(path);
        } else {
            full_path.extend_from_slice(self.prefix);
            full_path.extend_from_slice(path);
        }
        if full_path.last().copied() != Some(0) {
            full_path.push(0);
        }

        let fd = unsafe { open(full_path.as_ptr(), O_RDONLY) };
        if fd < 0 {
            return false;
        }

        let mut ok = true;
        let mut chunk = [0u8; 512];
        loop {
            let n = unsafe { read(fd, chunk.as_mut_ptr(), chunk.len()) };
            if n < 0 {
                ok = false;
                break;
            }
            if n == 0 {
                break;
            }
            out.extend_from_slice(&chunk[..n as usize]);
        }
        unsafe {
            let _ = close(fd);
        }
        ok
    }
}

pub fn poll_sim_events() {
    unsafe {
        sim_x11events();
    }
}

pub fn now_us() -> u64 {
    let mut ts = Timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let ret = unsafe { clock_gettime(CLOCK_MONOTONIC, &mut ts as *mut Timespec) };
    if ret < 0 {
        0
    } else {
        (ts.tv_sec as u64)
            .saturating_mul(1_000_000)
            .saturating_add((ts.tv_nsec.max(0) as u64) / 1_000)
    }
}

pub fn elapsed_us(start: u64, end: u64) -> u32 {
    end.saturating_sub(start).min(u32::MAX as u64) as u32
}

pub fn sleep_remaining(frame_start: u64, frame_stats: FrameStats) {
    let elapsed = elapsed_us(frame_start, now_us());
    let sleep_us = frame_stats
        .sleep_us
        .saturating_sub(elapsed.saturating_sub(frame_stats.frame_work_us));
    if sleep_us > 0 {
        unsafe {
            let _ = usleep(sleep_us);
        }
    }
}

fn map_key_code(code: u32) -> KeyCode {
    match code {
        XK_BACKSPACE => KeyCode::Back,
        XK_RETURN => KeyCode::Enter,
        XK_ESCAPE => KeyCode::Escape,
        XK_HOME => KeyCode::Home,
        XK_LEFT => KeyCode::Left,
        XK_UP => KeyCode::Up,
        XK_RIGHT => KeyCode::Right,
        XK_DOWN => KeyCode::Down,
        0x20..=0x7e => KeyCode::Char(code),
        _ => KeyCode::Unknown((code & 0xffff) as u16),
    }
}

pub fn cached_image_view(cache: &mut fhre::ImageCache, image: fhre::ImageId) -> Option<ImageView> {
    cache.view(image)
}
