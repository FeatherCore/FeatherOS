use fhre::{Commands, Events, Pickable, PickableBounds, Pointer, Query, Res, ResMut, Transform};

use crate::types::SurfaceId;
use crate::components::{AppSurface, BottomBar, CardStackRoot, GestureZone, HomeSurface, NotificationCard, NotificationLayer, NotificationStackRoot, NotificationText, NotificationTextRole, OverlayLayer, QuickSettingsPanel, ShellRoot, StatusBar, SurfacePreviewCard, SurfaceStackRoot, SurfaceText, WidgetLayoutNode};
use crate::resources::{DesktopMetrics, ShellState};

const HOME_SURFACE_ID: SurfaceId = 1;
const APP_SURFACE_ID: SurfaceId = 2;

pub fn setup_wing_shell(
    mut commands: Commands,
    screen: Res<fhre::PrimaryScreen>,
    mut shell_state: ResMut<ShellState>,
) {
    let width = screen.width as f32;
    let height = screen.height as f32;
    let center_x = width * 0.5;
    let center_y = height * 0.5;

    commands.spawn()
        .insert(ShellRoot)
        .insert(Transform::from_position(center_x, center_y, 0.0))
        .insert(WidgetLayoutNode { width, height });

    commands.spawn()
        .insert(HomeSurface::active())
        .insert(PickableBounds::from_size(width, height - 68.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, center_y + 10.0, 0.02))
        .insert(WidgetLayoutNode { width, height: height - 68.0 });

    commands.spawn()
        .insert(StatusBar)
        .insert(PickableBounds::from_size(width, 28.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, 14.0, 0.04))
        .insert(WidgetLayoutNode { width, height: 28.0 });

    commands.spawn()
        .insert(BottomBar)
        .insert(PickableBounds::from_size(width, 40.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, height - 20.0, 0.04))
        .insert(WidgetLayoutNode { width, height: 40.0 });

    commands.spawn()
        .insert(SurfaceStackRoot)
        .insert(Transform::from_position(center_x, center_y + 12.0, 0.021))
        .insert(WidgetLayoutNode { width, height: height - 68.0 });

    commands.spawn()
        .insert(CardStackRoot)
        .insert(Transform::from_position(center_x, center_y + 6.0, 0.044))
        .insert(WidgetLayoutNode { width: width - 40.0, height: height - 96.0 });

    commands.spawn()
        .insert(OverlayLayer::hidden())
        .insert(PickableBounds::from_size(width, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, center_y, 0.045))
        .insert(WidgetLayoutNode { width, height });

    commands.spawn()
        .insert(NotificationLayer::hidden())
        .insert(PickableBounds::from_size(width - 24.0, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, 56.0, 0.048))
        .insert(WidgetLayoutNode { width: width - 24.0, height: 96.0 });

    commands.spawn()
        .insert(NotificationStackRoot)
        .insert(Transform::from_position(center_x, 142.0, 0.049))
        .insert(WidgetLayoutNode { width: width - 36.0, height: 144.0 });

    commands.spawn()
        .insert(GestureZone)
        .insert(PickableBounds::from_size(width * 0.4, 24.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, height - 20.0, 0.05))
        .insert(WidgetLayoutNode { width: width * 0.4, height: 24.0 });

    commands.spawn()
        .insert(QuickSettingsPanel::closed())
        .insert(PickableBounds::from_size(width - 24.0, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, 88.0, 0.05))
        .insert(WidgetLayoutNode { width: width - 24.0, height: 120.0 });

    commands.spawn()
        .insert(AppSurface::new(APP_SURFACE_ID))
        .insert(PickableBounds::from_size(240.0, 180.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, center_y + 28.0, 0.03))
        .insert(WidgetLayoutNode { width: 240.0, height: 180.0 });

    commands.spawn()
        .insert(SurfacePreviewCard::hidden(HOME_SURFACE_ID, 0))
        .insert(PickableBounds::from_size(220.0, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, center_y - 8.0, 0.046))
        .insert(WidgetLayoutNode { width: 220.0, height: 136.0 });

    commands.spawn()
        .insert(SurfacePreviewCard::hidden(APP_SURFACE_ID, 1))
        .insert(PickableBounds::from_size(220.0, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, center_y + 24.0, 0.047))
        .insert(WidgetLayoutNode { width: 220.0, height: 144.0 });

    commands.spawn()
        .insert(NotificationCard::hidden(0))
        .insert(PickableBounds::from_size(width - 48.0, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, 118.0, 0.05))
        .insert(WidgetLayoutNode { width: width - 48.0, height: 48.0 });

    commands.spawn()
        .insert(NotificationText::title(0, "System Update"))
        .insert(Transform::from_position(center_x - 72.0, 108.0, 0.052))
        .insert(WidgetLayoutNode { width: 120.0, height: 14.0 });

    commands.spawn()
        .insert(NotificationText::summary(0, "Shell card stack ready"))
        .insert(Transform::from_position(center_x - 52.0, 126.0, 0.052))
        .insert(WidgetLayoutNode { width: 168.0, height: 14.0 });

    commands.spawn()
        .insert(NotificationCard::hidden(1))
        .insert(PickableBounds::from_size(width - 60.0, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, 174.0, 0.051))
        .insert(WidgetLayoutNode { width: width - 60.0, height: 44.0 });

    commands.spawn()
        .insert(NotificationText::title(1, "Watch Shell"))
        .insert(Transform::from_position(center_x - 68.0, 164.0, 0.053))
        .insert(WidgetLayoutNode { width: 112.0, height: 14.0 });

    commands.spawn()
        .insert(NotificationText::summary(1, "Notifications stack online"))
        .insert(Transform::from_position(center_x - 36.0, 180.0, 0.053))
        .insert(WidgetLayoutNode { width: 184.0, height: 14.0 });

    commands.spawn()
        .insert(SurfaceText::shell("Wing"))
        .insert(Transform::from_position(32.0, 14.0, 0.045))
        .insert(WidgetLayoutNode { width: 80.0, height: 16.0 });

    commands.spawn()
        .insert(SurfaceText::shell("Quick Settings"))
        .insert(Transform::from_position(center_x, 56.0, 0.055))
        .insert(WidgetLayoutNode { width: 140.0, height: 16.0 });

    commands.spawn()
        .insert(SurfaceText::shell("Notifications"))
        .insert(Transform::from_position(center_x, 84.0, 0.056))
        .insert(WidgetLayoutNode { width: 120.0, height: 16.0 });

    commands.spawn()
        .insert(SurfaceText::for_surface(APP_SURFACE_ID, "No apps"))
        .insert(Transform::from_position(center_x, center_y + 28.0, 0.035))
        .insert(WidgetLayoutNode { width: 120.0, height: 16.0 });

    commands.spawn()
        .insert(SurfaceText::shell("Card Stack"))
        .insert(Transform::from_position(center_x, center_y - 40.0, 0.057))
        .insert(WidgetLayoutNode { width: 96.0, height: 16.0 });

    commands.spawn()
        .insert(SurfaceText::shell("Home Gesture"))
        .insert(Transform::from_position(center_x, height - 20.0, 0.055))
        .insert(WidgetLayoutNode { width: 100.0, height: 16.0 });

    shell_state.active_surface = Some(HOME_SURFACE_ID);
    shell_state.quick_settings_open = false;
    shell_state.overlay_visible = false;
    shell_state.notifications_visible = false;
    shell_state.app_switcher_open = false;
}

