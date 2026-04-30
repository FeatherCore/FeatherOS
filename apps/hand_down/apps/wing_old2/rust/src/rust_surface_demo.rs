use ::core::ffi::c_char;

use crate::app_ui_sdk::prelude::*;
use crate::app_ui_sdk::settings::parse_settings_snapshot;
use crate::platform::nuttx::load_runtime_surface_demo_app_svg_store;

const DEMO_LIST_ROW_HEIGHT: u16 = 34;
const KEY_BACK: AppUiKey = AppUiKey::new(700);
const KEY_DIALOG: AppUiKey = AppUiKey::new(800);
const KEY_DIALOG_OK: AppUiKey = AppUiKey::new(820);
const KEY_DIALOG_CANCEL: AppUiKey = AppUiKey::new(840);
const CMD_EXIT: AppUiSignalId = AppUiSignalId::new(900);
const CMD_CLOSE_DIALOG: AppUiSignalId = AppUiSignalId::new(910);
const CMD_TOGGLE_WARM: AppUiSignalId = AppUiSignalId::new(920);
const CMD_SET_WARM: AppUiSignalId = AppUiSignalId::new(930);
const CMD_LEVEL_DELTA: AppUiSignalId = AppUiSignalId::new(940);
const CMD_SET_MODE: AppUiSignalId = AppUiSignalId::new(950);
const CMD_SELECT_ITEM: AppUiSignalId = AppUiSignalId::new(960);
const CMD_SET_LIST_FIRST: AppUiSignalId = AppUiSignalId::new(970);
const DEMO_SEGMENTS: [AppUiSegment; 3] = [
    AppUiSegment::new(AppUiKey::new(610), "SYS"),
    AppUiSegment::new(AppUiKey::new(620), "INPUT"),
    AppUiSegment::new(AppUiKey::new(630), "FX"),
];
const DEMO_LIST_ITEMS: [AppUiListItem; 5] = [
    AppUiListItem::new(AppUiKey::new(510), "THEME", "AURORA"),
    AppUiListItem::new(AppUiKey::new(520), "INPUT", "GESTURE"),
    AppUiListItem::new(AppUiKey::new(530), "SURFACE", "RGB565"),
    AppUiListItem::new(AppUiKey::new(540), "DIRTY", "RECT"),
    AppUiListItem::new(AppUiKey::new(550), "TASK", "NUTTX"),
];
const DEMO_DIALOG_BUTTONS: [AppUiDialogButton; 2] = [
    AppUiDialogButton::new(KEY_DIALOG_CANCEL, "CANCEL", false),
    AppUiDialogButton::new(KEY_DIALOG_OK, "OK", true),
];

#[no_mangle]
pub extern "C" fn wing_surface_rust_demo_main(argc: i32, argv: *mut *mut c_char) -> i32 {
    let snapshot = unsafe { parse_settings_snapshot(argc, argv as *const *const c_char) }
        .ok()
        .flatten()
        .unwrap_or_default();
    let theme = snapshot.theme as usize;
    let mut app = DemoApp {
        theme,
        warm: theme == 1,
        level: snapshot.brightness,
        list_first: 0,
        mode: (snapshot.preview_effect as usize).min(DEMO_SEGMENTS.len() - 1),
        selected_item: DEMO_LIST_ITEMS[0].key,
        reduce_motion: snapshot.reduce_motion,
        toast_ticks: 0,
        show_dialog: false,
        last_budget: AppUiFrameBudget::empty(),
    };

    unsafe {
        run_app_ui_rgb565_from_argv_with_svg_store::<128, _>(
            argc,
            argv,
            demo_background(theme),
            AppUiFramePacing::from_hz(if snapshot.reduce_motion { 15 } else { 30 }),
            load_runtime_surface_demo_app_svg_store(),
            &mut app,
        )
    }
}

struct DemoApp {
    theme: usize,
    warm: bool,
    level: u8,
    list_first: usize,
    mode: usize,
    selected_item: AppUiKey,
    reduce_motion: bool,
    toast_ticks: u16,
    show_dialog: bool,
    last_budget: AppUiFrameBudget,
}

