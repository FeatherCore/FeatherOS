//! Application implementation for FHRE
//!
//! Provides the main App struct and lifecycle management.
//! Uses X11 window for input and display on SIM platform.

use alloc::vec::Vec;

use crate::main_world::{MainWorld, IntoSystem, Entity};
use crate::render_world::RenderWorld;
use crate::extract::{extract_renderable_components};
use crate::resources::{Time, RenderConfig, WindowConfig, PrimaryScreen};
use crate::schedule::{Schedules, ScheduleLabel};
use crate::node::{Node, NodeType, Transform3D, Camera3D};
use super::AppConfig;

// External C functions for timing
extern "C" {
    fn clock() -> i64;
    fn usleep(usec: u32) -> i32;
}

const CLOCKS_PER_SEC: i64 = 1000000; // Standard POSIX value

// Platform-specific imports
#[cfg(feature = "sim")]
use crate::platform::x11_window::X11Window;

/// FHRE version
pub const FHRE_VERSION: &str = "2.0.0";

/// Resource to store the default UI camera entity
#[derive(Clone, Copy, Debug)]
pub struct DefaultUiCamera {
    pub entity: Entity,
}

/// Resource to store the default game camera entity (if any)
#[derive(Clone, Copy, Debug)]
pub struct DefaultGameCamera {
    pub entity: Entity,
}

impl crate::resources::Resource for DefaultUiCamera {}
impl crate::resources::Resource for DefaultGameCamera {}

/// The main Application struct
pub struct App {
    pub main_world: MainWorld,
    pub render_world: RenderWorld,
    pub schedules: Schedules,
    pub config: AppConfig,
    #[cfg(feature = "sim")]
    pub x11_window: Option<X11Window>,
    running: bool,
}

impl App {
    /// Create a new application with X11 window for input and display
    #[cfg(feature = "sim")]
    pub fn new_with_x11_window(window_width: u32, window_height: u32, title: &str) -> Self {
        let config = AppConfig::default();

        // Create X11 window for input and display
        let x11_window = X11Window::new(window_width, window_height, title);

        // Use window dimensions
        let (width, height) = if let Some(ref window) = x11_window {
            window.get_dimensions()
        } else {
            (window_width, window_height)
        };

        let mut app = Self {
            main_world: MainWorld::new(),
            render_world: RenderWorld::new(width, height),
            schedules: Schedules::new(),
            config,
            x11_window,
            running: true,
        };

        // Setup default resources
        app.setup_primary_screen(width, height);
        app.setup_default_ui_camera(width, height);

        // Initialize default schedules
        app.init_schedules();

        app
    }

    /// Setup the primary screen resource
    fn setup_primary_screen(&mut self, width: u32, height: u32) {
        let primary_screen = PrimaryScreen::new(width, height);
        self.main_world.resources_mut().insert(primary_screen);
    }

    /// Setup the default UI camera
    fn setup_default_ui_camera(&mut self, width: u32, height: u32) {
        let camera_entity = self.main_world.spawn();
        
        self.main_world.insert_component(
            camera_entity,
            Node::ui_control(NodeType::Camera)
        );
        
        let camera_x = width as f32 / 2.0;
        let camera_y = height as f32 / 2.0;
        let camera_z = 400.0;
        
        self.main_world.insert_component(
            camera_entity,
            Transform3D::from_position(camera_x, camera_y, camera_z)
        );
        
        self.main_world.insert_component(
            camera_entity,
            Camera3D {
                fov: 60.0,
                near: 0.1,
                far: 2000.0,
                background_color: crate::math::Color::BLACK,
                orthographic: false,
                orthographic_size: height as f32 / 2.0,
                viewport: crate::math::Rect::new(0.0, 0.0, 1.0, 1.0),
                culling_mask: 0xFFFFFFFF,
                depth: -100,
            }
        );
        
        self.main_world.resources_mut().insert(DefaultUiCamera {
            entity: camera_entity,
        });
    }

    /// Get the primary screen dimensions
    pub fn screen_dimensions(&self) -> (u32, u32) {
        if let Some(screen) = self.main_world.resources().get::<PrimaryScreen>() {
            screen.dimensions()
        } else {
            (self.config.width, self.config.height)
        }
    }

    /// Get the default UI camera entity
    pub fn default_ui_camera(&self) -> Option<Entity> {
        self.main_world.resources()
            .get::<DefaultUiCamera>()
            .map(|cam| cam.entity)
    }

    /// Initialize default schedules
    fn init_schedules(&mut self) {
        // Schedules are already created in Schedules::new()
    }

