//! Feather Hybrid Render Engine Demo
//!
//! A 3D model viewer with touch button controls for NuttX SIM platform.
//! Features:
//! - Two 3D models: Cube and Soccer Ball
//! - Touch button controls: Previous, Pause/Resume, Next
//! - Smooth rotation animation with easing

#![no_std]
#![no_main]

mod components;
mod extract;
mod platform;

extern crate alloc;

use platform::framebuffer;

use fhre::{
    App, FHRE_VERSION,
    node::{Node, NodeType, Transform2D, Transform3D},
    math::{Color, Vec3},
    resources::PrimaryScreen,
    window::MousePosition,
    Res, ResMut, Query, Commands, Entity,
    ButtonInput, MouseButton, KeyCode,
    Update, PreUpdate, Startup,
    DefaultPlugins,
    SyncToRenderWorld,
    declare_system,
};

use components::{Button, Cube, SoccerBall};
use fhre::window::WindowRunner;
use fhre::animation::{
    AnimationClip, AnimationClipHandle, AnimationPlayer,
    AnimationProperty, AnimationTargetId, AnimationResources,
    KeyframeCurve, Keyframe, Easing,
    apply_animations,
};

// ============================================================
// Demo Configuration Constants
// ============================================================

/// Model configuration
mod config {
    /// Number of models in the demo
    pub const MODEL_COUNT: usize = 2;
    
    /// Model names for display
    pub const MODEL_NAMES: [&'static str; MODEL_COUNT] = ["Cube", "Soccer Ball"];
    
    /// Initial model size in pixels
    pub const MODEL_SIZE: f32 = 120.0;
    
    /// Model size when scaled up (for animation)
    pub const MODEL_SIZE_MAX: f32 = 160.0;
    
    /// Initial rotation angles for cube (degrees)
    pub const CUBE_ROT_X: f32 = 45.0;
    pub const CUBE_ROT_Z: f32 = 45.0;
}

/// UI button configuration
mod ui {
    /// Button width in pixels
    pub const BUTTON_WIDTH: f32 = 100.0;
    
    /// Button height in pixels
    pub const BUTTON_HEIGHT: f32 = 40.0;
    
    /// Gap between buttons in pixels
    pub const BUTTON_GAP: f32 = 50.0;
    
    /// Distance from bottom of screen to button center
    pub const BUTTON_BOTTOM_MARGIN: f32 = 80.0;
    
    /// Touch detection radius around button center
    pub const BUTTON_TOUCH_RADIUS: f32 = 20.0;
    
    /// Button spacing (width + gap)
    pub const BUTTON_SPACING: f32 = BUTTON_WIDTH + BUTTON_GAP;
}

/// Animation configuration
mod anim {
    /// Total animation duration in seconds
    pub const DURATION: f32 = 6.0;
    
    /// Half animation duration (for symmetric keyframes)
    pub const DURATION_HALF: f32 = DURATION * 0.5;
    
    /// Rotation angle range (degrees)
    pub const ROTATION_RANGE: f32 = 360.0;
    
    /// Wobble amplitude for X-axis rotation (degrees)
    pub const WOBBLE_AMPLITUDE: f32 = 30.0;
    
    /// Initial Z rotation offset (degrees)
    pub const Z_ROTATION_OFFSET: f32 = 45.0;
    
    /// Animation target ID for property binding
    pub const TARGET_ID: u64 = 1;
}

/// Frame timing
mod timing {
    /// Target frame delay in milliseconds (~60 FPS)
    pub const FRAME_DELAY_MS: u32 = 16;
}

// ============================================================
// Resources
// ============================================================

/// Demo state tracking current model and animation state
#[derive(Clone, Debug)]
pub struct DemoState {
    /// Whether the model is currently rotating
    pub is_rotating: bool,
    /// Index of the currently displayed model
    pub current_model: usize,
}

impl DemoState {
    pub const fn new() -> Self {
        Self { 
            is_rotating: true,
            current_model: 0,
        }
    }
}

impl Default for DemoState {
    fn default() -> Self { Self::new() }
}

impl fhre::resources::Resource for DemoState {}

/// Handle to the shared rotation animation clip
#[derive(Clone, Copy, Debug)]
pub struct RotationClip {
    pub handle: AnimationClipHandle,
}

impl fhre::resources::Resource for RotationClip {}

/// Flag to track if animation has been initialized
#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationInitialized {
    pub done: bool,
}

impl fhre::resources::Resource for AnimationInitialized {}

/// Signal that a model switch was requested
#[derive(Clone, Copy, Debug, Default)]
pub struct ModelSwitchRequested {
    pub requested: bool,
}

impl fhre::resources::Resource for ModelSwitchRequested {}