pub fn wing_shell_interaction_system(
    events: Res<Events>,
    mut shell_state: ResMut<ShellState>,
    mut status_bar_query: Query<&StatusBar>,
    mut bottom_bar_query: Query<&BottomBar>,
    mut gesture_query: Query<&GestureZone>,
    mut overlay_query: Query<&OverlayLayer>,
    mut card_query: Query<&SurfacePreviewCard>,
    mut app_surface_query: Query<&AppSurface>,
) {
    if let Some(click_events) = events.get_events_current::<Pointer<fhre::Click>>() {
        for event in click_events {
            if status_bar_query.get_pair(event.entity).is_some() {
                shell_state.quick_settings_open = !shell_state.quick_settings_open;
                shell_state.overlay_visible = shell_state.quick_settings_open;
                shell_state.notifications_visible = shell_state.quick_settings_open;
                shell_state.app_switcher_open = false;
                continue;
            }
            if bottom_bar_query.get_pair(event.entity).is_some() {
                shell_state.app_switcher_open = !shell_state.app_switcher_open;
                shell_state.quick_settings_open = false;
                shell_state.notifications_visible = false;
                shell_state.overlay_visible = shell_state.app_switcher_open;
                continue;
            }
            if gesture_query.get_pair(event.entity).is_some() {
                shell_state.active_surface = Some(HOME_SURFACE_ID);
                shell_state.quick_settings_open = false;
                shell_state.overlay_visible = false;
                shell_state.notifications_visible = false;
                shell_state.app_switcher_open = false;
                continue;
            }
            if overlay_query.get_pair(event.entity).is_some() {
                shell_state.quick_settings_open = false;
                shell_state.overlay_visible = false;
                shell_state.notifications_visible = false;
                shell_state.app_switcher_open = false;
                continue;
            }
            if let Some((_, card)) = card_query.get_pair(event.entity) {
                shell_state.active_surface = Some(card.surface_id);
                shell_state.app_switcher_open = false;
                shell_state.overlay_visible = false;
                continue;
            }
            if let Some((_, surface)) = app_surface_query.get_pair(event.entity) {
                shell_state.active_surface = Some(surface.id);
                shell_state.notifications_visible = false;
            }
        }
    }
}

