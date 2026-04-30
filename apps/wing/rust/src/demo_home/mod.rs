use crate::{
    action::{ACTION_CORTANA, ACTION_HOME},
    asset::{WP_BACK, WP_LOGO, WP_SEARCH},
    demo::WingDemoState,
    demo_overlay::render_overlay,
    key::UiKey,
    spec::{UiKind, UiSpec},
    theme::{scale_metric, LumiaMetrics, LumiaTheme},
};
use fhre::{Color, ImageFit, Rect, Surface};

pub(crate) fn draw_home_handle(surface: &mut Surface, state: &mut WingDemoState) {
    let w = surface.width();
    let h = surface.height();
    let metrics = LumiaMetrics::for_screen(w, h);
    let theme = LumiaTheme::DARK;
    let nav_y = h.saturating_sub(metrics.nav_h) as i32;
    let center_y = nav_y + metrics.nav_h as i32 / 2;
    let third = (w / 3).max(1) as i32;
    let icon = Color::rgba(245, 248, 255, 218);
    let icon_edge = scale_metric(w.min(h), 22).max(18);
    {
        let ui = &mut state.overlay_builder;
        ui.clear();
        ui.bar(UiKey(400), Rect::new(0, nav_y, w, metrics.nav_h), 20, theme.nav, 0);
        ui.push(UiSpec::rect(
            UiKey(410),
            UiKind::Tile,
            Rect::new(0, nav_y, (w / 3).max(1), metrics.nav_h),
            20,
            Color::TRANSPARENT,
            0,
        ).with_action(ACTION_HOME));
        ui.push(UiSpec::rect(
            UiKey(411),
            UiKind::Tile,
            Rect::new(third, nav_y, (w / 3).max(1), metrics.nav_h),
            20,
            Color::TRANSPARENT,
            0,
        ).with_action(ACTION_HOME));
        ui.push(UiSpec::rect(
            UiKey(412),
            UiKind::Tile,
            Rect::new(third * 2, nav_y, w.saturating_sub((w / 3).saturating_mul(2)), metrics.nav_h),
            20,
            Color::TRANSPARENT,
            0,
        ).with_action(ACTION_CORTANA));
        ui.image_tint(
            UiKey(401),
            Rect::new(third / 2 - icon_edge as i32 / 2, center_y - icon_edge as i32 / 2, icon_edge, icon_edge),
            21,
            WP_BACK.image(),
            230,
            ImageFit::Contain,
            icon,
        );
        ui.image_tint(
            UiKey(402),
            Rect::new(w as i32 / 2 - icon_edge as i32 / 2, center_y - icon_edge as i32 / 2, icon_edge, icon_edge),
            21,
            WP_LOGO.image(),
            230,
            ImageFit::Contain,
            icon,
        );
        ui.image_tint(
            UiKey(403),
            Rect::new(third * 2 + third / 2 - icon_edge as i32 / 2, center_y - icon_edge as i32 / 2, icon_edge, icon_edge),
            21,
            WP_SEARCH.image(),
            230,
            ImageFit::Contain,
            icon,
        );
    }
    render_overlay(surface, state);
}
