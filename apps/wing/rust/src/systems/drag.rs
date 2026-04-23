use fhre::{Events, MousePosition, Pointer, Press, Query, Res, ResMut, Transform};

use crate::components::{WidgetLayoutNode, WindowChrome, WindowContentRoot, WindowContentText, WindowControlButton, WindowControlKind, WindowFrame, WindowPlacement, WindowTitleBar, WindowTitleIcon, WindowTitleText};
use crate::resources::{DesktopMetrics, DragTransaction, WindowManagerState};
use crate::types::WindowState;

pub fn wing_window_drag_system(
    mouse_pos: Res<MousePosition>,
    mouse_input: Res<crate::ButtonInput<crate::MouseButton>>,
    events: Res<Events>,
    mut drag_state: ResMut<DragTransaction>,
    mut window_manager_state: ResMut<WindowManagerState>,
    mut transform_query: Query<&mut Transform>,
    mut window_query: Query<&WindowFrame>,
    mut placement_query: Query<&mut WindowPlacement>,
    mut title_bar_query: Query<&WindowTitleBar>,
    mut chrome_query: Query<&WindowChrome>,
) {
    if let Some(press_events) = events.get_events_current::<Pointer<Press>>() {
        for event in press_events {
            let Some((_, title_bar)) = title_bar_query.get_pair(event.entity) else {
                continue;
            };
            let visible = window_query
                .iter()
                .find(|(_, window)| window.id == title_bar.window_id)
                .map(|(_, window)| matches!(window.state, WindowState::Normal))
                .unwrap_or(false);
            if !visible {
                continue;
            }
            let draggable = chrome_query
                .get_pair_mut(event.entity)
                .map(|(_, chrome)| chrome.draggable)
                .unwrap_or(false);
            if !draggable {
                continue;
            }

            drag_state.active = true;
            drag_state.window_id = Some(title_bar.window_id);
            drag_state.origin_x = mouse_pos.x as f32;
            drag_state.origin_y = mouse_pos.y as f32;
            window_manager_state.dragging_window = Some(title_bar.window_id);
            window_manager_state.bring_to_front(title_bar.window_id);
        }
    }

    if drag_state.active && mouse_input.pressed(crate::MouseButton::Left) {
        let current_x = mouse_pos.x as f32;
        let current_y = mouse_pos.y as f32;
        let delta_x = current_x - drag_state.origin_x;
        let delta_y = current_y - drag_state.origin_y;

        if (delta_x != 0.0 || delta_y != 0.0) && drag_state.window_id.is_some() {
            let window_id = drag_state.window_id;
            for (entity, window) in window_query.iter() {
                if Some(window.id) != window_id {
                    continue;
                }
                if window.state != WindowState::Normal {
                    continue;
                }
                if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
                    transform.translate_2d(delta_x, delta_y);
                }
                if let Some((_, placement)) = placement_query.get_pair_mut(entity) {
                    placement.normal_center_x += delta_x;
                    placement.normal_center_y += delta_y;
                }
            }
            for (entity, title_bar) in title_bar_query.iter() {
                if Some(title_bar.window_id) != window_id {
                    continue;
                }
                if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
                    transform.translate_2d(delta_x, delta_y);
                }
            }
            drag_state.origin_x = current_x;
            drag_state.origin_y = current_y;
        }
    }

    if drag_state.active && !mouse_input.pressed(crate::MouseButton::Left) {
        drag_state.active = false;
        drag_state.window_id = None;
        window_manager_state.dragging_window = None;
        drag_state.origin_x = mouse_pos.x as f32;
        drag_state.origin_y = mouse_pos.y as f32;
    }
}

