//! App Module
//!
//! The main application orchestrator that coordinates Main World and Render World.
//! Aligned with Bevy's App design - provides plugin system and schedule management.

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::resources::{Time, PrimaryScreen};
use crate::plugin::{Plugin, PluginGroup};
use crate::extract::extract_renderable_components;
use crate::sync::{entity_sync_system, detect_sync_changes_system, PendingSyncEntity};
use crate::extract::ExtractSchedule;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;

// Re-export schedule types from schedule module
pub use crate::schedule::{Startup as ScheduleStartup, PreUpdate as SchedulePreUpdate, 
                          Update as ScheduleUpdate, PostUpdate as SchedulePostUpdate};

/// FHRE Version
pub const FHRE_VERSION: &str = "2.0.0";

/// App Builder for fluent API
pub struct AppBuilder;

/// App exit status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppExit {
    Success,
    Error(i32),
}

/// Trait for custom app runners
pub trait AppRunner {
    fn run(self: Box<Self>, app: App) -> AppExit;
}

/// Schedule labels
pub struct Startup;
pub struct PreUpdate;
pub struct Update;
pub struct PostUpdate;

/// The main application struct
///
/// Similar to Bevy's App, this orchestrates the entire application lifecycle.
/// It manages plugins, schedules, and coordinates between Main World and Render World.
///
/// # Example
/// ```rust
/// fn main() {
///     App::new(640, 480)
///         .add_plugin(MyPlugin)
///         .run();
/// }
/// ```
pub struct App {
    /// Main World - holds all game logic state (entities, components, resources)
    pub main_world: MainWorld,
    
    /// Render World - holds render-specific state (views, render commands)
    pub render_world: RenderWorld,
    
    /// Plugins registered with the app
    plugins: Vec<Box<dyn Plugin>>,
    
    /// Whether the app has been initialized
    initialized: bool,
    
    /// Custom runner (optional)
    runner: Option<Box<dyn AppRunner>>,
}

impl App {
    /// Create a new App with the specified screen dimensions
    ///
    /// # Example
    /// ```rust
    /// let app = App::new(640, 480);
    /// ```
    pub fn new(width: u32, height: u32) -> Self {
        let mut app = Self {
            main_world: MainWorld::new(),
            render_world: RenderWorld::new(width, height),
            plugins: Vec::new(),
            initialized: false,
            runner: None,
        };
        
        // Insert default resources
        app.insert_resource(Time::default());
        app.insert_resource(PrimaryScreen::new(width, height));
        
        app
    }

