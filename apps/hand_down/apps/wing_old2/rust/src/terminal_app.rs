use ::core::cell::UnsafeCell;
use ::core::ffi::{c_char, c_int};

use crate::app_ui_sdk::prelude::*;
use crate::app_ui_sdk::settings::parse_settings_snapshot;
use crate::platform::nuttx::load_runtime_terminal_app_svg_store;

const KEY_BACK: AppUiKey = AppUiKey::new(100);
const KEY_CMD_HELP: AppUiKey = AppUiKey::new(410);
const KEY_CMD_LS: AppUiKey = AppUiKey::new(420);
const KEY_CMD_PS: AppUiKey = AppUiKey::new(430);
const KEY_CMD_FREE: AppUiKey = AppUiKey::new(440);
const KEY_TOKEN_LS: AppUiKey = AppUiKey::new(510);
const KEY_TOKEN_CAT: AppUiKey = AppUiKey::new(520);
const KEY_TOKEN_PWD: AppUiKey = AppUiKey::new(530);
const KEY_TOKEN_CLEAR: AppUiKey = AppUiKey::new(540);
const KEY_PATH_ROOT: AppUiKey = AppUiKey::new(610);
const KEY_PATH_ETC: AppUiKey = AppUiKey::new(620);
const KEY_PATH_WING: AppUiKey = AppUiKey::new(630);
const KEY_PATH_RESOURCE: AppUiKey = AppUiKey::new(640);
const KEY_EDIT_SPACE: AppUiKey = AppUiKey::new(710);
const KEY_EDIT_BACKSPACE: AppUiKey = AppUiKey::new(720);
const KEY_EDIT_CLEAR: AppUiKey = AppUiKey::new(730);
const KEY_EDIT_ENTER: AppUiKey = AppUiKey::new(740);
const CMD_EXIT: AppUiSignalId = AppUiSignalId::new(900);
const CMD_SEND_COMMAND: AppUiSignalId = AppUiSignalId::new(910);
const CMD_APPEND_TOKEN: AppUiSignalId = AppUiSignalId::new(920);
const CMD_EDIT_INPUT: AppUiSignalId = AppUiSignalId::new(930);
const TERMINAL_UI_CAPACITY: usize = 144;
const TERMINAL_ROWS: usize = 7;
const TERMINAL_COLS: usize = 42;
const TERMINAL_INPUT_COLS: usize = 40;
const TERMINAL_READ_CHUNK: usize = 96;
const TERMINAL_READ_BUDGET: usize = 4;

const TERMINAL_COMMANDS: [AppUiSegment; 4] = [
    AppUiSegment::new(KEY_CMD_HELP, "HELP"),
    AppUiSegment::new(KEY_CMD_LS, "LS"),
    AppUiSegment::new(KEY_CMD_PS, "PS"),
    AppUiSegment::new(KEY_CMD_FREE, "FREE"),
];

const TERMINAL_TOKEN_SEGMENTS: [AppUiSegment; 4] = [
    AppUiSegment::new(KEY_TOKEN_LS, "ls"),
    AppUiSegment::new(KEY_TOKEN_CAT, "cat"),
    AppUiSegment::new(KEY_TOKEN_PWD, "pwd"),
    AppUiSegment::new(KEY_TOKEN_CLEAR, "clear"),
];

const TERMINAL_PATH_SEGMENTS: [AppUiSegment; 4] = [
    AppUiSegment::new(KEY_PATH_ROOT, "/"),
    AppUiSegment::new(KEY_PATH_ETC, "etc"),
    AppUiSegment::new(KEY_PATH_WING, "wing"),
    AppUiSegment::new(KEY_PATH_RESOURCE, "resource"),
];

const TERMINAL_EDIT_SEGMENTS: [AppUiSegment; 4] = [
    AppUiSegment::new(KEY_EDIT_SPACE, "SP"),
    AppUiSegment::new(KEY_EDIT_BACKSPACE, "BK"),
    AppUiSegment::new(KEY_EDIT_CLEAR, "CLR"),
    AppUiSegment::new(KEY_EDIT_ENTER, "ENT"),
];

