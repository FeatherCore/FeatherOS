#![no_std]
#![no_main]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout};
use core::ffi::{c_int, c_void};
use core::panic::PanicInfo;
use fhre::{
    Camera, DirtyRegion, FhreRuntime, FrameClock, FramePolicy, ImageCache, ImageId, ImageView,
    PresentStats, Rect,
};

#[path = "../../../common/fhre_nuttx_runtime.rs"]
mod fhre_nuttx_runtime;

use fhre_nuttx_runtime::{
    elapsed_us, now_us, poll_sim_events, sleep_remaining, NuttxFramebuffer, NuttxInput,
    NuttxResourceLoader,
};

const WING_ASSET_PREFIX: &[u8] = b"/etc/wing/resource/windows10_mobile/assets/";
const IMAGE_CACHE_MAX_SLOTS: usize = 8;
const IMAGE_CACHE_MAX_BYTES: usize = 2 * 1024 * 1024;

static mut WING_DEMO_STATE: wing::WingDemoState = wing::WingDemoState::new();
static mut WING_IMAGE_CACHE: Option<ImageCache> = None;

struct NuttxAllocator;

unsafe impl GlobalAlloc for NuttxAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size().max(1)).cast::<u8>()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr.cast::<c_void>());
    }

    unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
        realloc(ptr.cast::<c_void>(), new_size.max(1)).cast::<u8>()
    }
}

#[global_allocator]
static ALLOCATOR: NuttxAllocator = NuttxAllocator;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn printf(format: *const u8, ...) -> c_int;
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

fn wing_image_resolver(image: ImageId) -> Option<ImageView> {
    unsafe {
        let cache_slot = core::ptr::addr_of_mut!(WING_IMAGE_CACHE);
        let cache = match &mut *cache_slot {
            Some(cache) => cache,
            None => return None,
        };
        fhre_nuttx_runtime::cached_image_view(cache, image)
    }
}

fn prewarm_wing_image_cache() {
    unsafe {
        let cache_slot = core::ptr::addr_of_mut!(WING_IMAGE_CACHE);
        if (*cache_slot).is_none() {
            *cache_slot = Some(ImageCache::new(IMAGE_CACHE_MAX_SLOTS, IMAGE_CACHE_MAX_BYTES));
        }
        let Some(cache) = &mut *cache_slot else {
            return;
        };
        let mut loader = NuttxResourceLoader::new(WING_ASSET_PREFIX);
        for asset in wing::WINDOWS10_MOBILE_PREWARM_ASSETS {
            let _ = cache.prewarm(asset.image(), asset.path(), &mut loader, true);
        }
    }
}

fn prewarm_route_assets(mode: wing::ShellMode) {
    const HOME: [wing::WingAssetId; 6] = [
        wing::WingAssetId(23),
        wing::WingAssetId(28),
        wing::WingAssetId(29),
        wing::WingAssetId(30),
        wing::WingAssetId(32),
        wing::WingAssetId(33),
    ];
    const NOTIFICATIONS: [wing::WingAssetId; 6] = [
        wing::WingAssetId(1),
        wing::WingAssetId(2),
        wing::WingAssetId(3),
        wing::WingAssetId(4),
        wing::WingAssetId(33),
        wing::WingAssetId(38),
    ];
    const SETTINGS: [wing::WingAssetId; 10] = [
        wing::WingAssetId(33),
        wing::WingAssetId(44),
        wing::WingAssetId(45),
        wing::WingAssetId(47),
        wing::WingAssetId(49),
        wing::WingAssetId(50),
        wing::WingAssetId(51),
        wing::WingAssetId(53),
        wing::WingAssetId(54),
        wing::WingAssetId(56),
    ];
    const LOCK: [wing::WingAssetId; 1] = [wing::WingAssetId(46)];
    const APPS: [wing::WingAssetId; 2] = [
        wing::WingAssetId(37),
        wing::WingAssetId(48),
    ];

    let assets: &[wing::WingAssetId] = match mode {
        wing::ShellMode::Home | wing::ShellMode::AllApps | wing::ShellMode::AppSwitcher => &HOME,
        wing::ShellMode::Notifications => &NOTIFICATIONS,
        wing::ShellMode::Settings(_) => &SETTINGS,
        wing::ShellMode::LockScreen => &LOCK,
        wing::ShellMode::App(_) => &APPS,
        wing::ShellMode::Cortana | wing::ShellMode::TileDetail(_) => &HOME,
    };

    unsafe {
        let cache_slot = core::ptr::addr_of_mut!(WING_IMAGE_CACHE);
        if (*cache_slot).is_none() {
            *cache_slot = Some(ImageCache::new(IMAGE_CACHE_MAX_SLOTS, IMAGE_CACHE_MAX_BYTES));
        }
        let Some(cache) = &mut *cache_slot else {
            return;
        };
        let mut loader = NuttxResourceLoader::new(WING_ASSET_PREFIX);
        let mut index = 0;
        while index < assets.len() {
            let asset = assets[index];
            let _ = cache.prewarm(asset.image(), asset.path(), &mut loader, false);
            index += 1;
        }
    }
}