    /// Add a plugin to the app
    ///
    /// Plugins are the primary way to extend the app with functionality.
    /// They can add systems, resources, and other setup.
    ///
    /// # Example
    /// ```rust
    /// app.add_plugin(MyPlugin);
    /// ```
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        // Always build plugin immediately, regardless of initialization state
        // This ensures resources and entities are created when add_plugin is called
        plugin.build(self);
        self
    }

    /// Add a boxed plugin to the app
    ///
    /// This is used internally by PluginGroup.
    pub fn add_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) -> &mut Self {
        if !self.initialized {
            // Store plugin for later initialization
            self.plugins.push(plugin);
        } else {
            // Initialize immediately if app is already running
            plugin.build(self);
        }
        self
    }

    /// Add a group of plugins
    ///
    /// # Example
    /// ```rust
    /// app.add_plugins(DefaultPlugins);
    /// ```
    pub fn add_plugins<P: PluginGroup>(&mut self, group: P) -> &mut Self {
        let builder = group.build();
        builder.finish(self);
        self
    }

    /// Insert a resource into the Main World
    ///
    /// Resources are global singletons that can be accessed by systems.
    ///
    /// # Example
    /// ```rust
    /// app.insert_resource(MyResource::new());
    /// ```
    pub fn insert_resource<R: crate::resources::Resource>(&mut self, resource: R) -> &mut Self {
        self.main_world.resources_mut().insert(resource);
        self
    }

    /// Add systems to a schedule
    ///
    /// # Example
    /// ```rust
    /// use fhre::prelude::*;
    ///
    /// app.add_systems(Update, my_system);
    /// ```
    pub fn add_systems(&mut self, _schedule: impl ScheduleLabel, systems: impl IntoSystems) -> &mut Self {
        systems.add_to_app(self, _schedule);
        self
    }

    /// Set a custom runner for the app
    ///
    /// The runner controls the main loop of the application.
    /// By default, the app uses a simple runner that runs once.
    ///
    /// # Example
    /// ```rust
    /// app.set_runner(MyCustomRunner);
    /// ```
    pub fn set_runner(&mut self, runner: impl AppRunner + 'static) -> &mut Self {
        self.runner = Some(Box::new(runner));
        self
    }

    /// Initialize all plugins
    fn initialize_plugins(&mut self) {
        if self.initialized {
            return;
        }

        // Take ownership of plugins to avoid borrow issues
        let plugins: Vec<Box<dyn Plugin>> = self.plugins.drain(..).collect();
        
        for plugin in plugins {
            plugin.build(self);
        }

        self.initialized = true;
    }

    /// Run the application
    ///
    /// This will initialize all plugins and then run the main loop.
    /// If no runner is set, it will use the default runner.
    pub fn run(mut self) -> AppExit {
        self.initialize_plugins();

        if let Some(runner) = self.runner.take() {
            runner.run(self)
        } else {
            // Default runner - just run once
            self.update();
            AppExit::Success
        }
    }

    /// Run a single update cycle
    ///
    /// This is called by the runner each frame.
    pub fn update(&mut self) {
        // Initialize plugins on first update
        self.initialize_plugins();

        // Run startup systems (only on first frame)
        self.main_world.run_startup_systems();

        // Update time
        if let Some(time) = self.main_world.resources_mut().get_mut::<Time>() {
            time.update(1.0 / 60.0);
        }

        // Run Main World systems
        self.main_world.run_systems();

        // =========================================================================
        // Extract Phase (NEW: aligned with Bevy's dual-world architecture)
        // =========================================================================

        // Step 1: Detect new SyncToRenderWorld markers
        detect_sync_changes_system(&mut self.main_world);

        // Step 2: Sync entities between Main World and Render World
        entity_sync_system(&mut self.main_world, &mut self.render_world);

        // Step 3: Run registered extract systems from ExtractSchedule
        self.render_world.clear_views();
        if let Some(schedule) = self.main_world.resources().get::<ExtractSchedule>() {
            let schedule = schedule.clone();
            schedule.run(&self.main_world, &mut self.render_world);
        }

        // Step 4: Legacy extract (for backward compatibility)
        extract_renderable_components(&self.main_world, &mut self.render_world);

        // =========================================================================
        // Render Phase
        // =========================================================================
        self.render_world.execute_render();

        // Clear for next frame
        self.render_world.clear_commands();
        self.render_world.clear_views();
    }

    /// Update and render a single frame (for manual control)
    ///
    /// This is used when the Demo wants to control the main loop.
    pub fn update_and_render(&mut self) {
        // Initialize plugins on first call (same as run()/update())
        self.initialize_plugins();

        // Run startup systems (only on first frame)
        self.main_world.run_startup_systems();

        // Update time
        if let Some(time) = self.main_world.resources_mut().get_mut::<Time>() {
            time.update(1.0 / 60.0);
        }

        // Run Main World systems
        self.main_world.run_systems();

        // =========================================================================
        // Extract Phase (NEW: aligned with Bevy's dual-world architecture)
        // =========================================================================

        // Step 1: Detect new SyncToRenderWorld markers
        detect_sync_changes_system(&mut self.main_world);

        // Step 2: Sync entities between Main World and Render World
        entity_sync_system(&mut self.main_world, &mut self.render_world);

        // Step 3: Run registered extract systems from ExtractSchedule
        self.render_world.clear_views();
        if let Some(schedule) = self.main_world.resources().get::<ExtractSchedule>() {
            let schedule = schedule.clone();
            schedule.run(&self.main_world, &mut self.render_world);
        }

        // Step 4: Legacy extract (for backward compatibility)
        extract_renderable_components(&self.main_world, &mut self.render_world);

        // =========================================================================
        // Render Phase
        // =========================================================================
        self.render_world.execute_render();

        // Clear for next frame
        self.render_world.clear_commands();
        self.render_world.clear_views();
    }

    /// Get the framebuffer from Render World
    pub fn framebuffer(&self) -> &[u32] {
        self.render_world.framebuffer()
    }

    /// Get mutable access to Main World
    pub fn main_world_mut(&mut self) -> &mut MainWorld {
        &mut self.main_world
    }

    /// Get mutable access to Render World
    pub fn render_world_mut(&mut self) -> &mut RenderWorld {
        &mut self.render_world
    }

    /// Check if the app is still running
    ///
    /// This is used by the runner to determine when to stop the main loop.
    pub fn is_running(&self) -> bool {
        true
    }
}

/// Trait for types that can be used as schedule labels
pub trait ScheduleLabel {
    fn label(&self) -> &'static str;
}

// Implement ScheduleLabel for references
impl<T: ScheduleLabel> ScheduleLabel for &T {
    fn label(&self) -> &'static str {
        (*self).label()
    }
}

impl ScheduleLabel for Startup {
    fn label(&self) -> &'static str {
        "Startup"
    }
}

impl ScheduleLabel for PreUpdate {
    fn label(&self) -> &'static str {
        "PreUpdate"
    }
}

impl ScheduleLabel for Update {
    fn label(&self) -> &'static str {
        "Update"
    }
}

impl ScheduleLabel for PostUpdate {
    fn label(&self) -> &'static str {
        "PostUpdate"
    }
}

/// Trait for converting types into systems that can be added to the app
pub trait IntoSystems {
    fn add_to_app(self, app: &mut App, schedule: impl ScheduleLabel);
}

impl<S> IntoSystems for S
where
    S: crate::main_world::IntoSystem + 'static,
    S::System: crate::main_world::System + 'static,
{
    fn add_to_app(self, app: &mut App, schedule: impl ScheduleLabel) {
        let system = crate::main_world::IntoSystem::into_system(self);
        let system_box = alloc::boxed::Box::new(system);
        match schedule.label() {
            "Startup" => app.main_world.add_startup_system_boxed(system_box),
            stage => app.main_world.add_boxed_system_to_stage(stage, system_box),
        }
    }
}

macro_rules! impl_into_systems_tuple {
    ($($name:ident),+) => {
        impl<$($name,)+> IntoSystems for ($($name,)+)
        where
            $($name: IntoSystems,)+
        {
            fn add_to_app(self, app: &mut App, schedule: impl ScheduleLabel) {
                let ($($name,)+) = self;
                $($name.add_to_app(app, &schedule);)+
            }
        }
    };
}

impl_into_systems_tuple!(A, B);
impl_into_systems_tuple!(A, B, C);
