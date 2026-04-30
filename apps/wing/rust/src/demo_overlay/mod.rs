use crate::{demo::WingDemoState, tree::apply_ui_frame};
use fhre::{Camera, Surface};

pub(crate) fn render_overlay(surface: &mut Surface, state: &mut WingDemoState) {
    let camera = Camera::screen_canvas(surface.width(), surface.height());
    let _stats = apply_ui_frame(&mut state.overlay, &state.overlay_builder, &camera);
    state.overlay.render(surface, &camera);
}
