use alloc::{boxed::Box, vec::Vec};

use crate::app::{
    AppLaunchRequest, AppManager, AppRegistry, AppRenderState, PlatformTaskCapabilities,
};
use crate::core::{Phase, Schedule, World};
use crate::diagnostics::{CapacitySnapshot, RuntimeDiagnostics, RuntimeFrameSnapshot};
use crate::input::InputState;
use crate::math::{Point, Rect, Size};
use crate::platform::Platform;
use crate::render::{
    register_bootstrap_svg_icons, warm_builtin_shell_svg_cache, DirtyRegion, DrawList, FontStore,
    ImageId, PixelFormat, RendererBackend, RendererCapabilities, RendererKind, SoftwareRenderer,
    SvgStore, Texture, TextureStore, VectorIcon,
    IMAGE_WALLPAPER_AURORA_LANDSCAPE, IMAGE_WALLPAPER_AURORA_PORTRAIT,
    IMAGE_WALLPAPER_DUSK_LANDSCAPE,
    IMAGE_WALLPAPER_DUSK_PORTRAIT,
};
use crate::resource_file::{RuntimeResourceId, RuntimeResourceSummary};
use crate::resource_image::{decode_png_rgb565_cover, png_runtime_decode_enabled};
use crate::settings::{SettingsEvent, SettingsKey, SettingsSnapshot, SettingsStorageSource};
use crate::shell::{external_surface_rect, warm_shell_font_cache, ShellMode, ShellState};
use crate::shell_notification::NotificationStore;
use crate::surface::{
    SurfaceCapabilities, SurfaceDescriptor, SurfaceError, SurfaceEvent, SurfaceHandle,
    SurfaceId, SurfaceInputPolicy, SurfacePointerEvent, SurfacePointerKind, SurfaceRequest,
    SurfaceTable,
};
use crate::ui::UiFrame;

const FRAME_DELAY_MS: u32 = 16;
const SURFACE_EVENT_DRAIN_LIMIT: usize = 8;
const SETTINGS_EVENT_DRAIN_LIMIT: usize = 8;
const TASK_EXIT_DRAIN_LIMIT: usize = 4;
const DIRTY_FULL_REDRAW_NUMERATOR: u64 = 3;
const DIRTY_FULL_REDRAW_DENOMINATOR: u64 = 4;
const DIRTY_COMPACT_TARGET_RECTS: usize = 4;
const DIRTY_COMPACT_EXTRA_AREA_PERCENT: u8 = 18;
const DIRTY_TILE_SIZE: u16 = 16;
const DIRTY_TILE_MAX_EXTRA_AREA_PERCENT: u8 = 28;
const WING_RUNTIME_SVG_ICON_CAPACITY: usize = 8 * 1024;

#[derive(Clone, Copy)]
struct ShellSvgIconResource {
    icon: VectorIcon,
    path: &'static [u8],
}

const SHELL_SVG_ICON_RESOURCES: &[ShellSvgIconResource] = &[
    ShellSvgIconResource {
        icon: VectorIcon::Wifi,
        path: b"/etc/wing/resource/icons/shell/wifi.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Bluetooth,
        path: b"/etc/wing/resource/icons/shell/bluetooth.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Phone,
        path: b"/etc/wing/resource/icons/shell/phone.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Chat,
        path: b"/etc/wing/resource/icons/shell/chat.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Settings,
        path: b"/etc/wing/resource/icons/shell/settings.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Camera,
        path: b"/etc/wing/resource/icons/shell/camera.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Flashlight,
        path: b"/etc/wing/resource/icons/shell/flashlight.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Airplane,
        path: b"/etc/wing/resource/icons/shell/airplane.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Moon,
        path: b"/etc/wing/resource/icons/shell/moon.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Sync,
        path: b"/etc/wing/resource/icons/shell/sync.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Mail,
        path: b"/etc/wing/resource/icons/shell/mail.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Cloud,
        path: b"/etc/wing/resource/icons/shell/cloud.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Folder,
        path: b"/etc/wing/resource/icons/shell/folder.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Music,
        path: b"/etc/wing/resource/icons/shell/music.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Play,
        path: b"/etc/wing/resource/icons/shell/play.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Wing,
        path: b"/etc/wing/resource/icons/shell/wing.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Check,
        path: b"/etc/wing/resource/icons/shell/check.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Close,
        path: b"/etc/wing/resource/icons/shell/close.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::Alert,
        path: b"/etc/wing/resource/icons/shell/alert.svg\0",
    },
    ShellSvgIconResource {
        icon: VectorIcon::More,
        path: b"/etc/wing/resource/icons/shell/more.svg\0",
    },
];

