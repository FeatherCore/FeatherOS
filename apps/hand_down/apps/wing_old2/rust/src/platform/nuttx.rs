use alloc::{boxed::Box, vec::Vec};

use ::core::ffi::c_int;
use ::core::ptr::null_mut;

use crate::app::{
    PlatformTask, PlatformTaskCapabilities, PlatformTaskExit, TaskLaunchError, TaskPriority,
    TaskStackSize, TaskStdio, TaskSurface, COMMAND_ARG_CAPACITY,
};
use crate::input::{InputState, KeyCode};
use crate::math::Rect;
use crate::render::{
    register_bootstrap_svg_icons, register_default_app_svg_icons, register_settings_app_svg_icons,
    register_surface_demo_app_svg_icons, register_system_app_svg_icons,
    register_terminal_app_svg_icons, DirtyRegion, FontStore, ResourcePartition, SvgStore,
    VectorIcon,
};
use crate::resource_file::{inspect_resource_header, RuntimeResourceId, RuntimeResourceInfo};
use crate::settings::{
    SettingsBlock, SettingsEvent, SettingsMemoryStore, SettingsMessage, SettingsSnapshot,
    SettingsStorageSource, SETTINGS_BLOCK_SIZE,
};
use crate::surface::{
    SurfaceCapabilities, SurfaceClosedEvent, SurfaceDescriptor, SurfaceDirtyEvent, SurfaceEvent,
    SurfaceFrame, SurfaceFrameInfoAbi, SurfaceHandle, SurfaceInputPolicy, SurfacePixelFormat,
    SurfacePointerEvent, SurfaceTransport,
};

use super::Platform;

extern "C" {
    fn open(path: *const u8, flags: c_int, ...) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut u8, count: usize) -> isize;
    fn write(fd: c_int, buf: *const u8, count: usize) -> isize;
    fn fsync(fd: c_int) -> c_int;
    fn ioctl(fd: c_int, request: c_int, ...) -> c_int;
    fn wing_task_spawn(config: *const WingTaskLaunchConfig) -> c_int;
    fn wing_task_close(pid: c_int, frame: u32, token: u32) -> c_int;
    fn wing_surface_frame_reset() -> c_int;
    fn wing_surface_frame_resolve(
        handle: u32,
        token: u32,
        info: *mut SurfaceFrameInfoAbi,
    ) -> c_int;
    fn wing_surface_dirty_open() -> c_int;
    fn wing_surface_dirty_receive(
        queue: c_int,
        message: *mut WingSurfaceDirtyMessage,
    ) -> c_int;
    fn wing_surface_dirty_close(queue: c_int) -> c_int;
    fn wing_surface_input_open() -> c_int;
    fn wing_surface_input_send(
        queue: c_int,
        handle: u32,
        token: u32,
        x: i16,
        y: i16,
        event: u8,
        buttons: u8,
    ) -> c_int;
    fn wing_surface_input_close(queue: c_int) -> c_int;
    fn wing_settings_open() -> c_int;
    fn wing_settings_receive(queue: c_int, message: *mut SettingsMessage) -> c_int;
    fn wing_settings_close(queue: c_int) -> c_int;
    fn sim_x11events();
    fn sched_yield() -> c_int;
    fn usleep(usec: u32) -> c_int;
    fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int;
}

const O_RDWR: c_int = 3;
const O_RDONLY: c_int = 1;
const O_WRONLY: c_int = 2;
const O_CREAT: c_int = 1 << 2;
const O_TRUNC: c_int = 1 << 5;
const O_NONBLOCK: c_int = 0x40;
const WING_FILE_MODE: c_int = 0o666;
const FBIOGET_VIDEOINFO: c_int = 0x2801;
const FBIOGET_PLANEINFO: c_int = 0x2802;

const TOUCH_GESTURE_VALID: u8 = 1 << 7;
const TOUCH_SLIDE_UP: u16 = 0x01;
const TOUCH_SLIDE_DOWN: u16 = 0x02;

const WING_TASK_STDIO_INHERIT: c_int = 0;
const WING_TASK_STDIO_NULL: c_int = 1;
const WING_TASK_DEFAULT_PRIORITY: c_int = -1;
const WING_TASK_DEFAULT_STACK: c_int = 0;
const WING_TASK_DETACHED_SURFACE: c_int = 0;
const WING_TASK_MANAGED_SURFACE: c_int = 1;
const WING_SURFACE_FORMAT_RGB565: c_int = 1;
const WING_SURFACE_FORMAT_ARGB8888: c_int = 2;
const WING_SURFACE_TRANSPORT_SHARED_MEMORY: c_int = 1;
const WING_SURFACE_TRANSPORT_STREAM_TEXTURE: c_int = 2;
const WING_SURFACE_INPUT_NONE: c_int = 0;
const WING_SURFACE_INPUT_POINTER: c_int = 1;
const WING_SURFACE_INPUT_POINTER_KEYBOARD: c_int = 2;
const WAIT_ANY: c_int = -1;
const WNOHANG: c_int = 1 << 1;

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
const X11_KEY_SPACE: u32 = 0x0020;

