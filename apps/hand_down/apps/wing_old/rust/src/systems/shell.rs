//! Shell setup and layout systems.

use fhre::{Commands, Pickable, PickableBounds, Query, Res, ResMut, Transform};

use crate::types::SurfaceId;
use crate::components::{
    app_switcher_ball_center, app_switcher_ball_size, point_in_polygon,
    surface_index_for_face, AppSurface, BrightnessControl, CardStackRoot, HomeSurface,
    LauncherIcon, LauncherIconLabel, NotificationCard, NotificationCardContent,
    NotificationCardTitle, NotificationPanel, OverlayLayer, PreviewSoccerBall,
    QuickControlTile, QuickControlType, SettingsAction, SettingsPanel,
    SettingsRow, SettingsRowLabel, SettingsRowValue, ShellRoot,
    SurfaceCardSubtitle, SurfaceCardTitle, SurfacePreviewCard,
    SurfaceStackRoot, SurfaceText, SystemNavAction, SystemNavButton, ThemeBackdrop,
    SystemNavButtonLabel, WidgetLayoutNode,
};
use crate::resources::{
    PreviewEffect, ShellContent, ShellMetrics, ShellOverlayAnimation, ShellOverlayMode, ShellState,
    ShellSurfaceEntry, SurfaceState, ThemeState, WingAppLaunchKind,
};
use crate::platform::{self, TaskStatus};

const HOME_SURFACE_ID: SurfaceId = 1;
const SETTINGS_SURFACE_ID: SurfaceId = 2;
const SETTINGS_ACTIONS: [SettingsAction; 9] = [
    SettingsAction::ToggleWifi,
    SettingsAction::ToggleBluetooth,
    SettingsAction::ToggleAirplaneMode,
    SettingsAction::ToggleFlashlight,
    SettingsAction::ToggleDnd,
    SettingsAction::ToggleAutoRotate,
    SettingsAction::CycleBrightness,
    SettingsAction::CycleTheme,
    SettingsAction::CyclePreviewEffect,
];

/// Card stack layout for surface preview cards.
struct CardStackLayout {
    card_width: f32,
    card_base_height: f32,
    card_height_step: f32,
    card_y_step: f32,
    base_y_offset: f32,
}

impl CardStackLayout {
    fn compute(surface_count: usize, metrics: &ShellMetrics) -> Self {
        let available_height = metrics.height() - metrics.height() * 0.13;
        let card_width = metrics.width() * 0.55;
        let card_base_height = (available_height / surface_count.max(1) as f32)
            .min(metrics.height() * 0.27)
            .max(metrics.height() * 0.13);
        let card_height_step = metrics.height() * 0.013;
        let card_y_step = card_base_height * 0.25;
        let base_y_offset = -metrics.height() * 0.013;
        
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
        self.card_width - (stack_index as f32 * self.card_width * 0.05)
    }
}

/// Notification card stack layout.
struct NotificationStackLayout {
    card_width: f32,
    card_height: f32,
    card_spacing: f32,
    start_y: f32,
}

impl NotificationStackLayout {
    fn compute(metrics: &ShellMetrics) -> Self {
        let card_width = metrics.width() - metrics.side_inset() * 2.0;
        let card_height = metrics.notification_card_height();
        let card_spacing = metrics.notification_card_spacing();
        
        // Notification cards start after the header section
        let header_bottom = metrics.notification_panel_header_height();
        let start_y = header_bottom + card_height * 0.5;
        
        Self {
            card_width,
            card_height,
            card_spacing,
            start_y,
        }
    }
    
    fn compute_with_qc_height(metrics: &ShellMetrics, qc_bottom: f32) -> Self {
        let card_width = metrics.width() - metrics.side_inset() * 2.0;
        let card_height = metrics.notification_card_height();
        let card_spacing = metrics.notification_card_spacing();
        
        // Add spacing after quick controls, then start cards
        let spacing = metrics.section_spacing();
        let start_y = qc_bottom + spacing + card_height * 0.5;
        
        Self {
            card_width,
            card_height,
            card_spacing,
            start_y,
        }
    }
    
    fn card_y(&self, stack_index: u8) -> f32 {
        self.start_y + (stack_index as f32 * (self.card_height + self.card_spacing))
    }
}

fn launcher_icon_size(metrics: &ShellMetrics) -> f32 {
    (metrics.width() * 0.12).clamp(44.0, 64.0)
}

fn launcher_icon_position(metrics: &ShellMetrics, index: u8) -> fhre::Vec2 {
    let icon_size = launcher_icon_size(metrics);
    let spacing_x = icon_size * 1.35;
    let spacing_y = icon_size * 1.55;
    let columns = 3usize;
    let col = index as usize % columns;
    let row = index as usize / columns;
    let total_width = spacing_x * (columns.saturating_sub(1)) as f32;
    let x = metrics.center_x() - total_width * 0.5 + col as f32 * spacing_x;
    let y = metrics.center_y() - metrics.height() * 0.08 + row as f32 * spacing_y;
    fhre::Vec2::new(x, y)
}

fn settings_panel_size(metrics: &ShellMetrics) -> fhre::Vec2 {
    let width = (metrics.width() * 0.78).min(metrics.width() - metrics.side_inset() * 2.0);
    let row_height = settings_row_height(metrics);
    let height = row_height * SETTINGS_ACTIONS.len() as f32 + metrics.section_spacing() * 2.0;
    fhre::Vec2::new(width, height)
}

fn settings_row_height(metrics: &ShellMetrics) -> f32 {
    (metrics.height() * 0.075).clamp(34.0, 52.0)
}

