use crate::{
    app::{AppId, APP_FHRE_SAMPLE, APP_NEWS, APP_STARS, APP_THERMAL},
    asset::{EMBEDDED_TILE, NEWS_IMAGE, NEWS_TILE, SKY_BG, STARS_ICON, WEATHER_ICON},
    demo::WingDemoState,
    demo_overlay::render_overlay,
    key::UiKey,
    theme::{scale_metric, LumiaMetrics, LumiaTheme},
};
use fhre::{Color, DepthSpan, DrawCommand, DrawList, ImageFit, Point, Rect, Surface};

pub(crate) fn draw_sample_app(surface: &mut Surface, frame: u32, state: &mut WingDemoState, app: AppId) {
    if app == APP_NEWS {
        draw_news_app(surface, frame, state);
    } else if app == APP_STARS {
        draw_stars_app(surface, frame, state);
    } else if app == APP_THERMAL {
        draw_thermal_app(surface, frame, state);
    } else {
        draw_fhre_scene(surface, frame);
        draw_sample_hud(surface, frame, state, app);
    }
}

fn draw_fhre_scene(surface: &mut Surface, frame: u32) {
    let w = surface.width();
    let h = surface.height();
    let short = w.min(h);
    let mut list: DrawList<96> = DrawList::new();
    list.push(DrawCommand::FillGradient {
        rect: Rect::new(0, 0, w, h),
        depth: -48,
        top: Color::rgb(4, 10, 20),
        bottom: Color::rgb(0, 40, 56),
    });

    let horizon = h as i32 * 58 / 100;
    list.push(DrawCommand::FillRect {
        rect: Rect::new(0, horizon, w, h.saturating_sub(horizon as u16)),
        depth: -42,
        color: Color::rgba(0, 70, 92, 210),
    });

    let grid = scale_metric(short, 16).max(12) as i32;
    let vanishing = Point::new(w as i32 / 2 + wave(frame, 41, 24), horizon - scale_metric(short, 18).max(12) as i32);
    for i in 0..9 {
        let x = (i as i32 * w as i32) / 8;
        list.push(DrawCommand::StrokeLine {
            from: Point::new(x, h as i32),
            to: vanishing,
            depth: -38,
            width: 1,
            color: Color::rgba(68, 220, 232, 92),
        });
    }
    let mut gy = horizon;
    while gy < h as i32 {
        let alpha = 132u8.saturating_sub(((gy - horizon) / 5).min(90) as u8);
        list.push(DrawCommand::StrokeLine {
            from: Point::new(0, gy),
            to: Point::new(w as i32, gy),
            depth: -37,
            width: 1,
            color: Color::rgba(68, 220, 232, alpha),
        });
        gy += grid;
    }

    let cx = w as i32 / 2;
    let cy = h as i32 / 2 + scale_metric(short, 22).max(14) as i32;
    let size = scale_metric(short, 54).max(44) as i32;
    let ox = wave(frame, 0, size / 3);
    let oy = wave(frame, 19, size / 4);
    let skew = wave(frame, 47, size / 5);
    let front = [
        Point::new(cx - size + ox / 3, cy - size + oy / 4),
        Point::new(cx + size + ox / 3, cy - size / 2 - oy / 5),
        Point::new(cx + size - ox / 4, cy + size + oy / 6),
        Point::new(cx - size - ox / 4, cy + size / 2 - oy / 3),
    ];
    let back = [
        Point::new(front[0].x + skew + size / 2, front[0].y - size / 2),
        Point::new(front[1].x + skew + size / 2, front[1].y - size / 2),
        Point::new(front[2].x + skew + size / 2, front[2].y - size / 2),
        Point::new(front[3].x + skew + size / 2, front[3].y - size / 2),
    ];

    list.push(DrawCommand::DrawTriangle {
        p0: back[0],
        p1: back[1],
        p2: back[2],
        depth: DepthSpan { a: -18, b: -18, c: -18 },
        color: Color::rgba(0, 128, 190, 120),
    });
    list.push(DrawCommand::DrawTriangle {
        p0: front[0],
        p1: front[1],
        p2: front[2],
        depth: DepthSpan { a: -12, b: -12, c: -12 },
        color: Color::rgba(0, 188, 242, 150),
    });
    list.push(DrawCommand::DrawTriangle {
        p0: front[0],
        p1: front[2],
        p2: front[3],
        depth: DepthSpan { a: -11, b: -11, c: -11 },
        color: Color::rgba(0, 120, 215, 166),
    });

    let edge = Color::rgba(235, 250, 255, 218);
    for i in 0..4 {
        let next = (i + 1) & 3;
        list.push(DrawCommand::StrokeLine {
            from: front[i],
            to: front[next],
            depth: -6,
            width: 2,
            color: edge,
        });
        list.push(DrawCommand::StrokeLine {
            from: back[i],
            to: back[next],
            depth: -7,
            width: 1,
            color: Color::rgba(235, 250, 255, 128),
        });
        list.push(DrawCommand::StrokeLine {
            from: front[i],
            to: back[i],
            depth: -5,
            width: 1,
            color: Color::rgba(235, 250, 255, 166),
        });
    }

    for i in 0..12 {
        let x = ((i * 53 + 17 + (frame as usize & 31)) % w.max(1) as usize) as i32;
        let y = ((i * 31 + 11) % horizon.max(1) as usize) as i32;
        let r = 1 + ((i + frame as usize) & 1) as u16;
        list.push(DrawCommand::FillCircle {
            center: Point::new(x, y),
            depth: -30,
            radius: r,
            color: Color::rgba(210, 242, 255, 150),
        });
    }

    list.sort_by_depth();
    list.execute(surface);
}

