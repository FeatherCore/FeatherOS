#![no_std]
#![no_main]

//! FHRE 3D Demo - Pure Declarative ECS (Bevy-aligned)
//!
//! 展示旋转立方体或足球（截角二十面体），底部有三个按钮
//! 通过 feature "cube" 选择绘制对象
//!
//! ## Architecture (aligned with Bevy)
//!
//! ```ignore
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)     // EventPlugin + AnimationPlugin (+ 4 auto systems)
//!         .add_systems(Startup, setup)      // create entities + AnimationPlayer
//!         .add_systems(Startup, setup_anim) // create AnimationClip → store handle
//!         .add_systems(PreUpdate, input)    // Space=pause, Escape=exit
//!         .run();                          // ★ No custom animation systems needed!
//! }
//! ```
//!
//! ### How animation works (fully automatic):
//!
//! 1. **AnimationPlugin** registers 4 systems automatically in Update schedule:
//!    - `advance_animations`   → update playback time on all AnimationPlayers
//!    - `animate_targets`      → sample curves → store in `sampled_properties`
//!    - `apply_cube_animation` → write sampled values to Cube components
//!    - `apply_soccer_ball_animation` → write sampled values to SoccerBall components
//!
//! 2. User only declares: Clip definition + Entity with AnimationPlayer
//! 3. Zero manual apply code required

extern crate alloc;

extern "C" {
    fn printf(format: *const u8, ...) -> i32;
    fn usleep(usec: u32) -> i32;
}

#[cfg(feature = "sim")]
mod x11_window;

#[cfg(feature = "cube")]
const USE_CUBE: bool = true;
#[cfg(not(feature = "cube"))]
const USE_CUBE: bool = false;

use fhre::{
    App, FHRE_VERSION,
    node::{Node, NodeType, Transform2D, Transform3D},
    ui::{Button, Cube, SoccerBall},
    math::{Color, Vec3},
    resources::{Time, PrimaryScreen},
    Res, ResMut, Query, Commands,
    ButtonInput, MouseButton, KeyCode,
    Update, PreUpdate, Startup,
    system2, system3,
    DefaultPlugins,
    // NEW: Bevy-aligned dual-world sync markers
    SyncToRenderWorld,
};

use fhre::animation::{
    AnimationClip, AnimationClipHandle, AnimationPlayer,
    AnimationProperty, AnimationTargetId, AnimationResources,
    KeyframeCurve, Keyframe, Easing,
};

// ============================================================
// Resources
// ============================================================

/// Demo control state (pause/resume via input)
#[derive(Clone, Debug)]
pub struct DemoState {
    pub is_rotating: bool,
}

impl DemoState {
    pub const fn new() -> Self {
        Self { is_rotating: true }
    }
}

impl Default for DemoState {
    fn default() -> Self { Self::new() }
}

impl fhre::resources::Resource for DemoState {}

/// Shared rotation clip handle (set by setup_animation Startup system)
#[derive(Clone, Copy, Debug)]
pub struct RotationClip {
    pub handle: AnimationClipHandle,
}

impl fhre::resources::Resource for RotationClip {}

// ============================================================
// Systems
// ============================================================

/// Create scene entities (cube/ball + buttons) + attach AnimationPlayer
///
/// Bevy-style: just declare what exists. Animation is automatic.
fn setup(mut commands: Commands, screen: Res<PrimaryScreen>) {
    let (width, height) = screen.dimensions();
    let obj_x = width as f32 / 2.0;
    let obj_y = height as f32 / 3.0;

    #[cfg(feature = "cube")]
    {
        commands.spawn()
            .insert(Node::game_entity(NodeType::Empty))
            .insert(Transform3D::from_position(obj_x, obj_y, 0.0))
            .insert(Cube::new(120.0)
                .with_face_colors([
                    Color::rgb(255, 100, 100),
                    Color::rgb(100, 255, 100),
                    Color::rgb(100, 100, 255),
                    Color::rgb(255, 255, 100),
                    Color::rgb(255, 255, 255),
                    Color::rgb(100, 255, 255),
                ])
                .with_rotation(Vec3::new(45.0, 0.0, 45.0))
                .with_wireframe(true, Color::WHITE))
            .insert(AnimationPlayer::new())
            // NEW: Mark entity for sync to Render World (Bevy-aligned)
            .insert(SyncToRenderWorld);
    }

    #[cfg(not(feature = "cube"))]
    {
        commands.spawn()
            .insert(Node::game_entity(NodeType::Empty))
            .insert(Transform3D::from_position(obj_x, obj_y, 0.0))
            .insert(SoccerBall::new(120.0)
                .with_rotation(Vec3::new(0.0, 0.0, 0.0))
                .with_wireframe(true, Color::WHITE))
            .insert(AnimationPlayer::new())
            // NEW: Mark entity for sync to Render World (Bevy-aligned)
            .insert(SyncToRenderWorld);
    }

    let button_y = height as f32 - 80.0;
    let button_spacing = 140.0;
    let center_x = width as f32 / 2.0;

    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform2D::from_position(center_x - button_spacing, button_y))
        .insert(Button::new(100.0, 40.0)
            .with_text("Reset")
            .with_colors(
                Color::rgb(70, 130, 180),
                Color::rgb(100, 160, 210),
                Color::rgb(50, 100, 150),
            ));

    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform2D::from_position(center_x, button_y))
        .insert(Button::new(100.0, 40.0)
            .with_text("Pause")
            .with_colors(
                Color::rgb(60, 150, 80),
                Color::rgb(90, 180, 110),
                Color::rgb(40, 120, 60),
            ));

    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform2D::from_position(center_x + button_spacing, button_y))
        .insert(Button::new(100.0, 40.0)
            .with_text("Exit")
            .with_colors(
                Color::rgb(180, 70, 70),
                Color::rgb(210, 100, 100),
                Color::rgb(150, 50, 50),
            ));
}

