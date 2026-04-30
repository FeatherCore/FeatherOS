use ::core::ffi::c_char;

use crate::app_ui_sdk::prelude::*;
use crate::app_ui_sdk::settings::parse_settings_snapshot;
use crate::platform::nuttx::load_runtime_system_app_svg_store;

const KEY_BACK: AppUiKey = AppUiKey::new(100);
const CMD_EXIT: AppUiSignalId = AppUiSignalId::new(900);

#[no_mangle]
pub extern "C" fn wing_system_main(argc: i32, argv: *mut *mut c_char) -> i32 {
    let snapshot = unsafe { parse_settings_snapshot(argc, argv as *const *const c_char) }
        .ok()
        .flatten()
        .unwrap_or_default();
    let mut app = SystemApp {
        theme: snapshot.theme as usize,
        preview: snapshot.preview_effect as usize,
        brightness: snapshot.brightness,
        haptic: snapshot.haptic_enabled,
        reduce_motion: snapshot.reduce_motion,
    };

    unsafe {
        run_app_ui_rgb565_from_argv_with_svg_store::<96, _>(
            argc,
            argv,
            Color::rgb(7, 16, 24),
            AppUiFramePacing::from_hz(15),
            load_runtime_system_app_svg_store(),
            &mut app,
        )
    }
}

struct SystemApp {
    theme: usize,
    preview: usize,
    brightness: u8,
    haptic: bool,
    reduce_motion: bool,
}

impl AppUiScheduledApp<96> for SystemApp {
    type Schedule = SystemSchedule;

    fn schedule(&mut self) -> Self::Schedule {
        system_schedule()
    }

    fn enqueue_scheduled_event(
        &mut self,
        _context: AppUiContext,
        gesture: AppUiGesture,
        commands: &mut AppUiCommandQueue,
    ) -> AppUiFlow {
        match gesture {
            AppUiGesture::Click { key, .. } | AppUiGesture::Release { key, .. }
                if key == KEY_BACK =>
            {
                commands.signal(CMD_EXIT, 0);
            }
            _ => {}
        }

        AppUiFlow::Continue
    }

    fn view_scheduled(&self, context: AppUiContext, frame: &mut AppUiBuilder<'_, 96>) {
        compose_system_frame(
            frame,
            context.width(),
            context.height(),
            context.tick(),
            self.theme,
            self.preview,
            self.brightness,
            self.haptic,
            self.reduce_motion,
        );
    }

    fn frame_hint_scheduled(&self, _context: AppUiContext) -> AppUiFrameHint {
        AppUiFrameHint::idle_default()
    }
}

type SystemSchedule =
    AppUiStagedSchedule<fn(&mut SystemApp, AppUiContext, AppUiCommand) -> AppUiFlow>;

fn system_schedule() -> SystemSchedule {
    AppUiStagedSchedule::new(system_flow_system, ())
}

fn system_flow_system(
    _app: &mut SystemApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    match command {
        AppUiCommand::Signal(signal) if signal.is(CMD_EXIT) => AppUiFlow::Exit,
        _ => AppUiFlow::Continue,
    }
}