const WING_RESOURCE_MANIFEST_PATH: &[u8] = b"/etc/wing/wing_manifest.bin\0";
const WING_RESOURCE_PAYLOAD_PATH: &[u8] = b"/etc/wing/wing_payload.bin\0";
const WING_RESOURCE_MANIFEST_CAPACITY: usize = 4096;
const WING_RESOURCE_PAYLOAD_CAPACITY: usize = 3 * 1024 * 1024;
const WING_RESOURCE_PROBE_CAPACITY: usize = 512;
const WING_RUNTIME_IMAGE_CAPACITY: usize = 3 * 1024 * 1024;
const WING_RUNTIME_FONT_CAPACITY: usize = 12 * 1024 * 1024;
const WING_RUNTIME_GLYPH_WORKSET_CAPACITY: usize = 512;
const WING_RUNTIME_SVG_ICON_CAPACITY: usize = 8 * 1024;
const WING_RUNTIME_FILE_CHUNK: usize = 1024;
const WING_RESOURCE_STATE_UNINIT: u8 = 0;
const WING_RESOURCE_STATE_BUILTIN: u8 = 1;
const WING_RESOURCE_STATE_FILE: u8 = 2;
const WING_SETTINGS_PRIMARY_PATH: &[u8] = b"/data/wing_settings.bin\0";
const WING_SETTINGS_FALLBACK_PATH: &[u8] = b"/tmp/wing_settings.bin\0";

#[derive(Clone, Copy)]
struct RuntimeSvgIconResource {
    icon: VectorIcon,
    path: &'static [u8],
}

const RUNTIME_SVG_ICON_RESOURCES: &[RuntimeSvgIconResource] = &[
    RuntimeSvgIconResource {
        icon: VectorIcon::Wifi,
        path: b"/etc/wing/resource/icons/shell/wifi.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Bluetooth,
        path: b"/etc/wing/resource/icons/shell/bluetooth.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Phone,
        path: b"/etc/wing/resource/icons/shell/phone.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Chat,
        path: b"/etc/wing/resource/icons/shell/chat.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Settings,
        path: b"/etc/wing/resource/icons/shell/settings.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Camera,
        path: b"/etc/wing/resource/icons/shell/camera.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Flashlight,
        path: b"/etc/wing/resource/icons/shell/flashlight.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Airplane,
        path: b"/etc/wing/resource/icons/shell/airplane.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Moon,
        path: b"/etc/wing/resource/icons/shell/moon.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Sync,
        path: b"/etc/wing/resource/icons/shell/sync.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Mail,
        path: b"/etc/wing/resource/icons/shell/mail.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Cloud,
        path: b"/etc/wing/resource/icons/shell/cloud.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Folder,
        path: b"/etc/wing/resource/icons/shell/folder.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Music,
        path: b"/etc/wing/resource/icons/shell/music.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Play,
        path: b"/etc/wing/resource/icons/shell/play.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Wing,
        path: b"/etc/wing/resource/icons/shell/wing.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Check,
        path: b"/etc/wing/resource/icons/shell/check.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Close,
        path: b"/etc/wing/resource/icons/shell/close.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Alert,
        path: b"/etc/wing/resource/icons/shell/alert.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::More,
        path: b"/etc/wing/resource/icons/shell/more.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::System,
        path: b"/etc/wing/resource/icons/shell/system.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Terminal,
        path: b"/etc/wing/resource/icons/shell/terminal.svg\0",
    },
    RuntimeSvgIconResource {
        icon: VectorIcon::Surface,
        path: b"/etc/wing/resource/icons/shell/surface.svg\0",
    },
];

static mut WING_RESOURCE_MANIFEST_BUFFER: [u8; WING_RESOURCE_MANIFEST_CAPACITY] =
    [0; WING_RESOURCE_MANIFEST_CAPACITY];
static mut WING_RESOURCE_PAYLOAD_BUFFER: [u8; WING_RESOURCE_PAYLOAD_CAPACITY] =
    [0; WING_RESOURCE_PAYLOAD_CAPACITY];
static mut WING_RESOURCE_PROBE_BUFFER: [u8; WING_RESOURCE_PROBE_CAPACITY] =
    [0; WING_RESOURCE_PROBE_CAPACITY];
static mut WING_RESOURCE_MANIFEST_LEN: usize = 0;
static mut WING_RESOURCE_PAYLOAD_LEN: usize = 0;
static mut WING_RESOURCE_STATE: u8 = WING_RESOURCE_STATE_UNINIT;

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
struct KeyboardEvent {
    keycode: u16,
    pressed: u8,
}