fn settings_row_position(metrics: &ShellMetrics, index: u8) -> fhre::Vec2 {
    let panel_size = settings_panel_size(metrics);
    let row_height = settings_row_height(metrics);
    let top = metrics.center_y() - panel_size.y * 0.5 + metrics.section_spacing();
    fhre::Vec2::new(
        metrics.center_x(),
        top + row_height * 0.5 + index as f32 * row_height,
    )
}

fn system_nav_button_size(metrics: &ShellMetrics) -> fhre::Vec2 {
    let width = (metrics.width() * 0.17).clamp(64.0, 88.0);
    let height = (metrics.height() * 0.052).clamp(32.0, 42.0);
    fhre::Vec2::new(width, height)
}

fn system_nav_button_position(metrics: &ShellMetrics) -> fhre::Vec2 {
    let size = system_nav_button_size(metrics);
    fhre::Vec2::new(
        metrics.side_inset() + size.x * 0.5,
        metrics.panel_top_padding() + size.y * 0.5,
    )
}

fn system_surface_title_position(metrics: &ShellMetrics) -> fhre::Vec2 {
    let nav_pos = system_nav_button_position(metrics);
    fhre::Vec2::new(metrics.center_x(), nav_pos.y)
}

/// Setup the Wing shell - spawn all shell entities.
pub fn setup_wing_shell(
    mut commands: Commands,
    screen: Res<fhre::PrimaryScreen>,
    content: Res<ShellContent>,
    mut shell_state: ResMut<ShellState>,
) {
    let metrics = ShellMetrics::new(fhre::Vec2::new(screen.width as f32, screen.height as f32));
    
    let center_x = metrics.center_x();
    let width = metrics.width();
    let height = metrics.height();
    
    let card_layout = CardStackLayout::compute(content.surfaces.len(), &metrics);

    // Theme backdrop (full screen 3D shell object; extracted to FHRE texture draw).
    commands.spawn()
        .insert(ThemeBackdrop::default())
        .insert(Transform::from_position(center_x, metrics.center_y(), -0.01))
        .insert(WidgetLayoutNode { width, height });

    // Shell root
    commands.spawn()
        .insert(ShellRoot)
        .insert(Transform::from_position(center_x, metrics.center_y(), 0.0))
        .insert(WidgetLayoutNode { width, height });

    // Home surface (full screen)
    commands.spawn()
        .insert(HomeSurface::active())
        .insert(PickableBounds::from_size(width, height))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, metrics.center_y(), 0.02))
        .insert(WidgetLayoutNode { width, height });

    // Surface stack root (full screen)
    commands.spawn()
        .insert(SurfaceStackRoot)
        .insert(Transform::from_position(center_x, metrics.center_y(), 0.021))
        .insert(WidgetLayoutNode { width, height });

    // Card stack root (full screen)
    commands.spawn()
        .insert(CardStackRoot)
        .insert(PreviewSoccerBall::for_app_switcher())
        .insert(Transform::from_position(center_x, metrics.center_y(), 0.044))
        .insert(WidgetLayoutNode { width, height });

    // Overlay layer
    commands.spawn()
        .insert(OverlayLayer::hidden())
        .insert(PickableBounds::from_size(width, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, metrics.center_y(), 0.045))
        .insert(WidgetLayoutNode { width, height });

    // === Android-style Notification Panel ===
    
    // Notification panel container
    commands.spawn()
        .insert(NotificationPanel::closed())
        .insert(PickableBounds::from_size(width, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, metrics.notification_panel_height() * 0.5, 0.05))
        .insert(WidgetLayoutNode { width, height: metrics.notification_panel_height() });

    // === Layout order: Brightness (top) → QuickControls → Notification Cards ===
    
    let top_padding = metrics.panel_top_padding();
    let section_spacing = metrics.section_spacing();
    
    // 1. Brightness control (at the top)
    let brightness_y = top_padding + metrics.brightness_height() * 0.5;
    commands.spawn()
        .insert(BrightnessControl::hidden())
        .insert(PickableBounds::from_size(width - metrics.side_inset() * 2.0, metrics.brightness_height()))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, brightness_y, 0.051))
        .insert(WidgetLayoutNode { width: width - metrics.side_inset() * 2.0, height: metrics.brightness_height() });

    // 2. Quick control tiles (below brightness)
    let tile_types = [
        QuickControlType::WiFi,
        QuickControlType::Bluetooth,
        QuickControlType::AirplaneMode,
        QuickControlType::Flashlight,
        QuickControlType::Dnd,
        QuickControlType::AutoRotate,
    ];
    
    // Compute layout once and use it for both positioning and header height calculation
    let qc_layout = metrics.quick_controls_layout();
    
    // QuickControls start after brightness + spacing
    let qc_start_y = top_padding + metrics.brightness_height() + section_spacing;
    
    // Center the tile grid horizontally
    let grid_width = qc_layout.tile_size * qc_layout.columns as f32 
                   + qc_layout.tile_spacing * (qc_layout.columns.saturating_sub(1)) as f32;
    let grid_start_x = center_x - grid_width * 0.5;  // Left edge of grid
    let grid_start_y = qc_start_y;  // Top edge of grid
    
    for (i, tile_type) in tile_types.iter().enumerate() {
        // Wrap to next row if exceeds columns
        let col = i % qc_layout.columns;
        let row = i / qc_layout.columns;
        
        // tile_position returns center position
        let (tile_x, tile_y) = qc_layout.tile_position(col, row, grid_start_x, grid_start_y);
        
        let (enabled, active) = match tile_type {
            QuickControlType::WiFi => (content.quick_controls.wifi_enabled, content.quick_controls.wifi_connected),
            QuickControlType::Bluetooth => (content.quick_controls.bluetooth_enabled, content.quick_controls.bluetooth_connected),
            QuickControlType::AirplaneMode => (content.quick_controls.airplane_mode, content.quick_controls.airplane_mode),
            QuickControlType::Flashlight => (content.quick_controls.flashlight_on, content.quick_controls.flashlight_on),
            QuickControlType::Dnd => (content.quick_controls.dnd_mode, content.quick_controls.dnd_mode),
            QuickControlType::AutoRotate => (content.quick_controls.auto_rotate, content.quick_controls.auto_rotate),
            QuickControlType::BatterySaver => (content.quick_controls.battery_saver, content.quick_controls.battery_saver),
        };
        
        commands.spawn()
            .insert(QuickControlTile::new(*tile_type).with_enabled(enabled).with_active(active))
            .insert(PickableBounds::from_size(qc_layout.tile_size, qc_layout.tile_size))
            .insert(Pickable::DEFAULT)
            .insert(Transform::from_position(tile_x, tile_y, 0.051))
            .insert(WidgetLayoutNode { width: qc_layout.tile_size, height: qc_layout.tile_size });
    }

    // 3. Notification cards (below quick controls)
    // Calculate header height using the same qc_layout
    let qc_bottom = qc_start_y + qc_layout.total_height;
    let notification_layout = NotificationStackLayout::compute_with_qc_height(&metrics, qc_bottom);
    for (stack_index, notification) in content.notifications.iter().enumerate() {
        let stack_index = stack_index as u8;
        
        // Notification card
        commands.spawn()
            .insert(NotificationCard::hidden(notification.id, stack_index)
                .with_priority(into_component_priority(notification.priority)))
            .insert(PickableBounds::from_size(notification_layout.card_width, 0.0))
            .insert(Pickable::DEFAULT)
            .insert(Transform::from_position(center_x, notification_layout.card_y(stack_index), 0.052 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: notification_layout.card_width, height: notification_layout.card_height });

        // Notification title text
        let text_offset_x = notification_layout.card_width * 0.4;
        let text_size = metrics.text_size();
        commands.spawn()
            .insert(NotificationCardTitle::for_notification(notification.id))
            .insert(Transform::from_position(
                center_x - text_offset_x,
                notification_layout.card_y(stack_index) - text_size * 0.5,
                0.053 + (stack_index as f32 * 0.001)
            ))
            .insert(WidgetLayoutNode { width: notification_layout.card_width * 0.8, height: text_size });

        // Notification content text
        let text_size_small = metrics.text_size_small();
        commands.spawn()
            .insert(NotificationCardContent::for_notification(notification.id))
            .insert(Transform::from_position(
                center_x - text_offset_x,
                notification_layout.card_y(stack_index) + text_size * 0.5,
                0.053 + (stack_index as f32 * 0.001)
            ))
            .insert(WidgetLayoutNode { width: notification_layout.card_width * 0.8, height: text_size_small });
    }

    // Built-in Wing Settings surface content.
    let settings_panel_size = settings_panel_size(&metrics);
    let settings_text_size = metrics.text_size();
    let settings_text_size_small = metrics.text_size_small();
    commands.spawn()
        .insert(SettingsPanel::hidden(SETTINGS_SURFACE_ID))
        .insert(Transform::from_position(center_x, metrics.center_y(), 0.034))
        .insert(WidgetLayoutNode { width: settings_panel_size.x, height: settings_panel_size.y });

    let system_nav_size = system_nav_button_size(&metrics);
    let system_nav_pos = system_nav_button_position(&metrics);
    commands.spawn()
        .insert(SystemNavButton::hidden(SETTINGS_SURFACE_ID, SystemNavAction::BackHome))
        .insert(PickableBounds::from_size(system_nav_size.x, 0.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(system_nav_pos.x, system_nav_pos.y, 0.048))
        .insert(WidgetLayoutNode { width: system_nav_size.x, height: system_nav_size.y });

    commands.spawn()
        .insert(SystemNavButtonLabel::for_button(SETTINGS_SURFACE_ID, SystemNavAction::BackHome))
        .insert(Transform::from_position(system_nav_pos.x, system_nav_pos.y, 0.049))
        .insert(WidgetLayoutNode { width: system_nav_size.x, height: settings_text_size_small });

    let settings_row_width = settings_panel_size.x - metrics.side_inset();
    let settings_row_height = settings_row_height(&metrics);
    for (index, action) in SETTINGS_ACTIONS.iter().copied().enumerate() {
        let index = index as u8;
        let row_pos = settings_row_position(&metrics, index);
        commands.spawn()
            .insert(SettingsRow::hidden(SETTINGS_SURFACE_ID, index, action))
            .insert(PickableBounds::from_size(settings_row_width, 0.0))
            .insert(Pickable::DEFAULT)
            .insert(Transform::from_position(row_pos.x, row_pos.y, 0.036 + index as f32 * 0.001))
            .insert(WidgetLayoutNode { width: settings_row_width, height: settings_row_height });

        commands.spawn()
            .insert(SettingsRowLabel::for_row(SETTINGS_SURFACE_ID, index, action))
            .insert(Transform::from_position(
                center_x - settings_row_width * 0.42,
                row_pos.y,
                0.037 + index as f32 * 0.001,
            ))
            .insert(WidgetLayoutNode { width: settings_row_width * 0.55, height: settings_text_size });

        commands.spawn()
            .insert(SettingsRowValue::for_row(SETTINGS_SURFACE_ID, index, action))
            .insert(Transform::from_position(
                center_x + settings_row_width * 0.25,
                row_pos.y,
                0.037 + index as f32 * 0.001,
            ))
            .insert(WidgetLayoutNode { width: settings_row_width * 0.28, height: settings_text_size });
    }

    // Home launcher icons.
    for (launcher_index, app_entry) in content.apps.iter().copied().enumerate() {
        let launcher_index = launcher_index as u8;
        let launcher_pos = launcher_icon_position(&metrics, launcher_index);
        let icon_size = launcher_icon_size(&metrics);
        commands.spawn()
            .insert(LauncherIcon::hidden(app_entry.id, launcher_index).with_icon(app_entry.icon_hint))
            .insert(PickableBounds::from_size(icon_size, 0.0))
            .insert(Pickable::DEFAULT)
            .insert(Transform::from_position(launcher_pos.x, launcher_pos.y, 0.031))
            .insert(WidgetLayoutNode { width: icon_size, height: icon_size });

        commands.spawn()
            .insert(LauncherIconLabel::for_icon(app_entry.id, launcher_index))
            .insert(Transform::from_position(
                launcher_pos.x - icon_size * 0.45,
                launcher_pos.y + icon_size * 0.72,
                0.032,
            ))
            .insert(WidgetLayoutNode { width: icon_size * 1.1, height: settings_text_size_small });
    }

    // App surfaces and preview cards.
    let app_surface_width = metrics.app_surface_width();
    let app_surface_height = metrics.app_surface_height();
    
    for surface in content.surfaces.iter().copied() {
        if surface.id != HOME_SURFACE_ID {
            // App surface entity
            commands.spawn()
                .insert(AppSurface::new(surface.id))
                .insert(PickableBounds::from_size(app_surface_width, app_surface_height))
                .insert(Pickable::DEFAULT)
                .insert(Transform::from_position(center_x, metrics.center_y(), 0.03))
                .insert(WidgetLayoutNode { width: app_surface_width, height: app_surface_height });

            // Surface label text
            let title_pos = system_surface_title_position(&metrics);
            commands.spawn()
                .insert(SurfaceText::for_surface(surface.id, surface.label))
                .insert(Transform::from_position(title_pos.x, title_pos.y, 0.035))
                .insert(WidgetLayoutNode { width: 120.0, height: 16.0 });
        }

        let stack_index = surface_index(surface.id, &content).unwrap_or(0);
        
        // Surface preview card
        commands.spawn()
            .insert(SurfacePreviewCard::hidden(surface.id, stack_index)
                .with_state(surface.state)
                .with_icon(surface.icon_hint))
            .insert(PickableBounds::from_size(card_layout.card_width(stack_index), 0.0))
            .insert(Pickable::DEFAULT)
            .insert(Transform::from_position(center_x, card_layout.card_y(height, stack_index), 0.046 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: card_layout.card_width(stack_index), height: card_layout.card_height(stack_index) });

        // Card title text
        commands.spawn()
            .insert(SurfaceCardTitle::for_card(surface.id, stack_index))
            .insert(Transform::from_position(center_x - 60.0, card_layout.card_y(height, stack_index) - 12.0, 0.047 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: 120.0, height: 14.0 });

        // Card subtitle text
        commands.spawn()
            .insert(SurfaceCardSubtitle::for_card(surface.id, stack_index))
            .insert(Transform::from_position(center_x - 40.0, card_layout.card_y(height, stack_index) + 8.0, 0.0475 + (stack_index as f32 * 0.001)))
            .insert(WidgetLayoutNode { width: 100.0, height: 12.0 });
    }

    // Shell label text
    commands.spawn()
        .insert(SurfaceText::for_surface(HOME_SURFACE_ID, "Wing"))
        .insert(Transform::from_position(32.0, 14.0, 0.045))
        .insert(WidgetLayoutNode { width: 80.0, height: 16.0 });

    // Home surface preview title
    if let Some(surface) = content.surfaces.first() {
        commands.spawn()
            .insert(SurfaceText::for_surface(HOME_SURFACE_ID, surface.preview_title))
            .insert(Transform::from_position(center_x, metrics.center_y() - 40.0, 0.057))
            .insert(WidgetLayoutNode { width: 96.0, height: 16.0 });
    }

    shell_state.active_surface = Some(HOME_SURFACE_ID);
    shell_state.overlay_mode = ShellOverlayMode::None;
    shell_state.overlay_origin_surface = None;
}

