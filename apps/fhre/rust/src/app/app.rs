//! App Module
//!
//! The main application orchestrator that coordinates Main World and Render World.
//! Aligned with Bevy's App design - provides plugin system and schedule management.

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::resources::{Time, PrimaryScreen};
use crate::plugin::{Plugin, PluginGroup};
use crate::extract::Extractors;
use crate::schedule::ScheduleLabel;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Default target FPS for the application
pub const DEFAULT_FPS: f32 = 60.0;
/// Default frame time in seconds
pub const DEFAULT_FRAME_TIME: f32 = 1.0 / DEFAULT_FPS;

/// FHRE Version
pub const FHRE_VERSION: &[u8] = b"2.8.0\0";

/// The main application struct
///
/// Similar to Bevy's App, this orchestrates the entire application lifecycle.
/// It manages plugins, schedules, and coordinates between Main World and Render World.
///
/// # Usage
///
/// ```ignore
/// let mut app = App::new(640, 480);
/// app.add_plugins(DefaultPlugins)
///     .insert_resource(MyResource::new())
///     .add_systems(Startup, setup)
///     .add_systems(Update, my_system);
///
/// // Main loop controlled by user (e.g., WindowRunner)
/// loop {
///     app.update_and_render();
///     window.present(app.framebuffer());
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
    
    /// Extract systems - run during extract phase
    extractors: Extractors,
}

impl App {
    /// Create a new App with the specified screen dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let mut app = Self {
            main_world: MainWorld::new(),
            render_world: RenderWorld::new(width, height),
            plugins: Vec::new(),
            initialized: false,
            extractors: Extractors::new(),
        };
        
        app.insert_resource(Time::default());
        app.insert_resource(PrimaryScreen::new(width, height));
        app.insert_resource(crate::sync::PendingSyncEntity::new());
        
        app
    }
    
    /// Add an extract system
    /// 
    /// Extract systems run during the extract phase, after Main World systems
    /// and before rendering. They sync data from Main World to Render World.
    pub fn add_extractor<F>(&mut self, extractor: F) -> &mut Self
    where
        F: Fn(&MainWorld, &mut RenderWorld) + Send + Sync + 'static,
    {
        self.extractors.add(extractor);
        self
    }

    /// Add a plugin to the app
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    /// Add a boxed plugin to the app
    pub fn add_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) -> &mut Self {
        if !self.initialized {
            self.plugins.push(plugin);
        } else {
            plugin.build(self);
        }
        self
    }

    /// Add a group of plugins
    pub fn add_plugins<P: PluginGroup>(&mut self, group: P) -> &mut Self {
        let builder = group.build();
        builder.finish(self);
        self
    }

    /// Insert a resource into the Main World
    pub fn insert_resource<R: crate::resources::Resource>(&mut self, resource: R) -> &mut Self {
        self.main_world.resources_mut().insert(resource);
        self
    }

    /// Add systems to a schedule
    pub fn add_systems(&mut self, _schedule: impl ScheduleLabel, systems: impl IntoSystems) -> &mut Self {
        systems.add_to_app(self, _schedule);
        self
    }

    /// Initialize all plugins
    fn initialize_plugins(&mut self) {
        if self.initialized {
            return;
        }

        let plugins: Vec<Box<dyn Plugin>> = self.plugins.drain(..).collect();
        
        for plugin in plugins {
            plugin.build(self);
        }

        self.initialized = true;
    }

    /// Run a single frame: systems + extract + render
    ///
    /// This is the main entry point for each frame.
    /// Call this from your main loop (e.g., WindowRunner).
    pub fn update_and_render(&mut self) {
        self.initialize_plugins();

        // Run startup systems (only on first frame)
        self.main_world.run_startup_systems();

        // Update time
        if let Some(time) = self.main_world.resources_mut().get_mut::<Time>() {
            time.update(DEFAULT_FRAME_TIME);
        }

        // Run Main World systems
        self.main_world.run_systems();

        // Update Events buffers before extract (so startup events are visible)
        if let Some(events) = self.main_world.resources_mut().get_mut::<crate::event::Events>() {
            events.update();
        }

        // Extract Phase - sync Main World to Render World
        crate::sync::entity_sync_system(&mut self.main_world, &mut self.render_world);
        
        self.render_world.clear_views();
        self.extractors.run(&self.main_world, &mut self.render_world);

        // Render Phase
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
        let label_name = schedule.name();
        match label_name.as_str() {
            "Startup" => app.main_world.add_startup_system_boxed(system_box),
            stage => app.main_world.add_boxed_system_to_stage(stage, system_box),
        }
    }
}

#[allow(non_snake_case)]
macro_rules! impl_into_systems_tuple {
    ($($name:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($name,)+> IntoSystems for ($($name,)+)
        where
            $($name: IntoSystems,)+
        {
            fn add_to_app(self, app: &mut App, schedule: impl ScheduleLabel) {
                let ($($name,)+) = self;
                $($name.add_to_app(app, schedule.clone());)+
            }
        }
    };
}

impl_into_systems_tuple!(A, B);
impl_into_systems_tuple!(A, B, C);
