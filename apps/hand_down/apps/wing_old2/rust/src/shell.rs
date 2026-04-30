use crate::animation::{SlideDirection, SlideTransition};
use crate::app::{
    AppId, AppLaunchRequest, AppManifest, BuiltinAppId, PlatformTaskCapabilities, TaskSurface,
};
use crate::core::{Entity, Phase};
use crate::diagnostics::{RuntimeDiagnostics, RuntimeFrameFlags};
use crate::input::{KeyCode, Swipe};
use crate::math::{Color, Point, Rect};
use crate::platform::Platform;
use crate::render::{
    DirtyRegion, DirtyRegionSummary, DrawCmd, DrawTaskKind, DrawTaskSummary, EffectKind,
    EffectParams, FontCacheSummary, FontResourceSource, FontStore, FontWarmupSummary,
    GlyphCachePressure, GlyphCacheProfile, ImageId, RendererBackend, RendererCapabilities,
    RendererKind, RenderBackgroundKind, RenderBackgroundRoute, RenderLayerSummary,
    SvgCachePressure, SvgCacheProfile, SvgCacheSummary, TextureResourceSource, VectorIcon,
    IMAGE_WALLPAPER_AURORA_LANDSCAPE, IMAGE_WALLPAPER_AURORA_PORTRAIT,
    IMAGE_WALLPAPER_DUSK_LANDSCAPE, IMAGE_WALLPAPER_DUSK_PORTRAIT,
};
use crate::runtime::WingRuntime;
use crate::resource_file::RuntimeResourceSummary;
use crate::settings::SettingsStorageSource;
use crate::shell_notification::{NotificationEntry, NotificationIcon};
use crate::shell_style::ShellStyleTokens;
use crate::surface::SurfaceCapabilities;
use crate::ui::{
    ButtonAction, Layer, Layout, UiFrame, UiKey, UiLayer, UiPadding, Visual,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellMode {
    Home,
    Notification,
    AppSwitcher,
    ExternalApp,
    Settings,
    SystemInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeKind {
    Aurora,
    Dusk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewEffect {
    Cards,
    Soccer,
    Cube,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuickControlState {
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub airplane_mode: bool,
    pub dnd_enabled: bool,
    pub light_enabled: bool,
    pub sync_enabled: bool,
}

impl Default for QuickControlState {
    fn default() -> Self {
        Self {
            wifi_enabled: true,
            bluetooth_enabled: true,
            airplane_mode: false,
            dnd_enabled: false,
            light_enabled: true,
            sync_enabled: true,
        }
    }
}

const PREVIEW_ITEM_LIMIT: usize = 5;
const SHELL_VECTOR_ICON_COUNT: u8 = 20;
const SHELL_GLYPH_WORKSET_TIME_PREFIX: &str = "time=";
const SHELL_GLYPH_WORKSET_SCALE1_PREFIX: &str = "scale1=";
const SHELL_GLYPH_WORKSET_SCALE2_PREFIX: &str = "scale2=";
const SHELL_GLYPH_FALLBACK_TIME: &str = "0108:";
const SHELL_GLYPH_FALLBACK_SCALE_1: &str =
    " 0123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz,./+-<>";
const SHELL_GLYPH_FALLBACK_SCALE_2: &str = " 24ACEFGHIMNOPRSTWY";

pub fn warm_shell_font_cache(
    fonts: &FontStore,
    height: u16,
    workset: Option<&[u8]>,
) -> FontWarmupSummary {
    if let Some(summary) = workset.and_then(|bytes| warm_shell_font_cache_from_resource(fonts, height, bytes)) {
        return summary;
    }

    warm_shell_font_cache_from_sets(
        fonts,
        height,
        SHELL_GLYPH_FALLBACK_TIME,
        SHELL_GLYPH_FALLBACK_SCALE_1,
        SHELL_GLYPH_FALLBACK_SCALE_2,
    )
}

fn warm_shell_font_cache_from_resource(
    fonts: &FontStore,
    height: u16,
    bytes: &[u8],
) -> Option<FontWarmupSummary> {
    let text = core::str::from_utf8(bytes).ok()?;
    let mut summary = FontWarmupSummary::empty();
    let mut matched = false;

    for raw in text.lines() {
        let line = trim_resource_line(raw);
        if line.is_empty() || line.as_bytes()[0] == b'#' {
            continue;
        }
        if let Some(chars) = line.strip_prefix(SHELL_GLYPH_WORKSET_TIME_PREFIX) {
            summary.merge(fonts.warmup_chars(chars, home_time_scale(height)));
            matched = true;
        } else if let Some(chars) = line.strip_prefix(SHELL_GLYPH_WORKSET_SCALE1_PREFIX) {
            summary.merge(fonts.warmup_chars(chars, 1));
            matched = true;
        } else if let Some(chars) = line.strip_prefix(SHELL_GLYPH_WORKSET_SCALE2_PREFIX) {
            summary.merge(fonts.warmup_chars(chars, 2));
            matched = true;
        }
    }

    if matched {
        Some(summary)
    } else {
        None
    }
}

fn warm_shell_font_cache_from_sets(
    fonts: &FontStore,
    height: u16,
    time: &str,
    scale1: &str,
    scale2: &str,
) -> FontWarmupSummary {
    let mut summary = FontWarmupSummary::empty();
    summary.merge(fonts.warmup_chars(time, home_time_scale(height)));
    summary.merge(fonts.warmup_chars(scale1, 1));
    summary.merge(fonts.warmup_chars(scale2, 2));
    summary
}

fn trim_resource_line(line: &str) -> &str {
    let mut start = 0usize;
    let mut end = line.len();
    let bytes = line.as_bytes();
    while start < end && matches!(bytes[start], b' ' | b'\t' | b'\r') {
        start += 1;
    }
    while end > start && matches!(bytes[end - 1], b' ' | b'\t' | b'\r') {
        end -= 1;
    }
    &line[start..end]
}

fn home_time_scale(height: u16) -> u8 {
    if height < 560 {
        7
    } else {
        9
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThemedIcon {
    Wifi,
    Bluetooth,
    Phone,
    Chat,
    Settings,
    Camera,
    Flashlight,
    Airplane,
    Moon,
    Sync,
    Mail,
    Cloud,
    Folder,
    Music,
    Play,
    Wing,
    Check,
    Close,
    Alert,
    More,
    System,
    Terminal,
    Surface,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LaunchBanner {
    None,
    Started,
    Failed(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellState {
    pub mode: ShellMode,
    pub return_to: ShellMode,
    pub theme: ThemeKind,
    pub preview_effect: PreviewEffect,
    pub quick: QuickControlState,
    pub brightness: u8,
    pub haptic_enabled: bool,
    pub reduce_motion: bool,
    pub transition: SlideTransition,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            mode: ShellMode::Home,
            return_to: ShellMode::Home,
            theme: ThemeKind::Aurora,
            preview_effect: PreviewEffect::Cards,
            quick: QuickControlState::default(),
            brightness: 192,
            haptic_enabled: true,
            reduce_motion: false,
            transition: SlideTransition::none(),
        }
    }
}

pub fn install_shell<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>) {
    rt.schedule.add(Phase::Input, shell_input_system::<P, R>);
    rt.schedule.add(Phase::Update, shell_transition_system::<P, R>);
    rt.schedule.add(Phase::Layout, compose_shell_ui_system::<P, R>);
    rt.schedule.add(Phase::RenderBuild, build_draw_list_system::<P, R>);
}

fn shell_input_system<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>) {
    if matches!(rt.input.last_key_pressed, Some(KeyCode::Escape | KeyCode::Home)) {
        return_home(rt);
        return;
    }

    if matches!(rt.input.last_key_pressed, Some(KeyCode::KeyS)) {
        launch_app(rt, AppId(1));
        return;
    }

    let gesture = if rt.input.wheel_delta > 0 {
        Swipe::Up
    } else if rt.input.wheel_delta < 0 {
        Swipe::Down
    } else {
        rt.input.swipe
    };

    match (rt.shell.mode, gesture) {
        (ShellMode::Home, Swipe::Down) => {
            rt.shell.return_to = ShellMode::Home;
            enter_mode(rt, ShellMode::Notification, SlideDirection::FromTop);
        }
        (ShellMode::Home, Swipe::Up) => {
            rt.shell.return_to = ShellMode::Home;
            enter_mode(rt, ShellMode::AppSwitcher, SlideDirection::FromBottom);
        }
        (ShellMode::Notification, Swipe::Up) | (ShellMode::AppSwitcher, Swipe::Down) => {
            enter_mode(rt, rt.shell.return_to, return_transition(rt.shell.mode));
        }
        (ShellMode::Notification, Swipe::Left | Swipe::Right)
        | (ShellMode::AppSwitcher, Swipe::Left | Swipe::Right) => {
            enter_mode(rt, rt.shell.return_to, return_transition(rt.shell.mode));
        }
        (ShellMode::ExternalApp, Swipe::Down) => {
            rt.shell.return_to = ShellMode::ExternalApp;
            enter_mode(rt, ShellMode::Notification, SlideDirection::FromTop);
        }
        (ShellMode::ExternalApp, Swipe::Up) => {
            rt.shell.return_to = ShellMode::ExternalApp;
            enter_mode(rt, ShellMode::AppSwitcher, SlideDirection::FromBottom);
        }
        (ShellMode::ExternalApp, Swipe::Left | Swipe::Right)
        | (ShellMode::Settings, Swipe::Left | Swipe::Right)
        | (ShellMode::SystemInfo, Swipe::Left | Swipe::Right) => {
            return_home(rt);
        }
        _ => {}
    }

    if rt.input.primary.released {
        if let Some(action) = hit_button(rt, rt.input.pointer) {
            run_action(rt, action);
        }
    }
}

fn shell_transition_system<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>) {
    rt.shell.transition.advance();
}

fn compose_shell_ui_system<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>) {
    let width = rt.width;
    let height = rt.height;
    let shell = rt.shell;
    let renderer_kind = rt.renderer_kind();
    let renderer_capabilities = rt.renderer_capabilities();
    let task_capabilities = rt.task_capabilities();
    let surface_capabilities = rt.surface_capabilities();
    let settings_source = rt.settings_storage_source();
    let texture_source = rt.textures.resource_source();
    let font_source = rt.fonts.source();
    let font_cache = rt.fonts.cache_summary();
    let svg_cache = rt.svgs.cache_summary();
    let shell_wallpapers_ready = rt.textures.shell_wallpapers_ready();
    let runtime_resources = rt.runtime_resource_summary();
    let diagnostics = rt.diagnostics;

    rt.ui_frame.clear();
    if shell.mode != ShellMode::ExternalApp {
        compose_status(&mut rt.ui_frame, width, height, shell.theme);
    }

    match shell.mode {
        ShellMode::Home => compose_home(&mut rt.ui_frame, width, height, shell.theme),
        ShellMode::Notification => compose_notification(
            &mut rt.ui_frame,
            width,
            height,
            shell,
            rt.notifications.entries(),
        ),
        ShellMode::AppSwitcher => {
            let launch_banner = if let Some(error) = rt.app_manager.last_error() {
                LaunchBanner::Failed(error.label())
            } else if rt.app_manager.last_pid().is_some() {
                LaunchBanner::Started
            } else {
                LaunchBanner::None
            };
            compose_app_switcher(
                &mut rt.ui_frame,
                width,
                height,
                resolve_preview_effect(shell.preview_effect, renderer_capabilities),
                shell.theme,
                launch_banner,
                rt.app_registry.entries(),
            )
        }
        ShellMode::Settings => {
            compose_settings(&mut rt.ui_frame, width, height, shell, renderer_capabilities)
        }
        ShellMode::ExternalApp => compose_external_app(
            &mut rt.ui_frame,
            width,
            height,
            shell.theme,
            rt.app_manager.focused_surface().is_some(),
            rt.app_manager.last_error(),
        ),
        ShellMode::SystemInfo => compose_system_info(
            &mut rt.ui_frame,
            width,
            height,
            shell.theme,
            renderer_kind,
            renderer_capabilities,
            task_capabilities,
            surface_capabilities,
            settings_source,
            texture_source,
            font_source,
            font_cache,
            svg_cache,
            shell_wallpapers_ready,
            runtime_resources,
            diagnostics,
        ),
    }

    let (dx, dy) = shell.transition.offset(width, height);
    rt.ui_frame.translate_layer(mode_layer(shell.mode), dx, dy);
    rt.ui_frame.fade_layer(mode_layer(shell.mode), shell.transition.alpha());

    let revision = rt.frame_index().wrapping_add(1);
    let mut ui_damage = DirtyRegion::empty();
    if rt
        .world
        .apply_ui_frame(&rt.ui_frame, revision, |rect| ui_damage.include_rect(rect))
    {
        for rect in ui_damage.rects() {
            rt.mark_dirty(*rect);
        }
    }
}

fn compose_status(frame: &mut UiFrame, width: u16, _height: u16, _theme: ThemeKind) {
    frame.builder().layer(Layer::Always, |ui| {
        ui.text(k(1), 24, 24, 10, "10:08", Color::WHITE, 1);
        let right = width as i32 - 24;
        ui.icon(
            k(2),
            Rect::new(right - 94, 18, 24, 24),
            10,
            vector_icon(ThemedIcon::Phone),
            Color::rgba(255, 255, 255, 220),
        );
        ui.icon(
            k(3),
            Rect::new(right - 64, 18, 24, 24),
            10,
            vector_icon(ThemedIcon::Wifi),
            Color::rgba(255, 255, 255, 220),
        );
        ui.icon(
            k(4),
            Rect::new(right - 34, 18, 24, 24),
            10,
            vector_icon(ThemedIcon::Bluetooth),
            Color::rgba(255, 255, 255, 220),
        );
        ui.text(
            k(5),
            right - 7,
            23,
            10,
            "72",
            Color::rgba(255, 255, 255, 205),
            1,
        );
    });
}

fn compose_home(frame: &mut UiFrame, width: u16, height: u16, theme: ThemeKind) {
    let w = width as i32;
    let h = height as i32;
    let cx = w / 2;
    let time_scale = home_time_scale(height);
    let time_x = cx - if time_scale >= 9 { 58 } else { 46 };
    let time_y = h / 2 - if height < 560 { 104 } else { 142 };
    let launch_size = if height < 560 { 88u16 } else { 116u16 };
    let launch_rect = Rect::new(cx - launch_size as i32 / 2, h - 172, launch_size, launch_size);

    frame.builder().layer(Layer::Home, |ui| {
        ui.text(k(100), time_x, time_y, 12, "10", Color::WHITE, time_scale);
        ui.text(
            k(101),
            time_x,
            time_y + (time_scale as i32 * 18),
            12,
            "08",
            Color::rgba(255, 255, 255, 235),
            time_scale,
        );
        ui.text(
            k(102),
            cx - 52,
            time_y + (time_scale as i32 * 38),
            12,
            "THU, MAY 15",
            Color::rgba(255, 255, 255, 222),
            1,
        );
        ui.icon(
            k(103),
            Rect::new(cx - 30, time_y + (time_scale as i32 * 43), 22, 22),
            12,
            vector_icon(ThemedIcon::Cloud),
            Color::rgba(255, 255, 255, 220),
        );
        ui.text(
            k(104),
            cx - 2,
            time_y + (time_scale as i32 * 44),
            12,
            "24C",
            Color::rgba(255, 255, 255, 225),
            2,
        );

        ui.circle(
            k(110),
            Rect::new(launch_rect.x - 16, launch_rect.y - 16, launch_size + 32, launch_size + 32),
            10,
            Color::rgba(255, 255, 255, 24),
        );
        ui.circle(
            k(111),
            launch_rect,
            11,
            Color::rgba(255, 255, 255, 36),
        );
        ui.button(
            k(112),
            launch_rect,
            12,
            ButtonAction::OpenAppSwitcher,
            Visual::Circle {
                color: Color::rgba(255, 255, 255, 18),
            },
        );
        ui.icon(
            k(113),
            Rect::new(cx - 27, launch_rect.y + launch_rect.h as i32 / 2 - 27, 54, 54),
            13,
            vector_icon(ThemedIcon::Wing),
            Color::rgba(255, 255, 255, 230),
        );

        compose_home_resource_rail(
            ui,
            150,
            cx,
            launch_rect.y - if height < 560 { 46 } else { 60 },
            theme,
            height < 560,
        );

        let dock_y = h - 52;
        ui.button(
            k(120),
            Rect::new(28, dock_y - 28, 56, 56),
            12,
            ButtonAction::OpenSettings,
            Visual::Circle {
                color: Color::rgba(255, 255, 255, 34),
            },
        );
        ui.icon(
            k(121),
            Rect::new(43, dock_y - 13, 26, 26),
            13,
            vector_icon(ThemedIcon::Settings),
            Color::rgba(255, 255, 255, 220),
        );
        ui.button(
            k(122),
            Rect::new(w - 84, dock_y - 28, 56, 56),
            12,
            ButtonAction::OpenNotification,
            Visual::Circle {
                color: Color::rgba(255, 255, 255, 34),
            },
        );
        ui.icon(
            k(123),
            Rect::new(w - 69, dock_y - 13, 26, 26),
            13,
            vector_icon(ThemedIcon::Alert),
            Color::rgba(255, 255, 255, 220),
        );
        ui.circle(
            k(130),
            Rect::new(cx - 16, h - 35, 6, 6),
            12,
            Color::rgba(255, 255, 255, 220),
        );
        ui.circle(
            k(131),
            Rect::new(cx - 3, h - 35, 6, 6),
            12,
            Color::rgba(255, 255, 255, 128),
        );
        ui.circle(
            k(132),
            Rect::new(cx + 10, h - 35, 6, 6),
            12,
            Color::rgba(255, 255, 255, 128),
        );
        ui.text(
            k(140),
            cx - 92,
            h - 78,
            3,
            "SWIPE DOWN / UP",
            Color::rgba(255, 255, 255, 150),
            1,
        );
    });
}

fn compose_home_resource_rail(
    ui: &mut UiLayer<'_>,
    base: u16,
    cx: i32,
    y: i32,
    _theme: ThemeKind,
    compact: bool,
) {
    let icons = [
        ThemedIcon::Phone,
        ThemedIcon::Camera,
        ThemedIcon::Music,
        ThemedIcon::Folder,
    ];
    let chip = if compact { 42u16 } else { 52u16 };
    let step = if compact { 48i32 } else { 60i32 };
    let start = cx - step * 3 / 2 - chip as i32 / 2;

    for index in 0..icons.len() {
        let x = start + index as i32 * step;
        ui.icon(
            k(base + index as u16),
            Rect::new(x, y, chip, chip),
            13,
            vector_icon(icons[index]),
            Color::rgba(255, 255, 255, 230),
        );
    }
}

fn compose_notification(
    frame: &mut UiFrame,
    width: u16,
    height: u16,
    shell: ShellState,
    notifications: &[NotificationEntry],
) {
    let w = width as i32;
    let h = height as i32;
    let compact = height < 560;
    let panel = Rect::new(
        18,
        70,
        width.saturating_sub(36),
        height.saturating_sub(110),
    );
    let inner_x = 44;
    let inner_w = width.saturating_sub(88);
    let title_y = if compact { 92 } else { 104 };
    let brightness_y = title_y + if compact { 38 } else { 46 };
    let quick_y = brightness_y + if compact { 30 } else { 42 };
    let card_y = quick_y + if compact { 112 } else { 148 };
    let quick_gap = 8u16;
    let quick_h = if compact { 52 } else { 64 };
    let quick_w = inner_w.saturating_sub(quick_gap.saturating_mul(2)) / 3;
    let card_gap = if compact { 7u16 } else { 8u16 };
    let card_w = inner_w;
    let card_h = if compact { 48 } else { 58 };
    let bar_w = inner_w.saturating_sub(104);
    let fill_w = ((bar_w as u32 * shell.brightness as u32) / 255) as u16;
    let tokens = ShellStyleTokens::for_theme(shell.theme);

    frame.builder().layer(Layer::Notification, |ui| {
        shell_panel(ui, k(180), panel, tokens);
        ui.text(k(201), inner_x, title_y, 12, "NOTIFICATIONS", Color::WHITE, 2);
        ui.text(
            k(206),
            inner_x + 128,
            title_y + 7,
            12,
            "THU, MAY 15",
            Color::rgba(255, 255, 255, 190),
            1,
        );
        ui.icon(
            k(202),
            Rect::new(w - 82, title_y - 2, 34, 34),
            13,
            vector_icon(ThemedIcon::Settings),
            Color::rgba(255, 255, 255, 225),
        );

        ui.round_rect(
            k(203),
            Rect::new(inner_x, brightness_y - 10, inner_w, 32),
            12,
            16,
            Color::rgba(255, 255, 255, 222),
        );
        ui.icon(
            k(207),
            Rect::new(inner_x + 12, brightness_y - 5, 22, 22),
            13,
            vector_icon(ThemedIcon::Flashlight),
            tokens.accent.with_alpha(235),
        );
        ui.round_rect(
            k(204),
            Rect::new(inner_x + 52, brightness_y, bar_w, 10),
            12,
            6,
            Color::rgba(70, 88, 120, 42),
        );
        ui.round_rect(
            k(205),
            Rect::new(inner_x + 52, brightness_y, fill_w.max(8), 10),
            13,
            6,
            tokens.accent_bar,
        );

        let quick = [
            (
                ThemedIcon::Wifi,
                "WIFI",
                shell.quick.wifi_enabled && !shell.quick.airplane_mode,
                ButtonAction::ToggleWifi,
            ),
            (
                ThemedIcon::Bluetooth,
                "BT",
                shell.quick.bluetooth_enabled && !shell.quick.airplane_mode,
                ButtonAction::ToggleBluetooth,
            ),
            (
                ThemedIcon::Airplane,
                "AIR",
                shell.quick.airplane_mode,
                ButtonAction::ToggleAirplane,
            ),
            (
                ThemedIcon::Moon,
                "DND",
                shell.quick.dnd_enabled,
                ButtonAction::ToggleDnd,
            ),
            (
                ThemedIcon::Flashlight,
                "LITE",
                shell.quick.light_enabled,
                ButtonAction::ToggleLight,
            ),
            (
                ThemedIcon::Sync,
                "SYNC",
                shell.quick.sync_enabled,
                ButtonAction::ToggleSync,
            ),
        ];
        for index in 0..quick.len() {
            let column = index % 3;
            let row = index / 3;
            let rect = Rect::new(
                inner_x + column as i32 * (quick_w + quick_gap) as i32,
                quick_y + row as i32 * (quick_h + quick_gap) as i32,
                quick_w,
                quick_h,
            );
            let (icon, label, active, action) = quick[index];
            notification_toggle(
                ui,
                210 + index as u16 * 6,
                rect,
                shell.theme,
                tokens,
                icon,
                label,
                active,
                action,
            );
        }

        let bottom_limit = h - 64;
        let card_step = (card_h + card_gap) as i32;
        let visible_cards = if bottom_limit <= card_y {
            0usize
        } else {
            ((bottom_limit - card_y) / card_step).max(1) as usize
        };
        if visible_cards != 0 {
            let list_clip = Rect::new(
                inner_x,
                card_y,
                card_w,
                (bottom_limit - card_y).max(0) as u16,
            );
            ui.with_clip(list_clip, |ui| {
                for (index, notification) in notifications.iter().take(visible_cards).enumerate() {
                    let rect = Rect::new(
                        inner_x,
                        card_y + index as i32 * (card_h + card_gap) as i32,
                        card_w,
                        card_h,
                    );
                    notification_card(
                        ui,
                        250 + index as u16 * 8,
                        rect,
                        shell.theme,
                        tokens,
                        notification_icon(notification.icon),
                        notification.title,
                        notification.body,
                        notification.time,
                        notification.priority.is_high(),
                    );
                }
            });
        }

        ui.text(
            k(292),
            w / 2 - 58,
            h - 52,
            12,
            "SWIPE UP",
            Color::rgba(255, 255, 255, 175),
            1,
        );
        ui.round_rect(
            k(293),
            Rect::new(w / 2 - 22, h - 25, 44, 4),
            13,
            2,
            Color::rgba(255, 255, 255, 165),
        );
    });
}

fn notification_toggle(
    ui: &mut UiLayer<'_>,
    base: u16,
    rect: Rect,
    _theme: ThemeKind,
    tokens: ShellStyleTokens,
    icon: ThemedIcon,
    label: &'static str,
    active: bool,
    action: ButtonAction,
) {
    let face = Rect::new(rect.x, rect.y - 2, rect.w, rect.h);
    let content_key = ui.button_styled_round_rect(
        k(base),
        face,
        14,
        action,
        tokens.quick_toggle(active),
    );
    let icon_size = rect.h.saturating_sub(22).min(42);
    let icon_x = rect.x + rect.w as i32 / 2 - icon_size as i32 / 2;
    let icon_y = rect.y + 5;
    ui.icon(
        content_key,
        Rect::new(icon_x, icon_y, icon_size, icon_size),
        17,
        vector_icon(icon),
        tokens.quick_icon_tint(active),
    );
    ui.text(
        content_key.offset(1),
        rect.x + rect.w as i32 / 2 - 22,
        rect.y + rect.h as i32 - 17,
        17,
        label,
        tokens.quick_label_color(active),
        1,
    );
}

fn notification_icon(icon: NotificationIcon) -> ThemedIcon {
    match icon {
        NotificationIcon::Chat => ThemedIcon::Chat,
        NotificationIcon::Mail => ThemedIcon::Mail,
        NotificationIcon::Cloud => ThemedIcon::Cloud,
        NotificationIcon::Alert => ThemedIcon::Alert,
        NotificationIcon::Settings => ThemedIcon::Settings,
        NotificationIcon::Play => ThemedIcon::Play,
    }
}

fn notification_card(
    ui: &mut UiLayer<'_>,
    base: u16,
    rect: Rect,
    _theme: ThemeKind,
    tokens: ShellStyleTokens,
    icon: ThemedIcon,
    title: &'static str,
    body: &'static str,
    time: &'static str,
    priority: bool,
) {
    let title_color = tokens.notification_title_color(priority);
    let body_color = tokens.notification_body_color(priority);
    ui.styled_round_rect(
        k(base),
        rect,
        14,
        tokens.notification_card(priority),
    );
    let icon_size = rect.h.saturating_sub(12).min(48);
    let icon_y = rect.y + rect.h as i32 / 2 - icon_size as i32 / 2;
    ui.icon(
        k(base + 2),
        Rect::new(rect.x + 10, icon_y, icon_size, icon_size),
        17,
        vector_icon(icon),
        tokens.notification_icon_color(priority),
    );
    ui.text(
        k(base + 3),
        rect.x + icon_size as i32 + 26,
        rect.y + 12,
        16,
        title,
        title_color,
        1,
    );
    ui.text(
        k(base + 4),
        rect.x + icon_size as i32 + 26,
        rect.y + rect.h as i32 - 24,
        16,
        body,
        body_color,
        1,
    );
    ui.text(
        k(base + 5),
        rect.x + rect.w as i32 - 32,
        rect.y + 12,
        16,
        time,
        body_color,
        1,
    );
}

fn compose_settings(
    frame: &mut UiFrame,
    width: u16,
    height: u16,
    shell: ShellState,
    capabilities: RendererCapabilities,
) {
    let tokens = ShellStyleTokens::for_theme(shell.theme);
    frame.builder().layer(Layer::Settings, |ui| {
        shell_panel(
            ui,
            k(300),
            Rect::new(
                18,
                70,
                width.saturating_sub(36),
                height.saturating_sub(110),
            ),
            tokens,
        );
        back_button(ui, 302, Rect::new(38, 92, 86, 40), 20, tokens);
        ui.text(k(306), 44, 154, 12, "SETTINGS", Color::WHITE, 2);

        let mut rows = ui.vstack(
            Rect::new(44, 210, width.saturating_sub(88), 126),
            UiPadding::ZERO,
            18,
        );
        settings_row(
            ui,
            310,
            rows.next(54),
            ButtonAction::ToggleTheme,
            "THEME",
            tokens,
            match shell.theme {
                ThemeKind::Aurora => "AURORA",
                ThemeKind::Dusk => "DUSK",
            },
        );
        settings_row(
            ui,
            320,
            rows.next(54),
            ButtonAction::TogglePreviewEffect,
            "PREVIEW",
            tokens,
            preview_effect_label(resolve_preview_effect(
                shell.preview_effect,
                capabilities,
            )),
        );
    });
}

fn compose_system_info(
    frame: &mut UiFrame,
    width: u16,
    height: u16,
    theme: ThemeKind,
    renderer: RendererKind,
    capabilities: RendererCapabilities,
    task_capabilities: PlatformTaskCapabilities,
    surface_capabilities: SurfaceCapabilities,
    settings_source: SettingsStorageSource,
    texture_source: TextureResourceSource,
    font_source: FontResourceSource,
    font_cache: FontCacheSummary,
    svg_cache: SvgCacheSummary,
    shell_wallpapers_ready: bool,
    runtime_resources: RuntimeResourceSummary,
    diagnostics: RuntimeDiagnostics,
) {
    let tokens = ShellStyleTokens::for_theme(theme);
    frame.builder().layer(Layer::SystemInfo, |ui| {
        shell_panel(
            ui,
            k(340),
            Rect::new(
                18,
                70,
                width.saturating_sub(36),
                height.saturating_sub(110),
            ),
            tokens,
        );
        back_button(ui, 342, Rect::new(38, 92, 86, 40), 20, tokens);
        ui.text(k(346), 44, 154, 12, "SYSTEM", Color::WHITE, 2);

        const SYSTEM_ROW_COUNT: u16 = 19;
        let row_gap = if height < 520 { 1 } else { 4 };
        let row_top = if height < 520 { 176 } else { 206 };
        let row_area = height.saturating_sub(row_top + if height < 520 { 44 } else { 50 });
        let row_height = system_row_height(row_area, SYSTEM_ROW_COUNT, row_gap);
        let mut rows = ui.vstack(
            Rect::new(44, row_top as i32, width.saturating_sub(88), row_area),
            UiPadding::ZERO,
            row_gap,
        );
        system_row(ui, 350, rows.next(row_height), tokens, "RUNTIME", "NUTTX RUST");
        system_row(
            ui,
            360,
            rows.next(row_height),
            tokens,
            "RENDER",
            renderer_kind_label(renderer),
        );
        system_row(
            ui,
            370,
            rows.next(row_height),
            tokens,
            "CAPS",
            renderer_caps_label(capabilities),
        );
        system_row(
            ui,
            380,
            rows.next(row_height),
            tokens,
            "RESOURCE",
            texture_source_label(texture_source),
        );
        system_row(
            ui,
            390,
            rows.next(row_height),
            tokens,
            "BG",
            background_resource_label(texture_source, shell_wallpapers_ready, runtime_resources),
        );
        system_row(
            ui,
            400,
            rows.next(row_height),
            tokens,
            "FONT",
            font_resource_label(runtime_resources, font_source),
        );
        system_row(
            ui,
            410,
            rows.next(row_height),
            tokens,
            "GLYPH",
            glyph_cache_label(font_cache),
        );
        system_row(
            ui,
            420,
            rows.next(row_height),
            tokens,
            "GHIT",
            glyph_cache_health_label(font_cache),
        );
        system_row(
            ui,
            430,
            rows.next(row_height),
            tokens,
            "ICON",
            icon_resource_label(runtime_resources),
        );
        system_row(
            ui,
            435,
            rows.next(row_height),
            tokens,
            "SVG",
            svg_cache_label(svg_cache),
        );
        system_row(
            ui,
            440,
            rows.next(row_height),
            tokens,
            "FILES",
            resource_files_label(runtime_resources),
        );
        system_row(
            ui,
            445,
            rows.next(row_height),
            tokens,
            "SURFACE",
            surface_caps_label(surface_capabilities),
        );
        system_row(
            ui,
            450,
            rows.next(row_height),
            tokens,
            "TASK",
            task_caps_label(task_capabilities),
        );
        system_row(
            ui,
            460,
            rows.next(row_height),
            tokens,
            "SETTINGS",
            settings_source_label(settings_source),
        );
        system_row(
            ui,
            470,
            rows.next(row_height),
            tokens,
            "DRAW",
            draw_task_label(diagnostics.frames().last_draw_tasks()),
        );
        system_row(
            ui,
            475,
            rows.next(row_height),
            tokens,
            "LAYER",
            render_layer_label(diagnostics.frames().last_render_plan().layers),
        );
        system_row(
            ui,
            480,
            rows.next(row_height),
            tokens,
            "DIRTY",
            dirty_region_label(diagnostics.frames().last_dirty_region()),
        );
        system_row(
            ui,
            490,
            rows.next(row_height),
            tokens,
            "FRAME",
            runtime_frame_label(diagnostics),
        );
        system_row(
            ui,
            500,
            rows.next(row_height),
            tokens,
            "HEALTH",
            runtime_health_label(diagnostics),
        );
    });
}

fn compose_external_app(
    frame: &mut UiFrame,
    width: u16,
    height: u16,
    theme: ThemeKind,
    has_surface: bool,
    last_error: Option<crate::app::TaskLaunchError>,
) {
    if has_surface && last_error.is_none() {
        return;
    }

    let tokens = ShellStyleTokens::for_theme(theme);
    frame.builder().layer(Layer::ExternalApp, |ui| {
        if let Some(error) = last_error {
            shell_panel(
                ui,
                k(395),
                Rect::new(
                    18,
                    70,
                    width.saturating_sub(36),
                    height.saturating_sub(110),
                ),
                tokens,
            );
            ui.text(
                k(396),
                44,
                height as i32 - 84,
                35,
                error.label(),
                Color::rgba(255, 220, 220, 230),
                2,
            );
        } else if !has_surface {
            shell_panel(
                ui,
                k(395),
                Rect::new(
                    18,
                    70,
                    width.saturating_sub(36),
                    height.saturating_sub(110),
                ),
                tokens,
            );
            ui.text(
                k(396),
                44,
                height as i32 - 84,
                35,
                "DETACHED APP",
                Color::rgba(255, 255, 255, 210),
                2,
            );
        }
    });
}

fn compose_app_switcher(
    frame: &mut UiFrame,
    width: u16,
    height: u16,
    effect: PreviewEffect,
    theme: ThemeKind,
    launch_banner: LaunchBanner,
    apps: &[AppManifest],
) {
    let tokens = ShellStyleTokens::for_theme(theme);
    frame.builder().layer(Layer::AppSwitcher, |ui| {
        shell_panel(
            ui,
            k(400),
            Rect::new(18, 70, width.saturating_sub(36), height.saturating_sub(110)),
            tokens,
        );
        ui.text(k(401), 44, 104, 12, "APP SWITCHER", Color::WHITE, 2);

        match effect {
            PreviewEffect::Cards => compose_card_preview(ui, width, height, theme, apps),
            PreviewEffect::Soccer => compose_soccer_preview(ui, width, height, theme, apps),
            PreviewEffect::Cube => compose_cube_preview(ui, width, height, theme, apps),
        }

        match launch_banner {
            LaunchBanner::None => {}
            LaunchBanner::Started => ui.text(
                k(471),
                44,
                height as i32 - 104,
                12,
                "APP STARTED",
                Color::rgba(255, 255, 255, 210),
                1,
            ),
            LaunchBanner::Failed(label) => {
                ui.text(
                    k(471),
                    44,
                    height as i32 - 104,
                    12,
                    "LAUNCH FAILED",
                    Color::rgba(255, 210, 210, 230),
                    1,
                );
                ui.text(
                    k(472),
                    160,
                    height as i32 - 104,
                    12,
                    label,
                    Color::rgba(255, 255, 255, 190),
                    1,
                );
            }
        }

        ui.text(
            k(470),
            width as i32 / 2 - 68,
            height as i32 - 52,
            12,
            "SWIPE DOWN",
            Color::rgba(255, 255, 255, 175),
            1,
        );
        ui.round_rect(
            k(473),
            Rect::new(width as i32 / 2 - 24, height as i32 - 25, 48, 4),
            13,
            2,
            Color::rgba(255, 255, 255, 150),
        );
    });
}

fn compose_card_preview(
    ui: &mut UiLayer<'_>,
    width: u16,
    height: u16,
    theme: ThemeKind,
    apps: &[AppManifest],
) {
    let compact = height < 560;
    let cx = width as i32 / 2;
    let card_w = width.saturating_sub(118).min(340);
    let card_h = if compact { 118 } else { 166 };
    let stack_top = if compact { 126 } else { 152 };
    let step_y = if compact { 34 } else { 42 };
    let tokens = ShellStyleTokens::for_theme(theme);

    for index in 0..apps.len().min(4) {
        let app = apps[index];
        let inset = index as i32 * 10;
        let rect = Rect::new(
            cx - card_w as i32 / 2 + inset,
            stack_top + index as i32 * step_y,
            card_w.saturating_sub(index as u16 * 20),
            card_h,
        );
        let base = 700 + index as u16 * 24;
        let z = 14 + index as i16 * 3;
        let icon_size = if compact { 36 } else { 44 };
        let front = index == apps.len().min(4).saturating_sub(1);

        let content_key = ui.button_styled_round_rect(
            k(base),
            rect,
            z,
            ButtonAction::LaunchApp(app.id),
            tokens.app_card,
        );
        ui.icon(
            content_key,
            Rect::new(rect.x + 14, rect.y + 11, icon_size, icon_size),
            z + 3,
            app_icon(app),
            tokens.app_icon_color(),
        );
        ui.text(
            content_key.offset(1),
            rect.x + 66,
            rect.y + 24,
            z + 3,
            app.name,
            Color::rgba(18, 32, 58, 238),
            1,
        );
        ui.text(
            content_key.offset(2),
            rect.x + 66,
            rect.y + 46,
            z + 3,
            app_subtitle(app),
            Color::rgba(52, 70, 100, 190),
            1,
        );
        ui.icon(
            content_key.offset(3),
            Rect::new(rect.x + rect.w as i32 - 38, rect.y + 16, 24, 24),
            z + 3,
            vector_icon(ThemedIcon::More),
            Color::rgba(22, 34, 58, 170),
        );

        if front {
            compose_gallery_preview(ui, base + 9, rect, theme, width, height, z + 2);
        }
    }

    compose_preview_bottom_controls(ui, width, height, theme);
    ui.text(
        k(468),
        44,
        height as i32 - 132,
        12,
        "CARDS",
        Color::rgba(255, 255, 255, 175),
        1,
    );
}

fn compose_gallery_preview(
    ui: &mut UiLayer<'_>,
    base: u16,
    rect: Rect,
    theme: ThemeKind,
    width: u16,
    height: u16,
    z: i16,
) {
    let compact = rect.h < 140;
    let gallery_x = rect.x + 16;
    let gallery_w = rect.w.saturating_sub(32);
    let main_h = if compact { 34 } else { 54 };
    let main_y = rect.y + if compact { 58 } else { 64 };
    let main = Rect::new(gallery_x, main_y, gallery_w, main_h);

    ui.round_rect(
        k(base),
        Rect::new(
            main.x - 2,
            main.y - 2,
            main.w.saturating_add(4),
            main.h.saturating_add(4),
        ),
        z,
        10,
        Color::rgba(10, 22, 42, 80),
    );
    ui.image(
        k(base + 1),
        main,
        z + 1,
        theme_wallpaper(theme, width, height),
        Color::rgba(255, 255, 255, 250),
    );

    let chip_icons = [
        ThemedIcon::Camera,
        ThemedIcon::Music,
        ThemedIcon::Phone,
        ThemedIcon::Check,
    ];
    let chip_size = if compact { 18 } else { 22 };
    let chip_gap = if compact { 5 } else { 7 };
    let chip_y = main.y + if compact { 7 } else { 9 };
    for index in 0..chip_icons.len() {
        let chip_x = main.x + main.w as i32
            - (chip_size + 8) as i32
            - index as i32 * (chip_size + chip_gap) as i32;
        ui.circle(
            k(base + 2 + index as u16 * 2),
            Rect::new(chip_x, chip_y, chip_size, chip_size),
            z + 2,
            Color::rgba(255, 255, 255, 64),
        );
        ui.icon(
            k(base + 3 + index as u16 * 2),
            Rect::new(chip_x + 3, chip_y + 3, chip_size - 6, chip_size - 6),
            z + 3,
            vector_icon(chip_icons[index]),
            Color::rgba(255, 255, 255, 235),
        );
    }

    let label_y = main.y + main.h as i32 + if compact { 5 } else { 10 };
    ui.text(
        k(base + 10),
        gallery_x,
        label_y,
        z + 2,
        "TODAY",
        Color::rgba(18, 32, 58, 218),
        1,
    );

    let thumb_gap = 6u16;
    let thumb_w = gallery_w.saturating_sub(thumb_gap.saturating_mul(2)) / 3;
    let thumb_h = if compact { 18 } else { 34 };
    let thumb_y = label_y + if compact { 16 } else { 20 };
    for index in 0..3 {
        let thumb_x = gallery_x + index as i32 * (thumb_w + thumb_gap) as i32;
        let thumb = Rect::new(thumb_x, thumb_y, thumb_w, thumb_h);
        ui.round_rect(
            k(base + 11 + index as u16 * 2),
            Rect::new(
                thumb.x - 1,
                thumb.y - 1,
                thumb.w.saturating_add(2),
                thumb.h.saturating_add(2),
            ),
            z + 1,
            6,
            Color::rgba(10, 22, 42, 46),
        );
        ui.image(
            k(base + 12 + index as u16 * 2),
            thumb,
            z + 2,
            theme_wallpaper(theme, width, height),
            Color::rgba(255, 255, 255, 230),
        );
    }
}

fn compose_preview_bottom_controls(
    ui: &mut UiLayer<'_>,
    width: u16,
    height: u16,
    theme: ThemeKind,
) {
    let cy = height as i32 - if height < 560 { 96 } else { 108 };
    let cx = width as i32 / 2;
    switcher_circle_button(
        ui,
        610,
        Rect::new(cx - 132, cy - 25, 50, 50),
        ButtonAction::OpenSettings,
        theme,
        ThemedIcon::Settings,
    );
    switcher_circle_button(
        ui,
        616,
        Rect::new(cx - 25, cy - 31, 62, 62),
        ButtonAction::BackHome,
        theme,
        ThemedIcon::Close,
    );
    switcher_circle_button(
        ui,
        622,
        Rect::new(cx + 88, cy - 25, 50, 50),
        ButtonAction::LaunchTerminal,
        theme,
        ThemedIcon::Terminal,
    );
}

fn switcher_circle_button(
    ui: &mut UiLayer<'_>,
    base: u16,
    rect: Rect,
    action: ButtonAction,
    _theme: ThemeKind,
    icon: ThemedIcon,
) {
    ui.button(
        k(base),
        rect,
        24,
        action,
        Visual::Circle {
            color: Color::rgba(255, 255, 255, 54),
        },
    );
    ui.icon(
        k(base + 1),
        Rect::new(rect.x + rect.w as i32 / 2 - 14, rect.y + rect.h as i32 / 2 - 14, 28, 28),
        25,
        vector_icon(icon),
        Color::rgba(255, 255, 255, 230),
    );
}

fn compose_soccer_preview(
    ui: &mut UiLayer<'_>,
    width: u16,
    height: u16,
    _theme: ThemeKind,
    apps: &[AppManifest],
) {
    let cx = width as i32 / 2;
    let cy = height as i32 / 2 + 28;
    let colors = [
        Color::rgba(255, 255, 255, 230),
        Color::rgba(35, 39, 54, 240),
        Color::rgba(255, 255, 255, 225),
        Color::rgba(26, 29, 42, 240),
        Color::rgba(255, 255, 255, 215),
    ];
    let rects = [
        Rect::new(cx - 42, cy - 72, 84, 84),
        Rect::new(cx - 90, cy - 22, 76, 76),
        Rect::new(cx + 14, cy - 22, 76, 76),
        Rect::new(cx - 42, cy + 30, 84, 84),
        Rect::new(cx - 28, cy - 18, 56, 56),
    ];

    for index in 0..rects.len() {
        let rect = rects[index];
        let color = colors[index];
        let base = 430 + index as u16 * 8;
        let z = 15 + index as i16;

        if index < apps.len() && index < PREVIEW_ITEM_LIMIT {
            let app = apps[index];
            ui.button(
                k(base),
                rect,
                z,
                ButtonAction::LaunchApp(app.id),
                Visual::Circle { color },
            );
            ui.icon(
                k(base + 1),
                Rect::new(rect.x + rect.w as i32 / 2 - 13, rect.y + 16, 26, 26),
                z + 8,
                app_icon(app),
                Color::rgba(255, 255, 255, 230),
            );
            ui.text(
                k(base + 2),
                rect.x + 12,
                rect.y + rect.h as i32 - 24,
                z + 8,
                app.name,
                Color::rgba(255, 255, 255, 220),
                1,
            );
        } else {
            ui.circle(k(base), rect, z, color);
        }
    }

    ui.text(
        k(475),
        44,
        height as i32 - 132,
        24,
        "SOCCER",
        Color::rgba(255, 255, 255, 190),
        1,
    );
}

fn compose_cube_preview(
    ui: &mut UiLayer<'_>,
    width: u16,
    height: u16,
    _theme: ThemeKind,
    apps: &[AppManifest],
) {
    let cx = width as i32 / 2;
    let cy = height as i32 / 2 + 8;
    let item_count = apps.len().min(4) as u8;

    ui.effect(
        k(500),
        Rect::new(cx - 92, cy - 102, 184, 184),
        14,
        EffectKind::PreviewCube,
        EffectParams::new(0, item_count, 0),
    );

    let face_rects = [
        Rect::new(cx - 54, cy - 52, 108, 108),
        Rect::new(cx + 46, cy - 28, 74, 74),
        Rect::new(cx - 120, cy - 28, 74, 74),
        Rect::new(cx - 38, cy + 62, 76, 76),
    ];

    for index in 0..apps.len().min(face_rects.len()) {
        let app = apps[index];
        let rect = face_rects[index];
        let base = 510 + index as u16 * 8;
        let z = 24 + index as i16;

        ui.button(
            k(base),
            rect,
            z,
            ButtonAction::LaunchApp(app.id),
            Visual::Rect {
                color: Color::TRANSPARENT,
            },
        );
        ui.icon(
            k(base + 1),
            Rect::new(rect.x + rect.w as i32 / 2 - 15, rect.y + 18, 30, 30),
            z + 1,
            app_icon(app),
            Color::rgba(255, 255, 255, 230),
        );
        ui.text(
            k(base + 2),
            rect.x + 10,
            rect.y + rect.h as i32 - 24,
            z + 1,
            app.name,
            Color::rgba(255, 255, 255, 220),
            1,
        );
    }

    ui.text(
        k(545),
        44,
        height as i32 - 132,
        24,
        "CUBE",
        Color::rgba(255, 255, 255, 190),
        1,
    );
}

fn app_icon(app: AppManifest) -> VectorIcon {
    vector_icon(app_themed_icon(app))
}

fn app_themed_icon(app: AppManifest) -> ThemedIcon {
    match app.icon {
        16 => ThemedIcon::Settings,
        17 => ThemedIcon::Terminal,
        18 => ThemedIcon::Alert,
        19 => ThemedIcon::Play,
        20 => ThemedIcon::System,
        21 => ThemedIcon::Surface,
        _ => ThemedIcon::Wing,
    }
}

fn app_subtitle(app: AppManifest) -> &'static str {
    match app.id.0 {
        1 => "CONFIG SURFACE",
        2 => "RUNTIME DIAG",
        16 => "WING TTY",
        17 => "SURFACE LAB",
        _ => "WING APP",
    }
}

const fn vector_icon(icon: ThemedIcon) -> VectorIcon {
    match icon {
        ThemedIcon::Wifi => VectorIcon::Wifi,
        ThemedIcon::Bluetooth => VectorIcon::Bluetooth,
        ThemedIcon::Phone => VectorIcon::Phone,
        ThemedIcon::Chat => VectorIcon::Chat,
        ThemedIcon::Settings => VectorIcon::Settings,
        ThemedIcon::Camera => VectorIcon::Camera,
        ThemedIcon::Flashlight => VectorIcon::Flashlight,
        ThemedIcon::Airplane => VectorIcon::Airplane,
        ThemedIcon::Moon => VectorIcon::Moon,
        ThemedIcon::Sync => VectorIcon::Sync,
        ThemedIcon::Mail => VectorIcon::Mail,
        ThemedIcon::Cloud => VectorIcon::Cloud,
        ThemedIcon::Folder => VectorIcon::Folder,
        ThemedIcon::Music => VectorIcon::Music,
        ThemedIcon::Play => VectorIcon::Play,
        ThemedIcon::Wing => VectorIcon::Wing,
        ThemedIcon::Check => VectorIcon::Check,
        ThemedIcon::Close => VectorIcon::Close,
        ThemedIcon::Alert => VectorIcon::Alert,
        ThemedIcon::More => VectorIcon::More,
        ThemedIcon::System => VectorIcon::System,
        ThemedIcon::Terminal => VectorIcon::Terminal,
        ThemedIcon::Surface => VectorIcon::Surface,
    }
}

fn resolve_preview_effect(
    requested: PreviewEffect,
    capabilities: RendererCapabilities,
) -> PreviewEffect {
    if supports_preview_effect(requested, capabilities) {
        return requested;
    }

    if supports_preview_effect(PreviewEffect::Soccer, capabilities) {
        PreviewEffect::Soccer
    } else {
        PreviewEffect::Cards
    }
}

fn next_preview_effect(
    current: PreviewEffect,
    capabilities: RendererCapabilities,
) -> PreviewEffect {
    let mut next = match current {
        PreviewEffect::Cards => PreviewEffect::Soccer,
        PreviewEffect::Soccer => PreviewEffect::Cube,
        PreviewEffect::Cube => PreviewEffect::Cards,
    };

    for _ in 0..3 {
        if supports_preview_effect(next, capabilities) {
            return next;
        }
        next = match next {
            PreviewEffect::Cards => PreviewEffect::Soccer,
            PreviewEffect::Soccer => PreviewEffect::Cube,
            PreviewEffect::Cube => PreviewEffect::Cards,
        };
    }

    PreviewEffect::Cards
}

fn supports_preview_effect(effect: PreviewEffect, capabilities: RendererCapabilities) -> bool {
    match effect {
        PreviewEffect::Cards => capabilities.supports_basic_2d(),
        PreviewEffect::Soccer => capabilities.supports_basic_2d() && capabilities.circle,
        PreviewEffect::Cube => {
            capabilities.supports_basic_2d() && capabilities.supports_cube_preview()
        }
    }
}

fn preview_effect_label(effect: PreviewEffect) -> &'static str {
    match effect {
        PreviewEffect::Cards => "CARDS",
        PreviewEffect::Soccer => "SOCCER",
        PreviewEffect::Cube => "CUBE",
    }
}

fn renderer_kind_label(kind: RendererKind) -> &'static str {
    match kind {
        RendererKind::Software => "SOFTWARE",
        RendererKind::Gpu2d => "GPU2D",
        RendererKind::Gles => "GLES",
        RendererKind::Custom => "CUSTOM",
    }
}

