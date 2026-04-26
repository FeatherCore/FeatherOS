//! Wing shell plugin for assembling the default shell pipeline.

use fhre::{App, Plugin};

use crate::extract::{extract_view, extract_wing_shell, queue_wing_primitives};
use crate::resources::{GestureState, ShellContent, ShellMetrics, ShellOverlayAnimation, ShellState, ThemeAnimation, ThemeState};
use crate::systems::{
    setup_wing_shell, wing_gesture_system, wing_minimal_button_interaction_system,
    wing_overlay_animation_system, wing_picking_system,
    wing_shell_interaction_system, wing_shell_layout_system, wing_shell_overlay_layout_system,
    wing_shell_notification_panel_layout_system, wing_shell_quick_controls_layout_system,
    wing_shell_notification_cards_layout_system, wing_shell_overlay_card_layout_system, wing_shell_stack_layout_system,
};

/// Registers the default Wing shell resources, systems, and extractors.
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
        if app.main_world.resources().get::<ShellContent>().is_none() {
            app.insert_resource(ShellContent::default());
        }
        if app.main_world.resources().get::<ThemeState>().is_none() {
            app.insert_resource(ThemeState::default());
        }
        if app.main_world.resources().get::<ShellMetrics>().is_none() {
            app.insert_resource(ShellMetrics::new(fhre::Vec2::new(width, height)));
        }
        if app.main_world.resources().get::<GestureState>().is_none() {
            app.insert_resource(GestureState::default());
        }
        if app.main_world.resources().get::<ShellOverlayAnimation>().is_none() {
            app.insert_resource(ShellOverlayAnimation::default());
        }
        if app.main_world.resources().get::<ThemeAnimation>().is_none() {
            app.insert_resource(ThemeAnimation::default());
        }

        app.add_systems(
            fhre::Startup,
            fhre::declare_system!(setup_wing_shell; fhre::Commands, fhre::Res<fhre::PrimaryScreen>, fhre::Res<ShellContent>, fhre::ResMut<ShellState>),
        )
        .add_systems(
            fhre::PreUpdate,
            fhre::declare_system!(wing_picking_system; fhre::Res<fhre::MousePosition>, fhre::Res<crate::ButtonInput<crate::MouseButton>>, fhre::Query<&fhre::Transform>, fhre::Query<&fhre::PickableBounds>, fhre::Query<&fhre::Pickable>, fhre::ResMut<fhre::HoverMap>, fhre::ResMut<fhre::PreviousHoverMap>, fhre::ResMut<fhre::PointerPress>, fhre::ResMut<fhre::PointerLocation>, fhre::ResMut<fhre::Events>),
        )
        .add_systems(
            fhre::PreUpdate,
            fhre::declare_system!(wing_minimal_button_interaction_system; fhre::Res<fhre::Events>, fhre::Query<&mut crate::ButtonWidget>),
        )
        .add_systems(
            fhre::PreUpdate,
            fhre::declare_system!(wing_shell_interaction_system; fhre::Res<fhre::Events>, fhre::ResMut<ShellState>, fhre::Query<&crate::OverlayLayer>, fhre::Query<&crate::SurfacePreviewCard>, fhre::Query<&crate::AppSurface>, fhre::Query<&crate::NotificationPanel>),
        )
        .add_systems(
            fhre::PreUpdate,
            fhre::declare_system!(wing_gesture_system; fhre::Res<fhre::MousePosition>, fhre::Res<crate::ButtonInput<crate::MouseButton>>, fhre::Res<fhre::Time>, fhre::ResMut<GestureState>, fhre::ResMut<ShellState>, fhre::ResMut<ThemeState>, fhre::Res<fhre::Events>),
        )
        .add_systems(
            fhre::Update,
            fhre::declare_system!(wing_overlay_animation_system; fhre::Res<fhre::Time>, fhre::Res<ShellState>, fhre::ResMut<ShellOverlayAnimation>, fhre::ResMut<ThemeAnimation>, fhre::ResMut<ThemeState>),
        )
        .add_systems(
            fhre::Update,
            fhre::declare_system!(wing_shell_layout_system; fhre::Res<ShellMetrics>, fhre::Res<ShellState>, fhre::Query<&mut fhre::Transform>, fhre::Query<&mut fhre::PickableBounds>, fhre::Query<&mut crate::HomeSurface>, fhre::Query<&mut crate::AppSurface>),
        )
        .add_systems(
            fhre::Update,
            fhre::declare_system!(wing_shell_stack_layout_system; fhre::Res<ShellMetrics>, fhre::Query<&mut fhre::Transform>, fhre::Query<&crate::SurfaceStackRoot>, fhre::Query<&crate::CardStackRoot>),
        )
        .add_systems(
            fhre::Update,
            fhre::declare_system!(wing_shell_overlay_layout_system; fhre::Res<ShellMetrics>, fhre::Res<ShellState>, fhre::Res<ShellOverlayAnimation>, fhre::Query<&mut fhre::Transform>, fhre::Query<&mut fhre::PickableBounds>, fhre::Query<&mut crate::OverlayLayer>),
        )
        .add_systems(
            fhre::Update,
            fhre::declare_system!(wing_shell_notification_panel_layout_system; fhre::Res<ShellMetrics>, fhre::Res<ShellState>, fhre::Res<ShellOverlayAnimation>, fhre::Query<&mut fhre::Transform>, fhre::Query<&mut fhre::PickableBounds>, fhre::Query<&mut crate::NotificationPanel>),
        )
        .add_systems(
            fhre::Update,
            fhre::declare_system!(wing_shell_quick_controls_layout_system; fhre::Res<ShellMetrics>, fhre::Res<ShellState>, fhre::Res<ShellOverlayAnimation>, fhre::Res<ShellContent>, fhre::Query<&mut fhre::Transform>, fhre::Query<&mut crate::BrightnessControl>, fhre::Query<&mut crate::QuickControlTile>),
        )
        .add_systems(
            fhre::Update,
            fhre::declare_system!(wing_shell_notification_cards_layout_system; fhre::Res<ShellMetrics>, fhre::Res<ShellState>, fhre::Res<ShellOverlayAnimation>, fhre::Query<&mut fhre::Transform>, fhre::Query<&mut fhre::PickableBounds>, fhre::Query<&mut crate::NotificationCard>),
        )
        .add_systems(
            fhre::Update,
            fhre::declare_system!(wing_shell_overlay_card_layout_system; fhre::Res<ShellMetrics>, fhre::Res<ShellContent>, fhre::Res<ShellState>, fhre::Res<ShellOverlayAnimation>, fhre::Query<&mut fhre::Transform>, fhre::Query<&mut fhre::PickableBounds>, fhre::Query<&mut crate::SurfacePreviewCard>),
        )
        .add_extractor(extract_view)
        .add_extractor(extract_wing_shell)
        .add_extractor(queue_wing_primitives);
    }
}