#[repr(C)]
struct WingTaskLaunchConfig {
    program: *const u8,
    argv: *const *mut u8,
    stdio: c_int,
    priority: c_int,
    stack_size: c_int,
    surface: c_int,
    surface_id: c_int,
    surface_width: c_int,
    surface_height: c_int,
    surface_stride: c_int,
    surface_format: c_int,
    surface_buffers: c_int,
    surface_transport: c_int,
    surface_input: c_int,
    surface_token: c_int,
    surface_frame: c_int,
    surface_dirty: c_int,
    surface_input_handle: c_int,
    settings_valid: c_int,
    settings_theme: c_int,
    settings_preview: c_int,
    settings_brightness: c_int,
    settings_haptic: c_int,
    settings_reduce_motion: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct WingSurfaceDirtyMessage {
    handle: u32,
    token: u32,
    x: i16,
    y: i16,
    w: u16,
    h: u16,
}

pub struct NuttXPlatform {
    width: u16,
    height: u16,
    fb_fd: c_int,
    fb_ptr: *mut u32,
    fb_stride_pixels: usize,
    input_fd: c_int,
    kbd_fd: c_int,
    surface_dirty_queue: c_int,
    surface_input_queue: c_int,
    settings_queue: c_int,
    settings_store: SettingsMemoryStore,
    settings_source: SettingsStorageSource,
    touch_pressed: bool,
    last_touch_x: i32,
    last_touch_y: i32,
}

impl NuttXPlatform {
    pub fn new() -> Option<Self> {
        unsafe {
            let fb_fd = open(b"/dev/fb0\0".as_ptr(), O_RDWR);
            if fb_fd < 0 {
                return None;
            }

            let mut vinfo: VideoInfo = ::core::mem::zeroed();
            if ioctl(fb_fd, FBIOGET_VIDEOINFO, &mut vinfo as *mut VideoInfo) < 0 {
                close(fb_fd);
                return None;
            }

            let mut pinfo: PlaneInfo = ::core::mem::zeroed();
            if ioctl(fb_fd, FBIOGET_PLANEINFO, &mut pinfo as *mut PlaneInfo) < 0 {
                close(fb_fd);
                return None;
            }

            let input_fd = open(b"/dev/input0\0".as_ptr(), O_RDWR | O_NONBLOCK);
            let kbd_fd = open(b"/dev/kbd\0".as_ptr(), O_RDWR | O_NONBLOCK);
            wing_surface_frame_reset();
            let surface_dirty_queue = wing_surface_dirty_open();
            let surface_input_queue = wing_surface_input_open();
            let settings_queue = wing_settings_open();

            Some(Self {
                width: vinfo.xres,
                height: vinfo.yres,
                fb_fd,
                fb_ptr: pinfo.fbmem as *mut u32,
                fb_stride_pixels: if pinfo.stride >= 4 {
                    pinfo.stride as usize / 4
                } else {
                    vinfo.xres as usize
                },
                input_fd,
                kbd_fd,
                surface_dirty_queue,
                surface_input_queue,
                settings_queue,
                settings_store: SettingsMemoryStore::default(),
                settings_source: SettingsStorageSource::Memory,
                touch_pressed: false,
                last_touch_x: 0,
                last_touch_y: 0,
            })
        }
    }

    fn read_touch(&mut self, input: &mut InputState) {
        if self.input_fd < 0 {
            return;
        }

        unsafe {
            sim_x11events();

            let mut sample: TouchSample = ::core::mem::zeroed();
            let n = read(
                self.input_fd,
                &mut sample as *mut _ as *mut u8,
                ::core::mem::size_of::<TouchSample>(),
            );

            if n <= 0 || sample.npoints <= 0 {
                return;
            }

            let x = sample.point.x as i32;
            let y = sample.point.y as i32;
            input.move_pointer(x, y);

            if (sample.point.flags & TOUCH_GESTURE_VALID) != 0 {
                match sample.point.gesture {
                    TOUCH_SLIDE_UP => input.wheel_delta = input.wheel_delta.saturating_add(1),
                    TOUCH_SLIDE_DOWN => input.wheel_delta = input.wheel_delta.saturating_sub(1),
                    _ => {}
                }
            }

            if sample.point.pressure > 0 {
                if !self.touch_pressed {
                    self.touch_pressed = true;
                    input.set_primary(true, x, y);
                }
                self.last_touch_x = x;
                self.last_touch_y = y;
            } else if self.touch_pressed {
                self.touch_pressed = false;
                input.set_primary(false, self.last_touch_x, self.last_touch_y);
            }
        }
    }

    fn read_keyboard(&mut self, input: &mut InputState) {
        if self.kbd_fd < 0 {
            return;
        }

        unsafe {
            sim_x11events();

            let mut event: KeyboardEvent = ::core::mem::zeroed();
            let n = read(
                self.kbd_fd,
                &mut event as *mut _ as *mut u8,
                ::core::mem::size_of::<KeyboardEvent>(),
            );

            if n <= 0 {
                return;
            }

            if let Some(key) = map_keycode(event.keycode as u32) {
                if event.pressed != 0 {
                    input.press_key(key);
                } else {
                    input.release_key(key);
                }
            }
        }
    }