pub fn wing_shell_layout_system(
    metrics: Res<DesktopMetrics>,
    shell_state: Res<ShellState>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut home_query: Query<&mut HomeSurface>,
    mut status_query: Query<&StatusBar>,
    mut bottom_query: Query<&BottomBar>,
    mut app_query: Query<&mut AppSurface>,
) {
    let width = metrics.screen_size.x;
    let height = metrics.screen_size.y;
    let center_x = width * 0.5;
    let content_height = height - 68.0;

    for (entity, home_surface) in home_query.iter_mut() {
        home_surface.active = shell_state.active_surface == Some(HOME_SURFACE_ID);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = 28.0 + content_height * 0.5;
            transform.position.z = if home_surface.active { 0.02 } else { 0.01 };
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width;
            bounds.height = if home_surface.active { content_height } else { 0.0 };
        }
    }

    for (entity, _) in status_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = 14.0;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width;
            bounds.height = 28.0;
        }
    }

    for (entity, _) in bottom_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = height - 20.0;
        }
    }

    for (entity, app_surface) in app_query.iter_mut() {
        app_surface.active = shell_state.active_surface == Some(app_surface.id);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = 28.0 + content_height * 0.5;
            transform.position.z = if app_surface.active { 0.03 } else { 0.015 };
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = 240.0;
            bounds.height = if app_surface.active { 180.0 } else { 0.0 };
        }
    }
}

pub fn wing_shell_stack_layout_system(
    metrics: Res<DesktopMetrics>,
    mut transform_query: Query<&mut Transform>,
    mut stack_query: Query<&SurfaceStackRoot>,
    mut card_stack_query: Query<&CardStackRoot>,
    mut notification_stack_query: Query<&NotificationStackRoot>,
) {
    let width = metrics.screen_size.x;
    let height = metrics.screen_size.y;
    let center_x = width * 0.5;
    let content_height = height - 68.0;

    for (entity, _) in stack_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = 28.0 + content_height * 0.5;
        }
    }

    for (entity, _) in card_stack_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = height * 0.5 + 6.0;
        }
    }

    for (entity, _) in notification_stack_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = 142.0;
        }
    }
}

