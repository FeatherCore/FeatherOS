use fhre::{render_world::ExtractedUI, MainWorld, RenderWorld, Transform};

use crate::components::{DesktopWallpaper, TaskbarRoot, WidgetLayoutNode};

use super::clear_extracted_ui;

pub fn extract_wing_desktop(main_world: &MainWorld, render_world: &mut RenderWorld) {
    clear_extracted_ui(render_world);

    for (entity, transform) in main_world.query::<Transform>() {
        if main_world.get_component::<DesktopWallpaper>(entity).is_some() {
            if let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) {
                let render_entity = render_world.get_or_spawn_synced(entity);
                render_world.insert_component(
                    render_entity,
                    ExtractedUI {
                        position: transform.xy(),
                        width: layout.width,
                        height: layout.height,
                        color: crate::shell_palette().background,
                    },
                );
            }
            continue;
        }

        if main_world.get_component::<TaskbarRoot>(entity).is_some() {
            if let Some(layout) = main_world.get_component::<WidgetLayoutNode>(entity) {
                let render_entity = render_world.get_or_spawn_synced(entity);
                render_world.insert_component(
                    render_entity,
                    ExtractedUI {
                        position: transform.xy(),
                        width: layout.width,
                        height: layout.height,
                        color: crate::shell_palette().surface_alt,
                    },
                );
            }
        }
    }
}