pub struct WingRuntime<P: Platform, R: RendererBackend = SoftwareRenderer> {
    pub world: World,
    pub app_registry: AppRegistry,
    pub app_manager: AppManager,
    pub shell: ShellState,
    pub notifications: NotificationStore,
    pub input: InputState,
    pub draw_list: DrawList,
    pub ui_frame: UiFrame,
    pub surfaces: SurfaceTable,
    pub textures: TextureStore,
    pub fonts: FontStore,
    pub svgs: SvgStore,
    pub renderer: R,
    pub schedule: Schedule<WingRuntime<P, R>>,
    pub diagnostics: RuntimeDiagnostics,
    pub runtime_resources: RuntimeResourceSummary,
    pub platform: P,
    pub width: u16,
    pub height: u16,
    frame_index: u32,
    dirty: DirtyRegion,
    last_presented: Option<PresentedState>,
    surface_pointer_capture: Option<SurfaceId>,
    persisted_settings: SettingsSnapshot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PresentedState {
    shell: ShellState,
    app: AppRenderState,
    notifications_revision: u32,
}

impl<P: Platform> WingRuntime<P, SoftwareRenderer> {
    pub fn new(platform: P) -> Self {
        let (width, height) = platform.dimensions();
        Self::new_with_renderer(platform, SoftwareRenderer::new(width, height))
    }
}

impl<P: Platform, R: RendererBackend> WingRuntime<P, R> {
    pub fn new_with_renderer(mut platform: P, renderer: R) -> Self {
        let (width, height) = platform.dimensions();
        let settings = platform
            .load_settings_snapshot()
            .unwrap_or_default()
            .sanitized();
        let resource_partition = platform.resource_partition();
        let mut runtime_resources = platform.inspect_runtime_resources();
        let mut textures = TextureStore::with_resource_partition(resource_partition);
        register_runtime_wallpapers(&mut platform, &mut textures, width, height);
        let mut fonts = FontStore::default();
        register_runtime_default_font(&mut platform, &mut fonts);
        let glyph_workset = platform.load_runtime_resource(RuntimeResourceId::ShellGlyphWorkset);
        let _ = warm_shell_font_cache(&fonts, height, glyph_workset.as_deref());
        let mut svgs = SvgStore::default();
        register_bootstrap_svg_icons(&mut svgs);
        let runtime_svg_icons = register_runtime_shell_svgs(&mut platform, &mut svgs);
        runtime_resources.include_shell_svg_icons(runtime_svg_icons);
        let _ = warm_builtin_shell_svg_cache(&svgs, height);
        Self {
            world: World::default(),
            app_registry: AppRegistry::with_builtin_apps(),
            app_manager: AppManager::default(),
            shell: shell_from_settings(settings),
            notifications: NotificationStore::with_shell_defaults(),
            input: InputState::default(),
            draw_list: DrawList::default(),
            ui_frame: UiFrame::default(),
            surfaces: SurfaceTable::default(),
            textures,
            fonts,
            svgs,
            renderer,
            schedule: Schedule::default(),
            diagnostics: RuntimeDiagnostics::default(),
            runtime_resources,
            platform,
            width,
            height,
            frame_index: 0,
            dirty: DirtyRegion::full(width, height),
            last_presented: None,
            surface_pointer_capture: None,
            persisted_settings: settings,
        }
    }

