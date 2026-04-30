use crate::{
    action::{
        ACTION_HOME,
        ACTION_SETTINGS_ABOUT, ACTION_SETTINGS_ACCOUNT, ACTION_SETTINGS_APPS,
        ACTION_SETTINGS_DEVICES, ACTION_SETTINGS_NETWORK, ACTION_SETTINGS_PERSONALIZATION,
        ACTION_SETTINGS_PRIVACY, ACTION_SETTINGS_SYSTEM, ACTION_SETTINGS_TIME,
    },
    asset::{
        SETTINGS_BACK_ICON, WingAssetId, WP_ABOUT, WP_ACCOUNT, WP_APPS, WP_DEVICES, WP_NETWORK,
        WP_BACK, WP_LOGO, WP_NEXT, WP_PERSONALIZATION, WP_PRIVACY, WP_SEARCH, WP_SETTINGS,
        WP_SYSTEM, WP_TIME,
    },
    builder::UiBuilder,
    demo::WingDemoState,
    demo_overlay::render_overlay,
    key::UiKey,
    layout::{ContentInset, StackLayout},
    spec::{UiKind, UiSpec},
    shell::SettingsRoute,
    theme::{scale_metric, LumiaMetrics, LumiaTheme},
};
use fhre::{Color, ImageFit, Point, Rect, Size, Surface};