    /// Add a system to a schedule
    pub fn add_system<S: IntoSystem>(&mut self, _schedule: ScheduleLabel, system: S) -> &mut Self
    where
        <S as IntoSystem>::System: 'static,
    {
        self.main_world.add_system(system);
        self
    }

    /// Run one update frame
    pub fn update(&mut self) {
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[FHRE_UPDATE] ========== Frame Start ==========\n\0".as_ptr());
        }

        // 0. Poll X11 window events if available
        #[cfg(feature = "sim")]
        {
            unsafe {
                extern "C" {
                    fn printf(format: *const u8, ...) -> i32;
                }
                printf(b"[FHRE_UPDATE] Step 0: Polling X11 events...\n\0".as_ptr());
            }
            if let Some(ref mut window) = self.x11_window {
                window.poll_events();
            }
        }
        
        // 1. Update time
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[FHRE_UPDATE] Step 1: Updating time...\n\0".as_ptr());
        }
        if let Some(time) = self.main_world.resources_mut().get_mut::<Time>() {
            time.update(1.0 / 60.0);
        }

        // 2. Run Main World systems
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[FHRE_UPDATE] Step 2: Running Main World systems...\n\0".as_ptr());
        }
        self.main_world.run_systems();

        // 3. Extract phase
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[FHRE_UPDATE] Step 3: Extract phase...\n\0".as_ptr());
        }
        self.render_world.clear_views();
        extract_renderable_components(&self.main_world, &mut self.render_world);

        // 4. Render phase
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[FHRE_UPDATE] Step 4: Render phase...\n\0".as_ptr());
        }
        self.render_world.execute_render();

        // 5. Present to X11 window
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[FHRE_UPDATE] Step 5: Present to display...\n\0".as_ptr());
        }
        #[cfg(feature = "sim")]
        {
            if let Some(ref window) = self.x11_window {
                let framebuffer = self.render_world.framebuffer();
                window.present(framebuffer);
            }
        }

        // 6. Clear for next frame
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[FHRE_UPDATE] Step 6: Clear render commands...\n\0".as_ptr());
        }
        self.render_world.clear_commands();
        self.render_world.clear_views();

        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[FHRE_UPDATE] ========== Frame End ==========\n\n\0".as_ptr());
        }
    }

    /// Run the application
    pub fn run(&mut self) {
        self.run_with_callback(|| {});
    }
    
    /// Run with callback
    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(),
    {
        self.running = true;
        
        while self.running {
            let start_time = unsafe { clock() };
            
            callback();
            self.update();
            
            let elapsed = unsafe { clock() } - start_time;
            let elapsed_ms = (elapsed * 1000 / CLOCKS_PER_SEC) as u64;
            
            if elapsed_ms < 16 {
                unsafe {
                    usleep(((16 - elapsed_ms) * 1000) as u32);
                }
            }
        }
    }

    /// Check if app is running
    pub fn is_running(&self) -> bool {
        if !self.running {
            return false;
        }

        #[cfg(feature = "sim")]
        {
            if let Some(ref window) = self.x11_window {
                return window.is_running();
            }
        }

        true
    }

    /// Stop the application
    pub fn exit(&mut self) {
        self.running = false;
    }

    /// Get framebuffer data
    pub fn get_framebuffer(&self) -> &[u32] {
        self.render_world.framebuffer()
    }

    /// Get X11 window reference
    #[cfg(feature = "sim")]
    pub fn x11_window(&self) -> Option<&X11Window> {
        self.x11_window.as_ref()
    }

    /// Get mutable X11 window reference
    #[cfg(feature = "sim")]
    pub fn x11_window_mut(&mut self) -> Option<&mut X11Window> {
        self.x11_window.as_mut()
    }
}

/// Application builder
pub struct AppBuilder {
    app: App,
}

impl AppBuilder {
    /// Create a new app builder with X11 window
    #[cfg(feature = "sim")]
    pub fn new_with_x11(width: u32, height: u32, title: &str) -> Self {
        Self {
            app: App::new_with_x11_window(width, height, title),
        }
    }

    /// Set window configuration
    pub fn with_window_config(mut self, config: WindowConfig) -> Self {
        self.app.config.width = config.width;
        self.app.config.height = config.height;
        self
    }

    /// Set render configuration
    pub fn with_render_config(mut self, config: RenderConfig) -> Self {
        self.app.main_world.resources_mut().insert(config);
        self
    }

    /// Add a system
    pub fn add_system<S: IntoSystem>(mut self, system: S) -> Self
    where
        <S as IntoSystem>::System: 'static,
    {
        self.app.add_system(ScheduleLabel::Update, system);
        self
    }

    /// Build the application
    pub fn build(self) -> App {
        self.app
    }
}