fn compose_demo_frame(
    frame: &mut AppUiBuilder<'_, 128>,
    width: u16,
    height: u16,
    tick: u16,
    theme: usize,
    warm: bool,
    level: u8,
    list_first: usize,
    mode: usize,
    selected_item: AppUiKey,
    reduce_motion: bool,
    toast_ticks: u16,
    show_dialog: bool,
    last_budget: AppUiFrameBudget,
) {
    let w = width as i32;
    let h = height as i32;
    let motion_tick = if reduce_motion { 0 } else { tick };
    let pulse = triangle_wave(motion_tick as i32 * 4, 200);
    let orb = 30 + ((pulse as u16 + level as u16) / 9);
    let x = if reduce_motion {
        w / 2 - orb as i32 / 2
    } else {
        28 + ((tick as i32 * 3) % (w - orb as i32 - 56).max(1))
    };
    let progress = 32 + (pulse as u16 * width.saturating_sub(96)) / 255;
    let accent = demo_accent(theme, warm, level);
    let muted = demo_muted(theme);
    let segment_rect = segment_rect(width);
    let list_rect = list_rect(width, height);
    let mut bottom_rows = frame.vstack(
        Rect::new(42, height as i32 - 172, width.saturating_sub(84), 132),
        AppUiPadding::ZERO,
        6,
    );
    let status_rect = bottom_rows.next(40);
    let switch_rect = bottom_rows.next(40);
    let stepper_rect = bottom_rows.next(40);
    let mode_label = match mode {
        1 => "INPUT PATH",
        2 => "FX PREVIEW",
        _ => "SYSTEM LIST",
    };
    let row_style = AppUiRowStyle::new(
        Color::rgba(255, 255, 255, 24),
        Color::WHITE,
        muted,
        Color::rgba(255, 255, 255, 24),
        accent.with_alpha(220),
        Color::rgba(255, 255, 255, 34),
        Color::rgba(255, 255, 255, 42),
        Color::WHITE,
    );

    frame.view(|frame| {
        frame.add(AppUiRect {
            key: k(0),
            rect: Rect::new(0, 0, width, height),
            z: 0,
            color: demo_background(theme),
        });
        frame.add((
            AppUiRoundRect {
                key: k(1),
                rect: Rect::new(18, 22, width.saturating_sub(36), height.saturating_sub(44)),
                z: 1,
                radius: 20,
                color: Color::rgba(255, 255, 255, 28),
            },
            AppUiTopBar {
                key: k(70),
                rect: top_bar_rect(width),
                z: 6,
                title: "SURFACE LAB",
                subtitle: "RGB565 APP",
                back_key: Some(KEY_BACK),
                icon: Some(VectorIcon::Surface),
                style: AppUiTopBarStyle::new(
                    Color::rgba(255, 255, 255, 24),
                    Color::WHITE,
                    muted,
                    Color::rgba(255, 255, 255, 34),
                    Color::WHITE,
                ),
            },
            AppUiSegmented {
                key: k(60),
                rect: segment_rect,
                z: 5,
                segments: &DEMO_SEGMENTS,
                selected: mode,
                style: AppUiSegmentStyle::new(
                    Color::rgba(255, 255, 255, 26),
                    accent.with_alpha(160),
                    muted,
                    Color::WHITE,
                    Color::rgba(255, 255, 255, 34),
                ),
            },
            AppUiRoundRect {
                key: k(4),
                rect: Rect::new(42, h - 96, width.saturating_sub(84), 16),
                z: 2,
                radius: 8,
                color: Color::rgba(255, 255, 255, 34),
            },
            AppUiRoundRect {
                key: k(5),
                rect: Rect::new(42, h - 96, progress, 16),
                z: 3,
                radius: 8,
                color: accent,
            },
            AppUiCircle {
                key: k(6),
                rect: Rect::new(x, list_rect.y + 6, orb, orb),
                z: 2,
                color: accent.with_alpha(64),
            },
            AppUiCircle {
                key: k(7),
                rect: Rect::new(w / 2 - 20, list_rect.y + list_rect.h as i32 - 30, 40, 40),
                z: 2,
                color: Color::rgba(255, 255, 255, 48),
            },
            AppUiLabel {
                key: k(24),
                x: 42,
                y: list_rect.y - 14,
                z: 4,
                text: mode_label,
                color: muted,
                scale: 1,
            },
        ));
    });
    frame.add(AppUiRadioList {
        key: k(40),
        rect: list_rect,
        z: 5,
        items: &DEMO_LIST_ITEMS,
        selected_key: selected_item,
        first: list_first,
        row_height: DEMO_LIST_ROW_HEIGHT,
        style: AppUiRadioStyle::new(
            Color::rgba(255, 255, 255, 20),
            Color::rgba(255, 255, 255, 24),
            accent.with_alpha(64),
            Color::WHITE,
            muted,
            Color::WHITE,
            Color::rgba(255, 255, 255, 150),
            accent.with_alpha(235),
            Color::rgba(255, 255, 255, 22),
            Color::rgba(255, 255, 255, 24),
            accent.with_alpha(210),
        ),
    });
    frame.add((
        AppUiStatusRow {
            key: k(100),
            rect: status_rect,
            z: 6,
            title: "APP BUDGET",
            value: app_ui_budget_label(last_budget),
            progress: app_ui_budget_pressure(last_budget),
            icon: Some(VectorIcon::Preview),
            style: row_style,
        },
        AppUiSwitchRow {
            key: k(110),
            rect: switch_rect,
            z: 6,
            title: "WARM MODE",
            subtitle: "SWITCH ROW",
            checked: warm,
            icon: Some(VectorIcon::Palette),
            style: row_style,
        },
        AppUiStepper {
            key: k(120),
            rect: stepper_rect,
            z: 6,
            title: "LEVEL",
            value_label: level_label(level),
            icon: Some(VectorIcon::Brightness),
            style: row_style,
        },
    ));
    frame.add(AppUiLabel {
        key: k(8),
        x: 42,
        y: h - 22,
        z: 4,
        text: if reduce_motion { "DIFF + LOW MOTION" } else { "DIFF + BUDGET" },
        color: muted,
        scale: 1,
    });

    frame.add(if show_dialog {
        Some(AppUiDialog {
            key: KEY_DIALOG,
            bounds: Rect::new(0, 0, width, height),
            z: 40,
            title: "TASK",
            message: "NUTTX TASK THREAD",
            buttons: &DEMO_DIALOG_BUTTONS,
            style: AppUiDialogStyle::new(
                Color::rgba(0, 0, 0, 138),
                Color::rgba(19, 28, 45, 244),
                Color::WHITE,
                Color::rgba(210, 230, 238, 224),
                Color::rgba(255, 255, 255, 40),
                accent.with_alpha(210),
                Color::WHITE,
                Color::WHITE,
            ),
        })
    } else {
        None
    });
    frame.add(if !show_dialog && toast_ticks > 0 {
        Some(AppUiToast {
            key: k(90),
            bounds: Rect::new(0, 0, width, height),
            z: 40,
            message: "ITEM SELECTED",
            style: AppUiToastStyle::new(
                Color::rgba(8, 14, 26, 230),
                accent.with_alpha(230),
                Color::WHITE,
            ),
        })
    } else {
        None
    });
}

