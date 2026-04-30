use crate::{
    builder::UiBuilder,
    demo_all_apps::draw_all_apps,
    demo_cortana::draw_cortana,
    demo_detail::draw_tile_detail,
    demo_home::draw_home_handle,
    demo_launcher::draw_launcher,
    demo_lock::draw_lock_screen,
    demo_notifications::draw_notifications,
    demo_sample_app::draw_sample_app,
    demo_settings::draw_settings,
    demo_status::draw_status,
    demo_switcher::draw_app_switcher,
    demo_wallpaper::draw_wallpaper,
    node::UiHit,
    shell::{ShellAction, ShellActionQueue, ShellMode, ShellState},
    tree::UiTree,
};
use fhre::{Camera, InputEvent, InputQueue, Point, PointerEvent, Surface};
use fhre::{DirtyRegion, Rect};

pub struct WingDemoState {
    pub(crate) status: UiTree<24, 8>,
    pub(crate) launcher: UiTree<112, 24>,
    pub(crate) overlay: UiTree<96, 24>,
    pub(crate) status_builder: UiBuilder<16>,
    pub(crate) launcher_builder: UiBuilder<80>,
    pub(crate) overlay_builder: UiBuilder<96>,
    pub(crate) shell: ShellState,
}

impl WingDemoState {
    pub const fn new() -> Self {
        Self {
            status: UiTree::new(),
            launcher: UiTree::new(),
            overlay: UiTree::new(),
            status_builder: UiBuilder::new(),
            launcher_builder: UiBuilder::new(),
            overlay_builder: UiBuilder::new(),
            shell: ShellState::new(),
        }
    }

    pub fn reset(&mut self) {
        self.status.clear();
        self.launcher.clear();
        self.overlay.clear();
        self.status_builder.clear();
        self.launcher_builder.clear();
        self.overlay_builder.clear();
        self.shell = ShellState::new();
    }

    pub fn hit_launcher(&self, point: Point, width: u16, height: u16) -> Option<UiHit> {
        let camera = Camera::screen_canvas(width, height);
        self.launcher.hit_test(point, &camera)
    }

    pub const fn shell(&self) -> &ShellState {
        &self.shell
    }

    pub fn dispatch_pointer(&mut self, event: PointerEvent, width: u16, height: u16) -> Option<ShellAction> {
        let hit = self.hit_interactive(event.position, width, height);
        self.shell.dispatch_pointer(event, hit)
    }

    pub fn dispatch_input(&mut self, event: InputEvent, width: u16, height: u16) -> Option<ShellAction> {
        let hit = match event {
            InputEvent::Pointer(pointer) => self.hit_interactive(pointer.position, width, height),
            InputEvent::Key(_) => None,
        };
        self.shell.dispatch_input(event, hit)
    }

    fn hit_interactive(&self, point: Point, width: u16, height: u16) -> Option<UiHit> {
        let camera = Camera::screen_canvas(width, height);
        if self.shell.overlay_interactive() {
            if let Some(hit) = self.overlay.hit_test(point, &camera) {
                return Some(hit);
            }
        }
        if self.shell.launcher_interactive() {
            self.launcher.hit_test(point, &camera)
        } else {
            None
        }
    }

    pub fn dispatch_input_queue<const INPUT: usize, const ACTIONS: usize>(
        &mut self,
        input: &mut InputQueue<INPUT>,
        actions: &mut ShellActionQueue<ACTIONS>,
        width: u16,
        height: u16,
    ) -> usize {
        let mut handled = 0;
        while let Some(event) = input.pop() {
            if let Some(action) = self.dispatch_input(event, width, height) {
                actions.push(action);
            }
            handled += 1;
        }
        handled
    }

    pub fn collect_dirty_into<const N: usize>(&self, out: &mut DirtyRegion<N>, screen: Rect) {
        out.clear();
        if matches!(self.shell.mode, ShellMode::App(_) | ShellMode::LockScreen) {
            out.full_redraw(screen);
            return;
        }
        self.status.dirty().copy_into(out, screen);
        self.launcher.dirty().copy_into(out, screen);
        self.overlay.dirty().copy_into(out, screen);
    }
}

pub fn draw_demo(surface: &mut Surface, frame: u32) {
    static mut STATE: WingDemoState = WingDemoState::new();
    let state = unsafe {
        let state = core::ptr::addr_of_mut!(STATE);
        (*state).reset();
        &mut *state
    };
    draw_demo_with_state(surface, frame, state);
}

pub fn draw_demo_with_state(surface: &mut Surface, frame: u32, state: &mut WingDemoState) {
    if let ShellMode::App(app) = state.shell.mode {
        draw_sample_app(surface, frame, state, app);
        return;
    }
    if matches!(state.shell.mode, ShellMode::LockScreen) {
        draw_lock_screen(surface, frame, state);
        return;
    }

    draw_wallpaper(surface, frame);
    draw_status(surface, state);
    draw_launcher(surface, frame, state);
    match state.shell.mode {
        ShellMode::Home => draw_home_handle(surface, state),
        ShellMode::TileDetail(action) => draw_tile_detail(surface, action, state),
        ShellMode::AllApps => draw_all_apps(surface, frame, state),
        ShellMode::LockScreen => draw_lock_screen(surface, frame, state),
        ShellMode::Cortana => draw_cortana(surface, frame, state),
        ShellMode::Settings(route) => draw_settings(surface, frame, state, route),
        ShellMode::App(app) => draw_sample_app(surface, frame, state, app),
        ShellMode::Notifications => draw_notifications(surface, frame, state),
        ShellMode::AppSwitcher => draw_app_switcher(surface, frame, state),
    }
}