fn renderer_caps_label(capabilities: RendererCapabilities) -> &'static str {
    if capabilities.supports_effects() {
        "EFFECT"
    } else if capabilities.supports_basic_2d() {
        "BASIC 2D"
    } else {
        "LIMITED"
    }
}

fn task_caps_label(capabilities: PlatformTaskCapabilities) -> &'static str {
    if capabilities.custom_priority || capabilities.custom_stack || capabilities.wing_managed_surface
    {
        "CUSTOM"
    } else if capabilities.inherit_stdio
        && capabilities.builtin_default_priority
        && capabilities.builtin_default_stack
        && capabilities.detached_surface
    {
        "BUILTIN"
    } else {
        "LIMITED"
    }
}

fn surface_caps_label(capabilities: SurfaceCapabilities) -> &'static str {
    if capabilities.stream_texture {
        "STREAM"
    } else if capabilities.shared_memory {
        "SHARED"
    } else if capabilities.max_surfaces > 0 {
        "RESERVED"
    } else {
        "NONE"
    }
}

fn settings_source_label(source: SettingsStorageSource) -> &'static str {
    match source {
        SettingsStorageSource::DataFile => "DATA",
        SettingsStorageSource::TempFile => "TMP",
        SettingsStorageSource::Memory => "RAM",
    }
}

