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
use platform::input::{ButtonInput, KeyCode, MouseButton};
use platform::runner::PlatformInputPlugin;

use fhre::{
    App,
    node::{Node, Transform, Transform3D},
    math::{Color, Vec2, Vec3},
    resources::PrimaryScreen,
    window::MousePosition,
    Res, ResMut, Query, Commands, Entity,
    Update, PreUpdate, Startup,
    DefaultPlugins,
    SyncToRenderWorld,
    declare_system,
    Pickable, PickableBounds, HoverMap, PreviousHoverMap,
    ui_picking_backend, update_hover_map, PointerHitsBuffer,
    PointerId, PointerPress, PointerLocation, PointerInput, PointerAction, PointerButton,
    Pointer, Over, Out, Press, Release, Click,
    Events, EventReader,
    asset::{Asset, Assets, Handle, Image, GpuTextures, TextureAssetPlugin},
    pipeline::{Texture, Sampler},
};

use components::{Button, ButtonState, Cube, SoccerBall};
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

/// Texture handles for cube faces (declarative asset approach)
#[derive(Clone, Debug)]
pub struct CubeTextures {
    pub front: Handle<Image>,
    pub back: Handle<Image>,
    pub top: Handle<Image>,
    pub bottom: Handle<Image>,
    pub left: Handle<Image>,
    pub right: Handle<Image>,
}

impl fhre::resources::Resource for CubeTextures {}

/// Texture handles for soccer ball faces (declarative asset approach)
#[derive(Clone, Debug)]
pub struct SoccerBallTextures {
    pub pentagon: Handle<Image>,
    pub hexagon: Handle<Image>,
}

impl fhre::resources::Resource for SoccerBallTextures {}

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

fn create_checker_texture(width: u32, height: u32, color1: Color, color2: u32) -> Image {
    let mut data = alloc::vec![0u8; (width * height * 4) as usize];
    
    for y in 0..height {
        for x in 0..width {
            let checker = ((x / 8) + (y / 8)) % 2;
            let color = if checker == 0 { color1 } else { Color::from_u32(color2) };
            
            let idx = ((y * width + x) * 4) as usize;
            data[idx] = color.r;
            data[idx + 1] = color.g;
            data[idx + 2] = color.b;
            data[idx + 3] = color.a;
        }
    }
    
    Image::from_rgba32(width, height, data).with_sampler(Sampler::NEAREST)
}

fn create_gradient_texture(width: u32, height: u32, top_color: Color, bottom_color: Color) -> Image {
    let mut data = alloc::vec![0u8; (width * height * 4) as usize];
    
    for y in 0..height {
        let t = y as f32 / height as f32;
        let color = Color::lerp(top_color, bottom_color, t);
        
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            data[idx] = color.r;
            data[idx + 1] = color.g;
            data[idx + 2] = color.b;
            data[idx + 3] = color.a;
        }
    }
    
    Image::from_rgba32(width, height, data).with_sampler(Sampler::LINEAR)
}

fn create_solid_texture(width: u32, height: u32, color: Color) -> Image {
    let mut data = alloc::vec![0u8; (width * height * 4) as usize];
    
    for i in 0..(width * height) {
        let idx = (i * 4) as usize;
        data[idx] = color.r;
        data[idx + 1] = color.g;
        data[idx + 2] = color.b;
        data[idx + 3] = color.a;
    }
    
    Image::from_rgba32(width, height, data)
}

