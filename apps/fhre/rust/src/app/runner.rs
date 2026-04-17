//! App Runner Implementation
//!
//! Provides the runner trait and implementations for controlling the app lifecycle.
//! This enables Bevy-style `app.run()` blocking execution.

use super::App;
use alloc::boxed::Box;

/// Trait for types that can run an App
///
/// This is similar to Bevy's runner system. Plugins can set a custom runner
/// to control the application's main loop.
///
/// # Example
///
/// ```rust
/// pub struct WinitRunner;
///
/// impl AppRunner for WinitRunner {
///     fn run(self: Box<Self>, mut app: App) -> AppExit {
///         // Event loop implementation
///         loop {
///             if !app.is_running() {
///                 break AppExit::Success;
///             }
///             app.update();
///         }
///     }
/// }
/// ```
pub trait AppRunner: Send + Sync {
    /// Run the application
    ///
    /// This method takes ownership of the App and runs it until completion.
    /// The runner is responsible for:
    /// - Setting up the event loop
    /// - Calling app.update() each frame
    /// - Handling window events
    /// - Managing frame timing
    fn run(self: Box<Self>, app: App) -> AppExit;
}

/// Application exit status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppExit {
    /// Application exited successfully
    Success,
    /// Application exited with an error
    Failure(i32),
}

impl AppExit {
    /// Check if the exit was successful
    pub fn is_success(&self) -> bool {
        matches!(self, AppExit::Success)
    }

    /// Get the exit code
    pub fn code(&self) -> i32 {
        match self {
            AppExit::Success => 0,
            AppExit::Failure(code) => *code,
        }
    }
}

/// Default runner that runs the app once
///
/// This is the default runner that simply calls app.update() once.
/// It's useful for testing and headless applications.
pub struct RunOnceRunner;

impl AppRunner for RunOnceRunner {
    fn run(self: Box<Self>, mut app: App) -> AppExit {
        app.update();
        AppExit::Success
    }
}

/// Headless runner that runs at a fixed timestep
///
/// This runner runs the app at a fixed timestep without any window.
/// It's useful for server applications or testing.
pub struct FixedTimestepRunner {
    /// Target updates per second
    pub updates_per_second: u32,
    /// Maximum number of updates to run (None = infinite)
    pub max_updates: Option<u32>,
}

impl FixedTimestepRunner {
    /// Create a new fixed timestep runner
    pub fn new(updates_per_second: u32) -> Self {
        Self {
            updates_per_second,
            max_updates: None,
        }
    }

    /// Set the maximum number of updates
    pub fn with_max_updates(mut self, max: u32) -> Self {
        self.max_updates = Some(max);
        self
    }
}

impl AppRunner for FixedTimestepRunner {
    fn run(self: Box<Self>, mut app: App) -> AppExit {
        let target_delta = 1.0 / self.updates_per_second as f32;
        let mut update_count = 0u32;

        loop {
            if !app.is_running() {
                return AppExit::Success;
            }

            if let Some(max) = self.max_updates {
                if update_count >= max {
                    return AppExit::Success;
                }
            }

            app.update();
            update_count += 1;

            // Frame timing
            let elapsed = unsafe {
                extern "C" { fn clock() -> i64; }
                clock()
            };
            let elapsed_secs = elapsed as f32 / 1_000_000.0;
            if elapsed_secs < target_delta {
                let sleep_us = ((target_delta - elapsed_secs) * 1_000_000.0) as u32;
                unsafe {
                    extern "C" { fn usleep(usec: u32) -> i32; }
                    usleep(sleep_us);
                }
            }
        }
    }
}

/// Type alias for runner function
pub type RunnerFn = Box<dyn AppRunner>;

/// Default runner function
pub fn run_once_runner() -> RunnerFn {
    Box::new(RunOnceRunner)
}
