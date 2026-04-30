use ::core::ffi::c_char;

use crate::app_ui_sdk::prelude::*;
use crate::platform::nuttx::load_runtime_settings_app_svg_store;
use crate::app_ui_sdk::settings::{
    parse_settings_snapshot, WingSettingsClient, WingSettingsKey,
};

const KEY_BACK: AppUiKey = AppUiKey::new(100);
const KEY_BRIGHTNESS: AppUiKey = AppUiKey::new(210);
const KEY_HAPTIC: AppUiKey = AppUiKey::new(220);
const KEY_MOTION: AppUiKey = AppUiKey::new(230);
const CMD_EXIT: AppUiSignalId = AppUiSignalId::new(900);
const CMD_BRIGHTNESS_DELTA: AppUiSignalId = AppUiSignalId::new(910);
const CMD_TOGGLE_HAPTIC: AppUiSignalId = AppUiSignalId::new(920);
const CMD_TOGGLE_MOTION: AppUiSignalId = AppUiSignalId::new(930);
const CMD_SET_THEME: AppUiSignalId = AppUiSignalId::new(940);
const CMD_SET_PREVIEW: AppUiSignalId = AppUiSignalId::new(950);

const THEME_SEGMENTS: [AppUiSegment; 2] = [
    AppUiSegment::new(AppUiKey::new(310), "AUR"),
    AppUiSegment::new(AppUiKey::new(320), "DUS"),
];

const PREVIEW_SEGMENTS: [AppUiSegment; 3] = [
    AppUiSegment::new(AppUiKey::new(410), "CARD"),
    AppUiSegment::new(AppUiKey::new(420), "BALL"),
    AppUiSegment::new(AppUiKey::new(430), "CUBE"),
];

#[no_mangle]
pub extern "C" fn wing_settings_main(argc: i32, argv: *mut *mut c_char) -> i32 {
    let snapshot = unsafe { parse_settings_snapshot(argc, argv as *const *const c_char) }
        .ok()
        .flatten()
        .unwrap_or_default();
    let mut app = SettingsApp {
        settings: WingSettingsClient::open().ok(),
        theme: (snapshot.theme as usize).min(THEME_SEGMENTS.len() - 1),
        preview: (snapshot.preview_effect as usize).min(PREVIEW_SEGMENTS.len() - 1),
        brightness: snapshot.brightness,
        haptic: snapshot.haptic_enabled,
        reduce_motion: snapshot.reduce_motion,
        toast_ticks: 0,
    };

    unsafe {
        run_app_ui_rgb565_from_argv_with_svg_store::<128, _>(
            argc,
            argv,
            Color::rgb(8, 14, 24),
            AppUiFramePacing::from_hz(30),
            load_runtime_settings_app_svg_store(),
            &mut app,
        )
    }
}

struct SettingsApp {
    settings: Option<WingSettingsClient>,
    theme: usize,
    preview: usize,
    brightness: u8,
    haptic: bool,
    reduce_motion: bool,
    toast_ticks: u16,
}

