use alloc::{format, string::String};

use fhre::{render_world::ExtractedUI, Color, Handle, Image, MainWorld, RenderWorld, Transform};

use crate::components::{
    AppSurface, BrightnessControl, CardStackRoot, HomeSurface,
    LauncherIcon, LauncherIconLabel, NotificationCard, NotificationCardContent,
    NotificationCardTitle, NotificationPanel, OverlayLayer, QuickControlTile,
    SettingsAction, SettingsPanel, SettingsRow, SettingsRowLabel, SettingsRowValue,
    ShellRoot,
    SurfaceCardSubtitle, SurfaceCardTitle, SurfacePreviewCard,
    SurfaceStackRoot, SurfaceText, SystemNavButton, SystemNavButtonLabel, ThemeBackdrop,
    WidgetLayoutNode,
};
use crate::resources::{
    PreviewEffect, ShellContent, ShellOverlayAnimation, ShellState, SurfaceState, ThemeState,
    ThemeVariant, WingImageResources,
};
use crate::types::SurfaceId;

const HOME_SURFACE_ID: SurfaceId = 1;
const SETTINGS_SURFACE_ID: SurfaceId = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedShellText {
    pub position: fhre::Vec2,
    pub text: String,
    pub color: fhre::Color,
    pub size: f32,
}

impl fhre::Component for ExtractedShellText {
    fn type_name() -> &'static str { "ExtractedShellText" }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedThemeBackdrop {
    pub position: fhre::Vec2,
    pub width: f32,
    pub height: f32,
    pub variant: ThemeVariant,
    pub palette: crate::ThemePalette,
    pub image: Handle<Image>,
}

impl fhre::Component for ExtractedThemeBackdrop {
    fn type_name() -> &'static str { "ExtractedThemeBackdrop" }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedShellImage {
    pub position: fhre::Vec2,
    pub width: f32,
    pub height: f32,
    pub color: fhre::Color,
    pub image: Handle<Image>,
    pub sort_key: i32,
}

impl fhre::Component for ExtractedShellImage {
    fn type_name() -> &'static str { "ExtractedShellImage" }
}