extern "C" {
    fn wing_terminal_supported() -> c_int;
    fn wing_terminal_open() -> c_int;
    fn wing_terminal_read(buffer: *mut u8, buflen: usize) -> c_int;
    fn wing_terminal_write(buffer: *const u8, buflen: usize) -> c_int;
    fn wing_terminal_close() -> c_int;
}

#[no_mangle]
pub extern "C" fn wing_terminal_main(argc: i32, argv: *mut *mut c_char) -> i32 {
    let snapshot = unsafe { parse_settings_snapshot(argc, argv as *const *const c_char) }
        .ok()
        .flatten()
        .unwrap_or_default();
    let mut app = TerminalApp::new(snapshot.theme as usize, snapshot.brightness);

    unsafe {
        run_app_ui_rgb565_from_argv_with_svg_store::<TERMINAL_UI_CAPACITY, _>(
            argc,
            argv,
            Color::rgb(4, 10, 16),
            AppUiFramePacing::from_hz(15),
            load_runtime_terminal_app_svg_store(),
            &mut app,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminalBackend {
    Unsupported,
    Running,
    Failed(i32),
}

struct TerminalApp {
    theme: usize,
    brightness: u8,
    backend: TerminalBackend,
    selected_command: usize,
    input: TerminalInputBuffer,
    selected_token: usize,
    selected_edit: usize,
}

impl TerminalApp {
    fn new(theme: usize, brightness: u8) -> Self {
        terminal_text_reset();
        terminal_input_reset();
        terminal_text_set_line(0, b"wing terminal");
        terminal_text_set_line(1, b"opening pty...");

        let backend = unsafe {
            if wing_terminal_supported() == 0 {
                terminal_text_set_line(1, b"pty: disabled in config");
                terminal_text_set_line(2, b"enable CONFIG_PSEUDOTERM");
                TerminalBackend::Unsupported
            } else {
                let result = wing_terminal_open();
                if result == 0 {
                    terminal_text_set_line(1, b"pty: connected");
                    terminal_text_set_line(2, b"tap HELP LS PS FREE");
                    TerminalBackend::Running
                } else {
                    terminal_text_set_line(1, b"pty: open failed");
                    terminal_text_set_line(2, b"shell bridge unavailable");
                    TerminalBackend::Failed(result)
                }
            }
        };

        Self {
            theme,
            brightness,
            backend,
            selected_command: 0,
            input: TerminalInputBuffer::new(),
            selected_token: 0,
            selected_edit: 0,
        }
    }

    fn close_session(&mut self) {
        if self.backend == TerminalBackend::Running {
            unsafe {
                let _ = wing_terminal_close();
            }
        }
        self.backend = TerminalBackend::Unsupported;
    }

    fn drain_terminal(&mut self) {
        if self.backend != TerminalBackend::Running {
            return;
        }

        let mut buffer = [0u8; TERMINAL_READ_CHUNK];
        for _ in 0..TERMINAL_READ_BUDGET {
            let count = unsafe { wing_terminal_read(buffer.as_mut_ptr(), buffer.len()) };
            if count > 0 {
                terminal_text_push_bytes(&buffer[..count as usize]);
            } else if count < 0 {
                self.backend = TerminalBackend::Failed(count);
                terminal_text_push_line(b"pty: read error");
                break;
            } else {
                break;
            }
        }
    }

    fn send_command(&mut self, index: usize) {
        self.selected_command = index.min(TERMINAL_COMMANDS.len().saturating_sub(1));
        if self.backend != TerminalBackend::Running {
            terminal_text_push_line(b"terminal: no active pty");
            return;
        }

        let command = terminal_command_bytes(self.selected_command);
        let result = unsafe { wing_terminal_write(command.as_ptr(), command.len()) };
        if result < 0 {
            self.backend = TerminalBackend::Failed(result);
            terminal_text_push_line(b"pty: write error");
        }
    }

    fn append_token(&mut self, index: usize) {
        self.selected_token = index.min(TERMINAL_TOKEN_COUNT.saturating_sub(1));
        let Some(token) = terminal_token_bytes(self.selected_token) else {
            return;
        };
        self.input.append_token(token);
        terminal_input_set(self.input.as_bytes());
    }

    fn append_path_token(&mut self, index: usize) {
        let Some(token) = terminal_path_token_bytes(index) else {
            return;
        };
        self.input.append_token(token);
        terminal_input_set(self.input.as_bytes());
    }

    fn edit_input(&mut self, index: usize) {
        self.selected_edit = index.min(TERMINAL_EDIT_COUNT.saturating_sub(1));
        match self.selected_edit {
            0 => self.input.push_byte(b' '),
            1 => self.input.backspace(),
            2 => self.input.clear(),
            _ => self.send_input_buffer(),
        }
        terminal_input_set(self.input.as_bytes());
    }

    fn send_input_buffer(&mut self) {
        if self.input.is_empty() {
            terminal_text_push_line(b"input: empty command");
            return;
        }
        if self.backend != TerminalBackend::Running {
            terminal_text_push_line(b"terminal: no active pty");
            return;
        }

        let mut command = [0u8; TERMINAL_INPUT_COLS + 1];
        let len = self.input.copy_into(&mut command);
        command[len] = b'\n';
        let result = unsafe { wing_terminal_write(command.as_ptr(), len + 1) };
        if result < 0 {
            self.backend = TerminalBackend::Failed(result);
            terminal_text_push_line(b"pty: write error");
        } else {
            self.input.clear();
        }
    }
}

impl Drop for TerminalApp {
    fn drop(&mut self) {
        if self.backend == TerminalBackend::Running {
            unsafe {
                let _ = wing_terminal_close();
            }
        }
    }
}

impl AppUiScheduledApp<TERMINAL_UI_CAPACITY> for TerminalApp {
    type Schedule = TerminalSchedule;

    fn schedule(&mut self) -> Self::Schedule {
        terminal_schedule()
    }

    fn enqueue_scheduled_event(
        &mut self,
        _context: AppUiContext,
        gesture: AppUiGesture,
        commands: &mut AppUiCommandQueue,
    ) -> AppUiFlow {
        if let Some(key) = activation_key(gesture) {
            if key == KEY_BACK {
                commands.signal(CMD_EXIT, 0);
            } else if let Some(index) = terminal_command_index(key) {
                commands.signal(CMD_SEND_COMMAND, index as i32);
            } else if let Some(index) = terminal_token_index(key) {
                commands.signal(CMD_APPEND_TOKEN, index as i32);
            } else if let Some(index) = terminal_path_token_index(key) {
                commands.signal(CMD_APPEND_TOKEN, (TERMINAL_TOKEN_COUNT + index) as i32);
            } else if let Some(index) = terminal_edit_index(key) {
                commands.signal(CMD_EDIT_INPUT, index as i32);
            }
        }

        AppUiFlow::Continue
    }

    fn before_scheduled_frame(&mut self, _context: AppUiContext) {
        self.drain_terminal();
    }

    fn view_scheduled(
        &self,
        context: AppUiContext,
        frame: &mut AppUiBuilder<'_, TERMINAL_UI_CAPACITY>,
    ) {
        terminal_input_set(self.input.as_bytes());
        compose_terminal_frame(
            frame,
            context.width(),
            context.height(),
            context.tick(),
            self.theme,
            self.brightness,
            self.backend,
            self.selected_command,
            self.selected_token,
            self.selected_edit,
        );
    }

    fn frame_hint_scheduled(&self, _context: AppUiContext) -> AppUiFrameHint {
        if self.backend == TerminalBackend::Running {
            AppUiFrameHint::active()
        } else {
            AppUiFrameHint::idle_default()
        }
    }
}

type TerminalSchedule = AppUiStagedSchedule<
    (
        fn(&mut TerminalApp, AppUiContext, AppUiCommand) -> AppUiFlow,
        fn(&mut TerminalApp, AppUiContext, AppUiCommand) -> AppUiFlow,
    ),
    (),
>;

fn terminal_schedule() -> TerminalSchedule {
    AppUiStagedSchedule::new((terminal_flow_system, terminal_state_system), ())
}

fn terminal_flow_system(
    app: &mut TerminalApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    match command {
        AppUiCommand::Signal(signal) if signal.is(CMD_EXIT) => {
            app.close_session();
            AppUiFlow::Exit
        }
        _ => AppUiFlow::Continue,
    }
}

fn terminal_state_system(
    app: &mut TerminalApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    match command {
        AppUiCommand::Signal(signal) if signal.is(CMD_SEND_COMMAND) => {
            app.send_command(signal.value().max(0) as usize);
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_APPEND_TOKEN) => {
            let index = signal.value().max(0) as usize;
            if index < TERMINAL_TOKEN_COUNT {
                app.append_token(index);
            } else {
                app.append_path_token(index - TERMINAL_TOKEN_COUNT);
            }
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_EDIT_INPUT) => {
            app.edit_input(signal.value().max(0) as usize);
        }
        _ => {}
    }

    AppUiFlow::Continue
}

fn compose_terminal_frame(
    frame: &mut AppUiBuilder<'_, TERMINAL_UI_CAPACITY>,
    width: u16,
    height: u16,
    tick: u16,
    theme: usize,
    brightness: u8,
    backend: TerminalBackend,
    selected_command: usize,
    selected_token: usize,
    selected_edit: usize,
) {
    let accent = terminal_accent(theme, brightness);
    let muted = terminal_muted(theme);
    let pulse = triangle_wave(tick as i32 * 2, 220);
    let glow = accent.with_alpha(18 + (pulse / 12) as u8);
    let row_style = AppUiRowStyle::new(
        Color::rgba(255, 255, 255, 22),
        Color::WHITE,
        muted,
        Color::rgba(255, 255, 255, 18),
        accent.with_alpha(220),
        Color::rgba(255, 255, 255, 34),
        Color::rgba(255, 255, 255, 42),
        Color::WHITE,
    );
    let segment_style = AppUiSegmentStyle::new(
        Color::rgba(0, 0, 0, 106),
        accent.with_alpha(154),
        muted,
        Color::WHITE,
        Color::rgba(255, 255, 255, 28),
    );

    frame.add((
        AppUiRect {
            key: AppUiKey::new(1),
            rect: Rect::new(0, 0, width, height),
            z: 0,
            color: terminal_background(theme),
        },
        AppUiCircle {
            key: AppUiKey::new(2),
            rect: Rect::new(width as i32 - 116, 18, 100, 100),
            z: 1,
            color: glow,
        },
        AppUiCircle {
            key: AppUiKey::new(3),
            rect: Rect::new(-48, height as i32 - 118, 140, 140),
            z: 1,
            color: accent.with_alpha(18),
        },
        AppUiRoundRect {
            key: AppUiKey::new(4),
            rect: Rect::new(14, 14, width.saturating_sub(28), height.saturating_sub(28)),
            z: 2,
            radius: 18,
            color: Color::rgba(255, 255, 255, 18),
        },
        AppUiTopBar {
            key: AppUiKey::new(10),
            rect: Rect::new(22, 18, width.saturating_sub(44), 48),
            z: 8,
            title: "TERMINAL",
            subtitle: "WING TTY",
            back_key: Some(KEY_BACK),
            icon: Some(VectorIcon::Terminal),
            style: AppUiTopBarStyle::new(
                Color::rgba(255, 255, 255, 24),
                Color::WHITE,
                muted,
                Color::rgba(255, 255, 255, 38),
                Color::WHITE,
            ),
        },
    ));

    let content = Rect::new(26, 82, width.saturating_sub(52), height.saturating_sub(116));
    let row_h = if height >= 560 { 42 } else { 36 };
    let cmd_h = if height >= 560 { 34 } else { 30 };
    let input_h = if height >= 560 { 32 } else { 28 };
    let mut rows = frame.vstack(content, AppUiPadding::ZERO, 6);

    frame.add((
        AppUiStatusRow {
            key: AppUiKey::new(200),
            rect: rows.next(row_h),
            z: 6,
            title: "SESSION",
            value: terminal_session_label(backend),
            progress: terminal_backend_progress(backend),
            icon: Some(VectorIcon::Terminal),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(210),
            rect: rows.next(row_h),
            z: 6,
            title: "SHELL",
            value: terminal_shell_label(backend),
            progress: if backend == TerminalBackend::Running { 210 } else { 72 },
            icon: Some(VectorIcon::System),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(220),
            rect: rows.next(row_h),
            z: 6,
            title: "SURFACE",
            value: "RGB565 APP",
            progress: 214,
            icon: Some(VectorIcon::Surface),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(230),
            rect: rows.next(row_h),
            z: 6,
            title: "BACKEND",
            value: terminal_backend_label(backend),
            progress: terminal_backend_progress(backend),
            icon: Some(terminal_backend_icon(backend)),
            style: row_style,
        },
    ));

    rows.spacer(4);
    let input_rect = rows.next(input_h);
    frame.add((
        AppUiRoundRect {
            key: AppUiKey::new(500),
            rect: input_rect,
            z: 6,
            radius: 10,
            color: Color::rgba(0, 0, 0, 126),
        },
        AppUiLabel {
            key: AppUiKey::new(501),
            x: input_rect.x + 12,
            y: input_rect.y + ((input_rect.h.saturating_sub(12)) / 2) as i32,
            z: 8,
            text: terminal_input_line(),
            color: Color::WHITE,
            scale: 1,
        },
    ));
    rows.spacer(2);
    frame.add(AppUiSegmented {
        key: AppUiKey::new(400),
        rect: rows.next(cmd_h),
        z: 7,
        segments: &TERMINAL_COMMANDS,
        selected: selected_command,
        style: segment_style,
    });
    if height >= 540 {
        rows.spacer(2);
        frame.add(AppUiSegmented {
            key: AppUiKey::new(510),
            rect: rows.next(cmd_h),
            z: 7,
            segments: &TERMINAL_TOKEN_SEGMENTS,
            selected: selected_token.min(TERMINAL_TOKEN_SEGMENTS.len().saturating_sub(1)),
            style: segment_style,
        });
    }
    if height >= 600 {
        rows.spacer(2);
        frame.add(AppUiSegmented {
            key: AppUiKey::new(610),
            rect: rows.next(cmd_h),
            z: 7,
            segments: &TERMINAL_PATH_SEGMENTS,
            selected: 0,
            style: segment_style,
        });
    }
    rows.spacer(2);
    frame.add(AppUiSegmented {
        key: AppUiKey::new(700),
        rect: rows.next(cmd_h),
        z: 7,
        segments: &TERMINAL_EDIT_SEGMENTS,
        selected: selected_edit.min(TERMINAL_EDIT_SEGMENTS.len().saturating_sub(1)),
        style: segment_style,
    });
    rows.spacer(4);

    let console = rows.remaining();
    frame.add(AppUiRoundRect {
        key: AppUiKey::new(300),
        rect: console,
        z: 5,
        radius: 14,
        color: Color::rgba(0, 0, 0, 118),
    });

    let x = console.x + 12;
    let mut y = console.y + 14;
    let visible_rows = ((console.h.saturating_sub(20) / 18) as usize)
        .min(TERMINAL_ROWS)
        .max(1);
    let first_row = TERMINAL_ROWS.saturating_sub(visible_rows);
    for index in first_row..TERMINAL_ROWS {
        let text = terminal_text_line(index);
        if !text.is_empty() {
            terminal_line(
                frame,
                AppUiKey::new(310 + index as u16),
                x,
                y,
                7,
                text,
                if index == TERMINAL_ROWS - 1 { accent } else { muted },
            );
        }
        y += 18;
    }
}

fn activation_key(gesture: AppUiGesture) -> Option<AppUiKey> {
    match gesture {
        AppUiGesture::Click { key, .. } | AppUiGesture::Release { key, .. } => Some(key),
        _ => None,
    }
}

fn terminal_command_index(key: AppUiKey) -> Option<usize> {
    TERMINAL_COMMANDS
        .iter()
        .position(|segment| segment.key == key)
}

const TERMINAL_TOKEN_COUNT: usize = TERMINAL_TOKEN_SEGMENTS.len();
const TERMINAL_EDIT_COUNT: usize = TERMINAL_EDIT_SEGMENTS.len();

fn terminal_token_index(key: AppUiKey) -> Option<usize> {
    TERMINAL_TOKEN_SEGMENTS
        .iter()
        .position(|segment| segment.key == key)
}

fn terminal_path_token_index(key: AppUiKey) -> Option<usize> {
    TERMINAL_PATH_SEGMENTS
        .iter()
        .position(|segment| segment.key == key)
}

fn terminal_edit_index(key: AppUiKey) -> Option<usize> {
    TERMINAL_EDIT_SEGMENTS
        .iter()
        .position(|segment| segment.key == key)
}

fn terminal_command_bytes(index: usize) -> &'static [u8] {
    match index {
        1 => b"ls /\n",
        2 => b"ps\n",
        3 => b"free\n",
        _ => b"help\n",
    }
}

fn terminal_token_bytes(index: usize) -> Option<&'static [u8]> {
    match index {
        0 => Some(b"ls"),
        1 => Some(b"cat"),
        2 => Some(b"pwd"),
        3 => Some(b"clear"),
        _ => None,
    }
}

fn terminal_path_token_bytes(index: usize) -> Option<&'static [u8]> {
    match index {
        0 => Some(b"/"),
        1 => Some(b"etc"),
        2 => Some(b"wing"),
        3 => Some(b"resource"),
        _ => None,
    }
}

