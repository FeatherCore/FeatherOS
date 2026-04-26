use alloc::string::String;

use fhre::{render_world::ExtractedUI, Color, MainWorld, RenderWorld, Transform};

use crate::components::{
    AppSurface, BrightnessControl, CardStackRoot, HomeSurface,
    NotificationCard, NotificationCardContent, NotificationCardTitle, NotificationPanel,
    OverlayLayer, QuickControlTile, ShellRoot,
    SurfaceCardSubtitle, SurfaceCardTitle, SurfacePreviewCard,
    SurfaceStackRoot, SurfaceText, WidgetLayoutNode,
};
use crate::resources::{ShellContent, ShellOverlayAnimation, ShellState, ThemeState};

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
    let card_alpha = main_world
        .resources()
        .get::<ShellOverlayAnimation>()
        .map(|anim| anim.card_alpha_eased())
        .unwrap_or(1.0);
    let app_switcher_open = main_world
        .resources()
        .get::<ShellState>()
        .map(|state| state.app_switcher_open())
        .unwrap_or(false);
    let notification_panel_open = main_world
        .resources()
        .get::<ShellState>()
        .map(|state| state.notification_panel_open())
        .unwrap_or(false);

    // === Extract UI rectangles ===
    
    for (entity, transform) in main_world.query::<Transform>() {
        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };

        // Determine if this UI element should be rendered
        let (should_render, render_color) = if main_world.get_component::<ShellRoot>(entity).is_some() {
            (true, Some(palette.background))
        } else if main_world.get_component::<SurfaceStackRoot>(entity).is_some() {
            (false, None)  // Invisible container
        } else if main_world.get_component::<CardStackRoot>(entity).is_some() {
            (false, None)  // Invisible container
        } else if let Some(home_surface) = main_world.get_component::<HomeSurface>(entity) {
            if home_surface.active { (true, Some(palette.surface)) } else { (false, None) }
        } else if let Some(overlay) = main_world.get_component::<OverlayLayer>(entity) {
            if overlay.visible { 
                (true, Some(Color::new(10, 16, 32, (160.0 * overlay_alpha) as u8)))
            } else { (false, None) }
        } else if let Some(card) = main_world.get_component::<SurfacePreviewCard>(entity) {
            if card.visible {
                let base_color = if card.stack_index == 0 { palette.surface } else { palette.surface_alt };
                (true, Some(Color::new(base_color.r, base_color.g, base_color.b, (255.0 * card_alpha) as u8)))
            } else { (false, None) }
        // Notification panel container
        } else if let Some(panel) = main_world.get_component::<NotificationPanel>(entity) {
            if panel.open { 
                let base = palette.overlay;
                (true, Some(Color::new(base.r, base.g, base.b, (base.a as f32 * card_alpha) as u8)))
            } else { (false, None) }
        // Quick control tile
        } else if let Some(tile) = main_world.get_component::<QuickControlTile>(entity) {
            if notification_panel_open || card_alpha > 0.0 {
                let tile_color = if tile.active { palette.accent } else { palette.surface_alt };
                (true, Some(Color::new(tile_color.r, tile_color.g, tile_color.b, (255.0 * card_alpha) as u8)))
            } else { (false, None) }
        // Brightness control
        } else if let Some(brightness) = main_world.get_component::<BrightnessControl>(entity) {
            if brightness.visible {
                (true, Some(Color::new(palette.surface_alt.r, palette.surface_alt.g, palette.surface_alt.b, (200.0 * card_alpha) as u8)))
            } else { (false, None) }
        // Notification card
        } else if let Some(card) = main_world.get_component::<NotificationCard>(entity) {
            if card.visible {
                let base_color = match card.priority {
                    crate::components::NotificationPriority::High => palette.accent,
                    _ => palette.surface,
                };
                (true, Some(Color::new(base_color.r, base_color.g, base_color.b, (255.0 * card_alpha) as u8)))
            } else { (false, None) }
        } else if let Some(app_surface) = main_world.get_component::<AppSurface>(entity) {
            if app_surface.active { (true, Some(palette.surface_alt)) } else { (false, None) }
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

    // === Extract shell text (SurfaceText) ===
    
    for (entity, transform) in main_world.query::<Transform>() {
        let Some(text) = main_world.get_component::<SurfaceText>(entity) else {
            continue;
        };
        if let Some(surface_id) = text.surface_id {
            if active_surface != Some(surface_id) {
                continue;
            }
        }
        let render_entity = render_world.get_or_spawn_synced(entity);
        render_world.insert_component(
            render_entity,
            ExtractedShellText {
                position: transform.xy(),
                text: text.text.into(),
                color: palette.text,
                size: 12.0,
            },
        );
    }

    // === Extract card title/subtitle text (only when AppSwitcher is open) ===
    
    for (entity, transform) in main_world.query::<Transform>() {
        let Some(card_title) = main_world.get_component::<SurfaceCardTitle>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);
        
        if app_switcher_open {
            let title = main_world
                .resources()
                .get::<ShellContent>()
                .and_then(|c| c.get_surface(card_title.surface_id))
                .map(|s| s.preview_title)
                .unwrap_or("App");
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
        let Some(card_subtitle) = main_world.get_component::<SurfaceCardSubtitle>(entity) else {
            continue;
        };
        let render_entity = render_world.get_or_spawn_synced(entity);
        
        if app_switcher_open {
            let subtitle = main_world
                .resources()
                .get::<ShellContent>()
                .and_then(|c| c.get_surface(card_subtitle.surface_id))
                .and_then(|s| if s.icon_hint.is_empty() { None } else { Some(s.icon_hint) })
                .unwrap_or("Running");
            render_world.insert_component(
                render_entity,
                ExtractedShellText {
                    position: transform.xy(),
                    text: subtitle.into(),
                    color: palette.text_muted,
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