/// Extract shell UI components to render world.
/// 
/// This extractor reads shell state from MainWorld and creates render components.
/// Components are conditionally added/removed based on visibility state.
pub fn extract_wing_shell(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let palette = main_world
        .resources()
        .get::<ThemeState>()
        .map(|theme| theme.current.shell)
        .unwrap_or(crate::shell_palette());
    let active_surface = main_world
        .resources()
        .get::<ShellState>()
        .and_then(|state| state.active_surface);
    let overlay_alpha = main_world
        .resources()
        .get::<ShellOverlayAnimation>()
        .map(|anim| anim.overlay_alpha_eased())
        .unwrap_or(0.0);
    let notification_alpha = main_world
        .resources()
        .get::<ShellOverlayAnimation>()
        .map(|anim| anim.notification_panel_eased())
        .unwrap_or(0.0);
    let app_switcher_alpha = main_world
        .resources()
        .get::<ShellOverlayAnimation>()
        .map(|anim| anim.app_switcher_eased())
        .unwrap_or(0.0);
    let notification_panel_open = main_world
        .resources()
        .get::<ShellState>()
        .map(|state| state.notification_panel_open())
        .unwrap_or(false);
    let app_switcher_open = main_world
        .resources()
        .get::<ShellState>()
        .map(|state| state.app_switcher_open())
        .unwrap_or(false);
    let preview_effect = main_world
        .resources()
        .get::<ShellContent>()
        .map(|content| content.preview_effect)
        .unwrap_or_default();
    let notification_panel_visible = notification_panel_open || notification_alpha > 0.001;
    let app_switcher_visible = app_switcher_open || app_switcher_alpha > 0.001;
    let card_preview_visible = app_switcher_visible && preview_effect == PreviewEffect::Card;
    let base_content_visible = !notification_panel_visible && !app_switcher_visible;
    let home_active = active_surface == Some(HOME_SURFACE_ID);
    let settings_active = active_surface == Some(SETTINGS_SURFACE_ID);
    let image_resources = main_world
        .resources()
        .get::<WingImageResources>()
        .filter(|images| images.loaded);

    // === Extract theme backdrop ===

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(backdrop) = main_world.get_component::<ThemeBackdrop>(entity) else {
            continue;
        };
        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };

        let render_entity = render_world.get_or_spawn_synced(entity);
        let image = image_resources
            .map(|images| images.theme_background(backdrop.variant).clone());

        if backdrop.visible {
            if let Some(image) = image {
                render_world.insert_component(
                    render_entity,
                    ExtractedThemeBackdrop {
                        position: transform.xy(),
                        width: layout.width,
                        height: layout.height,
                        variant: backdrop.variant,
                        palette,
                        image,
                    },
                );
            } else {
                render_world.remove_component::<ExtractedThemeBackdrop>(render_entity);
            }
        } else {
            render_world.remove_component::<ExtractedThemeBackdrop>(render_entity);
        }
    }

    // === Extract UI rectangles ===
    
    for (entity, transform) in main_world.query::<Transform>() {
        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };

        // Determine if this UI element should be rendered
        let (should_render, render_color) = if main_world.get_component::<ShellRoot>(entity).is_some() {
            (false, None)
        } else if main_world.get_component::<SurfaceStackRoot>(entity).is_some() {
            (false, None)  // Invisible container
        } else if main_world.get_component::<CardStackRoot>(entity).is_some() {
            (false, None)  // Invisible container
        } else if let Some(home_surface) = main_world.get_component::<HomeSurface>(entity) {
            if home_surface.active && base_content_visible { (false, None) } else { (false, None) }
        } else if let Some(icon) = main_world.get_component::<LauncherIcon>(entity) {
            if icon.visible && base_content_visible {
                (true, Some(Color::new(palette.accent.r, palette.accent.g, palette.accent.b, 220)))
            } else { (false, None) }
        } else if let Some(overlay) = main_world.get_component::<OverlayLayer>(entity) {
            if overlay.visible { 
                (true, Some(Color::new(10, 16, 32, (160.0 * overlay_alpha) as u8)))
            } else { (false, None) }
        } else if let Some(card) = main_world.get_component::<SurfacePreviewCard>(entity) {
            if card.visible && card_preview_visible {
                let alpha = (235.0 * app_switcher_alpha.clamp(0.0, 1.0)) as u8;
                (true, Some(preview_card_color(card.stack_index, card.state, alpha, palette)))
            } else { (false, None) }
        // Notification panel container
        } else if let Some(panel) = main_world.get_component::<NotificationPanel>(entity) {
            if panel.open { 
                let base = palette.overlay;
                (true, Some(Color::new(base.r, base.g, base.b, (base.a as f32 * notification_alpha) as u8)))
            } else { (false, None) }
        // Quick control tile
        } else if let Some(tile) = main_world.get_component::<QuickControlTile>(entity) {
            if notification_panel_visible {
                let tile_color = if tile.active { palette.accent } else { palette.surface_alt };
                (true, Some(Color::new(tile_color.r, tile_color.g, tile_color.b, (255.0 * notification_alpha) as u8)))
            } else { (false, None) }
        // Brightness control
        } else if let Some(brightness) = main_world.get_component::<BrightnessControl>(entity) {
            if brightness.visible {
                (true, Some(Color::new(palette.surface_alt.r, palette.surface_alt.g, palette.surface_alt.b, (200.0 * notification_alpha) as u8)))
            } else { (false, None) }
        // Notification card
        } else if let Some(card) = main_world.get_component::<NotificationCard>(entity) {
            if card.visible {
                let base_color = match card.priority {
                    crate::components::NotificationPriority::High => palette.accent,
                    _ => palette.surface,
                };
                (true, Some(Color::new(base_color.r, base_color.g, base_color.b, (255.0 * notification_alpha) as u8)))
            } else { (false, None) }
        } else if let Some(panel) = main_world.get_component::<SettingsPanel>(entity) {
            if panel.visible && base_content_visible {
                let base = palette.surface;
                (true, Some(Color::new(base.r, base.g, base.b, 238)))
            } else { (false, None) }
        } else if let Some(button) = main_world.get_component::<SystemNavButton>(entity) {
            if button.visible && base_content_visible {
                let base = palette.accent;
                (true, Some(Color::new(base.r, base.g, base.b, 235)))
            } else { (false, None) }
        } else if let Some(row) = main_world.get_component::<SettingsRow>(entity) {
            if row.visible && base_content_visible {
                let active = main_world
                    .resources()
                    .get::<ShellContent>()
                    .map(|content| row.action.is_active(content))
                    .unwrap_or(false);
                let base = if active { palette.accent } else { palette.surface };
                (true, Some(Color::new(base.r, base.g, base.b, 235)))
            } else { (false, None) }
        } else if let Some(app_surface) = main_world.get_component::<AppSurface>(entity) {
            if app_surface.active && base_content_visible {
                let base = palette.background;
                (true, Some(Color::new(base.r, base.g, base.b, 218)))
            } else { (false, None) }
        } else {
            (false, None)
        };

        let render_entity = render_world.get_or_spawn_synced(entity);
        
        if should_render {
            if let Some(color) = render_color {
                render_world.insert_component(
                    render_entity,
                    ExtractedUI {
                        position: transform.xy(),
                        width: layout.width,
                        height: layout.height,
                        color,
                    },
                );
            }
        } else {
            // Remove component when not visible to avoid stale rendering
            render_world.remove_component::<ExtractedUI>(render_entity);
        }
    }

    // === Extract image-backed shell glyphs ===

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };

        let render_entity = render_world.get_or_spawn_synced(entity);
        let extracted = image_resources.and_then(|images| {
            if let Some(icon) = main_world.get_component::<LauncherIcon>(entity) {
                if icon.visible && home_active && base_content_visible {
                    images.icon_for_hint(icon.icon_hint).map(|image| {
                        let size = (layout.width.min(layout.height) * 0.54).max(18.0);
                        ExtractedShellImage {
                            position: transform.xy(),
                            width: size,
                            height: size,
                            color: Color::new(255, 255, 255, 232),
                            image: image.clone(),
                            sort_key: image_sort_key(transform, 520),
                        }
                    })
                } else {
                    None
                }
            } else if let Some(tile) = main_world.get_component::<QuickControlTile>(entity) {
                if notification_panel_visible {
                    images.icon_for_hint(tile.icon()).map(|image| {
                        let size = (layout.width.min(layout.height) * 0.36).clamp(15.0, 30.0);
                        let color = if tile.active {
                            Color::new(255, 255, 255, (235.0 * notification_alpha) as u8)
                        } else {
                            Color::new(
                                palette.text_muted.r,
                                palette.text_muted.g,
                                palette.text_muted.b,
                                (210.0 * notification_alpha) as u8,
                            )
                        };
                        ExtractedShellImage {
                            position: fhre::Vec2::new(
                                transform.position.x,
                                transform.position.y - layout.height * 0.08,
                            ),
                            width: size,
                            height: size,
                            color,
                            image: image.clone(),
                            sort_key: image_sort_key(transform, 520),
                        }
                    })
                } else {
                    None
                }
            } else if let Some(row) = main_world.get_component::<SettingsRow>(entity) {
                if row.visible && settings_active && base_content_visible {
                    images.icon_for_hint(settings_action_icon_hint(row.action)).map(|image| {
                        let active = main_world
                            .resources()
                            .get::<ShellContent>()
                            .map(|content| row.action.is_active(content))
                            .unwrap_or(false);
                        let color = if active { palette.background } else { palette.accent };
                        let size = (layout.height * 0.42).clamp(14.0, 22.0);
                        ExtractedShellImage {
                            position: fhre::Vec2::new(
                                transform.position.x - layout.width * 0.43,
                                transform.position.y,
                            ),
                            width: size,
                            height: size,
                            color: Color::new(color.r, color.g, color.b, 235),
                            image: image.clone(),
                            sort_key: image_sort_key(transform, 520),
                        }
                    })
                } else {
                    None
                }
            } else if let Some(button) = main_world.get_component::<SystemNavButton>(entity) {
                if button.visible
                    && active_surface == Some(button.surface_id)
                    && base_content_visible
                {
                    images.icon_for_hint("home").map(|image| {
                        let size = (layout.height * 0.48).clamp(14.0, 22.0);
                        ExtractedShellImage {
                            position: fhre::Vec2::new(
                                transform.position.x - layout.width * 0.28,
                                transform.position.y,
                            ),
                            width: size,
                            height: size,
                            color: palette.background,
                            image: image.clone(),
                            sort_key: image_sort_key(transform, 520),
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            }
        });

        if let Some(image) = extracted {
            render_world.insert_component(render_entity, image);
        } else {
            render_world.remove_component::<ExtractedShellImage>(render_entity);
        }
    }

    // === Extract Home launcher icon labels ===

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(icon_label) = main_world.get_component::<LauncherIconLabel>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);

        if home_active && base_content_visible {
            let label = main_world
                .resources()
                .get::<ShellContent>()
                .and_then(|content| content.get_app(icon_label.app_id))
                .map(|app| app.label)
                .unwrap_or("App");
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: label.into(),
                    color: palette.text,
                    size: 10.0,
                },
            );
        } else {
            render_world.remove_component::<ExtractedShellText>(render_entity);
        }
    }

    // === Extract built-in Settings app text ===

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(row_label) = main_world.get_component::<SettingsRowLabel>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);

        if settings_active && base_content_visible {
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: row_label.action.label().into(),
                    color: palette.text,
                    size: 11.0,
                },
            );
        } else {
            render_world.remove_component::<ExtractedShellText>(render_entity);
        }
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(row_value) = main_world.get_component::<SettingsRowValue>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);

        if settings_active && base_content_visible {
            let theme_name = main_world
                .resources()
                .get::<ThemeState>()
                .map(|theme| theme.current_theme_name())
                .unwrap_or("--");
            let value = main_world
                .resources()
                .get::<ShellContent>()
                .map(|content| settings_value_text(row_value.action, content, theme_name))
                .unwrap_or_else(|| String::from("--"));
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: value,
                    color: palette.text_muted,
                    size: 10.0,
                },
            );
        } else {
            render_world.remove_component::<ExtractedShellText>(render_entity);
        }
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(button_label) = main_world.get_component::<SystemNavButtonLabel>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);

        if active_surface == Some(button_label.surface_id) && base_content_visible {
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: button_label.action.label().into(),
                    color: palette.background,
                    size: 10.0,
                },
            );
        } else {
            render_world.remove_component::<ExtractedShellText>(render_entity);
        }
    }

    // === Extract shell text (SurfaceText) ===
    
    for (entity, transform) in main_world.query::<Transform>() {
        let Some(text) = main_world.get_component::<SurfaceText>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);
        let visible = base_content_visible
            && text.surface_id.map_or(true, |surface_id| active_surface == Some(surface_id));
        if visible {
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: text.text.into(),
                    color: palette.text,
                    size: 12.0,
                },
            );
        } else {
            render_world.remove_component::<ExtractedShellText>(render_entity);
        }
    }

    // The football preview queues its own surface labels; card labels are used by card mode.
    
    for (entity, transform) in main_world.query::<Transform>() {
        let Some(card_title) = main_world.get_component::<SurfaceCardTitle>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);
        if card_preview_visible {
            let text_alpha = (255.0 * app_switcher_alpha.clamp(0.0, 1.0)) as u8;
            let title = main_world
                .resources()
                .get::<ShellContent>()
                .and_then(|content| content.get_surface(card_title.surface_id))
                .map(|surface| surface.preview_title)
                .unwrap_or("Preview");
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: title.into(),
                    color: Color::new(palette.text.r, palette.text.g, palette.text.b, text_alpha),
                    size: 11.0,
                },
            );
        } else {
            render_world.remove_component::<ExtractedShellText>(render_entity);
        }
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(card_subtitle) = main_world.get_component::<SurfaceCardSubtitle>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);
        if card_preview_visible {
            let text_alpha = (220.0 * app_switcher_alpha.clamp(0.0, 1.0)) as u8;
            let subtitle = main_world
                .resources()
                .get::<ShellContent>()
                .and_then(|content| content.get_surface(card_subtitle.surface_id))
                .map(|surface| state_label(surface.state))
                .unwrap_or("--");
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: subtitle.into(),
                    color: Color::new(
                        palette.text_muted.r,
                        palette.text_muted.g,
                        palette.text_muted.b,
                        text_alpha,
                    ),
                    size: 10.0,
                },
            );
        } else {
            render_world.remove_component::<ExtractedShellText>(render_entity);
        }
    }

    // === Extract notification card title/content text (only when NotificationPanel is open) ===
    
    for (entity, transform) in main_world.query::<Transform>() {
        let Some(card_title) = main_world.get_component::<NotificationCardTitle>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);
        
        if notification_panel_open {
            let title = main_world
                .resources()
                .get::<ShellContent>()
                .and_then(|c| c.get_notification(card_title.notification_id))
                .map(|n| n.title)
                .unwrap_or("Notification");
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: title.into(),
                    color: palette.text,
                    size: 12.0,
                },
            );
        } else {
            render_world.remove_component::<ExtractedShellText>(render_entity);
        }
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(card_content) = main_world.get_component::<NotificationCardContent>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);
        
        if notification_panel_open {
            let content = main_world
                .resources()
                .get::<ShellContent>()
                .and_then(|c| c.get_notification(card_content.notification_id))
                .map(|n| n.summary)
                .unwrap_or("");
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: content.into(),
                    color: palette.text_muted,
                    size: 10.0,
                },
            );
        } else {
            render_world.remove_component::<ExtractedShellText>(render_entity);
        }
    }
}