fn terminal_session_label(backend: TerminalBackend) -> &'static str {
    match backend {
        TerminalBackend::Running => "PTY RUN",
        TerminalBackend::Unsupported => "NO PTY",
        TerminalBackend::Failed(_) => "PTY ERR",
    }
}

fn terminal_shell_label(backend: TerminalBackend) -> &'static str {
    match backend {
        TerminalBackend::Running => "NSH LIVE",
        TerminalBackend::Unsupported => "NSH OFF",
        TerminalBackend::Failed(_) => "NSH FAIL",
    }
}

fn terminal_backend_label(backend: TerminalBackend) -> &'static str {
    match backend {
        TerminalBackend::Running => "PSEUDOTERM",
        TerminalBackend::Unsupported => "CONFIG OFF",
        TerminalBackend::Failed(_) => "OPEN FAIL",
    }
}

fn terminal_backend_progress(backend: TerminalBackend) -> u8 {
    match backend {
        TerminalBackend::Running => 232,
        TerminalBackend::Unsupported => 64,
        TerminalBackend::Failed(_) => 96,
    }
}

fn terminal_backend_icon(backend: TerminalBackend) -> VectorIcon {
    match backend {
        TerminalBackend::Running => VectorIcon::Terminal,
        TerminalBackend::Unsupported | TerminalBackend::Failed(_) => VectorIcon::Alert,
    }
}

