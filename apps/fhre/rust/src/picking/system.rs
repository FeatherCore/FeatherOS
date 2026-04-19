//! Picking System

use alloc::vec::Vec;
use crate::{Entity, Component};
use crate::math::Vec2;
use crate::node::Transform;
use super::{Pickable, PickableBounds, PointerHits, HitData, HoverMap, PreviousHoverMap};

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
    pointers: &[(super::hover::PointerId, f32, f32, bool)],
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