pub(crate) fn draw_settings(
    surface: &mut Surface,
    frame: u32,
    state: &mut WingDemoState,
    route: SettingsRoute,
) {
    let w = surface.width();
    let h = surface.height();
    let short = w.min(h);
    let metrics = LumiaMetrics::for_screen(w, h);
    let theme = LumiaTheme::DARK;
    let panel = Rect::new(0, 0, w, h);
    let panel_key = UiKey(800);
    let inset = ContentInset::new(
        metrics.pad_x.saturating_add(scale_metric(short, 6).max(4)),
        metrics.status_h.saturating_add(scale_metric(short, 12).max(10)),
        metrics.pad_x.saturating_add(scale_metric(short, 6).max(4)),
        metrics.nav_h.saturating_add(scale_metric(short, 8).max(6)),
    );
    let content = inset.content_rect(panel);
    let header_icon = scale_metric(short, 34).max(28);
    let row_h = scale_metric(short, 34).max(30);
    let row_gap = scale_metric(short, 5).max(4);
    let rows_y = scale_metric(short, 54).max(46) as i32;
    let list_bottom_pad = scale_metric(short, 22).max(18);
    let list_h = content
        .h
        .saturating_sub(rows_y.max(0) as u16)
        .saturating_sub(list_bottom_pad);
    let total_h = settings_content_height(route, row_h, row_gap, content, short);
    let max_offset = (total_h as i32).saturating_sub(list_h as i32).max(0);
    state.shell.settings_scroll.set_max_offset(max_offset);
    let scroll = state.shell.settings_scroll.offset;
    let row_stack = StackLayout::column(Point::new(0, rows_y.saturating_sub(scroll)), Size::new(content.w, row_h), row_gap);
    let list_clip = Rect::new(0, rows_y, content.w, list_h);
    let accent = theme.accent.mix(theme.accent_alt, ((frame & 63) as u8).saturating_mul(3));
    let ui = &mut state.overlay_builder;
    ui.clear();

    ui.panel_with_content(panel_key, panel, 18, Color::rgba(2, 12, 18, 238), 0, inset);
    ui.push(
        UiSpec::image(
            UiKey(801),
            Rect::new(header_icon as i32 + metrics.gap as i32, 0, header_icon, header_icon),
            20,
            WP_SETTINGS.image(),
            255,
        )
        .with_content_parent(panel_key),
    );
    ui.push(
        UiSpec::image_tint(
            UiKey(805),
            Rect::new(0, 0, header_icon, header_icon),
            21,
            SETTINGS_BACK_ICON.image(),
            232,
            fhre::ImageFit::Contain,
            Color::rgba(255, 255, 255, 220),
        )
        .with_content_parent(panel_key)
        .with_action(crate::action::ACTION_HOME),
    );
    ui.content_text(
        panel_key,
        UiKey(802),
        header_icon as i32 * 2 + metrics.gap as i32 * 2,
        0,
        20,
        "SETTINGS",
        theme.text,
        metrics.label_scale,
    );
    ui.content_text(
        panel_key,
        UiKey(803),
        header_icon as i32 * 2 + metrics.gap as i32 * 2,
        (metrics.label_scale as i32).saturating_mul(11),
        20,
        "SYSTEM APP",
        theme.muted,
        metrics.small_scale,
    );
    ui.content_bar(
        panel_key,
        UiKey(804),
        Rect::new(0, scale_metric(short, 44).max(38) as i32, content.w, 2),
        20,
        Color::rgba(255, 255, 255, 48),
        0,
    );

    if route != SettingsRoute::Main {
        append_settings_subpage(ui, panel_key, content, route, scroll, list_clip, accent, metrics, theme);
        append_settings_nav(ui, w, h, metrics, theme);
        render_overlay(surface, state);
        return;
    }

    append_settings_row(
        ui,
        820,
        panel_key,
        row_stack,
        0,
        "SYSTEM",
        "DISPLAY + POWER",
        WP_SYSTEM,
        accent,
        ACTION_SETTINGS_SYSTEM,
        list_clip,
        metrics,
    );
    append_settings_row(
        ui,
        830,
        panel_key,
        row_stack,
        1,
        "PERSONALIZATION",
        "ACCENT + LIVE TILES",
        WP_PERSONALIZATION,
        Color::rgba(0, 150, 136, 218),
        ACTION_SETTINGS_PERSONALIZATION,
        list_clip,
        metrics,
    );
    append_settings_row(
        ui,
        840,
        panel_key,
        row_stack,
        2,
        "NETWORK",
        "WIFI + BLUETOOTH",
        WP_NETWORK,
        Color::rgba(0, 120, 215, 218),
        ACTION_SETTINGS_NETWORK,
        list_clip,
        metrics,
    );
    append_settings_row(
        ui,
        850,
        panel_key,
        row_stack,
        3,
        "ACCOUNT",
        "MICROSOFT PROFILE",
        WP_ACCOUNT,
        Color::rgba(88, 86, 214, 218),
        ACTION_SETTINGS_ACCOUNT,
        list_clip,
        metrics,
    );
    append_settings_row(
        ui,
        860,
        panel_key,
        row_stack,
        4,
        "APPS",
        "BACKGROUND TASKS",
        WP_APPS,
        Color::rgba(60, 170, 220, 218),
        ACTION_SETTINGS_APPS,
        list_clip,
        metrics,
    );
    append_settings_row(
        ui,
        870,
        panel_key,
        row_stack,
        5,
        "DEVICES",
        "DISPLAY + INPUT",
        WP_DEVICES,
        Color::rgba(60, 170, 220, 218),
        ACTION_SETTINGS_DEVICES,
        list_clip,
        metrics,
    );
    append_settings_row(
        ui,
        880,
        panel_key,
        row_stack,
        6,
        "PRIVACY",
        "LOCATION + SENSORS",
        WP_PRIVACY,
        Color::rgba(60, 170, 220, 218),
        ACTION_SETTINGS_PRIVACY,
        list_clip,
        metrics,
    );
    append_settings_row(
        ui,
        890,
        panel_key,
        row_stack,
        7,
        "TIME",
        "CLOCK + REGION",
        WP_TIME,
        Color::rgba(60, 170, 220, 218),
        ACTION_SETTINGS_TIME,
        list_clip,
        metrics,
    );
    append_settings_row(
        ui,
        900,
        panel_key,
        row_stack,
        8,
        "ABOUT",
        "FHRE ECS NO_STD",
        WP_ABOUT,
        Color::rgba(60, 170, 220, 218),
        ACTION_SETTINGS_ABOUT,
        list_clip,
        metrics,
    );

    let slider_y = rows_y
        + (row_h.saturating_add(row_gap).saturating_mul(9)) as i32
        - scroll
        + scale_metric(short, 8).max(6) as i32;
    if content.h > 430 {
        append_brightness_slider(ui, panel_key, content.w, slider_y, accent, metrics);
    }
    append_settings_nav(ui, w, h, metrics, theme);
    render_overlay(surface, state);
}

fn append_settings_row<const N: usize>(
    ui: &mut UiBuilder<N>,
    base_key: u32,
    parent: UiKey,
    stack: StackLayout,
    index: u8,
    title: &'static str,
    subtitle: &'static str,
    icon: WingAssetId,
    accent: Color,
    action: crate::action::ActionId,
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
            Color::rgba(25, 47, 53, 224),
            0,
        )
        .with_content_parent(parent)
        .with_stack_item(stack, index)
        .with_clip(list_clip)
        .with_action(action),
    );
    ui.child_image_tint(
        row_key,
        UiKey(base_key + 1),
        Rect::new(metrics.gap as i32, metrics.gap as i32, icon_edge, icon_edge),
        21,
        icon.image(),
        255,
        accent,
    );
    ui.child_text(
        row_key,
        UiKey(base_key + 2),
        text_x,
        metrics.gap as i32,
        21,
        title,
        Color::rgba(248, 250, 255, 238),
        metrics.label_scale,
    );
    ui.child_text(
        row_key,
        UiKey(base_key + 3),
        text_x,
        metrics.gap as i32 + (metrics.label_scale as i32).saturating_mul(11),
        21,
        subtitle,
        Color::rgba(185, 211, 222, 190),
        metrics.small_scale,
    );
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

