use fhre::{Events, MousePosition, Query, Res, ResMut, Time};

use crate::input::{ButtonInput, MouseButton};
use crate::resources::{GesturePhase, GestureState, ShellOverlayMode, ShellState, ThemeState};

const STATUS_BAR_SWIPE_ZONE_HEIGHT: f32 = 60.0;
const BOTTOM_BAR_SWIPE_Y_THRESHOLD: f32 = 440.0;

pub fn wing_gesture_system(
    mouse_pos: Res<MousePosition>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut gesture_state: ResMut<GestureState>,
    mut shell_state: ResMut<ShellState>,
    mut theme_state: ResMut<ThemeState>,
    _events: Res<Events>,
) {
    let current_pos = fhre::Vec2::new(mouse_pos.x as f32, mouse_pos.y as f32);
    let is_pressed = mouse_input.pressed(MouseButton::Left);

    match gesture_state.phase {
        GesturePhase::None => {
            if is_pressed {
                let in_status_bar_zone = is_in_status_bar_zone(current_pos);
                let in_bottom_bar_zone = is_in_bottom_bar_zone(current_pos);
                let in_gesture_zone = is_in_gesture_zone(current_pos);
                
                if in_status_bar_zone || in_bottom_bar_zone || in_gesture_zone || shell_state.overlay_visible() {
                    gesture_state.start(current_pos);
                }
            }
        }
        GesturePhase::Started | GesturePhase::Updated => {
            if is_pressed {
                gesture_state.update(current_pos);
                gesture_state.update_velocity(time.delta());
                gesture_state.update_hold_time(time.delta());
            } else {
                gesture_state.end();
                handle_gesture_end(&gesture_state, &mut shell_state, &mut theme_state);
                gesture_state.reset();
            }
        }
        GesturePhase::Ended => {
            gesture_state.reset();
        }
    }
}

fn is_in_status_bar_zone(pos: fhre::Vec2) -> bool {
    pos.y < STATUS_BAR_SWIPE_ZONE_HEIGHT
}

fn is_in_bottom_bar_zone(pos: fhre::Vec2) -> bool {
    pos.y > BOTTOM_BAR_SWIPE_Y_THRESHOLD
}

fn is_in_gesture_zone(pos: fhre::Vec2) -> bool {
    pos.y > BOTTOM_BAR_SWIPE_Y_THRESHOLD && pos.x > 120.0 && pos.x < 360.0
}

fn handle_gesture_end(
    gesture_state: &GestureState,
    shell_state: &mut ShellState,
    theme_state: &mut ThemeState,
) {
    let started_in_status_bar = gesture_state.start_position.y < STATUS_BAR_SWIPE_ZONE_HEIGHT;
    let started_in_bottom_bar = gesture_state.start_position.y > BOTTOM_BAR_SWIPE_Y_THRESHOLD;

    if gesture_state.is_long_press_triggered() {
        if started_in_status_bar {
            theme_state.switch_next_theme();
        }
        return;
    }

    if let Some(direction) = gesture_state.swipe_direction_with_velocity() {
        match shell_state.overlay_mode {
            ShellOverlayMode::None => {
                if started_in_status_bar && direction == crate::resources::SwipeDirection::Down {
                    shell_state.overlay_mode = ShellOverlayMode::QuickSettings;
                } else if started_in_bottom_bar && direction == crate::resources::SwipeDirection::Up {
                    shell_state.overlay_mode = ShellOverlayMode::AppSwitcher;
                }
            }
            ShellOverlayMode::QuickSettings => {
                if direction == crate::resources::SwipeDirection::Up {
                    shell_state.overlay_mode = ShellOverlayMode::None;
                }
            }
            ShellOverlayMode::AppSwitcher => {
                if direction == crate::resources::SwipeDirection::Down {
                    shell_state.overlay_mode = ShellOverlayMode::None;
                }
            }
        }
    }
}