    pub fn run(&mut self) {
        while self.platform.is_running() {
            self.run_frame();
            self.platform.sleep_ms(FRAME_DELAY_MS);
        }
    }

    pub fn run_frame(&mut self) {
        let mut frame = RuntimeFrameSnapshot::default();
        self.input.begin_frame();
        self.platform.poll_input(&mut self.input);
        frame.input = self.input.has_activity();
        frame.task_exits = saturating_usize_to_u8(self.poll_task_exits());
        frame.surface_events = saturating_usize_to_u8(self.poll_surface_events());
        frame.settings_events = saturating_usize_to_u8(self.poll_settings_events());

        if !self.schedule.startup_done() {
            self.run_phase(Phase::Startup);
            self.schedule.mark_startup_done();
        }
        self.collect_capacity_diagnostics();

        self.run_phase(Phase::Input);
        self.dispatch_surface_input();
        self.run_phase(Phase::Update);
        self.persist_settings_if_changed();
        self.handle_app_launch_requests();

        let presented = self.presented_state();
        if !self.prepare_frame(presented) {
            self.finish_frame(frame);
            return;
        }

        self.run_phase(Phase::Layout);
        self.collect_capacity_diagnostics();
        if self.dirty.is_empty() {
            self.finish_frame(frame);
            return;
        }

        self.run_phase(Phase::RenderBuild);
        self.collect_capacity_diagnostics();
        self.prepare_dirty_region_for_present();

        frame.dirty = true;
        frame.rendered = true;
        frame.dirty_region = self
            .dirty
            .summary_with_tile(self.width, self.height, DIRTY_TILE_SIZE);
        let render_plan = self.renderer.render_plan(&self.draw_list, &self.textures);
        frame.render_plan = render_plan;
        self.renderer
            .draw(
                &self.draw_list,
                render_plan,
                &self.textures,
                &self.fonts,
                &self.svgs,
                self.dirty,
            );
        self.platform.present(self.renderer.framebuffer(), self.dirty);
        self.dirty.clear();
        self.finish_frame(frame);
    }

    pub fn frame_index(&self) -> u32 {
        self.frame_index
    }

    pub fn renderer_kind(&self) -> RendererKind {
        self.renderer.kind()
    }

    pub fn renderer_capabilities(&self) -> RendererCapabilities {
        self.renderer.capabilities()
    }

    pub fn task_capabilities(&self) -> PlatformTaskCapabilities {
        self.platform.task_capabilities()
    }

    pub fn surface_capabilities(&self) -> SurfaceCapabilities {
        self.platform.surface_capabilities()
    }

    pub fn settings_storage_source(&self) -> SettingsStorageSource {
        self.platform.settings_storage_source()
    }

    pub fn runtime_resource_summary(&self) -> RuntimeResourceSummary {
        self.runtime_resources
    }

    pub fn mark_all_dirty(&mut self) {
        self.dirty.set_full(self.width, self.height);
    }

    pub fn mark_dirty(&mut self, rect: Rect) {
        let viewport = Rect::new(0, 0, self.width, self.height);
        let Some(rect) = rect.intersect(viewport) else {
            return;
        };

        self.dirty.include_rect(rect);
        self.coalesce_dirty_if_large();
    }

    fn coalesce_dirty_if_large(&mut self) {
        let screen_area = self.width as u64 * self.height as u64;
        if screen_area == 0 {
            return;
        }

        if self
            .dirty
            .covered_area()
            .saturating_mul(DIRTY_FULL_REDRAW_DENOMINATOR)
            >= screen_area.saturating_mul(DIRTY_FULL_REDRAW_NUMERATOR)
        {
            self.dirty.set_full(self.width, self.height);
        }
    }

    fn prepare_dirty_region_for_present(&mut self) {
        if self.dirty.is_empty() || self.dirty.is_full_for(self.width, self.height) {
            return;
        }

        if self.dirty.align_to_tiles(
            self.width,
            self.height,
            DIRTY_TILE_SIZE,
            DIRTY_TILE_MAX_EXTRA_AREA_PERCENT,
        ) {
            self.coalesce_dirty_if_large();
        }

        if self
            .dirty
            .compact_to_budget(DIRTY_COMPACT_TARGET_RECTS, DIRTY_COMPACT_EXTRA_AREA_PERCENT)
        {
            self.coalesce_dirty_if_large();
        }
    }

