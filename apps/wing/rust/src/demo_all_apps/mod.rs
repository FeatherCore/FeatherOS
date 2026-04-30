use crate::{
    app::{builtin_app_registry, AppEntry, AppSurfaceState},
    asset::WP_NEXT,
    builder::UiBuilder,
    demo::WingDemoState,
    demo_overlay::render_overlay,
    key::UiKey,
    layout::{ContentInset, StackLayout},
    spec::{UiKind, UiSpec},
    theme::{scale_metric, LumiaMetrics, LumiaTheme},
};
use fhre::{Color, ImageFit, Point, Rect, Size, Surface};

pub(crate) fn draw_all_apps(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    let w = surface.width();
    let h = surface.height();
    let short = w.min(h);
    let metrics = LumiaMetrics::for_screen(w, h);
    let theme = LumiaTheme::DARK;
    let panel = Rect::new(0, 0, w, h);
    let inset = ContentInset::new(
        metrics.pad_x.saturating_add(scale_metric(short, 6).max(4)),
        metrics.status_h.saturating_add(scale_metric(short, 12).max(10)),
        metrics.pad_x.saturating_add(scale_metric(short, 6).max(4)),
        metrics.nav_h.saturating_add(scale_metric(short, 8).max(6)),
    );
    let content = inset.content_rect(panel);
    let row_h = scale_metric(short, 38).max(34);
    let row_gap = scale_metric(short, 5).max(4);
    let rows_y = scale_metric(short, 54).max(46) as i32;
    let list_bottom_pad = scale_metric(short, 22).max(18);
    let list_h = content
        .h
        .saturating_sub(rows_y.max(0) as u16)
        .saturating_sub(list_bottom_pad);
    let accent = theme.accent.mix(theme.accent_alt, ((frame & 63) as u8).saturating_mul(3));
    let registry = builtin_app_registry();
    let total_rows_h = registry
        .len()
        .saturating_mul(row_h.saturating_add(row_gap) as usize)
        .saturating_sub(row_gap as usize);
    let max_offset = (total_rows_h as i32).saturating_sub(list_h as i32).max(0);
    state.shell.all_apps_scroll.set_max_offset(max_offset);
    let scroll = state.shell.all_apps_scroll.offset;
    let stack = StackLayout::column(Point::new(0, rows_y.saturating_sub(scroll)), Size::new(content.w, row_h), row_gap);
    let list_clip = Rect::new(0, rows_y, content.w, list_h);
    let panel_key = UiKey(1000);
    let ui = &mut state.overlay_builder;
    ui.clear();

    ui.panel_with_content(panel_key, panel, 18, Color::rgba(2, 12, 18, 240), 0, inset);
    ui.content_text(panel_key, UiKey(1001), 0, 0, 20, "ALL APPS", theme.text, metrics.label_scale);
    ui.content_text(
        panel_key,
        UiKey(1002),
        0,
        (metrics.label_scale as i32).saturating_mul(12),
        20,
        "FIXED REGISTRY",
        theme.muted,
        metrics.small_scale,
    );
    ui.content_bar(
        panel_key,
        UiKey(1003),
        Rect::new(0, scale_metric(short, 42).max(36) as i32, content.w, 2),
        20,
        Color::rgba(255, 255, 255, 44),
        0,
    );

    let mut index = 0;
    while index < registry.len() {
        if let Some(entry) = registry.get(index) {
            append_app_row(
                ui,
                1020 + index as u32 * 8,
                panel_key,
                stack,
                index as u8,
                entry,
                state.shell.app_surface_state(entry.id),
                if index == 0 { accent } else { Color::rgba(72, 136, 210, 220) },
                list_clip,
                metrics,
            );
        }
        index += 1;
    }

    ui.content_text(
        panel_key,
        UiKey(1090),
        0,
        content.h.saturating_sub(8) as i32,
        20,
        if max_offset > 0 { "DRAG: SCROLL   LEFT/RIGHT: HOME" } else { "LEFT/RIGHT: HOME" },
        Color::rgba(195, 218, 232, 176),
        metrics.small_scale,
    );
    render_overlay(surface, state);
}

fn append_app_row<const N: usize>(
    ui: &mut UiBuilder<N>,
    base_key: u32,
    parent: UiKey,
    stack: StackLayout,
    index: u8,
    entry: AppEntry,
    surface_state: AppSurfaceState,
    accent: Color,
    list_clip: Rect,
    metrics: LumiaMetrics,
) {
    let local = stack.item_rect(index);
    let row_key = UiKey(base_key);
    let icon_edge = local.h.saturating_sub(metrics.gap.saturating_mul(2)).max(18);
    let text_x = icon_edge as i32 + metrics.gap as i32 * 2;
    ui.push(
        UiSpec::rect(
            row_key,
            UiKind::Tile,
            Rect::new(0, 0, local.w, local.h),
            20,
            Color::rgba(18, 42, 52, 222),
            0,
        )
        .with_content_parent(parent)
        .with_stack_item(stack, index)
        .with_clip(list_clip)
        .with_action(entry.action),
    );
    ui.child_image(
        row_key,
        UiKey(base_key + 1),
        Rect::new(metrics.gap as i32, metrics.gap as i32, icon_edge, icon_edge),
        21,
        entry.icon_asset.image(),
        255,
    );
    ui.child_text(
        row_key,
        UiKey(base_key + 2),
        text_x,
        metrics.gap as i32,
        21,
        entry.title,
        Color::rgba(248, 250, 255, 238),
        metrics.label_scale,
    );
    ui.child_text(
        row_key,
        UiKey(base_key + 3),
        text_x,
        metrics.gap as i32 + (metrics.label_scale as i32).saturating_mul(11),
        21,
        app_subtitle(entry, surface_state),
        Color::rgba(185, 211, 222, 190),
        metrics.small_scale,
    );
    if surface_state != AppSurfaceState::Stopped {
        ui.child_text(
            row_key,
            UiKey(base_key + 5),
            local.w.saturating_sub(scale_metric(local.h, 88).max(76)) as i32,
            metrics.gap as i32,
            21,
            app_state_label(surface_state),
            accent,
            metrics.small_scale,
        );
    }
    ui.child_image_fit(
        row_key,
        UiKey(base_key + 4),
        Rect::new(
            local.w.saturating_sub(scale_metric(local.h, 18).max(14)) as i32,
            (local.h.saturating_sub(scale_metric(local.h, 18).max(14)) / 2) as i32,
            scale_metric(local.h, 18).max(14),
            scale_metric(local.h, 18).max(14),
        ),
        21,
        WP_NEXT.image(),
        190,
        ImageFit::Contain,
    );
}

fn app_subtitle(entry: AppEntry, state: AppSurfaceState) -> &'static str {
    match state {
        AppSurfaceState::Stopped => entry.subtitle,
        AppSurfaceState::Running => "RUNNING",
        AppSurfaceState::Focused => "FOCUSED",
        AppSurfaceState::Preview => "PREVIEW",
    }
}

fn app_state_label(state: AppSurfaceState) -> &'static str {
    match state {
        AppSurfaceState::Stopped => "",
        AppSurfaceState::Running => "RUN",
        AppSurfaceState::Focused => "FOCUS",
        AppSurfaceState::Preview => "CARD",
    }
}
