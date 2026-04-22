use fhre::ResMut;

use crate::{
    DesktopMetrics, DragTransaction, FocusState, LauncherState, LayoutInvalidation,
    TaskbarState, TextInputState, ThemeState, WindowManagerState, WingDesktopState,
    WingRuntime,
};

pub fn wing_shell_state_sync_system(
    mut runtime: ResMut<WingRuntime>,
    mut desktop_state: ResMut<WingDesktopState>,
    mut launcher_state: ResMut<LauncherState>,
    mut taskbar_state: ResMut<TaskbarState>,
    mut theme_state: ResMut<ThemeState>,
    mut desktop_metrics: ResMut<DesktopMetrics>,
    mut layout_invalidation: ResMut<LayoutInvalidation>,
) {
    let runtime = &mut *runtime;
    let wing = &runtime.wing;

    desktop_state.initialized = true;

    launcher_state.open = wing.launcher.is_open();
    taskbar_state.launcher_open = wing.launcher.is_open();

    theme_state.current = wing.theme;

    desktop_metrics.screen_size = wing.screen_size;
    desktop_metrics.taskbar_height = wing.taskbar.height();

    // Keep layout clean until dedicated ECS layout invalidation is introduced.
    layout_invalidation.pending = false;
}

pub fn wing_window_state_sync_system(
    mut runtime: ResMut<WingRuntime>,
    mut window_manager_state: ResMut<WindowManagerState>,
    mut focus_state: ResMut<FocusState>,
    mut text_input_state: ResMut<TextInputState>,
    mut drag_state: ResMut<DragTransaction>,
) {
    let runtime = &mut *runtime;
    let window_manager = &runtime.wing.window_manager;

    window_manager_state.active_window = window_manager.focused_window();
    window_manager_state.dragging_window = window_manager.dragging_window();
    window_manager_state.sync_from_order(window_manager.window_order());

    focus_state.window_id = window_manager.focused_window();
    focus_state.widget_id = None;

    text_input_state.active_window_id = window_manager.focused_window();
    text_input_state.active_widget_id = text_input_state
        .active_window_id
        .and_then(|window_id| window_manager.focused_text_widget_id(window_id));

    drag_state.window_id = window_manager.dragging_window();
    if !drag_state.active {
        drag_state.origin_x = runtime.pointer_pos.x;
        drag_state.origin_y = runtime.pointer_pos.y;
    }
}