fn surface_index(surface_id: SurfaceId, content: &ShellContent) -> Option<u8> {
    content.surfaces.iter().position(|s| s.id == surface_id).map(|i| i as u8)
}

/// Convert resource NotificationPriority to component NotificationPriority.
fn into_component_priority(p: crate::resources::NotificationPriority) -> crate::components::NotificationPriority {
    match p {
        crate::resources::NotificationPriority::Low => crate::components::NotificationPriority::Low,
        crate::resources::NotificationPriority::Normal => crate::components::NotificationPriority::Normal,
        crate::resources::NotificationPriority::High => crate::components::NotificationPriority::High,
    }
}

/// Handle shell interactions - clicks on StatusBar, BottomBar, GestureZone, etc.
pub fn wing_shell_interaction_system(
    events: Res<fhre::Events>,
    mut content: ResMut<ShellContent>,
    metrics: Res<ShellMetrics>,
    animation: Res<ShellOverlayAnimation>,
    mut shell_state: ResMut<ShellState>,
    mut overlay_query: Query<&OverlayLayer>,
    mut launcher_query: Query<&LauncherIcon>,
    mut notification_panel_query: Query<&NotificationPanel>,
    mut preview_query: Query<&PreviewSoccerBall>,
) {
    if let Some(click_events) = events.get_events_current::<fhre::Pointer<fhre::Click>>() {
        for event in click_events {
            if shell_state.app_switcher_open() {
                let target_surface = match content.preview_effect {
                    PreviewEffect::Card => {
                        hit_preview_card_surface(event.pointer_location.position, &content, &metrics, &animation)
                    }
                    PreviewEffect::Soccer => hit_preview_football_surface(
                        event.pointer_location.position,
                        &content,
                        &metrics,
                        &animation,
                        &mut preview_query,
                    ),
                };

                if let Some(surface_id) = target_surface {
                    activate_surface(surface_id, &mut content, &mut shell_state);
                    shell_state.dismiss_overlay();
                    continue;
                }
            }

            // Overlay layer click: close any overlay
            if overlay_query.get_pair(event.entity).is_some() {
                shell_state.close_overlay_to_origin();
                continue;
            }
            // Notification panel click: close notification panel
            if notification_panel_query.get_pair(event.entity).is_some() {
                shell_state.close_overlay_to_origin();
                continue;
            }
            // Launcher icon click: activate app surface from Home.
            if shell_state.active_surface == Some(HOME_SURFACE_ID) {
                if let Some((_, icon)) = launcher_query.get_pair(event.entity) {
                    launch_app(icon.app_id, &mut content, &mut shell_state);
                    shell_state.dismiss_overlay();
                    continue;
                }
            }
        }
    }
}

