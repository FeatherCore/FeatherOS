use crate::{
    accel2d::{
        Accel2dCommand, Accel2dCommandList, Accel2dExecutor, Accel2dExecutorCapabilities,
        Accel2dExecutorError, Accel2dFence, Accel2dImageSourceDescriptor,
        SurfaceAccel2dExecutor,
    },
    Color, ImageFit, ImageId, Rect, Surface,
};

#[cfg(feature = "sim-opengl-egl")]
use crate::{
    accel2d::{
        build_accel_2d_command_list, execute_accel_2d_command_list_on_surface,
        Accel2dCommandBuildError, Accel2dFenceState, Accel2dFramebufferDescriptor,
        Accel2dRingError, Accel2dSubmissionRing,
    },
    backend::{
        BackendCapabilities, DrawChain, DrawChainRunContract, DrawChainSubmitResult,
        DrawFeatureFlags, ParallelDrawChainSubmitResult, ParallelRenderBackend, RenderBackend,
        RenderStats,
    },
    image::ImageFormat,
    DrawCommand, ImageResolver, LayerSpec, MaskSpec, PixelFormat,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OpenGl2dProbeStats {
    pub mapped_commands: u32,
    pub scissor_commands: u32,
    pub fill_quads: u32,
    pub texture_quads: u32,
    pub unsupported: u32,
}

impl OpenGl2dProbeStats {
    pub const fn new() -> Self {
        Self {
            mapped_commands: 0,
            scissor_commands: 0,
            fill_quads: 0,
            texture_quads: 0,
            unsupported: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpenGl2dCommand {
    Scissor {
        clip: Option<Rect>,
    },
    FillQuad {
        rect: Rect,
        clip: Option<Rect>,
        color: Color,
    },
    TextureQuad {
        rect: Rect,
        clip: Option<Rect>,
        image: ImageId,
        source: Accel2dImageSourceDescriptor,
        opacity: u8,
        fit: ImageFit,
        tint: Option<Color>,
    },
}

impl OpenGl2dCommand {
    pub const fn from_accel2d(command: Accel2dCommand) -> Self {
        match command {
            Accel2dCommand::SetClip { clip } => Self::Scissor { clip },
            Accel2dCommand::Fill { rect, clip, color } => Self::FillQuad { rect, clip, color },
            Accel2dCommand::Image {
                rect,
                clip,
                image,
                source,
                opacity,
                fit,
                tint,
            } => Self::TextureQuad {
                rect,
                clip,
                image,
                source,
                opacity,
                fit,
                tint,
            },
        }
    }
}

pub struct SimOpenGlAccel2dExecutor<'a, const RUNS: usize, const COMMANDS: usize> {
    surface_executor: SurfaceAccel2dExecutor<'a, RUNS, COMMANDS>,
    stats: OpenGl2dProbeStats,
}

impl<'a, const RUNS: usize, const COMMANDS: usize> SimOpenGlAccel2dExecutor<'a, RUNS, COMMANDS> {
    pub const fn new(surface: &'a mut Surface) -> Self {
        Self {
            surface_executor: SurfaceAccel2dExecutor::new(surface),
            stats: OpenGl2dProbeStats::new(),
        }
    }

    pub const fn stats(&self) -> OpenGl2dProbeStats {
        self.stats
    }

    fn map_command_list(
        &mut self,
        list: &Accel2dCommandList<COMMANDS>,
    ) -> Result<(), Accel2dExecutorError> {
        let caps = self.capabilities();
        if list.is_empty() {
            return Err(Accel2dExecutorError::SubmitFailed);
        }
        if !caps.supports_framebuffer(list.framebuffer()) {
            self.stats.unsupported = self.stats.unsupported.saturating_add(1);
            return Err(Accel2dExecutorError::Unsupported);
        }

        for command in list.commands().iter().copied() {
            match OpenGl2dCommand::from_accel2d(command) {
                OpenGl2dCommand::Scissor { .. } if caps.clip => {
                    self.stats.scissor_commands = self.stats.scissor_commands.saturating_add(1);
                }
                OpenGl2dCommand::FillQuad { .. } if caps.fill => {
                    self.stats.fill_quads = self.stats.fill_quads.saturating_add(1);
                }
                OpenGl2dCommand::TextureQuad { source, .. }
                    if caps.image && caps.supports_source(source) =>
                {
                    self.stats.texture_quads = self.stats.texture_quads.saturating_add(1);
                }
                _ => {
                    self.stats.unsupported = self.stats.unsupported.saturating_add(1);
                    return Err(Accel2dExecutorError::Unsupported);
                }
            }
            self.stats.mapped_commands = self.stats.mapped_commands.saturating_add(1);
        }

        Ok(())
    }
}

impl<const RUNS: usize, const COMMANDS: usize> Accel2dExecutor<COMMANDS>
    for SimOpenGlAccel2dExecutor<'_, RUNS, COMMANDS>
{
    fn capabilities(&self) -> Accel2dExecutorCapabilities {
        Accel2dExecutorCapabilities::OPENGL_PROBE
    }

    fn queue(
        &mut self,
        command_list: Accel2dCommandList<COMMANDS>,
    ) -> Result<Accel2dFence, Accel2dExecutorError> {
        if self.surface_executor.ring().len() >= RUNS {
            return Err(Accel2dExecutorError::Overflow);
        }
        self.map_command_list(&command_list)?;
        self.surface_executor.queue(command_list)
    }

    fn flush(&mut self) -> Result<u32, Accel2dExecutorError> {
        self.surface_executor.flush()
    }

    fn execute_submitted(&mut self) -> Result<u32, Accel2dExecutorError> {
        self.surface_executor.execute_submitted()
    }

    fn complete_fence(&mut self, fence: Accel2dFence) -> Result<(), Accel2dExecutorError> {
        self.surface_executor.complete_fence(fence)
    }
}

#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_OK: i32 = 0;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_UNAVAILABLE: i32 = 1;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_UNSUPPORTED: i32 = 2;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_OVERFLOW: i32 = 3;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_FENCE_FAILED: i32 = 5;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_COMMAND_SET_CLIP: u8 = 1;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_COMMAND_FILL: u8 = 2;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_COMMAND_IMAGE: u8 = 3;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_FORMAT_RGB565: u8 = 1;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_FORMAT_RGB888: u8 = 2;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_FORMAT_RGBA8888: u8 = 3;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_FORMAT_A8: u8 = 4;
#[cfg(feature = "sim-opengl-egl")]
const FHRE_EGL_FIT_STRETCH: u8 = 0;

#[cfg(feature = "sim-opengl-egl")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FhreEglRect {
    x: i32,
    y: i32,
    w: u16,
    h: u16,
}

#[cfg(feature = "sim-opengl-egl")]
impl FhreEglRect {
    const EMPTY: Self = Self {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    };

    const fn from_rect(rect: Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
        }
    }

    const fn to_rect(self) -> Rect {
        Rect::new(self.x, self.y, self.w, self.h)
    }
}

#[cfg(feature = "sim-opengl-egl")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FhreEglColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[cfg(feature = "sim-opengl-egl")]
impl FhreEglColor {
    const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    const fn from_color(color: Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

#[cfg(feature = "sim-opengl-egl")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FhreEglFramebuffer {
    data: *mut u8,
    len: usize,
    width: u16,
    height: u16,
    stride: usize,
    bpp: u8,
    format: u8,
}

#[cfg(feature = "sim-opengl-egl")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FhreEglImageSource {
    data: *const u8,
    len: usize,
    width: u16,
    height: u16,
    stride: usize,
    format: u8,
}

#[cfg(feature = "sim-opengl-egl")]
impl FhreEglImageSource {
    const EMPTY: Self = Self {
        data: core::ptr::null(),
        len: 0,
        width: 0,
        height: 0,
        stride: 0,
        format: 0,
    };
}

#[cfg(feature = "sim-opengl-egl")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FhreEglCommand {
    kind: u8,
    rect: FhreEglRect,
    has_clip: u8,
    clip: FhreEglRect,
    color: FhreEglColor,
    source: FhreEglImageSource,
    opacity: u8,
    fit: u8,
    has_tint: u8,
    tint: FhreEglColor,
}

