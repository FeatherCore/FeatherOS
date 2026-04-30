use crate::{
    demo::WingDemoState,
    key::UiKey,
    theme::{LumiaMetrics, LumiaTheme},
    tree::apply_ui_frame,
};
use fhre::{Camera, Color, Point, Rect, Surface};

pub(crate) fn draw_status(surface: &mut Surface, state: &mut WingDemoState) {
    let metrics = LumiaMetrics::for_screen(surface.width(), surface.height());
    let theme = LumiaTheme::DARK;
    let w = surface.width() as i32;
    let top = (metrics.status_h as i32 / 2).saturating_sub((metrics.label_scale as i32 * 4).max(4));
    let text = theme.text;
    let icon = Color::rgba(255, 255, 255, 220);
    let ui = &mut state.status_builder;
    ui.clear();

    ui.text(UiKey(700), metrics.pad_x as i32 + 10, top, 16, "10:08", text, metrics.label_scale);
    ui.text(
        UiKey(701),
        w - metrics.pad_x as i32 - 46,
        top,
        16,
        "72",
        Color::rgba(245, 248, 255, 202),
        metrics.label_scale,
    );
    ui.line(
        UiKey(702),
        Point::new(w - metrics.pad_x as i32 - 92, top + 11),
        Point::new(w - metrics.pad_x as i32 - 72, top + 11),
        16,
        metrics.small_scale.max(1),
        icon,
    );
    ui.line(
        UiKey(703),
        Point::new(w - metrics.pad_x as i32 - 82, top + 3),
        Point::new(w - metrics.pad_x as i32 - 82, top + 19),
        16,
        metrics.small_scale.max(1),
        icon,
    );
    ui.circle(
        UiKey(704),
        Rect::new(w - metrics.pad_x as i32 - 64, top, 22, 22),
        16,
        icon,
    );
    ui.circle(
        UiKey(705),
        Rect::new(w - metrics.pad_x as i32 - 60, top + 4, 14, 14),
        17,
        Color::rgba(5, 12, 28, 224),
    );

    let camera = Camera::screen_canvas(surface.width(), surface.height());
    let _stats = apply_ui_frame(&mut state.status, ui, &camera);
    state.status.render(surface, &camera);
}
