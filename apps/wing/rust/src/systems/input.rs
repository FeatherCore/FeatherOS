use fhre::{MousePosition, Res, ResMut, Vec2};

use crate::{ButtonInput, DragTransaction, FocusState, KeyCode, LauncherState, MouseButton, MouseWheel, TaskbarState, TextEditCommand, TextInputState, WindowManagerState, WingRuntime};

const TEXT_KEYS: [KeyCode; 36] = [
    KeyCode::Backspace, KeyCode::Delete, KeyCode::Enter,
    KeyCode::ArrowLeft, KeyCode::ArrowRight, KeyCode::ArrowUp, KeyCode::ArrowDown,
    KeyCode::Home, KeyCode::End,
    KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD,
    KeyCode::KeyE, KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::KeyI, KeyCode::KeyJ,
    KeyCode::KeyK, KeyCode::KeyL, KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO, KeyCode::KeyP,
    KeyCode::KeyQ, KeyCode::KeyR, KeyCode::KeyS, KeyCode::KeyT, KeyCode::KeyU, KeyCode::KeyV,
    KeyCode::KeyW, KeyCode::KeyX, KeyCode::KeyY, KeyCode::KeyZ, KeyCode::Space,
];

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
        KeyCode::Escape => Some(TextEditCommand::Cancel),
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

pub fn wing_pointer_input_system(
    mouse_pos: Res<MousePosition>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mouse_wheel: Res<MouseWheel>,
    mut drag_state: ResMut<DragTransaction>,
    mut runtime: ResMut<WingRuntime>,
) {
    let current = Vec2::new(mouse_pos.x as f32, mouse_pos.y as f32);
    runtime.wing.handle_mouse_move(current);

    if mouse_input.just_pressed(MouseButton::Left) {
        drag_state.active = true;
        drag_state.origin_x = current.x;
        drag_state.origin_y = current.y;
        runtime.pointer_pos = current;
        runtime.wing.handle_click(current);
        drag_state.window_id = runtime.wing.window_manager.dragging_window();
    } else if mouse_input.pressed(MouseButton::Left) && drag_state.active {
        let delta = Vec2::new(
            current.x - runtime.pointer_pos.x,
            current.y - runtime.pointer_pos.y,
        );
        if delta.x != 0.0 || delta.y != 0.0 {
            runtime.wing.handle_drag(delta);
            runtime.pointer_pos = current;
            drag_state.window_id = runtime.wing.window_manager.dragging_window();
        }
    }

    if mouse_input.just_released(MouseButton::Left) {
        drag_state.active = false;
        drag_state.window_id = None;
        drag_state.origin_x = current.x;
        drag_state.origin_y = current.y;
        runtime.pointer_pos = current;
        runtime.wing.handle_mouse_release(current);
    }

    if mouse_wheel.delta != 0 {
        runtime.wing.handle_scroll(current, mouse_wheel.delta);
    }
}

pub fn wing_shell_shortcut_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut focus_state: ResMut<FocusState>,
    mut launcher_state: ResMut<LauncherState>,
    mut taskbar_state: ResMut<TaskbarState>,
    mut window_manager_state: ResMut<WindowManagerState>,
    mut runtime: ResMut<WingRuntime>,
) {
    if key_input.just_pressed(KeyCode::Space) {
        runtime.wing.launcher.toggle();
        let launcher_open = runtime.wing.launcher.is_open();
        runtime.wing.taskbar.set_launcher_open(launcher_open);
        launcher_state.open = launcher_open;
        taskbar_state.launcher_open = launcher_open;
    }

    if key_input.just_pressed(KeyCode::Escape) {
        if runtime.wing.launcher.is_open() {
            runtime.wing.launcher.close();
            runtime.wing.taskbar.set_launcher_open(false);
            launcher_state.open = false;
            taskbar_state.launcher_open = false;
        } else if !runtime.wing.handle_text_edit(TextEditCommand::Cancel) {
            if let Some(window_id) = runtime.wing.window_manager.focused_window() {
                runtime.wing.close_window(window_id);
                focus_state.window_id = runtime.wing.window_manager.focused_window();
                focus_state.widget_id = None;
                window_manager_state.active_window = runtime.wing.window_manager.focused_window();
                window_manager_state.dragging_window = runtime.wing.window_manager.dragging_window();
                window_manager_state.sync_from_order(runtime.wing.window_manager.window_order());
            }
        }
    }
}

pub fn wing_text_input_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut text_input_state: ResMut<TextInputState>,
    mut runtime: ResMut<WingRuntime>,
) {
    text_input_state.active_window_id = runtime.wing.window_manager.focused_window();
    text_input_state.active_widget_id = text_input_state
        .active_window_id
        .and_then(|window_id| runtime.wing.window_manager.focused_text_widget_id(window_id));

    for key in TEXT_KEYS {
        if key_input.just_pressed(key) {
            if let Some(command) = keycode_to_edit_command(key) {
                if runtime.wing.handle_text_edit(command) {
                    text_input_state.active_window_id = runtime.wing.window_manager.focused_window();
                    text_input_state.active_widget_id = text_input_state
                        .active_window_id
                        .and_then(|window_id| runtime.wing.window_manager.focused_text_widget_id(window_id));
                }
            }
        }
    }

    if text_input_state.active_window_id.is_none() {
        text_input_state.active_widget_id = None;
    }
}