#[cfg(feature = "sim-opengl-egl")]
impl FhreEglCommand {
    const EMPTY: Self = Self {
        kind: 0,
        rect: FhreEglRect::EMPTY,
        has_clip: 0,
        clip: FhreEglRect::EMPTY,
        color: FhreEglColor::WHITE,
        source: FhreEglImageSource::EMPTY,
        opacity: 255,
        fit: FHRE_EGL_FIT_STRETCH,
        has_tint: 0,
        tint: FhreEglColor::WHITE,
    };
}

#[cfg(feature = "sim-opengl-egl")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FhreEglPacket {
    framebuffer: FhreEglFramebuffer,
    run_bounds: FhreEglRect,
    commands: *const FhreEglCommand,
    command_count: usize,
}

#[cfg(feature = "sim-opengl-egl")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FhreEglFence {
    token: u32,
}

#[cfg(feature = "sim-opengl-egl")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FhreEglWorkerStatsRaw {
    queued: u32,
    submitted: u32,
    completed: u32,
    readbacks: u32,
    unsupported: u32,
    submit_failures: u32,
    fence_failures: u32,
    overflows: u32,
}

#[cfg(feature = "sim-opengl-egl")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EglOpenGlWorkerStats {
    pub queued: u32,
    pub submitted: u32,
    pub completed: u32,
    pub readbacks: u32,
    pub unsupported: u32,
    pub submit_failures: u32,
    pub fence_failures: u32,
    pub overflows: u32,
}

#[cfg(feature = "sim-opengl-egl")]
impl From<FhreEglWorkerStatsRaw> for EglOpenGlWorkerStats {
    fn from(raw: FhreEglWorkerStatsRaw) -> Self {
        Self {
            queued: raw.queued,
            submitted: raw.submitted,
            completed: raw.completed,
            readbacks: raw.readbacks,
            unsupported: raw.unsupported,
            submit_failures: raw.submit_failures,
            fence_failures: raw.fence_failures,
            overflows: raw.overflows,
        }
    }
}

