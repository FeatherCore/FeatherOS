use crate::{
    action::{
        ActionId, ACTION_ALL_APPS, ACTION_APP_MESSAGING, ACTION_APP_NEWS, ACTION_APP_PEOPLE,
        ACTION_APP_PHONE, ACTION_APP_PHOTOS, ACTION_DRAW, ACTION_SETTINGS,
    },
    animation::Animation,
    asset::{
        MESSAGE_ICON, NEWS_ICON, NEWS_TILE, OUTLOOK_ICON, PEOPLE_ICON, PHONE_ICON, PHOTOS_ICON,
        SETTINGS_ICON, WingAssetId,
    },
    builder::UiBuilder,
    demo::WingDemoState,
    key::UiKey,
    spec::{UiKind, UiSpec},
    theme::{scale_metric, LumiaMetrics, LumiaTheme},
    tree::apply_ui_frame,
};
use fhre::{Camera, Color, Easing, Rect, Surface, Tween};

pub(crate) fn draw_launcher(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    let w = surface.width();
    let h = surface.height();
    let metrics = LumiaMetrics::for_screen(w, h);
    let theme = LumiaTheme::DARK;
    let camera = Camera::screen_canvas(surface.width(), surface.height());
    let panel = metrics.content_rect(w, h);
    let panel_key = UiKey(1);
    let panel_inset = metrics.start_inset();
    let panel_content = panel_inset.content_rect(panel);

    let ui = &mut state.launcher_builder;
    ui.clear();

    ui.panel_with_content(panel_key, panel, 0, theme.bg, 0, panel_inset);
    ui.content_text(panel_key, UiKey(3), 0, 0, 4, "WING", theme.text, metrics.label_scale);
    ui.content_text(
        panel_key,
        UiKey(4),
        0,
        (10u16.saturating_mul(metrics.label_scale)) as i32,
        4,
        "START",
        theme.muted,
        metrics.small_scale,
    );

    let gap = metrics.gap;
    let tile = metrics.tile;
    let wide = tile.saturating_mul(2).saturating_add(gap);
    let x0 = 0;
    let x1 = tile as i32 + gap as i32;
    let x2 = wide as i32 + gap as i32;
    let row0 = scale_metric(w.min(h), 34).max(28) as i32;
    let row1 = row0 + tile as i32 + gap as i32;
    let row2 = row1 + tile as i32 + gap as i32;
    let row3 = row2 + tile as i32 + gap as i32;
    let accent = theme.accent.mix(theme.accent_alt, ((frame & 63) as u8).saturating_mul(3));
    let soft_tile = Color::rgba(16, 72, 104, 206);

    append_start_tile(
        ui,
        100,
        panel_key,
        Rect::new(x0, row0, wide, tile),
        "Phone",
        "",
        ACTION_APP_PHONE,
        PHONE_ICON,
        None,
        accent,
        frame,
        metrics,
    );
    append_start_tile(
        ui,
        120,
        panel_key,
        Rect::new(x2, row0, tile, tile),
        "People",
        "",
        ACTION_APP_PEOPLE,
        PEOPLE_ICON,
        None,
        theme.tile,
        frame.wrapping_add(11),
        metrics,
    );
    append_start_tile(
        ui,
        140,
        panel_key,
        Rect::new(x0, row1, tile, tile),
        "Mail",
        "",
        ACTION_DRAW,
        OUTLOOK_ICON,
        None,
        soft_tile,
        frame.wrapping_add(23),
        metrics,
    );
    append_start_tile(
        ui,
        160,
        panel_key,
        Rect::new(x1, row1, wide, tile),
        "Messaging",
        "",
        ACTION_APP_MESSAGING,
        MESSAGE_ICON,
        None,
        Color::rgba(0, 116, 178, 214),
        frame.wrapping_add(37),
        metrics,
    );
    append_start_tile(
        ui,
        180,
        panel_key,
        Rect::new(x0, row2, wide, tile),
        "News",
        "",
        ACTION_APP_NEWS,
        NEWS_ICON,
        Some(NEWS_TILE),
        Color::rgba(0, 150, 136, 214),
        frame.wrapping_add(51),
        metrics,
    );
    append_start_tile(
        ui,
        200,
        panel_key,
        Rect::new(x2, row2, tile, tile),
        "Photos",
        "",
        ACTION_APP_PHOTOS,
        PHOTOS_ICON,
        None,
        Color::rgba(88, 86, 214, 214),
        frame.wrapping_add(65),
        metrics,
    );

    let settings_h = scale_metric(w.min(h), 34).max(30);
    let settings_icon = settings_h.saturating_sub(metrics.gap.saturating_mul(2)).max(18);
    let settings_y = row3 + scale_metric(w.min(h), 4).max(3) as i32;
    let settings_key = UiKey(220);
    ui.push(
        UiSpec::rect(
            settings_key,
            UiKind::Tile,
            Rect::new(0, settings_y, panel_content.w, settings_h),
            3,
            Color::rgba(18, 32, 54, 214),
            0,
        )
        .with_content_parent(panel_key)
        .with_action(ACTION_SETTINGS),
    );
    ui.push(
        UiSpec::image(
            UiKey(224),
            Rect::new(metrics.gap as i32 + 2, metrics.gap as i32, settings_icon, settings_icon),
            5,
            SETTINGS_ICON.image(),
            255,
        )
        .with_local_parent(settings_key),
    );
    ui.push(
        UiSpec::icon(
            UiKey(225),
            Rect::new(metrics.gap as i32 + 2, metrics.gap as i32, settings_icon, settings_icon),
            4,
            crate::ICON_SETTINGS,
            Color::rgba(255, 255, 255, 224),
        )
        .with_local_parent(settings_key),
    );
    ui.child_text(
        settings_key,
        UiKey(222),
        settings_icon as i32 + metrics.gap as i32 * 2,
        (settings_h.saturating_sub(metrics.label_scale.saturating_mul(8)) / 2) as i32,
        4,
        "SETTINGS",
        theme.text,
        metrics.label_scale,
    );
    ui.child_text(
        settings_key,
        UiKey(223),
        panel_content.w.saturating_sub(22) as i32,
        (settings_h.saturating_sub(metrics.small_scale.saturating_mul(8)) / 2) as i32,
        4,
        ">",
        theme.muted,
        metrics.small_scale,
    );

    let apps_y = settings_y + settings_h as i32 + metrics.gap as i32;
    ui.push(
        UiSpec::rect(
            UiKey(230),
            UiKind::Tile,
            Rect::new(0, apps_y - 6, panel_content.w, scale_metric(w.min(h), 28).max(24)),
            3,
            Color::rgba(0, 0, 0, 92),
            0,
        )
        .with_content_parent(panel_key)
        .with_action(ACTION_ALL_APPS),
    );
    ui.content_text(panel_key, UiKey(231), 0, apps_y, 4, "ALL APPS", theme.text, metrics.small_scale);
    ui.content_text(
        panel_key,
        UiKey(232),
        panel_content.w.saturating_sub(22) as i32,
        apps_y,
        4,
        ">",
        theme.text,
        metrics.small_scale,
    );

    ui.content_text(
        panel_key,
        UiKey(300),
        0,
        panel_content.h.saturating_sub(8) as i32,
        4,
        state.shell.status_label(),
        theme.muted,
        metrics.small_scale,
    );

    let _stats = apply_ui_frame(&mut state.launcher, ui, &camera);
    state.launcher.render(surface, &camera);
}

