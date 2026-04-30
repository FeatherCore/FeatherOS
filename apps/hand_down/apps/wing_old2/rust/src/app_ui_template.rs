use ::core::ffi::c_char;

use crate::app_ui_sdk::prelude::*;

pub const APP_UI_TEMPLATE_CAPACITY: usize = 64;

const KEY_BACK: AppUiKey = AppUiKey::new(100);
const CMD_EXIT: AppUiSignalId = AppUiSignalId::new(900);

pub fn run_app_ui_template(argc: i32, argv: *mut *mut c_char) -> i32 {
    let mut app = TemplateApp::new();

    unsafe {
        run_app_ui_rgb565_from_argv::<APP_UI_TEMPLATE_CAPACITY, _>(
            argc,
            argv,
            Color::rgb(7, 11, 18),
            AppUiFramePacing::from_hz(30),
            &mut app,
        )
    }
}

struct TemplateApp {
    label: &'static str,
}

impl TemplateApp {
    const fn new() -> Self {
        Self { label: "READY" }
    }
}

impl AppUiScheduledApp<APP_UI_TEMPLATE_CAPACITY> for TemplateApp {
    type Schedule = TemplateSchedule;

    fn schedule(&mut self) -> Self::Schedule {
        template_schedule()
    }

    fn enqueue_scheduled_event(
        &mut self,
        _context: AppUiContext,
        gesture: AppUiGesture,
        commands: &mut AppUiCommandQueue,
    ) -> AppUiFlow {
        if let AppUiGesture::Click { key, .. } = gesture {
            if key == KEY_BACK {
                commands.signal(CMD_EXIT, 0);
            }
        }

        AppUiFlow::Continue
    }

    fn view_scheduled(
        &self,
        context: AppUiContext,
        frame: &mut AppUiBuilder<'_, APP_UI_TEMPLATE_CAPACITY>,
    ) {
        let width = context.width();
        let height = context.height();
        let panel = Rect::new(18, 18, width.saturating_sub(36), height.saturating_sub(36));
        let status = Rect::new(30, 94, width.saturating_sub(60), 44);
        let accent = Color::rgb(80, 220, 220);

        frame.add((
            AppUiRect {
                key: AppUiKey::new(1),
                rect: Rect::new(0, 0, width, height),
                z: 0,
                color: Color::rgb(7, 11, 18),
            },
            AppUiRoundRect {
                key: AppUiKey::new(2),
                rect: panel,
                z: 1,
                radius: 18,
                color: Color::rgba(255, 255, 255, 24),
            },
            AppUiTopBar {
                key: AppUiKey::new(10),
                rect: Rect::new(26, 28, width.saturating_sub(52), 48),
                z: 4,
                title: "APP TEMPLATE",
                subtitle: "SDK PRELUDE",
                back_key: Some(KEY_BACK),
                icon: Some(VectorIcon::Wing),
                style: AppUiTopBarStyle::new(
                    Color::rgba(255, 255, 255, 26),
                    Color::WHITE,
                    Color::rgba(190, 225, 230, 220),
                    Color::rgba(255, 255, 255, 42),
                    Color::WHITE,
                ),
            },
            AppUiStatusRow {
                key: AppUiKey::new(20),
                rect: status,
                z: 4,
                title: "APP STATE",
                value: self.label,
                progress: 180,
                icon: Some(VectorIcon::Preview),
                style: AppUiRowStyle::new(
                    Color::rgba(255, 255, 255, 24),
                    Color::WHITE,
                    Color::rgba(190, 225, 230, 220),
                    Color::rgba(255, 255, 255, 24),
                    accent,
                    Color::rgba(255, 255, 255, 36),
                    Color::rgba(255, 255, 255, 44),
                    Color::WHITE,
                ),
            },
        ));

        frame.add(AppUiLabel {
            key: AppUiKey::new(30),
            x: 32,
            y: height as i32 - 28,
            z: 4,
            text: "NO MACRO APPUI",
            color: Color::rgba(190, 225, 230, 220),
            scale: 1,
        });
    }

    fn frame_hint_scheduled(&self, _context: AppUiContext) -> AppUiFrameHint {
        AppUiFrameHint::idle_default()
    }
}

type TemplateSchedule = AppUiStagedSchedule<
    fn(&mut TemplateApp, AppUiContext, AppUiCommand) -> AppUiFlow,
    (),
>;

fn template_schedule() -> TemplateSchedule {
    AppUiStagedSchedule::new(template_flow_system, ())
}

fn template_flow_system(
    _app: &mut TemplateApp,
    _context: AppUiContext,
    command: AppUiCommand,
) -> AppUiFlow {
    match command {
        AppUiCommand::Signal(signal) if signal.is(CMD_EXIT) => AppUiFlow::Exit,
        _ => AppUiFlow::Continue,
    }
}