#[cfg(feature = "sim-opengl-egl")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EglOpenGlWorkerError {
    Packet(Accel2dCommandBuildError),
    Executor(Accel2dExecutorError),
    Worker(i32),
}

#[cfg(feature = "sim-opengl-egl")]
unsafe extern "C" {
    fn fhre_egl2d_available() -> core::ffi::c_int;
    fn fhre_egl2d_init(width: u16, height: u16) -> core::ffi::c_int;
    fn fhre_egl2d_shutdown() -> core::ffi::c_int;
    fn fhre_egl2d_submit(
        packet: *const FhreEglPacket,
        fence: *mut FhreEglFence,
    ) -> core::ffi::c_int;
    fn fhre_egl2d_pending_bounds(out: *mut FhreEglRect) -> core::ffi::c_int;
    fn fhre_egl2d_flush() -> core::ffi::c_int;
    fn fhre_egl2d_stats(out: *mut FhreEglWorkerStatsRaw) -> core::ffi::c_int;
    fn fhre_egl2d_last_error() -> core::ffi::c_int;
}

#[cfg(feature = "sim-opengl-egl")]
pub struct EglOpenGlAccel2dExecutor<'a, const RUNS: usize, const COMMANDS: usize> {
    probe: SimOpenGlAccel2dExecutor<'a, RUNS, COMMANDS>,
}

#[cfg(feature = "sim-opengl-egl")]
impl<'a, const RUNS: usize, const COMMANDS: usize> EglOpenGlAccel2dExecutor<'a, RUNS, COMMANDS> {
    pub const fn new(surface: &'a mut Surface) -> Self {
        Self {
            probe: SimOpenGlAccel2dExecutor::new(surface),
        }
    }

    pub fn egl_available() -> bool {
        unsafe { fhre_egl2d_available() != 0 }
    }

    pub fn egl_last_error() -> i32 {
        unsafe { fhre_egl2d_last_error() as i32 }
    }

    pub const fn stats(&self) -> OpenGl2dProbeStats {
        self.probe.stats()
    }

    fn ensure_available(&mut self) -> Result<(), Accel2dExecutorError> {
        if Self::egl_available() {
            Ok(())
        } else {
            self.probe.stats.unsupported = self.probe.stats.unsupported.saturating_add(1);
            Err(Accel2dExecutorError::Unsupported)
        }
    }
}

#[cfg(feature = "sim-opengl-egl")]
impl<const RUNS: usize, const COMMANDS: usize> Accel2dExecutor<COMMANDS>
    for EglOpenGlAccel2dExecutor<'_, RUNS, COMMANDS>
{
    fn capabilities(&self) -> Accel2dExecutorCapabilities {
        if Self::egl_available() {
            Accel2dExecutorCapabilities::OPENGL_PROBE
        } else {
            Accel2dExecutorCapabilities::NONE
        }
    }

    fn queue(
        &mut self,
        command_list: Accel2dCommandList<COMMANDS>,
    ) -> Result<Accel2dFence, Accel2dExecutorError> {
        self.ensure_available()?;
        self.probe.queue(command_list)
    }

    fn flush(&mut self) -> Result<u32, Accel2dExecutorError> {
        self.ensure_available()?;
        self.probe.flush()
    }

    fn execute_submitted(&mut self) -> Result<u32, Accel2dExecutorError> {
        self.ensure_available()?;
        self.probe.execute_submitted()
    }

    fn complete_fence(&mut self, fence: Accel2dFence) -> Result<(), Accel2dExecutorError> {
        self.ensure_available()?;
        self.probe.complete_fence(fence)
    }
}

#[cfg(feature = "sim-opengl-egl")]
fn egl_status_to_executor_error(status: i32) -> Accel2dExecutorError {
    match status {
        FHRE_EGL_UNAVAILABLE | FHRE_EGL_UNSUPPORTED => Accel2dExecutorError::Unsupported,
        FHRE_EGL_OVERFLOW => Accel2dExecutorError::Overflow,
        FHRE_EGL_FENCE_FAILED => Accel2dExecutorError::FenceFailed,
        _ => Accel2dExecutorError::SubmitFailed,
    }
}

#[cfg(feature = "sim-opengl-egl")]
const fn egl_pixel_format(format: PixelFormat) -> Option<u8> {
    match format {
        PixelFormat::Rgb565 => Some(FHRE_EGL_FORMAT_RGB565),
        PixelFormat::Rgb888 => Some(FHRE_EGL_FORMAT_RGB888),
        PixelFormat::Rgba8888 => Some(FHRE_EGL_FORMAT_RGBA8888),
        PixelFormat::Unknown(_) => None,
    }
}