fn draw_sample_hud(surface: &mut Surface, frame: u32, state: &mut WingDemoState, app: AppId) {
    let w = surface.width();
    let h = surface.height();
    let short = w.min(h);
    let metrics = LumiaMetrics::for_screen(w, h);
    let theme = LumiaTheme::DARK;
    let ui = &mut state.overlay_builder;
    ui.clear();
    ui.panel(
        UiKey(1000),
        Rect::new(
            metrics.pad_x as i32,
            metrics.status_h as i32,
            w.saturating_sub(metrics.pad_x.saturating_mul(2)),
            scale_metric(short, 42).max(34),
        ),
        20,
        Color::rgba(0, 0, 0, 122),
        0,
    );
    ui.text(
        UiKey(1001),
        metrics.pad_x as i32 + metrics.gap as i32,
        metrics.status_h as i32 + metrics.gap as i32,
        21,
        app_title(app),
        theme.text,
        metrics.label_scale,
    );
    ui.text(
        UiKey(1002),
        metrics.pad_x as i32 + metrics.gap as i32,
        metrics.status_h as i32 + metrics.gap as i32 + metrics.label_scale as i32 * 11,
        21,
        "3D-LIKE DRAWLIST + TRIANGLES",
        theme.muted,
        metrics.small_scale,
    );
    let pulse = 128u8.saturating_add(((frame as u8) & 63).saturating_mul(2));
    ui.bar(
        UiKey(1003),
        Rect::new(
            metrics.pad_x as i32,
            h.saturating_sub(metrics.nav_h).saturating_sub(scale_metric(short, 4).max(3)) as i32,
            w.saturating_sub(metrics.pad_x.saturating_mul(2)),
            scale_metric(short, 4).max(3),
        ),
        21,
        Color::rgba(theme.accent.r, theme.accent.g, theme.accent.b, pulse),
        2,
    );
    render_overlay(surface, state);
}