fn texture_source_label(source: TextureResourceSource) -> &'static str {
    match source {
        TextureResourceSource::Builtin => "BUILTIN",
        TextureResourceSource::External => "ROMFS",
        TextureResourceSource::RuntimeFilesystem => "FS",
        TextureResourceSource::BuiltinFallback => "FALLBACK",
    }
}

fn background_resource_label(
    texture_source: TextureResourceSource,
    shell_wallpapers_ready: bool,
    summary: RuntimeResourceSummary,
) -> &'static str {
    if texture_source == TextureResourceSource::RuntimeFilesystem
        && shell_wallpapers_ready
        && summary.shell_wallpaper_files_ready()
    {
        "PNG DECODE"
    } else if shell_wallpapers_ready && summary.shell_wallpaper_files_ready() {
        "PNG RGB565"
    } else if shell_wallpapers_ready {
        match texture_source {
            TextureResourceSource::Builtin => "BUILTIN",
            TextureResourceSource::External => "PACKED",
            TextureResourceSource::RuntimeFilesystem => "PNG DECODE",
            TextureResourceSource::BuiltinFallback => "FALLBACK",
        }
    } else if summary.shell_wallpapers != 0 {
        "PNG FILE"
    } else {
        "GRADIENT"
    }
}

fn font_resource_label(
    summary: RuntimeResourceSummary,
    font_source: FontResourceSource,
) -> &'static str {
    if matches!(font_source, FontResourceSource::RuntimeTrueType) {
        "TTF DRAW"
    } else if summary.default_font_ready() {
        "TTF FILE"
    } else if summary.vector_fonts != 0 {
        "FONT FILE"
    } else {
        "PLACEHLD"
    }
}