/// Handle clicks inside the built-in Settings surface.
pub fn wing_settings_interaction_system(
    events: Res<fhre::Events>,
    mut content: ResMut<ShellContent>,
    mut theme_state: ResMut<ThemeState>,
    mut shell_state: ResMut<ShellState>,
    mut nav_button_query: Query<&SystemNavButton>,
    mut settings_row_query: Query<&SettingsRow>,
) {
    if shell_state.active_surface != Some(SETTINGS_SURFACE_ID) {
        return;
    }

    if let Some(click_events) = events.get_events_current::<fhre::Pointer<fhre::Click>>() {
        for event in click_events {
            if let Some((_, button)) = nav_button_query.get_pair(event.entity) {
                match button.action {
                    SystemNavAction::BackHome => {
                        shell_state.active_surface = Some(HOME_SURFACE_ID);
                        shell_state.dismiss_overlay();
                    }
                }
                continue;
            }

            if let Some((_, row)) = settings_row_query.get_pair(event.entity) {
                apply_settings_action(row.action, &mut content, &mut theme_state);
            }
        }
    }
}

fn launch_app(app_id: u32, content: &mut ShellContent, shell_state: &mut ShellState) {
    let Some(app) = content.get_app(app_id).copied() else {
        return;
    };

    match app.launch {
        WingAppLaunchKind::BuiltInSurface { surface_id } => {
            if content.get_surface(surface_id).is_some() {
                shell_state.active_surface = Some(surface_id);
                content.update_app_state(app_id, SurfaceState::Running);
                content.update_surface_state(surface_id, SurfaceState::Running);
            }
        }
        WingAppLaunchKind::NuttXBuiltin { surface_id, command } => {
            if content.get_surface(surface_id).is_none() {
                let surface = ShellSurfaceEntry::new(surface_id, app.label, app.preview_title)
                    .with_state(app.state)
                    .with_icon(app.icon_hint);
                content.add_surface(surface);
            }

            if app.state == SurfaceState::Running && app.pid > 0 {
                shell_state.active_surface = Some(surface_id);
                content.update_surface_state(surface_id, SurfaceState::Running);
                return;
            }

            match platform::launch_nuttx_builtin(command) {
                Ok(pid) => {
                    shell_state.active_surface = Some(surface_id);
                    content.update_app_state(app_id, SurfaceState::Running);
                    content.update_app_pid(app_id, pid);
                    content.update_surface_state(surface_id, SurfaceState::Running);
                }
                Err(_) => {
                    content.update_app_state(app_id, SurfaceState::Closed);
                    content.update_app_pid(app_id, -1);
                    content.update_surface_state(surface_id, SurfaceState::Closed);
                }
            }
        }
    }
}

