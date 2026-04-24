use fhre::{Res, ResMut, Time};

use crate::resources::{ShellOverlayAnimation, ShellState, ThemeAnimation, ThemeState};

pub fn wing_overlay_animation_system(
    time: Res<Time>,
    shell_state: Res<ShellState>,
    mut animation: ResMut<ShellOverlayAnimation>,
    mut theme_animation: ResMut<ThemeAnimation>,
    mut theme_state: ResMut<ThemeState>,
) {
    animation.set_quick_settings_open(shell_state.quick_settings_open());
    animation.set_app_switcher_open(shell_state.app_switcher_open());
    
    let delta = time.delta();
    animation.update(delta);

    if theme_state.is_transitioning {
        theme_animation.update(delta);
        if !theme_animation.is_transitioning {
            theme_state.apply_pending_theme();
        }
    }
}
