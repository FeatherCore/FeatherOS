use fhre::render_world::{PhaseItem, RenderPhaseType};
use fhre::{MainWorld, RenderCommand, RenderWorld};

use crate::{DesktopMetrics, ThemeState, WingRuntime};

fn queue_shell_command(render_world: &mut RenderWorld, command: RenderCommand, sort_key: i32) {
    match command {
        RenderCommand::Clear { .. } => {
            render_world.add_phase_item(
                RenderPhaseType::Background,
                PhaseItem::new(command).with_sort_key(sort_key),
            );
        }
        _ => {
            render_world.add_phase_item(
                RenderPhaseType::Ui,
                PhaseItem::ui(command, sort_key),
            );
        }
    }
}

pub fn queue_wing_shell(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let theme = main_world
        .resources()
        .get::<ThemeState>()
        .map(|theme| theme.current)
        .unwrap_or_default();
    let metrics = main_world.resources().get::<DesktopMetrics>();

    let Some(runtime) = main_world.resources().get::<WingRuntime>() else {
        queue_shell_command(render_world, RenderCommand::clear(theme.shell.background), 0);

        if let Some(metrics) = metrics {
            if metrics.screen_size.x > 0.0 && metrics.screen_size.y > 0.0 {
                queue_shell_command(
                    render_world,
                    RenderCommand::draw_rect(
                        fhre::math::Rect::new(0.0, 0.0, metrics.screen_size.x, metrics.screen_size.y),
                        theme.shell.background,
                    ),
                    1,
                );
            }
        }
        return;
    };

    for (index, command) in runtime.wing.generate_render_commands().into_iter().enumerate() {
        let command = match command {
            RenderCommand::Clear { .. } => RenderCommand::clear(theme.shell.background),
            other => other,
        };
        queue_shell_command(render_world, command, index as i32);
    }
}