fn activate_surface(surface_id: SurfaceId, content: &mut ShellContent, shell_state: &mut ShellState) {
    if surface_id == HOME_SURFACE_ID {
        shell_state.active_surface = Some(HOME_SURFACE_ID);
        return;
    }

    if let Some(app) = content.get_app_for_surface(surface_id).copied() {
        launch_app(app.id, content, shell_state);
        return;
    }

    if content.get_surface(surface_id).is_some() {
        shell_state.active_surface = Some(surface_id);
        content.update_surface_state(surface_id, SurfaceState::Running);
    }
}

/// Reconcile Wing's app model with NuttX tasks launched from the shell.
pub fn wing_app_lifecycle_system(
    mut content: ResMut<ShellContent>,
    mut shell_state: ResMut<ShellState>,
) {
    for index in 0..content.apps.len() {
        let app = content.apps[index];
        let WingAppLaunchKind::NuttXBuiltin { surface_id, .. } = app.launch else {
            continue;
        };

        if app.state != SurfaceState::Running || app.pid <= 0 {
            continue;
        }

        if platform::poll_nuttx_task(app.pid) == TaskStatus::Exited {
            content.update_app_state(app.id, SurfaceState::Closed);
            content.update_app_pid(app.id, -1);
            content.update_surface_state(surface_id, SurfaceState::Closed);
            if shell_state.active_surface == Some(surface_id) {
                shell_state.active_surface = Some(HOME_SURFACE_ID);
            }
        }
    }
}

fn apply_settings_action(
    action: SettingsAction,
    content: &mut ShellContent,
    theme_state: &mut ThemeState,
) {
    match action {
        SettingsAction::ToggleWifi => content.toggle_wifi(),
        SettingsAction::ToggleBluetooth => content.toggle_bluetooth(),
        SettingsAction::ToggleAirplaneMode => content.toggle_airplane_mode(),
        SettingsAction::ToggleFlashlight => content.toggle_flashlight(),
        SettingsAction::ToggleDnd => {
            content.quick_controls.dnd_mode = !content.quick_controls.dnd_mode;
        }
        SettingsAction::ToggleAutoRotate => {
            content.quick_controls.auto_rotate = !content.quick_controls.auto_rotate;
        }
        SettingsAction::CycleBrightness => {
            let next = if content.quick_controls.brightness >= 0.95 {
                0.25
            } else {
                content.quick_controls.brightness + 0.25
            };
            content.set_brightness(next);
        }
        SettingsAction::CycleTheme => theme_state.switch_next_theme(),
        SettingsAction::CyclePreviewEffect => content.cycle_preview_effect(),
    }
}

