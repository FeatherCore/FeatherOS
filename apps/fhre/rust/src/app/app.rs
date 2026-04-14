//! Application implementation for FHRE
//!
//! Provides the main App struct and lifecycle management.
//! Integrates with NuttX SIM platform framebuffer similar to LVGL.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::main_world::{MainWorld, IntoSystem};
use crate::render_world::RenderWorld;
use crate::extract::extract_system;
use crate::renderer::Renderer;
use crate::resources::{Time, RenderConfig, WindowConfig};
use crate::schedule::{Schedules, ScheduleLabel};
use super::AppConfig;

// External C functions for timing
extern "C" {
    fn clock() -> i64;
    fn usleep(usec: u32) -> i32;
}

const CLOCKS_PER_SEC: i64 = 1000000; // Standard POSIX value

// Platform-specific imports
#[cfg(feature = "sim")]
use crate::platform::sim::SimDisplay;

/// FHRE version
pub const FHRE_VERSION: &str = "2.0.0";

/// The main Application struct
pub struct App {
    pub main_world: MainWorld,
    pub render_world: RenderWorld,
    pub renderer: Renderer,
    pub schedules: Schedules,
    pub config: AppConfig,
    #[cfg(feature = "sim")]
    pub sim_display: Option<SimDisplay>,
    #[cfg(not(feature = "sim"))]
    pub sim_display: Option<()>,
    running: bool,
}

impl App {
    /// Create a new application
    pub fn new() -> Self {
        let config = AppConfig::default();
        let (width, height) = (config.width, config.height);

        // Try to create SIM display (for NuttX SIM platform)
        #[cfg(feature = "sim")]
        let sim_display = SimDisplay::new();
        
        #[cfg(not(feature = "sim"))]
        let sim_display: Option<()> = None;
        
        // If SIM display is available, use its dimensions
        #[cfg(feature = "sim")]
        let (width, height) = if let Some(ref display) = sim_display {
            display.dimensions()
        } else {
            (width, height)
        };

        let mut app = Self {
            main_world: MainWorld::new(),
            render_world: RenderWorld::new(width, height),
            renderer: Renderer::new(width, height),
            schedules: Schedules::new(),
            config,
            sim_display,
            running: false,
        };

        // Initialize default schedules
        app.init_schedules();

        app
    }

    /// Initialize default schedules
    fn init_schedules(&mut self) {
        // Schedules are already created in Schedules::new()
        // We can add custom initialization here if needed
    }

    /// Add a system to a schedule
    pub fn add_system<S: IntoSystem>(&mut self, _schedule: ScheduleLabel, system: S) -> &mut Self
    where
        <S as IntoSystem>::System: 'static,
    {
        self.main_world.add_system(system);
        self
    }

    /// Add a startup system
    pub fn add_startup_system<S: IntoSystem>(&mut self, _system: S) -> &mut Self
    where
        <S as IntoSystem>::System: 'static,
    {
        // Startup systems would be added to the startup schedule
        // Implementation depends on how we handle systems
        self
    }

    /// Run the startup phase once
    pub fn startup(&mut self) {
        // Run startup schedule if it exists
        // self.schedules.run(ScheduleLabel::PreUpdate);
    }

    /// Run one update frame
    pub fn update(&mut self) {
        // 1. Update time
        if let Some(time) = self.main_world.resources_mut().get_mut::<Time>() {
            time.update(1.0 / 60.0); // Default to 60 FPS
        }

        // 2. Run Main World systems
        self.main_world.run_systems();

        // 3. Extract phase - sync Main World to Render World
        extract_system(&self.main_world, &mut self.render_world);

        // 4. Render the world
        self.renderer.render(&mut self.render_world);

        // 5. Present to SIM display if available (NuttX SIM platform)
        // Similar to LVGL's flush_cb calling FBIO_UPDATE
        #[cfg(feature = "sim")]
        {
            if let Some(ref mut display) = self.sim_display {
                display.present(&self.render_world);
            }
        }
    }

    /// Run the application with integrated refresh loop
    /// Similar to LVGL's lv_nuttx_run() - manages the main loop internally
    pub fn run(&mut self) {
        self.run_with_callback(|| {});
    }
    
    /// Run the application with a custom update callback
    /// The callback is called before each frame update, allowing custom game logic
    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(),
    {
        self.running = true;
        
        // Run startup
        self.startup();

        // Main loop with frame rate control (similar to LVGL)
        // Default to 60 FPS with usleep for CPU yield
        while self.running {
            let start_time = unsafe { clock() };
            
            // Call custom callback (e.g., game logic updates)
            callback();
            
            // Update one frame
            self.update();
            
            // Calculate elapsed time and sleep to maintain frame rate
            let elapsed = unsafe { clock() } - start_time;
            let elapsed_ms = (elapsed * 1000 / CLOCKS_PER_SEC) as u64;
            
            // Target 16ms per frame (~60 FPS)
            if elapsed_ms < 16 {
                unsafe {
                    usleep(((16 - elapsed_ms) * 1000) as u32);
                }
            }
        }
    }

    /// Check if the app is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stop the application
    pub fn exit(&mut self) {
        self.running = false;
    }

    /// Get framebuffer data for output
    pub fn get_framebuffer(&self) -> &[u32] {
        self.render_world.framebuffer()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Application builder for more convenient setup
pub struct AppBuilder {
    app: App,
}

impl AppBuilder {
    /// Create a new app builder
    pub fn new() -> Self {
        Self {
            app: App::new(),
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
        // Store render config in resources
        self.app.main_world.resources_mut().insert(config);
        self
    }

    /// Add a system to the update schedule
    pub fn add_system<S: IntoSystem>(mut self, system: S) -> Self
    where
        <S as IntoSystem>::System: 'static,
    {
        self.app.add_system(ScheduleLabel::Update, system);
        self
    }

    /// Add a startup system
    pub fn add_startup_system<S: IntoSystem>(mut self, system: S) -> Self
    where
        <S as IntoSystem>::System: 'static,
    {
        self.app.add_startup_system(system);
        self
    }

    /// Build the application
    pub fn build(self) -> App {
        self.app
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}
