use crate::{
    accel2d::{
        build_accel_2d_command_list, execute_accel_2d_command_list_on_surface,
        Accel2dCommandBuildError, Accel2dCommandList, Accel2dExecutor, Accel2dExecutorError,
        Accel2dFenceState, Accel2dRingError, Accel2dSubmissionRing, SurfaceAccel2dExecutor,
    },
    backend::{
        BackendCapabilities, DrawChain, DrawChainOpPayload, DrawChainRunContract,
        DrawChainSubmitResult, DrawFeatureFlags, ParallelDrawChainSubmitResult,
        ParallelRenderBackend, RenderBackend,
    },
    image::{builtin_image, ImageResolver, ImageView},
    Color, DrawCommand, ImageFit, ImageId, LayerSpec, MaskSpec, Rect, RenderStats, Surface,
};

const DEFAULT_MOCK_ACCEL_CHAIN_OPS: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawChainImageResolveError {
    NotImagePayload,
    MissingImage,
    EmptyImage,
    SourceOutOfBounds,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MockAccelSubmitError {
    EmptyChain,
    ContractMismatch,
    UnsupportedOp,
    PayloadMismatch,
    UnsupportedState,
    Image(DrawChainImageResolveError),
    Packet(Accel2dCommandBuildError),
    Executor(Accel2dExecutorError),
    Ring(Accel2dRingError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawChainImageDescriptor {
    pub rect: Rect,
    pub clip: Option<Rect>,
    pub image: ImageId,
    pub view: ImageView,
    pub opacity: u8,
    pub fit: ImageFit,
    pub tint: Option<Color>,
}

pub fn resolve_draw_chain_image_descriptor(
    payload: DrawChainOpPayload,
    clip: Option<Rect>,
    resolver: Option<ImageResolver>,
) -> Result<DrawChainImageDescriptor, DrawChainImageResolveError> {
    let DrawChainOpPayload::Image {
        rect,
        image,
        opacity,
        fit,
        tint,
    } = payload
    else {
        return Err(DrawChainImageResolveError::NotImagePayload);
    };

    let view = match builtin_image(image) {
        Some(view) => Some(view),
        None => resolver.and_then(|resolve| resolve(image)),
    }
    .ok_or(DrawChainImageResolveError::MissingImage)?;

    if view.data.is_null()
        || view.width == 0
        || view.height == 0
        || view.stride == 0
        || view.len == 0
    {
        return Err(DrawChainImageResolveError::EmptyImage);
    }

    Ok(DrawChainImageDescriptor {
        rect,
        clip,
        image,
        view,
        opacity,
        fit,
        tint,
    })
}

#[derive(Clone, Copy, Debug)]
pub struct MockAccelBackend {
    surface: Surface,
    max_chain_ops: usize,
    submit_calls: u32,
    last_error: Option<MockAccelSubmitError>,
}

impl MockAccelBackend {
    pub const fn new(surface: Surface) -> Self {
        Self {
            surface,
            max_chain_ops: DEFAULT_MOCK_ACCEL_CHAIN_OPS,
            submit_calls: 0,
            last_error: None,
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

    pub const fn submit_calls(&self) -> u32 {
        self.submit_calls
    }

    pub const fn last_error(&self) -> Option<MockAccelSubmitError> {
        self.last_error
    }

    pub fn set_image_resolver(&mut self, resolver: Option<ImageResolver>) {
        self.surface.set_image_resolver(resolver);
    }

    fn mock_capabilities(&self) -> BackendCapabilities {
        let mut capabilities = BackendCapabilities::software(self.surface.format);
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
        capabilities
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
        self.last_error = Some(MockAccelSubmitError::Executor(error));
        DrawChainSubmitResult::Fallback
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MockParallelAccelBackend<const RUNS: usize, const OPS: usize> {
    surface: Surface,
    pending: Accel2dSubmissionRing<RUNS, OPS>,
    max_chain_ops: usize,
    queue_calls: u32,
    flush_calls: u32,
    last_error: Option<MockAccelSubmitError>,
}

impl<const RUNS: usize, const OPS: usize> MockParallelAccelBackend<RUNS, OPS> {
    pub const fn new(surface: Surface) -> Self {
        Self {
            surface,
            pending: Accel2dSubmissionRing::new(),
            max_chain_ops: OPS,
            queue_calls: 0,
            flush_calls: 0,
            last_error: None,
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

    pub const fn last_error(&self) -> Option<MockAccelSubmitError> {
        self.last_error
    }

    pub fn set_image_resolver(&mut self, resolver: Option<ImageResolver>) {
        self.surface.set_image_resolver(resolver);
    }

    fn mock_capabilities(&self) -> BackendCapabilities {
        let mut capabilities = BackendCapabilities::software(self.surface.format);
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
        capabilities
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
        self.last_error = Some(MockAccelSubmitError::Executor(error));
        DrawChainSubmitResult::Fallback
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
}

impl<const RUNS: usize, const OPS: usize> RenderBackend for MockParallelAccelBackend<RUNS, OPS> {
    fn capabilities(&self) -> BackendCapabilities {
        self.mock_capabilities()
    }

    fn submit_draw_chain_with_contract<const CHAIN_OPS: usize>(
        &mut self,
        chain: &DrawChain<CHAIN_OPS>,
        contract: &DrawChainRunContract<CHAIN_OPS>,
        stats: &mut RenderStats,
    ) -> DrawChainSubmitResult {
        let command_list =
            match build_accel_2d_command_list::<CHAIN_OPS, OPS>(chain, contract, &self.surface) {
                Ok(command_list) => command_list,
                Err(error) => {
                    self.last_error = Some(MockAccelSubmitError::Packet(error));
                    return DrawChainSubmitResult::Fallback;
                }
            };

        let submit_result = {
            let mut executor = SurfaceAccel2dExecutor::<1, OPS>::new(&mut self.surface);
            let fence = match executor.queue(command_list) {
                Ok(fence) => {
                    stats.mark_accel2d_ring_submission();
                    stats.mark_accel2d_fence_issued();
                    fence
                }
                Err(error) => return self.submit_fallback_from_executor_error(stats, error),
            };

            if let Err(error) = executor.flush() {
                return self.submit_fallback_from_executor_error(stats, error);
            }
            stats.mark_accel2d_ring_flush();

            if let Err(error) = executor.execute_submitted() {
                return self.submit_fallback_from_executor_error(stats, error);
            }

            executor.complete_fence(fence)
        };

        match submit_result {
            Ok(()) => {
                stats.mark_accel2d_fence_completed();
                self.last_error = None;
                DrawChainSubmitResult::Submitted
            }
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

impl<const RUNS: usize, const OPS: usize> ParallelRenderBackend
    for MockParallelAccelBackend<RUNS, OPS>
{
    fn queue_draw_chain_with_contract<const CHAIN_OPS: usize>(
        &mut self,
        chain: &DrawChain<CHAIN_OPS>,
        contract: &DrawChainRunContract<CHAIN_OPS>,
        stats: &mut RenderStats,
    ) -> ParallelDrawChainSubmitResult {
        self.queue_calls = self.queue_calls.saturating_add(1);
        let command_list: Accel2dCommandList<OPS> =
            match build_accel_2d_command_list::<CHAIN_OPS, OPS>(chain, contract, &self.surface) {
                Ok(command_list) => command_list,
                Err(error) => {
                    self.last_error = Some(MockAccelSubmitError::Packet(error));
                    stats.mark_draw_chain_parallel_fallback();
                    return ParallelDrawChainSubmitResult::Fallback;
                }
            };

        match self.pending.queue(command_list) {
            Ok(_) => {
                stats.mark_accel2d_ring_submission();
                stats.mark_accel2d_fence_issued();
                stats.mark_draw_chain_parallel_queued();
                self.last_error = None;
                ParallelDrawChainSubmitResult::Queued
            }
            Err(error) => {
                if error == Accel2dRingError::Overflow {
                    stats.mark_accel2d_ring_overflow();
                }
                self.last_error = Some(MockAccelSubmitError::Ring(error));
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
                self.last_error = Some(MockAccelSubmitError::Ring(error));
                stats.mark_draw_chain_parallel_fallback();
                return DrawChainSubmitResult::Fallback;
            }
        }

        let mut executed = 0u32;
        let mut index = 0usize;
        while index < self.pending.len() {
            let submission = self.pending.submissions()[index];
            if submission.state() == Accel2dFenceState::Submitted {
                execute_accel_2d_command_list_on_surface(
                    &mut self.surface,
                    submission.command_list(),
                );
                match self.pending.complete_fence(submission.fence()) {
                    Ok(()) => {
                        stats.mark_accel2d_fence_completed();
                        stats.mark_draw_chain_parallel_completed();
                        executed = executed.saturating_add(1);
                    }
                    Err(error) => {
                        self.last_error = Some(MockAccelSubmitError::Ring(error));
                        stats.mark_draw_chain_parallel_fallback();
                        return DrawChainSubmitResult::Fallback;
                    }
                }
            }
            index += 1;
        }

        self.pending.clear();
        if executed == 0 {
            self.last_error = Some(MockAccelSubmitError::Ring(Accel2dRingError::NotSubmitted));
            stats.mark_draw_chain_parallel_fallback();
            return DrawChainSubmitResult::Fallback;
        }

        self.last_error = None;
        DrawChainSubmitResult::Submitted
    }
}

impl RenderBackend for MockAccelBackend {
    fn capabilities(&self) -> BackendCapabilities {
        self.mock_capabilities()
    }

    fn submit_draw_chain_with_contract<const OPS: usize>(
        &mut self,
        chain: &DrawChain<OPS>,
        contract: &DrawChainRunContract<OPS>,
        stats: &mut RenderStats,
    ) -> DrawChainSubmitResult {
        self.submit_calls = self.submit_calls.saturating_add(1);
        let command_list =
            match build_accel_2d_command_list::<OPS, OPS>(chain, contract, &self.surface) {
                Ok(command_list) => command_list,
                Err(error) => {
                    self.last_error = Some(MockAccelSubmitError::Packet(error));
                    return DrawChainSubmitResult::Fallback;
                }
            };

        let submit_result = (|| {
            let mut executor = SurfaceAccel2dExecutor::<1, OPS>::new(&mut self.surface);
            let fence = match executor.queue(command_list) {
                Ok(fence) => {
                    stats.mark_accel2d_ring_submission();
                    stats.mark_accel2d_fence_issued();
                    fence
                }
                Err(error) => return Err(error),
            };

            match executor.flush() {
                Ok(_) => stats.mark_accel2d_ring_flush(),
                Err(error) => return Err(error),
            }

            if let Err(error) = executor.execute_submitted() {
                return Err(error);
            }

            executor.complete_fence(fence)
        })();

        match submit_result {
            Ok(()) => {
                stats.mark_accel2d_fence_completed();
                self.last_error = None;
                DrawChainSubmitResult::Submitted
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Color, DrawCommand, DrawList, Fixed16, ImageId, MaskSpec, PixelFormat, Rect, RenderBackend,
        RenderStats,
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
                crate::ImageFormat::Rgb565,
                &TEST_IMAGE_RGB565,
            ))
        } else {
            None
        }
    }

    #[test]
    fn mock_accel_fill_alpha_and_clip_match_software_surface() {
        let mut list = DrawList::<8>::new();
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 8, 8),
            depth: 0,
            color: Color::rgb(12, 28, 44),
        }));
        assert!(list.push_clipped(
            DrawCommand::FillRect {
                rect: Rect::new(2, 2, 5, 5),
                depth: 0,
                color: Color::rgba(220, 40, 20, 128),
            },
            Some(Rect::new(3, 1, 3, 4)),
        ));

        let mut software_pixels = [0u8; 8 * 8 * 2];
        let mut mock_pixels = [0u8; 8 * 8 * 2];
        let mut software = test_surface(&mut software_pixels, 8, 8);
        let mock_surface = test_surface(&mut mock_pixels, 8, 8);
        let mut mock = MockAccelBackend::new(mock_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_tracked_on(&mut mock, &mut stats);

        assert_eq!(software_pixels, mock_pixels);
        assert_eq!(stats.draw_chain_hw_runs, 2);
        assert_eq!(stats.draw_chain_sw_runs, 0);
        assert_eq!(stats.commands_seen, 0);
        assert_eq!(stats.accel2d_ring_submissions, 2);
        assert_eq!(stats.accel2d_ring_flushes, 2);
        assert_eq!(stats.accel2d_fences_issued, 2);
        assert_eq!(stats.accel2d_fences_completed, 2);
        assert_eq!(stats.accel2d_ring_overflows, 0);
    }

    #[test]
    fn mock_accel_image_descriptor_resolves_builtin_and_resolver_images() {
        let mut list = DrawList::<8>::new();
        assert!(list.push(DrawCommand::DrawImageFit {
            rect: Rect::new(0, 0, 4, 4),
            depth: 0,
            image: crate::IMAGE_SWATCH,
            opacity: 255,
            fit: crate::ImageFit::Stretch,
        }));
        assert!(list.push(DrawCommand::DrawImageFit {
            rect: Rect::new(4, 0, 4, 4),
            depth: 0,
            image: ImageId(77),
            opacity: 160,
            fit: crate::ImageFit::Stretch,
        }));

        let mut software_pixels = [0u8; 8 * 4 * 2];
        let mut mock_pixels = [0u8; 8 * 4 * 2];
        let mut software = test_surface(&mut software_pixels, 8, 4);
        let mut mock_surface = test_surface(&mut mock_pixels, 8, 4);
        software.set_image_resolver(Some(test_image_resolver));
        mock_surface.set_image_resolver(Some(test_image_resolver));
        let mut mock = MockAccelBackend::new(mock_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_tracked_on(&mut mock, &mut stats);

        assert_eq!(software_pixels, mock_pixels);
        assert_eq!(stats.draw_chain_hw_runs, 1);
        assert_eq!(mock.last_error(), None);
    }

    #[test]
    fn mock_accel_missing_image_fallback_does_not_poison_pixels() {
        let mut list = DrawList::<8>::new();
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 8, 4),
            depth: 0,
            color: Color::rgb(8, 16, 24),
        }));
        assert!(list.push(DrawCommand::DrawImageFit {
            rect: Rect::new(2, 0, 4, 4),
            depth: 0,
            image: ImageId(222),
            opacity: 255,
            fit: crate::ImageFit::Stretch,
        }));
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(6, 0, 2, 4),
            depth: 0,
            color: Color::rgb(180, 20, 80),
        }));

        let mut software_pixels = [0u8; 8 * 4 * 2];
        let mut mock_pixels = [0u8; 8 * 4 * 2];
        let mut software = test_surface(&mut software_pixels, 8, 4);
        let mock_surface = test_surface(&mut mock_pixels, 8, 4);
        let mut mock = MockAccelBackend::new(mock_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_tracked_on(&mut mock, &mut stats);

        assert_eq!(software_pixels, mock_pixels);
        assert_eq!(stats.draw_chain_hw_runs, 2);
        assert_eq!(stats.draw_chain_sw_runs, 1);
        assert_eq!(stats.commands_seen, 1);
        assert_eq!(stats.accel2d_ring_submissions, 2);
        assert_eq!(stats.accel2d_fences_completed, 2);
        assert_eq!(mock.submit_calls(), 4);
    }

    #[test]
    fn mock_accel_mask_run_falls_back_to_software() {
        let mut list = DrawList::<8>::new();
        assert!(list.push(DrawCommand::PushMask {
            depth: 0,
            spec: MaskSpec::rect(Rect::new(1, 1, 3, 3)),
        }));
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 5, 5),
            depth: Fixed16::from(1),
            color: Color::rgb(250, 250, 250),
        }));
        assert!(list.push(DrawCommand::PopMask));

        let mut software_pixels = [0u8; 5 * 5 * 2];
        let mut mock_pixels = [0u8; 5 * 5 * 2];
        let mut software = test_surface(&mut software_pixels, 5, 5);
        let mock_surface = test_surface(&mut mock_pixels, 5, 5);
        let mut mock = MockAccelBackend::new(mock_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_tracked_on(&mut mock, &mut stats);

        assert_eq!(software_pixels, mock_pixels);
        assert_eq!(stats.draw_chain_hw_runs, 0);
        assert!(stats.draw_chain_fallbacks > 0);
        assert_eq!(stats.accel2d_ring_submissions, 0);
        assert!(stats.commands_seen > 0);
    }

    #[test]
    fn image_descriptor_reports_missing_before_submit_writes() {
        let payload = DrawChainOpPayload::Image {
            rect: Rect::new(0, 0, 4, 4),
            image: ImageId(999),
            opacity: 255,
            fit: crate::ImageFit::Stretch,
            tint: None,
        };

        assert_eq!(
            resolve_draw_chain_image_descriptor(payload, None, None),
            Err(DrawChainImageResolveError::MissingImage)
        );
    }

    #[test]
    fn mock_accel_capabilities_are_chain_limited_to_fill_image_clip() {
        let mut pixels = [0u8; 2 * 2 * 2];
        let mock = MockAccelBackend::new(test_surface(&mut pixels, 2, 2));
        let caps = mock.capabilities();

        assert!(caps.draw_chain);
        assert!(caps.accelerated_2d);
        assert!(caps.chain_continuous_submit);
        assert!(caps.chain_run_fence);
        assert_eq!(caps.pixel_format, PixelFormat::Rgb565);
        assert!(caps.chain_draw_features.supports(crate::DrawTaskKind::Fill));
        assert!(caps
            .chain_draw_features
            .supports(crate::DrawTaskKind::Image));
        assert!(!caps
            .chain_draw_features
            .supports(crate::DrawTaskKind::MaskRect));
        assert!(!caps
            .chain_draw_features
            .supports(crate::DrawTaskKind::Layer));
    }

    #[test]
    fn mock_accel_ring_preserves_multiple_packet_order() {
        fn assert_ordered_pixels(list: DrawList<4>, width: u16, height: u16) -> RenderStats {
            let mut software_pixels = [0u8; 8 * 4 * 2];
            let mut mock_pixels = [0u8; 8 * 4 * 2];
            let mut software = test_surface(&mut software_pixels, width, height);
            let mock_surface = test_surface(&mut mock_pixels, width, height);
            let mut mock = MockAccelBackend::new(mock_surface).with_max_chain_ops(1);
            let mut stats = RenderStats::new();

            list.execute_on(&mut software);
            list.execute_tracked_on(&mut mock, &mut stats);

            assert_eq!(software_pixels, mock_pixels);
            stats
        }

        let mut non_overlapping = DrawList::<4>::new();
        assert!(non_overlapping.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 2, 4),
            depth: 0,
            color: Color::rgb(180, 10, 10),
        }));
        assert!(non_overlapping.push(DrawCommand::FillRect {
            rect: Rect::new(2, 0, 2, 4),
            depth: 0,
            color: Color::rgb(10, 180, 10),
        }));
        let stats = assert_ordered_pixels(non_overlapping, 4, 4);
        assert_eq!(stats.draw_chain_hw_runs, 2);
        assert_eq!(stats.accel2d_ring_submissions, 2);
        assert_eq!(stats.accel2d_ring_flushes, 2);
        assert_eq!(stats.accel2d_fences_issued, 2);
        assert_eq!(stats.accel2d_fences_completed, 2);
        assert!(stats.draw_chain_parallel_hints > 0);

        let mut overlapping = DrawList::<4>::new();
        assert!(overlapping.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 4, 4),
            depth: 0,
            color: Color::rgb(0, 0, 200),
        }));
        assert!(overlapping.push(DrawCommand::FillRect {
            rect: Rect::new(1, 1, 2, 2),
            depth: 0,
            color: Color::rgb(240, 220, 0),
        }));
        let stats = assert_ordered_pixels(overlapping, 4, 4);
        assert_eq!(stats.draw_chain_hw_runs, 2);
        assert_eq!(stats.accel2d_ring_submissions, 2);
        assert_eq!(stats.accel2d_fences_completed, 2);
    }

    #[test]
    fn mock_parallel_accel_queues_disjoint_runs_and_matches_software() {
        let mut list = DrawList::<4>::new();
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 3, 3),
            depth: 0,
            color: Color::rgb(30, 50, 70),
        }));
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(5, 0, 3, 3),
            depth: 0,
            color: Color::rgb(120, 30, 10),
        }));

        let mut software_pixels = [0u8; 8 * 3 * 2];
        let mut parallel_pixels = [0u8; 8 * 3 * 2];
        let mut software = test_surface(&mut software_pixels, 8, 3);
        let parallel_surface = test_surface(&mut parallel_pixels, 8, 3);
        let mut parallel = MockParallelAccelBackend::<4, 1>::new(parallel_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_parallel_tracked_on(&mut parallel, &mut stats);

        assert_eq!(software_pixels, parallel_pixels);
        assert_eq!(parallel.queue_calls(), 2);
        assert_eq!(parallel.flush_calls(), 1);
        assert_eq!(stats.draw_chain_parallel_queued, 2);
        assert_eq!(stats.draw_chain_parallel_completed, 2);
        assert_eq!(stats.draw_chain_parallel_barriers, 0);
        assert_eq!(stats.draw_chain_hw_runs, 2);
    }

    #[test]
    fn mock_parallel_accel_allows_disjoint_software_while_pending() {
        let mut list = DrawList::<4>::new();
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 3, 4),
            depth: 0,
            color: Color::rgb(40, 80, 120),
        }));
        assert!(list.push(DrawCommand::StrokeLine {
            from: crate::Point::new(6, 0),
            to: crate::Point::new(7, 3),
            depth: 0,
            width: 1,
            color: Color::WHITE,
        }));

        let mut software_pixels = [0u8; 8 * 4 * 2];
        let mut parallel_pixels = [0u8; 8 * 4 * 2];
        let mut software = test_surface(&mut software_pixels, 8, 4);
        let parallel_surface = test_surface(&mut parallel_pixels, 8, 4);
        let mut parallel = MockParallelAccelBackend::<4, 1>::new(parallel_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_parallel_tracked_on(&mut parallel, &mut stats);

        assert_eq!(software_pixels, parallel_pixels);
        assert_eq!(stats.draw_chain_parallel_queued, 1);
        assert_eq!(stats.draw_chain_parallel_completed, 1);
        assert_eq!(stats.draw_chain_parallel_software_runs, 1);
        assert_eq!(stats.draw_chain_parallel_barriers, 0);
        assert!(stats.draw_chain_sw_runs > 0);
    }

    #[test]
    fn mock_parallel_accel_barriers_on_overlap() {
        let mut list = DrawList::<4>::new();
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 5, 4),
            depth: 0,
            color: Color::rgb(10, 20, 30),
        }));
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(2, 0, 5, 4),
            depth: 0,
            color: Color::rgba(240, 30, 10, 180),
        }));

        let mut software_pixels = [0u8; 8 * 4 * 2];
        let mut parallel_pixels = [0u8; 8 * 4 * 2];
        let mut software = test_surface(&mut software_pixels, 8, 4);
        let parallel_surface = test_surface(&mut parallel_pixels, 8, 4);
        let mut parallel = MockParallelAccelBackend::<4, 1>::new(parallel_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_parallel_tracked_on(&mut parallel, &mut stats);

        assert_eq!(software_pixels, parallel_pixels);
        assert_eq!(stats.draw_chain_parallel_queued, 2);
        assert_eq!(stats.draw_chain_parallel_completed, 2);
        assert_eq!(stats.draw_chain_parallel_barriers, 1);
        assert_eq!(stats.draw_chain_parallel_software_runs, 0);
    }

    #[test]
    fn mock_parallel_accel_barriers_before_stateful_mask_run() {
        let mut list = DrawList::<8>::new();
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(0, 0, 2, 4),
            depth: 0,
            color: Color::rgb(20, 40, 60),
        }));
        assert!(list.push(DrawCommand::PushMask {
            depth: 0,
            spec: MaskSpec::rect(Rect::new(5, 0, 2, 4)),
        }));
        assert!(list.push(DrawCommand::FillRect {
            rect: Rect::new(4, 0, 4, 4),
            depth: Fixed16::from(1),
            color: Color::rgb(220, 220, 220),
        }));
        assert!(list.push(DrawCommand::PopMask));

        let mut software_pixels = [0u8; 8 * 4 * 2];
        let mut parallel_pixels = [0u8; 8 * 4 * 2];
        let mut software = test_surface(&mut software_pixels, 8, 4);
        let parallel_surface = test_surface(&mut parallel_pixels, 8, 4);
        let mut parallel = MockParallelAccelBackend::<4, 1>::new(parallel_surface);
        let mut stats = RenderStats::new();

        list.execute_on(&mut software);
        list.execute_parallel_tracked_on(&mut parallel, &mut stats);

        assert_eq!(software_pixels, parallel_pixels);
        assert_eq!(stats.draw_chain_parallel_queued, 1);
        assert_eq!(stats.draw_chain_parallel_completed, 1);
        assert!(stats.draw_chain_parallel_barriers > 0);
        assert!(stats.draw_chain_sw_runs > 0);
    }
}
