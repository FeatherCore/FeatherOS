use crate::{
    action::ACTION_HOME,
    asset::{WP_SEARCH, WINDOWS_LOGO},
    builder::UiBuilder,
    demo::WingDemoState,
    demo_overlay::render_overlay,
    key::UiKey,
    spec::{UiKind, UiSpec},
    theme::{scale_metric, LumiaMetrics, LumiaTheme},
};
use fhre::{Color, ImageFit, Rect, Surface};

pub(crate) fn draw_cortana(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    let w = surface.width();
    let h = surface.height();
    let short = w.min(h);
    let metrics = LumiaMetrics::for_screen(w, h);
    let theme = LumiaTheme::DARK;
    let pulse = ((frame & 63) as u8).saturating_mul(3);
    let ui: &mut UiBuilder<96> = &mut state.overlay_builder;
    ui.clear();

    ui.panel(UiKey(1600), Rect::new(0, 0, w, h), 18, Color::rgb(0, 0, 0), 0);
    let logo_edge = scale_metric(short, 64).max(48);
    ui.image_tint(
        UiKey(1601),
        Rect::new(
            w as i32 / 2 - logo_edge as i32 / 2,
            metrics.status_h as i32 + scale_metric(short, 44).max(34) as i32,
            logo_edge,
            logo_edge,
        ),
        20,
        WP_SEARCH.image(),
        255,
        ImageFit::Contain,
        theme.accent.mix(theme.accent_alt, pulse),
    );
    ui.circle(
        UiKey(1602),
        Rect::new(
            w as i32 / 2 - logo_edge as i32,
            metrics.status_h as i32 + scale_metric(short, 28).max(22) as i32,
            logo_edge.saturating_mul(2),
            logo_edge.saturating_mul(2),
        ),
        19,
        Color::rgba(theme.accent.r, theme.accent.g, theme.accent.b, 42),
    );
    ui.text(
        UiKey(1603),
        metrics.pad_x as i32 + scale_metric(short, 18).max(14) as i32,
        h as i32 / 2 - scale_metric(short, 24).max(18) as i32,
        20,
        "CORTANA",
        Color::rgba(255, 255, 255, 238),
        if short >= 430 { 3 } else { 2 },
    );
    ui.text(
        UiKey(1604),
        metrics.pad_x as i32 + scale_metric(short, 18).max(14) as i32,
        h as i32 / 2 + scale_metric(short, 8).max(6) as i32,
        20,
        "WHAT CAN I HELP WITH?",
        Color::rgba(210, 226, 238, 188),
        metrics.label_scale,
    );
    let search_h = scale_metric(short, 42).max(34);
    let search_y = h.saturating_sub(metrics.nav_h).saturating_sub(search_h).saturating_sub(metrics.gap);
    ui.push(
        UiSpec::rect(
            UiKey(1605),
            UiKind::Tile,
            Rect::new(metrics.pad_x as i32, search_y as i32, w.saturating_sub(metrics.pad_x.saturating_mul(2)), search_h),
            20,
            Color::rgba(255, 255, 255, 28),
            0,
        )
        .with_action(ACTION_HOME),
    );
    ui.image_tint(
        UiKey(1606),
        Rect::new(
            metrics.pad_x as i32 + metrics.gap as i32,
            search_y as i32 + (search_h.saturating_sub(scale_metric(short, 22).max(18)) / 2) as i32,
            scale_metric(short, 22).max(18),
            scale_metric(short, 22).max(18),
        ),
        21,
        WINDOWS_LOGO.image(),
        220,
        ImageFit::Contain,
        Color::rgba(255, 255, 255, 210),
    );
    ui.text(
        UiKey(1607),
        metrics.pad_x as i32 + metrics.gap as i32 * 2 + scale_metric(short, 22).max(18) as i32,
        search_y as i32 + (search_h.saturating_sub(metrics.label_scale.saturating_mul(8)) / 2) as i32,
        21,
        "TAP TO RETURN HOME",
        Color::rgba(255, 255, 255, 176),
        metrics.label_scale,
    );

    render_overlay(surface, state);
}
