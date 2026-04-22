//! Wing plugin scaffolding for FHRE app assembly.

use fhre::{App, Plugin, Update, declare_system};

use crate::extract::queue_wing_shell;
use crate::resources::{DesktopMetrics, DragTransaction, FocusState, LauncherState, LayoutInvalidation, SelectionState, TaskbarState, TextInputState, ThemeState, WindowManagerState, WingDesktopState, WingRuntime};
use crate::systems::{wing_pointer_input_system, wing_shell_shortcut_system, wing_shell_state_sync_system, wing_text_input_system, wing_update_system, wing_window_state_sync_system};

/// Transitional plugin that registers current Wing runtime through FHRE resources.
pub struct WingDesktopPlugin;

impl Plugin for WingDesktopPlugin {
    fn build(&self, app: &mut App) {
        let screen = app.main_world.resources().get::<fhre::resources::PrimaryScreen>();
        let (width, height) = screen
            .map(|s| (s.width as f32, s.height as f32))
            .unwrap_or((800.0, 600.0));

        if app.main_world.resources().get::<WingRuntime>().is_none() {
            app.insert_resource(WingRuntime::new(width, height));
        }
        if app.main_world.resources().get::<WingDesktopState>().is_none() {
            app.insert_resource(WingDesktopState::default());
        }
        if app.main_world.resources().get::<WindowManagerState>().is_none() {
            app.insert_resource(WindowManagerState::default());
        }
        if app.main_world.resources().get::<DragTransaction>().is_none() {
            app.insert_resource(DragTransaction::default());
        }
        if app.main_world.resources().get::<FocusState>().is_none() {
            app.insert_resource(FocusState::default());
        }
        if app.main_world.resources().get::<SelectionState>().is_none() {
            app.insert_resource(SelectionState::default());
        }
        if app.main_world.resources().get::<TextInputState>().is_none() {
            app.insert_resource(TextInputState::default());
        }
        if app.main_world.resources().get::<LauncherState>().is_none() {
            app.insert_resource(LauncherState::default());
        }
        if app.main_world.resources().get::<TaskbarState>().is_none() {
            app.insert_resource(TaskbarState::default());
        }
        if app.main_world.resources().get::<LayoutInvalidation>().is_none() {
            app.insert_resource(LayoutInvalidation::default());
        }
        if app.main_world.resources().get::<ThemeState>().is_none() {
            app.insert_resource(ThemeState::default());
        }
        if app.main_world.resources().get::<DesktopMetrics>().is_none() {
            app.insert_resource(DesktopMetrics::new(fhre::Vec2::new(width, height), 48.0, 16.0));
        }

        app.add_systems(Update, declare_system!(wing_pointer_input_system; fhre::Res<fhre::MousePosition>, fhre::Res<crate::ButtonInput<crate::MouseButton>>, fhre::Res<crate::MouseWheel>, fhre::ResMut<DragTransaction>, fhre::ResMut<WingRuntime>))
            .add_systems(Update, declare_system!(wing_shell_shortcut_system; fhre::Res<crate::ButtonInput<crate::KeyCode>>, fhre::ResMut<FocusState>, fhre::ResMut<LauncherState>, fhre::ResMut<TaskbarState>, fhre::ResMut<WindowManagerState>, fhre::ResMut<WingRuntime>))
            .add_systems(Update, declare_system!(wing_text_input_system; fhre::Res<crate::ButtonInput<crate::KeyCode>>, fhre::ResMut<TextInputState>, fhre::ResMut<WingRuntime>))
            .add_systems(Update, declare_system!(wing_update_system; fhre::ResMut<WingRuntime>))
            .add_systems(Update, declare_system!(wing_shell_state_sync_system; fhre::ResMut<WingRuntime>, fhre::ResMut<WingDesktopState>, fhre::ResMut<LauncherState>, fhre::ResMut<TaskbarState>, fhre::ResMut<ThemeState>, fhre::ResMut<DesktopMetrics>, fhre::ResMut<LayoutInvalidation>))
            .add_systems(Update, declare_system!(wing_window_state_sync_system; fhre::ResMut<WingRuntime>, fhre::ResMut<WindowManagerState>, fhre::ResMut<FocusState>, fhre::ResMut<TextInputState>, fhre::ResMut<DragTransaction>))
            .add_extractor(queue_wing_shell);
    }
}
