#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

//! Feather Hybrid Render Engine (Rust version)
//! 
//! A lightweight hybrid rendering engine with declarative dual-world architecture.
//! Inspired by Bevy's ECS and render graph, adapted for embedded systems.
//!
//! Architecture:
//! - Main World: Game logic, scene management, user systems
//! - Render World: GPU resources, draw commands, framebuffer output
//! - Extract: Sync data from Main World to Render World
//! - Schedule: Ordered execution of systems

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;

extern "C" {
    fn malloc(size: usize) -> *mut core::ffi::c_void;
    fn free(ptr: *mut core::ffi::c_void);
}

struct CAllocator;

unsafe impl GlobalAlloc for CAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = malloc(layout.size()) as *mut u8;
        ptr
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if !ptr.is_null() {
            free(ptr as *mut core::ffi::c_void);
        }
    }
}

#[global_allocator]
static ALLOCATOR: CAllocator = CAllocator;

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: Layout) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn rust_eh_personality(
    _version: i32,
    _actions: i32,
    _exception_class: u32,
    _exception_object: *mut core::ffi::c_void,
    _context: *mut core::ffi::c_void,
) -> i32 {
    loop {}
}

// Module declarations
pub mod app;
pub mod main_world;
pub mod render_world;
pub mod extract;
pub mod schedule;
pub mod pipeline;
pub mod animation;
pub mod resources;
pub mod math;
pub mod node;
pub mod event;
pub mod camera;
pub mod plugin;
pub mod window;
pub mod sync;
pub mod picking;
pub mod asset;

// Re-export main types from app
pub use app::{App, FHRE_VERSION};

// Re-export schedule labels from schedule module
pub use schedule::{Startup, PreUpdate, Update, PostUpdate, Last};

// Re-export main types from main_world
pub use main_world::{MainWorld, Entity, Component, System, IntoSystem};
pub use main_world::{SystemParam, Res, ResMut, Query, Local, system1, system2, system3, system4, system5, system6, system7, system8, system9, system10};
pub use main_world::{Commands, Command, CommandsState, EntityCommands};
pub use main_world::query_filter;
pub use main_world::{Mut, Ref, QueryData};

// Re-export render_world types
pub use render_world::{RenderWorld, RenderCommand, View, ViewBundle, RenderComponent};

// Re-export pipeline types
pub use pipeline::{Texture, Sampler, TextureRegion};

// Re-export extract
pub use extract::{
    ExtractComponent, ExtractComponentPlugin, 
    ExtractComponentWithTransform, ExtractComponentWithTransformPlugin,
    ExtractSchedule, ExtractPlugin, Extractors, Extract,
};

// Re-export sync (NEW - aligned with Bevy)
pub use sync::{SyncToRenderWorld, RenderEntity, MainEntity, PendingSyncEntity, entity_sync_system};

// Re-export resources
pub use resources::{Time, PrimaryScreen, Resource};
pub use resources::{Camera, ProjectionType};

// Re-export plugin types
pub use plugin::{Plugin, PluginGroup, DefaultPlugins, SyncComponentPlugin, SyncComponents};
pub use camera::CameraPlugin;

// Re-export node types
pub use node::{Node, Transform, Transform3D};

// Re-export animation types
pub use animation::{AnimationClip, AnimationClipHandle, AnimationPlayer, AnimationResources,
    AnimationProperty, AnimationTargetId, KeyframeCurve, Keyframe, Easing, RepeatAnimation,
    AnimationReceiver, apply_animations};

// Re-export math types (includes Color)
pub use math::{Color, Vec2, Vec3, Mat4};

// Re-export window types (platform-agnostic trait and event types)
pub use window::{Window, WindowInputEvents, MouseButtonEvent, MouseMotionEvent, MouseWheelEvent, KeyboardEvent, MousePosition};

// Re-export picking types
pub use picking::{
    Pickable, PickableBounds, HitData,
    HoverMap, PreviousHoverMap, PointerId,
    PointerHits, PickingPlugin, PointerHitsBuffer,
    update_hover_map, ui_picking_backend, pointer_events,
    PointerPress, PointerLocation, PointerInput, PointerAction, PointerButton,
    Pointer, Over, Out, Enter, Leave, Press, Release, Click, Move, DragStart, Drag, DragEnd,
};

// Re-export event types
pub use event::{Events, Event, EventWriter, EventReader};

// Re-export asset types
pub use asset::{
    Asset, AssetId, AssetIndex, AssetEvent, Handle, Assets, 
    RenderAsset, RenderAssets, ExtractedAssets, RenderAssetPlugin, ExtractResourcePlugin,
    AssetServer, AssetRegistry, AssetLoader, AssetPlugin, AppAssetExt,
};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::schedule::{Startup, Update};
    pub use crate::app::App;
    pub use crate::main_world::{Commands, Query, Res, ResMut, Local, system1, system2, system3, system4};
    pub use crate::resources::{Time, PrimaryScreen};
    pub use crate::plugin::Plugin;
    pub use crate::node::{Node, Transform, Transform3D};
    pub use crate::math::{Color, Vec2, Vec3};
    
    // Sync markers for Bevy-aligned dual-world architecture
    pub use crate::sync::SyncToRenderWorld;
    pub use crate::extract::ExtractComponent;
}