fn append_settings_subpage<const N: usize>(
    ui: &mut UiBuilder<N>,
    parent: UiKey,
    content: Rect,
    route: SettingsRoute,
    scroll: i32,
    list_clip: Rect,
    accent: Color,
    metrics: LumiaMetrics,
    theme: LumiaTheme,
) {
    let title = match route {
        SettingsRoute::Main => "SETTINGS",
        SettingsRoute::System => "SYSTEM",
        SettingsRoute::Personalization => "PERSONALIZATION",
        SettingsRoute::Network => "NETWORK",
        SettingsRoute::About => "ABOUT",
        SettingsRoute::Account => "ACCOUNT",
        SettingsRoute::Apps => "APPS",
        SettingsRoute::Devices => "DEVICES",
        SettingsRoute::Privacy => "PRIVACY",
        SettingsRoute::Time => "TIME",
    };
    let subtitle = match route {
        SettingsRoute::Main => "SYSTEM APP",
        SettingsRoute::System => "DISPLAY  POWER  MEMORY",
        SettingsRoute::Personalization => "ACCENT  TILES  MOTION",
        SettingsRoute::Network => "WIFI  BT  AIRPLANE",
        SettingsRoute::About => "FHRE + WING RUST NO_STD",
        SettingsRoute::Account => "MICROSOFT PROFILE  SYNC",
        SettingsRoute::Apps => "BACKGROUND  DEFAULTS  STORAGE",
        SettingsRoute::Devices => "DISPLAY  TOUCH  SENSOR",
        SettingsRoute::Privacy => "LOCATION  CAMERA  MICROPHONE",
        SettingsRoute::Time => "CLOCK  REGION  CALENDAR",
    };
    let body = match route {
        SettingsRoute::Main => "READY",
        SettingsRoute::System => "BRIGHTNESS 72   FPS TARGET 60   DIRTY TRACK ON",
        SettingsRoute::Personalization => "THEME LUMIA DARK   TILE STYLE LIVE   ACCENT CYAN",
        SettingsRoute::Network => "WIFI READY   BT READY   INPUT SOURCE MOCK",
        SettingsRoute::About => "DECLARATIVE ECS   PURE 3D FHRE   NO CRATES",
        SettingsRoute::Account => "ACCOUNT LOCAL   RESOURCE CACHE READY",
        SettingsRoute::Apps => "NEWS  STARS  THERMAL  FHRE SAMPLE",
        SettingsRoute::Devices => "FRAMEBUFFER BACKEND   TOUCH INPUT",
        SettingsRoute::Privacy => "SHELL DEMO DATA ONLY   NO SERVICES",
        SettingsRoute::Time => "10:08   MAY 15   SIM CLOCK PROFILE",
    };
    let card_y = scale_metric(content.w.min(content.h), 74).max(58) as i32 - scroll;
    let card_h = scale_metric(content.w.min(content.h), 120).max(96);
    ui.push_content_scrolled(
        UiSpec::text(UiKey(870), 0, card_y - 22 + scroll, 21, title, theme.text, metrics.label_scale),
        parent,
        scroll,
        list_clip,
    );
    ui.content_text(
        parent,
        UiKey(871),
        0,
        card_y - 8,
        21,
        subtitle,
        theme.muted,
        metrics.small_scale,
    );
    ui.push(
        UiSpec::rect(
            UiKey(872),
            UiKind::Tile,
            Rect::new(0, card_y, content.w, card_h),
            21,
            Color::rgba(25, 47, 53, 224),
            scale_metric(content.w.min(content.h), 10).max(8),
        )
        .with_content_parent(parent)
        .with_clip(list_clip),
    );
    ui.child_image_tint_fit(
        UiKey(872),
        UiKey(873),
        Rect::new(metrics.gap as i32, metrics.gap as i32, card_h / 3, card_h / 3),
        22,
        match route {
            SettingsRoute::System => WP_SYSTEM.image(),
            SettingsRoute::Personalization => WP_PERSONALIZATION.image(),
            SettingsRoute::Network => WP_NETWORK.image(),
            SettingsRoute::About => WP_ABOUT.image(),
            SettingsRoute::Account => WP_ACCOUNT.image(),
            SettingsRoute::Apps => WP_APPS.image(),
            SettingsRoute::Devices => WP_DEVICES.image(),
            SettingsRoute::Privacy => WP_PRIVACY.image(),
            SettingsRoute::Time => WP_TIME.image(),
            SettingsRoute::Main => WP_SETTINGS.image(),
        },
        255,
        ImageFit::Contain,
        accent,
    );
    ui.child_text(
        UiKey(872),
        UiKey(874),
        metrics.gap as i32,
        card_h as i32 - metrics.gap as i32 * 3,
        22,
        body,
        Color::rgba(238, 248, 255, 226),
        metrics.small_scale,
    );
    ui.content_text(
        parent,
        UiKey(875),
        0,
        content.h.saturating_sub(8) as i32,
        21,
        "SWIPE LEFT/RIGHT: HOME",
        Color::rgba(195, 218, 232, 176),
        metrics.small_scale,
    );
}

