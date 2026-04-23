//! Wing shell demo running on FHRE.

#![no_std]
#![no_main]

mod platform;

extern crate alloc;

use fhre::{App, DefaultPlugins, MousePosition, Window as _};
use platform::framebuffer;
use platform::runner::PlatformInputPlugin;
use wing::{
    extract_view, extract_wing_shell, queue_wing_primitives, ButtonInput, DesktopMetrics,
    KeyCode, MouseButton, ThemeState, WingShellPlugin,
};

const FRAME_DELAY_MS: u32 = 16;

#[no_mangle]
pub extern "C" fn wing_rust_main() -> i32 {
    let mut window = match framebuffer::Window::new() {
        Some(window) => window,
        None => return 0,
    };

    let (width, height) = window.dimensions();
    let mut app = App::new(width, height);

    app.add_plugins(DefaultPlugins)
        .add_plugin(WingShellPlugin)
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(MousePosition::default())
        .insert_resource(ThemeState::default())
        .insert_resource(DesktopMetrics::new(fhre::Vec2::new(width as f32, height as f32), 48.0, 16.0))
        .add_systems(fhre::Startup, fhre::declare_system!(wing::setup_wing_shell; fhre::Commands, fhre::Res<fhre::PrimaryScreen>, fhre::ResMut<wing::ShellState>))
        .add_systems(fhre::PreUpdate, fhre::declare_system!(wing::wing_picking_system; fhre::Res<fhre::MousePosition>, fhre::Res<wing::ButtonInput<wing::MouseButton>>, fhre::Query<&fhre::Transform>, fhre::Query<&fhre::PickableBounds>, fhre::Query<&fhre::Pickable>, fhre::ResMut<fhre::HoverMap>, fhre::ResMut<fhre::PreviousHoverMap>, fhre::ResMut<fhre::PointerPress>, fhre::ResMut<fhre::PointerLocation>, fhre::ResMut<fhre::Events>))
        .add_systems(fhre::PreUpdate, fhre::declare_system!(wing::wing_minimal_button_interaction_system; fhre::Res<fhre::Events>, fhre::Query<&mut wing::ButtonWidget>))
        .add_systems(fhre::PreUpdate, fhre::declare_system!(wing::wing_shell_interaction_system; fhre::Res<fhre::Events>, fhre::ResMut<wing::ShellState>, fhre::Query<&wing::StatusBar>, fhre::Query<&wing::BottomBar>, fhre::Query<&wing::GestureZone>, fhre::Query<&wing::OverlayLayer>, fhre::Query<&wing::SurfacePreviewCard>, fhre::Query<&wing::AppSurface>))
        .add_systems(fhre::Update, fhre::declare_system!(wing::wing_shell_layout_system; fhre::Res<wing::DesktopMetrics>, fhre::Res<wing::ShellState>, fhre::Query<&mut fhre::Transform>, fhre::Query<&mut fhre::PickableBounds>, fhre::Query<&mut wing::HomeSurface>, fhre::Query<&wing::StatusBar>, fhre::Query<&wing::BottomBar>, fhre::Query<&mut wing::AppSurface>))
        .add_systems(fhre::Update, fhre::declare_system!(wing::wing_shell_stack_layout_system; fhre::Res<wing::DesktopMetrics>, fhre::Query<&mut fhre::Transform>, fhre::Query<&wing::SurfaceStackRoot>, fhre::Query<&wing::CardStackRoot>, fhre::Query<&wing::NotificationStackRoot>))
        .add_systems(fhre::Update, fhre::declare_system!(wing::wing_shell_overlay_layout_system; fhre::Res<wing::DesktopMetrics>, fhre::Res<wing::ShellState>, fhre::Query<&mut fhre::Transform>, fhre::Query<&mut fhre::PickableBounds>, fhre::Query<&mut wing::OverlayLayer>, fhre::Query<&mut wing::NotificationLayer>, fhre::Query<&mut wing::NotificationCard>, fhre::Query<&wing::GestureZone>, fhre::Query<&mut wing::QuickSettingsPanel>, fhre::Query<&mut wing::SurfacePreviewCard>))
        .add_systems(fhre::Update, fhre::declare_system!(wing::wing_notification_text_layout_system; fhre::Res<wing::DesktopMetrics>, fhre::Res<wing::ShellState>, fhre::Query<&mut fhre::Transform>, fhre::Query<&wing::NotificationText>));

    app.add_extractor(extract_view)
        .add_extractor(extract_wing_shell)
        .add_extractor(queue_wing_primitives);

    let input_plugin = PlatformInputPlugin::new(framebuffer::InputAdapter);
    app.run(&mut window, &input_plugin, FRAME_DELAY_MS);
    0
}

#[no_mangle]
pub extern "C" fn rust_wing_demo_init() {}