impl AppUiScheduledApp<128> for DemoApp {
    type Schedule = DemoSchedule;

    fn schedule(&mut self) -> Self::Schedule {
        demo_schedule()
    }

    fn enqueue_scheduled_event(
        &mut self,
        context: AppUiContext,
        gesture: AppUiGesture,
        commands: &mut AppUiCommandQueue,
    ) -> AppUiFlow {
        let settings_list = list_rect(context.width(), context.height());
        let list_visible =
            scroll_list_visible_rows(settings_list, DEMO_LIST_ROW_HEIGHT).min(DEMO_LIST_ITEMS.len());

        match gesture {
            AppUiGesture::Click { key, .. }
                if self.show_dialog
                    && (key == KEY_DIALOG
                        || key == KEY_DIALOG_OK
                        || key == KEY_DIALOG_CANCEL) =>
            {
                commands.signal(CMD_CLOSE_DIALOG, 0);
            }
            _ if self.show_dialog => {}
            AppUiGesture::Click { key, .. } | AppUiGesture::Release { key, .. }
                if key == KEY_BACK =>
            {
                commands.signal(CMD_EXIT, 0);
            }
            AppUiGesture::Click { key, .. } if key == k(110) => {
                commands.signal(CMD_TOGGLE_WARM, 0);
            }
            AppUiGesture::Click { key, .. } if key == stepper_decrement_key(k(120)) => {
                commands.signal(CMD_LEVEL_DELTA, -32);
            }
            AppUiGesture::Click { key, .. } if key == stepper_increment_key(k(120)) => {
                commands.signal(CMD_LEVEL_DELTA, 32);
            }
            AppUiGesture::Click { key, .. } if is_demo_segment_key(key) => {
                commands.signal(CMD_SET_MODE, segment_index(key) as i32);
            }
            AppUiGesture::Swipe { key, direction, .. } if is_demo_segment_key(key) => {
                commands.signal(CMD_SET_MODE, mode_after_swipe(self.mode, direction) as i32);
            }
            AppUiGesture::Click { key, .. } if is_demo_list_item_key(key) => {
                commands.signal(CMD_SELECT_ITEM, key.0 as i32);
            }
            AppUiGesture::Drag { key, point, .. }
                if key == scroll_list_handle_key(k(40)) =>
            {
                let first = scroll_list_first_from_point(
                    settings_list,
                    DEMO_LIST_ITEMS.len(),
                    DEMO_LIST_ROW_HEIGHT,
                    point,
                );
                commands.signal(CMD_SET_LIST_FIRST, first as i32);
            }
            AppUiGesture::Swipe {
                key,
                direction: AppUiSwipe::Left,
                ..
            } if key == k(110) => {
                commands.signal(CMD_SET_WARM, 0);
            }
            AppUiGesture::Swipe {
                key,
                direction: AppUiSwipe::Right,
                ..
            } if key == k(110) => {
                commands.signal(CMD_SET_WARM, 1);
            }
            AppUiGesture::Swipe { key, direction, .. } if is_demo_list_key(key) => {
                let first = scroll_list_first_after_swipe(
                    self.list_first,
                    DEMO_LIST_ITEMS.len(),
                    list_visible,
                    direction,
                );
                commands.signal(CMD_SET_LIST_FIRST, first as i32);
            }
            _ => {}
        }

        AppUiFlow::Continue
    }

