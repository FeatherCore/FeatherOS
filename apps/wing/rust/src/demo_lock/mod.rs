use crate::{
    asset::{PADLOCK_ICON, WALLPAPER_2},
    builder::UiBuilder,
    demo::WingDemoState,
    demo_overlay::render_overlay,
    key::UiKey,
    theme::{scale_metric, LumiaMetrics, LumiaTheme},
};
use fhre::{Color, DrawCommand, DrawList, ImageFit, Rect, Surface};

pub(crate) fn draw_lock_screen(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    let w = surface.width();
    let h = surface.height();
    let short = w.min(h);
    let mut list: DrawList<8> = DrawList::new();
    list.push(DrawCommand::FillGradient {
        rect: Rect::new(0, 0, w, h),
        depth: -32,
        top: Color::rgb(0, 0, 0),
        bottom: Color::rgb(8, 22, 40),
    });
    list.push(DrawCommand::DrawImageFit {
        rect: Rect::new(0, 0, w, h),
        depth: -31,
        image: WALLPAPER_2.image(),
        opacity: 255,
        fit: ImageFit::Cover,
    });
    list.execute(surface);

    let metrics = LumiaMetrics::for_screen(w, h);
    let theme = LumiaTheme::DARK;
    let pulse = 180u8.saturating_add(((frame as u8) & 31).saturating_mul(2));
    let ui: &mut UiBuilder<96> = &mut state.overlay_builder;
    ui.clear();
    ui.panel(
        UiKey(1500),
        Rect::new(0, 0, w, h),
        10,
        Color::rgba(0, 0, 0, 72),
        0,
    );
    ui.text(
        UiKey(1501),
        metrics.pad_x as i32 + scale_metric(short, 12).max(10) as i32,
        h as i32 / 2 - scale_metric(short, 76).max(60) as i32,
        12,
        "10:08",
        Color::rgba(255, 255, 255, 244),
        if short >= 430 { 5 } else { 4 },
    );
    ui.text(
        UiKey(1502),
        metrics.pad_x as i32 + scale_metric(short, 16).max(12) as i32,
        h as i32 / 2 - scale_metric(short, 28).max(22) as i32,
        12,
        "FRIDAY, MAY 15",
        theme.text,
        metrics.label_scale,
    );
    ui.image_tint(
        UiKey(1503),
        Rect::new(
            metrics.pad_x as i32 + scale_metric(short, 18).max(14) as i32,
            h.saturating_sub(metrics.nav_h).saturating_sub(scale_metric(short, 58).max(48)) as i32,
            scale_metric(short, 28).max(24),
            scale_metric(short, 28).max(24),
        ),
        12,
        PADLOCK_ICON.image(),
        pulse,
        ImageFit::Contain,
        Color::WHITE,
    );
    ui.text(
        UiKey(1504),
        metrics.pad_x as i32 + scale_metric(short, 58).max(48) as i32,
        h.saturating_sub(metrics.nav_h).saturating_sub(scale_metric(short, 52).max(42)) as i32,
        12,
        "SWIPE UP TO START",
        Color::rgba(255, 255, 255, 210),
        metrics.small_scale,
    );
    ui.bar(
        UiKey(1505),
        Rect::new(
            w as i32 / 2 - scale_metric(short, 28).max(22) as i32,
            h.saturating_sub(scale_metric(short, 12).max(10)) as i32,
            scale_metric(short, 56).max(44),
            scale_metric(short, 4).max(3),
        ),
        12,
        Color::rgba(255, 255, 255, 220),
        scale_metric(short, 2).max(1),
    );

    render_overlay(surface, state);
}