fn terminal_line(
    frame: &mut AppUiBuilder<'_, TERMINAL_UI_CAPACITY>,
    key: AppUiKey,
    x: i32,
    y: i32,
    z: i16,
    text: &'static str,
    color: Color,
) {
    frame.add(AppUiLabel {
        key,
        x,
        y,
        z,
        text,
        color,
        scale: 1,
    });
}

#[derive(Clone, Copy)]
struct TerminalInputBuffer {
    bytes: [u8; TERMINAL_INPUT_COLS],
    len: usize,
}

impl TerminalInputBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; TERMINAL_INPUT_COLS],
            len: 0,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn push_byte(&mut self, byte: u8) {
        if self.len >= TERMINAL_INPUT_COLS {
            return;
        }
        self.bytes[self.len] = sanitize_terminal_byte(byte);
        self.len += 1;
    }

    fn append_token(&mut self, token: &[u8]) {
        for byte in token.iter().copied() {
            self.push_byte(byte);
        }
    }

    fn backspace(&mut self) {
        self.len = self.len.saturating_sub(1);
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn copy_into(&self, output: &mut [u8]) -> usize {
        let len = self.len.min(output.len());
        output[..len].copy_from_slice(&self.bytes[..len]);
        len
    }
}

fn terminal_background(theme: usize) -> Color {
    if theme == 1 {
        Color::rgb(16, 14, 13)
    } else {
        Color::rgb(5, 12, 20)
    }
}