    fn view_scheduled(&self, context: AppUiContext, frame: &mut AppUiBuilder<'_, 128>) {
        compose_demo_frame(
            frame,
            context.width(),
            context.height(),
            context.tick(),
            self.theme,
            self.warm,
            self.level,
            self.list_first,
            self.mode,
            self.selected_item,
            self.reduce_motion,
            self.toast_ticks,
            self.show_dialog,
            self.last_budget,
        );
    }

    fn observe_scheduled_frame(&mut self, _context: AppUiContext, result: AppUiFrameResult) {
        self.last_budget = result.budget;
    }
}

type DemoSchedule = AppUiStagedSchedule<
    (
        fn(&mut DemoApp, AppUiContext, AppUiCommand) -> AppUiFlow,
        fn(&mut DemoApp, AppUiContext, AppUiCommand) -> AppUiFlow,
    ),
    fn(&mut DemoApp, AppUiContext, AppUiCommand) -> AppUiFlow,
>;

fn demo_schedule() -> DemoSchedule {
    AppUiStagedSchedule::new((demo_flow_system, demo_state_system), demo_tick_system)
}

fn demo_flow_system(
    _app: &mut DemoApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    match command {
        AppUiCommand::Signal(signal) if signal.is(CMD_EXIT) => AppUiFlow::Exit,
        _ => AppUiFlow::Continue,
    }
}

fn demo_state_system(
    app: &mut DemoApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    match command {
        AppUiCommand::Signal(signal) if signal.is(CMD_CLOSE_DIALOG) => {
            app.show_dialog = false;
            app.toast_ticks = 50;
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_TOGGLE_WARM) => {
            app.warm = !app.warm;
            app.toast_ticks = 38;
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_SET_WARM) => {
            app.warm = signal.value() != 0;
            app.toast_ticks = 38;
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_LEVEL_DELTA) => {
            app.level = apply_u8_delta(app.level, signal.value());
            app.toast_ticks = 38;
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_SET_MODE) => {
            app.mode = clamp_index(signal.value(), DEMO_SEGMENTS.len());
            apply_mode(app.mode, &mut app.warm, &mut app.level);
            app.toast_ticks = 42;
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_SELECT_ITEM) => {
            let selected = AppUiKey::new(signal.value().max(0) as u16);
            if is_demo_list_item_key(selected) {
                app.selected_item = selected;
                apply_list_item(selected, &mut app.warm, &mut app.level);
                if selected == DEMO_LIST_ITEMS[4].key {
                    app.show_dialog = true;
                    app.toast_ticks = 0;
                } else {
                    app.toast_ticks = 45;
                }
            }
        }
        AppUiCommand::Signal(signal) if signal.is(CMD_SET_LIST_FIRST) => {
            app.list_first = clamp_index(signal.value(), DEMO_LIST_ITEMS.len());
        }
        _ => {}
    }

    AppUiFlow::Continue
}

