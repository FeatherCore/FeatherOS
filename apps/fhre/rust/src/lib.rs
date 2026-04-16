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

// Import libc functions
extern "C" {
    fn malloc(size: usize) -> *mut core::ffi::c_void;
    fn free(ptr: *mut core::ffi::c_void);
}

// Simple global allocator that uses C's malloc and free
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

// Implement rust_eh_personality for no_std environment
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
pub mod schedule;      // System execution scheduling (PreUpdate, Update, PostUpdate, Extract, Render)
pub mod pipeline;      // GPU/CPU hybrid rendering pipeline and batch processing
pub mod animation;     // Animation system (inspired by Bevy's animation)
pub mod resources;
pub mod math;
pub mod platform;      // Platform abstraction (includes framebuffer)
pub mod node;          // Node system - unified minimum unit for game entities and UI controls
pub mod ui;            // UI components - buttons, labels, panels, etc.

// Re-export main types
pub use app::{App, AppBuilder, AppConfig, RunMode, FHRE_VERSION};
pub use main_world::{MainWorld, Entity, Component, System, IntoSystem, Transform, Sprite, Velocity};
pub use render_world::{RenderWorld, RenderCommand, DrawCall, RenderObject, ExtractedTransform, ExtractedSprite, Vertex, PrimitiveType};
pub use extract::{Extract, ExtractSchedule, default_extract_schedule, extract_sprites, extract_buttons, extract_cubes, extract_time};
pub use schedule::{Schedule, ScheduleLabel, SystemSet, Schedules};
pub use pipeline::{HybridScheduler, GpuTaskCollector, RenderBatch, BatchStats};
pub use animation::{AnimationClip, AnimationClipHandle, AnimationPlayer, ActiveAnimation, RepeatAnimation, AnimationState};
pub use animation::{AnimationGraph, AnimationGraphHandle, AnimationNodeIndex, AnimationTransitions};
pub use animation::{AnimationCurve, KeyframeCurve, Keyframe, Easing, AnimationProperty, AnimationTargetId};
pub use animation::{AnimationPlugin, AnimationResources, animate_system, transition_system};
pub use resources::{Resources, Resource, Time, RenderConfig, WindowConfig};
pub use platform::{Framebuffer, SimpleFramebuffer};
pub use math::{Vec2, Vec3, Color, Rect};

// Platform-specific exports - conditionally compiled
#[cfg(feature = "sim")]
pub use platform::sim::{SimDisplay, create_display, refresh_loop, FB_DEVICE_PATH};