fn glyph_cache_label(summary: FontCacheSummary) -> &'static str {
    match (summary.profile, glyph_cache_state(summary)) {
        (GlyphCacheProfile::Tiny, GlyphCacheState::Empty) => "TINY EMPTY",
        (GlyphCacheProfile::Tiny, GlyphCacheState::Warmup) => "TINY WARMUP",
        (GlyphCacheProfile::Tiny, GlyphCacheState::Warm) => "TINY WARM",
        (GlyphCacheProfile::Tiny, GlyphCacheState::Hot) => "TINY HOT",
        (GlyphCacheProfile::Tiny, GlyphCacheState::Full) => "TINY FULL",
        (GlyphCacheProfile::Tiny, GlyphCacheState::Evict) => "TINY EVICT",
        (GlyphCacheProfile::Balanced, GlyphCacheState::Empty) => "BASE EMPTY",
        (GlyphCacheProfile::Balanced, GlyphCacheState::Warmup) => "BASE WARMUP",
        (GlyphCacheProfile::Balanced, GlyphCacheState::Warm) => "BASE WARM",
        (GlyphCacheProfile::Balanced, GlyphCacheState::Hot) => "BASE HOT",
        (GlyphCacheProfile::Balanced, GlyphCacheState::Full) => "BASE FULL",
        (GlyphCacheProfile::Balanced, GlyphCacheState::Evict) => "BASE EVICT",
        (GlyphCacheProfile::Large, GlyphCacheState::Empty) => "LARGE EMPTY",
        (GlyphCacheProfile::Large, GlyphCacheState::Warmup) => "LARGE WARMUP",
        (GlyphCacheProfile::Large, GlyphCacheState::Warm) => "LARGE WARM",
        (GlyphCacheProfile::Large, GlyphCacheState::Hot) => "LARGE HOT",
        (GlyphCacheProfile::Large, GlyphCacheState::Full) => "LARGE FULL",
        (GlyphCacheProfile::Large, GlyphCacheState::Evict) => "LARGE EVICT",
        (GlyphCacheProfile::Custom, GlyphCacheState::Empty) => "CUSTOM EMPTY",
        (GlyphCacheProfile::Custom, GlyphCacheState::Warmup) => "CUSTOM WARMUP",
        (GlyphCacheProfile::Custom, GlyphCacheState::Warm) => "CUSTOM WARM",
        (GlyphCacheProfile::Custom, GlyphCacheState::Hot) => "CUSTOM HOT",
        (GlyphCacheProfile::Custom, GlyphCacheState::Full) => "CUSTOM FULL",
        (GlyphCacheProfile::Custom, GlyphCacheState::Evict) => "CUSTOM EVICT",
    }
}