#[cfg(feature = "sim-opengl-egl")]
const fn egl_image_format(format: ImageFormat) -> u8 {
    match format {
        ImageFormat::Rgb565 => FHRE_EGL_FORMAT_RGB565,
        ImageFormat::Rgb888 => FHRE_EGL_FORMAT_RGB888,
        ImageFormat::Rgba8888 => FHRE_EGL_FORMAT_RGBA8888,
        ImageFormat::A8 => FHRE_EGL_FORMAT_A8,
    }
}

#[cfg(feature = "sim-opengl-egl")]
fn egl_framebuffer(
    framebuffer: Accel2dFramebufferDescriptor,
) -> Result<FhreEglFramebuffer, Accel2dExecutorError> {
    let Some(format) = egl_pixel_format(framebuffer.format) else {
        return Err(Accel2dExecutorError::Unsupported);
    };
    if !framebuffer.is_valid() {
        return Err(Accel2dExecutorError::SubmitFailed);
    }
    Ok(FhreEglFramebuffer {
        data: framebuffer.data,
        len: framebuffer.len,
        width: framebuffer.width,
        height: framebuffer.height,
        stride: framebuffer.stride,
        bpp: framebuffer.bpp,
        format,
    })
}

#[cfg(feature = "sim-opengl-egl")]
fn egl_source(source: Accel2dImageSourceDescriptor) -> Result<FhreEglImageSource, Accel2dExecutorError> {
    if !source.is_valid() {
        return Err(Accel2dExecutorError::Unsupported);
    }
    Ok(FhreEglImageSource {
        data: source.data,
        len: source.len,
        width: source.width,
        height: source.height,
        stride: source.stride,
        format: egl_image_format(source.format),
    })
}

#[cfg(feature = "sim-opengl-egl")]
fn egl_fit(fit: ImageFit) -> Result<u8, Accel2dExecutorError> {
    match fit {
        ImageFit::Stretch => Ok(FHRE_EGL_FIT_STRETCH),
        ImageFit::Contain | ImageFit::Cover => Err(Accel2dExecutorError::Unsupported),
    }
}

#[cfg(feature = "sim-opengl-egl")]
fn egl_command(command: Accel2dCommand) -> Result<FhreEglCommand, Accel2dExecutorError> {
    match command {
        Accel2dCommand::SetClip { clip } => Ok(FhreEglCommand {
            kind: FHRE_EGL_COMMAND_SET_CLIP,
            has_clip: u8::from(clip.is_some()),
            clip: clip.map(FhreEglRect::from_rect).unwrap_or(FhreEglRect::EMPTY),
            ..FhreEglCommand::EMPTY
        }),
        Accel2dCommand::Fill { rect, clip, color } => Ok(FhreEglCommand {
            kind: FHRE_EGL_COMMAND_FILL,
            rect: FhreEglRect::from_rect(rect),
            has_clip: u8::from(clip.is_some()),
            clip: clip.map(FhreEglRect::from_rect).unwrap_or(FhreEglRect::EMPTY),
            color: FhreEglColor::from_color(color),
            ..FhreEglCommand::EMPTY
        }),
        Accel2dCommand::Image {
            rect,
            clip,
            source,
            opacity,
            fit,
            tint,
            ..
        } => Ok(FhreEglCommand {
            kind: FHRE_EGL_COMMAND_IMAGE,
            rect: FhreEglRect::from_rect(rect),
            has_clip: u8::from(clip.is_some()),
            clip: clip.map(FhreEglRect::from_rect).unwrap_or(FhreEglRect::EMPTY),
            source: egl_source(source)?,
            opacity,
            fit: egl_fit(fit)?,
            has_tint: u8::from(tint.is_some()),
            tint: tint
                .map(FhreEglColor::from_color)
                .unwrap_or(FhreEglColor::WHITE),
            ..FhreEglCommand::EMPTY
        }),
    }
}

#[cfg(feature = "sim-opengl-egl")]
fn egl_packet<const COMMANDS: usize>(
    list: &Accel2dCommandList<COMMANDS>,
    commands: &mut [FhreEglCommand; COMMANDS],
) -> Result<FhreEglPacket, Accel2dExecutorError> {
    if list.is_empty() {
        return Err(Accel2dExecutorError::SubmitFailed);
    }
    let framebuffer = egl_framebuffer(list.framebuffer())?;
    for (index, command) in list.commands().iter().copied().enumerate() {
        commands[index] = egl_command(command)?;
    }
    Ok(FhreEglPacket {
        framebuffer,
        run_bounds: FhreEglRect::from_rect(list.run().bounds),
        commands: commands.as_ptr(),
        command_count: list.len(),
    })
}

#[cfg(feature = "sim-opengl-egl")]
#[derive(Clone, Copy, Debug)]
pub struct EglParallelAccelBackend<const RUNS: usize, const OPS: usize> {
    surface: Surface,
    pending: Accel2dSubmissionRing<RUNS, OPS>,
    max_chain_ops: usize,
    queue_calls: u32,
    flush_calls: u32,
    last_error: Option<EglOpenGlWorkerError>,
    worker_stats: EglOpenGlWorkerStats,
}

