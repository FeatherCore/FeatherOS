//! Preview effect animation systems.

use fhre::{Query, Res, Time, Vec3};

use crate::{
    components::PreviewSoccerBall,
    resources::{PreviewEffect, ShellContent, ShellOverlayAnimation, ShellState},
};

pub fn wing_preview_effect_animation_system(
    time: Res<Time>,
    content: Res<ShellContent>,
    shell_state: Res<ShellState>,
    overlay_animation: Res<ShellOverlayAnimation>,
    mut preview_query: Query<&mut PreviewSoccerBall>,
) {
    if content.preview_effect != PreviewEffect::Soccer {
        return;
    }

    let visible = shell_state.app_switcher_open()
        || overlay_animation.app_switcher_progress > 0.001
        || overlay_animation.target_app_switcher > 0.001;

    if !visible {
        return;
    }

    let delta = time.delta();
    let rotation_delta = Vec3::new(16.0 * delta, 58.0 * delta, 9.0 * delta);

    for (_, ball) in preview_query.iter_mut() {
        ball.rotate(rotation_delta);
    }
}