fn compose_system_frame(
    frame: &mut AppUiBuilder<'_, 96>,
    width: u16,
    height: u16,
    tick: u16,
    theme: usize,
    preview: usize,
    brightness: u8,
    haptic: bool,
    reduce_motion: bool,
) {
    let accent = system_accent(theme, brightness);
    let muted = system_muted(theme);
    let pulse = triangle_wave(tick as i32 * 2, 240);
    let glow = accent.with_alpha(20 + (pulse / 10) as u8);
    let row_style = AppUiRowStyle::new(
        Color::rgba(255, 255, 255, 23),
        Color::WHITE,
        muted,
        Color::rgba(255, 255, 255, 20),
        accent.with_alpha(220),
        Color::rgba(255, 255, 255, 34),
        Color::rgba(255, 255, 255, 42),
        Color::WHITE,
    );

    frame.add((
        AppUiRect {
            key: AppUiKey::new(1),
            rect: Rect::new(0, 0, width, height),
            z: 0,
            color: system_background(theme),
        },
        AppUiCircle {
            key: AppUiKey::new(2),
            rect: Rect::new(width as i32 - 124, 24, 104, 104),
            z: 1,
            color: glow,
        },
        AppUiCircle {
            key: AppUiKey::new(3),
            rect: Rect::new(-42, height as i32 - 116, 132, 132),
            z: 1,
            color: accent.with_alpha(18),
        },
        AppUiRoundRect {
            key: AppUiKey::new(4),
            rect: Rect::new(14, 14, width.saturating_sub(28), height.saturating_sub(28)),
            z: 2,
            radius: 18,
            color: Color::rgba(255, 255, 255, 20),
        },
        AppUiTopBar {
            key: AppUiKey::new(10),
            rect: Rect::new(22, 18, width.saturating_sub(44), 48),
            z: 8,
            title: "SYSTEM",
            subtitle: "DEVICE APP",
            back_key: Some(KEY_BACK),
            icon: Some(VectorIcon::System),
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
    let row_count = 8u16;
    let gap: u16 = if height >= 560 { 6 } else { 4 };
    let row_h = content
        .h
        .saturating_sub(gap.saturating_mul(row_count.saturating_sub(1)))
        .checked_div(row_count)
        .unwrap_or(34)
        .clamp(30, 42);
    let mut rows = frame.vstack(content, AppUiPadding::ZERO, gap);

    frame.add((
        AppUiStatusRow {
            key: AppUiKey::new(200),
            rect: rows.next(row_h),
            z: 6,
            title: "RUNTIME",
            value: "NUTTX RUST",
            progress: 235,
            icon: Some(VectorIcon::System),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(210),
            rect: rows.next(row_h),
            z: 6,
            title: "DISPLAY",
            value: surface_label(width, height),
            progress: surface_pressure(width, height),
            icon: Some(VectorIcon::Surface),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(220),
            rect: rows.next(row_h),
            z: 6,
            title: "PIPELINE",
            value: "SOFT + 2D",
            progress: 210,
            icon: Some(VectorIcon::Preview),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(230),
            rect: rows.next(row_h),
            z: 6,
            title: "RESOURCES",
            value: "FS PNG SVG",
            progress: 224,
            icon: Some(VectorIcon::Settings),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(240),
            rect: rows.next(row_h),
            z: 6,
            title: "FONT",
            value: "SIMHEI TTF",
            progress: 220,
            icon: Some(VectorIcon::Terminal),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(250),
            rect: rows.next(row_h),
            z: 6,
            title: "THEME",
            value: theme_label(theme),
            progress: brightness,
            icon: Some(VectorIcon::Settings),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(260),
            rect: rows.next(row_h),
            z: 6,
            title: "PREVIEW",
            value: preview_label(preview),
            progress: 192,
            icon: Some(VectorIcon::Preview),
            style: row_style,
        },
        AppUiStatusRow {
            key: AppUiKey::new(270),
            rect: rows.next(row_h),
            z: 6,
            title: "INPUT",
            value: input_label(haptic, reduce_motion),
            progress: if reduce_motion { 132 } else { 230 },
            icon: Some(VectorIcon::Surface),
            style: row_style,
        },
    ));
}

fn system_accent(theme: usize, brightness: u8) -> Color {
    let alpha = 145 + brightness / 3;
    match theme {
        1 => Color::rgba(242, 126, 94, alpha),
        _ => Color::rgba(73, 218, 233, alpha),
    }
}

fn system_muted(theme: usize) -> Color {
    match theme {
        1 => Color::rgba(240, 198, 210, 220),
        _ => Color::rgba(184, 226, 238, 220),
    }
}

fn system_background(theme: usize) -> Color {
    match theme {
        1 => Color::rgb(30, 22, 42),
        _ => Color::rgb(7, 18, 29),
    }
}

fn theme_label(theme: usize) -> &'static str {
    match theme {
        1 => "DUSK",
        _ => "AURORA",
    }
}

fn preview_label(preview: usize) -> &'static str {
    match preview {
        1 => "BALL",
        2 => "CUBE",
        _ => "CARD",
    }
}

fn surface_label(width: u16, height: u16) -> &'static str {
    if width >= height {
        "RGB565 LAND"
    } else {
        "RGB565 PORT"
    }
}

fn surface_pressure(width: u16, height: u16) -> u8 {
    let pixels = width as u32 * height as u32;
    ((pixels.saturating_mul(255) / (640 * 640)).min(255)) as u8
}

fn input_label(haptic: bool, reduce_motion: bool) -> &'static str {
    match (haptic, reduce_motion) {
        (true, true) => "HAPTIC LOW",
        (true, false) => "HAPTIC FULL",
        (false, true) => "QUIET LOW",
        (false, false) => "QUIET FULL",
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
