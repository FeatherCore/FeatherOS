use ::core::ffi::c_char;

use crate::app_sdk::{
    WingDirtyRect, WingPointerEvent, WingPointerKind, WingSurface, WingSurfaceError,
    WingSurfaceFormat,
};
use crate::core::FixedList;
use crate::math::{Color, Point, Rect};
use crate::platform::nuttx::{
    load_runtime_default_app_svg_store, load_runtime_default_font_store,
};
use crate::render::{
    font_cell_advance, font_line_height, font_text_bounds, vector_icon_svg_id, FontStore,
    SvgRasterMask, SvgStore, VectorIcon,
};

pub const APP_UI_DEFAULT_CAPACITY: usize = 48;
pub const APP_UI_EVENT_CAPACITY: usize = 16;
pub const APP_UI_COMMAND_CAPACITY: usize = 16;
pub const APP_UI_DIRTY_RECT_CAPACITY: usize = 4;
pub const APP_UI_DEFAULT_IDLE_US: u32 = 200_000;

extern "C" {
    fn usleep(usec: u32) -> i32;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiKey(pub u16);

impl AppUiKey {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn child(self, offset: u16) -> Self {
        Self(self.0.wrapping_add(offset))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiSignalId(pub u16);

impl AppUiSignalId {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn from_key(key: AppUiKey) -> Self {
        Self(key.0)
    }

    pub const fn raw(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiSignal {
    id: AppUiSignalId,
    value: i32,
}

impl AppUiSignal {
    pub const fn new(id: AppUiSignalId, value: i32) -> Self {
        Self { id, value }
    }

    pub const fn id(self) -> AppUiSignalId {
        self.id
    }

    pub const fn value(self) -> i32 {
        self.value
    }

    pub const fn is(self, id: AppUiSignalId) -> bool {
        self.id.0 == id.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppVisual {
    None,
    Rect { color: Color },
    RoundRect { color: Color, radius: u8 },
    Circle { color: Color },
    VectorIcon { icon: VectorIcon, color: Color },
    Text { text: &'static str, color: Color, scale: u8 },
}

impl AppVisual {
    fn bounds(self, rect: Rect) -> Rect {
        match self {
            Self::None => Rect::new(rect.x, rect.y, 0, 0),
            Self::Text { text, scale, .. } => {
                font_text_bounds(rect.x, rect.y, text, scale.max(1))
            }
            _ => rect,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiSpec {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub visual: AppVisual,
    pub clickable: bool,
}

impl AppUiSpec {
    fn bounds(self) -> Rect {
        self.visual.bounds(self.rect)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppUiSwipe {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppUiGesture {
    Press { key: AppUiKey, point: Point },
    Click { key: AppUiKey, point: Point },
    Drag {
        key: AppUiKey,
        start: Point,
        point: Point,
        delta: Point,
    },
    Swipe {
        key: AppUiKey,
        direction: AppUiSwipe,
        start: Point,
        end: Point,
    },
    Release { key: AppUiKey, point: Point },
    Cancel { key: AppUiKey },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppUiFlow {
    #[default]
    Continue,
    Exit,
}

impl AppUiFlow {
    pub const fn should_continue(self) -> bool {
        matches!(self, Self::Continue)
    }

    pub const fn should_exit(self) -> bool {
        matches!(self, Self::Exit)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiTick {
    frame: u16,
}

impl AppUiTick {
    pub const fn new() -> Self {
        Self { frame: 0 }
    }

    pub const fn frame(self) -> u16 {
        self.frame
    }

    pub fn advance(&mut self) -> u16 {
        self.frame = self.frame.wrapping_add(1);
        self.frame
    }

    pub fn step_countdown(&mut self, value: &mut u16) -> u16 {
        self.advance();
        *value = value.saturating_sub(1);
        *value
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiFramePacing {
    frame_us: u32,
}

impl AppUiFramePacing {
    pub const fn from_hz(hz: u16) -> Self {
        let hz = if hz == 0 { 1 } else { hz as u32 };
        Self {
            frame_us: 1_000_000 / hz,
        }
    }

    pub const fn from_micros(frame_us: u32) -> Self {
        Self { frame_us }
    }

    pub const fn frame_us(self) -> u32 {
        self.frame_us
    }
}

impl Default for AppUiFramePacing {
    fn default() -> Self {
        Self::from_hz(30)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiLoop {
    tick: AppUiTick,
    pacing: AppUiFramePacing,
    max_input_events: u8,
    frame_stats: AppUiFrameStats,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiContext {
    width: u16,
    height: u16,
    tick: u16,
}

impl AppUiContext {
    pub const fn new(width: u16, height: u16, tick: u16) -> Self {
        Self {
            width,
            height,
            tick,
        }
    }

    pub const fn width(self) -> u16 {
        self.width
    }

    pub const fn height(self) -> u16 {
        self.height
    }

    pub const fn tick(self) -> u16 {
        self.tick
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiFrameResult {
    pub flow: AppUiFlow,
    pub rendered: bool,
    pub idle: bool,
    pub input_count: u8,
    pub command_count: u8,
    pub dirty_rects: u8,
    pub budget: AppUiFrameBudget,
    pub sleep_us: u32,
    pub input_overflowed: bool,
    pub command_overflowed: bool,
    pub dirty_overflowed: bool,
}

impl AppUiFrameResult {
    pub const fn should_continue(self) -> bool {
        self.flow.should_continue()
    }

    pub const fn is_idle(self) -> bool {
        self.idle
    }

    pub const fn has_input(self) -> bool {
        self.input_count != 0
    }

    pub const fn has_commands(self) -> bool {
        self.command_count != 0
    }

    pub fn flags(self) -> AppUiFrameFlags {
        let mut flags = AppUiFrameFlags::empty();
        if self.rendered {
            flags = flags.union(AppUiFrameFlags::RENDERED);
        }
        if self.idle {
            flags = flags.union(AppUiFrameFlags::IDLE);
        }
        if self.input_count != 0 {
            flags = flags.union(AppUiFrameFlags::INPUT);
        }
        if self.command_count != 0 {
            flags = flags.union(AppUiFrameFlags::COMMANDS);
        }
        if self.input_overflowed {
            flags = flags.union(AppUiFrameFlags::INPUT_OVERFLOW);
        }
        if self.command_overflowed {
            flags = flags.union(AppUiFrameFlags::COMMAND_OVERFLOW);
        }
        if self.dirty_overflowed {
            flags = flags.union(AppUiFrameFlags::DIRTY_OVERFLOW);
        }
        if self.budget.spec_overflowed {
            flags = flags.union(AppUiFrameFlags::SPEC_OVERFLOW);
        }
        if self.budget.previous_spec_overflowed {
            flags = flags.union(AppUiFrameFlags::PREVIOUS_SPEC_OVERFLOW);
        }
        if self.flow.should_exit() {
            flags = flags.union(AppUiFrameFlags::EXIT);
        }
        flags
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiRenderResult {
    pub rendered: bool,
    pub dirty_rects: u8,
    pub dirty_overflowed: bool,
}

impl AppUiRenderResult {
    pub const fn clean() -> Self {
        Self {
            rendered: false,
            dirty_rects: 0,
            dirty_overflowed: false,
        }
    }

    pub const fn rendered(dirty_rects: u8, dirty_overflowed: bool) -> Self {
        Self {
            rendered: dirty_rects != 0,
            dirty_rects,
            dirty_overflowed,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiFrameBudget {
    pub spec_count: u16,
    pub spec_capacity: u16,
    pub previous_spec_count: u16,
    pub event_count: u8,
    pub event_capacity: u8,
    pub command_count: u8,
    pub command_capacity: u8,
    pub dirty_rects: u8,
    pub dirty_rect_capacity: u8,
    pub spec_overflowed: bool,
    pub previous_spec_overflowed: bool,
    pub event_overflowed: bool,
    pub command_overflowed: bool,
    pub dirty_overflowed: bool,
}

impl AppUiFrameBudget {
    pub const fn empty() -> Self {
        Self {
            spec_count: 0,
            spec_capacity: 0,
            previous_spec_count: 0,
            event_count: 0,
            event_capacity: 0,
            command_count: 0,
            command_capacity: 0,
            dirty_rects: 0,
            dirty_rect_capacity: 0,
            spec_overflowed: false,
            previous_spec_overflowed: false,
            event_overflowed: false,
            command_overflowed: false,
            dirty_overflowed: false,
        }
    }

    pub const fn any_overflow(self) -> bool {
        self.spec_overflowed
            || self.previous_spec_overflowed
            || self.event_overflowed
            || self.command_overflowed
            || self.dirty_overflowed
    }

    pub const fn spec_remaining(self) -> u16 {
        if self.spec_count >= self.spec_capacity {
            0
        } else {
            self.spec_capacity - self.spec_count
        }
    }

    pub const fn event_remaining(self) -> u8 {
        if self.event_count >= self.event_capacity {
            0
        } else {
            self.event_capacity - self.event_count
        }
    }

    pub const fn command_remaining(self) -> u8 {
        if self.command_count >= self.command_capacity {
            0
        } else {
            self.command_capacity - self.command_count
        }
    }

    pub const fn dirty_rect_remaining(self) -> u8 {
        if self.dirty_rects >= self.dirty_rect_capacity {
            0
        } else {
            self.dirty_rect_capacity - self.dirty_rects
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiDiagnosticsSummary {
    pub label: &'static str,
    pub pressure: u8,
}

impl AppUiDiagnosticsSummary {
    pub const fn new(label: &'static str, pressure: u8) -> Self {
        Self { label, pressure }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiFrameStats {
    frames: u32,
    rendered_frames: u32,
    idle_frames: u32,
    input_frames: u32,
    command_frames: u32,
    exit_frames: u32,
    input_overflows: u32,
    command_overflows: u32,
    dirty_overflows: u32,
    spec_overflows: u32,
    previous_spec_overflows: u32,
    input_events: u32,
    commands: u32,
    dirty_rects: u32,
    last_flags: AppUiFrameFlags,
    last_budget: AppUiFrameBudget,
    last_sleep_us: u32,
}

impl AppUiFrameStats {
    pub const fn new() -> Self {
        Self {
            frames: 0,
            rendered_frames: 0,
            idle_frames: 0,
            input_frames: 0,
            command_frames: 0,
            exit_frames: 0,
            input_overflows: 0,
            command_overflows: 0,
            dirty_overflows: 0,
            spec_overflows: 0,
            previous_spec_overflows: 0,
            input_events: 0,
            commands: 0,
            dirty_rects: 0,
            last_flags: AppUiFrameFlags::empty(),
            last_budget: AppUiFrameBudget::empty(),
            last_sleep_us: 0,
        }
    }

    pub fn observe(&mut self, frame: AppUiFrameResult) {
        self.frames = self.frames.saturating_add(1);
        self.input_events = self.input_events.saturating_add(frame.input_count as u32);
        self.commands = self.commands.saturating_add(frame.command_count as u32);
        self.dirty_rects = self.dirty_rects.saturating_add(frame.dirty_rects as u32);
        self.last_flags = frame.flags();
        self.last_budget = frame.budget;
        self.last_sleep_us = frame.sleep_us;

        if frame.rendered {
            self.rendered_frames = self.rendered_frames.saturating_add(1);
        }
        if frame.idle {
            self.idle_frames = self.idle_frames.saturating_add(1);
        }
        if frame.has_input() {
            self.input_frames = self.input_frames.saturating_add(1);
        }
        if frame.has_commands() {
            self.command_frames = self.command_frames.saturating_add(1);
        }
        if frame.flow.should_exit() {
            self.exit_frames = self.exit_frames.saturating_add(1);
        }
        if frame.input_overflowed {
            self.input_overflows = self.input_overflows.saturating_add(1);
        }
        if frame.command_overflowed {
            self.command_overflows = self.command_overflows.saturating_add(1);
        }
        if frame.dirty_overflowed {
            self.dirty_overflows = self.dirty_overflows.saturating_add(1);
        }
        if frame.budget.spec_overflowed {
            self.spec_overflows = self.spec_overflows.saturating_add(1);
        }
        if frame.budget.previous_spec_overflowed {
            self.previous_spec_overflows = self.previous_spec_overflows.saturating_add(1);
        }
    }

    pub const fn frames(self) -> u32 {
        self.frames
    }

    pub const fn rendered_frames(self) -> u32 {
        self.rendered_frames
    }

    pub const fn idle_frames(self) -> u32 {
        self.idle_frames
    }

    pub const fn input_frames(self) -> u32 {
        self.input_frames
    }

    pub const fn command_frames(self) -> u32 {
        self.command_frames
    }

    pub const fn exit_frames(self) -> u32 {
        self.exit_frames
    }

    pub const fn input_overflows(self) -> u32 {
        self.input_overflows
    }

    pub const fn command_overflows(self) -> u32 {
        self.command_overflows
    }

    pub const fn dirty_overflows(self) -> u32 {
        self.dirty_overflows
    }

    pub const fn spec_overflows(self) -> u32 {
        self.spec_overflows
    }

    pub const fn previous_spec_overflows(self) -> u32 {
        self.previous_spec_overflows
    }

    pub const fn input_events(self) -> u32 {
        self.input_events
    }

    pub const fn commands(self) -> u32 {
        self.commands
    }

    pub const fn dirty_rects(self) -> u32 {
        self.dirty_rects
    }

    pub const fn last_flags(self) -> AppUiFrameFlags {
        self.last_flags
    }

    pub const fn last_budget(self) -> AppUiFrameBudget {
        self.last_budget
    }

    pub const fn last_sleep_us(self) -> u32 {
        self.last_sleep_us
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiFrameFlags {
    bits: u16,
}

impl AppUiFrameFlags {
    pub const RENDERED: Self = Self { bits: 1 << 0 };
    pub const IDLE: Self = Self { bits: 1 << 1 };
    pub const INPUT: Self = Self { bits: 1 << 2 };
    pub const COMMANDS: Self = Self { bits: 1 << 3 };
    pub const INPUT_OVERFLOW: Self = Self { bits: 1 << 4 };
    pub const COMMAND_OVERFLOW: Self = Self { bits: 1 << 5 };
    pub const DIRTY_OVERFLOW: Self = Self { bits: 1 << 6 };
    pub const SPEC_OVERFLOW: Self = Self { bits: 1 << 7 };
    pub const PREVIOUS_SPEC_OVERFLOW: Self = Self { bits: 1 << 8 };
    pub const EXIT: Self = Self { bits: 1 << 9 };

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

pub fn app_ui_budget_label(budget: AppUiFrameBudget) -> &'static str {
    if budget.spec_overflowed || budget.previous_spec_overflowed {
        "SPEC FULL"
    } else if budget.event_overflowed {
        "EVENT FULL"
    } else if budget.command_overflowed {
        "CMD FULL"
    } else if budget.dirty_overflowed {
        "DIRTY FULL"
    } else if budget.spec_capacity != 0 && budget.spec_remaining() <= 4 {
        "SPEC HIGH"
    } else if budget.dirty_rect_capacity != 0 && budget.dirty_rect_remaining() == 0 {
        "DIRTY HIGH"
    } else if budget.dirty_rects > 1 {
        "DIRTY MULTI"
    } else if budget.event_count != 0 {
        "INPUT"
    } else if budget.command_count != 0 {
        "COMMAND"
    } else {
        "OK"
    }
}

pub fn app_ui_budget_pressure(budget: AppUiFrameBudget) -> u8 {
    let spec = app_ui_usage_pressure(budget.spec_count as u32, budget.spec_capacity as u32);
    let events = app_ui_usage_pressure(budget.event_count as u32, budget.event_capacity as u32);
    let commands = app_ui_usage_pressure(budget.command_count as u32, budget.command_capacity as u32);
    let dirty = app_ui_usage_pressure(budget.dirty_rects as u32, budget.dirty_rect_capacity as u32);

    spec.max(events).max(commands).max(dirty)
}

pub fn app_ui_budget_summary(budget: AppUiFrameBudget) -> AppUiDiagnosticsSummary {
    AppUiDiagnosticsSummary::new(app_ui_budget_label(budget), app_ui_budget_pressure(budget))
}

pub fn app_ui_frame_flags_label(flags: AppUiFrameFlags) -> &'static str {
    if flags.contains(AppUiFrameFlags::EXIT) {
        "EXIT"
    } else if flags.contains(AppUiFrameFlags::SPEC_OVERFLOW) {
        "SPEC FULL"
    } else if flags.contains(AppUiFrameFlags::PREVIOUS_SPEC_OVERFLOW) {
        "SPEC PREV"
    } else if flags.contains(AppUiFrameFlags::COMMAND_OVERFLOW) {
        "CMD FULL"
    } else if flags.contains(AppUiFrameFlags::INPUT_OVERFLOW) {
        "INPUT FULL"
    } else if flags.contains(AppUiFrameFlags::DIRTY_OVERFLOW) {
        "DIRTY FULL"
    } else if flags.contains(AppUiFrameFlags::COMMANDS) {
        "COMMAND"
    } else if flags.contains(AppUiFrameFlags::INPUT) {
        "INPUT"
    } else if flags.contains(AppUiFrameFlags::RENDERED) {
        "DRAW"
    } else if flags.contains(AppUiFrameFlags::IDLE) {
        "IDLE"
    } else {
        "SKIP"
    }
}

pub fn app_ui_frame_result_label(result: AppUiFrameResult) -> &'static str {
    app_ui_frame_flags_label(result.flags())
}

fn app_ui_usage_pressure(used: u32, capacity: u32) -> u8 {
    if capacity == 0 {
        0
    } else {
        ((used.min(capacity) * u8::MAX as u32) / capacity) as u8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiFrameHint {
    active: bool,
    idle_sleep_us: u32,
}

impl AppUiFrameHint {
    pub const fn active() -> Self {
        Self {
            active: true,
            idle_sleep_us: 0,
        }
    }

    pub const fn idle(idle_sleep_us: u32) -> Self {
        Self {
            active: false,
            idle_sleep_us,
        }
    }

    pub const fn idle_default() -> Self {
        Self::idle(APP_UI_DEFAULT_IDLE_US)
    }

    pub const fn is_active(self) -> bool {
        self.active
    }

    pub const fn idle_sleep_us(self, frame_us: u32) -> u32 {
        if self.idle_sleep_us == 0 {
            frame_us
        } else {
            self.idle_sleep_us
        }
    }
}

impl Default for AppUiFrameHint {
    fn default() -> Self {
        Self::active()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiUpdateResult {
    pub flow: AppUiFlow,
    pub command_count: u8,
    pub command_overflowed: bool,
}

impl AppUiUpdateResult {
    pub const fn new(flow: AppUiFlow, command_count: u8, command_overflowed: bool) -> Self {
        Self {
            flow,
            command_count,
            command_overflowed,
        }
    }

    pub const fn should_exit(self) -> bool {
        self.flow.should_exit()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppUiCommand {
    Gesture(AppUiGesture),
    Signal(AppUiSignal),
    Prepare,
    Tick,
}

impl AppUiCommand {
    pub const fn gesture(gesture: AppUiGesture) -> Self {
        Self::Gesture(gesture)
    }

    pub const fn signal(id: AppUiSignalId, value: i32) -> Self {
        Self::Signal(AppUiSignal::new(id, value))
    }

    pub const fn signal_key(key: AppUiKey, value: i32) -> Self {
        Self::Signal(AppUiSignal::new(AppUiSignalId::from_key(key), value))
    }

    pub const fn prepare() -> Self {
        Self::Prepare
    }
}

pub trait AppUiSystem<A> {
    fn run(&mut self, app: &mut A, context: AppUiContext, command: AppUiCommand) -> AppUiFlow;
}

impl<A, F> AppUiSystem<A> for F
where
    F: FnMut(&mut A, AppUiContext, AppUiCommand) -> AppUiFlow,
{
    fn run(&mut self, app: &mut A, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        self(app, context, command)
    }
}

impl<A> AppUiSystem<A> for () {
    fn run(&mut self, _app: &mut A, _context: AppUiContext, _command: AppUiCommand) -> AppUiFlow {
        AppUiFlow::Continue
    }
}

impl<A, S0> AppUiSystem<A> for (S0,)
where
    S0: AppUiSystem<A>,
{
    fn run(&mut self, app: &mut A, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        self.0.run(app, context, command)
    }
}

impl<A, S0, S1> AppUiSystem<A> for (S0, S1)
where
    S0: AppUiSystem<A>,
    S1: AppUiSystem<A>,
{
    fn run(&mut self, app: &mut A, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        let flow = self.0.run(app, context, command);
        if flow.should_exit() {
            return flow;
        }

        self.1.run(app, context, command)
    }
}

impl<A, S0, S1, S2> AppUiSystem<A> for (S0, S1, S2)
where
    S0: AppUiSystem<A>,
    S1: AppUiSystem<A>,
    S2: AppUiSystem<A>,
{
    fn run(&mut self, app: &mut A, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        let flow = self.0.run(app, context, command);
        if flow.should_exit() {
            return flow;
        }

        let flow = self.1.run(app, context, command);
        if flow.should_exit() {
            return flow;
        }

        self.2.run(app, context, command)
    }
}

impl<A, S0, S1, S2, S3> AppUiSystem<A> for (S0, S1, S2, S3)
where
    S0: AppUiSystem<A>,
    S1: AppUiSystem<A>,
    S2: AppUiSystem<A>,
    S3: AppUiSystem<A>,
{
    fn run(&mut self, app: &mut A, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        let flow = self.0.run(app, context, command);
        if flow.should_exit() {
            return flow;
        }

        let flow = self.1.run(app, context, command);
        if flow.should_exit() {
            return flow;
        }

        let flow = self.2.run(app, context, command);
        if flow.should_exit() {
            return flow;
        }

        self.3.run(app, context, command)
    }
}

pub struct AppUiApplyCommand<const N: usize>;

impl<const N: usize, A> AppUiSystem<A> for AppUiApplyCommand<N>
where
    A: AppUiApp<N>,
{
    fn run(&mut self, app: &mut A, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        app.apply_command(context, command)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiSchedule<S> {
    systems: S,
}

impl<S> AppUiSchedule<S> {
    pub const fn new(systems: S) -> Self {
        Self { systems }
    }

    pub fn systems_mut(&mut self) -> &mut S {
        &mut self.systems
    }

    pub fn into_inner(self) -> S {
        self.systems
    }

    pub fn run_command<A>(
        &mut self,
        app: &mut A,
        context: AppUiContext,
        command: AppUiCommand,
    ) -> AppUiFlow
    where
        S: AppUiSystem<A>,
    {
        self.systems.run(app, context, command)
    }

    pub fn run_tick<A>(&mut self, app: &mut A, context: AppUiContext) -> AppUiFlow
    where
        S: AppUiSystem<A>,
    {
        self.run_command(app, context, AppUiCommand::Tick)
    }

    pub fn run_queue<A, const N: usize>(
        &mut self,
        app: &mut A,
        context: AppUiContext,
        commands: &AppUiCommandQueue<N>,
    ) -> AppUiFlow
    where
        S: AppUiSystem<A>,
    {
        commands.run_systems(app, context, &mut self.systems)
    }
}

impl<A, S> AppUiSystem<A> for AppUiSchedule<S>
where
    S: AppUiSystem<A>,
{
    fn run(&mut self, app: &mut A, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        self.systems.run(app, context, command)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiStagedSchedule<Input, Tick = (), Prepare = ()> {
    input: Input,
    tick: Tick,
    prepare: Prepare,
}

impl<Input, Tick> AppUiStagedSchedule<Input, Tick, ()> {
    pub const fn new(input: Input, tick: Tick) -> Self {
        Self {
            input,
            tick,
            prepare: (),
        }
    }
}

impl<Input, Tick, Prepare> AppUiStagedSchedule<Input, Tick, Prepare> {
    pub const fn with_prepare(input: Input, tick: Tick, prepare: Prepare) -> Self {
        Self {
            input,
            tick,
            prepare,
        }
    }

    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    pub fn tick_mut(&mut self) -> &mut Tick {
        &mut self.tick
    }

    pub fn prepare_mut(&mut self) -> &mut Prepare {
        &mut self.prepare
    }

    pub fn into_inner(self) -> (Input, Tick, Prepare) {
        (self.input, self.tick, self.prepare)
    }

    pub fn run_input<A>(
        &mut self,
        app: &mut A,
        context: AppUiContext,
        command: AppUiCommand,
    ) -> AppUiFlow
    where
        Input: AppUiSystem<A>,
    {
        self.input.run(app, context, command)
    }

    pub fn run_tick<A>(&mut self, app: &mut A, context: AppUiContext) -> AppUiFlow
    where
        Tick: AppUiSystem<A>,
    {
        self.tick.run(app, context, AppUiCommand::Tick)
    }

    pub fn run_prepare<A>(&mut self, app: &mut A, context: AppUiContext) -> AppUiFlow
    where
        Prepare: AppUiSystem<A>,
    {
        self.prepare.run(app, context, AppUiCommand::Prepare)
    }
}

impl<A, Input, Tick, Prepare> AppUiSystem<A> for AppUiStagedSchedule<Input, Tick, Prepare>
where
    Input: AppUiSystem<A>,
    Tick: AppUiSystem<A>,
    Prepare: AppUiSystem<A>,
{
    fn run(&mut self, app: &mut A, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        match command {
            AppUiCommand::Tick => self.tick.run(app, context, command),
            AppUiCommand::Prepare => self.prepare.run(app, context, command),
            _ => self.input.run(app, context, command),
        }
    }
}

pub trait AppUiApp<const N: usize> {
    fn update(&mut self, context: AppUiContext, gesture: AppUiGesture) -> AppUiFlow;

    fn enqueue_event(
        &mut self,
        _context: AppUiContext,
        gesture: AppUiGesture,
        commands: &mut AppUiCommandQueue,
    ) -> AppUiFlow {
        commands.push(AppUiCommand::gesture(gesture));
        AppUiFlow::Continue
    }

    fn apply_command(&mut self, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        match command {
            AppUiCommand::Gesture(gesture) => self.update(context, gesture),
            AppUiCommand::Signal(_) | AppUiCommand::Prepare | AppUiCommand::Tick => {
                AppUiFlow::Continue
            }
        }
    }

    fn update_events<const M: usize>(
        &mut self,
        context: AppUiContext,
        events: &AppUiEventQueue<M>,
    ) -> AppUiUpdateResult
    where
        Self: Sized,
    {
        let mut commands = AppUiCommandQueue::new();

        for gesture in events.as_slice() {
            let flow = self.enqueue_event(context, *gesture, &mut commands);
            if flow.should_exit() {
                return AppUiUpdateResult::new(
                    flow,
                    saturating_usize_to_u8(commands.len()),
                    commands.overflowed(),
                );
            }

            if commands.overflowed() {
                break;
            }
        }

        let mut system = AppUiApplyCommand::<N>;
        let flow = commands.run_systems(self, context, &mut system);
        AppUiUpdateResult::new(
            flow,
            saturating_usize_to_u8(commands.len()),
            commands.overflowed(),
        )
    }

    fn view(&self, context: AppUiContext, frame: &mut AppUiBuilder<'_, N>);

    fn frame_hint(&self, _context: AppUiContext) -> AppUiFrameHint {
        AppUiFrameHint::active()
    }

    fn before_frame(&mut self, _context: AppUiContext) {}

    fn after_frame(&mut self, _context: AppUiContext) {}

    fn observe_frame(&mut self, _context: AppUiContext, _result: AppUiFrameResult) {}
}

pub trait AppUiScheduledApp<const N: usize>: Sized {
    type Schedule: AppUiSystem<Self>;

    fn schedule(&mut self) -> Self::Schedule;

    fn enqueue_scheduled_event(
        &mut self,
        _context: AppUiContext,
        gesture: AppUiGesture,
        commands: &mut AppUiCommandQueue,
    ) -> AppUiFlow {
        commands.gesture(gesture);
        AppUiFlow::Continue
    }

    fn view_scheduled(&self, context: AppUiContext, frame: &mut AppUiBuilder<'_, N>);

    fn frame_hint_scheduled(&self, _context: AppUiContext) -> AppUiFrameHint {
        AppUiFrameHint::active()
    }

    fn before_scheduled_frame(&mut self, context: AppUiContext) {
        let mut schedule = self.schedule();
        let _ = schedule.run(self, context, AppUiCommand::Prepare);
    }

    fn after_scheduled_frame(&mut self, context: AppUiContext) {
        let mut schedule = self.schedule();
        let _ = schedule.run(self, context, AppUiCommand::Tick);
    }

    fn observe_scheduled_frame(&mut self, _context: AppUiContext, _result: AppUiFrameResult) {}

    fn update_scheduled(&mut self, context: AppUiContext, gesture: AppUiGesture) -> AppUiFlow {
        let mut commands = AppUiCommandQueue::new();
        let flow = self.enqueue_scheduled_event(context, gesture, &mut commands);
        if flow.should_exit() {
            return flow;
        }

        let mut schedule = self.schedule();
        commands.run_systems(self, context, &mut schedule)
    }

    fn update_scheduled_events<const M: usize>(
        &mut self,
        context: AppUiContext,
        events: &AppUiEventQueue<M>,
    ) -> AppUiUpdateResult {
        let mut commands = AppUiCommandQueue::new();

        for gesture in events.as_slice() {
            let flow = self.enqueue_scheduled_event(context, *gesture, &mut commands);
            if flow.should_exit() {
                return AppUiUpdateResult::new(
                    flow,
                    saturating_usize_to_u8(commands.len()),
                    commands.overflowed(),
                );
            }

            if commands.overflowed() {
                break;
            }
        }

        let mut schedule = self.schedule();
        let flow = commands.run_systems(self, context, &mut schedule);
        AppUiUpdateResult::new(
            flow,
            saturating_usize_to_u8(commands.len()),
            commands.overflowed(),
        )
    }
}

impl<const N: usize, A> AppUiApp<N> for A
where
    A: AppUiScheduledApp<N>,
{
    fn update(&mut self, context: AppUiContext, gesture: AppUiGesture) -> AppUiFlow {
        self.update_scheduled(context, gesture)
    }

    fn enqueue_event(
        &mut self,
        context: AppUiContext,
        gesture: AppUiGesture,
        commands: &mut AppUiCommandQueue,
    ) -> AppUiFlow {
        self.enqueue_scheduled_event(context, gesture, commands)
    }

    fn apply_command(&mut self, context: AppUiContext, command: AppUiCommand) -> AppUiFlow {
        let mut schedule = self.schedule();
        schedule.run(self, context, command)
    }

    fn update_events<const M: usize>(
        &mut self,
        context: AppUiContext,
        events: &AppUiEventQueue<M>,
    ) -> AppUiUpdateResult {
        self.update_scheduled_events(context, events)
    }

    fn view(&self, context: AppUiContext, frame: &mut AppUiBuilder<'_, N>) {
        self.view_scheduled(context, frame);
    }

    fn frame_hint(&self, context: AppUiContext) -> AppUiFrameHint {
        self.frame_hint_scheduled(context)
    }

    fn before_frame(&mut self, context: AppUiContext) {
        self.before_scheduled_frame(context);
    }

    fn after_frame(&mut self, context: AppUiContext) {
        self.after_scheduled_frame(context);
    }

    fn observe_frame(&mut self, context: AppUiContext, result: AppUiFrameResult) {
        self.observe_scheduled_frame(context, result);
    }
}

pub struct AppUiEventQueue<const N: usize = APP_UI_EVENT_CAPACITY> {
    events: FixedList<AppUiGesture, N>,
}

impl<const N: usize> AppUiEventQueue<N> {
    pub fn new() -> Self {
        Self {
            events: FixedList::new(),
        }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn push(&mut self, gesture: AppUiGesture) -> bool {
        self.events.push(gesture)
    }

    pub fn as_slice(&self) -> &[AppUiGesture] {
        self.events.as_slice()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.len() == 0
    }

    pub fn overflowed(&self) -> bool {
        self.events.overflowed()
    }
}

impl<const N: usize> Default for AppUiEventQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AppUiCommandQueue<const N: usize = APP_UI_COMMAND_CAPACITY> {
    commands: FixedList<AppUiCommand, N>,
}

impl<const N: usize> AppUiCommandQueue<N> {
    pub fn new() -> Self {
        Self {
            commands: FixedList::new(),
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn push(&mut self, command: AppUiCommand) -> bool {
        self.commands.push(command)
    }

    pub fn gesture(&mut self, gesture: AppUiGesture) -> bool {
        self.push(AppUiCommand::gesture(gesture))
    }

    pub fn signal(&mut self, id: AppUiSignalId, value: i32) -> bool {
        self.push(AppUiCommand::signal(id, value))
    }

    pub fn signal_key(&mut self, key: AppUiKey, value: i32) -> bool {
        self.push(AppUiCommand::signal_key(key, value))
    }

    pub fn tick(&mut self) -> bool {
        self.push(AppUiCommand::Tick)
    }

    pub fn run_systems<A, S>(
        &self,
        app: &mut A,
        context: AppUiContext,
        systems: &mut S,
    ) -> AppUiFlow
    where
        S: AppUiSystem<A>,
    {
        for command in self.as_slice() {
            let flow = systems.run(app, context, *command);
            if flow.should_exit() {
                return flow;
            }
        }

        AppUiFlow::Continue
    }

    pub fn as_slice(&self) -> &[AppUiCommand] {
        self.commands.as_slice()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.len() == 0
    }

    pub fn overflowed(&self) -> bool {
        self.commands.overflowed()
    }
}

impl<const N: usize> Default for AppUiCommandQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn run_app_ui_rgb565<const N: usize, A>(
    surface: &mut WingSurface,
    background: Color,
    pacing: AppUiFramePacing,
    app: &mut A,
) -> Result<(), WingSurfaceError>
where
    A: AppUiApp<N>,
{
    if surface.format() != WingSurfaceFormat::Rgb565 {
        return Err(WingSurfaceError::UnsupportedFormat);
    }

    let mut ui = AppUiRuntime::<N>::new(background);
    run_app_ui_rgb565_loop(surface, pacing, &mut ui, app)
}

pub fn run_app_ui_rgb565_with_svg_store<const N: usize, A>(
    surface: &mut WingSurface,
    background: Color,
    pacing: AppUiFramePacing,
    svgs: SvgStore,
    app: &mut A,
) -> Result<(), WingSurfaceError>
where
    A: AppUiApp<N>,
{
    if surface.format() != WingSurfaceFormat::Rgb565 {
        return Err(WingSurfaceError::UnsupportedFormat);
    }

    let mut ui = AppUiRuntime::<N>::new_with_svg_store(background, svgs);
    run_app_ui_rgb565_loop(surface, pacing, &mut ui, app)
}

fn run_app_ui_rgb565_loop<const N: usize, A>(
    surface: &mut WingSurface,
    pacing: AppUiFramePacing,
    ui: &mut AppUiRuntime<N>,
    app: &mut A,
) -> Result<(), WingSurfaceError>
where
    A: AppUiApp<N>,
{
    let mut app_loop = AppUiLoop::new(pacing);

    loop {
        let frame = app_loop.drive_rgb565(ui, surface, app)?;
        if !frame.should_continue() {
            return Ok(());
        }

        unsafe {
            let _ = usleep(frame.sleep_us);
        }
    }
}

pub unsafe fn run_app_ui_rgb565_from_argv_with_svg_store<const N: usize, A>(
    argc: i32,
    argv: *mut *mut c_char,
    background: Color,
    pacing: AppUiFramePacing,
    svgs: SvgStore,
    app: &mut A,
) -> i32
where
    A: AppUiApp<N>,
{
    let mut surface = match WingSurface::open_from_argv_mut(argc, argv) {
        Ok(surface) => surface,
        Err(error) => return error.exit_code(),
    };

    match run_app_ui_rgb565_with_svg_store::<N, _>(&mut surface, background, pacing, svgs, app) {
        Ok(()) => 0,
        Err(error) => error.exit_code(),
    }
}

pub unsafe fn run_app_ui_rgb565_from_argv<const N: usize, A>(
    argc: i32,
    argv: *mut *mut c_char,
    background: Color,
    pacing: AppUiFramePacing,
    app: &mut A,
) -> i32
where
    A: AppUiApp<N>,
{
    let mut surface = match WingSurface::open_from_argv_mut(argc, argv) {
        Ok(surface) => surface,
        Err(error) => return error.exit_code(),
    };

    match run_app_ui_rgb565::<N, _>(&mut surface, background, pacing, app) {
        Ok(()) => 0,
        Err(error) => error.exit_code(),
    }
}

impl AppUiLoop {
    pub const fn new(pacing: AppUiFramePacing) -> Self {
        Self {
            tick: AppUiTick::new(),
            pacing,
            max_input_events: 8,
            frame_stats: AppUiFrameStats::new(),
        }
    }

    pub const fn at_hz(hz: u16) -> Self {
        Self::new(AppUiFramePacing::from_hz(hz))
    }

    pub const fn with_max_input_events(mut self, max_input_events: u8) -> Self {
        self.max_input_events = max_input_events;
        self
    }

    pub const fn frame(self) -> u16 {
        self.tick.frame()
    }

    pub const fn frame_us(self) -> u32 {
        self.pacing.frame_us()
    }

    pub const fn max_input_events(self) -> u8 {
        self.max_input_events
    }

    pub const fn frame_stats(self) -> AppUiFrameStats {
        self.frame_stats
    }

    pub fn reset_frame_stats(&mut self) {
        self.frame_stats = AppUiFrameStats::new();
    }

    pub fn finish_frame(&mut self) -> u32 {
        self.tick.advance();
        self.frame_us()
    }

    pub fn finish_frame_countdown(&mut self, value: &mut u16) -> u32 {
        self.tick.step_countdown(value);
        self.frame_us()
    }

    pub fn finish_idle_frame(&mut self, hint: AppUiFrameHint) -> u32 {
        self.tick.advance();
        hint.idle_sleep_us(self.frame_us())
    }

    fn observe_frame(&mut self, result: AppUiFrameResult) -> AppUiFrameResult {
        self.frame_stats.observe(result);
        result
    }

    pub fn drain_gestures<const N: usize, F>(
        self,
        ui: &mut AppUiRuntime<N>,
        surface: &WingSurface,
        handle: F,
    ) -> Result<AppUiFlow, WingSurfaceError>
    where
        F: FnMut(AppUiGesture) -> AppUiFlow,
    {
        ui.drain_gestures(surface, self.max_input_events, handle)
    }

    pub fn drive_rgb565<const N: usize, A>(
        &mut self,
        ui: &mut AppUiRuntime<N>,
        surface: &mut WingSurface,
        app: &mut A,
    ) -> Result<AppUiFrameResult, WingSurfaceError>
    where
        A: AppUiApp<N>,
    {
        let context = AppUiContext::new(surface.width(), surface.height(), self.frame());
        let mut events = AppUiEventQueue::<APP_UI_EVENT_CAPACITY>::new();
        ui.collect_gestures(surface, self.max_input_events, &mut events)?;
        let input_count = saturating_usize_to_u8(events.len());

        let hint = app.frame_hint(context);
        if events.is_empty() && !hint.is_active() && ui.is_initialized() {
            let sleep_us = self.finish_idle_frame(hint);
            let budget = ui.frame_budget(input_count, 0, 0, events.overflowed(), false, false);
            let result = AppUiFrameResult {
                flow: AppUiFlow::Continue,
                rendered: false,
                idle: true,
                input_count,
                command_count: 0,
                dirty_rects: 0,
                budget,
                sleep_us,
                input_overflowed: events.overflowed(),
                command_overflowed: false,
                dirty_overflowed: false,
            };
            app.observe_frame(context, result);
            return Ok(self.observe_frame(result));
        }

        let update = app.update_events(context, &events);
        if update.should_exit() {
            let budget = ui.frame_budget(
                input_count,
                update.command_count,
                0,
                events.overflowed(),
                update.command_overflowed,
                false,
            );
            let result = AppUiFrameResult {
                flow: update.flow,
                rendered: false,
                idle: false,
                input_count,
                command_count: update.command_count,
                dirty_rects: 0,
                budget,
                sleep_us: 0,
                input_overflowed: events.overflowed(),
                command_overflowed: update.command_overflowed,
                dirty_overflowed: false,
            };
            app.observe_frame(context, result);
            return Ok(self.observe_frame(result));
        }

        app.before_frame(context);
        ui.compose(|frame| app.view(context, frame));
        let render = ui.render_rgb565_result(surface)?;
        app.after_frame(context);
        let sleep_us = self.finish_frame();
        let budget = ui.frame_budget(
            input_count,
            update.command_count,
            render.dirty_rects,
            events.overflowed(),
            update.command_overflowed,
            render.dirty_overflowed,
        );

        let result = AppUiFrameResult {
            flow: AppUiFlow::Continue,
            rendered: render.rendered,
            idle: false,
            input_count,
            command_count: update.command_count,
            dirty_rects: render.dirty_rects,
            budget,
            sleep_us,
            input_overflowed: events.overflowed(),
            command_overflowed: update.command_overflowed,
            dirty_overflowed: render.dirty_overflowed,
        };
        app.observe_frame(context, result);

        Ok(self.observe_frame(result))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiListItem {
    pub key: AppUiKey,
    pub title: &'static str,
    pub subtitle: &'static str,
}

impl AppUiListItem {
    pub const fn new(key: AppUiKey, title: &'static str, subtitle: &'static str) -> Self {
        Self {
            key,
            title,
            subtitle,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiSegment {
    pub key: AppUiKey,
    pub label: &'static str,
}

impl AppUiSegment {
    pub const fn new(key: AppUiKey, label: &'static str) -> Self {
        Self { key, label }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiSegmentStyle {
    pub background: Color,
    pub selected: Color,
    pub text: Color,
    pub selected_text: Color,
    pub divider: Color,
}

impl AppUiSegmentStyle {
    pub const fn new(
        background: Color,
        selected: Color,
        text: Color,
        selected_text: Color,
        divider: Color,
    ) -> Self {
        Self {
            background,
            selected,
            text,
            selected_text,
            divider,
        }
    }
}

impl Default for AppUiSegmentStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(255, 255, 255, 24),
            selected: Color::rgba(255, 255, 255, 84),
            text: Color::rgba(210, 228, 236, 220),
            selected_text: Color::WHITE,
            divider: Color::rgba(255, 255, 255, 30),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiListStyle {
    pub background: Color,
    pub row: Color,
    pub text: Color,
    pub muted_text: Color,
    pub divider: Color,
    pub scroll_track: Color,
    pub scroll_thumb: Color,
}

impl AppUiListStyle {
    pub const fn new(
        background: Color,
        row: Color,
        text: Color,
        muted_text: Color,
        divider: Color,
        scroll_track: Color,
        scroll_thumb: Color,
    ) -> Self {
        Self {
            background,
            row,
            text,
            muted_text,
            divider,
            scroll_track,
            scroll_thumb,
        }
    }
}

impl Default for AppUiListStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(255, 255, 255, 22),
            row: Color::rgba(255, 255, 255, 28),
            text: Color::WHITE,
            muted_text: Color::rgba(190, 220, 235, 210),
            divider: Color::rgba(255, 255, 255, 28),
            scroll_track: Color::rgba(255, 255, 255, 28),
            scroll_thumb: Color::rgba(255, 255, 255, 170),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiRadioStyle {
    pub background: Color,
    pub row: Color,
    pub selected_row: Color,
    pub text: Color,
    pub muted_text: Color,
    pub selected_text: Color,
    pub indicator: Color,
    pub mark: Color,
    pub divider: Color,
    pub scroll_track: Color,
    pub scroll_thumb: Color,
}

impl AppUiRadioStyle {
    pub const fn new(
        background: Color,
        row: Color,
        selected_row: Color,
        text: Color,
        muted_text: Color,
        selected_text: Color,
        indicator: Color,
        mark: Color,
        divider: Color,
        scroll_track: Color,
        scroll_thumb: Color,
    ) -> Self {
        Self {
            background,
            row,
            selected_row,
            text,
            muted_text,
            selected_text,
            indicator,
            mark,
            divider,
            scroll_track,
            scroll_thumb,
        }
    }
}

impl Default for AppUiRadioStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(255, 255, 255, 22),
            row: Color::rgba(255, 255, 255, 24),
            selected_row: Color::rgba(255, 255, 255, 48),
            text: Color::WHITE,
            muted_text: Color::rgba(190, 220, 235, 210),
            selected_text: Color::WHITE,
            indicator: Color::rgba(255, 255, 255, 150),
            mark: Color::WHITE,
            divider: Color::rgba(255, 255, 255, 28),
            scroll_track: Color::rgba(255, 255, 255, 28),
            scroll_thumb: Color::rgba(255, 255, 255, 170),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiTopBarStyle {
    pub background: Color,
    pub title: Color,
    pub subtitle: Color,
    pub button: Color,
    pub button_text: Color,
}

impl AppUiTopBarStyle {
    pub const fn new(
        background: Color,
        title: Color,
        subtitle: Color,
        button: Color,
        button_text: Color,
    ) -> Self {
        Self {
            background,
            title,
            subtitle,
            button,
            button_text,
        }
    }
}

impl Default for AppUiTopBarStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(255, 255, 255, 22),
            title: Color::WHITE,
            subtitle: Color::rgba(190, 220, 235, 220),
            button: Color::rgba(255, 255, 255, 36),
            button_text: Color::WHITE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiToastStyle {
    pub background: Color,
    pub accent: Color,
    pub text: Color,
}

impl AppUiToastStyle {
    pub const fn new(background: Color, accent: Color, text: Color) -> Self {
        Self {
            background,
            accent,
            text,
        }
    }
}

impl Default for AppUiToastStyle {
    fn default() -> Self {
        Self {
            background: Color::rgba(8, 14, 26, 220),
            accent: Color::rgba(90, 220, 235, 220),
            text: Color::WHITE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiDialogButton {
    pub key: AppUiKey,
    pub label: &'static str,
    pub primary: bool,
}

impl AppUiDialogButton {
    pub const fn new(key: AppUiKey, label: &'static str, primary: bool) -> Self {
        Self {
            key,
            label,
            primary,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiDialogStyle {
    pub scrim: Color,
    pub panel: Color,
    pub title: Color,
    pub message: Color,
    pub button: Color,
    pub primary_button: Color,
    pub button_text: Color,
    pub primary_text: Color,
}

impl AppUiDialogStyle {
    pub const fn new(
        scrim: Color,
        panel: Color,
        title: Color,
        message: Color,
        button: Color,
        primary_button: Color,
        button_text: Color,
        primary_text: Color,
    ) -> Self {
        Self {
            scrim,
            panel,
            title,
            message,
            button,
            primary_button,
            button_text,
            primary_text,
        }
    }
}

impl Default for AppUiDialogStyle {
    fn default() -> Self {
        Self {
            scrim: Color::rgba(0, 0, 0, 132),
            panel: Color::rgba(20, 28, 44, 242),
            title: Color::WHITE,
            message: Color::rgba(205, 228, 238, 224),
            button: Color::rgba(255, 255, 255, 38),
            primary_button: Color::rgba(90, 220, 235, 190),
            button_text: Color::WHITE,
            primary_text: Color::WHITE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiRowStyle {
    pub row: Color,
    pub text: Color,
    pub muted_text: Color,
    pub divider: Color,
    pub accent: Color,
    pub track: Color,
    pub control: Color,
    pub control_text: Color,
}

impl AppUiRowStyle {
    pub const fn new(
        row: Color,
        text: Color,
        muted_text: Color,
        divider: Color,
        accent: Color,
        track: Color,
        control: Color,
        control_text: Color,
    ) -> Self {
        Self {
            row,
            text,
            muted_text,
            divider,
            accent,
            track,
            control,
            control_text,
        }
    }
}

impl Default for AppUiRowStyle {
    fn default() -> Self {
        Self {
            row: Color::rgba(255, 255, 255, 24),
            text: Color::WHITE,
            muted_text: Color::rgba(190, 220, 235, 210),
            divider: Color::rgba(255, 255, 255, 24),
            accent: Color::rgba(90, 220, 235, 200),
            track: Color::rgba(255, 255, 255, 34),
            control: Color::rgba(255, 255, 255, 42),
            control_text: Color::WHITE,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiScrollMetrics {
    pub first: usize,
    pub visible: usize,
    pub total: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AppUiPress {
    key: AppUiKey,
    start: Point,
    last: Point,
    dragging: bool,
}

pub struct AppUiFrame<const N: usize = APP_UI_DEFAULT_CAPACITY> {
    specs: FixedList<AppUiSpec, N>,
}

impl<const N: usize> AppUiFrame<N> {
    pub fn new() -> Self {
        Self {
            specs: FixedList::new(),
        }
    }

    pub fn clear(&mut self) {
        self.specs.clear();
    }

    pub fn builder(&mut self) -> AppUiBuilder<'_, N> {
        AppUiBuilder::new(self)
    }

    pub fn rect(&mut self, key: AppUiKey, rect: Rect, z: i16, color: Color) {
        self.push(key, rect, z, AppVisual::Rect { color }, false);
    }

    pub fn round_rect(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        radius: u8,
        color: Color,
    ) {
        self.push(
            key,
            rect,
            z,
            AppVisual::RoundRect { color, radius },
            false,
        );
    }

    pub fn circle(&mut self, key: AppUiKey, rect: Rect, z: i16, color: Color) {
        self.push(key, rect, z, AppVisual::Circle { color }, false);
    }

    pub fn icon(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        icon: VectorIcon,
        color: Color,
    ) {
        self.push(
            key,
            rect,
            z,
            AppVisual::VectorIcon { icon, color },
            false,
        );
    }

    pub fn button_rect(&mut self, key: AppUiKey, rect: Rect, z: i16, color: Color) {
        self.push(key, rect, z, AppVisual::Rect { color }, true);
    }

    pub fn button_round_rect(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        radius: u8,
        color: Color,
    ) {
        self.push(
            key,
            rect,
            z,
            AppVisual::RoundRect { color, radius },
            true,
        );
    }

    pub fn button_circle(&mut self, key: AppUiKey, rect: Rect, z: i16, color: Color) {
        self.push(key, rect, z, AppVisual::Circle { color }, true);
    }

    pub fn hit_area(&mut self, key: AppUiKey, rect: Rect, z: i16) {
        self.push(key, rect, z, AppVisual::None, true);
    }

    pub fn back_button(&mut self, key: AppUiKey, rect: Rect, z: i16, style: AppUiTopBarStyle) {
        let radius = (rect.h / 2).min(u8::MAX as u16) as u8;
        self.round_rect(key.child(1), rect, z, radius, style.button);
        self.icon(key.child(2), inset_rect(rect, 8), z + 1, VectorIcon::Back, style.button_text);
        self.hit_area(key, inflate_rect(rect, 12), z + 2);
    }

    pub fn top_bar(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        title: &'static str,
        subtitle: &'static str,
        back_key: Option<AppUiKey>,
        icon: Option<VectorIcon>,
        style: AppUiTopBarStyle,
    ) {
        if rect.is_empty() {
            return;
        }

        let radius = (rect.h / 2).min(18).min(u8::MAX as u16) as u8;
        self.round_rect(key.child(1), rect, z, radius, style.background);

        let mut text_x = rect.x + 14;
        if let Some(back_key) = back_key {
            let size = rect.h.saturating_sub(14).clamp(24, 36).min(rect.w);
            let button_rect = Rect::new(
                rect.x + 8,
                rect.y + ((rect.h.saturating_sub(size)) / 2) as i32,
                size,
                size,
            );
            self.back_button(back_key, button_rect, z + 2, style);
            text_x = button_rect.x + button_rect.w as i32 + 10;
        }

        if let Some(icon) = icon {
            let size = rect.h.saturating_sub(18).clamp(18, 28).min(rect.w);
            let icon_rect = Rect::new(
                text_x,
                rect.y + ((rect.h.saturating_sub(size)) / 2) as i32,
                size,
                size,
            );
            self.icon(key.child(4), icon_rect, z + 1, icon, style.title);
            text_x = icon_rect.x + icon_rect.w as i32 + 8;
        }

        self.text(key.child(2), text_x, rect.y + 9, z + 1, title, style.title, 1);
        if !subtitle.is_empty() && rect.h >= 40 {
            self.text(
                key.child(3),
                text_x,
                rect.y + 23,
                z + 1,
                subtitle,
                style.subtitle,
                1,
            );
        }
    }

    pub fn slider(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        value: u8,
        track: Color,
        fill: Color,
        knob: Color,
    ) {
        let track_h = rect.h.min(16).max(4);
        let track_y = rect.y + (rect.h.saturating_sub(track_h) / 2) as i32;
        let track_rect = Rect::new(rect.x, track_y, rect.w, track_h);
        let fill_w = slider_extent(rect.w, value).max(track_h.min(rect.w));
        let knob_d = rect.h.min(32).max(12).min(rect.w.max(1));
        let knob_x = slider_knob_x(rect, knob_d, value);
        let knob_y = rect.y + (rect.h.saturating_sub(knob_d) / 2) as i32;

        self.round_rect(
            key.child(1),
            track_rect,
            z,
            (track_h / 2).min(u8::MAX as u16) as u8,
            track,
        );
        self.round_rect(
            key.child(2),
            Rect::new(rect.x, track_y, fill_w, track_h),
            z + 1,
            (track_h / 2).min(u8::MAX as u16) as u8,
            fill,
        );
        self.circle(
            key.child(3),
            Rect::new(knob_x, knob_y, knob_d, knob_d),
            z + 2,
            knob,
        );
        self.hit_area(key, rect, z + 3);
    }

    pub fn toggle(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        checked: bool,
        off_track: Color,
        on_track: Color,
        knob: Color,
    ) {
        let pad = rect.h.min(6).max(2) / 2;
        let knob_d = rect
            .h
            .saturating_sub(pad.saturating_mul(2))
            .max(8)
            .min(rect.w.max(1));
        let track_radius = (rect.h / 2).min(u8::MAX as u16) as u8;
        let knob_x = toggle_knob_x(rect, pad, knob_d, checked);
        let knob_y = rect.y + pad as i32;

        self.round_rect(
            key.child(1),
            rect,
            z,
            track_radius,
            if checked { on_track } else { off_track },
        );
        self.circle(
            key.child(2),
            Rect::new(knob_x, knob_y, knob_d, knob_d),
            z + 1,
            knob,
        );
        self.hit_area(key, rect, z + 2);
    }

    pub fn drag_handle(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        color: Color,
        grip: Color,
    ) {
        let radius = (rect.h / 2).min(u8::MAX as u16) as u8;
        self.round_rect(key.child(1), rect, z, radius, color);

        if rect.w >= 18 && rect.h >= 10 {
            let line_w = rect.w.saturating_sub(14).max(6);
            let line_x = rect.x + ((rect.w - line_w) / 2) as i32;
            let mid_y = rect.y + rect.h as i32 / 2;
            self.round_rect(
                key.child(2),
                Rect::new(line_x, mid_y - 2, line_w, 1),
                z + 1,
                1,
                grip,
            );
            self.round_rect(
                key.child(3),
                Rect::new(line_x, mid_y + 2, line_w, 1),
                z + 1,
                1,
                grip,
            );
        }

        self.hit_area(key, inflate_rect(rect, 8), z + 2);
    }

    pub fn segmented(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        segments: &[AppUiSegment],
        selected: usize,
        style: AppUiSegmentStyle,
    ) {
        if rect.is_empty() || segments.is_empty() {
            return;
        }

        let count = segments.len().min(rect.w.max(1) as usize);
        if count == 0 {
            return;
        }

        let radius = (rect.h / 2).min(u8::MAX as u16) as u8;
        self.round_rect(key.child(1), rect, z, radius, style.background);

        let base_w = rect.w / count as u16;
        for (index, segment) in segments.iter().copied().take(count).enumerate() {
            let x = rect.x + (index as i32 * base_w as i32);
            let w = if index + 1 == count {
                rect.w.saturating_sub(base_w.saturating_mul(index as u16))
            } else {
                base_w
            };
            let segment_rect = Rect::new(x, rect.y, w, rect.h);
            let is_selected = index == selected.min(count - 1);

            if is_selected {
                self.round_rect(
                    segment.key.child(1),
                    inset_rect(segment_rect, 3),
                    z + 1,
                    radius.saturating_sub(3),
                    style.selected,
                );
            } else if index > 0 {
                self.rect(
                    segment.key.child(1),
                    Rect::new(segment_rect.x, rect.y + 6, 1, rect.h.saturating_sub(12)),
                    z + 1,
                    style.divider,
                );
            }

            self.text(
                segment.key.child(2),
                centered_text_x(segment_rect, segment.label, 1),
                centered_text_y(segment_rect, 1),
                z + 2,
                segment.label,
                if is_selected {
                    style.selected_text
                } else {
                    style.text
                },
                1,
            );
            self.hit_area(segment.key, segment_rect, z + 3);
        }
    }

    pub fn scroll_list(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        items: &[AppUiListItem],
        first: usize,
        row_height: u16,
        style: AppUiListStyle,
    ) -> AppUiScrollMetrics {
        let metrics = scroll_list_metrics(rect, items.len(), first, row_height);
        if rect.is_empty() {
            return metrics;
        }

        self.round_rect(key.child(1), rect, z, 14, style.background);

        let has_scroll = metrics.total > metrics.visible && metrics.visible > 0;
        let content_w = rect.w.saturating_sub(if has_scroll { 18 } else { 0 });
        let row_height = normalized_row_height(rect, row_height);

        for slot in 0..metrics.visible {
            let Some(item) = items.get(metrics.first + slot).copied() else {
                break;
            };
            let row_y = rect.y + (slot as i32 * row_height as i32);
            let row_rect = Rect::new(
                rect.x + 4,
                row_y + 3,
                content_w.saturating_sub(8),
                row_height.saturating_sub(6),
            );

            self.round_rect(item.key.child(1), row_rect, z + 1, 10, style.row);
            self.text(
                item.key.child(2),
                row_rect.x + 10,
                row_rect.y + 7,
                z + 2,
                item.title,
                style.text,
                1,
            );

            if !item.subtitle.is_empty() && row_height >= 32 {
                self.text(
                    item.key.child(3),
                    row_rect.x + 10,
                    row_rect.y + 19,
                    z + 2,
                    item.subtitle,
                    style.muted_text,
                    1,
                );
            }

            if slot + 1 < metrics.visible {
                self.rect(
                    item.key.child(4),
                    Rect::new(
                        rect.x + 10,
                        row_y + row_height as i32 - 1,
                        content_w.saturating_sub(20),
                        1,
                    ),
                    z + 2,
                    style.divider,
                );
            }

            self.hit_area(
                item.key,
                Rect::new(rect.x, row_y, content_w, row_height),
                z + 3,
            );
        }

        if has_scroll {
            let Some(track) = scroll_track_rect(rect) else {
                return metrics;
            };
            let thumb_h = scroll_thumb_height(track.h, metrics.total, metrics.visible);
            let thumb_y =
                scroll_thumb_y(track, thumb_h, metrics.first, metrics.total, metrics.visible);
            self.round_rect(
                key.child(2),
                track,
                z + 1,
                (track.w / 2).min(u8::MAX as u16) as u8,
                style.scroll_track,
            );
            self.drag_handle(
                scroll_list_handle_key(key),
                Rect::new(track.x, thumb_y, track.w, thumb_h),
                z + 3,
                style.scroll_thumb,
                style.muted_text,
            );
        }

        metrics
    }

    pub fn radio_list(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        items: &[AppUiListItem],
        selected_key: AppUiKey,
        first: usize,
        row_height: u16,
        style: AppUiRadioStyle,
    ) -> AppUiScrollMetrics {
        let metrics = scroll_list_metrics(rect, items.len(), first, row_height);
        if rect.is_empty() {
            return metrics;
        }

        self.round_rect(key.child(1), rect, z, 14, style.background);

        let has_scroll = metrics.total > metrics.visible && metrics.visible > 0;
        let content_w = rect.w.saturating_sub(if has_scroll { 18 } else { 0 });
        let row_height = normalized_row_height(rect, row_height);

        for slot in 0..metrics.visible {
            let Some(item) = items.get(metrics.first + slot).copied() else {
                break;
            };
            let selected = item.key == selected_key;
            let row_y = rect.y + (slot as i32 * row_height as i32);
            let row_rect = Rect::new(
                rect.x + 4,
                row_y + 3,
                content_w.saturating_sub(8),
                row_height.saturating_sub(6),
            );
            let row_color = if selected { style.selected_row } else { style.row };
            let text_color = if selected {
                style.selected_text
            } else {
                style.text
            };

            self.round_rect(item.key.child(1), row_rect, z + 1, 10, row_color);

            let marker_d = row_rect.h.saturating_sub(12).clamp(8, 16);
            let marker_rect = Rect::new(
                row_rect.x + 9,
                row_rect.y + ((row_rect.h.saturating_sub(marker_d)) / 2) as i32,
                marker_d,
                marker_d,
            );
            self.circle(item.key.child(2), marker_rect, z + 2, style.indicator);
            self.circle(
                item.key.child(3),
                inset_rect(marker_rect, 3),
                z + 3,
                if selected { style.mark } else { row_color },
            );
            self.text(
                item.key.child(4),
                row_rect.x + 32,
                row_rect.y + 7,
                z + 2,
                item.title,
                text_color,
                1,
            );

            if !item.subtitle.is_empty() && row_height >= 32 {
                self.text(
                    item.key.child(5),
                    row_rect.x + 32,
                    row_rect.y + 19,
                    z + 2,
                    item.subtitle,
                    style.muted_text,
                    1,
                );
            }

            if slot + 1 < metrics.visible {
                self.rect(
                    item.key.child(6),
                    Rect::new(
                        rect.x + 10,
                        row_y + row_height as i32 - 1,
                        content_w.saturating_sub(20),
                        1,
                    ),
                    z + 2,
                    style.divider,
                );
            }

            self.hit_area(
                item.key,
                Rect::new(rect.x, row_y, content_w, row_height),
                z + 4,
            );
        }

        if has_scroll {
            let Some(track) = scroll_track_rect(rect) else {
                return metrics;
            };
            let thumb_h = scroll_thumb_height(track.h, metrics.total, metrics.visible);
            let thumb_y =
                scroll_thumb_y(track, thumb_h, metrics.first, metrics.total, metrics.visible);
            self.round_rect(
                key.child(2),
                track,
                z + 1,
                (track.w / 2).min(u8::MAX as u16) as u8,
                style.scroll_track,
            );
            self.drag_handle(
                scroll_list_handle_key(key),
                Rect::new(track.x, thumb_y, track.w, thumb_h),
                z + 3,
                style.scroll_thumb,
                style.muted_text,
            );
        }

        metrics
    }

    fn row_icon_text_x(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        icon: Option<VectorIcon>,
        color: Color,
    ) -> i32 {
        let Some(icon) = icon else {
            return rect.x + 12;
        };
        let size = rect.h.saturating_sub(14).clamp(18, 28).min(rect.w);
        let icon_rect = Rect::new(
            rect.x + 12,
            rect.y + ((rect.h.saturating_sub(size)) / 2) as i32,
            size,
            size,
        );
        self.icon(key, icon_rect, z, icon, color);
        icon_rect.x + icon_rect.w as i32 + 10
    }

    pub fn switch_row(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        title: &'static str,
        subtitle: &'static str,
        checked: bool,
        icon: Option<VectorIcon>,
        style: AppUiRowStyle,
    ) {
        if rect.is_empty() {
            return;
        }

        self.round_rect(key.child(1), rect, z, 12, style.row);
        let text_x = self.row_icon_text_x(key.child(12), rect, z + 1, icon, style.accent);
        self.text(key.child(2), text_x, rect.y + 9, z + 1, title, style.text, 1);
        if !subtitle.is_empty() && rect.h >= 36 {
            self.text(
                key.child(3),
                text_x,
                rect.y + 23,
                z + 1,
                subtitle,
                style.muted_text,
                1,
            );
        }

        let toggle_w = 52u16.min(rect.w.saturating_sub(24)).max(34);
        let toggle_h = 26u16.min(rect.h.saturating_sub(12)).max(18);
        let toggle_rect = Rect::new(
            rect.x + rect.w as i32 - toggle_w as i32 - 12,
            rect.y + ((rect.h.saturating_sub(toggle_h)) / 2) as i32,
            toggle_w,
            toggle_h,
        );
        self.draw_toggle_visual(
            key.child(10),
            toggle_rect,
            z + 1,
            checked,
            style.track,
            style.accent,
            style.control_text,
        );
        self.rect(
            key.child(11),
            Rect::new(
                rect.x + 10,
                rect.y + rect.h as i32 - 1,
                rect.w.saturating_sub(20),
                1,
            ),
            z + 1,
            style.divider,
        );
        self.hit_area(key, rect, z + 3);
    }

    pub fn status_row(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        title: &'static str,
        value: &'static str,
        progress: u8,
        icon: Option<VectorIcon>,
        style: AppUiRowStyle,
    ) {
        if rect.is_empty() {
            return;
        }

        self.round_rect(key.child(1), rect, z, 12, style.row);
        let text_x = self.row_icon_text_x(key.child(12), rect, z + 1, icon, style.accent);
        self.text(key.child(2), text_x, rect.y + 9, z + 1, title, style.text, 1);
        self.text(
            key.child(3),
            rect.x + rect.w as i32 - text_width(value, 1) as i32 - 12,
            rect.y + 9,
            z + 1,
            value,
            style.muted_text,
            1,
        );

        if rect.h >= 36 {
            let track_x = text_x;
            let track_right = rect.x + rect.w as i32 - 12;
            let track_w = (track_right - track_x).max(0).min(u16::MAX as i32) as u16;
            let track = Rect::new(
                track_x,
                rect.y + rect.h as i32 - 13,
                track_w,
                5,
            );
            let fill_w = slider_extent(track.w, progress);
            self.round_rect(key.child(4), track, z + 1, 3, style.track);
            self.round_rect(
                key.child(5),
                Rect::new(track.x, track.y, fill_w, track.h),
                z + 2,
                3,
                style.accent,
            );
        }

        self.rect(
            key.child(6),
            Rect::new(
                rect.x + 10,
                rect.y + rect.h as i32 - 1,
                rect.w.saturating_sub(20),
                1,
            ),
            z + 1,
            style.divider,
        );
    }

    pub fn stepper(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        title: &'static str,
        value_label: &'static str,
        icon: Option<VectorIcon>,
        style: AppUiRowStyle,
    ) {
        if rect.is_empty() {
            return;
        }

        self.round_rect(key.child(1), rect, z, 12, style.row);
        let text_x = self.row_icon_text_x(key.child(12), rect, z + 1, icon, style.accent);
        self.text(key.child(2), text_x, rect.y + 12, z + 1, title, style.text, 1);

        let button = 30u16.min(rect.h.saturating_sub(12)).max(22);
        let gap = 8i32;
        let plus_rect = Rect::new(
            rect.x + rect.w as i32 - button as i32 - 10,
            rect.y + ((rect.h.saturating_sub(button)) / 2) as i32,
            button,
            button,
        );
        let value_w = text_width(value_label, 1).saturating_add(16).clamp(34, 58);
        let value_rect = Rect::new(
            plus_rect.x - gap - value_w as i32,
            plus_rect.y,
            value_w,
            button,
        );
        let minus_rect = Rect::new(
            value_rect.x - gap - button as i32,
            plus_rect.y,
            button,
            button,
        );

        self.round_rect(
            stepper_decrement_key(key).child(1),
            minus_rect,
            z + 2,
            10,
            style.control,
        );
        self.text(
            stepper_decrement_key(key).child(2),
            centered_text_x(minus_rect, "-", 1),
            centered_text_y(minus_rect, 1),
            z + 3,
            "-",
            style.control_text,
            1,
        );
        self.hit_area(stepper_decrement_key(key), minus_rect, z + 4);

        self.round_rect(key.child(3), value_rect, z + 1, 10, style.track);
        self.text(
            key.child(4),
            centered_text_x(value_rect, value_label, 1),
            centered_text_y(value_rect, 1),
            z + 2,
            value_label,
            style.text,
            1,
        );

        self.round_rect(
            stepper_increment_key(key).child(1),
            plus_rect,
            z + 2,
            10,
            style.control,
        );
        self.text(
            stepper_increment_key(key).child(2),
            centered_text_x(plus_rect, "+", 1),
            centered_text_y(plus_rect, 1),
            z + 3,
            "+",
            style.control_text,
            1,
        );
        self.hit_area(stepper_increment_key(key), plus_rect, z + 4);
        self.rect(
            key.child(5),
            Rect::new(
                rect.x + 10,
                rect.y + rect.h as i32 - 1,
                rect.w.saturating_sub(20),
                1,
            ),
            z + 1,
            style.divider,
        );
    }

    pub fn toast(
        &mut self,
        key: AppUiKey,
        bounds: Rect,
        z: i16,
        message: &'static str,
        style: AppUiToastStyle,
    ) {
        if bounds.is_empty() || message.is_empty() {
            return;
        }

        let text_w = text_width(message, 1);
        let max_w = bounds.w.saturating_sub(36).max(48);
        let toast_w = text_w.saturating_add(34).clamp(64, max_w);
        let toast_h = 34u16.min(bounds.h.max(1));
        let x = bounds.x + (bounds.w.saturating_sub(toast_w) / 2) as i32;
        let y = bounds.y + bounds.h as i32 - toast_h as i32 - 24;
        let rect = Rect::new(x, y, toast_w, toast_h);

        self.round_rect(key.child(1), rect, z, 14, style.background);
        self.round_rect(
            key.child(2),
            Rect::new(rect.x + 8, rect.y + 8, 4, rect.h.saturating_sub(16)),
            z + 1,
            2,
            style.accent,
        );
        self.text(
            key.child(3),
            rect.x + 20,
            centered_text_y(rect, 1),
            z + 1,
            message,
            style.text,
            1,
        );
    }

    pub fn dialog(
        &mut self,
        key: AppUiKey,
        bounds: Rect,
        z: i16,
        title: &'static str,
        message: &'static str,
        buttons: &[AppUiDialogButton],
        style: AppUiDialogStyle,
    ) {
        if bounds.is_empty() {
            return;
        }

        self.rect(key.child(1), bounds, z, style.scrim);
        self.hit_area(key, bounds, z + 1);

        let panel_w = bounds.w.saturating_sub(56).clamp(120, 280);
        let panel_h = if message.is_empty() { 110 } else { 136 }
            .min(bounds.h.saturating_sub(40).max(96));
        let panel = Rect::new(
            bounds.x + (bounds.w.saturating_sub(panel_w) / 2) as i32,
            bounds.y + (bounds.h.saturating_sub(panel_h) / 2) as i32,
            panel_w,
            panel_h,
        );

        self.round_rect(key.child(2), panel, z + 2, 18, style.panel);
        self.text(
            key.child(3),
            panel.x + 18,
            panel.y + 18,
            z + 3,
            title,
            style.title,
            1,
        );

        if !message.is_empty() {
            self.text(
                key.child(4),
                panel.x + 18,
                panel.y + 42,
                z + 3,
                message,
                style.message,
                1,
            );
        }

        let count = buttons.len().min(3);
        if count == 0 {
            return;
        }

        let gap = 8u16;
        let total_gap = gap.saturating_mul(count.saturating_sub(1) as u16);
        let button_w = panel
            .w
            .saturating_sub(36)
            .saturating_sub(total_gap)
            .checked_div(count as u16)
            .unwrap_or(0)
            .max(42);
        let button_h = 32u16.min(panel.h.saturating_sub(24).max(24));
        let row_w = button_w
            .saturating_mul(count as u16)
            .saturating_add(total_gap);
        let start_x = panel.x + ((panel.w.saturating_sub(row_w)) / 2) as i32;
        let y = panel.y + panel.h as i32 - button_h as i32 - 16;

        for (index, button) in buttons.iter().copied().take(count).enumerate() {
            let x = start_x + index as i32 * (button_w + gap) as i32;
            let rect = Rect::new(x, y, button_w, button_h);
            let fill = if button.primary {
                style.primary_button
            } else {
                style.button
            };
            let text = if button.primary {
                style.primary_text
            } else {
                style.button_text
            };
            self.button_round_rect(button.key.child(1), rect, z + 4, 12, fill);
            self.text(
                button.key.child(2),
                centered_text_x(rect, button.label, 1),
                centered_text_y(rect, 1),
                z + 5,
                button.label,
                text,
                1,
            );
            self.hit_area(button.key, rect, z + 6);
        }
    }

    pub fn text(
        &mut self,
        key: AppUiKey,
        x: i32,
        y: i32,
        z: i16,
        text: &'static str,
        color: Color,
        scale: u8,
    ) {
        self.push(
            key,
            Rect::new(x, y, 1, 1),
            z,
            AppVisual::Text { text, color, scale },
            false,
        );
    }

    pub fn specs(&self) -> &[AppUiSpec] {
        self.specs.as_slice()
    }

    pub fn overflowed(&self) -> bool {
        self.specs.overflowed()
    }

    fn push(&mut self, key: AppUiKey, rect: Rect, z: i16, visual: AppVisual, clickable: bool) {
        let _ = self.specs.push(AppUiSpec {
            key,
            rect,
            z,
            visual,
            clickable,
        });
    }

    fn draw_toggle_visual(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        checked: bool,
        off_track: Color,
        on_track: Color,
        knob: Color,
    ) {
        let pad = rect.h.min(6).max(2) / 2;
        let knob_d = rect
            .h
            .saturating_sub(pad.saturating_mul(2))
            .max(8)
            .min(rect.w.max(1));
        let track_radius = (rect.h / 2).min(u8::MAX as u16) as u8;
        let knob_x = toggle_knob_x(rect, pad, knob_d, checked);
        let knob_y = rect.y + pad as i32;

        self.round_rect(
            key.child(1),
            rect,
            z,
            track_radius,
            if checked { on_track } else { off_track },
        );
        self.circle(
            key.child(2),
            Rect::new(knob_x, knob_y, knob_d, knob_d),
            z + 1,
            knob,
        );
    }
}

impl<const N: usize> Default for AppUiFrame<N> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AppUiBuilder<'a, const N: usize = APP_UI_DEFAULT_CAPACITY> {
    frame: &'a mut AppUiFrame<N>,
}

impl<'a, const N: usize> AppUiBuilder<'a, N> {
    pub fn new(frame: &'a mut AppUiFrame<N>) -> Self {
        Self { frame }
    }

    pub fn frame(&mut self) -> &mut AppUiFrame<N> {
        self.frame
    }

    pub fn add<B>(&mut self, bundle: B) -> B::Output
    where
        B: AppUiBundle<N>,
    {
        bundle.mount(self)
    }

    pub fn view<F, O>(&mut self, view: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
    {
        view(self)
    }

    pub fn rect(&mut self, key: AppUiKey, rect: Rect, z: i16, color: Color) {
        self.frame.rect(key, rect, z, color);
    }

    pub fn round_rect(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        radius: u8,
        color: Color,
    ) {
        self.frame.round_rect(key, rect, z, radius, color);
    }

    pub fn circle(&mut self, key: AppUiKey, rect: Rect, z: i16, color: Color) {
        self.frame.circle(key, rect, z, color);
    }

    pub fn icon(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        icon: VectorIcon,
        color: Color,
    ) {
        self.frame.icon(key, rect, z, icon, color);
    }

    pub fn button_rect(&mut self, key: AppUiKey, rect: Rect, z: i16, color: Color) {
        self.frame.button_rect(key, rect, z, color);
    }

    pub fn button_round_rect(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        radius: u8,
        color: Color,
    ) {
        self.frame.button_round_rect(key, rect, z, radius, color);
    }

    pub fn button_circle(&mut self, key: AppUiKey, rect: Rect, z: i16, color: Color) {
        self.frame.button_circle(key, rect, z, color);
    }

    pub fn hit_area(&mut self, key: AppUiKey, rect: Rect, z: i16) {
        self.frame.hit_area(key, rect, z);
    }

    pub fn top_bar(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        title: &'static str,
        subtitle: &'static str,
        back_key: Option<AppUiKey>,
        icon: Option<VectorIcon>,
        style: AppUiTopBarStyle,
    ) {
        self.frame
            .top_bar(key, rect, z, title, subtitle, back_key, icon, style);
    }

    pub fn slider(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        value: u8,
        track: Color,
        fill: Color,
        knob: Color,
    ) {
        self.frame.slider(key, rect, z, value, track, fill, knob);
    }

    pub fn toggle(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        checked: bool,
        off_track: Color,
        on_track: Color,
        knob: Color,
    ) {
        self.frame
            .toggle(key, rect, z, checked, off_track, on_track, knob);
    }

    pub fn drag_handle(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        color: Color,
        grip: Color,
    ) {
        self.frame.drag_handle(key, rect, z, color, grip);
    }

    pub fn segmented(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        segments: &[AppUiSegment],
        selected: usize,
        style: AppUiSegmentStyle,
    ) {
        self.frame
            .segmented(key, rect, z, segments, selected, style);
    }

    pub fn scroll_list(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        items: &[AppUiListItem],
        first: usize,
        row_height: u16,
        style: AppUiListStyle,
    ) -> AppUiScrollMetrics {
        self.frame
            .scroll_list(key, rect, z, items, first, row_height, style)
    }

    pub fn radio_list(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        items: &[AppUiListItem],
        selected_key: AppUiKey,
        first: usize,
        row_height: u16,
        style: AppUiRadioStyle,
    ) -> AppUiScrollMetrics {
        self.frame
            .radio_list(key, rect, z, items, selected_key, first, row_height, style)
    }

    pub fn switch_row(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        title: &'static str,
        subtitle: &'static str,
        checked: bool,
        icon: Option<VectorIcon>,
        style: AppUiRowStyle,
    ) {
        self.frame
            .switch_row(key, rect, z, title, subtitle, checked, icon, style);
    }

    pub fn status_row(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        title: &'static str,
        value: &'static str,
        progress: u8,
        icon: Option<VectorIcon>,
        style: AppUiRowStyle,
    ) {
        self.frame
            .status_row(key, rect, z, title, value, progress, icon, style);
    }

    pub fn stepper(
        &mut self,
        key: AppUiKey,
        rect: Rect,
        z: i16,
        title: &'static str,
        value_label: &'static str,
        icon: Option<VectorIcon>,
        style: AppUiRowStyle,
    ) {
        self.frame
            .stepper(key, rect, z, title, value_label, icon, style);
    }

    pub fn toast(
        &mut self,
        key: AppUiKey,
        bounds: Rect,
        z: i16,
        message: &'static str,
        style: AppUiToastStyle,
    ) {
        self.frame.toast(key, bounds, z, message, style);
    }

    pub fn dialog(
        &mut self,
        key: AppUiKey,
        bounds: Rect,
        z: i16,
        title: &'static str,
        message: &'static str,
        buttons: &[AppUiDialogButton],
        style: AppUiDialogStyle,
    ) {
        self.frame
            .dialog(key, bounds, z, title, message, buttons, style);
    }

    pub fn text(
        &mut self,
        key: AppUiKey,
        x: i32,
        y: i32,
        z: i16,
        text: &'static str,
        color: Color,
        scale: u8,
    ) {
        self.frame.text(key, x, y, z, text, color, scale);
    }

    pub fn vstack(&self, rect: Rect, padding: AppUiPadding, gap: u16) -> AppUiStack {
        AppUiStack::new(rect, AppUiAxis::Vertical, padding, gap)
    }

    pub fn hstack(&self, rect: Rect, padding: AppUiPadding, gap: u16) -> AppUiStack {
        AppUiStack::new(rect, AppUiAxis::Horizontal, padding, gap)
    }

    pub fn grid(
        &self,
        rect: Rect,
        columns: u8,
        rows: u8,
        padding: AppUiPadding,
        gap: u16,
    ) -> AppUiGrid {
        AppUiGrid::new(rect, columns, rows, padding, gap)
    }
}

pub trait AppUiBundle<const N: usize> {
    type Output;

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output;
}

pub trait AppUiView<const N: usize> {
    type Output;

    fn build(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output;
}

impl<const N: usize, B> AppUiView<N> for B
where
    B: AppUiBundle<N>,
{
    type Output = B::Output;

    fn build(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output {
        self.mount(ui)
    }
}

impl<const N: usize, B> AppUiBundle<N> for Option<B>
where
    B: AppUiBundle<N>,
{
    type Output = Option<B::Output>;

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output {
        self.map(|bundle| bundle.mount(ui))
    }
}

impl<const N: usize, A, B> AppUiBundle<N> for (A, B)
where
    A: AppUiBundle<N>,
    B: AppUiBundle<N>,
{
    type Output = (A::Output, B::Output);

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output {
        let (a, b) = self;
        (a.mount(ui), b.mount(ui))
    }
}

impl<const N: usize, A, B, C> AppUiBundle<N> for (A, B, C)
where
    A: AppUiBundle<N>,
    B: AppUiBundle<N>,
    C: AppUiBundle<N>,
{
    type Output = (A::Output, B::Output, C::Output);

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output {
        let (a, b, c) = self;
        (a.mount(ui), b.mount(ui), c.mount(ui))
    }
}

impl<const N: usize, A, B, C, D> AppUiBundle<N> for (A, B, C, D)
where
    A: AppUiBundle<N>,
    B: AppUiBundle<N>,
    C: AppUiBundle<N>,
    D: AppUiBundle<N>,
{
    type Output = (A::Output, B::Output, C::Output, D::Output);

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output {
        let (a, b, c, d) = self;
        (a.mount(ui), b.mount(ui), c.mount(ui), d.mount(ui))
    }
}

impl<const N: usize, A, B, C, D, E> AppUiBundle<N> for (A, B, C, D, E)
where
    A: AppUiBundle<N>,
    B: AppUiBundle<N>,
    C: AppUiBundle<N>,
    D: AppUiBundle<N>,
    E: AppUiBundle<N>,
{
    type Output = (A::Output, B::Output, C::Output, D::Output, E::Output);

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output {
        let (a, b, c, d, e) = self;
        (
            a.mount(ui),
            b.mount(ui),
            c.mount(ui),
            d.mount(ui),
            e.mount(ui),
        )
    }
}

impl<const N: usize, A, B, C, D, E, F> AppUiBundle<N> for (A, B, C, D, E, F)
where
    A: AppUiBundle<N>,
    B: AppUiBundle<N>,
    C: AppUiBundle<N>,
    D: AppUiBundle<N>,
    E: AppUiBundle<N>,
    F: AppUiBundle<N>,
{
    type Output = (
        A::Output,
        B::Output,
        C::Output,
        D::Output,
        E::Output,
        F::Output,
    );

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output {
        let (a, b, c, d, e, f) = self;
        (
            a.mount(ui),
            b.mount(ui),
            c.mount(ui),
            d.mount(ui),
            e.mount(ui),
            f.mount(ui),
        )
    }
}

impl<const N: usize, A, B, C, D, E, F, G> AppUiBundle<N> for (A, B, C, D, E, F, G)
where
    A: AppUiBundle<N>,
    B: AppUiBundle<N>,
    C: AppUiBundle<N>,
    D: AppUiBundle<N>,
    E: AppUiBundle<N>,
    F: AppUiBundle<N>,
    G: AppUiBundle<N>,
{
    type Output = (
        A::Output,
        B::Output,
        C::Output,
        D::Output,
        E::Output,
        F::Output,
        G::Output,
    );

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output {
        let (a, b, c, d, e, f, g) = self;
        (
            a.mount(ui),
            b.mount(ui),
            c.mount(ui),
            d.mount(ui),
            e.mount(ui),
            f.mount(ui),
            g.mount(ui),
        )
    }
}

impl<const N: usize, A, B, C, D, E, F, G, H> AppUiBundle<N> for (A, B, C, D, E, F, G, H)
where
    A: AppUiBundle<N>,
    B: AppUiBundle<N>,
    C: AppUiBundle<N>,
    D: AppUiBundle<N>,
    E: AppUiBundle<N>,
    F: AppUiBundle<N>,
    G: AppUiBundle<N>,
    H: AppUiBundle<N>,
{
    type Output = (
        A::Output,
        B::Output,
        C::Output,
        D::Output,
        E::Output,
        F::Output,
        G::Output,
        H::Output,
    );

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> Self::Output {
        let (a, b, c, d, e, f, g, h) = self;
        (
            a.mount(ui),
            b.mount(ui),
            c.mount(ui),
            d.mount(ui),
            e.mount(ui),
            f.mount(ui),
            g.mount(ui),
            h.mount(ui),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiRect {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub color: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiRect {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.rect(self.key, self.rect, self.z, self.color);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiRoundRect {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub radius: u8,
    pub color: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiRoundRect {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.round_rect(self.key, self.rect, self.z, self.radius, self.color);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiCircle {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub color: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiCircle {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.circle(self.key, self.rect, self.z, self.color);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiButtonRect {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub color: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiButtonRect {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.button_rect(self.key, self.rect, self.z, self.color);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiButtonRoundRect {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub radius: u8,
    pub color: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiButtonRoundRect {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.button_round_rect(self.key, self.rect, self.z, self.radius, self.color);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiButtonCircle {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub color: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiButtonCircle {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.button_circle(self.key, self.rect, self.z, self.color);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiHitArea {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
}

impl<const N: usize> AppUiBundle<N> for AppUiHitArea {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.hit_area(self.key, self.rect, self.z);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiLabel {
    pub key: AppUiKey,
    pub x: i32,
    pub y: i32,
    pub z: i16,
    pub text: &'static str,
    pub color: Color,
    pub scale: u8,
}

impl<const N: usize> AppUiBundle<N> for AppUiLabel {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.text(
            self.key, self.x, self.y, self.z, self.text, self.color, self.scale,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiIcon {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub icon: VectorIcon,
    pub color: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiIcon {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.icon(self.key, self.rect, self.z, self.icon, self.color);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiTopBar {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub back_key: Option<AppUiKey>,
    pub icon: Option<VectorIcon>,
    pub style: AppUiTopBarStyle,
}

impl<const N: usize> AppUiBundle<N> for AppUiTopBar {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.top_bar(
            self.key,
            self.rect,
            self.z,
            self.title,
            self.subtitle,
            self.back_key,
            self.icon,
            self.style,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiSegmented<'a> {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub segments: &'a [AppUiSegment],
    pub selected: usize,
    pub style: AppUiSegmentStyle,
}

impl<const N: usize> AppUiBundle<N> for AppUiSegmented<'_> {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.segmented(
            self.key,
            self.rect,
            self.z,
            self.segments,
            self.selected,
            self.style,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiSlider {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub value: u8,
    pub track: Color,
    pub fill: Color,
    pub knob: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiSlider {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.slider(
            self.key, self.rect, self.z, self.value, self.track, self.fill, self.knob,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiToggle {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub checked: bool,
    pub off_track: Color,
    pub on_track: Color,
    pub knob: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiToggle {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.toggle(
            self.key,
            self.rect,
            self.z,
            self.checked,
            self.off_track,
            self.on_track,
            self.knob,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiDragHandle {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub color: Color,
    pub grip: Color,
}

impl<const N: usize> AppUiBundle<N> for AppUiDragHandle {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.drag_handle(self.key, self.rect, self.z, self.color, self.grip);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiStatusRow {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub title: &'static str,
    pub value: &'static str,
    pub progress: u8,
    pub icon: Option<VectorIcon>,
    pub style: AppUiRowStyle,
}

impl<const N: usize> AppUiBundle<N> for AppUiStatusRow {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.status_row(
            self.key,
            self.rect,
            self.z,
            self.title,
            self.value,
            self.progress,
            self.icon,
            self.style,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiSwitchRow {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub checked: bool,
    pub icon: Option<VectorIcon>,
    pub style: AppUiRowStyle,
}

impl<const N: usize> AppUiBundle<N> for AppUiSwitchRow {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.switch_row(
            self.key,
            self.rect,
            self.z,
            self.title,
            self.subtitle,
            self.checked,
            self.icon,
            self.style,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiStepper {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub title: &'static str,
    pub value_label: &'static str,
    pub icon: Option<VectorIcon>,
    pub style: AppUiRowStyle,
}

impl<const N: usize> AppUiBundle<N> for AppUiStepper {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.stepper(
            self.key,
            self.rect,
            self.z,
            self.title,
            self.value_label,
            self.icon,
            self.style,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiScrollList<'a> {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub items: &'a [AppUiListItem],
    pub first: usize,
    pub row_height: u16,
    pub style: AppUiListStyle,
}

impl<const N: usize> AppUiBundle<N> for AppUiScrollList<'_> {
    type Output = AppUiScrollMetrics;

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> AppUiScrollMetrics {
        ui.scroll_list(
            self.key,
            self.rect,
            self.z,
            self.items,
            self.first,
            self.row_height,
            self.style,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiRadioList<'a> {
    pub key: AppUiKey,
    pub rect: Rect,
    pub z: i16,
    pub items: &'a [AppUiListItem],
    pub selected_key: AppUiKey,
    pub first: usize,
    pub row_height: u16,
    pub style: AppUiRadioStyle,
}

impl<const N: usize> AppUiBundle<N> for AppUiRadioList<'_> {
    type Output = AppUiScrollMetrics;

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) -> AppUiScrollMetrics {
        ui.radio_list(
            self.key,
            self.rect,
            self.z,
            self.items,
            self.selected_key,
            self.first,
            self.row_height,
            self.style,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiToast {
    pub key: AppUiKey,
    pub bounds: Rect,
    pub z: i16,
    pub message: &'static str,
    pub style: AppUiToastStyle,
}

impl<const N: usize> AppUiBundle<N> for AppUiToast {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.toast(self.key, self.bounds, self.z, self.message, self.style);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiDialog<'a> {
    pub key: AppUiKey,
    pub bounds: Rect,
    pub z: i16,
    pub title: &'static str,
    pub message: &'static str,
    pub buttons: &'a [AppUiDialogButton],
    pub style: AppUiDialogStyle,
}

impl<const N: usize> AppUiBundle<N> for AppUiDialog<'_> {
    type Output = ();

    fn mount(self, ui: &mut AppUiBuilder<'_, N>) {
        ui.dialog(
            self.key,
            self.bounds,
            self.z,
            self.title,
            self.message,
            self.buttons,
            self.style,
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AppUiPadding {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
}

impl AppUiPadding {
    pub const ZERO: Self = Self::all(0);

    pub const fn all(value: u16) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    pub const fn vh(vertical: u16, horizontal: u16) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppUiAxis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiStack {
    rect: Rect,
    axis: AppUiAxis,
    padding: AppUiPadding,
    gap: u16,
    cursor: u16,
    index: u16,
}

impl AppUiStack {
    pub const fn new(
        rect: Rect,
        axis: AppUiAxis,
        padding: AppUiPadding,
        gap: u16,
    ) -> Self {
        Self {
            rect,
            axis,
            padding,
            gap,
            cursor: 0,
            index: 0,
        }
    }

    pub fn next(&mut self, extent: u16) -> Rect {
        let gap = if self.index == 0 { 0 } else { self.gap };
        self.cursor = self.cursor.saturating_add(gap);

        let rect = match self.axis {
            AppUiAxis::Vertical => Rect::new(
                self.rect.x.saturating_add(self.padding.left as i32),
                self.rect
                    .y
                    .saturating_add(self.padding.top as i32)
                    .saturating_add(self.cursor as i32),
                app_ui_inner_width(self.rect, self.padding),
                extent.min(self.remaining_main()),
            ),
            AppUiAxis::Horizontal => Rect::new(
                self.rect
                    .x
                    .saturating_add(self.padding.left as i32)
                    .saturating_add(self.cursor as i32),
                self.rect.y.saturating_add(self.padding.top as i32),
                extent.min(self.remaining_main()),
                app_ui_inner_height(self.rect, self.padding),
            ),
        };

        self.cursor = self.cursor.saturating_add(extent);
        self.index = self.index.saturating_add(1);
        rect
    }

    pub fn remaining(&self) -> Rect {
        match self.axis {
            AppUiAxis::Vertical => Rect::new(
                self.rect.x.saturating_add(self.padding.left as i32),
                self.rect
                    .y
                    .saturating_add(self.padding.top as i32)
                    .saturating_add(self.cursor as i32),
                app_ui_inner_width(self.rect, self.padding),
                self.remaining_main(),
            ),
            AppUiAxis::Horizontal => Rect::new(
                self.rect
                    .x
                    .saturating_add(self.padding.left as i32)
                    .saturating_add(self.cursor as i32),
                self.rect.y.saturating_add(self.padding.top as i32),
                self.remaining_main(),
                app_ui_inner_height(self.rect, self.padding),
            ),
        }
    }

    pub fn spacer(&mut self, extent: u16) {
        self.cursor = self.cursor.saturating_add(extent);
    }

    fn remaining_main(self) -> u16 {
        let inner = match self.axis {
            AppUiAxis::Vertical => app_ui_inner_height(self.rect, self.padding),
            AppUiAxis::Horizontal => app_ui_inner_width(self.rect, self.padding),
        };
        inner.saturating_sub(self.cursor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppUiGrid {
    rect: Rect,
    columns: u8,
    rows: u8,
    padding: AppUiPadding,
    gap: u16,
}

impl AppUiGrid {
    pub const fn new(
        rect: Rect,
        columns: u8,
        rows: u8,
        padding: AppUiPadding,
        gap: u16,
    ) -> Self {
        Self {
            rect,
            columns,
            rows,
            padding,
            gap,
        }
    }

    pub fn cell(&self, index: usize) -> Rect {
        let columns = self.columns.max(1) as usize;
        let rows = self.rows.max(1) as usize;
        let column = (index % columns) as u16;
        let row = ((index / columns).min(rows.saturating_sub(1))) as u16;
        let gap_x = self.gap.saturating_mul(columns.saturating_sub(1) as u16);
        let gap_y = self.gap.saturating_mul(rows.saturating_sub(1) as u16);
        let cell_w =
            app_ui_inner_width(self.rect, self.padding).saturating_sub(gap_x) / columns as u16;
        let cell_h =
            app_ui_inner_height(self.rect, self.padding).saturating_sub(gap_y) / rows as u16;
        let x = self
            .rect
            .x
            .saturating_add(self.padding.left as i32)
            .saturating_add(column.saturating_mul(cell_w.saturating_add(self.gap)) as i32);
        let y = self
            .rect
            .y
            .saturating_add(self.padding.top as i32)
            .saturating_add(row.saturating_mul(cell_h.saturating_add(self.gap)) as i32);

        Rect::new(x, y, cell_w, cell_h)
    }
}

pub struct AppUiRuntime<const N: usize = APP_UI_DEFAULT_CAPACITY> {
    frame: AppUiFrame<N>,
    last: FixedList<AppUiSpec, N>,
    background: Color,
    fonts: FontStore,
    svgs: SvgStore,
    initialized: bool,
    pressed: Option<AppUiPress>,
}

struct AppUiDirtyRegion {
    rects: FixedList<Rect, APP_UI_DIRTY_RECT_CAPACITY>,
    union: Option<Rect>,
    overflowed: bool,
}

impl AppUiDirtyRegion {
    fn new() -> Self {
        Self {
            rects: FixedList::new(),
            union: None,
            overflowed: false,
        }
    }

    fn full(rect: Rect, overflowed: bool) -> Self {
        let mut region = Self::new();
        region.include(rect);
        region.overflowed = overflowed;
        region
    }

    fn include_clipped(&mut self, rect: Rect, width: u16, height: u16) {
        if let Some(rect) = clip_rect(rect, width, height) {
            self.include(rect);
        }
    }

    fn include(&mut self, rect: Rect) {
        if rect.is_empty() {
            return;
        }

        include_rect(&mut self.union, rect);
        if self.overflowed {
            return;
        }

        for existing in self.rects.as_mut_slice() {
            if rects_touch_or_overlap(*existing, rect) {
                *existing = existing.union(rect);
                return;
            }
        }

        if !self.rects.push(rect) {
            self.overflowed = true;
        }
    }

    fn is_empty(&self) -> bool {
        self.union.is_none()
    }

    fn len(&self) -> usize {
        self.rects.len()
    }

    fn rects(&self) -> &[Rect] {
        self.rects.as_slice()
    }

    fn union_rect(&self) -> Option<Rect> {
        self.union
    }

    fn overflowed(&self) -> bool {
        self.overflowed
    }
}

impl<const N: usize> AppUiRuntime<N> {
    pub fn new(background: Color) -> Self {
        Self::new_with_svg_store(background, load_runtime_default_app_svg_store())
    }

    pub fn new_with_svg_store(background: Color, svgs: SvgStore) -> Self {
        Self {
            frame: AppUiFrame::new(),
            last: FixedList::new(),
            background,
            fonts: load_runtime_default_font_store(),
            svgs,
            initialized: false,
            pressed: None,
        }
    }

    pub fn begin(&mut self) -> &mut AppUiFrame<N> {
        self.frame.clear();
        &mut self.frame
    }

    pub fn begin_builder(&mut self) -> AppUiBuilder<'_, N> {
        self.frame.clear();
        self.frame.builder()
    }

    pub fn compose<F, O>(&mut self, compose: F) -> O
    where
        F: FnOnce(&mut AppUiBuilder<'_, N>) -> O,
    {
        let mut builder = self.begin_builder();
        compose(&mut builder)
    }

    pub fn frame(&self) -> &AppUiFrame<N> {
        &self.frame
    }

    pub const fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn frame_budget(
        &self,
        event_count: u8,
        command_count: u8,
        dirty_rects: u8,
        event_overflowed: bool,
        command_overflowed: bool,
        dirty_overflowed: bool,
    ) -> AppUiFrameBudget {
        AppUiFrameBudget {
            spec_count: saturating_usize_to_u16(self.frame.specs.len()),
            spec_capacity: saturating_usize_to_u16(self.frame.specs.capacity()),
            previous_spec_count: saturating_usize_to_u16(self.last.len()),
            event_count,
            event_capacity: saturating_usize_to_u8(APP_UI_EVENT_CAPACITY),
            command_count,
            command_capacity: saturating_usize_to_u8(APP_UI_COMMAND_CAPACITY),
            dirty_rects,
            dirty_rect_capacity: saturating_usize_to_u8(APP_UI_DIRTY_RECT_CAPACITY),
            spec_overflowed: self.frame.overflowed(),
            previous_spec_overflowed: self.last.overflowed(),
            event_overflowed,
            command_overflowed,
            dirty_overflowed,
        }
    }

    pub fn handle_input(&mut self, event: WingPointerEvent) -> Option<AppUiKey> {
        match self.handle_gesture(event) {
            Some(AppUiGesture::Click { key, .. }) => Some(key),
            _ => None,
        }
    }

    pub fn handle_gesture(&mut self, event: WingPointerEvent) -> Option<AppUiGesture> {
        match event.kind {
            WingPointerKind::Down => {
                let key = self.hit_test(event.point)?;
                self.pressed = Some(AppUiPress {
                    key,
                    start: event.point,
                    last: event.point,
                    dragging: false,
                });
                Some(AppUiGesture::Press {
                    key,
                    point: event.point,
                })
            }
            WingPointerKind::Move => {
                let pressed = self.pressed.as_mut()?;
                let previous = pressed.last;
                pressed.last = event.point;
                let delta = Point::new(event.point.x - previous.x, event.point.y - previous.y);
                if delta == Point::default() {
                    return None;
                }

                pressed.dragging |= moved_past_drag_threshold(pressed.start, event.point);
                if pressed.dragging {
                    Some(AppUiGesture::Drag {
                        key: pressed.key,
                        start: pressed.start,
                        point: event.point,
                        delta,
                    })
                } else {
                    None
                }
            }
            WingPointerKind::Up => {
                let pressed = self.pressed.take()?;
                let released = self.hit_test(event.point);
                if let Some(direction) = classify_app_swipe(pressed.start, event.point) {
                    Some(AppUiGesture::Swipe {
                        key: pressed.key,
                        direction,
                        start: pressed.start,
                        end: event.point,
                    })
                } else if !pressed.dragging && released == Some(pressed.key) {
                    Some(AppUiGesture::Click {
                        key: pressed.key,
                        point: event.point,
                    })
                } else {
                    Some(AppUiGesture::Release {
                        key: pressed.key,
                        point: event.point,
                    })
                }
            }
            WingPointerKind::Cancel => {
                self.pressed
                    .take()
                    .map(|pressed| AppUiGesture::Cancel { key: pressed.key })
            }
        }
    }

    pub fn drain_gestures<F>(
        &mut self,
        surface: &WingSurface,
        max_events: u8,
        mut handle: F,
    ) -> Result<AppUiFlow, WingSurfaceError>
    where
        F: FnMut(AppUiGesture) -> AppUiFlow,
    {
        for _ in 0..max_events {
            let Some(event) = surface.poll_input()? else {
                break;
            };

            let Some(gesture) = self.handle_gesture(event) else {
                continue;
            };

            let flow = handle(gesture);
            if flow.should_exit() {
                return Ok(AppUiFlow::Exit);
            }
        }

        Ok(AppUiFlow::Continue)
    }

    pub fn collect_gestures<const M: usize>(
        &mut self,
        surface: &WingSurface,
        max_events: u8,
        events: &mut AppUiEventQueue<M>,
    ) -> Result<(), WingSurfaceError> {
        events.clear();

        for _ in 0..max_events {
            let Some(event) = surface.poll_input()? else {
                break;
            };

            let Some(gesture) = self.handle_gesture(event) else {
                continue;
            };

            if !events.push(gesture) {
                break;
            }
        }

        Ok(())
    }

    pub fn render_rgb565(&mut self, surface: &mut WingSurface) -> Result<bool, WingSurfaceError> {
        Ok(self.render_rgb565_result(surface)?.rendered)
    }

    pub fn render_rgb565_result(
        &mut self,
        surface: &mut WingSurface,
    ) -> Result<AppUiRenderResult, WingSurfaceError> {
        if surface.format() != WingSurfaceFormat::Rgb565 {
            return Err(WingSurfaceError::UnsupportedFormat);
        }

        let width = surface.width();
        let height = surface.height();
        let dirty = self.compute_dirty_region(width, height);
        if dirty.is_empty() {
            self.commit_frame();
            return Ok(AppUiRenderResult::clean());
        }

        {
            let stride = surface.stride_bytes() as usize / 2;
            let pixels = unsafe { surface.pixels_rgb565_mut()? };

            if dirty.overflowed() {
                if let Some(clip) = dirty.union_rect() {
                    let mut canvas = AppRgb565Canvas {
                        pixels,
                        width,
                        height,
                        stride,
                        clip,
                        fonts: &self.fonts,
                    };
                    canvas.clear(self.background);
                    canvas.draw_frame(&self.frame, &self.svgs);
                }
            } else {
                for clip in dirty.rects() {
                    let mut canvas = AppRgb565Canvas {
                        pixels,
                        width,
                        height,
                        stride,
                        clip: *clip,
                        fonts: &self.fonts,
                    };
                    canvas.clear(self.background);
                    canvas.draw_frame(&self.frame, &self.svgs);
                }
            }
        }

        self.commit_frame();
        if dirty.overflowed() {
            if let Some(rect) = dirty.union_rect() {
                surface.submit_dirty(dirty_to_surface_rect(rect))?;
                return Ok(AppUiRenderResult::rendered(1, true));
            }

            return Ok(AppUiRenderResult::clean());
        }

        for rect in dirty.rects() {
            surface.submit_dirty(dirty_to_surface_rect(*rect))?;
        }

        Ok(AppUiRenderResult::rendered(
            saturating_usize_to_u8(dirty.len()),
            false,
        ))
    }

    fn compute_dirty_region(&self, width: u16, height: u16) -> AppUiDirtyRegion {
        let full = Rect::new(0, 0, width, height);
        if !self.initialized || self.frame.overflowed() || self.last.overflowed() {
            return AppUiDirtyRegion::full(full, self.frame.overflowed() || self.last.overflowed());
        }

        let mut dirty = AppUiDirtyRegion::new();

        for spec in self.frame.specs() {
            let previous = find_by_key(self.last.as_slice(), spec.key);
            match previous {
                Some(old) if *old == *spec => {}
                Some(old) => {
                    dirty.include_clipped(old.bounds(), width, height);
                    dirty.include_clipped(spec.bounds(), width, height);
                }
                None => dirty.include_clipped(spec.bounds(), width, height),
            }
        }

        for old in self.last.as_slice() {
            if find_by_key(self.frame.specs(), old.key).is_none() {
                dirty.include_clipped(old.bounds(), width, height);
            }
        }

        dirty
    }

    fn commit_frame(&mut self) {
        self.last.clear();
        for spec in self.frame.specs() {
            let _ = self.last.push(*spec);
        }
        self.initialized = true;
    }

    fn hit_test(&self, point: Point) -> Option<AppUiKey> {
        let mut best: Option<(i16, usize, AppUiKey)> = None;

        for (index, spec) in self.frame.specs().iter().copied().enumerate() {
            if !spec.clickable || !spec.rect.contains(point) {
                continue;
            }

            if best
                .map(|(z, best_index, _)| spec.z > z || (spec.z == z && index >= best_index))
                .unwrap_or(true)
            {
                best = Some((spec.z, index, spec.key));
            }
        }

        best.map(|(_, _, key)| key)
    }
}

struct AppRgb565Canvas<'a> {
    pixels: &'a mut [u16],
    width: u16,
    height: u16,
    stride: usize,
    clip: Rect,
    fonts: &'a FontStore,
}

impl<'a> AppRgb565Canvas<'a> {
    fn clear(&mut self, color: Color) {
        self.fill_rect(self.clip, color);
    }

    fn draw_frame<const N: usize>(&mut self, frame: &AppUiFrame<N>, svgs: &SvgStore) {
        let mut last: Option<(i16, usize)> = None;

        loop {
            let mut next: Option<(i16, usize, AppUiSpec)> = None;

            for (index, spec) in frame.specs().iter().copied().enumerate() {
                if !rects_overlap(spec.bounds(), self.clip) || !is_after_last(last, spec.z, index)
                {
                    continue;
                }

                if next
                    .map(|(z, next_index, _)| spec.z < z || (spec.z == z && index < next_index))
                    .unwrap_or(true)
                {
                    next = Some((spec.z, index, spec));
                }
            }

            let Some((z, index, spec)) = next else {
                break;
            };

            self.draw_visual(spec.rect, spec.visual, svgs);
            last = Some((z, index));
        }
    }

    fn draw_visual(&mut self, rect: Rect, visual: AppVisual, svgs: &SvgStore) {
        match visual {
            AppVisual::None => {}
            AppVisual::Rect { color } => self.fill_rect(rect, color),
            AppVisual::RoundRect { color, radius } => self.fill_round_rect(rect, radius, color),
            AppVisual::Circle { color } => self.fill_circle(rect, color),
            AppVisual::VectorIcon { icon, color } => self.draw_vector_icon(rect, icon, color, svgs),
            AppVisual::Text { text, color, scale } => {
                self.draw_text(rect.x, rect.y, text, color, scale)
            }
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        if color.a == 255 {
            let packed = rgb565(color);
            for y in y0..y1 {
                let start = y as usize * self.stride + x0 as usize;
                let end = y as usize * self.stride + x1 as usize;
                for pixel in &mut self.pixels[start..end] {
                    *pixel = packed;
                }
            }
        } else {
            for y in y0..y1 {
                for x in x0..x1 {
                    self.blend_pixel(x, y, color);
                }
            }
        }
    }

    fn fill_round_rect(&mut self, rect: Rect, radius: u8, color: Color) {
        if radius == 0 {
            self.fill_rect(rect, color);
            return;
        }

        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        let r = radius as i32;
        let left = rect.x;
        let top = rect.y;
        let right = rect.x + rect.w as i32 - 1;
        let bottom = rect.y + rect.h as i32 - 1;

        for y in y0..y1 {
            for x in x0..x1 {
                let xi = x as i32;
                let yi = y as i32;
                let cx = if xi < left + r {
                    left + r
                } else if xi > right - r {
                    right - r
                } else {
                    xi
                };
                let cy = if yi < top + r {
                    top + r
                } else if yi > bottom - r {
                    bottom - r
                } else {
                    yi
                };
                let dx = xi - cx;
                let dy = yi - cy;
                if dx * dx + dy * dy <= r * r {
                    self.store_or_blend(x, y, color);
                }
            }
        }
    }

    fn fill_circle(&mut self, rect: Rect, color: Color) {
        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        let cx = rect.x + rect.w as i32 / 2;
        let cy = rect.y + rect.h as i32 / 2;
        let rx = (rect.w as i32 / 2).max(1);
        let ry = (rect.h as i32 / 2).max(1);
        let denom = (rx * rx * ry * ry).max(1);

        for y in y0..y1 {
            for x in x0..x1 {
                let dx = x as i32 - cx;
                let dy = y as i32 - cy;
                if dx * dx * ry * ry + dy * dy * rx * rx <= denom {
                    self.store_or_blend(x, y, color);
                }
            }
        }
    }

    fn draw_vector_icon(&mut self, rect: Rect, icon: VectorIcon, color: Color, svgs: &SvgStore) {
        if color.a == 0 {
            return;
        }

        let rect = app_ui_icon_square(rect);
        if rect.w < 2 || rect.h < 2 {
            return;
        }

        let id = vector_icon_svg_id(icon);
        let _ = svgs.with_rasterized(id, rect.w, rect.h, |mask| {
            self.draw_svg_mask(rect, mask, color);
        });
    }

    fn draw_svg_mask(&mut self, rect: Rect, mask: &SvgRasterMask, color: Color) {
        let Some((x0, y0, x1, y1)) = self.clip(rect) else {
            return;
        };

        for y in y0..y1 {
            for x in x0..x1 {
                let local_x = (x as i32 - rect.x).clamp(0, rect.w.saturating_sub(1) as i32);
                let local_y = (y as i32 - rect.y).clamp(0, rect.h.saturating_sub(1) as i32);
                let mask_x = (local_x as u32 * mask.width.saturating_sub(1) as u32
                    / rect.w.saturating_sub(1).max(1) as u32) as u16;
                let mask_y = (local_y as u32 * mask.height.saturating_sub(1) as u32
                    / rect.h.saturating_sub(1).max(1) as u32) as u16;
                let alpha = mask.alpha_at(mask_x, mask_y);
                if alpha == 0 {
                    continue;
                }
                self.blend_pixel(x, y, color.with_alpha(scale_alpha(color.a, alpha)));
            }
        }
    }

    fn draw_text(&mut self, x: i32, y: i32, text: &'static str, color: Color, scale: u8) {
        let scale = scale.max(1);
        if self.clip(font_text_bounds(x, y, text, scale)).is_none() {
            return;
        }

        let mut cursor_x = x;
        let mut cursor_y = y;
        for ch in text.chars() {
            if ch == '\n' {
                cursor_x = x;
                cursor_y += font_line_height(scale);
                continue;
            }

            let glyph = self.fonts.rasterize_cell(ch, scale);
            for gy in 0..glyph.height {
                for gx in 0..glyph.width {
                    if let Some(color) = glyph.alpha_at(gx, gy, color) {
                        if let Some((px, py)) =
                            self.visible_pixel(cursor_x + gx as i32, cursor_y + gy as i32)
                        {
                            self.blend_pixel(px, py, color);
                        }
                    }
                }
            }
            cursor_x += font_cell_advance(ch, scale);
        }
    }

    fn visible_pixel(&self, x: i32, y: i32) -> Option<(u16, u16)> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        if x < self.clip.x
            || y < self.clip.y
            || x >= self.clip.x + self.clip.w as i32
            || y >= self.clip.y + self.clip.h as i32
        {
            return None;
        }

        Some((x as u16, y as u16))
    }

    fn clip(&self, rect: Rect) -> Option<(u16, u16, u16, u16)> {
        let mut x0 = rect.x.max(0);
        let mut y0 = rect.y.max(0);
        let mut x1 = (rect.x + rect.w as i32).min(self.width as i32).max(0);
        let mut y1 = (rect.y + rect.h as i32).min(self.height as i32).max(0);

        x0 = x0.max(self.clip.x);
        y0 = y0.max(self.clip.y);
        x1 = x1.min(self.clip.x + self.clip.w as i32);
        y1 = y1.min(self.clip.y + self.clip.h as i32);

        if x0 >= x1 || y0 >= y1 {
            None
        } else {
            Some((x0 as u16, y0 as u16, x1 as u16, y1 as u16))
        }
    }

    fn store_or_blend(&mut self, x: u16, y: u16, color: Color) {
        if color.a == 255 {
            let index = y as usize * self.stride + x as usize;
            self.pixels[index] = rgb565(color);
        } else {
            self.blend_pixel(x, y, color);
        }
    }

    fn blend_pixel(&mut self, x: u16, y: u16, color: Color) {
        if color.a == 0 {
            return;
        }

        let index = y as usize * self.stride + x as usize;
        let dst = color_from_rgb565(self.pixels[index]);
        let src_a = color.a as u16;
        let inv = 255 - src_a;
        let r = (color.r as u16 * src_a + dst.r as u16 * inv) / 255;
        let g = (color.g as u16 * src_a + dst.g as u16 * inv) / 255;
        let b = (color.b as u16 * src_a + dst.b as u16 * inv) / 255;
        self.pixels[index] = rgb565(Color::rgb(r as u8, g as u8, b as u8));
    }
}

fn find_by_key(specs: &[AppUiSpec], key: AppUiKey) -> Option<&AppUiSpec> {
    specs.iter().find(|spec| spec.key == key)
}

fn include_rect(dirty: &mut Option<Rect>, rect: Rect) {
    if rect.is_empty() {
        return;
    }

    *dirty = Some(match *dirty {
        Some(current) => current.union(rect),
        None => rect,
    });
}

fn clip_rect(rect: Rect, width: u16, height: u16) -> Option<Rect> {
    let x0 = rect.x.max(0).min(width as i32);
    let y0 = rect.y.max(0).min(height as i32);
    let x1 = (rect.x + rect.w as i32).max(0).min(width as i32);
    let y1 = (rect.y + rect.h as i32).max(0).min(height as i32);

    if x1 <= x0 || y1 <= y0 {
        None
    } else {
        Some(Rect::new(x0, y0, (x1 - x0) as u16, (y1 - y0) as u16))
    }
}

fn dirty_to_surface_rect(rect: Rect) -> WingDirtyRect {
    WingDirtyRect::new(rect.x as i16, rect.y as i16, rect.w, rect.h)
}

pub fn slider_value_from_point(rect: Rect, point: Point) -> u8 {
    if rect.w <= 1 {
        return 0;
    }

    let local = (point.x - rect.x).clamp(0, rect.w as i32 - 1) as u32;
    ((local * 255) / rect.w.saturating_sub(1) as u32) as u8
}

pub fn scroll_list_handle_key(key: AppUiKey) -> AppUiKey {
    key.child(8)
}

pub fn stepper_decrement_key(key: AppUiKey) -> AppUiKey {
    key.child(8)
}

pub fn stepper_increment_key(key: AppUiKey) -> AppUiKey {
    key.child(16)
}

pub fn scroll_list_metrics(
    rect: Rect,
    item_count: usize,
    first: usize,
    row_height: u16,
) -> AppUiScrollMetrics {
    let visible = scroll_list_visible_rows(rect, row_height).min(item_count);
    let max_first = item_count.saturating_sub(visible);
    AppUiScrollMetrics {
        first: first.min(max_first),
        visible,
        total: item_count,
    }
}

pub fn scroll_list_visible_rows(rect: Rect, row_height: u16) -> usize {
    if rect.is_empty() {
        return 0;
    }

    let row_height = normalized_row_height(rect, row_height);
    (rect.h / row_height).max(1) as usize
}

pub fn scroll_list_first_after_swipe(
    first: usize,
    item_count: usize,
    visible: usize,
    direction: AppUiSwipe,
) -> usize {
    let max_first = item_count.saturating_sub(visible);
    match direction {
        AppUiSwipe::Up => first.saturating_add(1).min(max_first),
        AppUiSwipe::Down => first.saturating_sub(1),
        AppUiSwipe::Left | AppUiSwipe::Right => first.min(max_first),
    }
}

pub fn scroll_list_first_from_point(
    rect: Rect,
    item_count: usize,
    row_height: u16,
    point: Point,
) -> usize {
    let visible = scroll_list_visible_rows(rect, row_height).min(item_count);
    let max_first = item_count.saturating_sub(visible);
    let Some(track) = scroll_track_rect(rect) else {
        return 0;
    };
    if max_first == 0 {
        return 0;
    }

    let thumb_h = scroll_thumb_height(track.h, item_count, visible);
    let travel = track.h.saturating_sub(thumb_h) as i32;
    if travel <= 0 {
        return 0;
    }

    let centered = point.y - track.y - thumb_h as i32 / 2;
    let local = centered.clamp(0, travel) as usize;
    ((local * max_first) + (travel as usize / 2)) / travel as usize
}

fn slider_extent(width: u16, value: u8) -> u16 {
    if width == 0 {
        0
    } else {
        (((width as u32 * value as u32) + 254) / 255).clamp(1, width as u32) as u16
    }
}

fn slider_knob_x(rect: Rect, knob_d: u16, value: u8) -> i32 {
    let travel = rect.w.saturating_sub(knob_d) as u32;
    rect.x + ((travel * value as u32) / 255) as i32
}

fn toggle_knob_x(rect: Rect, pad: u16, knob_d: u16, checked: bool) -> i32 {
    let travel = rect.w.saturating_sub(knob_d).saturating_sub(pad.saturating_mul(2));
    rect.x + pad as i32 + if checked { travel as i32 } else { 0 }
}

fn centered_text_x(rect: Rect, text: &'static str, scale: u8) -> i32 {
    let width = text_width(text, scale) as i32;
    rect.x + ((rect.w as i32 - width).max(0) / 2)
}

fn centered_text_y(rect: Rect, scale: u8) -> i32 {
    let height = font_line_height(scale);
    rect.y + ((rect.h as i32 - height).max(0) / 2)
}

fn text_width(text: &'static str, scale: u8) -> u16 {
    font_text_bounds(0, 0, text, scale).w
}

fn normalized_row_height(rect: Rect, row_height: u16) -> u16 {
    row_height.max(24).min(rect.h.max(1))
}

fn scroll_track_rect(rect: Rect) -> Option<Rect> {
    if rect.w < 16 || rect.h < 18 {
        return None;
    }

    Some(Rect::new(
        rect.x + rect.w as i32 - 11,
        rect.y + 7,
        4,
        rect.h.saturating_sub(14),
    ))
}

fn scroll_thumb_height(track_height: u16, total: usize, visible: usize) -> u16 {
    if track_height == 0 || total == 0 || visible == 0 {
        return 0;
    }

    if total <= visible {
        return track_height;
    }

    let raw = (track_height as u32 * visible as u32) / total as u32;
    let min_h = track_height.min(14).max(1);
    raw.clamp(min_h as u32, track_height as u32) as u16
}

fn scroll_thumb_y(
    track: Rect,
    thumb_height: u16,
    first: usize,
    total: usize,
    visible: usize,
) -> i32 {
    let max_first = total.saturating_sub(visible);
    let travel = track.h.saturating_sub(thumb_height);
    if max_first == 0 || travel == 0 {
        return track.y;
    }

    track.y + ((travel as usize * first.min(max_first)) / max_first) as i32
}

fn inflate_rect(rect: Rect, amount: u16) -> Rect {
    Rect::new(
        rect.x - amount as i32,
        rect.y - amount as i32,
        rect.w.saturating_add(amount.saturating_mul(2)),
        rect.h.saturating_add(amount.saturating_mul(2)),
    )
}

fn inset_rect(rect: Rect, amount: u16) -> Rect {
    let dx = amount.min(rect.w / 2);
    let dy = amount.min(rect.h / 2);
    Rect::new(
        rect.x + dx as i32,
        rect.y + dy as i32,
        rect.w.saturating_sub(dx.saturating_mul(2)),
        rect.h.saturating_sub(dy.saturating_mul(2)),
    )
}

fn app_ui_icon_square(rect: Rect) -> Rect {
    let size = rect.w.min(rect.h);
    Rect::new(
        rect.x + ((rect.w - size) / 2) as i32,
        rect.y + ((rect.h - size) / 2) as i32,
        size,
        size,
    )
}

fn scale_alpha(a: u8, factor: u8) -> u8 {
    ((a as u16 * factor as u16) / 255) as u8
}

fn app_ui_inner_width(rect: Rect, padding: AppUiPadding) -> u16 {
    rect.w
        .saturating_sub(padding.left)
        .saturating_sub(padding.right)
}

fn app_ui_inner_height(rect: Rect, padding: AppUiPadding) -> u16 {
    rect.h
        .saturating_sub(padding.top)
        .saturating_sub(padding.bottom)
}

fn moved_past_drag_threshold(start: Point, point: Point) -> bool {
    let dx = point.x - start.x;
    let dy = point.y - start.y;
    dx.abs().max(dy.abs()) >= 6
}

fn classify_app_swipe(start: Point, end: Point) -> Option<AppUiSwipe> {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let abs_x = dx.abs();
    let abs_y = dy.abs();
    let threshold = 48;

    if abs_y >= threshold && abs_y > abs_x {
        Some(if dy < 0 {
            AppUiSwipe::Up
        } else {
            AppUiSwipe::Down
        })
    } else if abs_x >= threshold && abs_x > abs_y {
        Some(if dx < 0 {
            AppUiSwipe::Left
        } else {
            AppUiSwipe::Right
        })
    } else {
        None
    }
}

fn rects_overlap(a: Rect, b: Rect) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }

    let ax1 = a.x + a.w as i32;
    let ay1 = a.y + a.h as i32;
    let bx1 = b.x + b.w as i32;
    let by1 = b.y + b.h as i32;
    a.x < bx1 && b.x < ax1 && a.y < by1 && b.y < ay1
}

fn rects_touch_or_overlap(a: Rect, b: Rect) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }

    let ax1 = a.x + a.w as i32;
    let ay1 = a.y + a.h as i32;
    let bx1 = b.x + b.w as i32;
    let by1 = b.y + b.h as i32;
    a.x <= bx1 && b.x <= ax1 && a.y <= by1 && b.y <= ay1
}

fn is_after_last(last: Option<(i16, usize)>, z: i16, index: usize) -> bool {
    match last {
        Some((last_z, last_index)) => z > last_z || (z == last_z && index > last_index),
        None => true,
    }
}

fn rgb565(color: Color) -> u16 {
    (((color.r as u16 & 0xf8) << 8) | ((color.g as u16 & 0xfc) << 3) | (color.b as u16 >> 3))
        as u16
}

fn saturating_usize_to_u8(value: usize) -> u8 {
    value.min(u8::MAX as usize) as u8
}

fn saturating_usize_to_u16(value: usize) -> u16 {
    value.min(u16::MAX as usize) as u16
}

fn color_from_rgb565(value: u16) -> Color {
    Color::rgb(
        (((value >> 11) & 0x1f) as u8) << 3,
        (((value >> 5) & 0x3f) as u8) << 2,
        ((value & 0x1f) as u8) << 3,
    )
}
