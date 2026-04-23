//! Wing shell plugin scaffolding for FHRE app assembly.

use fhre::{App, Plugin};

use crate::resources::{DesktopMetrics, ShellState, ThemeState};

/// Minimal plugin for empty shell resource assembly.
pub struct WingShellPlugin;

impl Plugin for WingShellPlugin {
    fn build(&self, app: &mut App) {
        let screen = app.main_world.resources().get::<fhre::resources::PrimaryScreen>();
        let (width, height) = screen
            .map(|s| (s.width as f32, s.height as f32))
            .unwrap_or((800.0, 600.0));
        if app.main_world.resources().get::<ShellState>().is_none() {
            app.insert_resource(ShellState::default());
        }
        if app.main_world.resources().get::<ThemeState>().is_none() {
            app.insert_resource(ThemeState::default());
        }
        if app.main_world.resources().get::<DesktopMetrics>().is_none() {
            app.insert_resource(DesktopMetrics::new(fhre::Vec2::new(width, height), 48.0, 16.0));
        }
    }
}