/// Track previous rotation state to detect changes
#[derive(Clone, Copy, Debug, Default)]
pub struct LastRotationState {
    pub is_rotating: bool,
}

impl fhre::resources::Resource for LastRotationState {}

// ============================================================
// Color Presets
// ============================================================

mod colors {
    use fhre::math::Color;
    
    /// Cube face colors: Red, Green, Blue, Yellow, White, Cyan
    pub const CUBE_FACES: [Color; 6] = [
        Color::rgb(255, 100, 100),  // Red
        Color::rgb(100, 255, 100),  // Green
        Color::rgb(100, 100, 255),  // Blue
        Color::rgb(255, 255, 100),  // Yellow
        Color::rgb(255, 255, 255),  // White
        Color::rgb(100, 255, 255),  // Cyan
    ];
    
    /// Previous button: Steel Blue shades
    pub const BTN_PREV: (Color, Color, Color) = (
        Color::rgb(70, 130, 180),   // Normal
        Color::rgb(100, 160, 210),  // Hover
        Color::rgb(50, 100, 150),   // Pressed
    );
    
    /// Pause button: Green shades
    pub const BTN_PAUSE: (Color, Color, Color) = (
        Color::rgb(60, 150, 80),    // Normal
        Color::rgb(90, 180, 110),   // Hover
        Color::rgb(40, 120, 60),    // Pressed
    );
    
    /// Next button: Red shades
    pub const BTN_NEXT: (Color, Color, Color) = (
        Color::rgb(180, 70, 70),    // Normal
        Color::rgb(210, 100, 100),  // Hover
        Color::rgb(150, 50, 50),    // Pressed
    );
}

// ============================================================
// Systems
// ============================================================

/// Setup initial scene: 3D model and UI buttons
fn setup(mut commands: Commands, screen: Res<PrimaryScreen>) {
    let (width, height) = screen.dimensions();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    
    // Spawn initial 3D model (Cube)
    commands.spawn()
        .insert(Node::game_entity(NodeType::Empty))
        .insert(Transform3D::from_position(center_x, center_y, 0.0))
        .insert(Cube::new(config::MODEL_SIZE)
            .with_face_colors(colors::CUBE_FACES)
            .with_rotation(Vec3::new(config::CUBE_ROT_X, 0.0, config::CUBE_ROT_Z))
            .with_wireframe(true, Color::WHITE))
        .insert(AnimationPlayer::new())
        .insert(SyncToRenderWorld);
    
    // Spawn UI buttons
    let button_y = height as f32 - ui::BUTTON_BOTTOM_MARGIN;
    
    // Previous button (left)
    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform2D::from_position(center_x - ui::BUTTON_SPACING, button_y))
        .insert(Button::new(ui::BUTTON_WIDTH, ui::BUTTON_HEIGHT)
            .with_text("Prev")
            .with_colors(colors::BTN_PREV.0, colors::BTN_PREV.1, colors::BTN_PREV.2));
    
    // Pause button (center)
    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform2D::from_position(center_x, button_y))
        .insert(Button::new(ui::BUTTON_WIDTH, ui::BUTTON_HEIGHT)
            .with_text("Pause")
            .with_colors(colors::BTN_PAUSE.0, colors::BTN_PAUSE.1, colors::BTN_PAUSE.2));
    
    // Next button (right)
    commands.spawn()
        .insert(Node::ui_control(NodeType::Button))
        .insert(Transform2D::from_position(center_x + ui::BUTTON_SPACING, button_y))
        .insert(Button::new(ui::BUTTON_WIDTH, ui::BUTTON_HEIGHT)
            .with_text("Next")
            .with_colors(colors::BTN_NEXT.0, colors::BTN_NEXT.1, colors::BTN_NEXT.2));
}