    fn read_surface_event(&mut self) -> Option<SurfaceEvent> {
        if self.surface_dirty_queue < 0 {
            return None;
        }

        unsafe {
            let mut message: WingSurfaceDirtyMessage = ::core::mem::zeroed();
            let ret = wing_surface_dirty_receive(
                self.surface_dirty_queue,
                &mut message as *mut WingSurfaceDirtyMessage,
            );

            if ret <= 0 {
                return None;
            }

            let handle = SurfaceHandle(message.handle);
            if message.w == 0 && message.h == 0 {
                Some(SurfaceEvent::Closed(SurfaceClosedEvent::new(
                    handle,
                    message.token,
                )))
            } else {
                Some(SurfaceEvent::Dirty(SurfaceDirtyEvent::new(
                    handle,
                    message.token,
                    Rect::new(message.x as i32, message.y as i32, message.w, message.h),
                )))
            }
        }
    }

    fn read_settings_event(&mut self) -> Option<SettingsEvent> {
        if self.settings_queue < 0 {
            return None;
        }

        unsafe {
            let mut message: SettingsMessage = ::core::mem::zeroed();
            let ret = wing_settings_receive(
                self.settings_queue,
                &mut message as *mut SettingsMessage,
            );

            if ret <= 0 {
                return None;
            }

            message.event()
        }
    }

    fn present_rect(&mut self, framebuffer: &[u32], rect: crate::math::Rect) {
        let x0 = rect.x.max(0) as usize;
        let y0 = rect.y.max(0) as usize;
        let x1 = (rect.x + rect.w as i32)
            .min(self.width as i32)
            .max(0) as usize;
        let y1 = (rect.y + rect.h as i32)
            .min(self.height as i32)
            .max(0) as usize;

        if x0 >= x1 || y0 >= y1 {
            return;
        }

        unsafe {
            for y in y0..y1 {
                let src_start = y * self.width as usize + x0;
                let dst_start = y * self.fb_stride_pixels + x0;
                let count = (x1 - x0).min(framebuffer.len().saturating_sub(src_start));
                if count == 0 {
                    continue;
                }
                ::core::ptr::copy_nonoverlapping(
                    framebuffer.as_ptr().add(src_start),
                    self.fb_ptr.add(dst_start),
                    count,
                );
            }
        }
    }
}

impl Platform for NuttXPlatform {
    fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn poll_input(&mut self, input: &mut InputState) {
        unsafe {
            sched_yield();
        }
        self.read_touch(input);
        self.read_keyboard(input);
    }

    fn present(&mut self, framebuffer: &[u32], dirty: DirtyRegion) {
        if self.fb_ptr.is_null() {
            return;
        }

        for rect in dirty.rects() {
            self.present_rect(framebuffer, *rect);
        }
    }

    fn poll_surface_event(&mut self) -> Option<SurfaceEvent> {
        self.read_surface_event()
    }

    fn poll_settings_event(&mut self) -> Option<SettingsEvent> {
        self.read_settings_event()
    }

    fn load_settings_snapshot(&mut self) -> Option<SettingsSnapshot> {
        if let Some(snapshot) = load_file_settings_snapshot(WING_SETTINGS_PRIMARY_PATH.as_ptr()) {
            self.settings_source = SettingsStorageSource::DataFile;
            self.settings_store.save(snapshot);
            Some(snapshot)
        } else if let Some(snapshot) =
            load_file_settings_snapshot(WING_SETTINGS_FALLBACK_PATH.as_ptr())
        {
            self.settings_source = SettingsStorageSource::TempFile;
            self.settings_store.save(snapshot);
            Some(snapshot)
        } else {
            self.settings_source = SettingsStorageSource::Memory;
            self.settings_store.load()
        }
    }

    fn save_settings_snapshot(&mut self, snapshot: SettingsSnapshot) -> bool {
        if save_file_settings_snapshot(WING_SETTINGS_PRIMARY_PATH.as_ptr(), snapshot) {
            self.settings_source = SettingsStorageSource::DataFile;
            self.settings_store.save(snapshot);
            true
        } else if save_file_settings_snapshot(WING_SETTINGS_FALLBACK_PATH.as_ptr(), snapshot) {
            self.settings_source = SettingsStorageSource::TempFile;
            self.settings_store.save(snapshot);
            true
        } else {
            self.settings_source = SettingsStorageSource::Memory;
            self.settings_store.save(snapshot)
        }
    }

    fn settings_storage_source(&self) -> SettingsStorageSource {
        self.settings_source
    }

    fn resource_partition(&self) -> ResourcePartition {
        nuttx_resource_partition()
    }

    fn inspect_runtime_resource(&mut self, id: RuntimeResourceId) -> RuntimeResourceInfo {
        nuttx_runtime_resource_info(id)
    }

