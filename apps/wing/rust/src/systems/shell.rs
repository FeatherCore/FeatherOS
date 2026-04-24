use fhre::{Commands, Events, Pickable, PickableBounds, Pointer, Query, Res, ResMut, Transform};

use crate::types::SurfaceId;
use crate::components::{AppSurface, BottomBar, CardStackRoot, GestureZone, HomeSurface, NotificationCard, NotificationLayer, NotificationStackRoot, NotificationText, NotificationTextRole, OverlayLayer, QuickSettingsPanel, ShellRoot, StatusBar, SurfaceCardSubtitle, SurfaceCardTitle, SurfacePreviewCard, SurfaceStackRoot, SurfaceText, WidgetLayoutNode};
use crate::resources::{ShellContent, ShellMetrics, ShellOverlayAnimation, ShellOverlayMode, ShellState};

const HOME_SURFACE_ID: SurfaceId = 1;
const STATUS_BAR_HEIGHT: f32 = 28.0;
const BOTTOM_BAR_HEIGHT: f32 = 40.0;
const SHELL_CHROME_HEIGHT: f32 = STATUS_BAR_HEIGHT + BOTTOM_BAR_HEIGHT;
const SHELL_SIDE_INSET: f32 = 24.0;
const GESTURE_ZONE_HEIGHT: f32 = 24.0;
const GESTURE_ZONE_WIDTH_FACTOR: f32 = 0.4;
const APP_SURFACE_WIDTH: f32 = 240.0;
const APP_SURFACE_HEIGHT: f32 = 180.0;
const QUICK_SETTINGS_HEIGHT: f32 = 120.0;
const NOTIFICATION_PANEL_HEIGHT: f32 = 96.0;
const NOTIFICATION_PANEL_Y: f32 = 56.0;
const QUICK_SETTINGS_OPEN_Y: f32 = 88.0;

struct CardStackLayout {
    card_width: f32,
    card_base_height: f32,
    card_height_step: f32,
    card_y_step: f32,
    base_y_offset: f32,
}

impl CardStackLayout {
    fn compute(surface_count: usize, screen_height: f32) -> Self {
        let available_height = screen_height - STATUS_BAR_HEIGHT - BOTTOM_BAR_HEIGHT - 80.0;
        let card_width = 220.0;
        let card_base_height = (available_height / surface_count.max(1) as f32).min(160.0).max(80.0);
        let card_height_step = 8.0;
        let card_y_step = card_base_height * 0.25;
        let base_y_offset = -8.0;
        
        Self {
            card_width,
            card_base_height,
            card_height_step,
            card_y_step,
            base_y_offset,
        }
    }
    
    fn card_y(&self, screen_height: f32, stack_index: u8) -> f32 {
        screen_height * 0.5 + self.base_y_offset + (stack_index as f32 * self.card_y_step)
    }
    
    fn card_height(&self, stack_index: u8) -> f32 {
        self.card_base_height + (stack_index as f32 * self.card_height_step)
    }
    
    fn card_width(&self, stack_index: u8) -> f32 {
        self.card_width - (stack_index as f32 * 12.0)
    }
}

struct NotificationStackLayout {
    base_y: f32,
    y_step: f32,
    base_height: f32,
    height_step: f32,
    base_width_inset: f32,
    width_step: f32,
}

impl NotificationStackLayout {
    fn compute(notification_count: usize, screen_width: f32) -> Self {
        let available_height = 200.0;
        let base_height = (available_height / notification_count.max(1) as f32).min(64.0).max(32.0);
        let y_step = base_height + 8.0;
        let base_y = NOTIFICATION_PANEL_Y + NOTIFICATION_PANEL_HEIGHT + 8.0;
        
        Self {
            base_y,
            y_step,
            base_height,
            height_step: 4.0,
            base_width_inset: screen_width * 0.15,
            width_step: 12.0,
        }
    }
    
    fn card_y(&self, stack_index: u8) -> f32 {
        self.base_y + (stack_index as f32 * self.y_step)
    }
    
    fn card_height(&self, stack_index: u8) -> f32 {
        self.base_height - (stack_index as f32 * self.height_step)
    }
    
    fn card_width(&self, screen_width: f32, stack_index: u8) -> f32 {
        screen_width - self.base_width_inset - (stack_index as f32 * self.width_step)
    }
}

