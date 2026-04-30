use crate::app::AppId;
use crate::math::{Point, Rect, Size};

pub const SURFACE_CAPACITY: usize = 8;
const SURFACE_HANDLE_FRAME_KIND: u8 = 1;
const SURFACE_HANDLE_DIRTY_KIND: u8 = 2;
const SURFACE_HANDLE_INPUT_KIND: u8 = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceId(pub u16);

impl SurfaceId {
    pub const fn from_parts(slot: u8, generation: u8) -> Self {
        Self(((generation as u16) << 8) | slot as u16)
    }

    pub const fn slot(self) -> usize {
        (self.0 & 0xff) as usize
    }

    pub const fn generation(self) -> u8 {
        (self.0 >> 8) as u8
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceHandle(pub u32);

impl SurfaceHandle {
    pub const fn from_parts(id: SurfaceId, kind: u8) -> Self {
        Self(((kind as u32) << 24) | id.0 as u32)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn id(self) -> SurfaceId {
        SurfaceId(self.0 as u16)
    }

    pub const fn kind(self) -> Option<SurfaceHandleKind> {
        match (self.0 >> 24) as u8 {
            SURFACE_HANDLE_FRAME_KIND => Some(SurfaceHandleKind::Frame),
            SURFACE_HANDLE_DIRTY_KIND => Some(SurfaceHandleKind::Dirty),
            SURFACE_HANDLE_INPUT_KIND => Some(SurfaceHandleKind::Input),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceHandleKind {
    Frame,
    Dirty,
    Input,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfacePixelFormat {
    Rgb565,
    Argb8888,
}

impl SurfacePixelFormat {
    pub const fn bytes_per_pixel(self) -> u16 {
        match self {
            Self::Rgb565 => 2,
            Self::Argb8888 => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceBuffering {
    Single,
    Double,
}

impl SurfaceBuffering {
    pub const fn buffer_count(self) -> u8 {
        match self {
            Self::Single => 1,
            Self::Double => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceTransport {
    SharedMemory,
    StreamTexture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceInputPolicy {
    None,
    Pointer,
    PointerKeyboard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceRequest {
    pub owner: AppId,
    pub size: Size,
    pub format: SurfacePixelFormat,
    pub buffering: SurfaceBuffering,
    pub transport: SurfaceTransport,
    pub input: SurfaceInputPolicy,
}

impl SurfaceRequest {
    pub const fn new(owner: AppId, size: Size) -> Self {
        Self {
            owner,
            size,
            format: SurfacePixelFormat::Rgb565,
            buffering: SurfaceBuffering::Double,
            transport: SurfaceTransport::SharedMemory,
            input: SurfaceInputPolicy::Pointer,
        }
    }

    pub fn stride_bytes(self) -> Option<u16> {
        self.size.w.checked_mul(self.format.bytes_per_pixel())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceDescriptor {
    pub id: SurfaceId,
    pub request: SurfaceRequest,
    pub stride_bytes: u16,
    pub buffer_count: u8,
    pub transport: SurfaceTransportDescriptor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceTransportDescriptor {
    pub token: u32,
    pub frame: SurfaceHandle,
    pub dirty: SurfaceHandle,
    pub input: SurfaceHandle,
}

impl SurfaceTransportDescriptor {
    pub const fn new(id: SurfaceId, owner: AppId) -> Self {
        let token =
            ((((owner.0 as u32) << 12) ^ id.0 as u32 ^ 0x1749_4e47) & 0x7fff_ffff) | 1;
        Self {
            token,
            frame: SurfaceHandle::from_parts(id, SURFACE_HANDLE_FRAME_KIND),
            dirty: SurfaceHandle::from_parts(id, SURFACE_HANDLE_DIRTY_KIND),
            input: SurfaceHandle::from_parts(id, SURFACE_HANDLE_INPUT_KIND),
        }
    }
}

impl SurfaceDescriptor {
    pub const fn frame_bytes(self) -> u32 {
        self.stride_bytes as u32 * self.request.size.h as u32
    }

    pub const fn allocation_bytes(self) -> u32 {
        self.frame_bytes() * self.buffer_count as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceEndpoint {
    pub id: SurfaceId,
    pub kind: SurfaceHandleKind,
    pub descriptor: SurfaceDescriptor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceFrame {
    pub handle: SurfaceHandle,
    pub token: u32,
    pub pixels: *const u8,
    pub bytes: usize,
    pub width: u16,
    pub height: u16,
    pub stride_bytes: u16,
    pub format: SurfacePixelFormat,
    pub buffers: u8,
}

impl SurfaceFrame {
    pub const fn is_empty(self) -> bool {
        self.pixels.is_null() || self.bytes == 0 || self.width == 0 || self.height == 0
    }

    pub const fn frame_bytes(self) -> usize {
        self.stride_bytes as usize * self.height as usize
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SurfaceFrameInfoAbi {
    pub handle: u32,
    pub token: u32,
    pub pixels: *mut u8,
    pub bytes: usize,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format: i32,
    pub buffers: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceDirtyEvent {
    pub handle: SurfaceHandle,
    pub token: u32,
    pub rect: Rect,
}

impl SurfaceDirtyEvent {
    pub const fn new(handle: SurfaceHandle, token: u32, rect: Rect) -> Self {
        Self {
            handle,
            token,
            rect,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceClosedEvent {
    pub handle: SurfaceHandle,
    pub token: u32,
}

impl SurfaceClosedEvent {
    pub const fn new(handle: SurfaceHandle, token: u32) -> Self {
        Self { handle, token }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfacePointerKind {
    Move,
    Down,
    Up,
    Cancel,
}

impl SurfacePointerKind {
    pub const fn raw(self) -> u8 {
        match self {
            Self::Move => 1,
            Self::Down => 2,
            Self::Up => 3,
            Self::Cancel => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfacePointerEvent {
    pub handle: SurfaceHandle,
    pub token: u32,
    pub point: Point,
    pub kind: SurfacePointerKind,
    pub buttons: u8,
}

impl SurfacePointerEvent {
    pub const fn new(
        handle: SurfaceHandle,
        token: u32,
        point: Point,
        kind: SurfacePointerKind,
        buttons: u8,
    ) -> Self {
        Self {
            handle,
            token,
            point,
            kind,
            buttons,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceEvent {
    Dirty(SurfaceDirtyEvent),
    Closed(SurfaceClosedEvent),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceSlot {
    pub descriptor: SurfaceDescriptor,
    pub visible: bool,
    pub frame_serial: u32,
    pub dirty: Option<Rect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceError {
    CapacityFull,
    InvalidGeometry,
    UnsupportedFormat,
    UnsupportedBuffering,
    UnsupportedTransport,
    UnsupportedInput,
    InvalidHandle,
    InvalidToken,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceCapabilities {
    pub max_surfaces: u8,
    pub max_width: u16,
    pub max_height: u16,
    pub rgb565: bool,
    pub argb8888: bool,
    pub single_buffer: bool,
    pub double_buffer: bool,
    pub shared_memory: bool,
    pub stream_texture: bool,
    pub pointer_input: bool,
    pub keyboard_input: bool,
}

impl SurfaceCapabilities {
    pub const NONE: Self = Self {
        max_surfaces: 0,
        max_width: 0,
        max_height: 0,
        rgb565: false,
        argb8888: false,
        single_buffer: false,
        double_buffer: false,
        shared_memory: false,
        stream_texture: false,
        pointer_input: false,
        keyboard_input: false,
    };

    pub const SOFTWARE_SHARED_RGB565: Self = Self {
        max_surfaces: SURFACE_CAPACITY as u8,
        max_width: 640,
        max_height: 640,
        rgb565: true,
        argb8888: false,
        single_buffer: true,
        double_buffer: true,
        shared_memory: true,
        stream_texture: false,
        pointer_input: true,
        keyboard_input: false,
    };

    pub fn validate(self, request: SurfaceRequest) -> Result<(), SurfaceError> {
        if self.max_surfaces == 0
            || request.size.w == 0
            || request.size.h == 0
            || request.size.w > self.max_width
            || request.size.h > self.max_height
            || request.stride_bytes().is_none()
        {
            return Err(SurfaceError::InvalidGeometry);
        }

        match request.format {
            SurfacePixelFormat::Rgb565 if self.rgb565 => {}
            SurfacePixelFormat::Argb8888 if self.argb8888 => {}
            _ => return Err(SurfaceError::UnsupportedFormat),
        }

        match request.buffering {
            SurfaceBuffering::Single if self.single_buffer => {}
            SurfaceBuffering::Double if self.double_buffer => {}
            _ => return Err(SurfaceError::UnsupportedBuffering),
        }

        match request.transport {
            SurfaceTransport::SharedMemory if self.shared_memory => {}
            SurfaceTransport::StreamTexture if self.stream_texture => {}
            _ => return Err(SurfaceError::UnsupportedTransport),
        }

        match request.input {
            SurfaceInputPolicy::None => {}
            SurfaceInputPolicy::Pointer if self.pointer_input => {}
            SurfaceInputPolicy::PointerKeyboard if self.pointer_input && self.keyboard_input => {}
            _ => return Err(SurfaceError::UnsupportedInput),
        }

        Ok(())
    }
}

pub struct SurfaceTable {
    slots: [Option<SurfaceSlot>; SURFACE_CAPACITY],
    generations: [u8; SURFACE_CAPACITY],
    overflowed: bool,
}

impl SurfaceTable {
    pub const fn new() -> Self {
        Self {
            slots: [None; SURFACE_CAPACITY],
            generations: [0; SURFACE_CAPACITY],
            overflowed: false,
        }
    }

    pub fn create(
        &mut self,
        capabilities: SurfaceCapabilities,
        request: SurfaceRequest,
    ) -> Result<SurfaceDescriptor, SurfaceError> {
        capabilities.validate(request)?;

        let Some(index) = self.free_slot(capabilities.max_surfaces as usize) else {
            self.overflowed = true;
            return Err(SurfaceError::CapacityFull);
        };

        let generation = next_generation(self.generations[index]);
        self.generations[index] = generation;

        let descriptor = SurfaceDescriptor {
            id: SurfaceId::from_parts(index as u8, generation),
            request,
            stride_bytes: request.stride_bytes().ok_or(SurfaceError::InvalidGeometry)?,
            buffer_count: request.buffering.buffer_count(),
            transport: SurfaceTransportDescriptor::new(
                SurfaceId::from_parts(index as u8, generation),
                request.owner,
            ),
        };

        self.slots[index] = Some(SurfaceSlot {
            descriptor,
            visible: true,
            frame_serial: 0,
            dirty: Some(Rect::new(0, 0, request.size.w, request.size.h)),
        });

        Ok(descriptor)
    }

    pub fn destroy(&mut self, id: SurfaceId) -> bool {
        let Some(index) = self.valid_index(id) else {
            return false;
        };
        self.slots[index] = None;
        true
    }

    pub fn mark_dirty(&mut self, id: SurfaceId, rect: Rect) -> Result<(), SurfaceError> {
        if rect.is_empty() {
            return Ok(());
        }

        let Some(slot) = self.get_mut(id) else {
            return Err(SurfaceError::InvalidHandle);
        };

        slot.dirty = Some(match slot.dirty {
            None => rect,
            Some(current) => current.union(rect),
        });
        slot.frame_serial = slot.frame_serial.wrapping_add(1);
        Ok(())
    }

    pub fn resolve_handle(
        &self,
        handle: SurfaceHandle,
        token: u32,
    ) -> Result<SurfaceEndpoint, SurfaceError> {
        let kind = handle.kind().ok_or(SurfaceError::InvalidHandle)?;
        let slot = self
            .get(handle.id())
            .ok_or(SurfaceError::InvalidHandle)?;

        if slot.descriptor.transport.token != token {
            return Err(SurfaceError::InvalidToken);
        }

        let expected = match kind {
            SurfaceHandleKind::Frame => slot.descriptor.transport.frame,
            SurfaceHandleKind::Dirty => slot.descriptor.transport.dirty,
            SurfaceHandleKind::Input => slot.descriptor.transport.input,
        };

        if expected != handle {
            return Err(SurfaceError::InvalidHandle);
        }

        Ok(SurfaceEndpoint {
            id: slot.descriptor.id,
            kind,
            descriptor: slot.descriptor,
        })
    }

    pub fn submit_dirty(
        &mut self,
        handle: SurfaceHandle,
        token: u32,
        rect: Rect,
    ) -> Result<(), SurfaceError> {
        let endpoint = self.resolve_handle(handle, token)?;
        if endpoint.kind != SurfaceHandleKind::Dirty {
            return Err(SurfaceError::InvalidHandle);
        }

        self.mark_dirty(endpoint.id, clamp_surface_rect(endpoint.descriptor, rect))
    }

    pub fn take_dirty(&mut self, id: SurfaceId) -> Result<Option<Rect>, SurfaceError> {
        let Some(slot) = self.get_mut(id) else {
            return Err(SurfaceError::InvalidHandle);
        };
        let dirty = slot.dirty;
        slot.dirty = None;
        Ok(dirty)
    }

    pub fn get(&self, id: SurfaceId) -> Option<SurfaceSlot> {
        let index = self.valid_index(id)?;
        self.slots[index]
    }

    pub fn len(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_some()).count()
    }

    pub fn capacity(&self) -> usize {
        SURFACE_CAPACITY
    }

    pub fn overflowed(&self) -> bool {
        self.overflowed
    }

    fn free_slot(&self, max_surfaces: usize) -> Option<usize> {
        let limit = max_surfaces.min(SURFACE_CAPACITY);
        self.slots[..limit].iter().position(Option::is_none)
    }

    fn get_mut(&mut self, id: SurfaceId) -> Option<&mut SurfaceSlot> {
        let index = self.valid_index(id)?;
        self.slots[index].as_mut()
    }

    fn valid_index(&self, id: SurfaceId) -> Option<usize> {
        let index = id.slot();
        if index >= SURFACE_CAPACITY {
            return None;
        }

        let slot = self.slots[index]?;
        if slot.descriptor.id.generation() == id.generation() {
            Some(index)
        } else {
            None
        }
    }
}

fn clamp_surface_rect(descriptor: SurfaceDescriptor, rect: Rect) -> Rect {
    let x0 = rect.x.max(0).min(descriptor.request.size.w as i32);
    let y0 = rect.y.max(0).min(descriptor.request.size.h as i32);
    let x1 = (rect.x + rect.w as i32)
        .max(0)
        .min(descriptor.request.size.w as i32);
    let y1 = (rect.y + rect.h as i32)
        .max(0)
        .min(descriptor.request.size.h as i32);

    if x1 <= x0 || y1 <= y0 {
        Rect::new(0, 0, 0, 0)
    } else {
        Rect::new(x0, y0, (x1 - x0) as u16, (y1 - y0) as u16)
    }
}

impl Default for SurfaceTable {
    fn default() -> Self {
        Self::new()
    }
}

fn next_generation(current: u8) -> u8 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}