    fn load_runtime_resource(&mut self, id: RuntimeResourceId) -> Option<Vec<u8>> {
        let capacity = if id.is_shell_wallpaper() {
            WING_RUNTIME_IMAGE_CAPACITY
        } else if id.is_default_font() {
            WING_RUNTIME_FONT_CAPACITY
        } else if id.is_shell_glyph_workset() {
            WING_RUNTIME_GLYPH_WORKSET_CAPACITY
        } else {
            return None;
        };
        unsafe { read_file_to_vec(id.path().as_ptr(), capacity) }
    }

    fn load_runtime_resource_file(
        &mut self,
        path: &'static [u8],
        capacity: usize,
    ) -> Option<Vec<u8>> {
        unsafe { read_file_to_vec(path.as_ptr(), capacity) }
    }

    fn send_surface_input(&mut self, event: SurfacePointerEvent) -> bool {
        if self.surface_input_queue < 0 {
            return false;
        }

        unsafe {
            wing_surface_input_send(
                self.surface_input_queue,
                event.handle.raw(),
                event.token,
                event.point.x as i16,
                event.point.y as i16,
                event.kind.raw(),
                event.buttons,
            ) == 0
        }
    }

    fn poll_task_exit(&mut self) -> Option<PlatformTaskExit> {
        unsafe {
            let mut status: c_int = 0;
            let pid = waitpid(WAIT_ANY, &mut status as *mut c_int, WNOHANG);
            if pid > 0 {
                Some(PlatformTaskExit {
                    pid: pid as i32,
                    status: status as i32,
                })
            } else {
                None
            }
        }
    }

    fn resolve_surface_frame(
        &mut self,
        handle: SurfaceHandle,
        token: u32,
    ) -> Option<SurfaceFrame> {
        unsafe {
            let mut info: SurfaceFrameInfoAbi = ::core::mem::zeroed();
            if wing_surface_frame_resolve(handle.raw(), token, &mut info as *mut _) != 0 {
                return None;
            }

            let format = surface_pixel_format(info.format)?;
            Some(SurfaceFrame {
                handle: SurfaceHandle(info.handle),
                token: info.token,
                pixels: info.pixels as *const u8,
                bytes: info.bytes,
                width: c_int_to_u16(info.width)?,
                height: c_int_to_u16(info.height)?,
                stride_bytes: c_int_to_u16(info.stride)?,
                format,
                buffers: c_int_to_u8(info.buffers)?,
            })
        }
    }

    fn task_capabilities(&self) -> PlatformTaskCapabilities {
        let mut capabilities = PlatformTaskCapabilities::NUTTX_TASK_RUNNER;
        capabilities.wing_managed_surface =
            self.surface_dirty_queue >= 0 && self.surface_input_queue >= 0;
        capabilities
    }

    fn surface_capabilities(&self) -> SurfaceCapabilities {
        if self.surface_dirty_queue >= 0 && self.surface_input_queue >= 0 {
            SurfaceCapabilities::SOFTWARE_SHARED_RGB565
        } else {
            SurfaceCapabilities::NONE
        }
    }

    fn launch_task(
        &mut self,
        task: PlatformTask,
        surface: Option<SurfaceDescriptor>,
        settings: Option<SettingsSnapshot>,
    ) -> Result<i32, TaskLaunchError> {
        self.task_capabilities().validate(task)?;

        let command = task.command;
        if command.args.len() > COMMAND_ARG_CAPACITY {
            return Err(TaskLaunchError::TooManyArguments);
        }
        if !valid_cstr(command.program.cstr) {
            return Err(TaskLaunchError::InvalidProgram);
        }

        unsafe {
            let mut argv = [null_mut(); COMMAND_ARG_CAPACITY + 2];
            argv[0] = command.program.cstr.as_ptr() as *mut u8;

            for index in 0..command.args.len() {
                let arg = command.args[index];
                if !valid_cstr(arg.cstr) {
                    return Err(TaskLaunchError::InvalidArgument);
                }
                argv[index + 1] = arg.cstr.as_ptr() as *mut u8;
            }

            let settings = settings.unwrap_or_default();
            let settings_valid = if surface.is_some() { 1 } else { 0 };
            let config = WingTaskLaunchConfig {
                program: command.program.cstr.as_ptr(),
                argv: argv.as_ptr(),
                stdio: task_stdio(task.stdio),
                priority: task_priority(task.priority)?,
                stack_size: task_stack_size(task.stack_size)?,
                surface: task_surface_kind(task.surface, surface)?,
                surface_id: surface.map(|descriptor| descriptor.id.0 as c_int).unwrap_or(0),
                surface_width: surface
                    .map(|descriptor| descriptor.request.size.w as c_int)
                    .unwrap_or(0),
                surface_height: surface
                    .map(|descriptor| descriptor.request.size.h as c_int)
                    .unwrap_or(0),
                surface_stride: surface
                    .map(|descriptor| descriptor.stride_bytes as c_int)
                    .unwrap_or(0),
                surface_format: surface
                    .map(|descriptor| surface_format(descriptor.request.format))
                    .unwrap_or(0),
                surface_buffers: surface
                    .map(|descriptor| descriptor.buffer_count as c_int)
                    .unwrap_or(0),
                surface_transport: surface
                    .map(|descriptor| surface_transport(descriptor.request.transport))
                    .unwrap_or(0),
                surface_input: surface
                    .map(|descriptor| surface_input(descriptor.request.input))
                    .unwrap_or(0),
                surface_token: surface
                    .map(|descriptor| descriptor.transport.token as c_int)
                    .unwrap_or(0),
                surface_frame: surface
                    .map(|descriptor| descriptor.transport.frame.raw() as c_int)
                    .unwrap_or(0),
                surface_dirty: surface
                    .map(|descriptor| descriptor.transport.dirty.raw() as c_int)
                    .unwrap_or(0),
                surface_input_handle: surface
                    .map(|descriptor| descriptor.transport.input.raw() as c_int)
                    .unwrap_or(0),
                settings_valid,
                settings_theme: settings.theme as c_int,
                settings_preview: settings.preview_effect as c_int,
                settings_brightness: settings.brightness as c_int,
                settings_haptic: bool_int(settings.haptic_enabled),
                settings_reduce_motion: bool_int(settings.reduce_motion),
            };

            let pid = wing_task_spawn(&config as *const WingTaskLaunchConfig);
            if pid < 0 {
                Err(TaskLaunchError::SpawnFailed(pid))
            } else {
                Ok(pid)
            }
        }
    }