pub fn wing_shell_overlay_layout_system(
    metrics: Res<DesktopMetrics>,
    shell_state: Res<ShellState>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut overlay_query: Query<&mut OverlayLayer>,
    mut notification_query: Query<&mut NotificationLayer>,
    mut notification_card_query: Query<&mut NotificationCard>,
    mut gesture_query: Query<&GestureZone>,
    mut quick_query: Query<&mut QuickSettingsPanel>,
    mut card_query: Query<&mut SurfacePreviewCard>,
) {
    let width = metrics.screen_size.x;
    let height = metrics.screen_size.y;
    let center_x = width * 0.5;
    let center_y = height * 0.5;

    for (entity, overlay) in overlay_query.iter_mut() {
        overlay.visible = shell_state.overlay_visible;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = center_y;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width;
            bounds.height = if shell_state.overlay_visible { height } else { 0.0 };
        }
    }

    for (entity, notification) in notification_query.iter_mut() {
        notification.visible = shell_state.notifications_visible;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = 56.0;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width - 24.0;
            bounds.height = if shell_state.notifications_visible { 96.0 } else { 0.0 };
        }
    }

    for (entity, notification_card) in notification_card_query.iter_mut() {
        notification_card.visible = shell_state.notifications_visible;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = 118.0 + (notification_card.stack_index as f32 * 56.0);
            transform.position.z = 0.05 + (notification_card.stack_index as f32 * 0.001);
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width - 48.0 - (notification_card.stack_index as f32 * 12.0);
            bounds.height = if shell_state.notifications_visible {
                48.0 - (notification_card.stack_index as f32 * 4.0)
            } else {
                0.0
            };
        }
    }

    for (entity, _) in gesture_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = height - 20.0;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width * 0.4;
            bounds.height = 24.0;
        }
    }

    for (entity, panel) in quick_query.iter_mut() {
        panel.open = shell_state.quick_settings_open;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = if shell_state.quick_settings_open { 88.0 } else { 28.0 };
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width - 24.0;
            bounds.height = if shell_state.quick_settings_open { 120.0 } else { 0.0 };
        }
    }

    for (entity, card) in card_query.iter_mut() {
        card.visible = shell_state.app_switcher_open;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = height * 0.5 - 8.0 + (card.stack_index as f32 * 32.0);
            transform.position.z = 0.046 + (card.stack_index as f32 * 0.001);
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = 220.0 - (card.stack_index as f32 * 12.0);
            bounds.height = if shell_state.app_switcher_open {
                136.0 + (card.stack_index as f32 * 8.0)
            } else {
                0.0
            };
        }
    }
}

pub fn wing_notification_text_layout_system(
    metrics: Res<DesktopMetrics>,
    shell_state: Res<ShellState>,
    mut transform_query: Query<&mut Transform>,
    mut notification_text_query: Query<&NotificationText>,
) {
    let center_x = metrics.screen_size.x * 0.5;

    for (entity, notification_text) in notification_text_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            let base_y = 108.0 + (notification_text.stack_index as f32 * 56.0);
            transform.position.x = match notification_text.role {
                NotificationTextRole::Title => center_x - 72.0 + (notification_text.stack_index as f32 * 4.0),
                NotificationTextRole::Summary => center_x - 52.0 + (notification_text.stack_index as f32 * 16.0),
            };
            transform.position.y = match notification_text.role {
                NotificationTextRole::Title => base_y,
                NotificationTextRole::Summary => base_y + 18.0,
            };
            transform.position.z = if shell_state.notifications_visible { 0.052 } else { 0.0 };
        }
    }
}