fn content_height(height: f32) -> f32 {
    height - SHELL_CHROME_HEIGHT
}

fn content_center_y(height: f32) -> f32 {
    STATUS_BAR_HEIGHT + content_height(height) * 0.5
}

fn bottom_bar_center_y(height: f32) -> f32 {
    height - BOTTOM_BAR_HEIGHT * 0.5
}

pub fn setup_wing_shell(
    mut commands: Commands,
    screen: Res<fhre::PrimaryScreen>,
    content: Res<ShellContent>,
    mut shell_state: ResMut<ShellState>,
) {
    let width = screen.width as f32;
    let height = screen.height as f32;
    let center_x = width * 0.5;
    let center_y = height * 0.5;

    let card_layout = CardStackLayout::compute(content.surfaces.len(), height);
    let notif_layout = NotificationStackLayout::compute(content.notifications.len(), width);

    commands.spawn()
        .insert(ShellRoot)
        .insert(Transform::from_position(center_x, center_y, 0.0))
        .insert(WidgetLayoutNode { width, height });

    commands.spawn()
        .insert(HomeSurface::active())
        .insert(PickableBounds::from_size(width, content_height(height)))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, content_center_y(height), 0.02))
        .insert(WidgetLayoutNode { width, height: content_height(height) });

    commands.spawn()
        .insert(StatusBar)
        .insert(PickableBounds::from_size(width, STATUS_BAR_HEIGHT))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, STATUS_BAR_HEIGHT * 0.5, 0.04))
        .insert(WidgetLayoutNode { width, height: STATUS_BAR_HEIGHT });

    commands.spawn()
        .insert(BottomBar)
        .insert(PickableBounds::from_size(width, BOTTOM_BAR_HEIGHT))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, bottom_bar_center_y(height), 0.04))
        .insert(WidgetLayoutNode { width, height: BOTTOM_BAR_HEIGHT });

    commands.spawn()
        .insert(SurfaceStackRoot)
        .insert(Transform::from_position(center_x, content_center_y(height), 0.021))
        .insert(WidgetLayoutNode { width, height: content_height(height) });

    commands.spawn()
        .insert(CardStackRoot)
        .insert(Transform::from_position(center_x, center_y, 0.044))
        .insert(WidgetLayoutNode { width, height: content_height(height) });

    commands.spawn()
        .insert(OverlayLayer::hidden())
        .insert(PickableBounds::from_size(width, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, center_y, 0.045))
        .insert(WidgetLayoutNode { width, height });

    commands.spawn()
        .insert(NotificationLayer::hidden())
        .insert(PickableBounds::from_size(width - SHELL_SIDE_INSET, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, NOTIFICATION_PANEL_Y, 0.048))
        .insert(WidgetLayoutNode { width: width - SHELL_SIDE_INSET, height: NOTIFICATION_PANEL_HEIGHT });

    commands.spawn()
        .insert(NotificationStackRoot)
        .insert(Transform::from_position(center_x, notif_layout.base_y, 0.049))
        .insert(WidgetLayoutNode { width: width - SHELL_SIDE_INSET, height: notif_layout.base_height * content.notifications.len() as f32 });

    commands.spawn()
        .insert(GestureZone)
        .insert(PickableBounds::from_size(width * GESTURE_ZONE_WIDTH_FACTOR, GESTURE_ZONE_HEIGHT))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, bottom_bar_center_y(height), 0.05))
        .insert(WidgetLayoutNode { width: width * GESTURE_ZONE_WIDTH_FACTOR, height: GESTURE_ZONE_HEIGHT });

    commands.spawn()
        .insert(QuickSettingsPanel::closed())
        .insert(PickableBounds::from_size(width - SHELL_SIDE_INSET, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, QUICK_SETTINGS_OPEN_Y, 0.05))
        .insert(WidgetLayoutNode { width: width - SHELL_SIDE_INSET, height: QUICK_SETTINGS_HEIGHT });

    for surface in content.surfaces.iter().copied() {
        if surface.id != HOME_SURFACE_ID {
            commands.spawn()
                .insert(AppSurface::new(surface.id))
                .insert(PickableBounds::from_size(APP_SURFACE_WIDTH, APP_SURFACE_HEIGHT))
                .insert(Pickable::DEFAULT)
                .insert(Transform::from_position(center_x, content_center_y(height), 0.03))
                .insert(WidgetLayoutNode { width: APP_SURFACE_WIDTH, height: APP_SURFACE_HEIGHT });

            commands.spawn()
                .insert(SurfaceText::for_surface(surface.id, surface.label))
                .insert(Transform::from_position(center_x, content_center_y(height) + APP_SURFACE_HEIGHT * 0.5 + 16.0, 0.035))
                .insert(WidgetLayoutNode { width: 120.0, height: 16.0 });
        }

        let stack_index = surface_index(surface.id, &content).unwrap_or(0);
        commands.spawn()
            .insert(SurfacePreviewCard::hidden(surface.id, stack_index)
                .with_state(surface.state)
                .with_icon(surface.icon_hint))
            .insert(PickableBounds::from_size(card_layout.card_width(stack_index), 0.0))
            .insert(Pickable::DEFAULT)
            .insert(Transform::from_position(center_x, card_layout.card_y(height, stack_index), 0.046 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: card_layout.card_width(stack_index), height: card_layout.card_height(stack_index) });

        commands.spawn()
            .insert(SurfaceCardTitle::for_card(surface.id, stack_index))
            .insert(Transform::from_position(center_x - 60.0, card_layout.card_y(height, stack_index) - 12.0, 0.047 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: 120.0, height: 14.0 });

        commands.spawn()
            .insert(SurfaceCardSubtitle::for_card(surface.id, stack_index))
            .insert(Transform::from_position(center_x - 40.0, card_layout.card_y(height, stack_index) + 8.0, 0.0475 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: 100.0, height: 12.0 });
    }

    for (stack_index, notification) in content.notifications.iter().copied().enumerate() {
        let stack_index = stack_index as u8;
        commands.spawn()
            .insert(NotificationCard::hidden(notification.id, stack_index)
                .with_priority(notification.priority)
                .with_category(notification.category))
            .insert(PickableBounds::from_size(notif_layout.card_width(width, stack_index), 0.0))
            .insert(Pickable::DEFAULT)
            .insert(Transform::from_position(center_x, notif_layout.card_y(stack_index), 0.05 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: notif_layout.card_width(width, stack_index), height: notif_layout.card_height(stack_index) });

        commands.spawn()
            .insert(NotificationText::title(stack_index, notification.title))
            .insert(Transform::from_position(center_x - 60.0, notif_layout.card_y(stack_index) - 8.0, 0.052 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: 120.0, height: 14.0 });

        commands.spawn()
            .insert(NotificationText::summary(stack_index, notification.summary))
            .insert(Transform::from_position(center_x - 40.0, notif_layout.card_y(stack_index) + 8.0, 0.052 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: 184.0, height: 14.0 });
    }

    commands.spawn()
        .insert(SurfaceText::shell("Wing"))
        .insert(Transform::from_position(32.0, 14.0, 0.045))
        .insert(WidgetLayoutNode { width: 80.0, height: 16.0 });

    commands.spawn()
        .insert(SurfaceText::shell("Quick Settings"))
        .insert(Transform::from_position(center_x, NOTIFICATION_PANEL_Y, 0.055))
        .insert(WidgetLayoutNode { width: 140.0, height: 16.0 });

    commands.spawn()
        .insert(SurfaceText::shell("Notifications"))
        .insert(Transform::from_position(center_x, 84.0, 0.056))
        .insert(WidgetLayoutNode { width: 120.0, height: 16.0 });

    if let Some(surface) = content.surfaces.first() {
        commands.spawn()
            .insert(SurfaceText::shell(surface.preview_title))
            .insert(Transform::from_position(center_x, center_y - 40.0, 0.057))
            .insert(WidgetLayoutNode { width: 96.0, height: 16.0 });
    }

    commands.spawn()
        .insert(SurfaceText::shell("Home Gesture"))
        .insert(Transform::from_position(center_x, height - 20.0, 0.055))
        .insert(WidgetLayoutNode { width: 100.0, height: 16.0 });

    shell_state.active_surface = Some(HOME_SURFACE_ID);
    shell_state.overlay_mode = ShellOverlayMode::None;
}

