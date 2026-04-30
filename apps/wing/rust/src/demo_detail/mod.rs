use crate::{
    action::{action_label, ActionId},
    demo::WingDemoState,
    demo_overlay::render_overlay,
    key::UiKey,
};
use fhre::{Color, Rect, Surface};

pub(crate) fn draw_tile_detail(surface: &mut Surface, action: ActionId, state: &mut WingDemoState) {
    let w = surface.width() as i32;
    let h = surface.height() as i32;
    let card = Rect::new(48, h / 2 - 72, (w - 96).max(1) as u16, 124);
    let ui = &mut state.overlay_builder;
    ui.clear();

    ui.panel(UiKey(410), card, 18, Color::rgba(10, 18, 34, 224), 24);
    ui.tile(
        UiKey(411),
        Rect::new(card.x + 18, card.y + 18, 58, 58),
        19,
        Color::rgba(72, 132, 248, 235),
        14,
    );
    ui.text(UiKey(412), card.x + 94, card.y + 22, 20, action_label(action), Color::WHITE, 2);
    ui.text(
        UiKey(413),
        card.x + 94,
        card.y + 52,
        20,
        "SWIPE LEFT/RIGHT HOME",
        Color::rgba(211, 224, 245, 198),
        1,
    );
    ui.text(
        UiKey(414),
        card.x + 94,
        card.y + 72,
        20,
        "UP SWITCH  DOWN NOTIFY",
        Color::rgba(211, 224, 245, 184),
        1,
    );
    render_overlay(surface, state);
}
