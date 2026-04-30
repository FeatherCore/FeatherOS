use crate::{
    action::{
        action_label, ActionId, ACTION_ALL_APPS, ACTION_DRAW, ACTION_SETTINGS,
        ACTION_APP_NEWS, ACTION_APP_STARS, ACTION_APP_THERMAL, ACTION_CORTANA, ACTION_HOME,
        ACTION_LOCK_SCREEN, ACTION_SETTINGS_ABOUT, ACTION_SETTINGS_ACCOUNT, ACTION_SETTINGS_APPS,
        ACTION_SETTINGS_DEVICES, ACTION_SETTINGS_NETWORK, ACTION_SETTINGS_PERSONALIZATION,
        ACTION_SETTINGS_PRIVACY, ACTION_SETTINGS_SYSTEM, ACTION_SETTINGS_TIME,
    },
    app::{AppId, AppSurfaceState, APP_FHRE_SAMPLE, APP_NEWS, APP_STARS, APP_THERMAL},
    node::UiHit,
};
use fhre::{
    EventQueue, GestureDirection, GestureEvent, GestureKind, GestureRecognizer, InputEvent,
    KeyCode, PointerEvent, PointerPhase,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellMode {
    Home,
    TileDetail(ActionId),
    AllApps,
    LockScreen,
    Cortana,
    Settings(SettingsRoute),
    App(AppId),
    Notifications,
    AppSwitcher,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsRoute {
    Main,
    System,
    Personalization,
    Network,
    About,
    Account,
    Apps,
    Devices,
    Privacy,
    Time,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShellAction {
    Select(ActionId),
    BackHome,
    ShowNotifications,
    ShowAppSwitcher,
}

pub type ShellActionQueue<const N: usize> = EventQueue<ShellAction, N>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScrollState {
    pub offset: i32,
    pub max_offset: i32,
    pub dragging: bool,
    pub last_pointer_y: i32,
    start_pointer_y: i32,
}

impl ScrollState {
    pub const fn new() -> Self {
        Self {
            offset: 0,
            max_offset: 0,
            dragging: false,
            last_pointer_y: 0,
            start_pointer_y: 0,
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        self.max_offset = 0;
        self.dragging = false;
        self.last_pointer_y = 0;
        self.start_pointer_y = 0;
    }

    pub fn set_max_offset(&mut self, max_offset: i32) {
        self.max_offset = max_offset.max(0);
        self.offset = self.offset.clamp(0, self.max_offset);
    }

    pub fn begin(&mut self, pointer_y: i32) {
        self.dragging = false;
        self.last_pointer_y = pointer_y;
        self.start_pointer_y = pointer_y;
    }

    pub fn drag_to(&mut self, pointer_y: i32) -> bool {
        let delta_y = pointer_y.saturating_sub(self.last_pointer_y);
        self.last_pointer_y = pointer_y;
        if delta_y == 0 {
            return self.dragging;
        }

        let total_y = pointer_y.saturating_sub(self.start_pointer_y);
        if !self.dragging && abs_i32(total_y) > 3 {
            self.dragging = true;
        }
        if self.dragging {
            self.scroll_by(delta_y.saturating_neg());
        }
        self.dragging
    }

    pub fn end(&mut self) -> bool {
        let was_dragging = self.dragging;
        self.dragging = false;
        was_dragging
    }

    pub fn scroll_by(&mut self, delta: i32) {
        self.offset = self.offset.saturating_add(delta).clamp(0, self.max_offset);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShellState {
    pub mode: ShellMode,
    pub selected: Option<ActionId>,
    pub running_app: Option<AppId>,
    pub all_apps_scroll: ScrollState,
    pub settings_scroll: ScrollState,
    pub pressed: Option<UiHit>,
    pub last_hit: Option<UiHit>,
    pub last_gesture: Option<GestureEvent>,
    pub event_count: u32,
    gesture: GestureRecognizer,
}

impl ShellState {
    pub const fn new() -> Self {
        Self {
            mode: ShellMode::Home,
            selected: None,
            running_app: None,
            all_apps_scroll: ScrollState::new(),
            settings_scroll: ScrollState::new(),
            pressed: None,
            last_hit: None,
            last_gesture: None,
            event_count: 0,
            gesture: GestureRecognizer::new(),
        }
    }

    pub fn dispatch_input(&mut self, event: InputEvent, hit: Option<UiHit>) -> Option<ShellAction> {
        match event {
            InputEvent::Pointer(pointer) => self.dispatch_pointer(pointer, hit),
            InputEvent::Key(key) => {
                self.event_count = self.event_count.wrapping_add(1);
                if key.pressed
                    && matches!(key.code, KeyCode::Back | KeyCode::Home | KeyCode::Escape)
                {
                    self.apply_action(ShellAction::BackHome);
                    Some(ShellAction::BackHome)
                } else if key.pressed && self.scroll_key(key.code) {
                    None
                } else {
                    None
                }
            }
        }
    }

    pub fn dispatch_pointer(&mut self, event: PointerEvent, hit: Option<UiHit>) -> Option<ShellAction> {
        self.event_count = self.event_count.wrapping_add(1);
        self.last_hit = hit;
        let gesture = self.gesture.update(event);
        self.last_gesture = gesture;

        match event.phase {
            PointerPhase::Down => {
                if let Some(scroll) = self.active_scroll_mut() {
                    scroll.begin(event.position.y);
                }
                self.pressed = hit;
                None
            }
            PointerPhase::Up => {
                let scrolled = self.active_scroll_mut().map(|scroll| scroll.end()).unwrap_or(false);
                if scrolled {
                    self.pressed = None;
                    return None;
                }
                let action = gesture.and_then(|gesture| self.action_for_gesture(gesture, hit));
                self.pressed = None;
                if let Some(action) = action {
                    self.apply_action(action);
                    Some(action)
                } else {
                    None
                }
            }
            PointerPhase::Cancel => {
                if let Some(scroll) = self.active_scroll_mut() {
                    scroll.end();
                }
                self.pressed = None;
                None
            }
            PointerPhase::Move => {
                if let Some(scroll) = self.active_scroll_mut() {
                    if scroll.drag_to(event.position.y) {
                        self.pressed = None;
                        return None;
                    }
                }
                if matches!(
                    gesture.map(|gesture| gesture.kind),
                    Some(GestureKind::DragStart | GestureKind::Drag)
                ) {
                    self.pressed = None;
                }
                None
            }
        }
    }

    pub fn apply_action(&mut self, action: ShellAction) {
        match action {
            ShellAction::Select(action_id) => {
                self.selected = Some(action_id);
                let previous = self.mode;
                let next = if action_id == ACTION_HOME {
                    ShellMode::Home
                } else if action_id == ACTION_CORTANA {
                    ShellMode::Cortana
                } else if action_id == ACTION_LOCK_SCREEN {
                    ShellMode::LockScreen
                } else if action_id == ACTION_SETTINGS {
                    ShellMode::Settings(SettingsRoute::Main)
                } else if action_id == ACTION_ALL_APPS {
                    ShellMode::AllApps
                } else if action_id == ACTION_SETTINGS_SYSTEM {
                    ShellMode::Settings(SettingsRoute::System)
                } else if action_id == ACTION_SETTINGS_PERSONALIZATION {
                    ShellMode::Settings(SettingsRoute::Personalization)
                } else if action_id == ACTION_SETTINGS_NETWORK {
                    ShellMode::Settings(SettingsRoute::Network)
                } else if action_id == ACTION_SETTINGS_ABOUT {
                    ShellMode::Settings(SettingsRoute::About)
                } else if action_id == ACTION_SETTINGS_ACCOUNT {
                    ShellMode::Settings(SettingsRoute::Account)
                } else if action_id == ACTION_SETTINGS_APPS {
                    ShellMode::Settings(SettingsRoute::Apps)
                } else if action_id == ACTION_SETTINGS_DEVICES {
                    ShellMode::Settings(SettingsRoute::Devices)
                } else if action_id == ACTION_SETTINGS_PRIVACY {
                    ShellMode::Settings(SettingsRoute::Privacy)
                } else if action_id == ACTION_SETTINGS_TIME {
                    ShellMode::Settings(SettingsRoute::Time)
                } else if action_id == ACTION_DRAW {
                    self.running_app = Some(APP_FHRE_SAMPLE);
                    ShellMode::App(APP_FHRE_SAMPLE)
                } else if action_id == ACTION_APP_NEWS {
                    self.running_app = Some(APP_NEWS);
                    ShellMode::App(APP_NEWS)
                } else if action_id == ACTION_APP_STARS {
                    self.running_app = Some(APP_STARS);
                    ShellMode::App(APP_STARS)
                } else if action_id == ACTION_APP_THERMAL {
                    self.running_app = Some(APP_THERMAL);
                    ShellMode::App(APP_THERMAL)
                } else {
                    ShellMode::TileDetail(action_id)
                };
                self.apply_mode_transition(previous, next);
            }
            ShellAction::BackHome => {
                self.selected = None;
                let previous = self.mode;
                self.apply_mode_transition(previous, ShellMode::Home);
                self.pressed = None;
                self.gesture.reset();
            }
            ShellAction::ShowNotifications => {
                self.selected = None;
                let previous = self.mode;
                self.apply_mode_transition(previous, ShellMode::Notifications);
            }
            ShellAction::ShowAppSwitcher => {
                self.selected = None;
                let previous = self.mode;
                self.apply_mode_transition(previous, ShellMode::AppSwitcher);
            }
        }
    }

    pub fn status_label(&self) -> &'static str {
        match self.mode {
            ShellMode::Home => "ACTION: READY",
            ShellMode::TileDetail(action) => action_label(action),
            ShellMode::AllApps => "ACTION: ALL APPS",
            ShellMode::LockScreen => "ACTION: LOCK",
            ShellMode::Cortana => "ACTION: CORTANA",
            ShellMode::Settings(route) => settings_route_label(route),
            ShellMode::App(_) => "ACTION: FHRE APP",
            ShellMode::Notifications => "ACTION: NOTIFY",
            ShellMode::AppSwitcher => "ACTION: SWITCH",
        }
    }

    pub const fn launcher_interactive(&self) -> bool {
        matches!(self.mode, ShellMode::Home)
    }

    pub const fn overlay_interactive(&self) -> bool {
        matches!(
            self.mode,
            ShellMode::Home
                | ShellMode::AllApps
                | ShellMode::Cortana
                | ShellMode::Settings(_)
                | ShellMode::Notifications
                | ShellMode::AppSwitcher
        )
    }

    pub fn app_surface_state(&self, app: AppId) -> AppSurfaceState {
        if self.running_app != Some(app) {
            return AppSurfaceState::Stopped;
        }

        match self.mode {
            ShellMode::App(focused) if focused == app => AppSurfaceState::Focused,
            ShellMode::AppSwitcher => AppSurfaceState::Preview,
            _ => AppSurfaceState::Running,
        }
    }

    fn action_for_gesture(&self, gesture: GestureEvent, hit: Option<UiHit>) -> Option<ShellAction> {
        match gesture.kind {
            GestureKind::Tap => self.action_for_tap(hit),
            GestureKind::Swipe(GestureDirection::Down) => {
                if self.mode == ShellMode::LockScreen || self.is_scrollable_mode() {
                    None
                } else {
                    Some(ShellAction::ShowNotifications)
                }
            }
            GestureKind::Swipe(GestureDirection::Up) => {
                if self.mode == ShellMode::LockScreen {
                    Some(ShellAction::BackHome)
                } else if self.is_scrollable_mode() {
                    None
                } else {
                    Some(ShellAction::ShowAppSwitcher)
                }
            }
            GestureKind::Swipe(GestureDirection::Left) | GestureKind::Swipe(GestureDirection::Right) => {
                Some(ShellAction::BackHome)
            }
            GestureKind::DragStart | GestureKind::Drag | GestureKind::DragEnd => None,
        }
    }

    fn action_for_tap(&self, hit: Option<UiHit>) -> Option<ShellAction> {
        if let (Some(down), Some(up)) = (self.pressed, hit) {
            if down.key == up.key {
                return Some(ShellAction::Select(up.action));
            }
        }
        None
    }

    fn apply_mode_transition(&mut self, previous: ShellMode, next: ShellMode) {
        if !matches!(previous, ShellMode::AllApps) && matches!(next, ShellMode::AllApps) {
            self.all_apps_scroll.reset();
        }
        if !matches!(previous, ShellMode::Settings(_)) && matches!(next, ShellMode::Settings(_)) {
            self.settings_scroll.reset();
        }
        if let (ShellMode::Settings(old), ShellMode::Settings(new)) = (previous, next) {
            if old != new {
                self.settings_scroll.reset();
            }
        }
        self.mode = next;
    }

    fn is_scrollable_mode(&self) -> bool {
        matches!(self.mode, ShellMode::AllApps | ShellMode::Settings(_))
    }

    fn active_scroll_mut(&mut self) -> Option<&mut ScrollState> {
        match self.mode {
            ShellMode::AllApps => Some(&mut self.all_apps_scroll),
            ShellMode::Settings(_) => Some(&mut self.settings_scroll),
            _ => None,
        }
    }

    fn scroll_key(&mut self, code: KeyCode) -> bool {
        let step = 36;
        match code {
            KeyCode::Up => {
                if let Some(scroll) = self.active_scroll_mut() {
                    scroll.scroll_by(-step);
                    true
                } else {
                    false
                }
            }
            KeyCode::Down => {
                if let Some(scroll) = self.active_scroll_mut() {
                    scroll.scroll_by(step);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

const fn settings_route_label(route: SettingsRoute) -> &'static str {
    match route {
        SettingsRoute::Main => "ACTION: SETTINGS",
        SettingsRoute::System => "ACTION: SYSTEM",
        SettingsRoute::Personalization => "ACTION: PERSONALIZE",
        SettingsRoute::Network => "ACTION: NETWORK",
        SettingsRoute::About => "ACTION: ABOUT",
        SettingsRoute::Account => "ACTION: ACCOUNT",
        SettingsRoute::Apps => "ACTION: APPS",
        SettingsRoute::Devices => "ACTION: DEVICES",
        SettingsRoute::Privacy => "ACTION: PRIVACY",
        SettingsRoute::Time => "ACTION: TIME",
    }
}

const fn abs_i32(value: i32) -> i32 {
    if value < 0 {
        value.saturating_neg()
    } else {
        value
    }
}
