//! Extractors for the Wing desktop demo.

use fhre::{MainWorld, RenderCommand, RenderWorld};
use fhre::render_world::{PhaseItem, RenderPhaseType};

use crate::WingRuntime;

pub fn queue_wing_shell(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let runtime = match main_world.resources().get::<WingRuntime>() {
        Some(runtime) => runtime,
        None => return,
    };

    for (index, command) in runtime.wing.generate_render_commands().into_iter().enumerate() {
        match command {
            RenderCommand::Clear { .. } => {
                render_world.add_phase_item(
                    RenderPhaseType::Background,
                    PhaseItem::new(command).with_sort_key(index as i32),
                );
            }
            _ => {
                render_world.add_phase_item(
                    RenderPhaseType::Ui,
                    PhaseItem::ui(command, index as i32),
                );
            }
        }
    }
}