fn glyph_cache_health_label(summary: FontCacheSummary) -> &'static str {
    match summary.pressure() {
        GlyphCachePressure::Cold => "NO SAMPLE",
        GlyphCachePressure::Thrashing => "THRASH",
        GlyphCachePressure::Tight => glyph_cache_tight_label(summary),
        GlyphCachePressure::Healthy => glyph_cache_hit_rate_label(summary),
    }
}

fn glyph_cache_tight_label(summary: FontCacheSummary) -> &'static str {
    if summary.bytes >= 64 * 1024 {
        "MEM 64K+"
    } else if summary.bytes >= 32 * 1024 {
        "MEM 32K+"
    } else if summary.fill_percent() >= 95 {
        "FILL 95+"
    } else {
        "TIGHT"
    }
}

fn glyph_cache_hit_rate_label(summary: FontCacheSummary) -> &'static str {
    match summary.hit_rate_percent() {
        0..=24 => "HIT <25",
        25..=49 => "HIT 25+",
        50..=74 => "HIT 50+",
        75..=89 => "HIT 75+",
        90..=97 => "HIT 90+",
        _ => "HIT 98+",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GlyphCacheState {
    Empty,
    Warmup,
    Warm,
    Hot,
    Full,
    Evict,
}

fn glyph_cache_state(summary: FontCacheSummary) -> GlyphCacheState {
    if summary.entries == 0 {
        GlyphCacheState::Empty
    } else if summary.requests() == 0 {
        GlyphCacheState::Warmup
    } else if summary.evictions != 0 {
        GlyphCacheState::Evict
    } else if summary.entries >= summary.capacity {
        GlyphCacheState::Full
    } else if summary.hits >= summary.misses.saturating_mul(2) {
        GlyphCacheState::Hot
    } else if summary.hits >= summary.misses {
        GlyphCacheState::Warm
    } else {
        GlyphCacheState::Warmup
    }
}

fn icon_resource_label(summary: RuntimeResourceSummary) -> &'static str {
    if summary.shell_svg_icons >= SHELL_VECTOR_ICON_COUNT {
        "SVG FS"
    } else if summary.shell_svg_icons != 0 {
        "SVG MIX"
    } else if summary.svg != 0 {
        "SVG FILE"
    } else {
        "SVG FALLBACK"
    }
}

