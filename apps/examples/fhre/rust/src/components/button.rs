//! Button Component
//!
//! A simple UI button widget for touch/mouse interaction.

use fhre::{Component, Color, Vec2};

// ============================================================
// Button State
// ============================================================

/// Visual state of the button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// Normal unpressed state
    Normal,
    /// Mouse/touch hovering over button
    Hover,
    /// Button is pressed down
    Pressed,
    /// Button is disabled and cannot be interacted with
    Disabled,
}

// ============================================================
// Default Values
// ============================================================

/// Default button width in pixels
const DEFAULT_WIDTH: f32 = 120.0;

/// Default button height in pixels
const DEFAULT_HEIGHT: f32 = 40.0;

/// Default border width in pixels
const DEFAULT_BORDER_WIDTH: f32 = 2.0;

/// Default corner radius in pixels
const DEFAULT_CORNER_RADIUS: f32 = 8.0;

// ============================================================
// Button Component
// ============================================================

/// A UI button component with multiple visual states
#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    /// Button width in pixels
    pub width: f32,
    /// Button height in pixels
    pub height: f32,
    /// Current visual state
    pub state: ButtonState,
    /// Color when in normal state
    pub normal_color: Color,
    /// Color when hovered
    pub hover_color: Color,
    /// Color when pressed
    pub pressed_color: Color,
    /// Color when disabled
    pub disabled_color: Color,
    /// Border line color
    pub border_color: Color,
    /// Border line width
    pub border_width: f32,
    /// Corner rounding radius
    pub corner_radius: f32,
    /// Text label
    pub text: &'static str,
    /// Text color
    pub text_color: Color,
}

impl Button {
    /// Create a new button with the given dimensions
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            state: ButtonState::Normal,
            normal_color: Color::rgb(70, 130, 180),
            hover_color: Color::rgb(100, 160, 210),
            pressed_color: Color::rgb(50, 100, 150),
            disabled_color: Color::rgb(150, 150, 150),
            border_color: Color::rgb(200, 200, 200),
            border_width: DEFAULT_BORDER_WIDTH,
            corner_radius: DEFAULT_CORNER_RADIUS,
            text: "Button",
            text_color: Color::WHITE,
        }
    }

    /// Set the button text label
    pub fn with_text(mut self, text: &'static str) -> Self {
        self.text = text;
        self
    }

    /// Set the colors for normal, hover, and pressed states
    pub fn with_colors(mut self, normal: Color, hover: Color, pressed: Color) -> Self {
        self.normal_color = normal;
        self.hover_color = hover;
        self.pressed_color = pressed;
        self
    }

    /// Get the current color based on state
    pub fn current_color(&self) -> Color {
        match self.state {
            ButtonState::Normal => self.normal_color,
            ButtonState::Hover => self.hover_color,
            ButtonState::Pressed => self.pressed_color,
            ButtonState::Disabled => self.disabled_color,
        }
    }

    /// Set the button state
    pub fn set_state(&mut self, state: ButtonState) {
        self.state = state;
    }

    /// Check if a point is inside the button bounds
    pub fn contains_point(&self, button_pos: Vec2, point: Vec2) -> bool {
        let half_w = self.width / 2.0;
        let half_h = self.height / 2.0;
        
        point.x >= button_pos.x - half_w
            && point.x <= button_pos.x + half_w
            && point.y >= button_pos.y - half_h
            && point.y <= button_pos.y + half_h
    }

    /// Get the four corners of the button rectangle
    /// Returns (top-left, top-right, bottom-left, bottom-right)
    pub fn get_rect(&self, center_pos: Vec2) -> (Vec2, Vec2, Vec2, Vec2) {
        let half_w = self.width / 2.0;
        let half_h = self.height / 2.0;

        let tl = Vec2::new(center_pos.x - half_w, center_pos.y - half_h);
        let tr = Vec2::new(center_pos.x + half_w, center_pos.y - half_h);
        let bl = Vec2::new(center_pos.x - half_w, center_pos.y + half_h);
        let br = Vec2::new(center_pos.x + half_w, center_pos.y + half_h);

        (tl, tr, bl, br)
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    }
}

impl Component for Button {
    fn type_name() -> &'static str {
        "Button"
    }
}