#[cfg(feature = "sim-opengl-egl")]
impl<const RUNS: usize, const OPS: usize> EglParallelAccelBackend<RUNS, OPS> {
    pub const fn new(surface: Surface) -> Self {
        Self {
            surface,
            pending: Accel2dSubmissionRing::new(),
            max_chain_ops: OPS,
            queue_calls: 0,
            flush_calls: 0,
            last_error: None,
            worker_stats: EglOpenGlWorkerStats {
                queued: 0,
                submitted: 0,
                completed: 0,
                readbacks: 0,
                unsupported: 0,
                submit_failures: 0,
                fence_failures: 0,
                overflows: 0,
            },
        }
    }

    pub const fn with_max_chain_ops(mut self, max_chain_ops: usize) -> Self {
        self.max_chain_ops = max_chain_ops;
        self
    }

    pub const fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.surface
    }

    pub const fn into_surface(self) -> Surface {
        self.surface
    }

    pub const fn queue_calls(&self) -> u32 {
        self.queue_calls
    }

    pub const fn flush_calls(&self) -> u32 {
        self.flush_calls
    }

    pub const fn last_error(&self) -> Option<EglOpenGlWorkerError> {
        self.last_error
    }

    pub const fn worker_stats(&self) -> EglOpenGlWorkerStats {
        self.worker_stats
    }

    pub fn set_image_resolver(&mut self, resolver: Option<ImageResolver>) {
        self.surface.set_image_resolver(resolver);
    }

    pub fn worker_available() -> bool {
        unsafe { fhre_egl2d_available() != 0 }
    }

    pub fn worker_last_error() -> i32 {
        unsafe { fhre_egl2d_last_error() as i32 }
    }

    pub fn worker_pending_bounds() -> Option<Rect> {
        let mut rect = FhreEglRect::EMPTY;
        let status = unsafe { fhre_egl2d_pending_bounds(&mut rect) } as i32;
        if status == FHRE_EGL_OK && rect.w > 0 && rect.h > 0 {
            Some(rect.to_rect())
        } else {
            None
        }
    }

    pub fn shutdown_worker() {
        unsafe {
            let _ = fhre_egl2d_shutdown();
        }
    }

    fn refresh_worker_stats(&mut self) {
        let mut raw = FhreEglWorkerStatsRaw::default();
        if unsafe { fhre_egl2d_stats(&mut raw) } == FHRE_EGL_OK {
            self.worker_stats = raw.into();
        }
    }

    fn egl_capabilities(&self) -> BackendCapabilities {
        let mut capabilities = BackendCapabilities::software(self.surface.format);
        if Self::worker_available() {
            let accel_features = DrawFeatureFlags::FILL.union(DrawFeatureFlags::IMAGE);
            capabilities.accelerated_2d = true;
            capabilities.accelerated_draw_features = accel_features;
            capabilities.draw_chain = true;
            capabilities.max_chain_ops = self.max_chain_ops;
            capabilities.chain_draw_features = accel_features;
            capabilities.chain_mix_alpha = true;
            capabilities.chain_mix_clip = true;
            capabilities.chain_mix_mask = false;
            capabilities.chain_mix_layer = false;
            capabilities.chain_continuous_submit = true;
            capabilities.chain_run_fence = true;
        }
        capabilities
    }

    fn pending_command_bounds(&self) -> Option<Rect> {
        let mut bounds: Option<Rect> = None;
        for submission in self.pending.submissions().iter().copied() {
            if submission.state() == Accel2dFenceState::Completed {
                continue;
            }
            let run_bounds = submission.run().bounds;
            bounds = Some(match bounds {
                Some(current) => current.union(run_bounds),
                None => run_bounds,
            });
        }
        bounds
    }

    fn submit_fallback_from_executor_error(
        &mut self,
        stats: &mut RenderStats,
        error: Accel2dExecutorError,
    ) -> DrawChainSubmitResult {
        if matches!(
            error,
            Accel2dExecutorError::Overflow | Accel2dExecutorError::Ring(Accel2dRingError::Overflow)
        ) {
            stats.mark_accel2d_ring_overflow();
        }
        self.last_error = Some(EglOpenGlWorkerError::Executor(error));
        DrawChainSubmitResult::Fallback
    }

    fn submit_command_list_to_worker(
        &mut self,
        command_list: Accel2dCommandList<OPS>,
        stats: &mut RenderStats,
    ) -> Result<Accel2dFence, Accel2dExecutorError> {
        if self.pending.len() >= RUNS {
            self.last_error = Some(EglOpenGlWorkerError::Executor(Accel2dExecutorError::Overflow));
            return Err(Accel2dExecutorError::Overflow);
        }
        if !Self::worker_available() {
            self.last_error =
                Some(EglOpenGlWorkerError::Executor(Accel2dExecutorError::Unsupported));
            return Err(Accel2dExecutorError::Unsupported);
        }
        let init_status =
            unsafe { fhre_egl2d_init(self.surface.width(), self.surface.height()) } as i32;
        if init_status != FHRE_EGL_OK {
            let error = egl_status_to_executor_error(init_status);
            self.last_error = Some(EglOpenGlWorkerError::Worker(init_status));
            return Err(error);
        }

        let mut ffi_commands = [FhreEglCommand::EMPTY; OPS];
        let packet = match egl_packet(&command_list, &mut ffi_commands) {
            Ok(packet) => packet,
            Err(error) => {
                self.last_error = Some(EglOpenGlWorkerError::Executor(error));
                return Err(error);
            }
        };
        let mut egl_fence = FhreEglFence::default();
        let submit_status = unsafe { fhre_egl2d_submit(&packet, &mut egl_fence) } as i32;
        if submit_status != FHRE_EGL_OK || egl_fence.token == 0 {
            let error = egl_status_to_executor_error(submit_status);
            self.last_error = Some(EglOpenGlWorkerError::Worker(submit_status));
            return Err(error);
        }

        let fence = self
            .pending
            .queue(command_list)
            .map_err(Accel2dExecutorError::from_ring)?;
        stats.mark_accel2d_ring_submission();
        stats.mark_accel2d_fence_issued();
        self.refresh_worker_stats();
        self.last_error = None;
        Ok(fence)
    }

    fn complete_pending_after_worker_success(
        &mut self,
        stats: &mut RenderStats,
    ) -> Result<u32, Accel2dExecutorError> {
        let mut completed = 0u32;
        let mut index = 0usize;
        while index < self.pending.len() {
            let submission = self.pending.submissions()[index];
            if submission.state() == Accel2dFenceState::Submitted {
                self.pending
                    .complete_fence(submission.fence())
                    .map_err(Accel2dExecutorError::from_ring)?;
                stats.mark_accel2d_fence_completed();
                stats.mark_draw_chain_parallel_completed();
                completed = completed.saturating_add(1);
            }
            index += 1;
        }
        Ok(completed)
    }

    fn software_replay_pending(&mut self, stats: &mut RenderStats) -> u32 {
        let mut replayed = 0u32;
        let mut index = 0usize;
        while index < self.pending.len() {
            let submission = self.pending.submissions()[index];
            execute_accel_2d_command_list_on_surface(&mut self.surface, submission.command_list());
            stats.mark_draw_chain_parallel_fallback();
            replayed = replayed.saturating_add(1);
            index += 1;
        }
        self.pending.clear();
        replayed
    }
}

