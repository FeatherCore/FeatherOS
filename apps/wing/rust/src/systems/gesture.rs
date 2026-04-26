use fhre::{Events, MousePosition, Res, ResMut, Time};

use crate::input::{ButtonInput, MouseButton};
use crate::resources::{GesturePhase, GestureState, ShellOverlayMode, ShellState, ThemeState};

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
                // Gesture can start anywhere on screen when overlay is visible
                // or when no overlay is visible (to open notification panel)
                if shell_state.overlay_visible() || !shell_state.notification_panel_open() {
                    gesture_state.start(current_pos);
                }
            }
        }
        GesturePhase::Started | GesturePhase::Updated => {
            if is_pressed {
                gesture_state.update(current_pos);
                gesture_state.update_velocity(time.delta());
                gesture_state.update_hold_time(time.delta());
                
                // Check long press (hold without movement)
                if gesture_state.hold_time >= GestureState::LONG_PRESS_THRESHOLD
                    && gesture_state.total_delta.length() < 15.0
                {
                    gesture_state.is_long_press = true;
                }
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

fn handle_gesture_end(
    gesture_state: &GestureState,
    shell_state: &mut ShellState,
    theme_state: &mut ThemeState,
) {
    // Long press: theme toggle (only in top area)
    if gesture_state.is_long_press_triggered(15.0) {
        // Only trigger if started in top 10% of screen (approximate status bar area)
        if gesture_state.start_position.y < 50.0 {
            theme_state.switch_next_theme();
        }
        return;
    }

    // Use proportional thresholds
    let swipe_threshold = 50.0;
    let velocity_threshold = 200.0;

    if let Some(direction) = swipe_direction_with_velocity(
        gesture_state,
        swipe_threshold,
        20.0,
        velocity_threshold,
    ) {
        match shell_state.overlay_mode {
            ShellOverlayMode::None => {
                // Swipe down anywhere → open NotificationPanel
                if direction == crate::resources::SwipeDirection::Down {
                    shell_state.overlay_mode = ShellOverlayMode::NotificationPanel;
                }
                // Swipe up anywhere → open AppSwitcher
                if direction == crate::resources::SwipeDirection::Up {
                    shell_state.overlay_mode = ShellOverlayMode::AppSwitcher;
                }
            }
            ShellOverlayMode::NotificationPanel => {
                // Swipe up → close NotificationPanel
                if direction == crate::resources::SwipeDirection::Up {
                    shell_state.overlay_mode = ShellOverlayMode::None;
                }
            }
            ShellOverlayMode::AppSwitcher => {
                // Swipe down → close AppSwitcher
                if direction == crate::resources::SwipeDirection::Down {
                    shell_state.overlay_mode = ShellOverlayMode::None;
                }
            }
        }
    }
}

fn swipe_direction_with_velocity(
    gesture_state: &GestureState,
    swipe_threshold: f32,
    min_swipe_for_velocity: f32,
    velocity_threshold: f32,
) -> Option<crate::resources::SwipeDirection> {
    let total_delta = gesture_state.total_delta;
    let is_horizontal = total_delta.x.abs() > swipe_threshold
        && total_delta.x.abs() > total_delta.y.abs() * 2.0;
    let is_vertical = total_delta.y.abs() > swipe_threshold
        && total_delta.y.abs() > total_delta.x.abs() * 2.0;
    
    if !is_horizontal && !is_vertical {
        return None;
    }
    
    let direction = if is_horizontal {
        if total_delta.x > 0.0 {
            crate::resources::SwipeDirection::Right
        } else {
            crate::resources::SwipeDirection::Left
        }
    } else {
        if total_delta.y > 0.0 {
            crate::resources::SwipeDirection::Down
        } else {
            crate::resources::SwipeDirection::Up
        }
    };
    
    // Check velocity threshold
    let velocity = gesture_state.average_velocity;
    let is_fast_swipe = match direction {
        crate::resources::SwipeDirection::Up => velocity.y < -velocity_threshold,
        crate::resources::SwipeDirection::Down => velocity.y > velocity_threshold,
        crate::resources::SwipeDirection::Left => velocity.x < -velocity_threshold,
        crate::resources::SwipeDirection::Right => velocity.x > velocity_threshold,
    };
    
    let meets_displacement = total_delta.y.abs() > swipe_threshold * 0.6
        || total_delta.x.abs() > swipe_threshold * 0.6;
    
    if is_fast_swipe && total_delta.y.abs() > min_swipe_for_velocity {
        Some(direction)
    } else if meets_displacement {
        Some(direction)
    } else {
        None
    }
}
