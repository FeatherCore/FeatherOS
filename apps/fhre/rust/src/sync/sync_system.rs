//! Sync System - Entity synchronization between Main World and Render World
//!
//! Synchronizes entities marked with SyncToRenderWorld between Main World and Render World.
//! This runs before the Extract phase.

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::sync::sync_markers::{SyncToRenderWorld, RenderEntity, MainEntity};
use crate::sync::pending_sync::{PendingSyncEntity, EntityRecord};
use alloc::vec::Vec;

/// System that synchronizes entities between Main World and Render World
///
/// This should run in the Sync schedule, before ExtractSchedule.
pub fn entity_sync_system(main_world: &mut MainWorld, render_world: &mut RenderWorld) {
    // Process pending sync records
    let records: Vec<_> = {
        let pending = main_world.resources().get::<PendingSyncEntity>();
        match pending {
            Some(pending) => pending.records.clone(),
            None => return,
        }
    };
    
    for record in records {
        match record {
            EntityRecord::Added(main_entity) => {
                sync_added_entity(main_world, render_world, main_entity);
            }
            EntityRecord::Removed(render_entity) => {
                sync_removed_entity(render_world, render_entity);
            }
        }
    }
    
    // Clear pending records
    if let Some(pending) = main_world.resources_mut().get_mut::<PendingSyncEntity>() {
        pending.clear();
    }
}

/// Handle entity added to Main World with SyncToRenderWorld
fn sync_added_entity(
    main_world: &mut MainWorld,
    render_world: &mut RenderWorld,
    main_entity: crate::Entity,
) {
    // Check if entity already has RenderEntity (already synced)
    if main_world.get_component::<RenderEntity>(main_entity).is_some() {
        return;
    }
    
    // Spawn corresponding entity in Render World
    let render_entity = render_world.spawn_synced(main_entity);
    
    // Insert RenderEntity component in Main World
    main_world.insert_component(main_entity, RenderEntity(render_entity));
}

/// Handle entity removed from Main World or SyncToRenderWorld removed
fn sync_removed_entity(
    render_world: &mut RenderWorld,
    render_entity: RenderEntity,
) {
    // Despawn the corresponding entity in Render World
    render_world.despawn(render_entity.id());
}

/// System that detects new SyncToRenderWorld components
///
/// This runs in Main World and populates PendingSyncEntity
pub fn detect_sync_changes_system(main_world: &mut MainWorld) {
    // Query all entities with SyncToRenderWorld but no RenderEntity
    let new_syncs: Vec<_> = main_world.query::<SyncToRenderWorld>()
        .filter(|(entity, _)| {
            main_world.get_component::<RenderEntity>(*entity).is_none()
        })
        .map(|(entity, _)| entity)
        .collect();
    
    // Add to pending
    if let Some(pending) = main_world.resources_mut().get_mut::<PendingSyncEntity>() {
        for entity in new_syncs {
            pending.push(EntityRecord::Added(entity));
        }
    }
}

/// System that detects removed SyncToRenderWorld components
///
/// This should be called when entities are despawned
pub fn detect_sync_removals_system(main_world: &mut MainWorld) {
    // This is called when entities with SyncToRenderWorld are despawned
    // The actual removal detection would need to be integrated with the despawn system
    // For now, this is a placeholder for the concept
}
