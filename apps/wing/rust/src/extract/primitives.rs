use fhre::{
    math::Rect,
    render_world::{ExtractedUI, PhaseItem, RenderPhaseType},
    MainWorld, RenderCommand, RenderWorld,
};

use super::ExtractedShellText;

pub fn queue_wing_primitives(_main_world: &MainWorld, render_world: &mut RenderWorld) {
    let uis: alloc::vec::Vec<(fhre::Entity, ExtractedUI)> = render_world
        .query::<ExtractedUI>()
        .map(|(entity, ui)| (entity, ui.clone()))
        .collect();

    for (entity, ui) in uis {
        let rect = Rect::from_center_size(ui.position, fhre::Vec2::new(ui.width, ui.height));
        let command = RenderCommand::DrawRect { rect, color: ui.color };
        render_world.add_phase_item(RenderPhaseType::Ui, PhaseItem::ui(command, entity.id() as i32));
    }

    let shell_texts: alloc::vec::Vec<(fhre::Entity, ExtractedShellText)> = render_world
        .query::<ExtractedShellText>()
        .map(|(entity, text)| (entity, text.clone()))
        .collect();

    for (entity, text) in shell_texts {
        let command = RenderCommand::draw_text(text.position, text.text, text.color, text.size);
        render_world.add_phase_item(RenderPhaseType::Ui, PhaseItem::ui(command, entity.id() as i32));
    }
}
