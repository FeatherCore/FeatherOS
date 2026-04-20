//! Picking System

use alloc::vec::Vec;
use crate::Entity;
use crate::math::Vec2;
use crate::node::Transform;
use crate::event::Events;
use super::{
    Pickable, PickableBounds, PointerHits, HitData, HoverMap, PreviousHoverMap,
    PointerId, PointerButton, PointerPress, PointerLocation,
    Pointer, Over, Out, Press, Release, Click, Move,
};

pub fn update_hover_map(
    hits: &[PointerHits],
    pickables: &[(Entity, Pickable)],
    hover_map: &mut HoverMap,
    prev_hover_map: &mut PreviousHoverMap,
) {
    prev_hover_map.0.clear();
    core::mem::swap(&mut prev_hover_map.0, &mut hover_map.0);
    
    let mut sorted_hits: Vec<_> = hits.iter().collect();
    sorted_hits.sort_by(|a, b| b.order.partial_cmp(&a.order).unwrap_or(core::cmp::Ordering::Equal));
    
    for hit in sorted_hits {
        let pickable = pickables.iter()
            .find(|(e, _)| *e == hit.entity)
            .map(|(_, p)| p);
        
        let should_add = match pickable {
            Some(p) => p.is_hoverable,
            None => true,
        };
        
        if should_add {
            hover_map.insert(hit.pointer, (hit.entity, hit.hit));
        }
        
        let should_block = match pickable {
            Some(p) => p.should_block_lower,
            None => true,
        };
        
        if should_block {
            break;
        }
    }
}

pub fn ui_picking_backend(
    pointers: &[(PointerId, f32, f32, bool)],
    entities: &[(Entity, Transform, PickableBounds, Option<Pickable>)],
) -> Vec<PointerHits> {
    let mut hits = Vec::new();
    
    for (pointer_id, x, y, _pressed) in pointers {
        let point = Vec2::new(*x, *y);
        
        for (entity, transform, bounds, _pickable) in entities {
            let pos = Vec2::new(transform.position.x, transform.position.y);
            
            if bounds.contains_point(pos, point) {
                hits.push(PointerHits {
                    pointer: *pointer_id,
                    entity: *entity,
                    hit: HitData {
                        position: point,
                        depth: transform.position.z,
                    },
                    order: 0.0,
                });
            }
        }
    }
    
    hits
}

pub fn pointer_events(
    events: &mut Events,
    hover_map: &HoverMap,
    prev_hover_map: &PreviousHoverMap,
    pointer_press: &PointerPress,
    prev_pointer_press: &PointerPress,
    pointer_location: &PointerLocation,
) {
    let pointer_id = PointerId::Mouse;
    
    let current_hit = hover_map.get(&pointer_id);
    let prev_hit = prev_hover_map.0.get(&pointer_id);
    
    match (prev_hit, current_hit) {
        (Some((prev_entity, prev_hit_data)), Some((curr_entity, curr_hit_data))) => {
            if *prev_entity != *curr_entity {
                events.send(Pointer::new(
                    pointer_id,
                    PointerLocation { position: prev_hit_data.position },
                    Out { hit: *prev_hit_data },
                    *prev_entity,
                ));
                events.send(Pointer::new(
                    pointer_id,
                    PointerLocation { position: curr_hit_data.position },
                    Over { hit: *curr_hit_data },
                    *curr_entity,
                ));
            } else {
                events.send(Pointer::new(
                    pointer_id,
                    *pointer_location,
                    Move { hit: *curr_hit_data, delta: Vec2::ZERO },
                    *curr_entity,
                ));
            }
        }
        (Some((prev_entity, prev_hit_data)), None) => {
            events.send(Pointer::new(
                pointer_id,
                PointerLocation { position: prev_hit_data.position },
                Out { hit: *prev_hit_data },
                *prev_entity,
            ));
        }
        (None, Some((curr_entity, curr_hit_data))) => {
            events.send(Pointer::new(
                pointer_id,
                PointerLocation { position: curr_hit_data.position },
                Over { hit: *curr_hit_data },
                *curr_entity,
            ));
        }
        (None, None) => {}
    }
    
    if let Some((entity, hit_data)) = current_hit {
        if pointer_press.primary && !prev_pointer_press.primary {
            events.send(Pointer::new(
                pointer_id,
                PointerLocation { position: hit_data.position },
                Press { hit: *hit_data, button: PointerButton::Primary },
                *entity,
            ));
        }
        
        if !pointer_press.primary && prev_pointer_press.primary {
            events.send(Pointer::new(
                pointer_id,
                PointerLocation { position: hit_data.position },
                Release { hit: *hit_data, button: PointerButton::Primary },
                *entity,
            ));
            
            events.send(Pointer::new(
                pointer_id,
                PointerLocation { position: hit_data.position },
                Click { hit: *hit_data, button: PointerButton::Primary },
                *entity,
            ));
        }
    }
}