/// Create and assign rotation animation clip to all players
fn setup_animation(
    mut anim_resources: ResMut<AnimationResources>,
    mut clip_res: ResMut<RotationClip>,
    mut players: Query<AnimationPlayer>,
    mut initialized: ResMut<AnimationInitialized>,
) {
    if initialized.done {
        return;
    }
    
    if players.len() == 0 {
        return;
    }
    
    let target_id = AnimationTargetId::new(anim::TARGET_ID);
    let duration = anim::DURATION;
    
    // Create animation clip with multiple property curves
    let mut clip = AnimationClip::with_duration(duration);
    {
        use alloc::vec;
        
        // Y-axis rotation: 0 -> 360 degrees (full rotation)
        clip.add_curve_to_target(
            target_id,
            AnimationProperty::RotationY,
            KeyframeCurve::new(vec![
                Keyframe::new(0.0, 0.0, Easing::Linear),
                Keyframe::new(duration, anim::ROTATION_RANGE, Easing::Linear),
            ]),
        );
        
        // X-axis rotation: wobble effect (30 -> -30 -> 30 degrees)
        clip.add_curve_to_target(
            target_id,
            AnimationProperty::RotationX,
            KeyframeCurve::new(vec![
                Keyframe::new(0.0, anim::WOBBLE_AMPLITUDE, Easing::EaseInOut),
                Keyframe::new(anim::DURATION_HALF, -anim::WOBBLE_AMPLITUDE, Easing::EaseInOut),
                Keyframe::new(duration, anim::WOBBLE_AMPLITUDE, Easing::EaseInOut),
            ]),
        );
        
        // Z-axis rotation: 45 -> 405 degrees (full rotation with offset)
        clip.add_curve_to_target(
            target_id,
            AnimationProperty::RotationZ,
            KeyframeCurve::new(vec![
                Keyframe::new(0.0, anim::Z_ROTATION_OFFSET, Easing::Linear),
                Keyframe::new(duration, anim::Z_ROTATION_OFFSET + anim::ROTATION_RANGE, Easing::Linear),
            ]),
        );
        
        // Scale X: pulsing effect (120 -> 160 -> 120)
        clip.add_curve_to_target(
            target_id,
            AnimationProperty::ScaleX,
            KeyframeCurve::new(vec![
                Keyframe::new(0.0, config::MODEL_SIZE, Easing::EaseInOut),
                Keyframe::new(anim::DURATION_HALF, config::MODEL_SIZE_MAX, Easing::EaseInOut),
                Keyframe::new(duration, config::MODEL_SIZE, Easing::EaseInOut),
            ]),
        );
    }
    
    let handle = anim_resources.insert_clip(clip);
    clip_res.handle = handle;
    
    // Assign animation to all existing players
    for (_entity, player) in players.iter_mut() {
        player.play_with_target(handle, target_id);
    }
    
    initialized.done = true;
}

/// Handle touch/keyboard input for button presses
fn input_system(
    key_input: Res<ButtonInput<KeyCode>>, 
    mouse_input: Res<ButtonInput<MouseButton>>,
    mouse_pos: Res<MousePosition>,
    mut state: ResMut<DemoState>,
    mut switch_requested: ResMut<ModelSwitchRequested>,
    screen: Res<PrimaryScreen>,
) {
    // Handle touch/click input
    if mouse_input.just_pressed(MouseButton::Left) {
        let (width, height) = screen.dimensions();
        let center_x = width as f32 / 2.0;
        let button_y = height as f32 - ui::BUTTON_BOTTOM_MARGIN;
        
        let mx = mouse_pos.x as f32;
        let my = mouse_pos.y as f32;
        
        // Check if touch is within button row
        if (my - button_y).abs() < ui::BUTTON_TOUCH_RADIUS {
            // Previous button (left)
            if (mx - (center_x - ui::BUTTON_SPACING)).abs() < ui::BUTTON_WIDTH / 2.0 {
                state.current_model = if state.current_model == 0 {
                    config::MODEL_COUNT - 1
                } else {
                    state.current_model - 1
                };
                switch_requested.requested = true;
            }
            // Pause button (center)
            else if (mx - center_x).abs() < ui::BUTTON_WIDTH / 2.0 {
                state.is_rotating = !state.is_rotating;
            }
            // Next button (right)
            else if (mx - (center_x + ui::BUTTON_SPACING)).abs() < ui::BUTTON_WIDTH / 2.0 {
                state.current_model = (state.current_model + 1) % config::MODEL_COUNT;
                switch_requested.requested = true;
            }
        }
    }
    
    // Handle keyboard input
    if key_input.just_pressed(KeyCode::Space) {
        state.is_rotating = !state.is_rotating;
    }
}

/// Control animation playback based on rotation state
fn animation_control_system(
    state: Res<DemoState>, 
    mut last_state: ResMut<LastRotationState>,
    mut players: Query<AnimationPlayer>
) {
    // Only act on state changes
    if state.is_rotating == last_state.is_rotating {
        return;
    }
    
    last_state.is_rotating = state.is_rotating;
    
    // Apply to all animation players
    for (_entity, player) in players.iter_mut() {
        if state.is_rotating {
            player.resume_all();
        } else {
            player.pause_all();
        }
    }
}