fn terminal_accent(theme: usize, brightness: u8) -> Color {
    let boost = brightness / 6;
    if theme == 1 {
        Color::rgb(220, 148u8.saturating_add(boost / 2), 82)
    } else {
        Color::rgb(68, 194u8.saturating_add(boost / 2), 238)
    }
}

fn terminal_muted(theme: usize) -> Color {
    if theme == 1 {
        Color::rgba(245, 218, 186, 178)
    } else {
        Color::rgba(190, 224, 245, 172)
    }
}

fn triangle_wave(value: i32, period: i32) -> i32 {
    let period = period.max(1);
    let local = value.rem_euclid(period);
    let half = period / 2;
    if local <= half {
        local * 255 / half.max(1)
    } else {
        (period - local) * 255 / (period - half).max(1)
    }
}

struct TerminalInputStore {
    bytes: [u8; TERMINAL_INPUT_COLS + 3],
    len: usize,
}

impl TerminalInputStore {
    const fn new() -> Self {
        Self {
            bytes: [0; TERMINAL_INPUT_COLS + 3],
            len: 0,
        }
    }

    fn reset(&mut self) {
        self.len = 0;
        self.set(&[]);
    }

    fn set(&mut self, input: &[u8]) {
        self.bytes = [0; TERMINAL_INPUT_COLS + 3];
        self.bytes[0] = b'>';
        self.bytes[1] = b' ';
        let mut len = 2usize;
        for byte in input.iter().copied() {
            if len >= TERMINAL_INPUT_COLS + 2 {
                break;
            }
            self.bytes[len] = sanitize_terminal_byte(byte);
            len += 1;
        }
        if len < self.bytes.len() {
            self.bytes[len] = b'_';
            len += 1;
        }
        self.len = len;
    }
}