fn reset_dirty_to_full<const N: usize>(dirty: &mut DirtyRegion<N>, screen: Rect) {
    dirty.clear();
    dirty.full_redraw(screen);
}

#[no_mangle]
pub extern "C" fn wing_demo_main(_argc: i32, _argv: *mut *mut u8) -> i32 {
    let Some(mut fb) = NuttxFramebuffer::open() else {
        unsafe {
            printf(b"wing_demo: cannot open /dev/fb0\n\0".as_ptr());
        }
        return 1;
    };

    unsafe {
        printf(b"wing_demo: Rust no_std Wing UI demo\n\0".as_ptr());
    }

    let state = unsafe {
        let state = core::ptr::addr_of_mut!(WING_DEMO_STATE);
        (*state).reset();
        &mut *state
    };
    let mut runtime = FhreRuntime::<32, 16>::new(Camera::screen_canvas(fb.width(), fb.height()));
    let mut input = NuttxInput::open();
    let mut actions = wing::ShellActionQueue::<16>::new();
    let mut clock = FrameClock::new(FramePolicy::SIXTY_FPS);
    let screen = Rect::new(0, 0, fb.width(), fb.height());
    let mut pending_dirty: DirtyRegion<32> = DirtyRegion::new();
    reset_dirty_to_full(&mut pending_dirty, screen);
    let mut previous_mode = state.shell().mode;
    prewarm_wing_image_cache();
    prewarm_route_assets(previous_mode);
    loop {
        let frame = clock.frame();
        poll_sim_events();
        let frame_start = now_us();
        runtime.tick(FramePolicy::SIXTY_FPS.target_frame_us);
        let input_events = input.pump(&mut runtime.input, runtime.render.time.uptime_us) as u32;
        state.dispatch_input_queue(
            &mut runtime.input,
            &mut actions,
            fb.width(),
            fb.height(),
        );
        while actions.pop().is_some() {}
        let mode = state.shell().mode;
        if mode != previous_mode {
            reset_dirty_to_full(&mut pending_dirty, screen);
            prewarm_route_assets(mode);
            previous_mode = mode;
        }

        let draw_start = now_us();
        fb.surface.set_image_resolver(Some(wing_image_resolver));
        let had_dirty = !pending_dirty.is_empty();
        let dirty_copy_bytes = if had_dirty {
            fb.prepare_dirty_frame(&pending_dirty)
        } else {
            0
        };
        let full_redraw = pending_dirty.overflowed();
        let dirty_clip = if full_redraw {
            None
        } else {
            pending_dirty.union_rect()
        };
        if let Some(clip) = dirty_clip {
            fb.surface.set_clip(Some(clip));
        } else {
            fb.surface.clear_clip();
        }
        if !pending_dirty.is_empty() {
            wing::draw_demo_with_state(&mut fb.surface, frame, state);
        }
        fb.surface.clear_clip();
        pending_dirty.clear();
        state.collect_dirty_into(&mut pending_dirty, screen);
        let _ = dirty_copy_bytes;
        let draw_us = elapsed_us(draw_start, now_us());
        let present_start = now_us();
        let present = if had_dirty {
            fb.present()
        } else {
            PresentStats::skipped()
        };
        let present_us = elapsed_us(present_start, now_us());
        let frame_stats = clock.finish_frame(draw_us, present_us, input_events, present);
        sleep_remaining(frame_start, frame_stats);
    }
}