fn compose_settings_frame(
    frame: &mut AppUiBuilder<'_, 128>,
    width: u16,
    height: u16,
    tick: u16,
    theme: usize,
    preview: usize,
    brightness: u8,
    haptic: bool,
    reduce_motion: bool,
    toast_ticks: u16,
) {
    let accent = theme_accent(theme, brightness);
    let muted = theme_muted(theme);
    let pulse = triangle_wave(tick as i32 * 3, 220);
    let glow = accent.with_alpha(28 + (pulse / 8) as u8);
    let row_style = AppUiRowStyle::new(
        Color::rgba(255, 255, 255, 24),
        Color::WHITE,
        muted,
        Color::rgba(255, 255, 255, 22),
        accent.with_alpha(220),
        Color::rgba(255, 255, 255, 34),
        Color::rgba(255, 255, 255, 42),
        Color::WHITE,
    );
    let segment_style = AppUiSegmentStyle::new(
        Color::rgba(255, 255, 255, 24),
        accent.with_alpha(150),
        muted,
        Color::WHITE,
        Color::rgba(255, 255, 255, 30),
    );

    frame.view(|frame| {
        frame.add((
            AppUiRect {
                key: AppUiKey::new(1),
                rect: Rect::new(0, 0, width, height),
                z: 0,
                color: theme_background(theme),
            },
            AppUiCircle {
                key: AppUiKey::new(2),
                rect: Rect::new(width as i32 - 118, 18, 96, 96),
                z: 1,
                color: glow,
            },
            AppUiCircle {
                key: AppUiKey::new(3),
                rect: Rect::new(-30, height as i32 - 108, 132, 132),
                z: 1,
                color: accent.with_alpha(22),
            },
            AppUiRoundRect {
                key: AppUiKey::new(4),
                rect: Rect::new(14, 14, width.saturating_sub(28), height.saturating_sub(28)),
                z: 2,
                radius: 18,
                color: Color::rgba(255, 255, 255, 22),
            },
            AppUiTopBar {
                key: AppUiKey::new(10),
                rect: top_bar_rect(width),
                z: 8,
                title: "SETTINGS",
                subtitle: "SYSTEM APP",
                back_key: Some(KEY_BACK),
                icon: Some(VectorIcon::Settings),
                style: AppUiTopBarStyle::new(
                    Color::rgba(255, 255, 255, 24),
                    Color::WHITE,
                    muted,
                    Color::rgba(255, 255, 255, 38),
                    Color::WHITE,
                ),
            },
        ));
    });

    frame.view(|frame| {
        let row_h = if height >= 560 { 44 } else { 38 };
        let segment_h = if height >= 560 { 42 } else { 36 };
        let mut rows = frame.vstack(
            Rect::new(26, 82, width.saturating_sub(52), height.saturating_sub(112)),
            AppUiPadding::ZERO,
            6,
        );
        let display_label_rect = rows.next(13);
        let status_rect = rows.next(row_h);
        let stepper_rect = rows.next(row_h);
        rows.spacer(4);
        let feedback_label_rect = rows.next(13);
        let haptic_rect = rows.next(row_h);
        let motion_rect = rows.next(row_h);
        rows.spacer(4);
        let shell_label_rect = rows.next(13);
        let theme_rect = rows.next(segment_h);
        let preview_rect = rows.next(segment_h);

        section_label(frame, AppUiKey::new(120), display_label_rect, 6, "DISPLAY", muted);
        section_label(frame, AppUiKey::new(121), feedback_label_rect, 6, "FEEDBACK", muted);
        section_label(frame, AppUiKey::new(122), shell_label_rect, 6, "SHELL", muted);

        frame.add((
            AppUiStatusRow {
                key: AppUiKey::new(200),
                rect: status_rect,
                z: 6,
                title: "BRIGHTNESS",
                value: brightness_label(brightness),
                progress: brightness,
                icon: Some(VectorIcon::Brightness),
                style: row_style,
            },
            AppUiStepper {
                key: KEY_BRIGHTNESS,
                rect: stepper_rect,
                z: 6,
                title: "LEVEL",
                value_label: brightness_label(brightness),
                icon: Some(VectorIcon::Brightness),
                style: row_style,
            },
            AppUiSwitchRow {
                key: KEY_HAPTIC,
                rect: haptic_rect,
                z: 6,
                title: "HAPTIC",
                subtitle: "TOUCH FEEDBACK",
                checked: haptic,
                icon: Some(VectorIcon::Haptic),
                style: row_style,
            },
            AppUiSwitchRow {
                key: KEY_MOTION,
                rect: motion_rect,
                z: 6,
                title: "MOTION",
                subtitle: "LOW ANIMATION",
                checked: reduce_motion,
                icon: Some(VectorIcon::Motion),
                style: row_style,
            },
        ));
        segment_field(
            frame,
            AppUiKey::new(300),
            theme_rect,
            6,
            "THEME",
            VectorIcon::Palette,
            &THEME_SEGMENTS,
            theme,
            row_style,
            segment_style,
        );
        segment_field(
            frame,
            AppUiKey::new(400),
            preview_rect,
            6,
            "PREVIEW",
            VectorIcon::Preview,
            &PREVIEW_SEGMENTS,
            preview,
            row_style,
            segment_style,
        );
    });

    frame.add(AppUiLabel {
        key: AppUiKey::new(500),
        x: 30,
        y: height as i32 - 18,
        z: 6,
        text: preview_label(preview),
        color: muted,
        scale: 1,
    });

    frame.add(if toast_ticks > 0 {
        Some(AppUiToast {
            key: AppUiKey::new(510),
            bounds: Rect::new(0, 0, width, height),
            z: 40,
            message: "SAVED",
            style: AppUiToastStyle::new(Color::rgba(8, 14, 24, 230), accent, Color::WHITE),
        })
    } else {
        None
    });
}

impl AppUiScheduledApp<128> for SettingsApp {
    type Schedule = SettingsSchedule;

