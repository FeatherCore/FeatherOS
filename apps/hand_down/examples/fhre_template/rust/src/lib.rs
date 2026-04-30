//! FHRE 应用模板
//!
//! 展示如何使用 FHRE 引擎创建应用：
//! 1. 平台适配: 实现 Window trait
//! 2. 输入桥接: 实现 InputBridge trait
//! 3. 提取器: 实现 ExtractComponent
//! 4. 组件: 实现 AnimationReceiver
//! 5. 游戏逻辑: 使用 ECS 系统

#![no_std]
#![no_main]

mod components;
mod extract;
mod platform;

extern crate alloc;

use fhre::{
    App, DefaultPlugins,
    node::{Node, Transform},
    math::{Color, Vec2, Vec3},
    resources::PrimaryScreen,
    window::MousePosition,
    Res, ResMut, Query, Commands, Entity,
    Update, PreUpdate, Startup,
    declare_system,
    Pickable, PickableBounds, HoverMap, PreviousHoverMap,
    ui_picking_backend, update_hover_map, PointerHitsBuffer,
    PointerId, PointerPress, PointerLocation,
    Events,
    asset::{Assets, Handle, Image, TextureAssetPlugin},
    pipeline::Sampler,
    animation::{AnimationClip, AnimationClipHandle, AnimationPlayer, AnimationResources, AnimationTargetId, AnimationProperty, KeyframeCurve, Keyframe, Easing, apply_animations},
    SyncToRenderWorld,
};
use platform::runner::PlatformInputPlugin;
use components::MyModel;

// ============================================================
// 配置常量
// ============================================================

const FRAME_DELAY_MS: u32 = 16;

// ============================================================
// 资源定义
// ============================================================

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub running: bool,
}

impl fhre::resources::Resource for AppState {}

// ============================================================
// 系统函数
// ============================================================

fn setup(mut commands: Commands, screen: Res<PrimaryScreen>) {
    let (width, height) = screen.dimensions();
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    
    commands.spawn()
        .insert(Node::game_entity())
        .insert(Transform::from_position(center_x, center_y, 0.0))
        .insert(MyModel::new(100.0))
        .insert(AnimationPlayer::new())
        .insert(SyncToRenderWorld);
}

fn setup_animation(
    mut anim_resources: ResMut<AnimationResources>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if players.len() == 0 {
        return;
    }
    
    let target_id = AnimationTargetId::new(1);
    let duration = 4.0;
    
    let mut clip = AnimationClip::with_duration(duration);
    clip.add_curve_to_target(
        target_id,
        AnimationProperty::RotationY,
        KeyframeCurve::new(alloc::vec![
            Keyframe::new(0.0, 0.0, Easing::Linear),
            Keyframe::new(duration, 360.0, Easing::Linear),
        ]),
    );
    
    let handle = anim_resources.insert_clip(clip);
    
    for (_entity, player) in players.iter_mut() {
        player.play_with_target(handle, target_id);
    }
}

fn picking_system(
    mouse_pos: Res<MousePosition>,
    mut transform_query: Query<&Transform>,
    mut bounds_query: Query<&PickableBounds>,
    mut pickable_query: Query<&Pickable>,
    mut hover_map: ResMut<HoverMap>,
    mut prev_hover_map: ResMut<PreviousHoverMap>,
    mut pointer_press: ResMut<PointerPress>,
    mut pointer_location: ResMut<PointerLocation>,
) {
    pointer_location.position = Vec2::new(mouse_pos.x as f32, mouse_pos.y as f32);
    pointer_press.primary = false;
    
    let pointers: [(PointerId, f32, f32, bool); 1] = [
        (PointerId::Mouse, mouse_pos.x as f32, mouse_pos.y as f32, false),
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
        pickable_query.iter().map(|(e, p)| (e, p)).collect();
    
    update_hover_map(buffer.hits(), &pickable_data, &mut hover_map, &mut prev_hover_map);
}

// ============================================================
// 主入口
// ============================================================

#[no_mangle]
pub extern "C" fn fhre_template_main() -> i32 {
    let mut window = match platform::framebuffer::Window::new() {
        Some(w) => w,
        None => return 0,
    };
    
    let (width, height) = window.dimensions();
    
    let mut app = App::new(width, height);
    
    app.add_plugins(DefaultPlugins)
        .add_plugin(TextureAssetPlugin)
        .insert_resource(AppState::default())
        .insert_resource(fhre::window::MousePosition::default())
        .add_systems(Startup, declare_system!(setup; Commands, Res<PrimaryScreen>))
        .add_systems(Update, declare_system!(setup_animation; ResMut<AnimationResources>, Query<&mut AnimationPlayer>))
        .add_systems(PreUpdate, declare_system!(picking_system; Res<MousePosition>, Query<&Transform>, Query<&PickableBounds>, Query<&Pickable>, ResMut<HoverMap>, ResMut<PreviousHoverMap>, ResMut<PointerPress>, ResMut<PointerLocation>))
        .add_systems(Update, fhre::declare_system!(apply_animations::<MyModel>; Query<&fhre::animation::AnimationPlayer>, Query<&mut MyModel>));
    
    app.add_extractor(extract::extract_view)
        .add_extractor(extract::extract_models)
        .add_extractor(extract::queue_meshes);
    
    let input_plugin = PlatformInputPlugin::new(platform::framebuffer::InputAdapter);
    app.run(&mut window, &input_plugin, FRAME_DELAY_MS);
    
    0
}

#[no_mangle]
pub extern "C" fn rust_fhre_template_init() {}