fn svg_cache_label(summary: SvgCacheSummary) -> &'static str {
    match (summary.profile, svg_cache_state(summary)) {
        (SvgCacheProfile::Tiny, SvgCacheState::Empty) => "TINY EMPTY",
        (SvgCacheProfile::Tiny, SvgCacheState::Warmup) => "TINY WARMUP",
        (SvgCacheProfile::Tiny, SvgCacheState::Warm) => "TINY WARM",
        (SvgCacheProfile::Tiny, SvgCacheState::Hot) => "TINY HOT",
        (SvgCacheProfile::Tiny, SvgCacheState::Full) => "TINY FULL",
        (SvgCacheProfile::Tiny, SvgCacheState::Evict) => "TINY EVICT",
        (SvgCacheProfile::Balanced, SvgCacheState::Empty) => "BASE EMPTY",
        (SvgCacheProfile::Balanced, SvgCacheState::Warmup) => "BASE WARMUP",
        (SvgCacheProfile::Balanced, SvgCacheState::Warm) => "BASE WARM",
        (SvgCacheProfile::Balanced, SvgCacheState::Hot) => "BASE HOT",
        (SvgCacheProfile::Balanced, SvgCacheState::Full) => "BASE FULL",
        (SvgCacheProfile::Balanced, SvgCacheState::Evict) => "BASE EVICT",
        (SvgCacheProfile::Large, SvgCacheState::Empty) => "LARGE EMPTY",
        (SvgCacheProfile::Large, SvgCacheState::Warmup) => "LARGE WARMUP",
        (SvgCacheProfile::Large, SvgCacheState::Warm) => "LARGE WARM",
        (SvgCacheProfile::Large, SvgCacheState::Hot) => "LARGE HOT",
        (SvgCacheProfile::Large, SvgCacheState::Full) => "LARGE FULL",
        (SvgCacheProfile::Large, SvgCacheState::Evict) => "LARGE EVICT",
        (SvgCacheProfile::Custom, SvgCacheState::Empty) => "CUSTOM EMPTY",
        (SvgCacheProfile::Custom, SvgCacheState::Warmup) => "CUSTOM WARMUP",
        (SvgCacheProfile::Custom, SvgCacheState::Warm) => "CUSTOM WARM",
        (SvgCacheProfile::Custom, SvgCacheState::Hot) => "CUSTOM HOT",
        (SvgCacheProfile::Custom, SvgCacheState::Full) => "CUSTOM FULL",
        (SvgCacheProfile::Custom, SvgCacheState::Evict) => "CUSTOM EVICT",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SvgCacheState {
    Empty,
    Warmup,
    Warm,
    Hot,
    Full,
    Evict,
}

fn svg_cache_state(summary: SvgCacheSummary) -> SvgCacheState {
    match summary.pressure() {
        SvgCachePressure::Cold if summary.entries == 0 => SvgCacheState::Empty,
        SvgCachePressure::Cold => SvgCacheState::Warmup,
        SvgCachePressure::Thrashing => SvgCacheState::Evict,
        SvgCachePressure::Tight if summary.evictions != 0 => SvgCacheState::Evict,
        SvgCachePressure::Tight => SvgCacheState::Full,
        SvgCachePressure::Healthy if summary.hits >= summary.misses.saturating_mul(2) => {
            SvgCacheState::Hot
        }
        SvgCachePressure::Healthy if summary.hits >= summary.misses => SvgCacheState::Warm,
        SvgCachePressure::Healthy => SvgCacheState::Warmup,
    }
}

fn resource_files_label(summary: RuntimeResourceSummary) -> &'static str {
    if summary.missing != 0 {
        "MISSING"
    } else if summary.unknown != 0 {
        "UNKNOWN"
    } else if summary.png != 0 && summary.vector_fonts != 0 && summary.svg != 0 {
        "PNG+FONT+SVG"
    } else if summary.svg != 0 && summary.vector_fonts != 0 {
        "SVG+FONT"
    } else if summary.svg != 0 {
        "SVG"
    } else if summary.png != 0 && summary.vector_fonts != 0 {
        "PNG+FONT"
    } else if summary.png != 0 || summary.jpeg != 0 {
        "IMAGE"
    } else if summary.vector_fonts != 0 {
        "FONT"
    } else {
        "NONE"
    }
}

