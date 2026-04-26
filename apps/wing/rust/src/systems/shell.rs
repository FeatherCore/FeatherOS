//! Shell setup and layout systems.

use fhre::{Commands, Pickable, PickableBounds, Query, Res, ResMut, Transform};

use crate::types::SurfaceId;
use crate::components::{
    AppSurface, BrightnessControl, CardStackRoot, HomeSurface, 
    NotificationCard, NotificationCardContent, NotificationCardTitle, NotificationPanel, 
    OverlayLayer, QuickControlTile, QuickControlType, ShellRoot, 
    SurfaceCardSubtitle, SurfaceCardTitle, SurfacePreviewCard, 
    SurfaceStackRoot, SurfaceText, WidgetLayoutNode,
};
use crate::resources::{QuickControlsLayout, ShellContent, ShellMetrics, ShellOverlayAnimation, ShellOverlayMode, ShellState};

const HOME_SURFACE_ID: SurfaceId = 1;

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

    // App surfaces and preview cards
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
            commands.spawn()
                .insert(SurfaceText::for_surface(surface.id, surface.label))
                .insert(Transform::from_position(center_x, metrics.center_y() + app_surface_height * 0.5 + 16.0, 0.035))
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
        .insert(SurfaceText::shell("Wing"))
        .insert(Transform::from_position(32.0, 14.0, 0.045))
        .insert(WidgetLayoutNode { width: 80.0, height: 16.0 });

    // Home surface preview title
    if let Some(surface) = content.surfaces.first() {
        commands.spawn()
            .insert(SurfaceText::shell(surface.preview_title))
            .insert(Transform::from_position(center_x, metrics.center_y() - 40.0, 0.057))
            .insert(WidgetLayoutNode { width: 96.0, height: 16.0 });
    }

    shell_state.active_surface = Some(HOME_SURFACE_ID);
    shell_state.overlay_mode = ShellOverlayMode::None;
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
    mut shell_state: ResMut<ShellState>,
    mut overlay_query: Query<&OverlayLayer>,
    mut card_query: Query<&SurfacePreviewCard>,
    mut app_surface_query: Query<&AppSurface>,
    mut notification_panel_query: Query<&NotificationPanel>,
) {
    if let Some(click_events) = events.get_events_current::<fhre::Pointer<fhre::Click>>() {
        for event in click_events {
            // Overlay layer click: close any overlay
            if overlay_query.get_pair(event.entity).is_some() {
                shell_state.overlay_mode = ShellOverlayMode::None;
                continue;
            }
            // Notification panel click: close notification panel
            if notification_panel_query.get_pair(event.entity).is_some() {
                shell_state.overlay_mode = ShellOverlayMode::None;
                continue;
            }
            // Surface preview card click: activate surface
            if let Some((_, card)) = card_query.get_pair(event.entity) {
                shell_state.active_surface = Some(card.surface_id);
                shell_state.overlay_mode = ShellOverlayMode::None;
                continue;
            }
            // App surface click: activate surface
            if let Some((_, surface)) = app_surface_query.get_pair(event.entity) {
                shell_state.active_surface = Some(surface.id);
                if shell_state.overlay_visible() {
                    shell_state.overlay_mode = ShellOverlayMode::None;
                }
            }
        }
    }
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
) {
    let height = metrics.height();
    let center_x = metrics.center_x();

    let card_layout = CardStackLayout::compute(content.surfaces.len(), &metrics);
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
