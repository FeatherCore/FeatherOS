//! Wing desktop shell demo running on FHRE.

#![no_std]
#![no_main]

mod extract;
mod platform;

extern crate alloc;

use fhre::{App, DefaultPlugins, MousePosition, Res, ResMut, Update, Window as _, declare_system};
use platform::framebuffer;
use platform::input::{ButtonInput, KeyCode, MouseButton};
use platform::runner::PlatformInputPlugin;
use wing::{TextEditCommand, Wing};

fn keycode_to_char(key: KeyCode) -> Option<char> {
    match key {
        KeyCode::Space => Some(' '),
        KeyCode::KeyA => Some('a'), KeyCode::KeyB => Some('b'), KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'), KeyCode::KeyE => Some('e'), KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'), KeyCode::KeyH => Some('h'), KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'), KeyCode::KeyK => Some('k'), KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'), KeyCode::KeyN => Some('n'), KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'), KeyCode::KeyQ => Some('q'), KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'), KeyCode::KeyT => Some('t'), KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'), KeyCode::KeyW => Some('w'), KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'), KeyCode::KeyZ => Some('z'),
        _ => None,
    }
}

fn keycode_to_edit_command(key: KeyCode) -> Option<TextEditCommand> {
    match key {
        KeyCode::Backspace => Some(TextEditCommand::Backspace),
        KeyCode::Delete => Some(TextEditCommand::Delete),
        KeyCode::Enter => Some(TextEditCommand::Newline),
        KeyCode::ArrowLeft => Some(TextEditCommand::MoveLeft),
        KeyCode::ArrowRight => Some(TextEditCommand::MoveRight),
        KeyCode::ArrowUp => Some(TextEditCommand::MoveUp),
        KeyCode::ArrowDown => Some(TextEditCommand::MoveDown),
        KeyCode::Home => Some(TextEditCommand::MoveLineStart),
        KeyCode::End => Some(TextEditCommand::MoveLineEnd),
        _ => keycode_to_char(key).map(TextEditCommand::Insert),
    }
}

const FRAME_DELAY_MS: u32 = 16;

pub struct WingRuntime {
    pub wing: Wing,
    pointer_pos: fhre::Vec2,
    drag_active: bool,
}

impl WingRuntime {
    pub fn new(width: f32, height: f32) -> Self {
        let mut wing = Wing::new(width, height);
        wing.init();
        Self {
            wing,
            pointer_pos: fhre::Vec2::ZERO,
            drag_active: false,
        }
    }
}

impl fhre::resources::Resource for WingRuntime {}

fn wing_input_system(
    mouse_pos: Res<MousePosition>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut runtime: ResMut<WingRuntime>,
) {
    let current = fhre::Vec2::new(mouse_pos.x as f32, mouse_pos.y as f32);
    runtime.wing.handle_mouse_move(current);

    if mouse_input.just_pressed(MouseButton::Left) {
        runtime.drag_active = true;
        runtime.pointer_pos = current;
        runtime.wing.handle_click(current);
    } else if mouse_input.pressed(MouseButton::Left) && runtime.drag_active {
        let delta = fhre::Vec2::new(
            current.x - runtime.pointer_pos.x,
            current.y - runtime.pointer_pos.y,
        );
        if delta.x != 0.0 || delta.y != 0.0 {
            runtime.wing.handle_drag(delta);
            runtime.pointer_pos = current;
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        runtime.drag_active = false;
        runtime.pointer_pos = current;
        runtime.wing.handle_mouse_release(current);
    }

    if key_input.just_pressed(KeyCode::Space) {
        runtime.wing.launcher.toggle();
        let launcher_open = runtime.wing.launcher.is_open();
        runtime
            .wing
            .taskbar
            .set_launcher_open(launcher_open);
    }

    if key_input.just_pressed(KeyCode::Escape) {
        if runtime.wing.launcher.is_open() {
            runtime.wing.launcher.close();
            runtime.wing.taskbar.set_launcher_open(false);
        } else if let Some(window_id) = runtime.wing.window_manager.focused_window() {
            runtime.wing.close_window(window_id);
        }
    }

    let text_keys = [
        KeyCode::Backspace, KeyCode::Delete, KeyCode::Enter,
        KeyCode::ArrowLeft, KeyCode::ArrowRight, KeyCode::ArrowUp, KeyCode::ArrowDown,
        KeyCode::Home, KeyCode::End,
        KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD,
        KeyCode::KeyE, KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::KeyI, KeyCode::KeyJ,
        KeyCode::KeyK, KeyCode::KeyL, KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO, KeyCode::KeyP,
        KeyCode::KeyQ, KeyCode::KeyR, KeyCode::KeyS, KeyCode::KeyT, KeyCode::KeyU, KeyCode::KeyV,
        KeyCode::KeyW, KeyCode::KeyX, KeyCode::KeyY, KeyCode::KeyZ, KeyCode::Space,
    ];
    for key in text_keys {
        if key_input.just_pressed(key) {
            if let Some(command) = keycode_to_edit_command(key) {
                runtime.wing.handle_text_edit(command);
            }
        }
    }
}

fn wing_update_system(mut runtime: ResMut<WingRuntime>) {
    runtime.wing.update(1.0 / 60.0);
}

#[no_mangle]
pub extern "C" fn wing_rust_main() -> i32 {
    let mut window = match framebuffer::Window::new() {
        Some(window) => window,
        None => return 0,
    };

    let (width, height) = window.dimensions();
    let mut app = App::new(width, height);

    app.add_plugins(DefaultPlugins)
        .insert_resource(WingRuntime::new(width as f32, height as f32))
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(MousePosition::default())
        .add_systems(Update, declare_system!(wing_input_system; Res<MousePosition>, Res<ButtonInput<MouseButton>>, Res<ButtonInput<KeyCode>>, ResMut<WingRuntime>))
        .add_systems(Update, declare_system!(wing_update_system; ResMut<WingRuntime>));

    app.add_extractor(extract::queue_wing_shell);

    let input_plugin = PlatformInputPlugin::new(framebuffer::InputAdapter);
    app.run(&mut window, &input_plugin, FRAME_DELAY_MS);
    0
}

#[no_mangle]
pub extern "C" fn rust_wing_demo_init() {}
