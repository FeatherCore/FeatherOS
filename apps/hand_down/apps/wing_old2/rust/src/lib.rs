#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

use ::core::alloc::{GlobalAlloc, Layout};
use ::core::panic::PanicInfo;

pub mod app;
pub mod app_sdk;
pub mod app_ui;
pub mod app_ui_sdk;
pub mod app_ui_template;
pub mod animation;
pub mod core;
pub mod diagnostics;
pub mod input;
pub mod math;
pub mod platform;
pub mod render;
pub mod resource_file;
pub mod resource_image;
pub mod runtime;
pub mod rust_surface_demo;
pub mod settings_app;
pub mod settings;
pub mod system_app;
pub mod terminal_app;
pub mod shell;
pub mod shell_notification;
pub mod shell_style;
pub mod surface;
pub mod ui;

use platform::{nuttx::NuttXPlatform, Platform};
use render::{BackgroundPlaneRenderer, RendererBackend};
use runtime::WingRuntime;

extern "C" {
    fn malloc(size: usize) -> *mut ::core::ffi::c_void;
    fn free(ptr: *mut ::core::ffi::c_void);
}

struct CAllocator;

unsafe impl GlobalAlloc for CAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        extern "C" {
            fn aligned_alloc(alignment: usize, size: usize) -> *mut ::core::ffi::c_void;
        }

        let size = layout.size().max(1);
        if layout.align() <= 8 {
            malloc(size) as *mut u8
        } else {
            aligned_alloc(layout.align(), size) as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if !ptr.is_null() {
            free(ptr as *mut ::core::ffi::c_void);
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
    _exception_object: *mut ::core::ffi::c_void,
    _context: *mut ::core::ffi::c_void,
) -> i32 {
    loop {}
}

#[no_mangle]
pub extern "C" fn wing_rust_main(argc: i32, argv: *mut *mut u8) -> i32 {
    let platform = match NuttXPlatform::new() {
        Some(platform) => platform,
        None => return 1,
    };

    match renderer_selection(argc, argv) {
        RuntimeRendererSelection::Software => {
            run_wing_runtime(WingRuntime::new(platform));
        }
        RuntimeRendererSelection::BackgroundPlane => {
            let (width, height) = platform.dimensions();
            run_wing_runtime(WingRuntime::new_with_renderer(
                platform,
                BackgroundPlaneRenderer::new_gpu2d(width, height),
            ));
        }
    }
    0
}

fn run_wing_runtime<P, R>(mut runtime: WingRuntime<P, R>)
where
    P: Platform,
    R: RendererBackend,
{
    shell::install_shell(&mut runtime);
    runtime.run();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeRendererSelection {
    Software,
    BackgroundPlane,
}

fn renderer_selection(argc: i32, argv: *mut *mut u8) -> RuntimeRendererSelection {
    if argv.is_null() {
        return RuntimeRendererSelection::Software;
    }

    let mut index = 1;
    while index < argc {
        let arg = unsafe { *argv.add(index as usize) };
        if unsafe { c_arg_eq(arg, b"--renderer=plane") }
            || unsafe { c_arg_eq(arg, b"--renderer=bg-plane") }
            || unsafe { c_arg_eq(arg, b"--renderer=gpu2d-plane") }
        {
            return RuntimeRendererSelection::BackgroundPlane;
        }
        index += 1;
    }
    RuntimeRendererSelection::Software
}

unsafe fn c_arg_eq(arg: *const u8, expected: &[u8]) -> bool {
    if arg.is_null() {
        return false;
    }

    let mut index = 0usize;
    loop {
        let byte = *arg.add(index);
        if index == expected.len() {
            return byte == 0;
        }
        if byte != expected[index] {
            return false;
        }
        index += 1;
    }
}

#[no_mangle]
pub extern "C" fn rust_wing_init() {}