/// Create multi-property animation clip (3D rotation + scale) and start playing
///
/// Demonstrates full 3D transform animation:
/// - RotationX: 0° → 360° (pitch, continuous)
/// - RotationY: 0° → 360° (yaw, continuous, slightly offset phase)
/// - RotationZ: 0° → 360° (roll, continuous)
/// - Scale: 1.0 → 1.5 → 1.0 (breathe effect)
///
/// Runs as a Startup system after AnimationPlugin has registered AnimationResources.
fn setup_animation(
    mut anim_resources: ResMut<AnimationResources>,
    mut clip_res: ResMut<RotationClip>,
    mut players: Query<AnimationPlayer>,
) {
    let duration = 6.0_f32;
    let target_id = AnimationTargetId::new(1);

    let mut clip = AnimationClip::with_duration(duration);
    {
        use alloc::vec;

        // === 3D Rotation (all axes, different speeds for visual interest) ===
        // Y-axis rotation: main spin (one full revolution per duration)
        clip.add_curve_to_target(
            target_id,
            AnimationProperty::RotationY,
            KeyframeCurve::new(vec![
                Keyframe::new(0.0,   0.0, Easing::Linear),
                Keyframe::new(duration, 360.0, Easing::Linear),
            ]),
        );

        // X-axis tilt: slower oscillation (half rev per duration)
        clip.add_curve_to_target(
            target_id,
            AnimationProperty::RotationX,
            KeyframeCurve::new(vec![
                Keyframe::new(0.0,      30.0, Easing::EaseInOut),
                Keyframe::new(duration * 0.5, -30.0, Easing::EaseInOut),
                Keyframe::new(duration,   30.0, Easing::EaseInOut),
            ]),
        );

        // Z-axis roll: medium speed (1.5 revs per duration)
        clip.add_curve_to_target(
            target_id,
            AnimationProperty::RotationZ,
            KeyframeCurve::new(vec![
                Keyframe::new(0.0,     45.0, Easing::Linear),
                Keyframe::new(duration, 405.0, Easing::Linear), // 360 + initial 45
            ]),
        );

        // === Scale (breathing effect) ===
        clip.add_curve_to_target(
            target_id,
            AnimationProperty::ScaleX,
            KeyframeCurve::new(vec![
                Keyframe::new(0.0,           120.0, Easing::EaseInOut),
                Keyframe::new(duration * 0.5, 160.0, Easing::EaseInOut),
                Keyframe::new(duration,       120.0, Easing::EaseInOut),
            ]),
        );
    }

    let handle = anim_resources.insert_clip(clip);
    clip_res.handle = handle;

    // Start playing on all AnimationPlayers (Bevy-style)
    for mut player in players.iter_mut() {
        player.play_with_target(handle, target_id);
    }
}

/// Input system: Space=pause/resume animation
fn input_system(key_input: Res<ButtonInput<KeyCode>>, mut state: ResMut<DemoState>) {
    if key_input.just_pressed(KeyCode::Space) {
        state.is_rotating = !state.is_rotating;
        if state.is_rotating {
            unsafe { printf(b"[DEMO] Animation resumed\n\0".as_ptr()); }
        } else {
            unsafe { printf(b"[DEMO] Animation paused\n\0".as_ptr()); }
        }
    }
}

