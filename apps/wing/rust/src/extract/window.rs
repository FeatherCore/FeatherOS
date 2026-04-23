use alloc::string::String;

use fhre::{render_world::ExtractedUI, MainWorld, RenderWorld, Transform};

use crate::components::{ButtonWidget, WidgetLayoutNode, WindowContentRoot, WindowContentText, WindowControlButton, WindowControlKind, WindowFocus, WindowFrame, WindowTitleBar, WindowTitleIcon, WindowTitleText};
use crate::resources::ThemeState;
use crate::types::WindowState;

#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedWindowText {
    pub position: fhre::Vec2,
    pub text: String,
    pub color: fhre::Color,
    pub size: f32,
}

impl fhre::Component for ExtractedWindowText {
    fn type_name() -> &'static str {
        "ExtractedWindowText"
    }
}

fn control_color(kind: WindowControlKind, button: Option<&ButtonWidget>, palette: crate::ThemePalette) -> fhre::Color {
    let base = match kind {
        WindowControlKind::Minimize => palette.warning,
        WindowControlKind::Maximize => palette.success,
        WindowControlKind::Close => palette.danger,
    };

    match button {
        Some(button) if button.pressed => base.with_alpha(255),
        Some(button) if button.hovered => base.with_alpha(220),
        _ => base.with_alpha(180),
    }
}

pub fn extract_wing_windows(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let palette = main_world
        .resources()
        .get::<ThemeState>()
        .map(|theme| theme.current.shell)
        .unwrap_or(crate::shell_palette());

    for (entity, transform) in main_world.query::<Transform>() {
        if main_world.get_component::<WindowFrame>(entity).is_none() {
            continue;
        }

        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };

        let Some(window) = main_world.get_component::<WindowFrame>(entity) else {
            continue;
        };
        if matches!(window.state, WindowState::Minimized | WindowState::Closed) {
            continue;
        }

        let focused = main_world
            .get_component::<WindowFocus>(entity)
            .map(|focus| focus.focused)
            .unwrap_or(false);

        let render_entity = render_world.get_or_spawn_synced(entity);
        render_world.insert_component(
            render_entity,
            ExtractedUI {
                position: transform.xy(),
                width: layout.width,
                height: layout.height,
                color: if focused { palette.surface } else { palette.surface_alt },
            },
        );
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(title_bar) = main_world.get_component::<WindowTitleBar>(entity) else {
            continue;
        };
        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };
        let visible = main_world
            .query::<Transform>()
            .any(|(window_entity, _)| {
                main_world
                    .get_component::<WindowFrame>(window_entity)
                    .map(|window| window.id == title_bar.window_id && !matches!(window.state, WindowState::Minimized | WindowState::Closed))
                    .unwrap_or(false)
            });
        if !visible {
            continue;
        }

        let render_entity = render_world.get_or_spawn_synced(entity);
        render_world.insert_component(
            render_entity,
            ExtractedUI {
                position: transform.xy(),
                width: layout.width,
                height: layout.height,
                color: palette.accent,
            },
        );
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(title_text) = main_world.get_component::<WindowTitleText>(entity) else {
            continue;
        };
        let visible = main_world
            .query::<Transform>()
            .any(|(window_entity, _)| {
                main_world
                    .get_component::<WindowFrame>(window_entity)
                    .map(|window| window.id == title_text.window_id && !matches!(window.state, WindowState::Minimized | WindowState::Closed))
                    .unwrap_or(false)
            });
        if !visible {
            continue;
        }

        let render_entity = render_world.get_or_spawn_synced(entity);
        render_world.insert_component(
            render_entity,
            ExtractedWindowText {
                position: transform.xy(),
                text: title_text.text.into(),
                color: fhre::Color::WHITE,
                size: 14.0,
            },
        );
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(title_icon) = main_world.get_component::<WindowTitleIcon>(entity) else {
            continue;
        };
        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };
        let visible = main_world
            .query::<Transform>()
            .any(|(window_entity, _)| {
                main_world
                    .get_component::<WindowFrame>(window_entity)
                    .map(|window| window.id == title_icon.window_id && !matches!(window.state, WindowState::Minimized | WindowState::Closed))
                    .unwrap_or(false)
            });
        if !visible {
            continue;
        }

        let render_entity = render_world.get_or_spawn_synced(entity);
        render_world.insert_component(
            render_entity,
            ExtractedUI {
                position: transform.xy(),
                width: layout.width,
                height: layout.height,
                color: palette.overlay,
            },
        );
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(content_root) = main_world.get_component::<WindowContentRoot>(entity) else {
            continue;
        };
        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };
        let visible = main_world
            .query::<Transform>()
            .any(|(window_entity, _)| {
                main_world
                    .get_component::<WindowFrame>(window_entity)
                    .map(|window| window.id == content_root.window_id && !matches!(window.state, WindowState::Minimized | WindowState::Closed))
                    .unwrap_or(false)
            });
        if !visible {
            continue;
        }

        let render_entity = render_world.get_or_spawn_synced(entity);
        render_world.insert_component(
            render_entity,
            ExtractedUI {
                position: transform.xy(),
                width: layout.width,
                height: layout.height,
                color: palette.background,
            },
        );
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(content_text) = main_world.get_component::<WindowContentText>(entity) else {
            continue;
        };
        let visible = main_world
            .query::<Transform>()
            .any(|(window_entity, _)| {
                main_world
                    .get_component::<WindowFrame>(window_entity)
                    .map(|window| window.id == content_text.window_id && !matches!(window.state, WindowState::Minimized | WindowState::Closed))
                    .unwrap_or(false)
            });
        if !visible {
            continue;
        }

        let render_entity = render_world.get_or_spawn_synced(entity);
        render_world.insert_component(
            render_entity,
            ExtractedWindowText {
                position: transform.xy(),
                text: content_text.text.into(),
                color: palette.text_muted,
                size: 12.0,
            },
        );
    }

    for (entity, transform) in main_world.query::<Transform>() {
        let Some(control) = main_world.get_component::<WindowControlButton>(entity) else {
            continue;
        };
        let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) else {
            continue;
        };
        let visible = main_world
            .query::<Transform>()
            .any(|(window_entity, _)| {
                main_world
                    .get_component::<WindowFrame>(window_entity)
                    .map(|window| window.id == control.window_id && !matches!(window.state, WindowState::Minimized | WindowState::Closed))
                    .unwrap_or(false)
            });
        if !visible {
            continue;
        }

        let button = main_world.get_component::<ButtonWidget>(entity);
        let color = control_color(control.kind, button, palette);

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