#[cfg(feature = "sim-opengl-egl")]
impl<const RUNS: usize, const OPS: usize> RenderBackend for EglParallelAccelBackend<RUNS, OPS> {
    fn capabilities(&self) -> BackendCapabilities {
        self.egl_capabilities()
    }

    fn submit_draw_chain_with_contract<const CHAIN_OPS: usize>(
        &mut self,
        chain: &DrawChain<CHAIN_OPS>,
        contract: &DrawChainRunContract<CHAIN_OPS>,
        stats: &mut RenderStats,
    ) -> DrawChainSubmitResult {
        if !self.pending.is_empty() {
            let flush_result = self.flush_pending_draw_chain(stats);
            if flush_result != DrawChainSubmitResult::Submitted {
                return flush_result;
            }
        }

        let command_list =
            match build_accel_2d_command_list::<CHAIN_OPS, OPS>(chain, contract, &self.surface) {
                Ok(command_list) => command_list,
                Err(error) => {
                    self.last_error = Some(EglOpenGlWorkerError::Packet(error));
                    return DrawChainSubmitResult::Fallback;
                }
            };

        match self.submit_command_list_to_worker(command_list, stats) {
            Ok(_) => self.flush_pending_draw_chain(stats),
            Err(error) => self.submit_fallback_from_executor_error(stats, error),
        }
    }

    fn set_clip(&mut self, clip: Option<Rect>) {
        self.surface.set_clip(clip);
    }

    fn draw_command(&mut self, cmd: DrawCommand) {
        cmd.execute(&mut self.surface);
    }

    fn push_mask(&mut self, spec: MaskSpec) -> bool {
        self.surface.push_mask(spec)
    }

    fn pop_mask(&mut self) -> bool {
        self.surface.pop_mask()
    }

    fn clear_masks(&mut self) {
        self.surface.clear_masks();
    }

    fn draw_layer_commands(
        &mut self,
        rect: Rect,
        spec: LayerSpec,
        cmds: &[DrawCommand],
        clips: &[Option<Rect>],
        stats: &mut RenderStats,
    ) -> bool {
        self.surface
            .draw_layer_commands_impl(rect, spec, cmds, clips, stats)
    }
}

