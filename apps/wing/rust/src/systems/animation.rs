use fhre::{Res, ResMut, Time};

use crate::resources::{ShellOverlayAnimation, ShellOverlayMode, ShellState, ThemeAnimation, ThemeState};

pub fn wing_overlay_animation_system(
    time: Res<Time>,
    shell_state: Res<ShellState>,
    mut animation: ResMut<ShellOverlayAnimation>,
    mut theme_animation: ResMut<ThemeAnimation>,
    mut theme_state: ResMut<ThemeState>,
) {
    // Check for mode transition between NotificationPanel and AppSwitcher
    // If switching directly between them, immediately complete the outgoing animation
    match (&animation.previous_mode, &shell_state.overlay_mode) {
        (ShellOverlayMode::NotificationPanel, ShellOverlayMode::AppSwitcher) => {
            // Immediately complete notification panel animation
            animation.notification_panel_progress = 0.0;
            animation.target_notification_panel = 0.0;
        }
        (ShellOverlayMode::AppSwitcher, ShellOverlayMode::NotificationPanel) => {
            // Immediately complete app switcher animation
            animation.app_switcher_progress = 0.0;
            animation.target_app_switcher = 0.0;
        }
        _ => {}
    }
    
    // Update previous mode
    animation.previous_mode = shell_state.overlay_mode;
    
    animation.set_notification_panel_open(shell_state.notification_panel_open());
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