/// Pause/resume all AnimationPlayers based on DemoState
///
/// Bridges user input (Space key) → AnimationPlayer.pause_all()/resume_all().
fn animation_control_system(state: Res<DemoState>, mut players: Query<AnimationPlayer>) {
    if state.is_rotating {
        for mut player in players.iter_mut() { player.resume_all(); }
    } else {
        for mut player in players.iter_mut() { player.pause_all(); }
    }
}

// ============================================================
// Main Entry - Bevy-style pure declarative App builder
// ============================================================

#[no_mangle]
pub extern "C" fn fhre_rust_main() -> i32 {
    unsafe {
        printf(b"\n========================================\n\0".as_ptr());
        printf(b"  FHRE %s Demo (Declarative ECS)\n\0".as_ptr(), FHRE_VERSION.as_ptr());
        if USE_CUBE {
            printf(b"  Mode: Cube\n\0".as_ptr());
        } else {
            printf(b"  Mode: Soccer Ball\n\0".as_ptr());
        }
        printf(b"========================================\n\n\0".as_ptr());
    }

    // Pure declarative App builder — aligned with bevy/examples/3d/3d_scene.rs
    //
    // Animation is FULLY AUTOMATIC:
    //   DefaultPlugins → AnimationPlugin registers:
    //     1. advance_animations       (time update)
    //     2. animate_targets          (curve sampling)
    //     3. apply_cube_animation     (Cube.apply_animation)
    //     4. apply_soccer_ball_animation (SoccerBall.apply_animation)
    //
    // User only provides:
    //   - setup:          entity creation + AnimationPlayer component
    //   - setup_animation: clip creation + .play_with_target()
    //   - input_system:   Space→pause/resume
    //   - animation_control: bridges DemoState → Player.pause/resume
    let mut app = App::new(640, 480);
    app.add_plugins(DefaultPlugins)
        .insert_resource(DemoState::new())
        .insert_resource(RotationClip { handle: AnimationClipHandle::null() })
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .add_systems(Startup, system2::<Commands, Res<PrimaryScreen>, _>(setup))
        .add_systems(Startup, system3::<ResMut<AnimationResources>, ResMut<RotationClip>, Query<AnimationPlayer>, _>(setup_animation))
        .add_systems(PreUpdate, system2::<Res<ButtonInput<KeyCode>>, ResMut<DemoState>, _>(input_system))
        .add_systems(Update, system2::<Res<DemoState>, Query<AnimationPlayer>, _>(animation_control_system));

    #[cfg(feature = "sim")]
    {
        run_sim_loop(app);
        return 0;
    }

    #[cfg(not(feature = "sim"))]
    {
        app.run();
        return 0;
    }
}

/// SIM platform main loop with X11 window (platform adaptation layer)
///
/// FHRE knows nothing about X11. This function is the ONLY place where
/// platform-specific code lives — it bridges X11 events ↔ FHRE resources.
#[cfg(feature = "sim")]
fn run_sim_loop(mut app: App) {
    let mut window = match x11_window::X11Window::new(640, 480, "FHRE Demo") {
        Some(w) => w,
        None => {
            unsafe { printf(b"[ERROR] Failed to create X11 window\n\0".as_ptr()); }
            return;
        }
    };

    loop {
        let events = window.collect_input_events();
        if !window.is_running() { break; }

        // Bridge: X11 events → FHRE ButtonInput resources
        if let Some(key_input) = app.main_world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
            key_input.clear();
            for event in &events.keyboard_events {
                if let Some(kc) = x11_keycode_to_fhre(event.keycode) {
                    if event.pressed { key_input.press(kc); } else { key_input.release(kc); }
                }
            }
        }
        if let Some(mouse_input) = app.main_world.resources_mut().get_mut::<ButtonInput<MouseButton>>() {
            mouse_input.clear();
            for event in &events.mouse_button_events {
                if let Some(btn) = x11_button_to_fhre(event.button) {
                    if event.pressed { mouse_input.press(btn); } else { mouse_input.release(btn); }
                }
            }
        }

        // Core FHRE call: run ALL systems (including auto animation systems)
        app.update_and_render();

        // Platform output: present framebuffer to window
        window.present(app.framebuffer());

        unsafe { usleep(16_000); }
    }
}

#[cfg(feature = "sim")]
fn x11_keycode_to_fhre(kc: u32) -> Option<KeyCode> {
    match kc {
        65 => Some(KeyCode::Space),
        27 => Some(KeyCode::KeyR),
        9 => Some(KeyCode::Escape),
        _ => None,
    }
}

#[cfg(feature = "sim")]
fn x11_button_to_fhre(btn: u32) -> Option<MouseButton> {
    match btn {
        1 => Some(MouseButton::Left),
        2 => Some(MouseButton::Middle),
        3 => Some(MouseButton::Right),
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn rust_fhre_demo_init() {}