fn draw_news_app(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    let w = surface.width();
    let h = surface.height();
    let short = w.min(h);
    let metrics = LumiaMetrics::for_screen(w, h);
    let theme = LumiaTheme::DARK;
    let mut list: DrawList<16> = DrawList::new();
    list.push(DrawCommand::FillRect {
        rect: Rect::new(0, 0, w, h),
        depth: -20,
        color: Color::rgb(5, 5, 5),
    });
    list.push(DrawCommand::DrawImageFit {
        rect: Rect::new(0, metrics.status_h as i32, w, h.saturating_sub(metrics.nav_h + metrics.status_h)),
        depth: -18,
        image: NEWS_IMAGE.image(),
        opacity: 220,
        fit: ImageFit::Cover,
    });
    list.execute(surface);

    let ui = &mut state.overlay_builder;
    ui.clear();
    ui.panel(
        UiKey(2100),
        Rect::new(metrics.pad_x as i32, metrics.status_h as i32 + metrics.gap as i32, w.saturating_sub(metrics.pad_x * 2), scale_metric(short, 98).max(78)),
        20,
        Color::rgba(0, 0, 0, 168),
        0,
    );
    ui.text(UiKey(2101), metrics.pad_x as i32 + 12, metrics.status_h as i32 + 14, 21, "NEWS", theme.text, if short >= 430 { 3 } else { 2 });
    ui.text(UiKey(2102), metrics.pad_x as i32 + 12, metrics.status_h as i32 + 44, 21, "LIVE TILE STORY FEED", theme.muted, metrics.label_scale);
    ui.image_fit(
        UiKey(2103),
        Rect::new(metrics.pad_x as i32 + 12, metrics.status_h as i32 + scale_metric(short, 74).max(58) as i32, scale_metric(short, 90).max(72), scale_metric(short, 52).max(42)),
        21,
        NEWS_TILE.image(),
        230,
        ImageFit::Cover,
    );
    ui.text(
        UiKey(2104),
        metrics.pad_x as i32 + 12,
        h.saturating_sub(metrics.nav_h).saturating_sub(scale_metric(short, 24).max(18)) as i32,
        21,
        "SWIPE LEFT/RIGHT HOME",
        Color::rgba(255, 255, 255, 192),
        metrics.small_scale,
    );
    render_overlay(surface, state);
    let _ = frame;
}

fn draw_stars_app(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    let w = surface.width();
    let h = surface.height();
    let short = w.min(h);
    let metrics = LumiaMetrics::for_screen(w, h);
    let mut list: DrawList<32> = DrawList::new();
    list.push(DrawCommand::FillRect {
        rect: Rect::new(0, 0, w, h),
        depth: -30,
        color: Color::rgb(2, 4, 14),
    });
    list.push(DrawCommand::DrawImageFit {
        rect: Rect::new(0, 0, w, h),
        depth: -28,
        image: SKY_BG.image(),
        opacity: 230,
        fit: ImageFit::Cover,
    });
    for i in 0..14 {
        let x = ((i * 37 + frame as usize * 2) % w.max(1) as usize) as i32;
        let y = ((i * 29 + 13) % h.saturating_sub(metrics.nav_h).max(1) as usize) as i32;
        list.push(DrawCommand::FillCircle {
            center: Point::new(x, y),
            depth: -20,
            radius: 1 + ((frame as usize + i) & 1) as u16,
            color: Color::rgba(228, 244, 255, 150),
        });
    }
    list.execute(surface);

    let ui = &mut state.overlay_builder;
    ui.clear();
    ui.panel(UiKey(2200), Rect::new(0, 0, w, h), 20, Color::rgba(0, 0, 0, 70), 0);
    ui.image_tint(
        UiKey(2201),
        Rect::new(w as i32 / 2 - scale_metric(short, 38).max(30) as i32 / 2, metrics.status_h as i32 + 24, scale_metric(short, 38).max(30), scale_metric(short, 38).max(30)),
        22,
        STARS_ICON.image(),
        255,
        ImageFit::Contain,
        Color::rgba(255, 255, 255, 236),
    );
    ui.text(UiKey(2202), metrics.pad_x as i32 + 12, h as i32 / 2 - 18, 22, "STARS", Color::WHITE, if short >= 430 { 3 } else { 2 });
    ui.text(UiKey(2203), metrics.pad_x as i32 + 12, h as i32 / 2 + 12, 22, "FHRE SKY + MESH PREVIEW", Color::rgba(210, 226, 238, 196), metrics.label_scale);
    ui.text(
        UiKey(2204),
        metrics.pad_x as i32 + 12,
        h.saturating_sub(metrics.nav_h).saturating_sub(scale_metric(short, 24).max(18)) as i32,
        22,
        "SWIPE LEFT/RIGHT HOME",
        Color::rgba(255, 255, 255, 192),
        metrics.small_scale,
    );
    render_overlay(surface, state);
}