fn create_pentagon_texture(width: u32, height: u32, color: Color) -> Image {
    let mut data = alloc::vec![0u8; (width * height * 4) as usize];
    
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let radius = width as f32 / 2.0 - 1.0;
    
    for y in 0..height {
        for x in 0..width {
            let px = x as f32 - cx;
            let py = y as f32 - cy;
            
            let angle = libm::atan2f(py, px);
            let normalized_angle = if angle < 0.0 { angle + 6.283185 } else { angle };
            
            let dist = libm::sqrtf(px * px + py * py);
            
            let angle_step = 6.283185 / 5.0;
            let half_step = angle_step / 2.0;
            
            let segment = libm::floorf(normalized_angle / angle_step) as i32;
            let segment_start = segment as f32 * angle_step - half_step;
            let segment_end = segment_start + angle_step;
            
            let mid_angle = (segment_start + segment_end) / 2.0;
            
            let edge_dist = radius * libm::cosf(angle_step / 2.0) / libm::cosf(normalized_angle - mid_angle);
            
            let inside = dist <= edge_dist;
            
            let idx = ((y * width + x) * 4) as usize;
            if inside {
                data[idx] = color.r;
                data[idx + 1] = color.g;
                data[idx + 2] = color.b;
                data[idx + 3] = color.a;
            } else {
                data[idx] = 0;
                data[idx + 1] = 0;
                data[idx + 2] = 0;
                data[idx + 3] = 0;
            }
        }
    }
    
    Image::from_rgba32(width, height, data).with_sampler(Sampler::LINEAR)
}

fn create_hexagon_texture(width: u32, height: u32, color: Color) -> Image {
    let mut data = alloc::vec![0u8; (width * height * 4) as usize];
    
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let radius = width as f32 / 2.0 - 1.0;
    
    for y in 0..height {
        for x in 0..width {
            let px = x as f32 - cx;
            let py = y as f32 - cy;
            
            let angle = libm::atan2f(py, px);
            let normalized_angle = if angle < 0.0 { angle + 6.283185 } else { angle };
            
            let dist = libm::sqrtf(px * px + py * py);
            
            let angle_step = 6.283185 / 6.0;
            
            let segment = libm::floorf(normalized_angle / angle_step) as i32;
            let segment_start = segment as f32 * angle_step;
            let segment_end = segment_start + angle_step;
            
            let mid_angle = (segment_start + segment_end) / 2.0;
            
            let edge_dist = radius * libm::cosf(angle_step / 2.0) / libm::cosf(normalized_angle - mid_angle);
            
            let inside = dist <= edge_dist;
            
            let idx = ((y * width + x) * 4) as usize;
            if inside {
                data[idx] = color.r;
                data[idx + 1] = color.g;
                data[idx + 2] = color.b;
                data[idx + 3] = color.a;
            } else {
                data[idx] = 0;
                data[idx + 1] = 0;
                data[idx + 2] = 0;
                data[idx + 3] = 0;
            }
        }
    }
    
    Image::from_rgba32(width, height, data).with_sampler(Sampler::LINEAR)
}

