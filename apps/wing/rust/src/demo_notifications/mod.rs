use crate::{
    builder::UiBuilder,
    asset::{
        AIRPLANE_ICON, BATTERY_ICON, BLUETOOTH_ICON, BRIGHTNESS_ICON, CELLULAR_ICON, HOTSPOT_ICON,
        LOCATION_ICON, SETTINGS_ICON, VPN_ICON, WIFI_ICON, WingAssetId,
    },
    demo::WingDemoState,
    demo_overlay::render_overlay,
    key::UiKey,
    layout::{ContentInset, GridLayout, StackLayout},
    spec::{UiKind, UiSpec},
    theme::{scale_metric, LumiaMetrics, LumiaTheme},
};
use fhre::{Color, Point, Rect, Size, Surface};

pub(crate) fn draw_notifications(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    let w = surface.width();
    let h = surface.height();
    let metrics = LumiaMetrics::for_screen(w, h);
    let theme = LumiaTheme::DARK;
    let short = w.min(h);
    let panel = Rect::new(0, 0, w, h);
    let panel_key = UiKey(500);
    let inset = ContentInset::new(
        metrics.pad_x,
        metrics.status_h.saturating_add(scale_metric(short, 10).max(8)),
        metrics.pad_x,
        scale_metric(short, 18).max(12),
    );
    let content = inset.content_rect(panel);
    let quick_h = scale_metric(short, 48).max(40);
    let quick_w = content.w.saturating_sub(metrics.gap.saturating_mul(4)) / 5;
    let quick_y = scale_metric(short, 28).max(22) as i32;
    let quick_grid = GridLayout::new(
        Point::new(content.x, content.y + quick_y),
        Size::new(quick_w, quick_h),
        Size::new(metrics.gap, metrics.gap),
        5,
    );
    let used_y = quick_y.max(0) as u16
        + quick_h.saturating_mul(2)
        + metrics.gap.saturating_mul(3)
        + scale_metric(short, 12).max(8);
    let cards_y = content.y + used_y as i32;
    let available_cards_h = content
        .h
        .saturating_sub(used_y)
        .saturating_sub(metrics.gap.saturating_mul(2));
    let card_h = (available_cards_h / 3).max(scale_metric(short, 42).max(36));
    let card_stack = StackLayout::column(
        Point::new(content.x, cards_y),
        Size::new(content.w, card_h),
        metrics.gap,
    );
    let content_clip = content;
    let content_clip_local = content;
    let accent = theme.accent.mix(theme.accent_alt, (frame as u8) & 63);
    let ui = &mut state.overlay_builder;
    ui.clear();

    ui.panel_with_content(panel_key, panel, 18, Color::rgba(0, 0, 0, 218), 0, inset);
    ui.push_clipped(
        UiSpec::text(
            UiKey(501),
            content.x,
            content.y,
            20,
            "NOTIFICATIONS",
            theme.text,
            metrics.label_scale,
        ),
        content_clip,
    );
    ui.push_clipped(
        UiSpec::text(
            UiKey(502),
            content.x + content.w.saturating_sub(scale_metric(short, 70).max(54)) as i32,
            content.y,
            20,
            "MAY 15",
            theme.muted,
            metrics.small_scale,
        ),
        content_clip,
    );
    append_quick_action(ui, 510, panel_key, quick_grid, 0, "WIFI", WIFI_ICON, true, content_clip_local, metrics);
    append_quick_action(ui, 520, panel_key, quick_grid, 1, "BT", BLUETOOTH_ICON, true, content_clip_local, metrics);
    append_quick_action(ui, 530, panel_key, quick_grid, 2, "AIR", AIRPLANE_ICON, false, content_clip_local, metrics);
    append_quick_action(ui, 580, panel_key, quick_grid, 3, "CELL", CELLULAR_ICON, true, content_clip_local, metrics);
    append_quick_action(ui, 590, panel_key, quick_grid, 4, "BAT", BATTERY_ICON, true, content_clip_local, metrics);
    append_quick_action(ui, 600, panel_key, quick_grid, 5, "LITE", BRIGHTNESS_ICON, true, content_clip_local, metrics);
    append_quick_action(ui, 610, panel_key, quick_grid, 6, "HOT", HOTSPOT_ICON, false, content_clip_local, metrics);
    append_quick_action(ui, 620, panel_key, quick_grid, 7, "VPN", VPN_ICON, false, content_clip_local, metrics);
    append_quick_action(ui, 630, panel_key, quick_grid, 8, "LOC", LOCATION_ICON, true, content_clip_local, metrics);
    append_quick_action(ui, 640, panel_key, quick_grid, 9, "SET", SETTINGS_ICON, true, content_clip_local, metrics);

    append_notification_card(
        ui,
        540,
        panel_key,
        card_stack,
        0,
        panel,
        "MESSAGE",
        "AT 14:00",
        "NOW",
        accent,
        content_clip,
        content_clip_local,
    );
    append_notification_card(
        ui,
        550,
        panel_key,
        card_stack,
        1,
        panel,
        "MAIL",
        "BUILD PASSED",
        "2M",
        Color::rgba(248, 249, 253, 232),
        content_clip,
        content_clip_local,
    );
    append_notification_card(
        ui,
        560,
        panel_key,
        card_stack,
        2,
        panel,
        "SYSTEM",
        "SURFACE READY",
        "12M",
        Color::rgba(248, 249, 253, 226),
        content_clip,
        content_clip_local,
    );

    ui.bar(
        UiKey(570),
        Rect::new(0, h.saturating_sub(scale_metric(short, 10).max(8)) as i32, w, scale_metric(short, 10).max(8)),
        22,
        accent,
        0,
    );
    render_overlay(surface, state);
}