fn append_start_tile<const N: usize>(
    ui: &mut UiBuilder<N>,
    base_key: u32,
    parent: UiKey,
    rect: Rect,
    label: &'static str,
    sublabel: &'static str,
    action: ActionId,
    icon_asset: WingAssetId,
    live_asset: Option<WingAssetId>,
    fill: Color,
    frame: u32,
    metrics: LumiaMetrics,
) {
    let tile_key = UiKey(base_key);
    let icon_edge = rect
        .w
        .min(rect.h)
        .saturating_mul(36)
        .saturating_div(100)
        .max(scale_metric(rect.w.min(rect.h), 18).max(16));
    let label_y = rect.h.saturating_sub(scale_metric(rect.w.min(rect.h), 22).max(18)) as i32;
    let live_alpha = 184u8.saturating_add((frame as u8) & 31);

    ui.push(
        UiSpec::rect(tile_key, UiKind::Tile, rect, 3, fill, 0)
            .with_content_parent(parent)
            .with_action(action),
    );
    let tween = Tween::from_i32(202, 255, 820_000)
        .with_delay(base_key.saturating_sub(100).saturating_mul(2_000))
        .with_repeat(true)
        .with_ping_pong(true)
        .with_easing(Easing::EaseInOut);
    if let Some(live) = live_asset {
        ui.push(
            UiSpec::image(
                UiKey(base_key + 4),
                Rect::new(0, 0, rect.w, rect.h),
                2,
                live.image(),
                192,
            )
            .with_local_parent(tile_key)
            .with_animation(Animation::opacity(tween)),
        );
    }
    ui.push(
        UiSpec::image(
            UiKey(base_key + 1),
            Rect::new(metrics.gap as i32 + 2, metrics.gap as i32 + 2, icon_edge, icon_edge),
            3,
            icon_asset.image(),
            245,
        )
        .with_local_parent(tile_key)
        .with_animation(Animation::opacity(tween)),
    );
    ui.child_text(
        tile_key,
        UiKey(base_key + 2),
        metrics.gap as i32 + 2,
        label_y,
        2,
        label,
        Color::rgba(255, 255, 255, live_alpha),
        metrics.label_scale,
    );
    if !sublabel.is_empty() && rect.w > rect.h {
        ui.child_text(
            tile_key,
            UiKey(base_key + 3),
            metrics.gap as i32 + 2,
            label_y.saturating_add((metrics.label_scale as i32).saturating_mul(10)),
            2,
            sublabel,
            Color::rgba(230, 244, 255, 180),
            metrics.small_scale,
        );
    }
}