fn settings_value_text(
    action: SettingsAction,
    content: &ShellContent,
    theme_name: &'static str,
) -> String {
    match action {
        SettingsAction::ToggleWifi => on_off(content.quick_controls.wifi_enabled).into(),
        SettingsAction::ToggleBluetooth => on_off(content.quick_controls.bluetooth_enabled).into(),
        SettingsAction::ToggleAirplaneMode => on_off(content.quick_controls.airplane_mode).into(),
        SettingsAction::ToggleFlashlight => on_off(content.quick_controls.flashlight_on).into(),
        SettingsAction::ToggleDnd => on_off(content.quick_controls.dnd_mode).into(),
        SettingsAction::ToggleAutoRotate => on_off(content.quick_controls.auto_rotate).into(),
        SettingsAction::CycleBrightness => {
            let value = (content.quick_controls.brightness.clamp(0.0, 1.0) * 100.0) as u32;
            format!("{}%", value)
        }
        SettingsAction::CycleTheme => theme_name.into(),
        SettingsAction::CyclePreviewEffect => content.preview_effect.label().into(),
    }
}

fn settings_action_icon_hint(action: SettingsAction) -> &'static str {
    match action {
        SettingsAction::ToggleWifi => "wifi",
        SettingsAction::ToggleBluetooth => "bluetooth",
        SettingsAction::ToggleAirplaneMode => "airplane",
        SettingsAction::ToggleFlashlight => "flashlight",
        SettingsAction::ToggleDnd => "dnd",
        SettingsAction::ToggleAutoRotate => "rotate",
        SettingsAction::CycleBrightness => "brightness",
        SettingsAction::CycleTheme => "theme",
        SettingsAction::CyclePreviewEffect => "preview",
    }
}

fn image_sort_key(transform: &Transform, offset: i32) -> i32 {
    (transform.position.z * 100_000.0) as i32 + offset
}

fn on_off(value: bool) -> &'static str {
    if value { "On" } else { "Off" }
}

fn state_label(state: SurfaceState) -> &'static str {
    match state {
        SurfaceState::Running => "Running",
        SurfaceState::Paused => "Paused",
        SurfaceState::Background => "Background",
        SurfaceState::Closed => "Closed",
    }
}

fn preview_card_color(
    stack_index: u8,
    state: SurfaceState,
    alpha: u8,
    palette: crate::ThemePalette,
) -> Color {
    let base = match stack_index % 4 {
        0 => palette.surface,
        1 => Color::lerp(palette.surface_alt, palette.accent, 0.10),
        2 => Color::lerp(palette.surface, palette.accent, 0.16),
        _ => palette.surface_alt,
    };
    let shade = match state {
        SurfaceState::Running => 1.0,
        SurfaceState::Paused => 0.9,
        SurfaceState::Background => 0.82,
        SurfaceState::Closed => 0.62,
    };

    let shaded = Color::lerp(palette.background, base, shade);
    Color::new(
        shaded.r,
        shaded.g,
        shaded.b,
        alpha,
    )
}