/// Setup initial scene: 3D model and UI buttons
fn setup(mut commands: Commands, screen: Res<PrimaryScreen>, cube_textures: Res<CubeTextures>) {
    let (width, height) = screen.dimensions();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    
    commands.spawn()
        .insert(Node::game_entity())
        .insert(Transform3D::from_position(center_x, center_y, 0.0))
        .insert(Cube::new(config::MODEL_SIZE)
            .with_face_colors(colors::CUBE_FACES)
            .with_face_textures_handles([
                cube_textures.front.clone(),
                cube_textures.back.clone(),
                cube_textures.top.clone(),
                cube_textures.bottom.clone(),
                cube_textures.left.clone(),
                cube_textures.right.clone(),
            ])
            .with_rotation(Vec3::new(config::CUBE_ROT_X, 0.0, config::CUBE_ROT_Z))
            .with_wireframe(true, Color::WHITE))
        .insert(AnimationPlayer::new())
        .insert(SyncToRenderWorld);

    // TEMP: debug crash path by skipping Cube insertion
    // commands.spawn()
    //     .insert(Node::game_entity())
    //     .insert(Transform3D::from_position(center_x, center_y, 0.0))
    //     .insert(Cube::new(config::MODEL_SIZE)
    //         .with_face_colors(colors::CUBE_FACES)
    //         .with_face_textures_handles([
    //             cube_textures.front.clone(),
    //             cube_textures.back.clone(),
    //             cube_textures.top.clone(),
    //             cube_textures.bottom.clone(),
    //             cube_textures.left.clone(),
    //             cube_textures.right.clone(),
    //         ])
    //         .with_rotation(Vec3::new(config::CUBE_ROT_X, 0.0, config::CUBE_ROT_Z))
    //         .with_wireframe(true, Color::WHITE))
    //     .insert(AnimationPlayer::new())
    //     .insert(SyncToRenderWorld);
    
    let button_y = height as f32 - ui::BUTTON_BOTTOM_MARGIN;
    
    commands.spawn()
        .insert(Node::ui_control())
        .insert(Transform::from_2d(center_x - ui::BUTTON_SPACING, button_y))
        .insert(Button::new(ui::BUTTON_WIDTH, ui::BUTTON_HEIGHT)
            .with_text("Prev")
            .with_colors(colors::BTN_PREV.0, colors::BTN_PREV.1, colors::BTN_PREV.2))
        .insert(PickableBounds::from_size(ui::BUTTON_WIDTH, ui::BUTTON_HEIGHT))
        .insert(Pickable::DEFAULT);
    
    commands.spawn()
        .insert(Node::ui_control())
        .insert(Transform::from_2d(center_x, button_y))
        .insert(Button::new(ui::BUTTON_WIDTH, ui::BUTTON_HEIGHT)
            .with_text("Pause")
            .with_colors(colors::BTN_PAUSE.0, colors::BTN_PAUSE.1, colors::BTN_PAUSE.2))
        .insert(PickableBounds::from_size(ui::BUTTON_WIDTH, ui::BUTTON_HEIGHT))
        .insert(Pickable::DEFAULT);
    
    commands.spawn()
        .insert(Node::ui_control())
        .insert(Transform::from_2d(center_x + ui::BUTTON_SPACING, button_y))
        .insert(Button::new(ui::BUTTON_WIDTH, ui::BUTTON_HEIGHT)
            .with_text("Next")
            .with_colors(colors::BTN_NEXT.0, colors::BTN_NEXT.1, colors::BTN_NEXT.2))
        .insert(PickableBounds::from_size(ui::BUTTON_WIDTH, ui::BUTTON_HEIGHT))
        .insert(Pickable::DEFAULT);
    
    {
        extern "C" { fn printf(format: *const u8, ...) -> i32; }
        unsafe { printf(b"[setup] done\n\0".as_ptr()); }
    }
}

/// Setup textures declaratively using Assets<Image> (Bevy-style)
fn setup_textures(
    mut images: ResMut<Assets<Image>>,
    mut cube_textures: ResMut<CubeTextures>,
    mut soccer_textures: ResMut<SoccerBallTextures>,
    mut events: ResMut<Events>,
) {
    let tex_size = 64u32;
    
    cube_textures.front = images.add_with_event(create_checker_texture(tex_size, tex_size, Color::rgb(255, 100, 100), 0xFF404040), &mut events);
    cube_textures.back = images.add_with_event(create_checker_texture(tex_size, tex_size, Color::rgb(100, 255, 100), 0xFF404040), &mut events);
    cube_textures.top = images.add_with_event(create_gradient_texture(tex_size, tex_size, Color::rgb(100, 100, 255), Color::rgb(200, 200, 255)), &mut events);
    cube_textures.bottom = images.add_with_event(create_gradient_texture(tex_size, tex_size, Color::rgb(255, 255, 100), Color::rgb(255, 200, 50)), &mut events);
    cube_textures.left = images.add_with_event(create_solid_texture(tex_size, tex_size, Color::rgb(255, 100, 255)), &mut events);
    cube_textures.right = images.add_with_event(create_solid_texture(tex_size, tex_size, Color::rgb(100, 255, 255)), &mut events);
    
    soccer_textures.pentagon = images.add_with_event(create_pentagon_texture(tex_size, tex_size, Color::BLACK), &mut events);
    soccer_textures.hexagon = images.add_with_event(create_hexagon_texture(tex_size, tex_size, Color::WHITE), &mut events);
}

