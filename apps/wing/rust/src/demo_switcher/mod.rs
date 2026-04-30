use crate::{
    action::ActionId,
    app::{builtin_app_registry, APP_FHRE_SAMPLE},
    builder::UiBuilder,
    demo::WingDemoState,
    demo_overlay::render_overlay,
    key::UiKey,
};
use fhre::{Color, Rect, Surface};

pub(crate) fn draw_app_switcher(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    let w = surface.width() as i32;
    let h = surface.height() as i32;
    let panel = Rect::new(24, 92, (w - 48).max(1) as u16, (h - 150).max(1) as u16);
    let ui = &mut state.overlay_builder;
    ui.clear();
    ui.panel(UiKey(600), panel, 18, Color::rgba(10, 18, 34, 226), 26);
    ui.text(UiKey(601), panel.x + 24, panel.y + 24, 20, "APP SWITCHER", Color::WHITE, 3);
    ui.text(
        UiKey(602),
        panel.x + 24,
        panel.y + 58,
        20,
        "SWIPE LEFT/RIGHT HOME",
        Color::rgba(211, 224, 245, 190),
        1,
    );

    let card_w = panel.w.saturating_sub(70) / 2;
    append_switch_card(
        ui,
        610,
        Rect::new(panel.x + 24, panel.y + 92, card_w, panel.h.saturating_sub(128)),
        "WING",
        Color::rgb(72, 132, 248).mix(Color::rgb(82, 214, 232), (frame as u8) & 63),
        None,
    );
    let registry = builtin_app_registry();
    let running = state.shell.running_app.unwrap_or(APP_FHRE_SAMPLE);
    let running_entry = registry.by_id(running);
    let sample_running = state.shell.running_app.is_some();
    let running_label = running_entry.map(|entry| entry.title).unwrap_or("FHRE SAMPLE");
    let running_action = running_entry.map(|entry| entry.action).unwrap_or(crate::ACTION_DRAW);
    append_switch_card(
        ui,
        620,
        Rect::new(panel.x + 46 + card_w as i32, panel.y + 92, card_w, panel.h.saturating_sub(128)),
        if sample_running { running_label } else { "FHRE SAMPLE" },
        if sample_running {
            Color::rgb(82, 214, 232)
        } else {
            Color::rgba(248, 249, 253, 232)
        },
        Some(running_action),
    );
    append_sample_snapshot(ui, 640, Rect::new(panel.x + 46 + card_w as i32, panel.y + 92, card_w, panel.h.saturating_sub(128)), frame);
    ui.bar(
        UiKey(630),
        Rect::new(w / 2 - 28, h - 22, 56, 5),
        22,
        Color::rgba(245, 248, 255, 220),
        3,
    );
    render_overlay(surface, state);
}

fn append_switch_card<const N: usize>(
    ui: &mut UiBuilder<N>,
    base_key: u32,
    rect: Rect,
    label: &'static str,
    accent: Color,
    action: Option<ActionId>,
) {
    match action {
        Some(action) => {
            ui.tile_action(UiKey(base_key), rect, 19, Color::rgba(244, 247, 253, 222), 18, action);
        }
        None => {
            ui.tile(UiKey(base_key), rect, 19, Color::rgba(244, 247, 253, 222), 18);
        }
    }
    ui.panel(
        UiKey(base_key + 1),
        Rect::new(rect.x + 16, rect.y + 16, rect.w.saturating_sub(32), rect.h.saturating_sub(52)),
        20,
        Color::rgba(18, 32, 54, 230),
        12,
    );
    ui.circle(UiKey(base_key + 2), Rect::new(rect.x + 22, rect.y + 24, 36, 36), 21, accent);
    ui.text(UiKey(base_key + 3), rect.x + 18, rect.bottom() - 28, 21, label, Color::rgba(42, 54, 82, 225), 2);
}

fn append_sample_snapshot<const N: usize>(ui: &mut UiBuilder<N>, base_key: u32, rect: Rect, frame: u32) {
    let preview = Rect::new(
        rect.x + 26,
        rect.y + 28,
        rect.w.saturating_sub(52),
        rect.h.saturating_sub(84),
    );
    if preview.w < 18 || preview.h < 18 {
        return;
    }

    let horizon = preview.y + (preview.h as i32 * 58 / 100);
    let vanishing = fhre::Point::new(
        preview.x + preview.w as i32 / 2 + (((frame as i32 & 31) - 15) / 2),
        horizon - 10,
    );
    ui.bar(
        UiKey(base_key),
        Rect::new(preview.x, horizon, preview.w, preview.bottom().saturating_sub(horizon) as u16),
        22,
        Color::rgba(0, 70, 92, 178),
        0,
    );
    for i in 0..5 {
        let x = preview.x + (i as i32 * preview.w as i32) / 4;
        ui.line(
            UiKey(base_key + 1 + i as u32),
            fhre::Point::new(x, preview.bottom()),
            vanishing,
            23,
            1,
            Color::rgba(82, 214, 232, 114),
        );
    }
    for i in 0..3 {
        let y = horizon + i * 14;
        ui.line(
            UiKey(base_key + 8 + i as u32),
            fhre::Point::new(preview.x, y),
            fhre::Point::new(preview.right(), y),
            23,
            1,
            Color::rgba(82, 214, 232, 92),
        );
    }
    let box_size = (preview.w.min(preview.h) / 4).max(8) as i32;
    let cx = preview.x + preview.w as i32 / 2;
    let cy = preview.y + preview.h as i32 / 2;
    let skew = ((frame as i32 & 31) - 15) / 3;
    let a = fhre::Point::new(cx - box_size, cy - box_size / 2);
    let b = fhre::Point::new(cx + box_size, cy - box_size / 3);
    let c = fhre::Point::new(cx + box_size, cy + box_size);
    let d = fhre::Point::new(cx - box_size, cy + box_size / 2);
    let e = fhre::Point::new(a.x + box_size / 2 + skew, a.y - box_size / 2);
    let f = fhre::Point::new(b.x + box_size / 2 + skew, b.y - box_size / 2);
    let g = fhre::Point::new(c.x + box_size / 2 + skew, c.y - box_size / 2);
    let h = fhre::Point::new(d.x + box_size / 2 + skew, d.y - box_size / 2);
    let edge = Color::rgba(238, 250, 255, 210);
    ui.line(UiKey(base_key + 12), a, b, 24, 1, edge);
    ui.line(UiKey(base_key + 13), b, c, 24, 1, edge);
    ui.line(UiKey(base_key + 14), c, d, 24, 1, edge);
    ui.line(UiKey(base_key + 15), d, a, 24, 1, edge);
    ui.line(UiKey(base_key + 16), e, f, 24, 1, Color::rgba(238, 250, 255, 132));
    ui.line(UiKey(base_key + 17), f, g, 24, 1, Color::rgba(238, 250, 255, 132));
    ui.line(UiKey(base_key + 18), g, h, 24, 1, Color::rgba(238, 250, 255, 132));
    ui.line(UiKey(base_key + 19), h, e, 24, 1, Color::rgba(238, 250, 255, 132));
    ui.line(UiKey(base_key + 20), a, e, 24, 1, Color::rgba(238, 250, 255, 166));
    ui.line(UiKey(base_key + 21), b, f, 24, 1, Color::rgba(238, 250, 255, 166));
    ui.line(UiKey(base_key + 22), c, g, 24, 1, Color::rgba(238, 250, 255, 166));
    ui.line(UiKey(base_key + 23), d, h, 24, 1, Color::rgba(238, 250, 255, 166));
}
