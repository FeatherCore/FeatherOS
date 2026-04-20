//! Asset Extract Plugin - Automatic asset extraction to Render World
//!
//! This module provides plugins for automatically extracting assets from
//! Main World to Render World, following Bevy's architecture.

use crate::asset::{AssetEvent, AssetId, Assets, RenderAsset, RenderAssets, ExtractedAssets};
use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::resources::Resource;
use crate::plugin::Plugin;
use crate::app::App;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

/// Plugin that sets up automatic extraction and preparation for a render asset type.
///
/// This follows Bevy's `RenderAssetPlugin` pattern:
/// 1. Extract: Copy modified assets to Render World
/// 2. Prepare: Convert to GPU representation
pub struct RenderAssetPlugin<A: RenderAsset> {
    _marker: core::marker::PhantomData<A>,
}

impl<A: RenderAsset> Default for RenderAssetPlugin<A> {
    fn default() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<A: RenderAsset> Plugin for RenderAssetPlugin<A> {
    fn build(&self, app: &mut App) {
        app.render_world.insert_resource(ExtractedAssets::<A>::new());
        app.render_world.insert_resource(RenderAssets::<A>::new());
        
        app.add_extractor(extract_assets::<A>);
        app.add_extractor(prepare_assets::<A>);
    }
}

/// Extract modified assets from Main World to Render World.
fn extract_assets<A: RenderAsset>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    let events = main_world.resources()
        .get::<crate::event::Events>()
        .and_then(|e| e.read::<AssetEvent<A::SourceAsset>>());
    
    let mut needs_extract: BTreeSet<AssetId<A::SourceAsset>> = BTreeSet::new();
    let mut removed: Vec<AssetId<A::SourceAsset>> = Vec::new();
    
    if let Some(events) = events {
        for event in events.into_iter() {
            match event {
                AssetEvent::Added { id } | AssetEvent::Modified { id } => {
                    needs_extract.insert(id);
                }
                AssetEvent::Removed { id } | AssetEvent::Unused { id } => {
                    needs_extract.remove(&id);
                    removed.push(id);
                }
                AssetEvent::LoadedWithDependencies { id } => {
                    needs_extract.insert(id);
                }
            }
        }
    }
    
    let assets = main_world.resources().get::<Assets<A::SourceAsset>>();
    if assets.is_none() {
        return;
    }
    let assets = assets.unwrap();
    
    let extracted: Vec<(AssetId<A::SourceAsset>, A::SourceAsset)> = needs_extract
        .iter()
        .filter_map(|id| {
            assets.get(*id).map(|asset| (*id, asset.clone()))
        })
        .collect();
    
    if let Some(extracted_assets) = render_world.get_resource_mut::<ExtractedAssets<A>>() {
        extracted_assets.extracted = extracted;
        extracted_assets.removed = removed;
    }
}

/// Prepare extracted assets for GPU usage.
fn prepare_assets<A: RenderAsset>(
    _main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    let extracted = render_world.get_resource_mut::<ExtractedAssets<A>>();
    if extracted.is_none() {
        return;
    }
    
    let mut extracted = extracted.unwrap();
    let to_prepare = core::mem::take(&mut extracted.extracted);
    let removed = core::mem::take(&mut extracted.removed);
    
    for (id, source) in to_prepare {
        if let Some(prepared) = A::prepare_asset(&source, render_world) {
            if let Some(render_assets) = render_world.get_resource_mut::<RenderAssets<A>>() {
                render_assets.insert(id, prepared);
            }
        }
    }
    
    for id in removed {
        if let Some(render_assets) = render_world.get_resource_mut::<RenderAssets<A>>() {
            render_assets.remove(id);
        }
        A::unload_asset(id, render_world);
    }
}

/// Plugin that extracts a resource from Main World to Render World.
pub struct ExtractResourcePlugin<R: Resource + Clone> {
    _marker: core::marker::PhantomData<R>,
}

impl<R: Resource + Clone> Default for ExtractResourcePlugin<R> {
    fn default() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }
}

impl<R: Resource + Clone> Plugin for ExtractResourcePlugin<R> {
    fn build(&self, app: &mut App) {
        app.add_extractor(extract_resource::<R>);
    }
}

fn extract_resource<R: Resource + Clone>(
    main_world: &MainWorld,
    render_world: &mut RenderWorld,
) {
    if let Some(resource) = main_world.resources().get::<R>() {
        render_world.insert_resource(resource.clone());
    }
}