fn surface_index(surface_id: SurfaceId, content: &ShellContent) -> Option<u8> {
    content
        .surfaces
        .iter()
        .position(|surface| surface.id == surface_id)
        .map(|index| index as u8)
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
                shell_state.overlay_mode = if shell_state.quick_settings_open() {
                    ShellOverlayMode::None
                } else {
                    ShellOverlayMode::QuickSettings
                };
                continue;
            }
            if bottom_bar_query.get_pair(event.entity).is_some() {
                shell_state.overlay_mode = if shell_state.app_switcher_open() {
                    ShellOverlayMode::None
                } else {
                    ShellOverlayMode::AppSwitcher
                };
                continue;
            }
            if gesture_query.get_pair(event.entity).is_some() {
                shell_state.active_surface = Some(HOME_SURFACE_ID);
                shell_state.overlay_mode = ShellOverlayMode::None;
                continue;
            }
            if overlay_query.get_pair(event.entity).is_some() {
                shell_state.overlay_mode = ShellOverlayMode::None;
                continue;
            }
            if let Some((_, card)) = card_query.get_pair(event.entity) {
                shell_state.active_surface = Some(card.surface_id);
                shell_state.overlay_mode = ShellOverlayMode::None;
                continue;
            }
            if let Some((_, surface)) = app_surface_query.get_pair(event.entity) {
                shell_state.active_surface = Some(surface.id);
                if shell_state.quick_settings_open() {
                    shell_state.overlay_mode = ShellOverlayMode::None;
                }
            }
        }
    }
}