pub fn wing_window_layout_system(
    desktop_metrics: Res<DesktopMetrics>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut layout_query: Query<&WidgetLayoutNode>,
    mut window_query: Query<&mut WindowFrame>,
    mut placement_query: Query<&mut WindowPlacement>,
    mut title_bar_query: Query<&WindowTitleBar>,
    mut content_query: Query<&WindowContentRoot>,
) {
    for (window_entity, window) in window_query.iter_mut() {
        let Some((_, window_layout)) = layout_query.get_pair_mut(window_entity) else { continue; };

        let mut window_center_x = 0.0;
        let mut window_center_y = 0.0;
        let mut window_width = window_layout.width;
        let mut window_height = window_layout.height;
        let mut visible = true;

        if let Some((_, placement)) = placement_query.get_pair_mut(window_entity) {
            match window.state {
                WindowState::Normal => {
                    window_center_x = placement.normal_center_x;
                    window_center_y = placement.normal_center_y;
                    window_width = placement.normal_width;
                    window_height = placement.normal_height;
                }
                WindowState::Maximized => {
                    window_center_x = desktop_metrics.screen_size.x * 0.5;
                    window_center_y = (desktop_metrics.screen_size.y - desktop_metrics.taskbar_height) * 0.5;
                    window_width = desktop_metrics.screen_size.x - 32.0;
                    window_height = desktop_metrics.screen_size.y - desktop_metrics.taskbar_height - 32.0;
                }
                WindowState::Minimized | WindowState::Closed => {
                    visible = false;
                    window_center_x = placement.normal_center_x;
                    window_center_y = placement.normal_center_y;
                    window_width = 0.0;
                    window_height = 0.0;
                }
            }
        }

        if let Some((_, window_transform)) = transform_query.get_pair_mut(window_entity) {
            window_transform.position.x = window_center_x;
            window_transform.position.y = window_center_y;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(window_entity) {
            bounds.width = window_width;
            bounds.height = window_height;
        }

        let title_center_y = window_center_y - (window_height * 0.5) + 14.0;
        let content_center_y = window_center_y + 10.0;

        for (title_entity, title_bar) in title_bar_query.iter() {
            if title_bar.window_id != window.id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(title_entity) {
                transform.position.x = window_center_x;
                transform.position.y = title_center_y;
            }
            if let Some((_, bounds)) = bounds_query.get_pair_mut(title_entity) {
                bounds.width = if visible { window_width } else { 0.0 };
                bounds.height = if visible { 28.0 } else { 0.0 };
            }
        }

        for (content_entity, content_root) in content_query.iter() {
            if content_root.window_id != window.id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(content_entity) {
                transform.position.x = window_center_x;
                transform.position.y = content_center_y;
            }
            if let Some((_, bounds)) = bounds_query.get_pair_mut(content_entity) {
                bounds.width = if visible { (window_width - 24.0).max(0.0) } else { 0.0 };
                bounds.height = if visible { (window_height - 40.0).max(0.0) } else { 0.0 };
            }
        }
    }
}

pub fn wing_window_text_layout_system(
    mut transform_query: Query<&mut Transform>,
    mut layout_query: Query<&WidgetLayoutNode>,
    mut window_query: Query<&WindowFrame>,
    mut title_icon_query: Query<&WindowTitleIcon>,
    mut title_text_query: Query<&WindowTitleText>,
    mut content_text_query: Query<&WindowContentText>,
) {
    for (window_entity, window) in window_query.iter() {
        let Some((_, window_transform)) = transform_query.get_pair_mut(window_entity) else {
            continue;
        };
        let Some((_, window_layout)) = layout_query.get_pair_mut(window_entity) else {
            continue;
        };

        let title_center_y = window_transform.position.y - (window_layout.height * 0.5) + 14.0;
        let title_icon_x = window_transform.position.x - (window_layout.width * 0.5) + 12.0;
        let title_text_x = window_transform.position.x - (window_layout.width * 0.5) + 28.0;
        let content_center_y = window_transform.position.y + 10.0;

        for (icon_entity, title_icon) in title_icon_query.iter() {
            if title_icon.window_id != window.id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(icon_entity) {
                transform.position.x = title_icon_x;
                transform.position.y = title_center_y;
            }
        }

        for (text_entity, title_text) in title_text_query.iter() {
            if title_text.window_id != window.id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(text_entity) {
                transform.position.x = title_text_x;
                transform.position.y = title_center_y;
            }
        }

        for (content_text_entity, content_text) in content_text_query.iter() {
            if content_text.window_id != window.id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(content_text_entity) {
                transform.position.x = window_transform.position.x;
                transform.position.y = content_center_y;
            }
        }
    }
}

pub fn wing_window_control_layout_system(
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut layout_query: Query<&WidgetLayoutNode>,
    mut window_query: Query<&WindowFrame>,
    mut control_query: Query<&WindowControlButton>,
) {
    for (window_entity, window) in window_query.iter() {
        let Some((_, window_transform)) = transform_query.get_pair_mut(window_entity) else {
            continue;
        };
        let Some((_, window_layout)) = layout_query.get_pair_mut(window_entity) else {
            continue;
        };

        let visible = !matches!(window.state, WindowState::Minimized | WindowState::Closed);
        let title_center_y = window_transform.position.y - (window_layout.height * 0.5) + 14.0;
        let control_xs = [
            window_transform.position.x + 104.0,
            window_transform.position.x + 124.0,
            window_transform.position.x + 144.0,
        ];

        for (control_entity, control) in control_query.iter() {
            if control.window_id != window.id {
                continue;
            }
            let offset_index = match control.kind {
                WindowControlKind::Minimize => 0,
                WindowControlKind::Maximize => 1,
                WindowControlKind::Close => 2,
            };
            if let Some((_, transform)) = transform_query.get_pair_mut(control_entity) {
                transform.position.x = control_xs[offset_index];
                transform.position.y = title_center_y;
            }
            if let Some((_, bounds)) = bounds_query.get_pair_mut(control_entity) {
                bounds.width = if visible { 14.0 } else { 0.0 };
                bounds.height = if visible { 14.0 } else { 0.0 };
            }
        }
    }
}

pub fn wing_window_layer_system(
    window_manager_state: Res<WindowManagerState>,
    mut transform_query: Query<&mut Transform>,
    mut window_query: Query<&WindowFrame>,
    mut title_bar_query: Query<&WindowTitleBar>,
    mut title_icon_query: Query<&WindowTitleIcon>,
    mut title_text_query: Query<&WindowTitleText>,
    mut content_query: Query<&WindowContentRoot>,
    mut content_text_query: Query<&WindowContentText>,
    mut control_query: Query<&WindowControlButton>,
) {
    for (index, window_id) in window_manager_state.window_order.iter().enumerate() {
        for (entity, window) in window_query.iter() {
            if window.id != *window_id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
                transform.position.z = 0.10 + index as f32 * 0.01;
            }
        }
        for (entity, title_bar) in title_bar_query.iter() {
            if title_bar.window_id != *window_id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
                transform.position.z = 0.11 + index as f32 * 0.01;
            }
        }
        for (entity, title_text) in title_text_query.iter() {
            if title_text.window_id != *window_id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
                transform.position.z = 0.115 + index as f32 * 0.01;
            }
        }
        for (entity, title_icon) in title_icon_query.iter() {
            if title_icon.window_id != *window_id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
                transform.position.z = 0.114 + index as f32 * 0.01;
            }
        }
        for (entity, content_root) in content_query.iter() {
            if content_root.window_id != *window_id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
                transform.position.z = 0.105 + index as f32 * 0.01;
            }
        }
        for (entity, content_text) in content_text_query.iter() {
            if content_text.window_id != *window_id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
                transform.position.z = 0.106 + index as f32 * 0.01;
            }
        }
        for (entity, control) in control_query.iter() {
            if control.window_id != *window_id {
                continue;
            }
            if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
                transform.position.z = 0.12 + index as f32 * 0.01;
            }
        }
    }
}
