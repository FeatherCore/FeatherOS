use ::core::ffi::c_char;
use ::core::ptr;
use ::core::slice;

use crate::math::Point;
pub use crate::settings::SettingsKey as WingSettingsKey;
pub use crate::settings::SettingsSnapshot as WingSettingsSnapshot;
pub use crate::surface::SurfaceFrameInfoAbi as WingSurfaceFrameInfo;

pub const WING_SURFACE_ABI_VERSION: u32 = 1;

pub const WING_SURFACE_FORMAT_RGB565: i32 = 1;
pub const WING_SURFACE_FORMAT_ARGB8888: i32 = 2;

pub const WING_SURFACE_TRANSPORT_SHARED_MEMORY: i32 = 1;
pub const WING_SURFACE_TRANSPORT_STREAM_TEXTURE: i32 = 2;

pub const WING_SURFACE_INPUT_NONE: i32 = 0;
pub const WING_SURFACE_INPUT_POINTER: i32 = 1;
pub const WING_SURFACE_INPUT_POINTER_KEYBOARD: i32 = 2;

pub const WING_SURFACE_POINTER_MOVE: u8 = 1;
pub const WING_SURFACE_POINTER_DOWN: u8 = 2;
pub const WING_SURFACE_POINTER_UP: u8 = 3;
pub const WING_SURFACE_POINTER_CANCEL: u8 = 4;

pub const WING_SURFACE_CAP_FORMAT_RGB565: u32 = 1 << 0;
pub const WING_SURFACE_CAP_FORMAT_ARGB8888: u32 = 1 << 1;

pub const WING_SURFACE_CAP_TRANSPORT_SHARED_MEMORY: u32 = 1 << 0;
pub const WING_SURFACE_CAP_TRANSPORT_STREAM_TEXTURE: u32 = 1 << 1;

pub const WING_SURFACE_CAP_INPUT_NONE: u32 = 1 << 0;
pub const WING_SURFACE_CAP_INPUT_POINTER: u32 = 1 << 1;
pub const WING_SURFACE_CAP_INPUT_POINTER_KEYBOARD: u32 = 1 << 2;

const SURFACE_ARG_PREFIX: &[u8] = b"--wing-surface=";
const SETTINGS_ARG_PREFIX: &[u8] = b"--wing-settings=";
const SURFACE_FIELD_COUNT: usize = 12;
const SETTINGS_FIELD_COUNT: usize = 5;
const INPUT_DRAIN_LIMIT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WingSurfaceFormat {
    Rgb565,
    Argb8888,
}