struct TerminalInputGlobal(UnsafeCell<TerminalInputStore>);

unsafe impl Sync for TerminalInputGlobal {}

static TERMINAL_INPUT: TerminalInputGlobal =
    TerminalInputGlobal(UnsafeCell::new(TerminalInputStore::new()));

fn terminal_input_reset() {
    unsafe {
        (*TERMINAL_INPUT.0.get()).reset();
    }
}

fn terminal_input_set(input: &[u8]) {
    unsafe {
        (*TERMINAL_INPUT.0.get()).set(input);
    }
}

fn terminal_input_line() -> &'static str {
    unsafe {
        let store = &*TERMINAL_INPUT.0.get();
        let bytes = ::core::slice::from_raw_parts(store.bytes.as_ptr(), store.len);
        ::core::str::from_utf8_unchecked(bytes)
    }
}

struct TerminalTextStore {
    lines: [[u8; TERMINAL_COLS]; TERMINAL_ROWS],
    lens: [usize; TERMINAL_ROWS],
}

impl TerminalTextStore {
    const fn new() -> Self {
        Self {
            lines: [[0; TERMINAL_COLS]; TERMINAL_ROWS],
            lens: [0; TERMINAL_ROWS],
        }
    }

    fn reset(&mut self) {
        self.lines = [[0; TERMINAL_COLS]; TERMINAL_ROWS];
        self.lens = [0; TERMINAL_ROWS];
    }