/// Handle model switching when requested
fn model_switch_system(
    mut state: ResMut<DemoState>,
    mut switch_requested: ResMut<ModelSwitchRequested>,
    mut last_state: ResMut<LastRotationState>,
    clip_res: Res<RotationClip>,
    screen: Res<PrimaryScreen>,
    mut commands: Commands,
    cube_query: Query<Cube>,
    soccer_query: Query<SoccerBall>,
) {
    if !switch_requested.requested {
        return;
    }
    switch_requested.requested = false;
    
    // Ensure animation plays after switch
    state.is_rotating = true;
    last_state.is_rotating = false;
    
    let (width, height) = screen.dimensions();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    
    // Despawn existing models
    for (entity, _) in cube_query.iter() {
        commands.despawn(entity);
    }
    for (entity, _) in soccer_query.iter() {
        commands.despawn(entity);
    }
    
    // Create animation player with clip pre-configured
    // Note: Must set animation before spawn because Commands are deferred
    let target_id = AnimationTargetId::new(anim::TARGET_ID);
    let mut player = AnimationPlayer::new();
    player.play_with_target(clip_res.handle, target_id);
    
    // Spawn new model based on current selection
    match state.current_model {
        0 => {
            commands.spawn()
                .insert(Node::game_entity(NodeType::Empty))
                .insert(Transform3D::from_position(center_x, center_y, 0.0))
                .insert(Cube::new(config::MODEL_SIZE)
                    .with_face_colors(colors::CUBE_FACES)
                    .with_rotation(Vec3::new(config::CUBE_ROT_X, 0.0, config::CUBE_ROT_Z))
                    .with_wireframe(true, Color::WHITE))
                .insert(player)
                .insert(SyncToRenderWorld);
        }
        1 => {
            commands.spawn()
                .insert(Node::game_entity(NodeType::Empty))
                .insert(Transform3D::from_position(center_x, center_y, 0.0))
                .insert(SoccerBall::new(config::MODEL_SIZE)
                    .with_rotation(Vec3::new(0.0, 0.0, 0.0))
                    .with_wireframe(true, Color::WHITE))
                .insert(player)
                .insert(SyncToRenderWorld);
        }
        _ => {}
    }
}

// ============================================================
// Main Entry Point
// ============================================================

#[no_mangle]
pub extern "C" fn fhre_rust_main() -> i32 {
    // Initialize platform window
    let mut window = match framebuffer::Window::new() {
        Some(w) => w,
        None => return 0,
    };
    
    let (width, height) = window.dimensions();
    
    // Create application with default plugins
    let mut app = App::new(width, height);
    
    app.add_plugins(DefaultPlugins)
        // Resources
        .insert_resource(DemoState::new())
        .insert_resource(RotationClip { handle: AnimationClipHandle::null() })
        .insert_resource(AnimationInitialized::default())
        .insert_resource(ModelSwitchRequested::default())
        .insert_resource(LastRotationState::default())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(MousePosition::default())
        // Systems
        .add_systems(Startup, declare_system!(setup; Commands, Res<PrimaryScreen>))
        .add_systems(Update, declare_system!(setup_animation; ResMut<AnimationResources>, ResMut<RotationClip>, Query<AnimationPlayer>, ResMut<AnimationInitialized>))
        .add_systems(PreUpdate, declare_system!(input_system; Res<ButtonInput<KeyCode>>, Res<ButtonInput<MouseButton>>, Res<MousePosition>, ResMut<DemoState>, ResMut<ModelSwitchRequested>, Res<PrimaryScreen>))
        .add_systems(Update, declare_system!(animation_control_system; Res<DemoState>, ResMut<LastRotationState>, Query<AnimationPlayer>))
        .add_systems(Update, declare_system!(model_switch_system; ResMut<DemoState>, ResMut<ModelSwitchRequested>, ResMut<LastRotationState>, Res<RotationClip>, Res<PrimaryScreen>, Commands, Query<Cube>, Query<SoccerBall>));
    
    // Extractors for render world sync
    app.add_extractor(extract::extract_view)
       .add_extractor(extract::extract_3d_components)
       .add_extractor(extract::extract_buttons)
       .add_extractor(extract::queue_meshes)
       .add_extractor(extract::queue_ui);
    
    // Animation application systems
    app.add_systems(Update, fhre::declare_system!(apply_animations::<Cube>; Query<fhre::animation::AnimationPlayer>, Query<Cube>));
    app.add_systems(Update, fhre::declare_system!(apply_animations::<SoccerBall>; Query<fhre::animation::AnimationPlayer>, Query<SoccerBall>));
    
    // Run with input adapter
    let input_adapter = framebuffer::InputAdapter::default();
    
    WindowRunner::new(&mut app, &mut window, &input_adapter)
        .with_frame_delay_ms(timing::FRAME_DELAY_MS)
        .run();
    
    0
}

/// NuttX module initialization
#[no_mangle]
pub extern "C" fn rust_fhre_demo_init() {}