    pub fn submit_surface_dirty(
        &mut self,
        handle: SurfaceHandle,
        token: u32,
        rect: Rect,
    ) -> Result<(), SurfaceError> {
        self.surfaces.submit_dirty(handle, token, rect)?;
        self.mark_all_dirty();
        Ok(())
    }

    pub fn close_focused_app(&mut self) {
        if let Some(active) = self.app_manager.take_active_task() {
            self.release_active_task(active);
        } else {
            self.app_manager.clear_focus();
        }

        self.mark_all_dirty();
    }

    fn poll_task_exits(&mut self) -> usize {
        let mut count = 0;
        for _ in 0..TASK_EXIT_DRAIN_LIMIT {
            let Some(exit) = self.platform.poll_task_exit() else {
                break;
            };

            count += 1;
            if self.notifications.notify_task_exit() {
                self.mark_all_dirty();
            }
            if let Some(active) = self.app_manager.handle_task_exit(exit) {
                self.finish_active_task(active);
            }
        }
        count
    }

    fn finish_active_task(&mut self, active: crate::app::ActiveAppTask) {
        self.release_active_task(active);
        if self.shell.return_to == ShellMode::ExternalApp {
            self.shell.return_to = ShellMode::Home;
        }
        if self.shell.mode == ShellMode::ExternalApp {
            self.shell.mode = ShellMode::Home;
            self.shell.return_to = ShellMode::Home;
        }
        self.mark_all_dirty();
    }

    fn release_active_task(&mut self, active: crate::app::ActiveAppTask) {
        self.platform.close_task(active.pid, active.surface);
        if let Some(surface) = active.surface {
            self.surfaces.destroy(surface.id);
        }
    }

    fn run_phase(&mut self, phase: Phase) {
        let count = self.schedule.system_count(phase);
        for index in 0..count {
            if let Some(system) = self.schedule.system_at(phase, index) {
                system(self);
            }
        }
    }

    fn poll_surface_events(&mut self) -> usize {
        let mut count = 0;
        for _ in 0..SURFACE_EVENT_DRAIN_LIMIT {
            let Some(event) = self.platform.poll_surface_event() else {
                break;
            };

            count += 1;
            match event {
                SurfaceEvent::Dirty(event) => {
                    let _ = self.submit_surface_dirty(event.handle, event.token, event.rect);
                }
                SurfaceEvent::Closed(event) => {
                    if let Some(active) = self
                        .app_manager
                        .handle_surface_closed(event.handle, event.token)
                    {
                        self.finish_active_task(active);
                    }
                }
            }
        }
        count
    }

    fn poll_settings_events(&mut self) -> usize {
        let mut count = 0;
        for _ in 0..SETTINGS_EVENT_DRAIN_LIMIT {
            let Some(event) = self.platform.poll_settings_event() else {
                break;
            };

            count += 1;
            self.apply_settings_event(event);
        }
        count
    }

    fn apply_settings_event(&mut self, event: SettingsEvent) {
        let previous = self.shell;

        match event.key {
            SettingsKey::Theme => {
                self.shell.theme = match event.value {
                    1 => crate::shell::ThemeKind::Dusk,
                    _ => crate::shell::ThemeKind::Aurora,
                };
            }
            SettingsKey::PreviewEffect => {
                self.shell.preview_effect = match event.value {
                    1 => crate::shell::PreviewEffect::Soccer,
                    2 => crate::shell::PreviewEffect::Cube,
                    _ => crate::shell::PreviewEffect::Cards,
                };
            }
            SettingsKey::Brightness => {
                self.shell.brightness = event.value.min(u8::MAX as u32) as u8;
            }
            SettingsKey::Haptic => {
                self.shell.haptic_enabled = event.value != 0;
            }
            SettingsKey::ReduceMotion => {
                self.shell.reduce_motion = event.value != 0;
                if self.shell.reduce_motion {
                    self.shell.transition = crate::animation::SlideTransition::none();
                }
            }
        }

        if self.shell != previous {
            self.mark_all_dirty();
        }
    }