    fn set_line(&mut self, row: usize, text: &[u8]) {
        if row >= TERMINAL_ROWS {
            return;
        }

        let mut len = 0usize;
        for byte in text.iter().copied() {
            if len >= TERMINAL_COLS {
                break;
            }
            self.lines[row][len] = sanitize_terminal_byte(byte);
            len += 1;
        }
        self.lens[row] = len;
    }

    fn push_line(&mut self, text: &[u8]) {
        self.new_line();
        self.set_line(TERMINAL_ROWS - 1, text);
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes.iter().copied() {
            self.push_byte(byte);
        }
    }

    fn push_byte(&mut self, byte: u8) {
        match byte {
            b'\r' | 0 => {}
            b'\n' => self.new_line(),
            8 | 127 => self.backspace(),
            b'\t' => self.push_byte(b' '),
            _ => self.push_printable(sanitize_terminal_byte(byte)),
        }
    }

    fn push_printable(&mut self, byte: u8) {
        let row = TERMINAL_ROWS - 1;
        if self.lens[row] >= TERMINAL_COLS {
            self.new_line();
        }

        let len = self.lens[row];
        self.lines[row][len] = byte;
        self.lens[row] = len + 1;
    }

    fn backspace(&mut self) {
        let row = TERMINAL_ROWS - 1;
        if self.lens[row] > 0 {
            self.lens[row] -= 1;
        }
    }