fn append_brightness_slider<const N: usize>(
    ui: &mut UiBuilder<N>,
    parent: UiKey,
    content_w: u16,
    y: i32,
    accent: Color,
    metrics: LumiaMetrics,
) {
    let h = scale_metric(content_w, 14).max(10).min(18);
    let track_w = content_w.saturating_sub(metrics.gap.saturating_mul(2));
    let filled_w = track_w.saturating_mul(72).saturating_div(100);
    ui.content_text(
        parent,
        UiKey(930),
        0,
        y,
        21,
        "BRIGHTNESS",
        Color::rgba(248, 250, 255, 216),
        metrics.small_scale,
    );
    ui.content_bar(
        parent,
        UiKey(931),
        Rect::new(0, y + scale_metric(content_w, 18).max(14) as i32, track_w, h),
        21,
        Color::rgba(255, 255, 255, 54),
        h / 2,
    );
    ui.content_bar(
        parent,
        UiKey(932),
        Rect::new(0, y + scale_metric(content_w, 18).max(14) as i32, filled_w, h),
        22,
        accent,
        h / 2,
    );
}

fn append_settings_nav<const N: usize>(
    ui: &mut UiBuilder<N>,
    w: u16,
    h: u16,
    metrics: LumiaMetrics,
    theme: LumiaTheme,
) {
    let nav_y = h.saturating_sub(metrics.nav_h) as i32;
    let center_y = nav_y + metrics.nav_h as i32 / 2;
    let icon = Color::rgba(245, 248, 255, 218);
    let third = (w / 3).max(1) as i32;
    let icon_edge = scale_metric(w.min(h), 22).max(18);
    ui.bar(UiKey(950), Rect::new(0, nav_y, w, metrics.nav_h), 24, theme.nav, 0);
    ui.push(UiSpec::rect(UiKey(951), UiKind::Tile, Rect::new(0, nav_y, (w / 3).max(1), metrics.nav_h), 24, Color::TRANSPARENT, 0).with_action(ACTION_HOME));
    ui.push(UiSpec::rect(UiKey(952), UiKind::Tile, Rect::new(third, nav_y, (w / 3).max(1), metrics.nav_h), 24, Color::TRANSPARENT, 0).with_action(ACTION_HOME));
    ui.push(UiSpec::rect(UiKey(953), UiKind::Tile, Rect::new(third * 2, nav_y, w.saturating_sub((w / 3).saturating_mul(2)), metrics.nav_h), 24, Color::TRANSPARENT, 0).with_action(ACTION_HOME));
    ui.image_tint(
        UiKey(954),
        Rect::new(third / 2 - icon_edge as i32 / 2, center_y - icon_edge as i32 / 2, icon_edge, icon_edge),
        25,
        WP_BACK.image(),
        220,
        ImageFit::Contain,
        icon,
    );
    ui.image_tint(
        UiKey(955),
        Rect::new(w as i32 / 2 - icon_edge as i32 / 2, center_y - icon_edge as i32 / 2, icon_edge, icon_edge),
        25,
        WP_LOGO.image(),
        220,
        ImageFit::Contain,
        icon,
    );
    ui.image_tint(
        UiKey(956),
        Rect::new(third * 2 + third / 2 - icon_edge as i32 / 2, center_y - icon_edge as i32 / 2, icon_edge, icon_edge),
        25,
        WP_SEARCH.image(),
        220,
        ImageFit::Contain,
        icon,
    );
}

fn settings_content_height(route: SettingsRoute, row_h: u16, row_gap: u16, content: Rect, short: u16) -> u16 {
    match route {
        SettingsRoute::Main => {
            let rows = 9u16;
            rows.saturating_mul(row_h.saturating_add(row_gap))
                .saturating_sub(row_gap)
                .saturating_add(scale_metric(short, 24).max(18))
        }
        _ => scale_metric(content.w.min(content.h), 230).max(190),
    }
}