    fn persist_settings_if_changed(&mut self) {
        let current = self.settings_snapshot();
        if current == self.persisted_settings {
            return;
        }

        if self.platform.save_settings_snapshot(current) {
            self.persisted_settings = current;
        }
    }

    fn dispatch_surface_input(&mut self) {
        if self.shell.mode != ShellMode::ExternalApp {
            self.cancel_surface_pointer_capture();
            return;
        }

        let Some(id) = self.app_manager.focused_surface() else {
            self.cancel_surface_pointer_capture();
            return;
        };

        if self.surface_pointer_capture.is_some() && self.surface_pointer_capture != Some(id) {
            self.surface_pointer_capture = None;
        }

        if self.input.primary.pressed {
            let rect = external_surface_rect(self.width, self.height);
            if rect.contains(self.input.pointer) && self.send_surface_pointer(id, SurfacePointerKind::Down) {
                self.surface_pointer_capture = Some(id);
            }
            return;
        }

        if self.input.primary.released {
            if let Some(captured) = self.surface_pointer_capture.take() {
                let _ = self.send_surface_pointer(captured, SurfacePointerKind::Up);
            }
            return;
        }

        if self.input.primary.down
            && self.surface_pointer_capture == Some(id)
            && self.input.pointer_delta != Point::default()
        {
            let _ = self.send_surface_pointer(id, SurfacePointerKind::Move);
        }
    }

    fn cancel_surface_pointer_capture(&mut self) {
        if let Some(captured) = self.surface_pointer_capture.take() {
            let _ = self.send_surface_pointer(captured, SurfacePointerKind::Cancel);
        }
    }

    fn send_surface_pointer(&mut self, id: SurfaceId, kind: SurfacePointerKind) -> bool {
        let Some(slot) = self.surfaces.get(id) else {
            return false;
        };

        let descriptor = slot.descriptor;
        if descriptor.request.input == SurfaceInputPolicy::None {
            return false;
        }

        let rect = external_surface_rect(self.width, self.height);
        let Some(point) = map_surface_pointer(rect, descriptor, self.input.pointer) else {
            return false;
        };

        let buttons = match kind {
            SurfacePointerKind::Down | SurfacePointerKind::Move => 1,
            SurfacePointerKind::Up | SurfacePointerKind::Cancel => 0,
        };

        self.platform.send_surface_input(SurfacePointerEvent::new(
            descriptor.transport.input,
            descriptor.transport.token,
            point,
            kind,
            buttons,
        ))
    }

    fn presented_state(&self) -> PresentedState {
        PresentedState {
            shell: self.shell,
            app: self.app_manager.render_state(),
            notifications_revision: self.notifications.revision(),
        }
    }

    fn prepare_frame(&mut self, current: PresentedState) -> bool {
        let previous = self.last_presented;
        if previous
            .map(|state| state.shell.theme != current.shell.theme)
            .unwrap_or(true)
        {
            self.mark_all_dirty();
        }

        let state_changed = previous != Some(current);
        if state_changed {
            self.last_presented = Some(current);
        }

        state_changed || !self.dirty.is_empty()
    }

    fn handle_app_launch_requests(&mut self) {
        let Some(request) = self.app_manager.take_launch_request() else {
            return;
        };

        match request {
            AppLaunchRequest::Builtin(_) => {}
            AppLaunchRequest::PlatformTask(task) => self.handle_platform_task_launch(task),
        }
    }