    fn close_task(&mut self, pid: i32, surface: Option<SurfaceDescriptor>) {
        let frame = surface
            .map(|descriptor| descriptor.transport.frame.raw())
            .unwrap_or(0);
        let token = surface
            .map(|descriptor| descriptor.transport.token)
            .unwrap_or(0);

        unsafe {
            let _ = wing_task_close(pid as c_int, frame, token);
        }
    }

    fn is_running(&self) -> bool {
        true
    }

    fn sleep_ms(&mut self, ms: u32) {
        unsafe {
            usleep(ms.saturating_mul(1000));
        }
    }
}

pub fn load_runtime_default_font_store() -> FontStore {
    let mut fonts = FontStore::default();
    if let Some(bytes) = unsafe {
        read_file_to_vec(
            RuntimeResourceId::DefaultFont.path().as_ptr(),
            WING_RUNTIME_FONT_CAPACITY,
        )
    } {
        let data = Box::leak(bytes.into_boxed_slice());
        let _ = fonts.register_default_ttf(data);
    }
    fonts
}

pub fn load_runtime_shell_svg_store() -> SvgStore {
    let mut svgs = SvgStore::default();
    register_bootstrap_svg_icons(&mut svgs);
    for resource in RUNTIME_SVG_ICON_RESOURCES {
        let Some(bytes) = (unsafe {
            read_file_to_vec(resource.path.as_ptr(), WING_RUNTIME_SVG_ICON_CAPACITY)
        }) else {
            continue;
        };
        let _ = svgs.register_vector_icon(resource.icon, &bytes);
    }
    svgs
}

pub fn load_runtime_default_app_svg_store() -> SvgStore {
    let mut svgs = SvgStore::default();
    register_default_app_svg_icons(&mut svgs);
    svgs
}

pub fn load_runtime_settings_app_svg_store() -> SvgStore {
    let mut svgs = SvgStore::default();
    register_settings_app_svg_icons(&mut svgs);
    svgs
}

pub fn load_runtime_system_app_svg_store() -> SvgStore {
    let mut svgs = SvgStore::default();
    register_system_app_svg_icons(&mut svgs);
    svgs
}

pub fn load_runtime_terminal_app_svg_store() -> SvgStore {
    let mut svgs = SvgStore::default();
    register_terminal_app_svg_icons(&mut svgs);
    svgs
}

pub fn load_runtime_surface_demo_app_svg_store() -> SvgStore {
    let mut svgs = SvgStore::default();
    register_surface_demo_app_svg_icons(&mut svgs);
    svgs
}

fn bool_int(value: bool) -> c_int {
    if value {
        1
    } else {
        0
    }
}

fn nuttx_resource_partition() -> ResourcePartition {
    unsafe {
        match WING_RESOURCE_STATE {
            WING_RESOURCE_STATE_FILE => file_resource_partition(),
            WING_RESOURCE_STATE_BUILTIN => ResourcePartition::builtin(),
            _ => {
                if load_file_resource_partition() {
                    WING_RESOURCE_STATE = WING_RESOURCE_STATE_FILE;
                    file_resource_partition()
                } else {
                    WING_RESOURCE_STATE = WING_RESOURCE_STATE_BUILTIN;
                    ResourcePartition::builtin()
                }
            }
        }
    }
}

fn nuttx_runtime_resource_info(id: RuntimeResourceId) -> RuntimeResourceInfo {
    unsafe {
        let buffer = ::core::ptr::addr_of_mut!(WING_RESOURCE_PROBE_BUFFER) as *mut u8;
        let Some(len) = read_file_prefix(id.path().as_ptr(), buffer, WING_RESOURCE_PROBE_CAPACITY)
        else {
            return RuntimeResourceInfo::missing(id);
        };
        let bytes = ::core::slice::from_raw_parts(buffer as *const u8, len);
        inspect_resource_header(id, bytes)
    }
}

fn load_file_settings_snapshot(path: *const u8) -> Option<SettingsSnapshot> {
    unsafe {
        let mut bytes = [0u8; SETTINGS_BLOCK_SIZE];
        let len = read_file_into_buffer(path, bytes.as_mut_ptr(), SETTINGS_BLOCK_SIZE)?;
        if len != SETTINGS_BLOCK_SIZE {
            return None;
        }

        SettingsBlock::from_bytes(bytes).decode().ok()
    }
}

fn save_file_settings_snapshot(path: *const u8, snapshot: SettingsSnapshot) -> bool {
    let block = SettingsBlock::from_snapshot(snapshot);
    unsafe { write_file_exact(path, block.bytes()) }
}

unsafe fn load_file_resource_partition() -> bool {
    let manifest_ptr = ::core::ptr::addr_of_mut!(WING_RESOURCE_MANIFEST_BUFFER) as *mut u8;
    let payload_ptr = ::core::ptr::addr_of_mut!(WING_RESOURCE_PAYLOAD_BUFFER) as *mut u8;

    let Some(manifest_len) = read_file_into_buffer(
        WING_RESOURCE_MANIFEST_PATH.as_ptr(),
        manifest_ptr,
        WING_RESOURCE_MANIFEST_CAPACITY,
    ) else {
        return false;
    };
    let Some(payload_len) = read_file_into_buffer(
        WING_RESOURCE_PAYLOAD_PATH.as_ptr(),
        payload_ptr,
        WING_RESOURCE_PAYLOAD_CAPACITY,
    ) else {
        return false;
    };

    WING_RESOURCE_MANIFEST_LEN = manifest_len;
    WING_RESOURCE_PAYLOAD_LEN = payload_len;
    true
}

unsafe fn read_file_prefix(path: *const u8, buffer: *mut u8, capacity: usize) -> Option<usize> {
    let fd = open(path, O_RDONLY);
    if fd < 0 {
        return None;
    }

    let n = read(fd, buffer, capacity);
    close(fd);
    if n <= 0 {
        None
    } else {
        Some(n as usize)
    }
}

unsafe fn read_file_to_vec(path: *const u8, capacity: usize) -> Option<Vec<u8>> {
    let fd = open(path, O_RDONLY);
    if fd < 0 {
        return None;
    }

    let mut out = Vec::new();
    let mut chunk = [0u8; WING_RUNTIME_FILE_CHUNK];
    loop {
        let n = read(fd, chunk.as_mut_ptr(), chunk.len());
        if n < 0 {
            close(fd);
            return None;
        }
        if n == 0 {
            break;
        }
        let Some(next_len) = out.len().checked_add(n as usize) else {
            close(fd);
            return None;
        };
        if next_len > capacity {
            close(fd);
            return None;
        }
        out.extend_from_slice(&chunk[..n as usize]);
    }

    close(fd);
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

unsafe fn read_file_into_buffer(path: *const u8, buffer: *mut u8, capacity: usize) -> Option<usize> {
    let fd = open(path, O_RDONLY);
    if fd < 0 {
        return None;
    }

    let mut len = 0usize;
    loop {
        if len >= capacity {
            let mut extra = 0u8;
            let n = read(fd, &mut extra as *mut u8, 1);
            close(fd);
            return if n == 0 { Some(len) } else { None };
        }

        let remaining = capacity - len;
        let n = read(fd, buffer.add(len), remaining);
        if n < 0 {
            close(fd);
            return None;
        }
        if n == 0 {
            break;
        }
        len = len.saturating_add(n as usize);
    }

    close(fd);
    if len == 0 {
        None
    } else {
        Some(len)
    }
}

unsafe fn write_file_exact(path: *const u8, bytes: &[u8]) -> bool {
    let fd = open(path, O_WRONLY | O_CREAT | O_TRUNC, WING_FILE_MODE);
    if fd < 0 {
        return false;
    }

    let mut len = 0usize;
    while len < bytes.len() {
        let n = write(fd, bytes.as_ptr().add(len), bytes.len() - len);
        if n <= 0 {
            close(fd);
            return false;
        }
        len = len.saturating_add(n as usize);
    }

    let _ = fsync(fd);
    close(fd) == 0
}

unsafe fn file_resource_partition() -> ResourcePartition {
    let manifest = ::core::slice::from_raw_parts(
        ::core::ptr::addr_of!(WING_RESOURCE_MANIFEST_BUFFER) as *const u8,
        WING_RESOURCE_MANIFEST_LEN,
    );
    let payload = ::core::slice::from_raw_parts(
        ::core::ptr::addr_of!(WING_RESOURCE_PAYLOAD_BUFFER) as *const u8,
        WING_RESOURCE_PAYLOAD_LEN,
    );
    ResourcePartition::new(manifest, payload)
}

impl Drop for NuttXPlatform {
    fn drop(&mut self) {
        unsafe {
            if self.input_fd >= 0 {
                close(self.input_fd);
            }
            if self.kbd_fd >= 0 {
                close(self.kbd_fd);
            }
            if self.surface_dirty_queue >= 0 {
                wing_surface_dirty_close(self.surface_dirty_queue);
            }
            if self.surface_input_queue >= 0 {
                wing_surface_input_close(self.surface_input_queue);
            }
            if self.settings_queue >= 0 {
                wing_settings_close(self.settings_queue);
            }
            if self.fb_fd >= 0 {
                close(self.fb_fd);
            }
        }
    }
}

fn task_stdio(stdio: TaskStdio) -> c_int {
    match stdio {
        TaskStdio::Inherit => WING_TASK_STDIO_INHERIT,
        TaskStdio::Null => WING_TASK_STDIO_NULL,
    }
}

fn task_priority(priority: TaskPriority) -> Result<c_int, TaskLaunchError> {
    match priority {
        TaskPriority::BuiltinDefault => Ok(WING_TASK_DEFAULT_PRIORITY),
        TaskPriority::Value(value) if value >= 0 => Ok(value as c_int),
        TaskPriority::Value(_) => Err(TaskLaunchError::UnsupportedPriority),
    }
}

fn task_stack_size(stack_size: TaskStackSize) -> Result<c_int, TaskLaunchError> {
    match stack_size {
        TaskStackSize::BuiltinDefault => Ok(WING_TASK_DEFAULT_STACK),
        TaskStackSize::Bytes(bytes) if bytes <= i32::MAX as u32 => Ok(bytes as c_int),
        TaskStackSize::Bytes(_) => Err(TaskLaunchError::UnsupportedStackSize),
    }
}

fn task_surface_kind(
    task_surface: TaskSurface,
    descriptor: Option<SurfaceDescriptor>,
) -> Result<c_int, TaskLaunchError> {
    match (task_surface, descriptor) {
        (TaskSurface::Detached, None) => Ok(WING_TASK_DETACHED_SURFACE),
        (TaskSurface::WingManaged(_), Some(_)) => Ok(WING_TASK_MANAGED_SURFACE),
        (TaskSurface::WingManaged(_), None) => Err(TaskLaunchError::UnsupportedSurface),
        (TaskSurface::Detached, Some(_)) => Err(TaskLaunchError::InvalidSurface),
    }
}

fn surface_format(format: SurfacePixelFormat) -> c_int {
    match format {
        SurfacePixelFormat::Rgb565 => WING_SURFACE_FORMAT_RGB565,
        SurfacePixelFormat::Argb8888 => WING_SURFACE_FORMAT_ARGB8888,
    }
}

fn surface_transport(transport: SurfaceTransport) -> c_int {
    match transport {
        SurfaceTransport::SharedMemory => WING_SURFACE_TRANSPORT_SHARED_MEMORY,
        SurfaceTransport::StreamTexture => WING_SURFACE_TRANSPORT_STREAM_TEXTURE,
    }
}

fn surface_input(input: SurfaceInputPolicy) -> c_int {
    match input {
        SurfaceInputPolicy::None => WING_SURFACE_INPUT_NONE,
        SurfaceInputPolicy::Pointer => WING_SURFACE_INPUT_POINTER,
        SurfaceInputPolicy::PointerKeyboard => WING_SURFACE_INPUT_POINTER_KEYBOARD,
    }
}

fn surface_pixel_format(format: c_int) -> Option<SurfacePixelFormat> {
    match format {
        WING_SURFACE_FORMAT_RGB565 => Some(SurfacePixelFormat::Rgb565),
        WING_SURFACE_FORMAT_ARGB8888 => Some(SurfacePixelFormat::Argb8888),
        _ => None,
    }
}

fn c_int_to_u16(value: c_int) -> Option<u16> {
    if value >= 0 && value <= u16::MAX as c_int {
        Some(value as u16)
    } else {
        None
    }
}

fn c_int_to_u8(value: c_int) -> Option<u8> {
    if value >= 0 && value <= u8::MAX as c_int {
        Some(value as u8)
    } else {
        None
    }
}

fn valid_cstr(bytes: &'static [u8]) -> bool {
    if bytes.is_empty() || bytes[bytes.len() - 1] != 0 {
        return false;
    }

    for byte in &bytes[..bytes.len() - 1] {
        if *byte == 0 {
            return false;
        }
    }

    true
}

fn map_keycode(code: u32) -> Option<KeyCode> {
    match code {
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
        X11_KEY_SPACE => Some(KeyCode::Space),
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