    fn schedule(&mut self) -> Self::Schedule {
        settings_schedule()
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
            } else if key == stepper_decrement_key(KEY_BRIGHTNESS) {
                commands.signal(CMD_BRIGHTNESS_DELTA, -32);
            } else if key == stepper_increment_key(KEY_BRIGHTNESS) {
                commands.signal(CMD_BRIGHTNESS_DELTA, 32);
            } else if key == KEY_HAPTIC {
                commands.signal(CMD_TOGGLE_HAPTIC, 0);
            } else if key == KEY_MOTION {
                commands.signal(CMD_TOGGLE_MOTION, 0);
            } else if is_theme_segment_key(key) {
                commands.signal(CMD_SET_THEME, theme_index(key) as i32);
            } else if is_preview_segment_key(key) {
                commands.signal(CMD_SET_PREVIEW, preview_index(key) as i32);
            }
        }

        match gesture {
            AppUiGesture::Swipe { key, direction, .. } if is_theme_segment_key(key) => {
                commands.signal(
                    CMD_SET_THEME,
                    index_after_swipe(self.theme, THEME_SEGMENTS.len(), direction) as i32,
                );
            }
            AppUiGesture::Swipe { key, direction, .. } if is_preview_segment_key(key) => {
                commands.signal(
                    CMD_SET_PREVIEW,
                    index_after_swipe(self.preview, PREVIEW_SEGMENTS.len(), direction) as i32,
                );
            }
            _ => {}
        }

        AppUiFlow::Continue
    }

    fn view_scheduled(&self, context: AppUiContext, frame: &mut AppUiBuilder<'_, 128>) {
        compose_settings_frame(
            frame,
            context.width(),
            context.height(),
            context.tick(),
            self.theme,
            self.preview,
            self.brightness,
            self.haptic,
            self.reduce_motion,
            self.toast_ticks,
        );
    }

    fn frame_hint_scheduled(&self, _context: AppUiContext) -> AppUiFrameHint {
        if self.toast_ticks > 0 {
            AppUiFrameHint::active()
        } else {
            AppUiFrameHint::idle_default()
        }
    }
}

fn activation_key(gesture: AppUiGesture) -> Option<AppUiKey> {
    match gesture {
        AppUiGesture::Click { key, .. } | AppUiGesture::Release { key, .. } => Some(key),
        _ => None,
    }
}

fn send_setting(settings: Option<&WingSettingsClient>, key: WingSettingsKey, value: u32) {
    if let Some(settings) = settings {
        let _ = settings.send(key, value);
    }
}

fn bool_value(value: bool) -> u32 {
    if value {
        1
    } else {
        0
    }
}

type SettingsSchedule = AppUiStagedSchedule<
    (
        fn(&mut SettingsApp, AppUiContext, AppUiCommand) -> AppUiFlow,
        fn(&mut SettingsApp, AppUiContext, AppUiCommand) -> AppUiFlow,
    ),
    fn(&mut SettingsApp, AppUiContext, AppUiCommand) -> AppUiFlow,
>;

fn settings_schedule() -> SettingsSchedule {
    AppUiStagedSchedule::new(
        (settings_flow_system, settings_state_system),
        settings_tick_system,
    )
}

fn settings_flow_system(
    _app: &mut SettingsApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    match command {
        AppUiCommand::Signal(signal) if signal.is(CMD_EXIT) => AppUiFlow::Exit,
        _ => AppUiFlow::Continue,
    }
}

fn settings_state_system(
    app: &mut SettingsApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    match command {
        AppUiCommand::Signal(signal) if signal.is(CMD_BRIGHTNESS_DELTA) => {
            app.brightness = apply_u8_delta(app.brightness, signal.value());
            send_setting(
                app.settings.as_ref(),
                WingSettingsKey::Brightness,
                app.brightness as u32,
            );
            app.toast_ticks = 34;
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_TOGGLE_HAPTIC) => {
            app.haptic = !app.haptic;
            send_setting(
                app.settings.as_ref(),
                WingSettingsKey::Haptic,
                bool_value(app.haptic),
            );
            app.toast_ticks = 34;
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_TOGGLE_MOTION) => {
            app.reduce_motion = !app.reduce_motion;
            send_setting(
                app.settings.as_ref(),
                WingSettingsKey::ReduceMotion,
                bool_value(app.reduce_motion),
            );
            app.toast_ticks = 34;
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_SET_THEME) => {
            app.theme = clamp_index(signal.value(), THEME_SEGMENTS.len());
            send_setting(
                app.settings.as_ref(),
                WingSettingsKey::Theme,
                app.theme as u32,
            );
            app.toast_ticks = 34;
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_SET_PREVIEW) => {
            app.preview = clamp_index(signal.value(), PREVIEW_SEGMENTS.len());
            send_setting(
                app.settings.as_ref(),
                WingSettingsKey::PreviewEffect,
                app.preview as u32,
            );
            app.toast_ticks = 34;
        }
        _ => {}
    }

    AppUiFlow::Continue
}

fn settings_tick_system(
    app: &mut SettingsApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    if let AppUiCommand::Tick = command {
        app.toast_ticks = app.toast_ticks.saturating_sub(1);
    }

    AppUiFlow::Continue
}

fn apply_u8_delta(value: u8, delta: i32) -> u8 {
    let amount = delta.unsigned_abs().min(u8::MAX as u32) as u8;
    if delta < 0 {
        value.saturating_sub(amount)
    } else {
        value.saturating_add(amount)
    }
}

