//! Time Resource Implementation
//!
//! Provides time-related information for systems.

/// Time resource - Global time information
///
/// This resource is automatically updated each frame.
#[derive(Clone, Debug)]
pub struct Time {
    /// Time since startup (seconds)
    elapsed: f32,
    /// Time since last frame (seconds)
    delta: f32,
    /// Current frame number
    frame: u64,
    /// Time scale (1.0 = normal, 0.5 = half speed, 2.0 = double speed)
    scale: f32,
    /// Is the game paused?
    paused: bool,
    /// FPS counter
    fps: f32,
    /// Frame time accumulator for FPS calculation
    fps_accumulator: f32,
    /// Frame count for FPS calculation
    fps_frame_count: u32,
}

impl Time {
    /// Create a new Time resource
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            delta: 0.0,
            frame: 0,
            scale: 1.0,
            paused: false,
            fps: 0.0,
            fps_accumulator: 0.0,
            fps_frame_count: 0,
        }
    }

    /// Update time for a new frame
    pub fn update(&mut self, delta_seconds: f32) {
        let actual_delta = if self.paused { 0.0 } else { delta_seconds * self.scale };
        
        self.delta = actual_delta;
        self.elapsed += actual_delta;
        self.frame += 1;

        // Update FPS counter
        self.fps_accumulator += delta_seconds;
        self.fps_frame_count += 1;
        
        if self.fps_accumulator >= 1.0 {
            self.fps = self.fps_frame_count as f32 / self.fps_accumulator;
            self.fps_accumulator = 0.0;
            self.fps_frame_count = 0;
        }
    }

    /// Get elapsed time since startup
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Get delta time (time since last frame)
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Get current frame number
    pub fn frame(&self) -> u64 {
        self.frame
    }

    /// Get time scale
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Set time scale
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.0);
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Pause the game
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the game
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Get current FPS
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Get smoothed delta time (for more stable physics)
    pub fn delta_smooth(&self) -> f32 {
        // Clamp delta to avoid large time steps
        self.delta.min(0.1)
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.delta = 0.0;
        self.frame = 0;
        self.fps = 0.0;
        self.fps_accumulator = 0.0;
        self.fps_frame_count = 0;
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Resource for Time {}

/// Timer - A simple countdown timer
#[derive(Clone, Debug)]
pub struct Timer {
    /// Total duration
    duration: f32,
    /// Remaining time
    remaining: f32,
    /// Is the timer running?
    running: bool,
    /// Repeat when finished?
    repeating: bool,
}

impl Timer {
    /// Create a new timer with duration
    pub fn new(duration_seconds: f32) -> Self {
        Self {
            duration: duration_seconds,
            remaining: duration_seconds,
            running: false,
            repeating: false,
        }
    }

    /// Create a repeating timer
    pub fn repeating(duration_seconds: f32) -> Self {
        Self {
            duration: duration_seconds,
            remaining: duration_seconds,
            running: false,
            repeating: true,
        }
    }

    /// Start the timer
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Stop the timer
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.remaining = self.duration;
    }

    /// Tick the timer (call each frame)
    pub fn tick(&mut self, delta: f32) -> bool {
        if !self.running {
            return false;
        }

        self.remaining -= delta;

        if self.remaining <= 0.0 {
            if self.repeating {
                self.remaining = self.duration;
            } else {
                self.remaining = 0.0;
                self.running = false;
            }
            true // Timer finished
        } else {
            false // Timer still running
        }
    }

    /// Check if timer is finished
    pub fn is_finished(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Get remaining time
    pub fn remaining(&self) -> f32 {
        self.remaining
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> f32 {
        self.duration - self.remaining
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed() / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Check if timer is running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(1.0)
    }
}
