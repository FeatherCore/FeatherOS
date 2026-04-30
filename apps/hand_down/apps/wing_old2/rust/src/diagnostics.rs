use crate::render::{DirtyRegionSummary, DrawTaskSummary, RenderPlan};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDiagnostics {
    capacity: CapacityDiagnostics,
    frames: RuntimeFrameDiagnostics,
}

impl RuntimeDiagnostics {
    pub fn observe_frame(&mut self, frame_index: u32, snapshot: RuntimeFrameSnapshot) {
        self.frames.observe(frame_index, snapshot);
    }

    pub fn observe_capacity(&mut self, frame_index: u32, snapshot: CapacitySnapshot) {
        self.capacity.observe(frame_index, snapshot.overflow());
    }

    pub fn frames(&self) -> RuntimeFrameDiagnostics {
        self.frames
    }

    pub fn capacity(&self) -> CapacityDiagnostics {
        self.capacity
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeFrameDiagnostics {
    frames: u32,
    rendered_frames: u32,
    skipped_frames: u32,
    idle_frames: u32,
    dirty_frames: u32,
    input_frames: u32,
    surface_event_frames: u32,
    settings_event_frames: u32,
    task_exit_frames: u32,
    surface_events: u32,
    settings_events: u32,
    task_exits: u32,
    last_frame: Option<u32>,
    last_flags: RuntimeFrameFlags,
    last_dirty_region: DirtyRegionSummary,
    last_render_plan: RenderPlan,
}

impl RuntimeFrameDiagnostics {
    fn observe(&mut self, frame_index: u32, snapshot: RuntimeFrameSnapshot) {
        let flags = snapshot.flags();
        self.frames = self.frames.saturating_add(1);
        self.surface_events = self
            .surface_events
            .saturating_add(snapshot.surface_events as u32);
        self.settings_events = self
            .settings_events
            .saturating_add(snapshot.settings_events as u32);
        self.task_exits = self.task_exits.saturating_add(snapshot.task_exits as u32);
        self.last_frame = Some(frame_index);
        self.last_flags = flags;
        self.last_dirty_region = snapshot.dirty_region;
        self.last_render_plan = snapshot.render_plan;

        if flags.contains(RuntimeFrameFlags::RENDERED) {
            self.rendered_frames = self.rendered_frames.saturating_add(1);
        }
        if flags.contains(RuntimeFrameFlags::SKIPPED) {
            self.skipped_frames = self.skipped_frames.saturating_add(1);
        }
        if flags.contains(RuntimeFrameFlags::IDLE) {
            self.idle_frames = self.idle_frames.saturating_add(1);
        }
        if flags.contains(RuntimeFrameFlags::DIRTY) {
            self.dirty_frames = self.dirty_frames.saturating_add(1);
        }
        if flags.contains(RuntimeFrameFlags::INPUT) {
            self.input_frames = self.input_frames.saturating_add(1);
        }
        if flags.contains(RuntimeFrameFlags::SURFACE_EVENT) {
            self.surface_event_frames = self.surface_event_frames.saturating_add(1);
        }
        if flags.contains(RuntimeFrameFlags::SETTINGS_EVENT) {
            self.settings_event_frames = self.settings_event_frames.saturating_add(1);
        }
        if flags.contains(RuntimeFrameFlags::TASK_EXIT) {
            self.task_exit_frames = self.task_exit_frames.saturating_add(1);
        }
    }

    pub fn frames(self) -> u32 {
        self.frames
    }

    pub fn rendered_frames(self) -> u32 {
        self.rendered_frames
    }

    pub fn skipped_frames(self) -> u32 {
        self.skipped_frames
    }

    pub fn idle_frames(self) -> u32 {
        self.idle_frames
    }

    pub fn dirty_frames(self) -> u32 {
        self.dirty_frames
    }

    pub fn input_frames(self) -> u32 {
        self.input_frames
    }

    pub fn surface_event_frames(self) -> u32 {
        self.surface_event_frames
    }

    pub fn settings_event_frames(self) -> u32 {
        self.settings_event_frames
    }

    pub fn task_exit_frames(self) -> u32 {
        self.task_exit_frames
    }

    pub fn surface_events(self) -> u32 {
        self.surface_events
    }

    pub fn settings_events(self) -> u32 {
        self.settings_events
    }

    pub fn task_exits(self) -> u32 {
        self.task_exits
    }

    pub fn last_frame(self) -> Option<u32> {
        self.last_frame
    }

    pub fn last_flags(self) -> RuntimeFrameFlags {
        self.last_flags
    }

    pub fn last_draw_tasks(self) -> DrawTaskSummary {
        self.last_render_plan.tasks
    }

    pub fn last_dirty_region(self) -> DirtyRegionSummary {
        self.last_dirty_region
    }

    pub fn last_render_plan(self) -> RenderPlan {
        self.last_render_plan
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeFrameSnapshot {
    pub rendered: bool,
    pub dirty: bool,
    pub input: bool,
    pub surface_events: u8,
    pub settings_events: u8,
    pub task_exits: u8,
    pub dirty_region: DirtyRegionSummary,
    pub render_plan: RenderPlan,
}

impl RuntimeFrameSnapshot {
    pub fn flags(self) -> RuntimeFrameFlags {
        let mut flags = RuntimeFrameFlags::empty();
        if self.rendered {
            flags = flags.union(RuntimeFrameFlags::RENDERED);
        } else {
            flags = flags.union(RuntimeFrameFlags::SKIPPED);
        }
        if self.dirty {
            flags = flags.union(RuntimeFrameFlags::DIRTY);
        }
        if self.input {
            flags = flags.union(RuntimeFrameFlags::INPUT);
        }
        if self.surface_events != 0 {
            flags = flags.union(RuntimeFrameFlags::SURFACE_EVENT);
        }
        if self.settings_events != 0 {
            flags = flags.union(RuntimeFrameFlags::SETTINGS_EVENT);
        }
        if self.task_exits != 0 {
            flags = flags.union(RuntimeFrameFlags::TASK_EXIT);
        }
        if !self.rendered
            && !self.dirty
            && !self.input
            && self.surface_events == 0
            && self.settings_events == 0
            && self.task_exits == 0
        {
            flags = flags.union(RuntimeFrameFlags::IDLE);
        }
        flags
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeFrameFlags {
    bits: u16,
}

impl RuntimeFrameFlags {
    pub const RENDERED: Self = Self { bits: 1 << 0 };
    pub const SKIPPED: Self = Self { bits: 1 << 1 };
    pub const IDLE: Self = Self { bits: 1 << 2 };
    pub const DIRTY: Self = Self { bits: 1 << 3 };
    pub const INPUT: Self = Self { bits: 1 << 4 };
    pub const SURFACE_EVENT: Self = Self { bits: 1 << 5 };
    pub const SETTINGS_EVENT: Self = Self { bits: 1 << 6 };
    pub const TASK_EXIT: Self = Self { bits: 1 << 7 };

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self { bits }
    }

    pub const fn bits(self) -> u16 {
        self.bits
    }

    pub const fn any(self) -> bool {
        self.bits != 0
    }

    pub const fn contains(self, flag: Self) -> bool {
        (self.bits & flag.bits) != 0
    }

    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CapacityDiagnostics {
    current: CapacityOverflow,
    latched: CapacityOverflow,
    first_frame: Option<u32>,
    last_frame: Option<u32>,
    event_count: u32,
    last_event_frame: Option<u32>,
}

impl CapacityDiagnostics {
    fn observe(&mut self, frame_index: u32, overflow: CapacityOverflow) {
        self.current = overflow;
        if !overflow.any() {
            return;
        }

        self.latched = self.latched.union(overflow);
        if self.first_frame.is_none() {
            self.first_frame = Some(frame_index);
        }
        self.last_frame = Some(frame_index);

        if self.last_event_frame != Some(frame_index) {
            self.event_count = self.event_count.saturating_add(1);
            self.last_event_frame = Some(frame_index);
        }
    }

    pub fn current(&self) -> CapacityOverflow {
        self.current
    }

    pub fn latched(&self) -> CapacityOverflow {
        self.latched
    }

    pub fn first_frame(&self) -> Option<u32> {
        self.first_frame
    }

    pub fn last_frame(&self) -> Option<u32> {
        self.last_frame
    }

    pub fn event_count(&self) -> u32 {
        self.event_count
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CapacitySnapshot {
    pub ui_frame: bool,
    pub draw_list: bool,
    pub schedule: bool,
    pub app_registry: bool,
    pub surface_table: bool,
    pub texture_store: bool,
}

impl CapacitySnapshot {
    pub fn overflow(self) -> CapacityOverflow {
        let mut overflow = CapacityOverflow::empty();
        if self.ui_frame {
            overflow = overflow.union(CapacityOverflow::UI_FRAME);
        }
        if self.draw_list {
            overflow = overflow.union(CapacityOverflow::DRAW_LIST);
        }
        if self.schedule {
            overflow = overflow.union(CapacityOverflow::SCHEDULE);
        }
        if self.app_registry {
            overflow = overflow.union(CapacityOverflow::APP_REGISTRY);
        }
        if self.surface_table {
            overflow = overflow.union(CapacityOverflow::SURFACE_TABLE);
        }
        if self.texture_store {
            overflow = overflow.union(CapacityOverflow::TEXTURE_STORE);
        }
        overflow
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CapacityOverflow {
    bits: u16,
}

impl CapacityOverflow {
    pub const UI_FRAME: Self = Self { bits: 1 << 0 };
    pub const DRAW_LIST: Self = Self { bits: 1 << 1 };
    pub const SCHEDULE: Self = Self { bits: 1 << 2 };
    pub const APP_REGISTRY: Self = Self { bits: 1 << 3 };
    pub const SURFACE_TABLE: Self = Self { bits: 1 << 4 };
    pub const TEXTURE_STORE: Self = Self { bits: 1 << 5 };

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn from_bits(bits: u16) -> Self {
        Self { bits }
    }

    pub const fn bits(self) -> u16 {
        self.bits
    }

    pub const fn any(self) -> bool {
        self.bits != 0
    }

    pub const fn contains(self, flag: Self) -> bool {
        (self.bits & flag.bits) != 0
    }

    pub const fn union(self, other: Self) -> Self {
        Self {
            bits: self.bits | other.bits,
        }
    }
}
