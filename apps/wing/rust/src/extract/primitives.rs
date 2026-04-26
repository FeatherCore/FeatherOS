use fhre::{
    math::Rect,
    render_world::{ExtractedUI, PhaseItem, RenderPhaseType},
    MainWorld, RenderCommand, RenderWorld,
};

use super::ExtractedShellText;

/// Sort key ranges for proper rendering order within Ui phase.
/// Lower values render first (background), higher values render later (foreground).
mod sort_keys {
    /// Background elements (rendered first, covered by text)
    pub const BACKGROUND: i32 = 0;
    /// Text elements (rendered last, on top of backgrounds)
    pub const TEXT: i32 = 10000;
}

pub fn queue_wing_primitives(_main_world: &MainWorld, render_world: &mut RenderWorld) {
    // Queue UI rectangles first (backgrounds)
    let uis: alloc::vec::Vec<(fhre::Entity, ExtractedUI)> = render_world
        .query::<ExtractedUI>()
        .map(|(entity, ui)| (entity, ui.clone()))
        .collect();

    for (entity, ui) in uis {
        let rect = Rect::from_center_size(ui.position, fhre::Vec2::new(ui.width, ui.height));
        let command = RenderCommand::DrawRect { rect, color: ui.color };
        // Use low sort key so backgrounds render before text
        let sort_key = sort_keys::BACKGROUND + (entity.id() % 1000) as i32;
        render_world.add_phase_item(RenderPhaseType::Ui, PhaseItem::ui(command, sort_key));
    }

    // Queue text last (foreground, on top of backgrounds)
    let shell_texts: alloc::vec::Vec<(fhre::Entity, ExtractedShellText)> = render_world
        .query::<ExtractedShellText>()
        .map(|(entity, text)| (entity, text.clone()))
        .collect();

    for (entity, text) in shell_texts {
        let command = RenderCommand::draw_text(text.position, text.text, text.color, text.size);
        // Use high sort key so text renders after backgrounds
        let sort_key = sort_keys::TEXT + (entity.id() % 1000) as i32;
        render_world.add_phase_item(RenderPhaseType::Ui, PhaseItem::ui(command, sort_key));
    }
}
