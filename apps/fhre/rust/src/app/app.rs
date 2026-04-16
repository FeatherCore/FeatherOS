//! Application implementation for FHRE
//!
//! Provides the main App struct and lifecycle management.
//! Integrates with NuttX SIM platform framebuffer similar to LVGL.
//!
//! # Default Setup
//!
//! FHRE provides default built-in resources:
//! - **PrimaryScreen**: The main render target with global resolution
//! - **Default UI Camera**: Orthographic camera for UI rendering
//!
//! These are automatically created when App is initialized.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::main_world::{MainWorld, IntoSystem, Entity};
use crate::render_world::RenderWorld;
use crate::extract::{extract_sprites, extract_buttons, extract_renderable_components};
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
use crate::platform::sim::SimDisplay;

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
    pub sim_display: Option<SimDisplay>,
    #[cfg(not(feature = "sim"))]
    pub sim_display: Option<()>,
    running: bool,
}

impl App {
    /// Create a new application with default screen and camera
    ///
    /// This automatically sets up:
    /// 1. PrimaryScreen - the main render target
    /// 2. Default UI Camera - orthographic camera for UI rendering
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
            schedules: Schedules::new(),
            config,
            sim_display,
            running: false,
        };

        // Setup default resources
        app.setup_primary_screen(width, height);
        app.setup_default_ui_camera(width, height);
        
        // Initialize default schedules
        app.init_schedules();

        app
    }

    /// Setup the primary screen resource
    ///
    /// The primary screen is the default render target for FHRE.
    /// All rendering happens to this screen by default.
    fn setup_primary_screen(&mut self, width: u32, height: u32) {
        let primary_screen = PrimaryScreen::new(width, height);
        self.main_world.resources_mut().insert(primary_screen);
    }

    /// Setup the default UI camera
    ///
    /// Creates a perspective camera for 3D UI rendering (including book flip effect).
    /// This camera is positioned to show 3D effects like page flipping.
    fn setup_default_ui_camera(&mut self, width: u32, height: u32) {
        let camera_entity = self.main_world.spawn();
        
        // Create UI camera node
        self.main_world.insert_component(
            camera_entity,
            Node::ui_control(NodeType::Camera)
        );
        
        // Position camera for 3D perspective view
        // Positioned to see the book flip effect clearly
        let camera_x = width as f32 / 2.0;
        let camera_y = height as f32 / 2.0;
        let camera_z = 400.0; // Distance for good perspective view
        
        self.main_world.insert_component(
            camera_entity,
            Transform3D::from_position(camera_x, camera_y, camera_z)
        );
        
        // Perspective camera for 3D book flip effect
        self.main_world.insert_component(
            camera_entity,
            Camera3D {
                fov: 60.0,
                near: 0.1,
                far: 2000.0,
                background_color: crate::math::Color::BLACK,
                orthographic: false,  // Use perspective for 3D effect
                orthographic_size: height as f32 / 2.0,
                viewport: crate::math::Rect::new(0.0, 0.0, 1.0, 1.0),
                culling_mask: 0xFFFFFFFF,
                depth: -100,  // Render first (lowest depth)
            }
        );
        
        // Store as default UI camera resource
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

    /// Create a new camera for game rendering
    ///
    /// This creates a perspective camera that can be used for 3D game rendering.
    /// The camera is positioned at the given location and looks at the target.
    pub fn create_game_camera(
        &mut self,
        position: crate::math::Vec3,
        target: crate::math::Vec3,
        fov: f32,
    ) -> Entity {
        let camera_entity = self.main_world.spawn();
        
        self.main_world.insert_component(
            camera_entity,
            Node::game_entity(NodeType::Camera)
        );
        
        let mut transform = Transform3D::from_position(position.x, position.y, position.z);
        transform.look_at(target);
        self.main_world.insert_component(camera_entity, transform);
        
        self.main_world.insert_component(
            camera_entity,
            Camera3D {
                fov,
                near: 0.1,
                far: 1000.0,
                background_color: crate::math::Color::BLACK,
                orthographic: false,
                orthographic_size: 5.0,
                viewport: crate::math::Rect::new(0.0, 0.0, 1.0, 1.0),
                culling_mask: 0xFFFFFFFF,
                depth: 0,  // Render after UI camera
            }
        );
        
        // Store as default game camera
        self.main_world.resources_mut().insert(DefaultGameCamera {
            entity: camera_entity,
        });
        
        camera_entity
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
        // 重要：首先清除所有视图，避免视图不断累加
        self.render_world.clear_views();
        
        // Extract 3D renderable components (Cube, SoccerBall, etc.)
        // 这个函数会创建一个视图，并且提取所有 3D 渲染组件
        extract_renderable_components(&self.main_world, &mut self.render_world);

        // 4. Render phase - execute render commands through RenderWorld
        // RenderWorld manages the backend internally (Software/GPU/Hybrid)
        self.render_world.execute_render();

        // 5. Present to SIM display if available (NuttX SIM platform)
        // Similar to LVGL's flush_cb calling FBIO_UPDATE
        #[cfg(feature = "sim")]
        {
            if let Some(ref mut display) = self.sim_display {
                display.present(&self.render_world);
            }
        }

        // 6. Clear render commands and views for next frame
        // Prevent command accumulation between frames
        self.render_world.clear_commands();
        self.render_world.clear_views();
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
