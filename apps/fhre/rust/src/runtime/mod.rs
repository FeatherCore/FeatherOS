use crate::{Camera, DirtyRegion, EntityWorld, InputEvent, InputQueue};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FramePolicy {
    pub target_frame_us: u32,
}

impl FramePolicy {
    pub const SIXTY_FPS: Self = Self {
        target_frame_us: 16_666,
    };

    pub const fn new(target_frame_us: u32) -> Self {
        Self { target_frame_us }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentStats {
    pub presents: u32,
    pub pan_presents: u32,
    pub copy_presents: u32,
    pub skipped_presents: u32,
    pub bytes_copied: u32,
}

impl PresentStats {
    pub const fn new() -> Self {
        Self {
            presents: 0,
            pan_presents: 0,
            copy_presents: 0,
            skipped_presents: 0,
            bytes_copied: 0,
        }
    }

    pub fn pan() -> Self {
        let mut stats = Self::new();
        stats.presents = 1;
        stats.pan_presents = 1;
        stats
    }

    pub fn copied(bytes: usize) -> Self {
        let mut stats = Self::new();
        stats.presents = 1;
        stats.copy_presents = 1;
        stats.bytes_copied = bytes.min(u32::MAX as usize) as u32;
        stats
    }

    pub fn skipped() -> Self {
        let mut stats = Self::new();
        stats.skipped_presents = 1;
        stats
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameStats {
    pub frame: u32,
    pub target_frame_us: u32,
    pub draw_us: u32,
    pub present_us: u32,
    pub frame_work_us: u32,
    pub sleep_us: u32,
    pub input_events: u32,
    pub dropped_frames: u32,
    pub late_frames: u32,
    pub fps_x1000: u32,
    pub present: PresentStats,
}

impl FrameStats {
    pub const fn new(policy: FramePolicy) -> Self {
        Self {
            frame: 0,
            target_frame_us: policy.target_frame_us,
            draw_us: 0,
            present_us: 0,
            frame_work_us: 0,
            sleep_us: 0,
            input_events: 0,
            dropped_frames: 0,
            late_frames: 0,
            fps_x1000: 0,
            present: PresentStats::new(),
        }
    }
}

pub struct FrameClock {
    policy: FramePolicy,
    frame: u32,
    dropped_frames: u32,
    late_frames: u32,
}

impl FrameClock {
    pub const fn new(policy: FramePolicy) -> Self {
        Self {
            policy,
            frame: 0,
            dropped_frames: 0,
            late_frames: 0,
        }
    }

    pub const fn policy(&self) -> FramePolicy {
        self.policy
    }

    pub const fn frame(&self) -> u32 {
        self.frame
    }

    pub fn finish_frame(
        &mut self,
        draw_us: u32,
        present_us: u32,
        input_events: u32,
        present: PresentStats,
    ) -> FrameStats {
        let target = self.policy.target_frame_us.max(1);
        let work = draw_us.saturating_add(present_us);
        let sleep_us = target.saturating_sub(work);
        if work > target {
            self.late_frames = self.late_frames.saturating_add(1);
            self.dropped_frames = self
                .dropped_frames
                .saturating_add(work.saturating_sub(1) / target);
        }

        let mut stats = FrameStats::new(self.policy);
        stats.frame = self.frame;
        stats.draw_us = draw_us;
        stats.present_us = present_us;
        stats.frame_work_us = work;
        stats.sleep_us = sleep_us;
        stats.input_events = input_events;
        stats.dropped_frames = self.dropped_frames;
        stats.late_frames = self.late_frames;
        stats.fps_x1000 = 1_000_000_000u32 / work.max(target);
        stats.present = present;

        self.frame = self.frame.wrapping_add(1);
        stats
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Time {
    pub delta_us: u32,
    pub uptime_us: u64,
    pub frame: u32,
}

impl Time {
    pub const fn new() -> Self {
        Self {
            delta_us: 0,
            uptime_us: 0,
            frame: 0,
        }
    }

    pub fn tick(&mut self, delta_us: u32) {
        self.delta_us = delta_us;
        self.uptime_us = self.uptime_us.saturating_add(delta_us as u64);
        self.frame = self.frame.wrapping_add(1);
    }
}

pub struct RenderContext<const DIRTY: usize> {
    pub camera: Camera,
    pub time: Time,
    pub dirty: DirtyRegion<DIRTY>,
}

impl<const DIRTY: usize> RenderContext<DIRTY> {
    pub const fn new(camera: Camera) -> Self {
        Self {
            camera,
            time: Time::new(),
            dirty: DirtyRegion::new(),
        }
    }
}

pub struct FhreRuntime<const INPUT: usize, const DIRTY: usize> {
    pub render: RenderContext<DIRTY>,
    pub input: InputQueue<INPUT>,
}

impl<const INPUT: usize, const DIRTY: usize> FhreRuntime<INPUT, DIRTY> {
    pub const fn new(camera: Camera) -> Self {
        Self {
            render: RenderContext::new(camera),
            input: InputQueue::new(),
        }
    }

    pub fn tick(&mut self, delta_us: u32) {
        self.render.time.tick(delta_us);
    }

    pub fn push_input(&mut self, event: InputEvent) -> bool {
        self.input.push(event)
    }

    pub fn pop_input(&mut self) -> Option<InputEvent> {
        self.input.pop()
    }
}

pub struct GameRuntime<const ENTITIES: usize, const INPUT: usize, const DIRTY: usize> {
    pub entities: EntityWorld<ENTITIES>,
    pub render: RenderContext<DIRTY>,
    pub input: InputQueue<INPUT>,
}

impl<const ENTITIES: usize, const INPUT: usize, const DIRTY: usize>
    GameRuntime<ENTITIES, INPUT, DIRTY>
{
    pub const fn new(camera: Camera) -> Self {
        Self {
            entities: EntityWorld::new(),
            render: RenderContext::new(camera),
            input: InputQueue::new(),
        }
    }

    pub fn tick(&mut self, delta_us: u32) {
        self.render.time.tick(delta_us);
    }

    pub fn push_input(&mut self, event: InputEvent) -> bool {
        self.input.push(event)
    }

    pub fn pop_input(&mut self) -> Option<InputEvent> {
        self.input.pop()
    }
}