fn demo_tick_system(
    app: &mut DemoApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    if let AppUiCommand::Tick = command {
        app.toast_ticks = app.toast_ticks.saturating_sub(1);
    }

    AppUiFlow::Continue
}

fn top_bar_rect(width: u16) -> Rect {
    Rect::new(30, 38, width.saturating_sub(60), 48)
}

fn segment_rect(width: u16) -> Rect {
    Rect::new(42, 104, width.saturating_sub(84), 30)
}

fn list_rect(width: u16, height: u16) -> Rect {
    let y = 154;
    let bottom = (height as i32 - 194).max(y + 68);
    let list_h = (bottom - y).clamp(68, 126) as u16;
    Rect::new(42, y, width.saturating_sub(84), list_h)
}

fn is_demo_list_key(key: AppUiKey) -> bool {
    key == k(40)
        || key == scroll_list_handle_key(k(40))
        || DEMO_LIST_ITEMS.iter().any(|item| item.key == key)
}

fn is_demo_list_item_key(key: AppUiKey) -> bool {
    DEMO_LIST_ITEMS.iter().any(|item| item.key == key)
}

fn is_demo_segment_key(key: AppUiKey) -> bool {
    DEMO_SEGMENTS.iter().any(|segment| segment.key == key)
}

fn segment_index(key: AppUiKey) -> usize {
    DEMO_SEGMENTS
        .iter()
        .position(|segment| segment.key == key)
        .unwrap_or(0)
}

fn mode_after_swipe(mode: usize, direction: AppUiSwipe) -> usize {
    match direction {
        AppUiSwipe::Left => mode.saturating_add(1).min(DEMO_SEGMENTS.len() - 1),
        AppUiSwipe::Right => mode.saturating_sub(1),
        AppUiSwipe::Up | AppUiSwipe::Down => mode,
    }
}

fn clamp_index(value: i32, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (value.max(0) as usize).min(len - 1)
    }
}

fn apply_u8_delta(value: u8, delta: i32) -> u8 {
    if delta < 0 {
        value.saturating_sub(delta.unsigned_abs().min(u8::MAX as u32) as u8)
    } else {
        value.saturating_add(delta.min(u8::MAX as i32) as u8)
    }
}

fn apply_mode(mode: usize, warm: &mut bool, level: &mut u8) {
    match mode {
        1 => {
            *warm = false;
            *level = 220;
        }
        2 => {
            *warm = true;
            *level = 190;
        }
        _ => {
            *warm = false;
            *level = 150;
        }
    }
}

fn demo_background(theme: usize) -> Color {
    match theme {
        1 => Color::rgb(30, 20, 33),
        _ => Color::rgb(6, 15, 28),
    }
}

fn demo_accent(theme: usize, warm: bool, level: u8) -> Color {
    let alpha = 130 + level / 3;
    match (theme, warm) {
        (1, true) => Color::rgba(244, 145, 86, alpha),
        (1, false) => Color::rgba(220, 112, 150, alpha),
        (_, true) => Color::rgba(235, 168, 74, alpha),
        _ => Color::rgba(75, 211, 235, alpha),
    }
}

fn demo_muted(theme: usize) -> Color {
    match theme {
        1 => Color::rgba(244, 206, 218, 218),
        _ => Color::rgba(185, 224, 240, 210),
    }
}

fn level_label(level: u8) -> &'static str {
    match level {
        0..=31 => "000",
        32..=95 => "064",
        96..=159 => "128",
        160..=223 => "192",
        _ => "255",
    }
}

fn apply_list_item(key: AppUiKey, warm: &mut bool, level: &mut u8) {
    match key.0 {
        510 => {
            *warm = false;
            *level = 150;
        }
        520 => {
            *warm = false;
            *level = 220;
        }
        530 => {
            *warm = false;
            *level = 150;
        }
        540 => {
            *warm = true;
            *level = 80;
        }
        550 => {
            *warm = true;
            *level = 180;
        }
        _ => {}
    }
}

fn k(value: u16) -> AppUiKey {
    AppUiKey::new(value)
}

fn triangle_wave(value: i32, period: i32) -> u8 {
    let period = period.max(2);
    let half = period / 2;
    let mut phase = value % period;

    if phase < 0 {
        phase += period;
    }

    if phase > half {
        phase = period - phase;
    }

    ((phase * 255) / half.max(1)) as u8
}