    fn handle_platform_task_launch(&mut self, task: crate::app::PlatformTask) {
        let surface = match self.prepare_task_surface(task) {
            Ok(surface) => surface,
            Err(error) => {
                self.app_manager.fail_launch(error);
                if self.notifications.notify_app_failed(error.label()) {
                    self.mark_all_dirty();
                }
                return;
            }
        };

        let settings = surface.map(|_| self.settings_snapshot());
        match self.platform.launch_task(task, surface, settings) {
            Ok(pid) => {
                self.app_manager.complete_launch(pid, surface);
                if self.notifications.notify_app_started() {
                    self.mark_all_dirty();
                }
            }
            Err(error) => {
                if let Some(descriptor) = surface {
                    self.surfaces.destroy(descriptor.id);
                }
                self.app_manager.fail_launch(error);
                if self.notifications.notify_app_failed(error.label()) {
                    self.mark_all_dirty();
                }
            }
        }
    }

    fn prepare_task_surface(
        &mut self,
        task: crate::app::PlatformTask,
    ) -> Result<Option<SurfaceDescriptor>, crate::app::TaskLaunchError> {
        self.task_capabilities().validate(task)?;

        match task.surface {
            crate::app::TaskSurface::Detached => Ok(None),
            crate::app::TaskSurface::WingManaged(request) => {
                let capabilities = self.surface_capabilities();
                let request = fit_managed_surface_request(
                    request,
                    external_surface_rect(self.width, self.height),
                    capabilities,
                );
                self.surfaces
                    .create(capabilities, request)
                    .map(Some)
                    .map_err(surface_error_to_launch_error)
            }
        }
    }

    fn settings_snapshot(&self) -> SettingsSnapshot {
        SettingsSnapshot::new(
            match self.shell.theme {
                crate::shell::ThemeKind::Aurora => 0,
                crate::shell::ThemeKind::Dusk => 1,
            },
            match self.shell.preview_effect {
                crate::shell::PreviewEffect::Cards => 0,
                crate::shell::PreviewEffect::Soccer => 1,
                crate::shell::PreviewEffect::Cube => 2,
            },
            self.shell.brightness,
            self.shell.haptic_enabled,
            self.shell.reduce_motion,
        )
    }

    fn collect_capacity_diagnostics(&mut self) {
        self.diagnostics.observe_capacity(
            self.frame_index,
            CapacitySnapshot {
                ui_frame: self.ui_frame.overflowed(),
                draw_list: self.draw_list.overflowed(),
                schedule: self.schedule.overflowed(),
                app_registry: self.app_registry.overflowed(),
                surface_table: self.surfaces.overflowed(),
                texture_store: self.textures.overflowed(),
            },
        );
    }

