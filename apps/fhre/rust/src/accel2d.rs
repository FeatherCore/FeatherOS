use crate::{
    backend::{
        DrawChain, DrawChainOpKind, DrawChainOpPayload, DrawChainRunContract,
        DrawChainRunDescriptor,
    },
    image::{ImageFormat, ImageResolver, ImageView},
    mock_accel::{resolve_draw_chain_image_descriptor, DrawChainImageResolveError},
    Color, ImageFit, ImageId, PixelFormat, Rect, Surface,
};

const ACCEL2D_FENCE_START: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Accel2dCommandBuildError {
    EmptyChain,
    ContractMismatch,
    Overflow,
    EmptyFramebuffer,
    UnsupportedOp,
    PayloadMismatch,
    UnsupportedState,
    SourceOutOfBounds,
    Image(DrawChainImageResolveError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Accel2dRingError {
    Overflow,
    EmptySubmission,
    InvalidFence,
    NotSubmitted,
    AlreadyCompleted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Accel2dExecutorError {
    Unsupported,
    Fallback,
    Overflow,
    SubmitFailed,
    FenceFailed,
    Ring(Accel2dRingError),
}

impl Accel2dExecutorError {
    pub const fn from_ring(error: Accel2dRingError) -> Self {
        match error {
            Accel2dRingError::Overflow => Self::Overflow,
            Accel2dRingError::EmptySubmission => Self::SubmitFailed,
            Accel2dRingError::InvalidFence
            | Accel2dRingError::NotSubmitted
            | Accel2dRingError::AlreadyCompleted => Self::FenceFailed,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Accel2dFenceState {
    Empty,
    Queued,
    Submitted,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Accel2dFence(pub u32);

impl Accel2dFence {
    pub const INVALID: Self = Self(0);
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Accel2dFormatSupport {
    pub rgb565: bool,
    pub rgb888: bool,
    pub rgba8888: bool,
    pub a8: bool,
}

impl Accel2dFormatSupport {
    pub const NONE: Self = Self {
        rgb565: false,
        rgb888: false,
        rgba8888: false,
        a8: false,
    };

    pub const RGB_COLOR: Self = Self {
        rgb565: true,
        rgb888: true,
        rgba8888: true,
        a8: false,
    };

    pub const IMAGE_COLOR: Self = Self {
        rgb565: true,
        rgb888: true,
        rgba8888: true,
        a8: true,
    };

    pub const fn supports_pixel(self, format: PixelFormat) -> bool {
        match format {
            PixelFormat::Rgb565 => self.rgb565,
            PixelFormat::Rgb888 => self.rgb888,
            PixelFormat::Rgba8888 => self.rgba8888,
            PixelFormat::Unknown(_) => false,
        }
    }

    pub const fn supports_image(self, format: ImageFormat) -> bool {
        match format {
            ImageFormat::Rgb565 => self.rgb565,
            ImageFormat::Rgb888 => self.rgb888,
            ImageFormat::Rgba8888 => self.rgba8888,
            ImageFormat::A8 => self.a8,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Accel2dFramebufferDescriptor {
    pub data: *mut u8,
    pub len: usize,
    pub width: u16,
    pub height: u16,
    pub stride: usize,
    pub bpp: u8,
    pub format: PixelFormat,
    pub clip: Option<Rect>,
}

impl Accel2dFramebufferDescriptor {
    pub const EMPTY: Self = Self {
        data: core::ptr::null_mut(),
        len: 0,
        width: 0,
        height: 0,
        stride: 0,
        bpp: 0,
        format: PixelFormat::Unknown(0),
        clip: None,
    };

    pub fn from_surface(surface: &Surface) -> Result<Self, Accel2dCommandBuildError> {
        if surface.pixels.is_null()
            || surface.width == 0
            || surface.height == 0
            || surface.stride == 0
            || surface.bpp == 0
        {
            return Err(Accel2dCommandBuildError::EmptyFramebuffer);
        }
        let len = surface
            .stride
            .checked_mul(surface.height as usize)
            .ok_or(Accel2dCommandBuildError::Overflow)?;
        if len == 0 {
            return Err(Accel2dCommandBuildError::EmptyFramebuffer);
        }

        Ok(Self {
            data: surface.pixels,
            len,
            width: surface.width,
            height: surface.height,
            stride: surface.stride,
            bpp: surface.bpp,
            format: surface.format,
            clip: surface.clip,
        })
    }

    pub const fn is_valid(self) -> bool {
        !self.data.is_null()
            && self.len > 0
            && self.width > 0
            && self.height > 0
            && self.stride > 0
            && self.bpp > 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Accel2dImageSourceDescriptor {
    pub data: *const u8,
    pub len: usize,
    pub width: u16,
    pub height: u16,
    pub stride: usize,
    pub format: ImageFormat,
}

impl Accel2dImageSourceDescriptor {
    pub fn from_view(view: ImageView) -> Result<Self, Accel2dCommandBuildError> {
        if view.data.is_null()
            || view.width == 0
            || view.height == 0
            || view.stride == 0
            || view.len == 0
        {
            return Err(Accel2dCommandBuildError::Image(
                DrawChainImageResolveError::EmptyImage,
            ));
        }
        let required_len = view
            .stride
            .checked_mul(view.height as usize)
            .ok_or(Accel2dCommandBuildError::Overflow)?;
        if view.len < required_len {
            return Err(Accel2dCommandBuildError::SourceOutOfBounds);
        }

        Ok(Self {
            data: view.data,
            len: view.len,
            width: view.width,
            height: view.height,
            stride: view.stride,
            format: view.format,
        })
    }

    pub const fn image_view(self) -> ImageView {
        ImageView {
            width: self.width,
            height: self.height,
            stride: self.stride,
            format: self.format,
            data: self.data,
            len: self.len,
        }
    }

    pub const fn is_valid(self) -> bool {
        !self.data.is_null() && self.len > 0 && self.width > 0 && self.height > 0 && self.stride > 0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Accel2dExecutorCapabilities {
    pub fill: bool,
    pub image: bool,
    pub clip: bool,
    pub target_formats: Accel2dFormatSupport,
    pub source_formats: Accel2dFormatSupport,
    pub sync_fence: bool,
    pub async_fence: bool,
}

impl Accel2dExecutorCapabilities {
    pub const NONE: Self = Self {
        fill: false,
        image: false,
        clip: false,
        target_formats: Accel2dFormatSupport::NONE,
        source_formats: Accel2dFormatSupport::NONE,
        sync_fence: false,
        async_fence: false,
    };

    pub const SURFACE: Self = Self {
        fill: true,
        image: true,
        clip: true,
        target_formats: Accel2dFormatSupport::RGB_COLOR,
        source_formats: Accel2dFormatSupport::IMAGE_COLOR,
        sync_fence: true,
        async_fence: false,
    };

    pub const OPENGL_PROBE: Self = Self {
        fill: true,
        image: true,
        clip: true,
        target_formats: Accel2dFormatSupport::RGB_COLOR,
        source_formats: Accel2dFormatSupport::IMAGE_COLOR,
        sync_fence: true,
        async_fence: false,
    };

    pub const fn supports_framebuffer(self, framebuffer: Accel2dFramebufferDescriptor) -> bool {
        framebuffer.is_valid() && self.target_formats.supports_pixel(framebuffer.format)
    }

    pub const fn supports_source(self, source: Accel2dImageSourceDescriptor) -> bool {
        source.is_valid() && self.source_formats.supports_image(source.format)
    }
}

pub trait Accel2dExecutor<const COMMANDS: usize> {
    fn capabilities(&self) -> Accel2dExecutorCapabilities;

    fn queue(
        &mut self,
        command_list: Accel2dCommandList<COMMANDS>,
    ) -> Result<Accel2dFence, Accel2dExecutorError>;

    fn flush(&mut self) -> Result<u32, Accel2dExecutorError>;

    fn execute_submitted(&mut self) -> Result<u32, Accel2dExecutorError>;

    fn complete_fence(&mut self, fence: Accel2dFence) -> Result<(), Accel2dExecutorError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Accel2dCommand {
    SetClip {
        clip: Option<Rect>,
    },
    Fill {
        rect: Rect,
        clip: Option<Rect>,
        color: Color,
    },
    Image {
        rect: Rect,
        clip: Option<Rect>,
        image: ImageId,
        source: Accel2dImageSourceDescriptor,
        opacity: u8,
        fit: ImageFit,
        tint: Option<Color>,
    },
}

impl Accel2dCommand {
    pub const EMPTY: Self = Self::SetClip { clip: None };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Accel2dCommandList<const COMMANDS: usize> {
    framebuffer: Accel2dFramebufferDescriptor,
    run: DrawChainRunDescriptor,
    commands: [Accel2dCommand; COMMANDS],
    len: usize,
}

impl<const COMMANDS: usize> Accel2dCommandList<COMMANDS> {
    pub const EMPTY: Self = Self {
        framebuffer: Accel2dFramebufferDescriptor::EMPTY,
        run: DrawChainRunDescriptor::EMPTY,
        commands: [Accel2dCommand::EMPTY; COMMANDS],
        len: 0,
    };

    pub const fn new(
        framebuffer: Accel2dFramebufferDescriptor,
        run: DrawChainRunDescriptor,
    ) -> Self {
        Self {
            framebuffer,
            run,
            commands: [Accel2dCommand::EMPTY; COMMANDS],
            len: 0,
        }
    }

    pub const fn framebuffer(&self) -> Accel2dFramebufferDescriptor {
        self.framebuffer
    }

    pub const fn run(&self) -> DrawChainRunDescriptor {
        self.run
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn commands(&self) -> &[Accel2dCommand] {
        &self.commands[..self.len]
    }

    pub fn push(&mut self, command: Accel2dCommand) -> Result<(), Accel2dCommandBuildError> {
        if self.len >= COMMANDS {
            return Err(Accel2dCommandBuildError::Overflow);
        }
        self.commands[self.len] = command;
        self.len += 1;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Accel2dSubmission<const COMMANDS: usize> {
    fence: Accel2dFence,
    sequence: u32,
    state: Accel2dFenceState,
    list: Accel2dCommandList<COMMANDS>,
}

impl<const COMMANDS: usize> Accel2dSubmission<COMMANDS> {
    pub const EMPTY: Self = Self {
        fence: Accel2dFence::INVALID,
        sequence: 0,
        state: Accel2dFenceState::Empty,
        list: Accel2dCommandList::EMPTY,
    };

    pub const fn fence(&self) -> Accel2dFence {
        self.fence
    }

    pub const fn sequence(&self) -> u32 {
        self.sequence
    }

    pub const fn state(&self) -> Accel2dFenceState {
        self.state
    }

    pub const fn run(&self) -> DrawChainRunDescriptor {
        self.list.run()
    }

    pub const fn command_list(&self) -> &Accel2dCommandList<COMMANDS> {
        &self.list
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Accel2dSubmissionRing<const RUNS: usize, const COMMANDS: usize> {
    submissions: [Accel2dSubmission<COMMANDS>; RUNS],
    len: usize,
    next_sequence: u32,
    next_fence: u32,
    flushes: u32,
}

impl<const RUNS: usize, const COMMANDS: usize> Accel2dSubmissionRing<RUNS, COMMANDS> {
    pub const fn new() -> Self {
        Self {
            submissions: [Accel2dSubmission::EMPTY; RUNS],
            len: 0,
            next_sequence: 1,
            next_fence: ACCEL2D_FENCE_START,
            flushes: 0,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn flushes(&self) -> u32 {
        self.flushes
    }

    pub fn submissions(&self) -> &[Accel2dSubmission<COMMANDS>] {
        &self.submissions[..self.len]
    }

    pub fn queue(
        &mut self,
        list: Accel2dCommandList<COMMANDS>,
    ) -> Result<Accel2dFence, Accel2dRingError> {
        if list.is_empty() {
            return Err(Accel2dRingError::EmptySubmission);
        }
        if self.len >= RUNS {
            return Err(Accel2dRingError::Overflow);
        }

        let fence = Accel2dFence(self.next_fence);
        self.submissions[self.len] = Accel2dSubmission {
            fence,
            sequence: self.next_sequence,
            state: Accel2dFenceState::Queued,
            list,
        };
        self.len += 1;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.next_fence = self.next_fence.saturating_add(1).max(ACCEL2D_FENCE_START);
        Ok(fence)
    }

    pub fn flush(&mut self) -> Result<u32, Accel2dRingError> {
        self.flushes = self.flushes.saturating_add(1);
        let mut submitted = 0u32;
        let mut index = 0usize;
        while index < self.len {
            if self.submissions[index].state == Accel2dFenceState::Queued {
                self.submissions[index].state = Accel2dFenceState::Submitted;
                submitted = submitted.saturating_add(1);
            }
            index += 1;
        }
        Ok(submitted)
    }

    pub fn complete_fence(&mut self, fence: Accel2dFence) -> Result<(), Accel2dRingError> {
        let Some(index) = self.find_fence(fence) else {
            return Err(Accel2dRingError::InvalidFence);
        };
        match self.submissions[index].state {
            Accel2dFenceState::Submitted => {
                self.submissions[index].state = Accel2dFenceState::Completed;
                Ok(())
            }
            Accel2dFenceState::Queued => Err(Accel2dRingError::NotSubmitted),
            Accel2dFenceState::Completed => Err(Accel2dRingError::AlreadyCompleted),
            Accel2dFenceState::Empty => Err(Accel2dRingError::InvalidFence),
        }
    }

    fn find_fence(&self, fence: Accel2dFence) -> Option<usize> {
        let mut index = 0usize;
        while index < self.len {
            if self.submissions[index].fence == fence {
                return Some(index);
            }
            index += 1;
        }
        None
    }
}

pub struct SurfaceAccel2dExecutor<'a, const RUNS: usize, const COMMANDS: usize> {
    surface: &'a mut Surface,
    ring: Accel2dSubmissionRing<RUNS, COMMANDS>,
}

pub fn execute_accel_2d_command_list_on_surface<const COMMANDS: usize>(
    surface: &mut Surface,
    list: &Accel2dCommandList<COMMANDS>,
) {
    let old_clip = surface.clip();
    for command in list.commands().iter().copied() {
        match command {
            Accel2dCommand::SetClip { clip } => {
                surface.set_clip(clip);
            }
            Accel2dCommand::Fill { rect, clip, color } => {
                surface.set_clip(clip);
                surface.fill_rect(rect, color);
            }
            Accel2dCommand::Image {
                rect,
                clip,
                source,
                opacity,
                fit,
                tint,
                ..
            } => {
                let view = source.image_view();
                surface.set_clip(clip);
                if let Some(tint) = tint {
                    surface.draw_image_view_tint(rect, view, opacity, fit, tint);
                } else {
                    surface.draw_image_view_fit(rect, view, opacity, fit);
                }
            }
        }
    }
    surface.set_clip(old_clip);
}

impl<'a, const RUNS: usize, const COMMANDS: usize> SurfaceAccel2dExecutor<'a, RUNS, COMMANDS> {
    pub const fn new(surface: &'a mut Surface) -> Self {
        Self {
            surface,
            ring: Accel2dSubmissionRing::new(),
        }
    }

    pub fn ring(&self) -> &Accel2dSubmissionRing<RUNS, COMMANDS> {
        &self.ring
    }

    pub fn surface(&self) -> &Surface {
        &*self.surface
    }

    fn validate_command_list(
        &self,
        list: &Accel2dCommandList<COMMANDS>,
    ) -> Result<(), Accel2dExecutorError> {
        let capabilities = self.capabilities();
        if list.is_empty() {
            return Err(Accel2dExecutorError::SubmitFailed);
        }
        if !capabilities.supports_framebuffer(list.framebuffer()) {
            return Err(Accel2dExecutorError::Unsupported);
        }

        for command in list.commands().iter().copied() {
            match command {
                Accel2dCommand::SetClip { .. } if !capabilities.clip => {
                    return Err(Accel2dExecutorError::Unsupported)
                }
                Accel2dCommand::Fill { .. } if !capabilities.fill => {
                    return Err(Accel2dExecutorError::Unsupported)
                }
                Accel2dCommand::Image { source, .. }
                    if !capabilities.image || !capabilities.supports_source(source) =>
                {
                    return Err(Accel2dExecutorError::Unsupported)
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn execute_command_list(&mut self, list: &Accel2dCommandList<COMMANDS>) {
        execute_accel_2d_command_list_on_surface(self.surface, list);
    }
}

impl<const RUNS: usize, const COMMANDS: usize> Accel2dExecutor<COMMANDS>
    for SurfaceAccel2dExecutor<'_, RUNS, COMMANDS>
{
    fn capabilities(&self) -> Accel2dExecutorCapabilities {
        Accel2dExecutorCapabilities::SURFACE
    }

    fn queue(
        &mut self,
        command_list: Accel2dCommandList<COMMANDS>,
    ) -> Result<Accel2dFence, Accel2dExecutorError> {
        self.validate_command_list(&command_list)?;
        self.ring
            .queue(command_list)
            .map_err(Accel2dExecutorError::from_ring)
    }

    fn flush(&mut self) -> Result<u32, Accel2dExecutorError> {
        self.ring.flush().map_err(Accel2dExecutorError::from_ring)
    }

    fn execute_submitted(&mut self) -> Result<u32, Accel2dExecutorError> {
        let mut executed = 0u32;
        let mut index = 0usize;
        while index < self.ring.len() {
            let submission = self.ring.submissions()[index];
            if submission.state() == Accel2dFenceState::Submitted {
                self.execute_command_list(submission.command_list());
                executed = executed.saturating_add(1);
            }
            index += 1;
        }
        if executed == 0 {
            return Err(Accel2dExecutorError::FenceFailed);
        }
        Ok(executed)
    }

    fn complete_fence(&mut self, fence: Accel2dFence) -> Result<(), Accel2dExecutorError> {
        self.ring
            .complete_fence(fence)
            .map_err(Accel2dExecutorError::from_ring)
    }
}

pub fn build_accel_2d_command_list<const OPS: usize, const COMMANDS: usize>(
    chain: &DrawChain<OPS>,
    contract: &DrawChainRunContract<OPS>,
    surface: &Surface,
) -> Result<Accel2dCommandList<COMMANDS>, Accel2dCommandBuildError> {
    let framebuffer = Accel2dFramebufferDescriptor::from_surface(surface)?;
    build_accel_2d_command_list_for_framebuffer(
        chain,
        contract,
        framebuffer,
        surface.image_resolver,
    )
}

pub fn build_accel_2d_command_list_for_framebuffer<const OPS: usize, const COMMANDS: usize>(
    chain: &DrawChain<OPS>,
    contract: &DrawChainRunContract<OPS>,
    framebuffer: Accel2dFramebufferDescriptor,
    resolver: Option<ImageResolver>,
) -> Result<Accel2dCommandList<COMMANDS>, Accel2dCommandBuildError> {
    if chain.is_empty() {
        return Err(Accel2dCommandBuildError::EmptyChain);
    }
    if contract.len != chain.len() {
        return Err(Accel2dCommandBuildError::ContractMismatch);
    }
    if contract.len > COMMANDS {
        return Err(Accel2dCommandBuildError::Overflow);
    }
    if contract.descriptor.has_mask || contract.descriptor.has_layer {
        return Err(Accel2dCommandBuildError::UnsupportedState);
    }

    let mut list = Accel2dCommandList::new(framebuffer, contract.descriptor);
    for (index, op) in chain.ops().iter().copied().enumerate() {
        if contract.kinds[index] != op.kind {
            return Err(Accel2dCommandBuildError::ContractMismatch);
        }

        let command =
            match (op.kind, op.payload) {
                (DrawChainOpKind::Clip, DrawChainOpPayload::Clip { clip }) => {
                    Accel2dCommand::SetClip { clip }
                }
                (DrawChainOpKind::SolidFill, DrawChainOpPayload::FillRect { rect, color })
                    if color.a == 255 =>
                {
                    Accel2dCommand::Fill {
                        rect,
                        clip: op.clip,
                        color,
                    }
                }
                (DrawChainOpKind::AlphaFill, DrawChainOpPayload::FillRect { rect, color })
                    if color.a < 255 =>
                {
                    Accel2dCommand::Fill {
                        rect,
                        clip: op.clip,
                        color,
                    }
                }
                (
                    DrawChainOpKind::ImageBlit,
                    payload @ DrawChainOpPayload::Image { opacity, .. },
                ) if opacity == 255 => build_image_command(payload, op.clip, resolver)?,
                (
                    DrawChainOpKind::ImageBlend,
                    payload @ DrawChainOpPayload::Image { opacity, .. },
                ) if opacity < 255 => build_image_command(payload, op.clip, resolver)?,
                (
                    DrawChainOpKind::MaskEnter
                    | DrawChainOpKind::MaskExit
                    | DrawChainOpKind::LayerEnter
                    | DrawChainOpKind::LayerExit,
                    _,
                ) => return Err(Accel2dCommandBuildError::UnsupportedState),
                (DrawChainOpKind::FallbackRange, _) => {
                    return Err(Accel2dCommandBuildError::UnsupportedOp)
                }
                _ => return Err(Accel2dCommandBuildError::PayloadMismatch),
            };
        list.push(command)?;
    }

    Ok(list)
}

fn build_image_command(
    payload: DrawChainOpPayload,
    clip: Option<Rect>,
    resolver: Option<ImageResolver>,
) -> Result<Accel2dCommand, Accel2dCommandBuildError> {
    let descriptor = resolve_draw_chain_image_descriptor(payload, clip, resolver)
        .map_err(Accel2dCommandBuildError::Image)?;
    let source = Accel2dImageSourceDescriptor::from_view(descriptor.view)?;
    Ok(Accel2dCommand::Image {
        rect: descriptor.rect,
        clip: descriptor.clip,
        image: descriptor.image,
        source,
        opacity: descriptor.opacity,
        fit: descriptor.fit,
        tint: descriptor.tint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend::{DrawChainOp, DrawChainRunDescriptor},
        Color, ImageId, Rect,
    };

    static TEST_IMAGE_RGB565: [u8; 8] = [0x00, 0xf8, 0xe0, 0x07, 0x1f, 0x00, 0xff, 0xff];

    fn test_surface(buffer: &mut [u8], width: u16, height: u16) -> Surface {
        unsafe {
            Surface::from_raw(
                buffer.as_mut_ptr(),
                width,
                height,
                width as usize * 2,
                16,
                11,
            )
        }
    }

    fn test_image_resolver(id: ImageId) -> Option<ImageView> {
        if id == ImageId(77) {
            Some(ImageView::new(
                2,
                2,
                4,
                ImageFormat::Rgb565,
                &TEST_IMAGE_RGB565,
            ))
        } else {
            None
        }
    }

    fn contract_for<const OPS: usize>(
        chain: &DrawChain<OPS>,
        descriptor: DrawChainRunDescriptor,
    ) -> DrawChainRunContract<OPS> {
        let mut contract = DrawChainRunContract::EMPTY;
        for op in chain.ops().iter().copied() {
            assert!(contract.push_kind(op.kind));
        }
        contract.set_descriptor(descriptor);
        contract
    }

    #[test]
    fn packet_builder_emits_framebuffer_and_source_descriptors() {
        let mut pixels = [0u8; 4 * 4 * 2];
        let mut surface = test_surface(&mut pixels, 4, 4);
        surface.set_image_resolver(Some(test_image_resolver));

        let mut chain = DrawChain::<4>::new();
        assert!(chain.push(DrawChainOp::new(
            DrawChainOpKind::Clip,
            None,
            Rect::new(1, 1, 2, 2),
            Some(Rect::new(1, 1, 2, 2)),
            DrawChainOpPayload::Clip {
                clip: Some(Rect::new(1, 1, 2, 2)),
            },
            0,
            0,
        )));
        assert!(chain.push(DrawChainOp::new(
            DrawChainOpKind::SolidFill,
            None,
            Rect::new(1, 1, 2, 2),
            Some(Rect::new(1, 1, 2, 2)),
            DrawChainOpPayload::FillRect {
                rect: Rect::new(1, 1, 2, 2),
                color: Color::rgb(12, 24, 36),
            },
            0,
            1,
        )));
        assert!(chain.push(DrawChainOp::new(
            DrawChainOpKind::ImageBlend,
            None,
            Rect::new(0, 0, 2, 2),
            Some(Rect::new(0, 0, 2, 2)),
            DrawChainOpPayload::Image {
                rect: Rect::new(0, 0, 2, 2),
                image: ImageId(77),
                opacity: 128,
                fit: ImageFit::Stretch,
                tint: Some(Color::WHITE),
            },
            1,
            1,
        )));

        let contract = contract_for(
            &chain,
            DrawChainRunDescriptor {
                command_start: 0,
                command_end: 2,
                has_clip: true,
                has_mask: false,
                has_layer: false,
                bounds: Rect::new(0, 0, 3, 3),
            },
        );
        let list =
            build_accel_2d_command_list::<4, 4>(&chain, &contract, &surface).expect("packet");

        assert_eq!(list.len(), 3);
        assert_eq!(list.framebuffer().format, PixelFormat::Rgb565);
        assert_eq!(list.framebuffer().stride, 8);
        assert_eq!(list.framebuffer().len, 32);
        assert_eq!(
            list.commands()[0],
            Accel2dCommand::SetClip {
                clip: Some(Rect::new(1, 1, 2, 2))
            }
        );
        assert_eq!(
            list.commands()[1],
            Accel2dCommand::Fill {
                rect: Rect::new(1, 1, 2, 2),
                clip: Some(Rect::new(1, 1, 2, 2)),
                color: Color::rgb(12, 24, 36),
            }
        );
        match list.commands()[2] {
            Accel2dCommand::Image {
                source,
                opacity,
                fit,
                tint,
                clip,
                ..
            } => {
                assert_eq!(source.format, ImageFormat::Rgb565);
                assert_eq!(source.stride, 4);
                assert_eq!(source.len, TEST_IMAGE_RGB565.len());
                assert_eq!(opacity, 128);
                assert_eq!(fit, ImageFit::Stretch);
                assert_eq!(tint, Some(Color::WHITE));
                assert_eq!(clip, Some(Rect::new(0, 0, 2, 2)));
            }
            _ => panic!("expected image packet"),
        }
    }

    #[test]
    fn packet_builder_rejects_before_pixels_are_written() {
        let mut pixels = [0u8; 2 * 2 * 2];
        let surface = test_surface(&mut pixels, 2, 2);
        let mut chain = DrawChain::<2>::new();
        assert!(chain.push(DrawChainOp::new(
            DrawChainOpKind::ImageBlit,
            None,
            Rect::new(0, 0, 2, 2),
            None,
            DrawChainOpPayload::Image {
                rect: Rect::new(0, 0, 2, 2),
                image: ImageId(404),
                opacity: 255,
                fit: ImageFit::Stretch,
                tint: None,
            },
            0,
            1,
        )));
        let contract = contract_for(&chain, DrawChainRunDescriptor::EMPTY);

        assert_eq!(
            build_accel_2d_command_list::<2, 2>(&chain, &contract, &surface),
            Err(Accel2dCommandBuildError::Image(
                DrawChainImageResolveError::MissingImage
            ))
        );

        let empty_surface = unsafe { Surface::from_raw(core::ptr::null_mut(), 2, 2, 4, 16, 11) };
        assert_eq!(
            build_accel_2d_command_list::<2, 2>(&chain, &contract, &empty_surface),
            Err(Accel2dCommandBuildError::EmptyFramebuffer)
        );
    }

    #[test]
    fn packet_builder_rejects_overflow_and_stateful_runs() {
        let mut pixels = [0u8; 2 * 2 * 2];
        let surface = test_surface(&mut pixels, 2, 2);
        let mut chain = DrawChain::<2>::new();
        assert!(chain.push(DrawChainOp::new(
            DrawChainOpKind::SolidFill,
            None,
            Rect::new(0, 0, 1, 1),
            None,
            DrawChainOpPayload::FillRect {
                rect: Rect::new(0, 0, 1, 1),
                color: Color::WHITE,
            },
            0,
            1,
        )));
        assert!(chain.push(DrawChainOp::new(
            DrawChainOpKind::AlphaFill,
            None,
            Rect::new(1, 1, 1, 1),
            None,
            DrawChainOpPayload::FillRect {
                rect: Rect::new(1, 1, 1, 1),
                color: Color::rgba(0, 0, 0, 128),
            },
            1,
            1,
        )));
        let contract = contract_for(&chain, DrawChainRunDescriptor::EMPTY);

        assert_eq!(
            build_accel_2d_command_list::<2, 1>(&chain, &contract, &surface),
            Err(Accel2dCommandBuildError::Overflow)
        );

        let mut stateful_contract = contract;
        stateful_contract.set_descriptor(DrawChainRunDescriptor {
            has_mask: true,
            ..DrawChainRunDescriptor::EMPTY
        });
        assert_eq!(
            build_accel_2d_command_list::<2, 2>(&chain, &stateful_contract, &surface),
            Err(Accel2dCommandBuildError::UnsupportedState)
        );
    }

    #[test]
    fn submission_ring_queues_flushes_and_completes_in_order() {
        let mut pixels = [0u8; 4 * 2 * 2];
        let surface = test_surface(&mut pixels, 4, 2);
        let framebuffer = Accel2dFramebufferDescriptor::from_surface(&surface).unwrap();
        let mut first = Accel2dCommandList::<2>::new(framebuffer, DrawChainRunDescriptor::EMPTY);
        first
            .push(Accel2dCommand::Fill {
                rect: Rect::new(0, 0, 2, 2),
                clip: None,
                color: Color::rgb(1, 2, 3),
            })
            .unwrap();
        let mut second = Accel2dCommandList::<2>::new(framebuffer, DrawChainRunDescriptor::EMPTY);
        second
            .push(Accel2dCommand::Fill {
                rect: Rect::new(2, 0, 2, 2),
                clip: None,
                color: Color::rgb(4, 5, 6),
            })
            .unwrap();

        let mut ring = Accel2dSubmissionRing::<2, 2>::new();
        let first_fence = ring.queue(first).unwrap();
        let second_fence = ring.queue(second).unwrap();

        assert_eq!(ring.len(), 2);
        assert_eq!(ring.submissions()[0].sequence(), 1);
        assert_eq!(ring.submissions()[1].sequence(), 2);
        assert_eq!(ring.submissions()[0].fence(), first_fence);
        assert_eq!(ring.submissions()[1].fence(), second_fence);
        assert_eq!(ring.submissions()[0].state(), Accel2dFenceState::Queued);
        assert_eq!(ring.flush(), Ok(2));
        assert_eq!(ring.submissions()[0].state(), Accel2dFenceState::Submitted);
        assert_eq!(ring.submissions()[1].state(), Accel2dFenceState::Submitted);
        assert_eq!(ring.complete_fence(first_fence), Ok(()));
        assert_eq!(ring.submissions()[0].state(), Accel2dFenceState::Completed);
        assert_eq!(ring.submissions()[1].state(), Accel2dFenceState::Submitted);
        assert_eq!(ring.complete_fence(second_fence), Ok(()));
        assert_eq!(ring.submissions()[1].state(), Accel2dFenceState::Completed);
    }

    #[test]
    fn submission_ring_reports_stable_errors() {
        let mut pixels = [0u8; 2 * 2 * 2];
        let surface = test_surface(&mut pixels, 2, 2);
        let framebuffer = Accel2dFramebufferDescriptor::from_surface(&surface).unwrap();
        let empty = Accel2dCommandList::<1>::new(framebuffer, DrawChainRunDescriptor::EMPTY);
        let mut list = Accel2dCommandList::<1>::new(framebuffer, DrawChainRunDescriptor::EMPTY);
        list.push(Accel2dCommand::Fill {
            rect: Rect::new(0, 0, 1, 1),
            clip: None,
            color: Color::WHITE,
        })
        .unwrap();

        let mut ring = Accel2dSubmissionRing::<1, 1>::new();
        assert_eq!(ring.queue(empty), Err(Accel2dRingError::EmptySubmission));
        let fence = ring.queue(list).unwrap();
        assert_eq!(ring.queue(list), Err(Accel2dRingError::Overflow));
        assert_eq!(
            ring.complete_fence(Accel2dFence(99)),
            Err(Accel2dRingError::InvalidFence)
        );
        assert_eq!(
            ring.complete_fence(fence),
            Err(Accel2dRingError::NotSubmitted)
        );
        assert_eq!(ring.flush(), Ok(1));
        assert_eq!(ring.complete_fence(fence), Ok(()));
        assert_eq!(
            ring.complete_fence(fence),
            Err(Accel2dRingError::AlreadyCompleted)
        );
    }

    #[test]
    fn surface_executor_executes_packet_and_completes_fence() {
        let mut expected_pixels = [0u8; 4 * 4 * 2];
        let mut actual_pixels = [0u8; 4 * 4 * 2];
        let mut expected = test_surface(&mut expected_pixels, 4, 4);
        let mut actual = test_surface(&mut actual_pixels, 4, 4);
        expected.set_clip(Some(Rect::new(1, 1, 2, 2)));
        expected.fill_rect(Rect::new(0, 0, 4, 4), Color::rgb(10, 30, 50));

        let framebuffer = Accel2dFramebufferDescriptor::from_surface(&actual).unwrap();
        let mut list = Accel2dCommandList::<2>::new(framebuffer, DrawChainRunDescriptor::EMPTY);
        list.push(Accel2dCommand::SetClip {
            clip: Some(Rect::new(1, 1, 2, 2)),
        })
        .unwrap();
        list.push(Accel2dCommand::Fill {
            rect: Rect::new(0, 0, 4, 4),
            clip: Some(Rect::new(1, 1, 2, 2)),
            color: Color::rgb(10, 30, 50),
        })
        .unwrap();

        let mut executor = SurfaceAccel2dExecutor::<1, 2>::new(&mut actual);
        let caps = executor.capabilities();
        assert!(caps.fill);
        assert!(caps.clip);
        assert!(caps.sync_fence);
        let fence = executor.queue(list).unwrap();
        assert_eq!(executor.flush(), Ok(1));
        assert_eq!(executor.execute_submitted(), Ok(1));
        assert_eq!(executor.complete_fence(fence), Ok(()));
        drop(executor);

        assert_eq!(expected_pixels, actual_pixels);
    }

    #[test]
    fn surface_executor_rejects_unsupported_packet_before_pixels_change() {
        let mut pixels = [0u8; 2 * 2 * 2];
        let actual = test_surface(&mut pixels, 2, 2);
        let mut framebuffer = Accel2dFramebufferDescriptor::from_surface(&actual).unwrap();
        framebuffer.format = PixelFormat::Unknown(9);
        let mut list = Accel2dCommandList::<1>::new(framebuffer, DrawChainRunDescriptor::EMPTY);
        list.push(Accel2dCommand::Fill {
            rect: Rect::new(0, 0, 2, 2),
            clip: None,
            color: Color::WHITE,
        })
        .unwrap();

        let mut surface = test_surface(&mut pixels, 2, 2);
        let mut executor = SurfaceAccel2dExecutor::<1, 1>::new(&mut surface);
        assert_eq!(executor.queue(list), Err(Accel2dExecutorError::Unsupported));
        drop(executor);

        assert_eq!(pixels, [0u8; 2 * 2 * 2]);
    }

    #[test]
    fn surface_executor_maps_ring_errors_to_stable_executor_errors() {
        let mut pixels = [0u8; 2 * 2 * 2];
        let mut surface = test_surface(&mut pixels, 2, 2);
        let framebuffer = Accel2dFramebufferDescriptor::from_surface(&surface).unwrap();
        let mut list = Accel2dCommandList::<1>::new(framebuffer, DrawChainRunDescriptor::EMPTY);
        list.push(Accel2dCommand::Fill {
            rect: Rect::new(0, 0, 1, 1),
            clip: None,
            color: Color::WHITE,
        })
        .unwrap();

        let mut executor = SurfaceAccel2dExecutor::<1, 1>::new(&mut surface);
        let fence = executor.queue(list).unwrap();
        assert_eq!(executor.queue(list), Err(Accel2dExecutorError::Overflow));
        assert_eq!(
            executor.complete_fence(fence),
            Err(Accel2dExecutorError::FenceFailed)
        );
    }
}