fn append_quick_action<const N: usize>(
    ui: &mut UiBuilder<N>,
    base_key: u32,
    parent: UiKey,
    grid: GridLayout,
    index: u8,
    label: &'static str,
    icon: WingAssetId,
    active: bool,
    clip_local: Rect,
    metrics: LumiaMetrics,
) {
    let local = grid.cell_rect(index);
    let tile_key = UiKey(base_key);
    let fill = if active {
        LumiaTheme::DARK.accent
    } else {
        Color::rgba(45, 45, 45, 224)
    };
    let text = if active {
        Color::WHITE
    } else {
        Color::rgba(226, 232, 238, 212)
    };
    ui.push_clipped(
        UiSpec::rect(UiKey(base_key), UiKind::Tile, Rect::new(0, 0, local.w, local.h), 1, fill, 0)
            .with_local_parent(parent)
            .with_grid_cell(grid, index),
        clip_local,
    );
    ui.push_clipped(
        UiSpec::text(
            UiKey(base_key + 1),
            metrics.gap as i32 + 2,
            local.h.saturating_sub(scale_metric(local.h, 13).max(11)) as i32,
            1,
            label,
            text,
            metrics.small_scale,
        )
        .with_local_parent(tile_key),
        Rect::new(0, 0, local.w, local.h),
    );
    let icon_edge = local.h.saturating_mul(42).saturating_div(100).max(14);
    ui.push_clipped(
        UiSpec::image(
            UiKey(base_key + 2),
            Rect::new(metrics.gap as i32 + 2, metrics.gap as i32 + 2, icon_edge, icon_edge),
            2,
            icon.image(),
            if active { 255 } else { 190 },
        )
        .with_local_parent(tile_key),
        Rect::new(0, 0, local.w, local.h),
    );
}

fn append_notification_card<const N: usize>(
    ui: &mut UiBuilder<N>,
    base_key: u32,
    parent: UiKey,
    stack: StackLayout,
    index: u8,
    panel: Rect,
    title: &'static str,
    body: &'static str,
    time: &'static str,
    fill: Color,
    clip_screen: Rect,
    clip_local: Rect,
) {
    let local = stack.item_rect(index);
    let x = panel.x + local.x;
    let y = panel.y + local.y;
    let w = local.w;
    let dark_text = Color::rgba(42, 54, 82, 225);
    let active = fill.b > 190 && fill.r < 120;
    let title_color = if active { Color::WHITE } else { dark_text };
    let body_color = if active {
        Color::rgba(238, 246, 255, 218)
    } else {
        Color::rgba(80, 94, 124, 215)
    };
    ui.push_clipped(
        UiSpec::rect(UiKey(base_key), UiKind::Tile, Rect::new(0, 0, w, local.h), 1, fill, 16)
            .with_local_parent(parent)
            .with_stack_item(stack, index),
        clip_local,
    );
    ui.push_clipped(
        UiSpec::rect(
            UiKey(base_key + 1),
            UiKind::Circle,
            Rect::new(x + 12, y + 9, 26, 26),
            20,
            Color::rgba(74, 132, 248, 235),
            13,
        ),
        clip_screen,
    );
    ui.push_clipped(
        UiSpec::text(UiKey(base_key + 2), x + 54, y + 10, 21, title, title_color, 1),
        clip_screen,
    );
    ui.push_clipped(
        UiSpec::text(UiKey(base_key + 3), x + 54, y + 26, 21, body, body_color, 1),
        clip_screen,
    );
    ui.push_clipped(
        UiSpec::text(UiKey(base_key + 4), x + w as i32 - 38, y + 10, 21, time, body_color, 1),
        clip_screen,
    );
}