/// Create and assign rotation animation clip to all players
fn setup_animation(
    mut anim_resources: ResMut<AnimationResources>,
    mut clip_res: ResMut<RotationClip>,
    mut players: Query<&mut AnimationPlayer>,
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

const MOUSE_POINTER_ID: PointerId = PointerId::Mouse;

fn picking_system(
    mouse_pos: Res<MousePosition>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut transform_query: Query<&Transform>,
    mut bounds_query: Query<&PickableBounds>,
    mut pickable_query: Query<&Pickable>,
    mut hover_map: ResMut<HoverMap>,
    mut prev_hover_map: ResMut<PreviousHoverMap>,
    mut pointer_press: ResMut<PointerPress>,
    mut pointer_location: ResMut<PointerLocation>,
    mut events: ResMut<Events>,
) {
    pointer_location.position = Vec2::new(mouse_pos.x as f32, mouse_pos.y as f32);
    
    let prev_press = *pointer_press;
    pointer_press.primary = mouse_input.pressed(MouseButton::Left);
    
    let pointers: [(PointerId, f32, f32, bool); 1] = [
        (MOUSE_POINTER_ID, mouse_pos.x as f32, mouse_pos.y as f32, pointer_press.primary),
    ];
    
    let mut pickables: alloc::vec::Vec<(Entity, Transform, PickableBounds, Option<Pickable>)> = 
        alloc::vec::Vec::new();
    
    for (entity, transform) in transform_query.iter() {
        if let Some((_, bounds)) = bounds_query.get_pair(entity) {
            let pickable = pickable_query.get(entity).copied();
            pickables.push((entity, *transform, *bounds, pickable));
        }
    }
    
    let hits = ui_picking_backend(&pointers, &pickables);
    
    let mut buffer = PointerHitsBuffer::new();
    for hit in hits {
        buffer.push(hit);
    }
    
    let pickable_data: alloc::vec::Vec<(Entity, Pickable)> = 
        pickable_query.iter()
            .map(|(entity, pickable)| (entity, *pickable))
            .collect();
    
    update_hover_map(buffer.hits(), &pickable_data, &mut hover_map, &mut prev_hover_map);
    
    fhre::picking::pointer_events(&mut events, &hover_map, &prev_hover_map, &pointer_press, &prev_press, &pointer_location);
}

fn button_interaction_system(
    events: Res<Events>,
    mut button_query: Query<&mut Button>,
) {
    if let Some(click_events) = events.get_events_current::<Pointer<Click>>() {
        for event in click_events {
            if let Some((_, button)) = button_query.get_pair_mut(event.entity) {
                button.clicked = true;
                button.state = ButtonState::Hover;
            }
        }
    }
    
    if let Some(over_events) = events.get_events_current::<Pointer<Over>>() {
        for event in over_events {
            if let Some((_, button)) = button_query.get_pair_mut(event.entity) {
                button.state = ButtonState::Hover;
            }
        }
    }
    
    if let Some(out_events) = events.get_events_current::<Pointer<Out>>() {
        for event in out_events {
            if let Some((_, button)) = button_query.get_pair_mut(event.entity) {
                button.state = ButtonState::Normal;
            }
        }
    }
    
    if let Some(press_events) = events.get_events_current::<Pointer<Press>>() {
        for event in press_events {
            if let Some((_, button)) = button_query.get_pair_mut(event.entity) {
                button.state = ButtonState::Pressed;
            }
        }
    }
}

fn input_system(
    key_input: Res<ButtonInput<KeyCode>>, 
    mut state: ResMut<DemoState>,
    mut switch_requested: ResMut<ModelSwitchRequested>,
    mut button_query: Query<&mut Button>,
) {
    for (_, button) in button_query.iter_mut() {
        if button.clicked {
            button.clicked = false;
            match button.text {
                "Prev" => {
                    state.current_model = if state.current_model == 0 {
                        config::MODEL_COUNT - 1
                    } else {
                        state.current_model - 1
                    };
                    switch_requested.requested = true;
                }
                "Pause" => {
                    state.is_rotating = !state.is_rotating;
                }
                "Next" => {
                    state.current_model = (state.current_model + 1) % config::MODEL_COUNT;
                    switch_requested.requested = true;
                }
                _ => {}
            }
        }
    }
    
    if key_input.just_pressed(KeyCode::Space) {
        state.is_rotating = !state.is_rotating;
    }
}

/// Control animation playback based on rotation state
fn animation_control_system(
    state: Res<DemoState>, 
    mut last_state: ResMut<LastRotationState>,
    mut players: Query<&mut AnimationPlayer>
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

/// Example system using change detection with Mut<T> and Ref<T>
/// This demonstrates Bevy-style change detection in queries.
#[allow(dead_code)]
fn change_detection_example_system(
    mut cube_query: Query<&mut Cube>,
    mut soccer_query: Query<&SoccerBall>,
) {
    for (_entity, cube) in cube_query.iter_mut() {
        // Note: is_added() and is_changed() require Mut<T> wrapper
        // This example shows the query pattern; change detection would need Mut<T>
        let _ = cube.rotation.y;
    }
    
    for (_entity, soccer) in soccer_query.iter() {
        let _ = soccer.rotation.y;
    }
}

/// Handle model switching when requested
fn model_switch_system(
    mut state: ResMut<DemoState>,
    mut switch_requested: ResMut<ModelSwitchRequested>,
    mut last_state: ResMut<LastRotationState>,
    clip_res: Res<RotationClip>,
    screen: Res<PrimaryScreen>,
    cube_textures: Res<CubeTextures>,
    soccer_textures: Res<SoccerBallTextures>,
    mut commands: Commands,
    mut cube_query: Query<&Cube>,
    mut soccer_query: Query<&SoccerBall>,
) {
    if !switch_requested.requested {
        return;
    }
    switch_requested.requested = false;
    
    state.is_rotating = true;
    last_state.is_rotating = false;
    
    let (width, height) = screen.dimensions();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    
    for (entity, _) in cube_query.iter() {
        commands.despawn(entity);
    }
    for (entity, _) in soccer_query.iter() {
        commands.despawn(entity);
    }
    
    let target_id = AnimationTargetId::new(anim::TARGET_ID);
    let mut player = AnimationPlayer::new();
    player.play_with_target(clip_res.handle, target_id);
    
    match state.current_model {
        0 => {
            commands.spawn()
                .insert(Node::game_entity())
                .insert(Transform3D::from_position(center_x, center_y, 0.0))
                .insert(Cube::new(config::MODEL_SIZE)
                    .with_face_colors(colors::CUBE_FACES)
                    .with_face_textures_handles([
                        cube_textures.front.clone(),
                        cube_textures.back.clone(),
                        cube_textures.top.clone(),
                        cube_textures.bottom.clone(),
                        cube_textures.left.clone(),
                        cube_textures.right.clone(),
                    ])
                    .with_rotation(Vec3::new(config::CUBE_ROT_X, 0.0, config::CUBE_ROT_Z))
                    .with_wireframe(true, Color::WHITE))
                .insert(player)
                .insert(SyncToRenderWorld);
        }
        1 => {
            let mut pentagon_textures: [Option<Handle<Image>>; 12] = Default::default();
            for i in 0..12 {
                pentagon_textures[i] = Some(soccer_textures.pentagon.clone());
            }
            let mut hexagon_textures: [Option<Handle<Image>>; 20] = Default::default();
            for i in 0..20 {
                hexagon_textures[i] = Some(soccer_textures.hexagon.clone());
            }
            
            commands.spawn()
                .insert(Node::game_entity())
                .insert(Transform3D::from_position(center_x, center_y, 0.0))
                .insert(SoccerBall::new(config::MODEL_SIZE)
                    .with_pentagon_textures_handles(pentagon_textures)
                    .with_hexagon_textures_handles(hexagon_textures)
                    .with_rotation(Vec3::new(config::CUBE_ROT_X, 0.0, config::CUBE_ROT_Z))
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
    let mut window = match framebuffer::Window::new() {
        Some(w) => w,
        None => return 0,
    };
    
    let (width, height) = window.dimensions();
    
    let mut app = App::new(width, height);
    
    app.add_plugins(DefaultPlugins)
        .add_plugin(TextureAssetPlugin)
        .insert_resource(DemoState::new())
        .insert_resource(RotationClip { handle: AnimationClipHandle::null() })
        .insert_resource(AnimationInitialized::default())
        .insert_resource(ModelSwitchRequested::default())
        .insert_resource(LastRotationState::default())
        .insert_resource(ButtonInput::<KeyCode>::default())
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(MousePosition::default())
        .insert_resource(CubeTextures {
            front: Handle::default(),
            back: Handle::default(),
            top: Handle::default(),
            bottom: Handle::default(),
            left: Handle::default(),
            right: Handle::default(),
        })
        .insert_resource(SoccerBallTextures {
            pentagon: Handle::default(),
            hexagon: Handle::default(),
        })
        .add_systems(Startup, declare_system!(setup_textures; ResMut<Assets<Image>>, ResMut<CubeTextures>, ResMut<SoccerBallTextures>, ResMut<Events>))
        .add_systems(Startup, declare_system!(setup; Commands, Res<PrimaryScreen>, Res<CubeTextures>))
        .add_systems(Update, declare_system!(setup_animation; ResMut<AnimationResources>, ResMut<RotationClip>, Query<&mut AnimationPlayer>, ResMut<AnimationInitialized>))
        .add_systems(PreUpdate, declare_system!(picking_system; Res<MousePosition>, Res<ButtonInput<MouseButton>>, Query<&Transform>, Query<&PickableBounds>, Query<&Pickable>, ResMut<HoverMap>, ResMut<PreviousHoverMap>, ResMut<PointerPress>, ResMut<PointerLocation>, ResMut<Events>))
        .add_systems(PreUpdate, declare_system!(button_interaction_system; Res<Events>, Query<&mut Button>))
        .add_systems(PreUpdate, declare_system!(input_system; Res<ButtonInput<KeyCode>>, ResMut<DemoState>, ResMut<ModelSwitchRequested>, Query<&mut Button>))
        .add_systems(Update, declare_system!(animation_control_system; Res<DemoState>, ResMut<LastRotationState>, Query<&mut AnimationPlayer>))
        .add_systems(Update, declare_system!(model_switch_system; ResMut<DemoState>, ResMut<ModelSwitchRequested>, ResMut<LastRotationState>, Res<RotationClip>, Res<PrimaryScreen>, Res<CubeTextures>, Res<SoccerBallTextures>, Commands, Query<&Cube>, Query<&SoccerBall>));
    
    {
        extern "C" { fn printf(format: *const u8, ...) -> i32; }
        unsafe { printf(b"[main] plugins added\n\0".as_ptr()); }
    }
    
    app.add_extractor(extract::extract_view)
       .add_extractor(extract::extract_3d_components)
       .add_extractor(extract::extract_buttons)
       .add_extractor(extract::queue_meshes)
       .add_extractor(extract::queue_ui);
    
    app.add_systems(Update, fhre::declare_system!(apply_animations::<Cube>; Query<&fhre::animation::AnimationPlayer>, Query<&mut Cube>));
    app.add_systems(Update, fhre::declare_system!(apply_animations::<SoccerBall>; Query<&fhre::animation::AnimationPlayer>, Query<&mut SoccerBall>));
    
    let input_plugin = PlatformInputPlugin::new(framebuffer::InputAdapter);
    
    app.run(&mut window, &input_plugin, timing::FRAME_DELAY_MS);
    
    0
}

/// NuttX module initialization
#[no_mangle]
pub extern "C" fn rust_fhre_demo_init() {}
