//! Wing desktop shell demo running on FHRE.

#![no_std]
#![no_main]

mod platform;

extern crate alloc;

use fhre::{App, DefaultPlugins, MousePosition, Window as _};
use platform::framebuffer;
use platform::runner::PlatformInputPlugin;
use wing::{ButtonInput, DesktopMetrics, DragTransaction, FocusState, KeyCode, LauncherState, LayoutInvalidation, MouseButton, MouseWheel, SelectionState, TaskbarState, TextInputState, ThemeState, WindowManagerState, WingDesktopPlugin, WingDesktopState};

const FRAME_DELAY_MS: u32 = 16;

#[no_mangle]
pub extern "C" fn wing_rust_main() -> i32 {
    let mut window = match framebuffer::Window::new() {
        Some(window) => window,
        None => return 0,
    };

    let (width, height) = window.dimensions();
    let mut app = App::new(width, height);

    app.add_plugins(DefaultPlugins)
        .add_plugin(WingDesktopPlugin)
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(MouseWheel::default())
        .insert_resource(MousePosition::default())
        .insert_resource(WingDesktopState::default())
        .insert_resource(WindowManagerState::default())
        .insert_resource(DragTransaction::default())
        .insert_resource(FocusState::default())
        .insert_resource(SelectionState::default())
        .insert_resource(TextInputState::default())
        .insert_resource(LauncherState::default())
        .insert_resource(TaskbarState::default())
        .insert_resource(LayoutInvalidation::default())
        .insert_resource(ThemeState::default())
        .insert_resource(DesktopMetrics::new(fhre::Vec2::new(width as f32, height as f32), 48.0, 16.0));

    let input_plugin = PlatformInputPlugin::new(framebuffer::InputAdapter);
    app.run(&mut window, &input_plugin, FRAME_DELAY_MS);
    0
}

#[no_mangle]
pub extern "C" fn rust_wing_demo_init() {}
