use alloc::string::String;

use fhre::{render_world::ExtractedUI, Color, MainWorld, RenderWorld, Transform};

use crate::components::{AppSurface, BottomBar, CardStackRoot, GestureZone, HomeSurface, NotificationCard, NotificationLayer, NotificationStackRoot, NotificationText, NotificationTextRole, OverlayLayer, QuickSettingsPanel, ShellRoot, StatusBar, SurfacePreviewCard, SurfaceStackRoot, SurfaceText, WidgetLayoutNode};
use crate::resources::{ShellState, ThemeState};

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
    let notifications_visible = main_world
        .resources()
        .get::<ShellState>()
        .map(|state| state.notifications_visible)
        .unwrap_or(false);

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };

        let color = if main_world.get_component::<ShellRoot>(entity).is_some() {
            Some(palette.background)
        } else if main_world.get_component::<SurfaceStackRoot>(entity).is_some() {
            Some(Color::new(0, 0, 0, 0))
        } else if main_world.get_component::<CardStackRoot>(entity).is_some() {
            Some(Color::new(0, 0, 0, 0))
        } else if main_world.get_component::<NotificationStackRoot>(entity).is_some() {
            Some(Color::new(0, 0, 0, 0))
        } else if let Some(home_surface) = main_world.get_component::<HomeSurface>(entity) {
            Some(if home_surface.active { palette.surface } else { Color::new(0, 0, 0, 0) })
        } else if main_world.get_component::<StatusBar>(entity).is_some() {
            Some(palette.accent)
        } else if main_world.get_component::<BottomBar>(entity).is_some() {
            Some(palette.surface_alt)
        } else if let Some(overlay) = main_world.get_component::<OverlayLayer>(entity) {
            Some(if overlay.visible { Color::new(10, 16, 32, 160) } else { Color::new(0, 0, 0, 0) })
        } else if let Some(notification) = main_world.get_component::<NotificationLayer>(entity) {
            Some(if notification.visible { palette.surface_alt } else { Color::new(0, 0, 0, 0) })
        } else if let Some(notification_card) = main_world.get_component::<NotificationCard>(entity) {
            Some(if notification_card.visible {
                if notification_card.stack_index == 0 { palette.surface } else { palette.surface_alt }
            } else {
                Color::new(0, 0, 0, 0)
            })
        } else if let Some(card) = main_world.get_component::<SurfacePreviewCard>(entity) {
            Some(if card.visible {
                if card.stack_index == 0 { palette.surface } else { palette.surface_alt }
            } else {
                Color::new(0, 0, 0, 0)
            })
        } else if main_world.get_component::<GestureZone>(entity).is_some() {
            Some(Color::new(255, 255, 255, 96))
        } else if let Some(panel) = main_world.get_component::<QuickSettingsPanel>(entity) {
            Some(if panel.open { palette.overlay } else { Color::new(0, 0, 0, 0) })
        } else if let Some(app_surface) = main_world.get_component::<AppSurface>(entity) {
            Some(if app_surface.active { palette.surface_alt } else { Color::new(0, 0, 0, 0) })
        } else {
            None
        };

        if let Some(color) = color {
            let render_entity = render_world.get_or_spawn_synced(entity);
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
    }

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

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(text) = main_world.get_component::<NotificationText>(entity) else {
            continue;
        };
        if !notifications_visible {
            continue;
        }
        let render_entity = render_world.get_or_spawn_synced(entity);
        render_world.insert_component(
            render_entity,
            ExtractedShellText {
                position: transform.xy(),
                text: text.text.into(),
                color: match text.role {
                    NotificationTextRole::Title => palette.text,
                    NotificationTextRole::Summary => palette.text_muted,
                },
                size: match text.role {
                    NotificationTextRole::Title => 12.0,
                    NotificationTextRole::Summary => 10.0,
                },
            },
        );
    }
}
