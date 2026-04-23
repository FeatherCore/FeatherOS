use fhre::{Click, Events, Pointer, Query, Res, ResMut};

use crate::components::{WindowControlButton, WindowControlKind, WindowFocus, WindowFrame, WindowTitleBar};
use crate::resources::{FocusState, WindowManagerState};
use crate::types::WindowState;

pub fn wing_minimal_window_focus_system(
    events: Res<Events>,
    mut focus_state: ResMut<FocusState>,
    mut window_manager_state: ResMut<WindowManagerState>,
    mut focus_query: Query<&mut WindowFocus>,
    mut window_query: Query<&WindowFrame>,
    mut title_bar_query: Query<&WindowTitleBar>,
) {
    let mut focused_window = focus_state.window_id;

    if let Some(click_events) = events.get_events_current::<Pointer<Click>>() {
        for event in click_events {
            if let Some((_, window)) = window_query.get_pair(event.entity) {
                focused_window = Some(window.id);
                window_manager_state.bring_to_front(window.id);
                continue;
            }
            if let Some((_, title_bar)) = title_bar_query.get_pair(event.entity) {
                focused_window = Some(title_bar.window_id);
                window_manager_state.bring_to_front(title_bar.window_id);
            }
        }
    }

    focus_state.window_id = focused_window;
    focus_state.widget_id = None;
    window_manager_state.active_window = focused_window;

    for (_, focus) in focus_query.iter_mut() {
        focus.focused = false;
    }

    if let Some(window_id) = focused_window {
        for (entity, window) in window_query.iter() {
            if window.id == window_id {
                if let Some((_, focus)) = focus_query.get_pair_mut(entity) {
                    focus.focused = true;
                }
            }
        }
    }
}

pub fn wing_window_control_system(
    events: Res<Events>,
    mut window_manager_state: ResMut<WindowManagerState>,
    mut window_query: Query<&mut WindowFrame>,
    mut control_query: Query<&WindowControlButton>,
) {
    if let Some(click_events) = events.get_events_current::<Pointer<Click>>() {
        for event in click_events {
            let Some((_, control)) = control_query.get_pair(event.entity) else {
                continue;
            };

            for (_, window) in window_query.iter_mut() {
                if window.id != control.window_id {
                    continue;
                }

                match control.kind {
                    WindowControlKind::Close => {
                        window.state = WindowState::Closed;
                        if window_manager_state.active_window == Some(window.id) {
                            window_manager_state.active_window = None;
                        }
                        if window_manager_state.dragging_window == Some(window.id) {
                            window_manager_state.dragging_window = None;
                        }
                    }
                    WindowControlKind::Minimize => {
                        window.state = WindowState::Minimized;
                        if window_manager_state.active_window == Some(window.id) {
                            window_manager_state.active_window = None;
                        }
                    }
                    WindowControlKind::Maximize => {
                        window.state = if window.state == WindowState::Maximized {
                            WindowState::Normal
                        } else {
                            WindowState::Maximized
                        };
                        window_manager_state.active_window = Some(window.id);
                        window_manager_state.bring_to_front(window.id);
                    }
                }
            }
        }
    }
}