#[cfg(feature = "sim-opengl-egl")]
impl<const RUNS: usize, const OPS: usize> ParallelRenderBackend
    for EglParallelAccelBackend<RUNS, OPS>
{
    fn queue_draw_chain_with_contract<const CHAIN_OPS: usize>(
        &mut self,
        chain: &DrawChain<CHAIN_OPS>,
        contract: &DrawChainRunContract<CHAIN_OPS>,
        stats: &mut RenderStats,
    ) -> ParallelDrawChainSubmitResult {
        self.queue_calls = self.queue_calls.saturating_add(1);
        let command_list =
            match build_accel_2d_command_list::<CHAIN_OPS, OPS>(chain, contract, &self.surface) {
                Ok(command_list) => command_list,
                Err(error) => {
                    self.last_error = Some(EglOpenGlWorkerError::Packet(error));
                    stats.mark_draw_chain_parallel_fallback();
                    return ParallelDrawChainSubmitResult::Fallback;
                }
            };

        match self.submit_command_list_to_worker(command_list, stats) {
            Ok(_) => {
                stats.mark_draw_chain_parallel_queued();
                ParallelDrawChainSubmitResult::Queued
            }
            Err(error) => {
                if error == Accel2dExecutorError::Overflow {
                    stats.mark_accel2d_ring_overflow();
                }
                stats.mark_draw_chain_parallel_fallback();
                ParallelDrawChainSubmitResult::Fallback
            }
        }
    }

    fn pending_draw_chain_bounds(&self) -> Option<Rect> {
        self.pending_command_bounds()
    }

    fn flush_pending_draw_chain(&mut self, stats: &mut RenderStats) -> DrawChainSubmitResult {
        if self.pending.is_empty() {
            return DrawChainSubmitResult::Unsupported;
        }
        self.flush_calls = self.flush_calls.saturating_add(1);
        match self.pending.flush() {
            Ok(_) => stats.mark_accel2d_ring_flush(),
            Err(error) => {
                self.last_error = Some(EglOpenGlWorkerError::Executor(
                    Accel2dExecutorError::from_ring(error),
                ));
                self.software_replay_pending(stats);
                return DrawChainSubmitResult::Fallback;
            }
        }

        let flush_status = unsafe { fhre_egl2d_flush() } as i32;
        self.refresh_worker_stats();
        if flush_status != FHRE_EGL_OK {
            self.last_error = Some(EglOpenGlWorkerError::Worker(flush_status));
            self.software_replay_pending(stats);
            return DrawChainSubmitResult::Fallback;
        }

        match self.complete_pending_after_worker_success(stats) {
            Ok(0) => {
                self.last_error = Some(EglOpenGlWorkerError::Executor(
                    Accel2dExecutorError::FenceFailed,
                ));
                self.pending.clear();
                stats.mark_draw_chain_parallel_fallback();
                DrawChainSubmitResult::Fallback
            }
            Ok(_) => {
                self.pending.clear();
                self.last_error = None;
                DrawChainSubmitResult::Submitted
            }
            Err(error) => {
                self.last_error = Some(EglOpenGlWorkerError::Executor(error));
                self.software_replay_pending(stats);
                DrawChainSubmitResult::Fallback
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        accel2d::{Accel2dCommand, Accel2dCommandList, Accel2dFramebufferDescriptor},
        image::{ImageFormat, ImageView},
        DrawChainRunDescriptor, PixelFormat,
    };
    #[cfg(feature = "sim-opengl-egl")]
    use crate::DrawList;

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

    #[test]
    fn sim_opengl_probe_maps_packet_and_matches_surface_pixels() {
        let mut expected_pixels = [0u8; 4 * 2 * 2];
        let mut actual_pixels = [0u8; 4 * 2 * 2];
        let mut expected = test_surface(&mut expected_pixels, 4, 2);
        let mut actual = test_surface(&mut actual_pixels, 4, 2);

        let view = ImageView::new(2, 2, 4, ImageFormat::Rgb565, &TEST_IMAGE_RGB565);
        expected.set_clip(Some(Rect::new(0, 0, 4, 2)));
        expected.fill_rect(Rect::new(0, 0, 4, 2), Color::rgb(8, 16, 24));
        expected.draw_image_view_fit(Rect::new(1, 0, 2, 2), view, 255, ImageFit::Stretch);

        let framebuffer = Accel2dFramebufferDescriptor::from_surface(&actual).unwrap();
        assert_eq!(framebuffer.format, PixelFormat::Rgb565);
        let mut list = Accel2dCommandList::<3>::new(framebuffer, DrawChainRunDescriptor::EMPTY);
        list.push(Accel2dCommand::SetClip {
            clip: Some(Rect::new(0, 0, 4, 2)),
        })
        .unwrap();
        list.push(Accel2dCommand::Fill {
            rect: Rect::new(0, 0, 4, 2),
            clip: Some(Rect::new(0, 0, 4, 2)),
            color: Color::rgb(8, 16, 24),
        })
        .unwrap();
        list.push(Accel2dCommand::Image {
            rect: Rect::new(1, 0, 2, 2),
            clip: Some(Rect::new(0, 0, 4, 2)),
            image: ImageId(7),
            source: Accel2dImageSourceDescriptor::from_view(view).unwrap(),
            opacity: 255,
            fit: ImageFit::Stretch,
            tint: None,
        })
        .unwrap();

        let mut executor = SimOpenGlAccel2dExecutor::<1, 3>::new(&mut actual);
        let fence = executor.queue(list).unwrap();
        assert_eq!(executor.flush(), Ok(1));
        assert_eq!(executor.execute_submitted(), Ok(1));
        assert_eq!(executor.complete_fence(fence), Ok(()));
        let stats = executor.stats();
        drop(executor);

        assert_eq!(stats.mapped_commands, 3);
        assert_eq!(stats.scissor_commands, 1);
        assert_eq!(stats.fill_quads, 1);
        assert_eq!(stats.texture_quads, 1);
        assert_eq!(expected_pixels, actual_pixels);
    }

    #[cfg(feature = "sim-opengl-egl")]
    #[test]
    fn egl_opengl_executor_reports_stable_unavailable_or_runs_probe_path() {
        let mut expected_pixels = [0u8; 4 * 2 * 2];
        let mut actual_pixels = [0u8; 4 * 2 * 2];
        let mut expected = test_surface(&mut expected_pixels, 4, 2);
        let mut actual = test_surface(&mut actual_pixels, 4, 2);

        expected.fill_rect(Rect::new(0, 0, 4, 2), Color::rgb(16, 24, 32));

        let framebuffer = Accel2dFramebufferDescriptor::from_surface(&actual).unwrap();
        let mut list = Accel2dCommandList::<1>::new(framebuffer, DrawChainRunDescriptor::EMPTY);
        list.push(Accel2dCommand::Fill {
            rect: Rect::new(0, 0, 4, 2),
            clip: None,
            color: Color::rgb(16, 24, 32),
        })
        .unwrap();

        let mut executor = EglOpenGlAccel2dExecutor::<1, 1>::new(&mut actual);
        if EglOpenGlAccel2dExecutor::<1, 1>::egl_available() {
            let fence = executor.queue(list).unwrap();
            assert_eq!(executor.flush(), Ok(1));
            assert_eq!(executor.execute_submitted(), Ok(1));
            assert_eq!(executor.complete_fence(fence), Ok(()));
            drop(executor);
            assert_eq!(expected_pixels, actual_pixels);
        } else {
            assert_eq!(executor.queue(list), Err(Accel2dExecutorError::Unsupported));
            assert_eq!(executor.stats().unsupported, 1);
            assert!(matches!(
                EglOpenGlAccel2dExecutor::<1, 1>::egl_last_error(),
                0 | FHRE_EGL_UNAVAILABLE
            ));
        }
    }

    #[cfg(feature = "sim-opengl-egl")]
    #[test]
    fn egl_parallel_backend_matches_software_with_worker_or_replay_path() {
        let mut list = DrawList::<4>::new();
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 3, 3),
            depth: 0,
            color: Color::rgb(40, 80, 120),
        }));
        assert!(list.push(DrawCommand::StrokeLine {
            from: crate::Point::new(7, 0),
            to: crate::Point::new(7, 2),
            depth: 0,
            width: 1,
            color: Color::WHITE,
        }));
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(4, 0, 3, 3),
            depth: 0,
            color: Color::rgb(220, 40, 20),
        }));

        let mut software_pixels = [0u8; 8 * 3 * 2];
        let mut parallel_pixels = [0u8; 8 * 3 * 2];
        let mut software = test_surface(&mut software_pixels, 8, 3);
        let parallel_surface = test_surface(&mut parallel_pixels, 8, 3);
        let mut parallel = EglParallelAccelBackend::<4, 1>::new(parallel_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_parallel_tracked_on(&mut parallel, &mut stats);

        assert_eq!(software_pixels, parallel_pixels);
        if EglParallelAccelBackend::<4, 1>::worker_available() {
            assert_eq!(parallel.queue_calls(), 2);
            assert!(stats.draw_chain_parallel_queued > 0);
            assert!(
                stats.draw_chain_parallel_completed > 0
                    || stats.draw_chain_parallel_fallbacks > 0
            );
            assert_eq!(stats.draw_chain_parallel_software_runs, 1);
        }
    }

    #[cfg(feature = "sim-opengl-egl")]
    #[test]
    fn egl_parallel_backend_ring_overflow_falls_back_without_pixel_drift() {
        if !EglParallelAccelBackend::<0, 1>::worker_available() {
            return;
        }

        let mut list = DrawList::<4>::new();
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 3, 2),
            depth: 0,
            color: Color::rgb(12, 34, 56),
        }));
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(4, 0, 3, 2),
            depth: 0,
            color: Color::rgb(180, 90, 20),
        }));

        let mut software_pixels = [0u8; 8 * 2 * 2];
        let mut parallel_pixels = [0u8; 8 * 2 * 2];
        let mut software = test_surface(&mut software_pixels, 8, 2);
        let parallel_surface = test_surface(&mut parallel_pixels, 8, 2);
        let mut parallel = EglParallelAccelBackend::<0, 1>::new(parallel_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_parallel_tracked_on(&mut parallel, &mut stats);

        assert_eq!(software_pixels, parallel_pixels);
        assert_eq!(parallel.queue_calls(), 2);
        assert!(stats.accel2d_ring_overflows > 0);
        assert!(stats.draw_chain_parallel_fallbacks > 0);
    }
}
