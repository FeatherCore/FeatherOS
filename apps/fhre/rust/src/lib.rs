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
pub mod schedule;
pub mod pipeline;
pub mod animation;
pub mod resources;
pub mod math;
pub mod node;
pub mod ui;
pub mod event;
pub mod input;
pub mod plugin;

// Re-export main types from app
pub use app::{App, AppBuilder, FHRE_VERSION, DefaultUiCamera, DefaultGameCamera, AppRunner, AppExit, Startup, PreUpdate, Update, PostUpdate};

// Re-export main types from main_world
pub use main_world::{MainWorld, Entity, Component, System, IntoSystem};
pub use main_world::{SystemParam, Res, ResMut, Query, Local, system1, system2, system3};
pub use main_world::{Commands, Command, CommandsState, EntityCommands};

// Re-export render_world types
pub use render_world::{RenderWorld, RenderCommand, View, ViewBundle, RenderComponent};

// Re-export extract
pub use extract::extract_renderable_components;

// Re-export schedule types (now in app module)
// pub use schedule::{Startup, PreUpdate, Update, PostUpdate};

// Re-export resources
pub use resources::{Time, PrimaryScreen, Resource};

// Re-export plugin types
pub use plugin::{Plugin, PluginGroup};

// Re-export node types
pub use node::{Node, NodeType, Transform2D, Transform3D};

// Re-export UI types
pub use ui::{Button, Cube, SoccerBall};

// Re-export math types (includes Color)
pub use math::{Color, Vec2, Vec3, Mat4};

// Re-export input types
pub use input::{ButtonInput, MouseButton, KeyCode};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::app::{App, Startup, Update};
    pub use crate::main_world::{Commands, Query, Res, ResMut, Local, system1, system2, system3};
    pub use crate::resources::{Time, PrimaryScreen};
    pub use crate::plugin::Plugin;
    pub use crate::node::{Node, NodeType, Transform2D, Transform3D};
    pub use crate::ui::{Button, Cube, SoccerBall};
    pub use crate::math::{Color, Vec2, Vec3};
}
