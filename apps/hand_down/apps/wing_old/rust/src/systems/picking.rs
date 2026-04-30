use alloc::vec::Vec;

use fhre::{
    update_hover_map, ui_picking_backend, Entity, Events, HoverMap, MousePosition, Pickable,
    PickableBounds, PointerLocation, PointerPress, PreviousHoverMap, Query, Res, ResMut,
    Transform,
};

use crate::{ButtonInput, MouseButton};

pub fn wing_picking_system(
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
    let mouse = (mouse_pos.x as f32, mouse_pos.y as f32);
    let prev_press = *pointer_press;

    pointer_press.primary = mouse_input.pressed(MouseButton::Left);
    pointer_location.position = fhre::Vec2::new(mouse.0, mouse.1);

    let pointers = [(fhre::PointerId::Mouse, mouse.0, mouse.1, pointer_press.primary)];

    let mut entities: Vec<(Entity, Transform, PickableBounds, Option<Pickable>)> = Vec::new();
    for (entity, transform) in transform_query.iter_mut() {
        if let Some((_, bounds)) = bounds_query.get_pair_mut(entity) {
            let pickable = pickable_query.get_pair_mut(entity).map(|(_, p)| *p);
            entities.push((entity, *transform, *bounds, pickable));
        }
    }

    let hits = ui_picking_backend(&pointers, &entities);
    let pickable_data: Vec<(Entity, Pickable)> = entities
        .iter()
        .filter_map(|(entity, _, _, pickable)| pickable.map(|p| (*entity, p)))
        .collect();

    update_hover_map(&hits, &pickable_data, &mut hover_map, &mut prev_hover_map);

    fhre::picking::pointer_events(
        &mut events,
        &hover_map,
        &prev_hover_map,
        &pointer_press,
        &prev_press,
        &pointer_location,
    );
}