fn clamp_index(value: i32, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (value.max(0) as usize).min(len - 1)
    }
}

fn top_bar_rect(width: u16) -> Rect {
    Rect::new(22, 18, width.saturating_sub(44), 48)
}

fn section_label(
    frame: &mut AppUiBuilder<'_, 128>,
    key: AppUiKey,
    rect: Rect,
    z: i16,
    text: &'static str,
    color: Color,
) {
    if !rect.is_empty() {
        frame.text(key, rect.x + 8, rect.y + 1, z, text, color.with_alpha(180), 1);
    }
}

fn segment_field(
    frame: &mut AppUiBuilder<'_, 128>,
    key: AppUiKey,
    rect: Rect,
    z: i16,
    title: &'static str,
    icon: VectorIcon,
    segments: &[AppUiSegment],
    selected: usize,
    row_style: AppUiRowStyle,
    segment_style: AppUiSegmentStyle,
) {
    if rect.is_empty() {
        return;
    }

    frame.round_rect(key.child(1), rect, z, 12, row_style.row);
    let icon_size = rect.h.saturating_sub(14).clamp(18, 28).min(rect.w);
    let icon_rect = Rect::new(
        rect.x + 12,
        rect.y + ((rect.h.saturating_sub(icon_size)) / 2) as i32,
        icon_size,
        icon_size,
    );
    frame.icon(key.child(2), icon_rect, z + 1, icon, row_style.accent);

    let label_x = icon_rect.x + icon_rect.w as i32 + 10;
    frame.text(
        key.child(3),
        label_x,
        rect.y + ((rect.h.saturating_sub(12)) / 2) as i32,
        z + 1,
        title,
        row_style.text,
        1,
    );

    let label_w = if rect.w >= 360 { 138 } else { 104 };
    let segment_rect = Rect::new(
        rect.x + label_w as i32,
        rect.y + 6,
        rect.w.saturating_sub(label_w + 10),
        rect.h.saturating_sub(12),
    );
    frame.segmented(key.child(10), segment_rect, z + 1, segments, selected, segment_style);
    frame.rect(
        key.child(4),
        Rect::new(
            rect.x + 10,
            rect.y + rect.h as i32 - 1,
            rect.w.saturating_sub(20),
            1,
        ),
        z + 1,
        row_style.divider,
    );
}

fn theme_accent(theme: usize, brightness: u8) -> Color {
    let alpha = 150 + brightness / 3;
    match theme {
        1 => Color::rgba(244, 118, 92, alpha),
        _ => Color::rgba(72, 218, 232, alpha),
    }
}

fn theme_muted(theme: usize) -> Color {
    match theme {
        1 => Color::rgba(238, 196, 210, 220),
        _ => Color::rgba(185, 226, 238, 220),
    }
}

fn theme_background(theme: usize) -> Color {
    match theme {
        1 => Color::rgb(32, 22, 44),
        _ => Color::rgb(8, 22, 34),
    }
}

fn brightness_label(brightness: u8) -> &'static str {
    match brightness {
        0..=31 => "000",
        32..=95 => "064",
        96..=159 => "128",
        160..=223 => "192",
        _ => "255",
    }
}

fn preview_label(preview: usize) -> &'static str {
    match preview {
        1 => "PREVIEW: BALL",
        2 => "PREVIEW: CUBE",
        _ => "PREVIEW: CARD",
    }
}

fn is_theme_segment_key(key: AppUiKey) -> bool {
    key == THEME_SEGMENTS[0].key || key == THEME_SEGMENTS[1].key
}

fn theme_index(key: AppUiKey) -> usize {
    if key == THEME_SEGMENTS[1].key {
        1
    } else {
        0
    }
}

fn is_preview_segment_key(key: AppUiKey) -> bool {
    key == PREVIEW_SEGMENTS[0].key
        || key == PREVIEW_SEGMENTS[1].key
        || key == PREVIEW_SEGMENTS[2].key
}

fn preview_index(key: AppUiKey) -> usize {
    if key == PREVIEW_SEGMENTS[1].key {
        1
    } else if key == PREVIEW_SEGMENTS[2].key {
        2
    } else {
        0
    }
}

fn index_after_swipe(index: usize, count: usize, direction: AppUiSwipe) -> usize {
    match direction {
        AppUiSwipe::Left | AppUiSwipe::Up => (index + 1).min(count.saturating_sub(1)),
        AppUiSwipe::Right | AppUiSwipe::Down => index.saturating_sub(1),
    }
}

fn triangle_wave(value: i32, period: i32) -> i32 {
    let period = period.max(1);
    let phase = value.rem_euclid(period * 2);
    if phase <= period {
        phase * 255 / period
    } else {
        (period * 2 - phase) * 255 / period
    }
}