fn hit_preview_card_surface(
    point: fhre::Vec2,
    content: &ShellContent,
    metrics: &ShellMetrics,
    animation: &ShellOverlayAnimation,
) -> Option<SurfaceId> {
    let progress = animation.app_switcher_eased();
    if progress <= 0.001 || content.surfaces.is_empty() {
        return None;
    }

    let card_layout = CardStackLayout::compute(content.surfaces.len(), metrics);
    let start_y = metrics.height() + 50.0;

    for (surface_index, surface) in content.surfaces.iter().enumerate().rev() {
        let stack_index = surface_index as u8;
        let card_width = card_layout.card_width(stack_index);
        let card_height = card_layout.card_height(stack_index);
        let target_y = card_layout.card_y(metrics.height(), stack_index);
        let card_y = start_y + (target_y - start_y) * progress;
        let half_width = card_width * 0.5;
        let half_height = card_height * 0.5;

        if point.x >= metrics.center_x() - half_width
            && point.x <= metrics.center_x() + half_width
            && point.y >= card_y - half_height
            && point.y <= card_y + half_height
        {
            return Some(surface.id);
        }
    }

    None
}

fn hit_preview_football_surface(
    point: fhre::Vec2,
    content: &ShellContent,
    metrics: &ShellMetrics,
    animation: &ShellOverlayAnimation,
    preview_query: &mut Query<&PreviewSoccerBall>,
) -> Option<SurfaceId> {
    let progress = animation.app_switcher_eased();
    if progress <= 0.001 || content.surfaces.is_empty() {
        return None;
    }

    let screen_size = fhre::Vec2::new(metrics.width(), metrics.height());
    let center = app_switcher_ball_center(screen_size, progress);
    let base_size = app_switcher_ball_size(screen_size, progress);

    for (_, ball) in preview_query.iter() {
        let faces = ball.projected_faces(center, base_size, 255);
        for face in faces.iter().rev() {
            let Some(surface_index) = surface_index_for_face(face.face_index, content.surfaces.len()) else {
                continue;
            };

            if point_in_polygon(point, &face.vertices) {
                return Some(content.surfaces[surface_index].id);
            }
        }
        break;
    }

    None
}

/// Update shell layout - positions and sizes of shell chrome elements.
pub fn wing_shell_layout_system(
    metrics: Res<ShellMetrics>,
    shell_state: Res<ShellState>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut home_query: Query<&mut HomeSurface>,
    mut app_query: Query<&mut AppSurface>,
) {
    let width = metrics.width();
    let height = metrics.height();
    let center_x = metrics.center_x();
    let center_y = metrics.center_y();
    let app_surface_width = metrics.app_surface_width();
    let app_surface_height = metrics.app_surface_height();

    for (entity, home_surface) in home_query.iter_mut() {
        home_surface.active = shell_state.active_surface == Some(HOME_SURFACE_ID);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = center_y;
            transform.position.z = if home_surface.active { 0.02 } else { 0.01 };
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width;
            bounds.height = height;
        }
    }

    for (entity, app_surface) in app_query.iter_mut() {
        app_surface.active = shell_state.active_surface == Some(app_surface.id);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = metrics.center_y();
            transform.position.z = if app_surface.active { 0.03 } else { 0.015 };
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = app_surface_width;
            bounds.height = if app_surface.active { app_surface_height } else { 0.0 };
        }
    }
}

pub fn wing_theme_backdrop_layout_system(
    metrics: Res<ShellMetrics>,
    theme_state: Res<ThemeState>,
    mut backdrop_query: Query<(&mut Transform, &mut WidgetLayoutNode, &mut ThemeBackdrop)>,
) {
    for (_, (transform, layout, backdrop)) in backdrop_query.iter_mut() {
        transform.position.x = metrics.center_x();
        transform.position.y = metrics.center_y();
        transform.position.z = -0.01;
        layout.width = metrics.width();
        layout.height = metrics.height();
        backdrop.visible = true;
        backdrop.variant = theme_state.variant;
    }
}

/// Update Home launcher icon layout.
pub fn wing_launcher_layout_system(
    metrics: Res<ShellMetrics>,
    shell_state: Res<ShellState>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut icon_query: Query<&mut LauncherIcon>,
    mut label_query: Query<&LauncherIconLabel>,
) {
    let visible = shell_state.active_surface == Some(HOME_SURFACE_ID);
    let icon_size = launcher_icon_size(&metrics);

    for (entity, icon) in icon_query.iter_mut() {
        icon.visible = visible;
        let pos = launcher_icon_position(&metrics, icon.index);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = pos.x;
            transform.position.y = pos.y;
            transform.position.z = if visible { 0.031 } else { 0.012 };
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = icon_size;
            bounds.height = if visible { icon_size } else { 0.0 };
        }
    }

    for (entity, label) in label_query.iter() {
        let pos = launcher_icon_position(&metrics, label.index);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = pos.x - icon_size * 0.45;
            transform.position.y = pos.y + icon_size * 0.72;
            transform.position.z = if visible { 0.032 } else { 0.012 };
        }
    }
}