impl WingSurfaceFormat {
    pub const fn raw(self) -> i32 {
        match self {
            Self::Rgb565 => WING_SURFACE_FORMAT_RGB565,
            Self::Argb8888 => WING_SURFACE_FORMAT_ARGB8888,
        }
    }

    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgb565 => 2,
            Self::Argb8888 => 4,
        }
    }

    const fn from_raw(raw: u32) -> Option<Self> {
        match raw as i32 {
            WING_SURFACE_FORMAT_RGB565 => Some(Self::Rgb565),
            WING_SURFACE_FORMAT_ARGB8888 => Some(Self::Argb8888),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WingSurfaceTransport {
    SharedMemory,
    StreamTexture,
}

impl WingSurfaceTransport {
    pub const fn raw(self) -> i32 {
        match self {
            Self::SharedMemory => WING_SURFACE_TRANSPORT_SHARED_MEMORY,
            Self::StreamTexture => WING_SURFACE_TRANSPORT_STREAM_TEXTURE,
        }
    }

    const fn from_raw(raw: u32) -> Option<Self> {
        match raw as i32 {
            WING_SURFACE_TRANSPORT_SHARED_MEMORY => Some(Self::SharedMemory),
            WING_SURFACE_TRANSPORT_STREAM_TEXTURE => Some(Self::StreamTexture),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WingSurfaceInput {
    None,
    Pointer,
    PointerKeyboard,
}

impl WingSurfaceInput {
    pub const fn raw(self) -> i32 {
        match self {
            Self::None => WING_SURFACE_INPUT_NONE,
            Self::Pointer => WING_SURFACE_INPUT_POINTER,
            Self::PointerKeyboard => WING_SURFACE_INPUT_POINTER_KEYBOARD,
        }
    }

    const fn from_raw(raw: u32) -> Option<Self> {
        match raw as i32 {
            WING_SURFACE_INPUT_NONE => Some(Self::None),
            WING_SURFACE_INPUT_POINTER => Some(Self::Pointer),
            WING_SURFACE_INPUT_POINTER_KEYBOARD => Some(Self::PointerKeyboard),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WingPointerKind {
    Move,
    Down,
    Up,
    Cancel,
}

impl WingPointerKind {
    const fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            WING_SURFACE_POINTER_MOVE => Some(Self::Move),
            WING_SURFACE_POINTER_DOWN => Some(Self::Down),
            WING_SURFACE_POINTER_UP => Some(Self::Up),
            WING_SURFACE_POINTER_CANCEL => Some(Self::Cancel),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WingPointerEvent {
    pub point: Point,
    pub kind: WingPointerKind,
    pub buttons: u8,
}

impl WingPointerEvent {
    pub const fn primary_down(self) -> bool {
        (self.buttons & 1) != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WingSurfaceDescriptor {
    pub id: u32,
    pub width: u16,
    pub height: u16,
    pub stride_bytes: u16,
    pub format: WingSurfaceFormat,
    pub buffers: u8,
    pub transport: WingSurfaceTransport,
    pub input: WingSurfaceInput,
    pub token: u32,
    pub frame_handle: u32,
    pub dirty_handle: u32,
    pub input_handle: u32,
}

impl WingSurfaceDescriptor {
    pub const fn frame_bytes(self) -> usize {
        self.stride_bytes as usize * self.height as usize
    }

    pub const fn allocation_bytes(self) -> usize {
        self.frame_bytes() * self.buffers as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WingDirtyRect {
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
}

impl WingDirtyRect {
    pub const fn new(x: i16, y: i16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WingSurfaceCapabilities {
    pub abi_version: u32,
    pub format_mask: u32,
    pub transport_mask: u32,
    pub input_mask: u32,
    pub max_surfaces: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub max_frame_bytes: u32,
}

impl WingSurfaceCapabilities {
    pub fn query() -> Option<Self> {
        let mut capabilities = Self::default();
        let result = unsafe { wing_surface_query_capabilities(&mut capabilities as *mut Self) };
        if result == 0 {
            Some(capabilities)
        } else {
            None
        }
    }

    pub const fn supports_format(self, format: WingSurfaceFormat) -> bool {
        (self.format_mask & surface_format_capability(format)) != 0
    }

    pub const fn supports_transport(self, transport: WingSurfaceTransport) -> bool {
        (self.transport_mask & surface_transport_capability(transport)) != 0
    }

    pub const fn supports_input(self, input: WingSurfaceInput) -> bool {
        (self.input_mask & surface_input_capability(input)) != 0
    }

    pub const fn supports_current_abi(self) -> bool {
        self.abi_version == WING_SURFACE_ABI_VERSION
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WingSurfaceError {
    MissingArgument,
    BadArgument,
    NumberOverflow,
    UnsupportedFormat,
    UnsupportedTransport,
    UnsupportedInput,
    InvalidGeometry,
    CapabilityQueryFailed,
    UnsupportedAbi,
    SurfaceExceedsCapabilities,
    ResolveFailed,
    DirtyQueueOpenFailed,
    InputQueueOpenFailed,
    DirtySubmitFailed(i32),
    InputReceiveFailed(i32),
    DirtyQueueCloseFailed(i32),
    InputQueueCloseFailed(i32),
}

impl WingSurfaceError {
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::MissingArgument => 2,
            Self::BadArgument => 3,
            Self::NumberOverflow => 4,
            Self::UnsupportedFormat => 5,
            Self::UnsupportedTransport => 6,
            Self::UnsupportedInput => 7,
            Self::InvalidGeometry => 8,
            Self::ResolveFailed => 9,
            Self::DirtyQueueOpenFailed => 10,
            Self::InputQueueOpenFailed => 11,
            Self::DirtySubmitFailed(_) => 12,
            Self::InputReceiveFailed(_) => 13,
            Self::DirtyQueueCloseFailed(_) => 14,
            Self::InputQueueCloseFailed(_) => 15,
            Self::CapabilityQueryFailed => 16,
            Self::UnsupportedAbi => 17,
            Self::SurfaceExceedsCapabilities => 18,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WingSettingsError {
    OpenFailed(i32),
    SendFailed(i32),
    CloseFailed(i32),
}

pub struct WingSettingsClient {
    queue: i32,
}

impl WingSettingsClient {
    pub fn open() -> Result<Self, WingSettingsError> {
        let queue = unsafe { wing_settings_sender_open() };
        if queue < 0 {
            Err(WingSettingsError::OpenFailed(queue))
        } else {
            Ok(Self { queue })
        }
    }

    pub fn send(&self, key: WingSettingsKey, value: u32) -> Result<(), WingSettingsError> {
        let result = unsafe { wing_settings_send(self.queue, key.raw(), value) };
        if result < 0 {
            Err(WingSettingsError::SendFailed(result))
        } else {
            Ok(())
        }
    }

    pub fn close(mut self) -> Result<(), WingSettingsError> {
        if self.queue < 0 {
            return Ok(());
        }

        let queue = self.queue;
        self.queue = -1;
        let result = unsafe { wing_settings_close(queue) };
        if result < 0 {
            Err(WingSettingsError::CloseFailed(result))
        } else {
            Ok(())
        }
    }
}

impl Drop for WingSettingsClient {
    fn drop(&mut self) {
        if self.queue >= 0 {
            unsafe {
                let _ = wing_settings_close(self.queue);
            }
            self.queue = -1;
        }
    }
}

pub struct WingSurface {
    descriptor: WingSurfaceDescriptor,
    frame: WingSurfaceFrameInfo,
    dirty_queue: i32,
    input_queue: i32,
}

impl WingSurface {
    pub unsafe fn open_from_argv(
        argc: i32,
        argv: *const *const c_char,
    ) -> Result<Self, WingSurfaceError> {
        let descriptor = parse_surface_descriptor(argc, argv)?;
        Self::open(descriptor)
    }

    pub unsafe fn open_from_argv_mut(
        argc: i32,
        argv: *mut *mut c_char,
    ) -> Result<Self, WingSurfaceError> {
        Self::open_from_argv(argc, argv as *const *const c_char)
    }

    pub unsafe fn open(descriptor: WingSurfaceDescriptor) -> Result<Self, WingSurfaceError> {
        validate_descriptor(descriptor)?;
        validate_capabilities(descriptor)?;

        let mut frame = WingSurfaceFrameInfo {
            handle: 0,
            token: 0,
            pixels: ptr::null_mut(),
            bytes: 0,
            width: 0,
            height: 0,
            stride: 0,
            format: 0,
            buffers: 0,
        };

        if wing_app_surface_frame_resolve(descriptor.frame_handle, descriptor.token, &mut frame)
            < 0
        {
            return Err(WingSurfaceError::ResolveFailed);
        }

        validate_frame(descriptor, frame)?;

        let dirty_queue = wing_surface_dirty_sender_open();
        if dirty_queue < 0 {
            return Err(WingSurfaceError::DirtyQueueOpenFailed);
        }

        let input_queue = if descriptor.input == WingSurfaceInput::None {
            -1
        } else {
            let queue = wing_surface_input_receiver_open();
            if queue < 0 {
                let _ = wing_surface_dirty_close(dirty_queue);
                return Err(WingSurfaceError::InputQueueOpenFailed);
            }
            queue
        };

        Ok(Self {
            descriptor,
            frame,
            dirty_queue,
            input_queue,
        })
    }

    pub const fn descriptor(&self) -> WingSurfaceDescriptor {
        self.descriptor
    }

    pub const fn width(&self) -> u16 {
        self.descriptor.width
    }

    pub const fn height(&self) -> u16 {
        self.descriptor.height
    }

    pub const fn stride_bytes(&self) -> u16 {
        self.descriptor.stride_bytes
    }

    pub const fn format(&self) -> WingSurfaceFormat {
        self.descriptor.format
    }

    pub const fn frame_info(&self) -> WingSurfaceFrameInfo {
        self.frame
    }

    pub unsafe fn pixels_mut(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.frame.pixels as *mut u8, self.frame.bytes)
    }

    pub unsafe fn pixels_rgb565_mut(&mut self) -> Result<&mut [u16], WingSurfaceError> {
        if self.descriptor.format != WingSurfaceFormat::Rgb565 {
            return Err(WingSurfaceError::UnsupportedFormat);
        }

        let stride_pixels = self.descriptor.stride_bytes as usize / 2;
        let len = stride_pixels * self.descriptor.height as usize;
        Ok(slice::from_raw_parts_mut(
            self.frame.pixels as *mut u16,
            len,
        ))
    }

    pub fn submit_dirty(&self, rect: WingDirtyRect) -> Result<(), WingSurfaceError> {
        let result = unsafe {
            wing_surface_dirty_send(
                self.dirty_queue,
                self.descriptor.dirty_handle,
                self.descriptor.token,
                rect.x,
                rect.y,
                rect.w,
                rect.h,
            )
        };
        if result < 0 {
            Err(WingSurfaceError::DirtySubmitFailed(result))
        } else {
            Ok(())
        }
    }

    pub fn submit_full(&self) -> Result<(), WingSurfaceError> {
        self.submit_dirty(WingDirtyRect::new(
            0,
            0,
            self.descriptor.width,
            self.descriptor.height,
        ))
    }

    pub fn submit_close(&self) -> Result<(), WingSurfaceError> {
        self.submit_dirty(WingDirtyRect::new(0, 0, 0, 0))
    }

    pub fn poll_input(&self) -> Result<Option<WingPointerEvent>, WingSurfaceError> {
        if self.input_queue < 0 {
            return Ok(None);
        }

        for _ in 0..INPUT_DRAIN_LIMIT {
            let mut message = WingSurfaceInputMessage {
                handle: 0,
                token: 0,
                x: 0,
                y: 0,
                event: 0,
                buttons: 0,
                reserved: 0,
            };

            let result = unsafe {
                wing_surface_input_receive(
                    self.input_queue,
                    &mut message as *mut WingSurfaceInputMessage,
                )
            };

            if result < 0 {
                return Err(WingSurfaceError::InputReceiveFailed(result));
            }

            if result == 0 {
                return Ok(None);
            }

            if message.handle != self.descriptor.input_handle
                || message.token != self.descriptor.token
            {
                continue;
            }

            let Some(kind) = WingPointerKind::from_raw(message.event) else {
                return Err(WingSurfaceError::BadArgument);
            };

            return Ok(Some(WingPointerEvent {
                point: Point::new(message.x as i32, message.y as i32),
                kind,
                buttons: message.buttons,
            }));
        }

        Ok(None)
    }

    pub fn close(mut self) -> Result<(), WingSurfaceError> {
        let mut error = None;

        let _ = self.submit_close();

        if self.dirty_queue >= 0 {
            let queue = self.dirty_queue;
            self.dirty_queue = -1;
            let result = unsafe { wing_surface_dirty_close(queue) };
            if result < 0 {
                error = Some(WingSurfaceError::DirtyQueueCloseFailed(result));
            }
        }

        if self.input_queue >= 0 {
            let queue = self.input_queue;
            self.input_queue = -1;
            let result = unsafe { wing_surface_input_close(queue) };
            if result < 0 && error.is_none() {
                error = Some(WingSurfaceError::InputQueueCloseFailed(result));
            }
        }

        match error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl Drop for WingSurface {
    fn drop(&mut self) {
        if self.dirty_queue >= 0 {
            let _ = self.submit_close();
        }
        if self.dirty_queue >= 0 {
            unsafe {
                let _ = wing_surface_dirty_close(self.dirty_queue);
            }
            self.dirty_queue = -1;
        }
        if self.input_queue >= 0 {
            unsafe {
                let _ = wing_surface_input_close(self.input_queue);
            }
            self.input_queue = -1;
        }
    }
}

pub unsafe fn parse_surface_descriptor(
    argc: i32,
    argv: *const *const c_char,
) -> Result<WingSurfaceDescriptor, WingSurfaceError> {
    if argc <= 0 || argv.is_null() {
        return Err(WingSurfaceError::MissingArgument);
    }

    let mut index = 1;
    while index < argc {
        let arg = *argv.add(index as usize);
        if !arg.is_null() && has_surface_prefix(arg) {
            return parse_surface_arg(arg);
        }
        index += 1;
    }

    Err(WingSurfaceError::MissingArgument)
}

pub unsafe fn parse_settings_snapshot(
    argc: i32,
    argv: *const *const c_char,
) -> Result<Option<WingSettingsSnapshot>, WingSurfaceError> {
    if argc <= 0 || argv.is_null() {
        return Ok(None);
    }

    let mut index = 1;
    while index < argc {
        let arg = *argv.add(index as usize);
        if !arg.is_null() && has_settings_prefix(arg) {
            return parse_settings_arg(arg).map(Some);
        }
        index += 1;
    }

    Ok(None)
}

unsafe fn parse_surface_arg(
    arg: *const c_char,
) -> Result<WingSurfaceDescriptor, WingSurfaceError> {
    let mut offset = SURFACE_ARG_PREFIX.len();
    let mut fields = [0u32; SURFACE_FIELD_COUNT];
    let mut index = 0usize;

    while index < SURFACE_FIELD_COUNT {
        fields[index] = parse_u32_field(arg, &mut offset)?;
        let separator = byte_at(arg, offset);
        if index + 1 == SURFACE_FIELD_COUNT {
            if separator != 0 {
                return Err(WingSurfaceError::BadArgument);
            }
        } else if separator == b':' {
            offset += 1;
        } else {
            return Err(WingSurfaceError::BadArgument);
        }
        index += 1;
    }

    let format =
        WingSurfaceFormat::from_raw(fields[4]).ok_or(WingSurfaceError::UnsupportedFormat)?;
    let transport = WingSurfaceTransport::from_raw(fields[6])
        .ok_or(WingSurfaceError::UnsupportedTransport)?;
    let input = WingSurfaceInput::from_raw(fields[7]).ok_or(WingSurfaceError::UnsupportedInput)?;

    let descriptor = WingSurfaceDescriptor {
        id: fields[0],
        width: checked_u16(fields[1])?,
        height: checked_u16(fields[2])?,
        stride_bytes: checked_u16(fields[3])?,
        format,
        buffers: checked_u8(fields[5])?,
        transport,
        input,
        token: fields[8],
        frame_handle: fields[9],
        dirty_handle: fields[10],
        input_handle: fields[11],
    };

    validate_descriptor(descriptor)?;
    Ok(descriptor)
}

unsafe fn parse_settings_arg(
    arg: *const c_char,
) -> Result<WingSettingsSnapshot, WingSurfaceError> {
    let mut offset = SETTINGS_ARG_PREFIX.len();
    let mut fields = [0u32; SETTINGS_FIELD_COUNT];
    let mut index = 0usize;

    while index < SETTINGS_FIELD_COUNT {
        fields[index] = parse_u32_field(arg, &mut offset)?;
        let separator = byte_at(arg, offset);
        if index + 1 == SETTINGS_FIELD_COUNT {
            if separator != 0 {
                return Err(WingSurfaceError::BadArgument);
            }
        } else if separator == b':' {
            offset += 1;
        } else {
            return Err(WingSurfaceError::BadArgument);
        }
        index += 1;
    }

    Ok(WingSettingsSnapshot::new(
        checked_u8(fields[0])?,
        checked_u8(fields[1])?,
        checked_u8(fields[2])?,
        fields[3] != 0,
        fields[4] != 0,
    )
    .sanitized())
}

unsafe fn has_surface_prefix(arg: *const c_char) -> bool {
    let mut index = 0usize;
    while index < SURFACE_ARG_PREFIX.len() {
        if byte_at(arg, index) != SURFACE_ARG_PREFIX[index] {
            return false;
        }
        index += 1;
    }
    true
}

unsafe fn has_settings_prefix(arg: *const c_char) -> bool {
    let mut index = 0usize;
    while index < SETTINGS_ARG_PREFIX.len() {
        if byte_at(arg, index) != SETTINGS_ARG_PREFIX[index] {
            return false;
        }
        index += 1;
    }
    true
}

unsafe fn parse_u32_field(
    arg: *const c_char,
    offset: &mut usize,
) -> Result<u32, WingSurfaceError> {
    let mut value = 0u32;
    let mut seen = false;

    loop {
        let byte = byte_at(arg, *offset);
        match byte {
            b'0'..=b'9' => {
                seen = true;
                value = value
                    .checked_mul(10)
                    .and_then(|value| value.checked_add((byte - b'0') as u32))
                    .ok_or(WingSurfaceError::NumberOverflow)?;
                *offset += 1;
            }
            b':' | 0 => break,
            _ => return Err(WingSurfaceError::BadArgument),
        }
    }

    if seen {
        Ok(value)
    } else {
        Err(WingSurfaceError::BadArgument)
    }
}

unsafe fn byte_at(arg: *const c_char, offset: usize) -> u8 {
    *(arg.add(offset) as *const u8)
}

fn checked_u16(value: u32) -> Result<u16, WingSurfaceError> {
    if value <= u16::MAX as u32 {
        Ok(value as u16)
    } else {
        Err(WingSurfaceError::NumberOverflow)
    }
}

fn checked_u8(value: u32) -> Result<u8, WingSurfaceError> {
    if value <= u8::MAX as u32 {
        Ok(value as u8)
    } else {
        Err(WingSurfaceError::NumberOverflow)
    }
}

fn validate_descriptor(descriptor: WingSurfaceDescriptor) -> Result<(), WingSurfaceError> {
    if descriptor.width == 0
        || descriptor.height == 0
        || descriptor.buffers == 0
        || descriptor.token == 0
        || descriptor.frame_handle == 0
        || descriptor.dirty_handle == 0
        || (descriptor.input != WingSurfaceInput::None && descriptor.input_handle == 0)
    {
        return Err(WingSurfaceError::InvalidGeometry);
    }

    let min_stride = descriptor.width as usize * descriptor.format.bytes_per_pixel();
    if (descriptor.stride_bytes as usize) < min_stride {
        return Err(WingSurfaceError::InvalidGeometry);
    }

    Ok(())
}

fn validate_capabilities(descriptor: WingSurfaceDescriptor) -> Result<(), WingSurfaceError> {
    let capabilities =
        WingSurfaceCapabilities::query().ok_or(WingSurfaceError::CapabilityQueryFailed)?;

    if !capabilities.supports_current_abi() {
        return Err(WingSurfaceError::UnsupportedAbi);
    }

    if !capabilities.supports_format(descriptor.format) {
        return Err(WingSurfaceError::UnsupportedFormat);
    }

    if !capabilities.supports_transport(descriptor.transport) {
        return Err(WingSurfaceError::UnsupportedTransport);
    }

    if !capabilities.supports_input(descriptor.input) {
        return Err(WingSurfaceError::UnsupportedInput);
    }

    if descriptor.width as u32 > capabilities.max_width
        || descriptor.height as u32 > capabilities.max_height
        || descriptor.allocation_bytes() > capabilities.max_frame_bytes as usize
    {
        return Err(WingSurfaceError::SurfaceExceedsCapabilities);
    }

    Ok(())
}

fn validate_frame(
    descriptor: WingSurfaceDescriptor,
    frame: WingSurfaceFrameInfo,
) -> Result<(), WingSurfaceError> {
    if frame.pixels.is_null()
        || frame.bytes == 0
        || frame.width != descriptor.width as i32
        || frame.height != descriptor.height as i32
        || frame.stride != descriptor.stride_bytes as i32
        || frame.format != descriptor.format.raw()
        || frame.buffers != descriptor.buffers as i32
        || frame.bytes < descriptor.frame_bytes()
    {
        return Err(WingSurfaceError::InvalidGeometry);
    }

    Ok(())
}

const fn surface_format_capability(format: WingSurfaceFormat) -> u32 {
    match format {
        WingSurfaceFormat::Rgb565 => WING_SURFACE_CAP_FORMAT_RGB565,
        WingSurfaceFormat::Argb8888 => WING_SURFACE_CAP_FORMAT_ARGB8888,
    }
}

const fn surface_transport_capability(transport: WingSurfaceTransport) -> u32 {
    match transport {
        WingSurfaceTransport::SharedMemory => WING_SURFACE_CAP_TRANSPORT_SHARED_MEMORY,
        WingSurfaceTransport::StreamTexture => WING_SURFACE_CAP_TRANSPORT_STREAM_TEXTURE,
    }
}

const fn surface_input_capability(input: WingSurfaceInput) -> u32 {
    match input {
        WingSurfaceInput::None => WING_SURFACE_CAP_INPUT_NONE,
        WingSurfaceInput::Pointer => WING_SURFACE_CAP_INPUT_POINTER,
        WingSurfaceInput::PointerKeyboard => WING_SURFACE_CAP_INPUT_POINTER_KEYBOARD,
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct WingSurfaceInputMessage {
    handle: u32,
    token: u32,
    x: i16,
    y: i16,
    event: u8,
    buttons: u8,
    reserved: u16,
}

extern "C" {
    fn wing_surface_query_capabilities(caps: *mut WingSurfaceCapabilities) -> i32;
    #[link_name = "wing_surface_frame_resolve"]
    fn wing_app_surface_frame_resolve(
        handle: u32,
        token: u32,
        info: *mut WingSurfaceFrameInfo,
    ) -> i32;
    fn wing_surface_dirty_sender_open() -> i32;
    fn wing_surface_dirty_send(
        queue: i32,
        handle: u32,
        token: u32,
        x: i16,
        y: i16,
        w: u16,
        h: u16,
    ) -> i32;
    fn wing_surface_dirty_close(queue: i32) -> i32;
    fn wing_surface_input_receiver_open() -> i32;
    fn wing_surface_input_receive(queue: i32, message: *mut WingSurfaceInputMessage) -> i32;
    fn wing_surface_input_close(queue: i32) -> i32;
    fn wing_settings_sender_open() -> i32;
    fn wing_settings_send(queue: i32, key: u16, value: u32) -> i32;
    fn wing_settings_close(queue: i32) -> i32;
}