pub fn wing_shell_layout_system(
    metrics: Res<ShellMetrics>,
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
    let content_height = content_height(height);

    for (entity, home_surface) in home_query.iter_mut() {
        home_surface.active = shell_state.active_surface == Some(HOME_SURFACE_ID);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = content_center_y(height);
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
            bounds.height = STATUS_BAR_HEIGHT;
        }
    }

    for (entity, _) in bottom_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = bottom_bar_center_y(height);
        }
    }

    for (entity, app_surface) in app_query.iter_mut() {
        app_surface.active = shell_state.active_surface == Some(app_surface.id);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = content_center_y(height);
            transform.position.z = if app_surface.active { 0.03 } else { 0.015 };
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = APP_SURFACE_WIDTH;
            bounds.height = if app_surface.active { APP_SURFACE_HEIGHT } else { 0.0 };
        }
    }
}

pub fn wing_shell_stack_layout_system(
    metrics: Res<ShellMetrics>,
    content: Res<ShellContent>,
    mut transform_query: Query<&mut Transform>,
    mut stack_query: Query<&SurfaceStackRoot>,
    mut card_stack_query: Query<&CardStackRoot>,
    mut notification_stack_query: Query<&NotificationStackRoot>,
) {
    let width = metrics.screen_size.x;
    let height = metrics.screen_size.y;
    let center_x = width * 0.5;
    
    let notif_layout = NotificationStackLayout::compute(content.notifications.len(), width);
    
    for (entity, _) in stack_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = content_center_y(height);
        }
    }

    for (entity, _) in card_stack_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = height * 0.5;
        }
    }

    for (entity, _) in notification_stack_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = notif_layout.base_y;
        }
    }
}

pub fn wing_shell_overlay_layout_system(
    metrics: Res<ShellMetrics>,
    shell_state: Res<ShellState>,
    animation: Res<ShellOverlayAnimation>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut overlay_query: Query<&mut OverlayLayer>,
    mut notification_query: Query<&mut NotificationLayer>,
    mut gesture_query: Query<&GestureZone>,
    mut quick_query: Query<&mut QuickSettingsPanel>,
) {
    let width = metrics.screen_size.x;
    let height = metrics.screen_size.y;
    let center_x = width * 0.5;
    let center_y = height * 0.5;

    let qs_progress = animation.quick_settings_eased();

    for (entity, overlay) in overlay_query.iter_mut() {
        overlay.visible = shell_state.overlay_visible() || animation.is_animating();
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = center_y;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width;
            bounds.height = height * qs_progress.max(animation.app_switcher_eased());
        }
    }

    for (entity, notification) in notification_query.iter_mut() {
        notification.visible = shell_state.notifications_visible() || qs_progress > 0.0;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = NOTIFICATION_PANEL_Y;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width - SHELL_SIDE_INSET;
            bounds.height = NOTIFICATION_PANEL_HEIGHT * qs_progress;
        }
    }

    for (entity, _) in gesture_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = bottom_bar_center_y(height);
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width * GESTURE_ZONE_WIDTH_FACTOR;
            bounds.height = GESTURE_ZONE_HEIGHT;
        }
    }

    for (entity, panel) in quick_query.iter_mut() {
        panel.open = shell_state.quick_settings_open();
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            let target_y = QUICK_SETTINGS_OPEN_Y;
            let start_y = STATUS_BAR_HEIGHT;
            transform.position.y = start_y + (target_y - start_y) * qs_progress;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width - SHELL_SIDE_INSET;
            bounds.height = QUICK_SETTINGS_HEIGHT * qs_progress;
        }
    }
}