fn runtime_frame_label(diagnostics: RuntimeDiagnostics) -> &'static str {
    let flags = diagnostics.frames().last_flags();
    if flags.contains(RuntimeFrameFlags::TASK_EXIT) {
        "TASK"
    } else if flags.contains(RuntimeFrameFlags::SETTINGS_EVENT) {
        "SETTING"
    } else if flags.contains(RuntimeFrameFlags::SURFACE_EVENT) {
        "SURFACE"
    } else if flags.contains(RuntimeFrameFlags::INPUT) {
        "INPUT"
    } else if flags.contains(RuntimeFrameFlags::RENDERED) {
        "DRAW"
    } else if flags.contains(RuntimeFrameFlags::IDLE) {
        "IDLE"
    } else {
        "SKIP"
    }
}

fn runtime_health_label(diagnostics: RuntimeDiagnostics) -> &'static str {
    let render_plan = diagnostics.frames().last_render_plan();
    if !render_plan.fully_supported() {
        "DRAW CAP"
    } else if !render_plan.layers.background_supported() {
        "BG CAP"
    } else if matches!(
        render_plan.layers.background,
        RenderBackgroundKind::GradientMissingImage
    ) {
        "BG MISS"
    } else if render_plan.needs_software_fallback() {
        "DRAW SW"
    } else if render_plan.has_fragmented_runs() {
        "DRAW RUN"
    } else if diagnostics.frames().last_dirty_region().is_fragmented() {
        "DIRTY FRG"
    } else if diagnostics.capacity().current().any() {
        "FULL NOW"
    } else if diagnostics.capacity().latched().any() {
        "CAP FULL"
    } else {
        "OK"
    }
}

fn render_layer_label(summary: RenderLayerSummary) -> &'static str {
    if matches!(
        summary.background,
        RenderBackgroundKind::GradientMissingImage
    ) {
        return "BG MISS";
    }

    match summary.background_route {
        RenderBackgroundRoute::None => "NONE",
        RenderBackgroundRoute::NativePlane => "BG PLANE",
        RenderBackgroundRoute::Unsupported => "BG CAP",
        RenderBackgroundRoute::SoftwareFastPath => match summary.background {
            RenderBackgroundKind::Gradient => "BG GRAD",
            RenderBackgroundKind::GradientRgb565Image => "BG FAST",
            RenderBackgroundKind::GradientMissingImage | RenderBackgroundKind::None => "NONE",
        },
    }
}

fn dirty_region_label(summary: DirtyRegionSummary) -> &'static str {
    if summary.is_empty() {
        "IDLE"
    } else if summary.full {
        "FULL"
    } else if summary.is_fragmented() {
        "FRAG"
    } else if summary.tile_aligned {
        "TILED"
    } else if summary.is_broad() {
        "BROAD"
    } else if summary.rects >= 3 {
        "MULTI"
    } else {
        "TIGHT"
    }
}

fn draw_task_label(summary: DrawTaskSummary) -> &'static str {
    if summary.total == 0 {
        "IDLE"
    } else if summary.count(DrawTaskKind::Effect) != 0 {
        "EFFECT"
    } else if summary.count(DrawTaskKind::Surface) != 0 {
        "SURFACE"
    } else if summary.count(DrawTaskKind::Shadow) != 0 {
        "SHADOW"
    } else if summary.count(DrawTaskKind::Image) != 0 {
        "IMAGE"
    } else if summary.count(DrawTaskKind::Vector) != 0 {
        "VECTOR"
    } else if summary.count(DrawTaskKind::Text) != 0 {
        "TEXT"
    } else if summary.count(DrawTaskKind::Fill) != 0 {
        "FILL"
    } else {
        "CLEAR"
    }
}

fn system_row_height(available: u16, count: u16, gap: u16) -> u16 {
    if count == 0 {
        return 0;
    }

    let gaps = count.saturating_sub(1).saturating_mul(gap);
    if available <= gaps {
        return 14;
    }

    ((available - gaps) / count).clamp(12, 30)
}

fn system_row(
    ui: &mut UiLayer<'_>,
    base: u16,
    rect: Rect,
    tokens: ShellStyleTokens,
    title: &'static str,
    value: &'static str,
) {
    let text_y = rect.y + ((rect.h.saturating_sub(8)) / 2) as i32;
    let text_key = ui.styled_round_rect(
        k(base),
        rect,
        8,
        tokens.system_row,
    );
    ui.text(
        text_key,
        rect.x + 14,
        text_y,
        10,
        title,
        Color::WHITE,
        1,
    );
    ui.text(
        text_key.offset(1),
        rect.x + rect.w as i32 - 90,
        text_y,
        10,
        value,
        Color::rgba(255, 255, 255, 210),
        1,
    );
}

fn build_draw_list_system<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>) {
    rt.draw_list.clear();
    let (top, bottom) = theme_gradient(rt.shell.theme);
    rt.draw_list.push(DrawCmd::ClearGradient { top, bottom });
    rt.draw_list.push(DrawCmd::Image {
        rect: Rect::new(0, 0, rt.width, rt.height),
        image: theme_wallpaper(rt.shell.theme, rt.width, rt.height),
        tint: Color::rgba(255, 255, 255, wallpaper_alpha(rt.shell.brightness)),
    });

    queue_focused_surface(rt);
    queue_visible_visuals_in_order(rt);
}

fn queue_focused_surface<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>) {
    if !should_show_focused_surface(rt.shell.mode, rt.shell.return_to) {
        return;
    }

    let Some(id) = rt.app_manager.focused_surface() else {
        return;
    };
    let Some(slot) = rt.surfaces.get(id) else {
        return;
    };
    if !slot.visible {
        return;
    }

    let descriptor = slot.descriptor;
    let Some(frame) = rt
        .platform
        .resolve_surface_frame(descriptor.transport.frame, descriptor.transport.token)
    else {
        return;
    };

    rt.draw_list.push(DrawCmd::Surface {
        rect: external_surface_rect(rt.width, rt.height),
        frame,
        alpha: 255,
    });
}

fn should_show_focused_surface(mode: ShellMode, return_to: ShellMode) -> bool {
    mode == ShellMode::ExternalApp
        || (return_to == ShellMode::ExternalApp
            && matches!(mode, ShellMode::Notification | ShellMode::AppSwitcher))
}

fn queue_visible_visuals_in_order<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>) {
    let mut last: Option<(i16, usize)> = None;

    loop {
        let mut next: Option<(i16, usize, Entity)> = None;

        for index in 0..rt.world.visuals.len() {
            let entity = rt.world.visuals.entity_at(index);
            if !is_entity_visible(rt, entity) {
                continue;
            }

            let z = rt.world.transforms.get(entity).map(|t| t.z).unwrap_or(0);
            if !is_after_last(last, z, index) {
                continue;
            }
            if should_replace_next(next, z, index) {
                next = Some((z, index, entity));
            }
        }

        let Some((z, index, entity)) = next else {
            break;
        };

        queue_entity_visual(rt, entity);
        last = Some((z, index));
    }
}

fn is_after_last(last: Option<(i16, usize)>, z: i16, index: usize) -> bool {
    match last {
        Some((last_z, last_index)) => z > last_z || (z == last_z && index > last_index),
        None => true,
    }
}

fn should_replace_next(next: Option<(i16, usize, Entity)>, z: i16, index: usize) -> bool {
    match next {
        Some((next_z, next_index, _)) => z < next_z || (z == next_z && index < next_index),
        None => true,
    }
}

fn queue_entity_visual<P: Platform, R: RendererBackend>(
    rt: &mut WingRuntime<P, R>,
    entity: Entity,
) {
    let Some(layout) = rt.world.layouts.get(entity).copied() else {
        return;
    };
    let Some(visual) = rt.world.visuals.get(entity).copied() else {
        return;
    };
    let alpha = rt
        .world
        .visibility
        .get(entity)
        .map(|visibility| visibility.alpha)
        .unwrap_or(255);
    let clip = rt.world.clips.get(entity).copied();
    queue_visual(rt, layout, visual, alpha, clip);
}

fn hit_button<P: Platform, R: RendererBackend>(
    rt: &WingRuntime<P, R>,
    point: Point,
) -> Option<ButtonAction> {
    let mut best: Option<(i16, ButtonAction)> = None;
    for index in 0..rt.world.buttons.len() {
        let entity = rt.world.buttons.entity_at(index);
        if !is_entity_visible(rt, entity) {
            continue;
        }
        if rt
            .world
            .hitboxes
            .get(entity)
            .map(|hitbox| hitbox.enabled)
            != Some(true)
        {
            continue;
        }
        let Some(layout) = rt.world.layouts.get(entity).copied() else {
            continue;
        };
        if !layout.rect.contains(point) {
            continue;
        }
        if rt
            .world
            .clips
            .get(entity)
            .map(|clip| !clip.contains(point))
            .unwrap_or(false)
        {
            continue;
        }
        let z = rt.world.transforms.get(entity).map(|t| t.z).unwrap_or(0);
        let action = rt.world.buttons.value_at(index).action;
        if best.map(|(best_z, _)| z >= best_z).unwrap_or(true) {
            best = Some((z, action));
        }
    }
    best.map(|(_, action)| action)
}

fn run_action<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>, action: ButtonAction) {
    match action {
        ButtonAction::OpenSettings => launch_app(rt, AppId(1)),
        ButtonAction::OpenNotification => {
            rt.shell.return_to = rt.shell.mode;
            enter_mode(rt, ShellMode::Notification, SlideDirection::FromTop);
        }
        ButtonAction::OpenAppSwitcher => {
            rt.shell.return_to = rt.shell.mode;
            enter_mode(rt, ShellMode::AppSwitcher, SlideDirection::FromBottom);
        }
        ButtonAction::BackHome => return_home(rt),
        ButtonAction::ToggleTheme => {
            rt.shell.theme = match rt.shell.theme {
                ThemeKind::Aurora => ThemeKind::Dusk,
                ThemeKind::Dusk => ThemeKind::Aurora,
            };
        }
        ButtonAction::TogglePreviewEffect => {
            rt.shell.preview_effect =
                next_preview_effect(rt.shell.preview_effect, rt.renderer_capabilities());
        }
        ButtonAction::ToggleWifi => {
            rt.shell.quick.wifi_enabled = !rt.shell.quick.wifi_enabled;
            if rt.shell.quick.wifi_enabled {
                rt.shell.quick.airplane_mode = false;
            }
        }
        ButtonAction::ToggleBluetooth => {
            rt.shell.quick.bluetooth_enabled = !rt.shell.quick.bluetooth_enabled;
            if rt.shell.quick.bluetooth_enabled {
                rt.shell.quick.airplane_mode = false;
            }
        }
        ButtonAction::ToggleAirplane => {
            rt.shell.quick.airplane_mode = !rt.shell.quick.airplane_mode;
        }
        ButtonAction::ToggleDnd => {
            rt.shell.quick.dnd_enabled = !rt.shell.quick.dnd_enabled;
        }
        ButtonAction::ToggleLight => {
            rt.shell.quick.light_enabled = !rt.shell.quick.light_enabled;
        }
        ButtonAction::ToggleSync => {
            rt.shell.quick.sync_enabled = !rt.shell.quick.sync_enabled;
        }
        ButtonAction::LaunchTerminal => launch_app(rt, AppId(16)),
        ButtonAction::LaunchApp(id) => launch_app(rt, id),
    }
}