    fn new_line(&mut self) {
        let mut row = 1usize;
        while row < TERMINAL_ROWS {
            self.lines[row - 1] = self.lines[row];
            self.lens[row - 1] = self.lens[row];
            row += 1;
        }

        self.lines[TERMINAL_ROWS - 1] = [0; TERMINAL_COLS];
        self.lens[TERMINAL_ROWS - 1] = 0;
    }
}

struct TerminalTextGlobal(UnsafeCell<TerminalTextStore>);

unsafe impl Sync for TerminalTextGlobal {}

static TERMINAL_TEXT: TerminalTextGlobal =
    TerminalTextGlobal(UnsafeCell::new(TerminalTextStore::new()));

fn terminal_text_reset() {
    unsafe {
        (*TERMINAL_TEXT.0.get()).reset();
    }
}

fn terminal_text_set_line(row: usize, text: &[u8]) {
    unsafe {
        (*TERMINAL_TEXT.0.get()).set_line(row, text);
    }
}

fn terminal_text_push_line(text: &[u8]) {
    unsafe {
        (*TERMINAL_TEXT.0.get()).push_line(text);
    }
}

fn terminal_text_push_bytes(bytes: &[u8]) {
    unsafe {
        (*TERMINAL_TEXT.0.get()).push_bytes(bytes);
    }
}

fn terminal_text_line(index: usize) -> &'static str {
    if index >= TERMINAL_ROWS {
        return "";
    }

    unsafe {
        let store = &*TERMINAL_TEXT.0.get();
        let len = store.lens[index];
        let ptr = store.lines[index].as_ptr();
        let bytes = ::core::slice::from_raw_parts(ptr, len);
        ::core::str::from_utf8_unchecked(bytes)
    }
}

fn sanitize_terminal_byte(byte: u8) -> u8 {
    if (32..=126).contains(&byte) {
        byte
    } else {
        b'?'
    }
}