    fn finish_frame(&mut self, snapshot: RuntimeFrameSnapshot) {
        self.diagnostics.observe_frame(self.frame_index, snapshot);
        self.frame_index = self.frame_index.wrapping_add(1);
    }
}

fn register_runtime_wallpapers<P: Platform>(
    platform: &mut P,
    textures: &mut TextureStore,
    width: u16,
    height: u16,
) {
    if !png_runtime_decode_enabled() {
        return;
    }

    let short = width.min(height);
    let long = width.max(height);
    let wallpapers = [
        (
            RuntimeResourceId::AuroraPortrait,
            IMAGE_WALLPAPER_AURORA_PORTRAIT,
            short,
            long,
        ),
        (
            RuntimeResourceId::DuskPortrait,
            IMAGE_WALLPAPER_DUSK_PORTRAIT,
            short,
            long,
        ),
        (
            RuntimeResourceId::AuroraLandscape,
            IMAGE_WALLPAPER_AURORA_LANDSCAPE,
            long,
            short,
        ),
        (
            RuntimeResourceId::DuskLandscape,
            IMAGE_WALLPAPER_DUSK_LANDSCAPE,
            long,
            short,
        ),
    ];

    let mut decoded_wallpapers = Vec::new();
    for (resource, image, target_w, target_h) in wallpapers {
        let Some(bytes) = platform.load_runtime_resource(resource) else {
            return;
        };
        let Some(decoded) = decode_png_rgb565_cover(&bytes, target_w, target_h) else {
            return;
        };
        decoded_wallpapers.push((image, decoded));
    }

    for (image, decoded) in decoded_wallpapers {
        let data = Box::leak(decoded.data.into_boxed_slice());
        textures.register_runtime(Texture {
            id: ImageId(image.0),
            width: decoded.width,
            height: decoded.height,
            format: PixelFormat::Rgb565,
            data,
        });
    }
}

fn register_runtime_default_font<P: Platform>(platform: &mut P, fonts: &mut FontStore) {
    let Some(bytes) = platform.load_runtime_resource(RuntimeResourceId::DefaultFont) else {
        return;
    };
    let data = Box::leak(bytes.into_boxed_slice());
    let _ = fonts.register_default_ttf(data);
}

fn register_runtime_shell_svgs<P: Platform>(platform: &mut P, svgs: &mut SvgStore) -> usize {
    let mut loaded = 0usize;
    for resource in SHELL_SVG_ICON_RESOURCES {
        let Some(bytes) =
            platform.load_runtime_resource_file(resource.path, WING_RUNTIME_SVG_ICON_CAPACITY)
        else {
            continue;
        };

        if svgs.register_vector_icon(resource.icon, &bytes) {
            loaded = loaded.saturating_add(1);
        }
    }
    loaded
}

fn saturating_usize_to_u8(value: usize) -> u8 {
    value.min(u8::MAX as usize) as u8
}

fn shell_from_settings(settings: SettingsSnapshot) -> ShellState {
    let settings = settings.sanitized();
    let mut shell = ShellState::default();
    shell.theme = match settings.theme {
        1 => crate::shell::ThemeKind::Dusk,
        _ => crate::shell::ThemeKind::Aurora,
    };
    shell.preview_effect = match settings.preview_effect {
        1 => crate::shell::PreviewEffect::Soccer,
        2 => crate::shell::PreviewEffect::Cube,
        _ => crate::shell::PreviewEffect::Cards,
    };
    shell.brightness = settings.brightness;
    shell.haptic_enabled = settings.haptic_enabled;
    shell.reduce_motion = settings.reduce_motion;
    shell
}

fn map_surface_pointer(rect: Rect, descriptor: SurfaceDescriptor, point: Point) -> Option<Point> {
    if rect.w == 0
        || rect.h == 0
        || descriptor.request.size.w == 0
        || descriptor.request.size.h == 0
    {
        return None;
    }

    let local_x = (point.x - rect.x).clamp(0, rect.w as i32 - 1);
    let local_y = (point.y - rect.y).clamp(0, rect.h as i32 - 1);
    let x = local_x * descriptor.request.size.w as i32 / rect.w as i32;
    let y = local_y * descriptor.request.size.h as i32 / rect.h as i32;

    Some(Point::new(
        x.clamp(0, descriptor.request.size.w as i32 - 1),
        y.clamp(0, descriptor.request.size.h as i32 - 1),
    ))
}

fn fit_managed_surface_request(
    mut request: SurfaceRequest,
    rect: Rect,
    capabilities: SurfaceCapabilities,
) -> SurfaceRequest {
    let max_width = capabilities.max_width.max(1);
    let max_height = capabilities.max_height.max(1);
    let width = rect.w.max(1).min(max_width);
    let height = rect.h.max(1).min(max_height);
    request.size = Size::new(width, height);
    request
}

fn surface_error_to_launch_error(error: SurfaceError) -> crate::app::TaskLaunchError {
    match error {
        SurfaceError::CapacityFull => crate::app::TaskLaunchError::SurfaceCapacity,
        SurfaceError::InvalidGeometry | SurfaceError::InvalidHandle => {
            crate::app::TaskLaunchError::InvalidSurface
        }
        SurfaceError::InvalidToken => crate::app::TaskLaunchError::InvalidSurface,
        SurfaceError::UnsupportedFormat
        | SurfaceError::UnsupportedBuffering
        | SurfaceError::UnsupportedTransport
        | SurfaceError::UnsupportedInput => crate::app::TaskLaunchError::UnsupportedSurface,
    }
}