/// Update built-in Settings surface layout.
pub fn wing_settings_layout_system(
    metrics: Res<ShellMetrics>,
    shell_state: Res<ShellState>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut panel_query: Query<&mut SettingsPanel>,
    mut nav_button_query: Query<&mut SystemNavButton>,
    mut nav_label_query: Query<&SystemNavButtonLabel>,
    mut row_query: Query<&mut SettingsRow>,
    mut label_query: Query<&SettingsRowLabel>,
    mut value_query: Query<&SettingsRowValue>,
) {
    let active = shell_state.active_surface == Some(SETTINGS_SURFACE_ID);
    let panel_size = settings_panel_size(&metrics);
    let nav_size = system_nav_button_size(&metrics);
    let nav_pos = system_nav_button_position(&metrics);
    let row_width = panel_size.x - metrics.side_inset();
    let row_height = settings_row_height(&metrics);

    for (entity, panel) in panel_query.iter_mut() {
        panel.visible = active;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = metrics.center_x();
            transform.position.y = metrics.center_y();
            transform.position.z = if active { 0.034 } else { 0.012 };
        }
    }

    for (entity, button) in nav_button_query.iter_mut() {
        button.visible = shell_state.active_surface == Some(button.surface_id);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = nav_pos.x;
            transform.position.y = nav_pos.y;
            transform.position.z = if button.visible { 0.048 } else { 0.012 };
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = nav_size.x;
            bounds.height = if button.visible { nav_size.y } else { 0.0 };
        }
    }

    for (entity, label) in nav_label_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = nav_pos.x;
            transform.position.y = nav_pos.y;
            transform.position.z = if shell_state.active_surface == Some(label.surface_id) {
                0.049
            } else {
                0.012
            };
        }
    }

    for (entity, row) in row_query.iter_mut() {
        row.visible = active;
        let pos = settings_row_position(&metrics, row.index);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = pos.x;
            transform.position.y = pos.y;
            transform.position.z = if active { 0.036 + row.index as f32 * 0.001 } else { 0.012 };
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = row_width;
            bounds.height = if active { row_height } else { 0.0 };
        }
    }

    for (entity, label) in label_query.iter() {
        let pos = settings_row_position(&metrics, label.index);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = metrics.center_x() - row_width * 0.42;
            transform.position.y = pos.y;
            transform.position.z = if active { 0.037 + label.index as f32 * 0.001 } else { 0.012 };
        }
    }

    for (entity, value) in value_query.iter() {
        let pos = settings_row_position(&metrics, value.index);
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = metrics.center_x() + row_width * 0.25;
            transform.position.y = pos.y;
            transform.position.z = if active { 0.037 + value.index as f32 * 0.001 } else { 0.012 };
        }
    }
}

/// Update stack layout positions.
pub fn wing_shell_stack_layout_system(
    metrics: Res<ShellMetrics>,
    mut transform_query: Query<&mut Transform>,
    mut stack_query: Query<&SurfaceStackRoot>,
    mut card_stack_query: Query<&CardStackRoot>,
) {
    let center_x = metrics.center_x();
    
    for (entity, _) in stack_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = metrics.center_y();
        }
    }

    for (entity, _) in card_stack_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = metrics.center_y();
        }
    }
}

/// Update overlay layer layout.
pub fn wing_shell_overlay_layout_system(
    metrics: Res<ShellMetrics>,
    shell_state: Res<ShellState>,
    animation: Res<ShellOverlayAnimation>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut overlay_query: Query<&mut OverlayLayer>,
) {
    let width = metrics.width();
    let height = metrics.height();
    let center_x = metrics.center_x();
    let center_y = metrics.center_y();

    let notification_progress = animation.notification_panel_eased();
    let app_progress = animation.app_switcher_eased();

    // Overlay layer
    for (entity, overlay) in overlay_query.iter_mut() {
        overlay.visible = shell_state.overlay_visible() || animation.is_animating();
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            transform.position.y = center_y;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width;
            bounds.height = height * notification_progress.max(app_progress);
        }
    }
}

/// Update notification panel container layout.
pub fn wing_shell_notification_panel_layout_system(
    metrics: Res<ShellMetrics>,
    shell_state: Res<ShellState>,
    animation: Res<ShellOverlayAnimation>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut notification_panel_query: Query<&mut NotificationPanel>,
) {
    let width = metrics.width();
    let center_x = metrics.center_x();

    let notification_progress = animation.notification_panel_eased();

    // Notification panel container
    for (entity, panel) in notification_panel_query.iter_mut() {
        panel.open = shell_state.notification_panel_open() || notification_progress > 0.0;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            // Panel slides down from top of screen
            let target_y = metrics.notification_panel_height() * 0.5;
            let start_y = -metrics.notification_panel_height();
            transform.position.y = start_y + (target_y - start_y) * notification_progress;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = width;
            bounds.height = metrics.notification_panel_height() * notification_progress;
        }
    }
}