fn launch_app<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>, id: AppId) {
    let mut from = rt.shell.mode;
    if rt.app_manager.has_active_task() {
        rt.close_focused_app();
        if from == ShellMode::ExternalApp {
            from = ShellMode::Home;
        }
    }

    let Some(request) = rt.app_manager.launch(&rt.app_registry, id) else {
        return;
    };

    rt.shell.return_to = from;
    match request {
        AppLaunchRequest::Builtin(BuiltinAppId::Settings) => {
            enter_mode(rt, ShellMode::Settings, SlideDirection::FromRight);
        }
        AppLaunchRequest::Builtin(BuiltinAppId::SystemInfo) => {
            enter_mode(rt, ShellMode::SystemInfo, SlideDirection::FromRight);
        }
        AppLaunchRequest::PlatformTask(task) => {
            if matches!(task.surface, TaskSurface::WingManaged(_)) {
                enter_mode(rt, ShellMode::ExternalApp, SlideDirection::FromRight);
            } else {
                enter_mode(rt, ShellMode::AppSwitcher, SlideDirection::FromBottom);
            }
        }
    }
}

fn return_home<P: Platform, R: RendererBackend>(rt: &mut WingRuntime<P, R>) {
    if rt.app_manager.has_active_task() {
        rt.close_focused_app();
    }
    enter_mode(rt, ShellMode::Home, SlideDirection::FromLeft);
}

fn enter_mode<P: Platform, R: RendererBackend>(
    rt: &mut WingRuntime<P, R>,
    mode: ShellMode,
    transition: SlideDirection,
) {
    if rt.shell.mode == mode {
        return;
    }

    rt.shell.mode = mode;
    rt.shell.transition = if rt.shell.reduce_motion {
        SlideTransition::none()
    } else {
        SlideTransition::enter(transition)
    };
}

fn return_transition(mode: ShellMode) -> SlideDirection {
    match mode {
        ShellMode::Notification => SlideDirection::FromBottom,
        ShellMode::AppSwitcher => SlideDirection::FromTop,
        ShellMode::ExternalApp => SlideDirection::FromLeft,
        ShellMode::Settings => SlideDirection::FromLeft,
        ShellMode::SystemInfo => SlideDirection::FromLeft,
        ShellMode::Home => SlideDirection::None,
    }
}

fn mode_layer(mode: ShellMode) -> Layer {
    match mode {
        ShellMode::Home => Layer::Home,
        ShellMode::Notification => Layer::Notification,
        ShellMode::AppSwitcher => Layer::AppSwitcher,
        ShellMode::ExternalApp => Layer::ExternalApp,
        ShellMode::Settings => Layer::Settings,
        ShellMode::SystemInfo => Layer::SystemInfo,
    }
}

fn is_entity_visible<P: Platform, R: RendererBackend>(
    rt: &WingRuntime<P, R>,
    entity: Entity,
) -> bool {
    if rt
        .world
        .visibility
        .get(entity)
        .map(|visibility| !visibility.visible || visibility.alpha == 0)
        .unwrap_or(false)
    {
        return false;
    }

    match rt.world.layers.get(entity).copied().unwrap_or(Layer::Home) {
        Layer::Always => true,
        Layer::Home => rt.shell.mode == ShellMode::Home,
        Layer::Notification => rt.shell.mode == ShellMode::Notification,
        Layer::AppSwitcher => rt.shell.mode == ShellMode::AppSwitcher,
        Layer::ExternalApp => rt.shell.mode == ShellMode::ExternalApp,
        Layer::Settings => rt.shell.mode == ShellMode::Settings,
        Layer::SystemInfo => rt.shell.mode == ShellMode::SystemInfo,
    }
}

pub(crate) fn external_surface_rect(width: u16, height: u16) -> Rect {
    Rect::new(0, 0, width, height)
}

fn queue_visual<P: Platform, R: RendererBackend>(
    rt: &mut WingRuntime<P, R>,
    layout: Layout,
    visual: Visual,
    alpha: u8,
    clip: Option<Rect>,
) {
    match visual {
        Visual::Rect { color } => {
            let color = color.with_alpha(scale_alpha(color.a, alpha));
            if color.a != 0 {
                queue_draw_cmd(rt, clip, DrawCmd::FillRect {
                    rect: layout.rect,
                    color,
                });
            }
        }
        Visual::RoundRect { color, radius } => {
            let color = color.with_alpha(scale_alpha(color.a, alpha));
            if color.a != 0 {
                queue_draw_cmd(rt, clip, DrawCmd::FillRoundRect {
                    rect: layout.rect,
                    radius,
                    color,
                });
            }
        }
        Visual::RoundRectGradient {
            top,
            bottom,
            radius,
        } => {
            let top = top.with_alpha(scale_alpha(top.a, alpha));
            let bottom = bottom.with_alpha(scale_alpha(bottom.a, alpha));
            if top.a != 0 || bottom.a != 0 {
                queue_draw_cmd(rt, clip, DrawCmd::FillRoundRectGradient {
                    rect: layout.rect,
                    radius,
                    top,
                    bottom,
                });
            }
        }
        Visual::ShadowRoundRect {
            color,
            radius,
            offset_x,
            offset_y,
            blur,
            spread,
        } => {
            let color = color.with_alpha(scale_alpha(color.a, alpha));
            if color.a != 0 {
                queue_draw_cmd(rt, clip, DrawCmd::ShadowRoundRect {
                    rect: layout.rect,
                    radius,
                    color,
                    offset_x,
                    offset_y,
                    blur,
                    spread,
                });
            }
        }
        Visual::Circle { color } => {
            let color = color.with_alpha(scale_alpha(color.a, alpha));
            if color.a != 0 {
                queue_draw_cmd(rt, clip, DrawCmd::FillCircle {
                    rect: layout.rect,
                    color,
                });
            }
        }
        Visual::Image { image, tint } => {
            let tint = tint.with_alpha(scale_alpha(tint.a, alpha));
            if tint.a != 0 {
                queue_draw_cmd(rt, clip, DrawCmd::Image {
                    rect: layout.rect,
                    image,
                    tint,
                });
            }
        }
        Visual::Icon { icon, color } => {
            let color = color.with_alpha(scale_alpha(color.a, alpha));
            if color.a != 0 {
                queue_draw_cmd(rt, clip, DrawCmd::VectorIcon {
                    rect: layout.rect,
                    icon,
                    color,
                });
            }
        }
        Visual::Text { text, color, scale } => {
            let color = color.with_alpha(scale_alpha(color.a, alpha));
            if color.a != 0 {
                queue_draw_cmd(rt, clip, DrawCmd::Text {
                    x: layout.rect.x,
                    y: layout.rect.y,
                    text,
                    color,
                    scale,
                });
            }
        }
        Visual::Effect { effect, params } => {
            let params = params.with_alpha(scale_alpha(params.alpha, alpha));
            if params.alpha != 0 {
                queue_draw_cmd(rt, clip, DrawCmd::Effect {
                    rect: layout.rect,
                    effect,
                    params,
                });
            }
        }
    }
}

fn queue_draw_cmd<P: Platform, R: RendererBackend>(
    rt: &mut WingRuntime<P, R>,
    clip: Option<Rect>,
    cmd: DrawCmd,
) {
    if let Some(clip) = clip {
        if clip.is_empty() {
            return;
        }
        rt.draw_list.push_clipped(Some(clip), cmd);
    } else {
        rt.draw_list.push_clipped(None, cmd);
    }
}

fn shell_panel(ui: &mut UiLayer<'_>, key: UiKey, rect: Rect, tokens: ShellStyleTokens) {
    ui.styled_round_rect(
        key,
        rect,
        8,
        tokens.panel,
    );
}

fn back_button(
    ui: &mut UiLayer<'_>,
    base: u16,
    rect: Rect,
    z: i16,
    tokens: ShellStyleTokens,
) {
    let text_key = ui.button_styled_round_rect(
        k(base),
        rect,
        z,
        ButtonAction::BackHome,
        tokens.back_button,
    );
    ui.text(
        text_key,
        rect.x + 18,
        rect.y + 14,
        z + 2,
        "BACK",
        Color::WHITE,
        1,
    );
}

fn settings_row(
    ui: &mut UiLayer<'_>,
    base: u16,
    rect: Rect,
    action: ButtonAction,
    title: &'static str,
    tokens: ShellStyleTokens,
    value: &'static str,
) {
    let text_key = ui.button_styled_round_rect(
        k(base),
        rect,
        14,
        action,
        tokens.settings_row,
    );
    ui.text(
        text_key,
        rect.x + 14,
        rect.y + 17,
        16,
        title,
        Color::WHITE,
        2,
    );
    ui.text(
        text_key.offset(1),
        rect.x + rect.w as i32 - 90,
        rect.y + 20,
        16,
        value,
        Color::rgba(255, 255, 255, 210),
        1,
    );
}

fn theme_gradient(theme: ThemeKind) -> (Color, Color) {
    match theme {
        ThemeKind::Aurora => (Color::rgb(17, 28, 46), Color::rgb(16, 88, 96)),
        ThemeKind::Dusk => (Color::rgb(34, 25, 50), Color::rgb(112, 55, 70)),
    }
}

fn theme_wallpaper(theme: ThemeKind, width: u16, height: u16) -> ImageId {
    match (theme, width > height) {
        (ThemeKind::Aurora, true) => IMAGE_WALLPAPER_AURORA_LANDSCAPE,
        (ThemeKind::Aurora, false) => IMAGE_WALLPAPER_AURORA_PORTRAIT,
        (ThemeKind::Dusk, true) => IMAGE_WALLPAPER_DUSK_LANDSCAPE,
        (ThemeKind::Dusk, false) => IMAGE_WALLPAPER_DUSK_PORTRAIT,
    }
}

fn wallpaper_alpha(brightness: u8) -> u8 {
    120 + brightness / 2
}

fn scale_alpha(a: u8, factor: u8) -> u8 {
    ((a as u16 * factor as u16) / 255) as u8
}

const fn k(value: u16) -> UiKey {
    UiKey::new(value)
}