pub fn wing_notification_card_layout_system(
    metrics: Res<ShellMetrics>,
    content: Res<ShellContent>,
    shell_state: Res<ShellState>,
    animation: Res<ShellOverlayAnimation>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut notification_card_query: Query<&mut NotificationCard>,
) {
    let width = metrics.screen_size.x;
    let center_x = width * 0.5;
    let notif_layout = NotificationStackLayout::compute(content.notifications.len(), width);
    let qs_progress = animation.quick_settings_eased();

    for (entity, notification_card) in notification_card_query.iter_mut() {
        notification_card.visible = shell_state.notifications_visible() || qs_progress > 0.0;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            let target_y = notif_layout.card_y(notification_card.stack_index);
            let start_y = NOTIFICATION_PANEL_Y - 20.0;
            transform.position.y = start_y + (target_y - start_y) * qs_progress;
            transform.position.z = 0.05 + (notification_card.stack_index as f32 * 0.001);
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = notif_layout.card_width(width, notification_card.stack_index);
            bounds.height = notif_layout.card_height(notification_card.stack_index) * qs_progress;
        }
    }
}

pub fn wing_shell_overlay_card_layout_system(
    metrics: Res<ShellMetrics>,
    content: Res<ShellContent>,
    shell_state: Res<ShellState>,
    animation: Res<ShellOverlayAnimation>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut card_query: Query<&mut SurfacePreviewCard>,
) {
    let width = metrics.screen_size.x;
    let height = metrics.screen_size.y;
    let center_x = width * 0.5;

    let card_layout = CardStackLayout::compute(content.surfaces.len(), height);
    let app_progress = animation.app_switcher_eased();

    for (entity, card) in card_query.iter_mut() {
        card.visible = shell_state.app_switcher_open() || app_progress > 0.0;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            let target_y = card_layout.card_y(height, card.stack_index);
            let start_y = height + 50.0;
            transform.position.y = start_y + (target_y - start_y) * app_progress;
            transform.position.z = 0.046 + (card.stack_index as f32 * 0.001);
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = card_layout.card_width(card.stack_index);
            bounds.height = card_layout.card_height(card.stack_index) * app_progress;
        }
    }
}

pub fn wing_notification_text_layout_system(
    metrics: Res<ShellMetrics>,
    content: Res<ShellContent>,
    animation: Res<ShellOverlayAnimation>,
    mut transform_query: Query<&mut Transform>,
    mut notification_text_query: Query<&NotificationText>,
) {
    let center_x = metrics.screen_size.x * 0.5;
    let notif_layout = NotificationStackLayout::compute(content.notifications.len(), metrics.screen_size.x);
    let qs_progress = animation.quick_settings_eased();

    for (entity, notification_text) in notification_text_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            let text_x = center_x + match notification_text.role {
                NotificationTextRole::Title => -60.0,
                NotificationTextRole::Summary => -40.0,
            };
            let target_y = notif_layout.card_y(notification_text.stack_index) 
                + match notification_text.role {
                    NotificationTextRole::Title => -8.0,
                    NotificationTextRole::Summary => 8.0,
                };
            let start_y = NOTIFICATION_PANEL_Y - 20.0;
            let text_y = start_y + (target_y - start_y) * qs_progress;
            
            transform.position.x = text_x;
            transform.position.y = text_y;
            transform.position.z = if qs_progress > 0.0 { 0.052 } else { 0.0 };
        }
    }
}