fn draw_thermal_app(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    let w = surface.width();
    let h = surface.height();
    let short = w.min(h);
    let metrics = LumiaMetrics::for_screen(w, h);
    let mut list: DrawList<96> = DrawList::new();
    list.push(DrawCommand::FillGradient {
        rect: Rect::new(0, 0, w, h),
        depth: -20,
        top: Color::rgb(4, 12, 18),
        bottom: Color::rgb(54, 14, 28),
    });
    list.push(DrawCommand::DrawImageFit {
        rect: Rect::new(
            metrics.pad_x as i32,
            metrics.status_h as i32 + scale_metric(short, 10).max(8) as i32,
            w.saturating_sub(metrics.pad_x.saturating_mul(2)),
            scale_metric(short, 82).max(64),
        ),
        depth: -18,
        image: EMBEDDED_TILE.image(),
        opacity: 180,
        fit: ImageFit::Cover,
    });
    let grid_x = metrics.pad_x as i32 + 14;
    let grid_y = metrics.status_h as i32 + 72;
    let cell = scale_metric(short, 22).max(18);
    for row in 0..8 {
        for col in 0..8 {
            let heat = ((row * 17 + col * 23 + frame as usize) & 127) as u8;
            let color = Color::rgba(heat.saturating_add(96), 40u8.saturating_add(heat / 3), 180u8.saturating_sub(heat), 224);
            list.push(DrawCommand::FillRect {
                rect: Rect::new(grid_x + col as i32 * cell as i32, grid_y + row as i32 * cell as i32, cell, cell),
                depth: -10,
                color,
            });
        }
    }
    list.execute(surface);

    let ui = &mut state.overlay_builder;
    ui.clear();
    ui.text(UiKey(2300), metrics.pad_x as i32 + 12, metrics.status_h as i32 + 18, 22, "THERMAL", Color::WHITE, if short >= 430 { 3 } else { 2 });
    ui.text(UiKey(2301), metrics.pad_x as i32 + 12, metrics.status_h as i32 + 48, 22, "LOW POWER SENSOR VIEW", Color::rgba(210, 226, 238, 190), metrics.label_scale);
    ui.image_tint(
        UiKey(2302),
        Rect::new(w.saturating_sub(metrics.pad_x).saturating_sub(42) as i32, metrics.status_h as i32 + 18, 34, 34),
        22,
        WEATHER_ICON.image(),
        255,
        ImageFit::Contain,
        Color::rgba(255, 255, 255, 230),
    );
    ui.text(
        UiKey(2303),
        metrics.pad_x as i32 + 12,
        h.saturating_sub(metrics.nav_h).saturating_sub(scale_metric(short, 24).max(18)) as i32,
        22,
        "SWIPE LEFT/RIGHT HOME",
        Color::rgba(255, 255, 255, 192),
        metrics.small_scale,
    );
    render_overlay(surface, state);
}

fn app_title(app: AppId) -> &'static str {
    if app == APP_FHRE_SAMPLE {
        "FHRE SAMPLE APP"
    } else {
        "WING APP"
    }
}

fn wave(frame: u32, phase: u32, amplitude: i32) -> i32 {
    let t = ((frame.wrapping_add(phase)) & 63) as i32;
    let up = if t < 32 { t } else { 63 - t };
    ((up * 2 - 31) * amplitude) / 31
}