/// Update quick controls (tiles and brightness) layout.
pub fn wing_shell_quick_controls_layout_system(
    metrics: Res<ShellMetrics>,
    shell_state: Res<ShellState>,
    animation: Res<ShellOverlayAnimation>,
    content: Res<ShellContent>,
    mut transform_query: Query<&mut Transform>,
    mut brightness_query: Query<&mut BrightnessControl>,
    mut tile_query: Query<&mut QuickControlTile>,
) {
    let center_x = metrics.center_x();
    let notification_progress = animation.notification_panel_eased();

    let top_padding = metrics.panel_top_padding();
    let section_spacing = metrics.section_spacing();

    // Brightness control (at the top)
    for (entity, brightness) in brightness_query.iter_mut() {
        brightness.visible = shell_state.notification_panel_open() || notification_progress > 0.0;
        brightness.brightness = content.quick_controls.brightness;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            let brightness_y = top_padding + metrics.brightness_height() * 0.5;
            transform.position.y = brightness_y;
        }
    }

    // Quick control tiles (below brightness)
    let qc_layout = metrics.quick_controls_layout();
    
    // QuickControls start after brightness + spacing
    let qc_start_y = top_padding + metrics.brightness_height() + section_spacing;
    
    // Center the tile grid horizontally
    let grid_width = qc_layout.tile_size * qc_layout.columns as f32 
                   + qc_layout.tile_spacing * (qc_layout.columns.saturating_sub(1)) as f32;
    let grid_start_x = center_x - grid_width * 0.5;  // Left edge of grid
    let grid_start_y = qc_start_y;  // Top edge of grid
    
    let tile_types = [
        QuickControlType::WiFi,
        QuickControlType::Bluetooth,
        QuickControlType::AirplaneMode,
        QuickControlType::Flashlight,
        QuickControlType::Dnd,
        QuickControlType::AutoRotate,
    ];
    
    for (entity, tile) in tile_query.iter_mut() {
        tile.enabled = match tile.tile_type {
            QuickControlType::WiFi => content.quick_controls.wifi_enabled,
            QuickControlType::Bluetooth => content.quick_controls.bluetooth_enabled,
            QuickControlType::AirplaneMode => content.quick_controls.airplane_mode,
            QuickControlType::Flashlight => content.quick_controls.flashlight_on,
            QuickControlType::Dnd => content.quick_controls.dnd_mode,
            QuickControlType::AutoRotate => content.quick_controls.auto_rotate,
            QuickControlType::BatterySaver => content.quick_controls.battery_saver,
        };
        tile.active = tile.enabled;

        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            let tile_index = tile_types.iter().position(|t| *t == tile.tile_type).unwrap_or(0);
            let col = tile_index % qc_layout.columns;
            let row = tile_index / qc_layout.columns;
            
            // tile_position returns center position
            let (tile_x, tile_y) = qc_layout.tile_position(col, row, grid_start_x, grid_start_y);
            
            transform.position.x = tile_x;
            transform.position.y = tile_y;
        }
    }
}

/// Update notification cards layout.
pub fn wing_shell_notification_cards_layout_system(
    metrics: Res<ShellMetrics>,
    shell_state: Res<ShellState>,
    animation: Res<ShellOverlayAnimation>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut notification_card_query: Query<&mut NotificationCard>,
) {
    let center_x = metrics.center_x();

    let notification_progress = animation.notification_panel_eased();

    // Calculate QuickControls bottom position
    let top_padding = metrics.panel_top_padding();
    let section_spacing = metrics.section_spacing();
    let qc_layout = metrics.quick_controls_layout();
    let qc_bottom = top_padding + metrics.brightness_height() + section_spacing + qc_layout.total_height;
    
    // Notification cards
    let notification_layout = NotificationStackLayout::compute_with_qc_height(&metrics, qc_bottom);
    for (entity, card) in notification_card_query.iter_mut() {
        card.visible = shell_state.notification_panel_open() || notification_progress > 0.0;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            let target_y = notification_layout.card_y(card.stack_index);
            let start_y = -notification_layout.card_height;
            transform.position.y = start_y + (target_y - start_y) * notification_progress;
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = notification_layout.card_width;
            bounds.height = notification_layout.card_height * notification_progress;
        }
    }
}

/// Update surface preview card layout (AppSwitcher).
pub fn wing_shell_overlay_card_layout_system(
    metrics: Res<ShellMetrics>,
    content: Res<ShellContent>,
    shell_state: Res<ShellState>,
    animation: Res<ShellOverlayAnimation>,
    mut transform_query: Query<&mut Transform>,
    mut bounds_query: Query<&mut fhre::PickableBounds>,
    mut card_query: Query<&mut SurfacePreviewCard>,
    mut title_query: Query<&SurfaceCardTitle>,
    mut subtitle_query: Query<&SurfaceCardSubtitle>,
) {
    let height = metrics.height();
    let center_x = metrics.center_x();

    let card_layout = CardStackLayout::compute(content.surfaces.len(), &metrics);
    let app_progress = animation.app_switcher_eased();
    let show_cards = content.preview_effect == PreviewEffect::Card;
    let start_y = height + 50.0;

    for (entity, card) in card_query.iter_mut() {
        card.visible = shell_state.app_switcher_open() || app_progress > 0.0;
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            transform.position.x = center_x;
            let target_y = card_layout.card_y(height, card.stack_index);
            transform.position.y = start_y + (target_y - start_y) * app_progress;
            transform.position.z = 0.046 + (card.stack_index as f32 * 0.001);
        }
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            bounds.width = if show_cards && card.visible {
                card_layout.card_width(card.stack_index)
            } else {
                0.0
            };
            bounds.height = if show_cards && card.visible {
                card_layout.card_height(card.stack_index) * app_progress
            } else {
                0.0
            };
        }
    }

    for (entity, title) in title_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            let card_width = card_layout.card_width(title.stack_index);
            let card_height = card_layout.card_height(title.stack_index);
            let target_y = card_layout.card_y(height, title.stack_index);
            let card_y = start_y + (target_y - start_y) * app_progress;
            transform.position.x = center_x - card_width * 0.38;
            transform.position.y = card_y - card_height * 0.16;
            transform.position.z = 0.047 + (title.stack_index as f32 * 0.001);
        }
    }

    for (entity, subtitle) in subtitle_query.iter() {
        if let Some((_, transform)) = transform_query.get_pair_mut(entity) {
            let card_width = card_layout.card_width(subtitle.stack_index);
            let card_height = card_layout.card_height(subtitle.stack_index);
            let target_y = card_layout.card_y(height, subtitle.stack_index);
            let card_y = start_y + (target_y - start_y) * app_progress;
            transform.position.x = center_x - card_width * 0.38;
            transform.position.y = card_y + card_height * 0.08;
            transform.position.z = 0.0475 + (subtitle.stack_index as f32 * 0.001);
        }
    }
}
