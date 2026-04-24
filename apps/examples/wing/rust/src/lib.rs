//! Wing shell demo running on FHRE.

#![no_std]
#![no_main]

mod platform;

extern crate alloc;

use fhre::{App, DefaultPlugins, MousePosition, Window as _};
use platform::framebuffer;
use platform::runner::PlatformInputPlugin;
use wing::{ButtonInput, KeyCode, MouseButton, ThemeState, WingShellPlugin};

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
        .add_plugin(WingShellPlugin)
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(MousePosition::default())
        .insert_resource(ThemeState::default())
        .insert_resource(wing::ShellMetrics::new(fhre::Vec2::new(width as f32, height as f32)));

    let input_plugin = PlatformInputPlugin::new(framebuffer::InputAdapter);
    app.run(&mut window, &input_plugin, FRAME_DELAY_MS);
    0
}

#[no_mangle]
pub extern "C" fn rust_wing_demo_init() {}
