//! Button Component
//!
//! A UI button widget with built-in interaction state.

use fhre::{Component, Color, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
    Disabled,
}

const DEFAULT_WIDTH: f32 = 120.0;
const DEFAULT_HEIGHT: f32 = 40.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    pub width: f32,
    pub height: f32,
    pub state: ButtonState,
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
    pub disabled_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub corner_radius: f32,
    pub text: &'static str,
    pub text_color: Color,
    
    pub clicked: bool,
}

impl Button {
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
            border_width: 2.0,
            corner_radius: 8.0,
            text: "Button",
            text_color: Color::WHITE,
            clicked: false,
        }
    }

    pub fn with_text(mut self, text: &'static str) -> Self {
        self.text = text;
        self
    }

    pub fn with_colors(mut self, normal: Color, hover: Color, pressed: Color) -> Self {
        self.normal_color = normal;
        self.hover_color = hover;
        self.pressed_color = pressed;
        self
    }

    pub fn current_color(&self) -> Color {
        match self.state {
            ButtonState::Normal => self.normal_color,
            ButtonState::Hover => self.hover_color,
            ButtonState::Pressed => self.pressed_color,
            ButtonState::Disabled => self.disabled_color,
        }
    }

    pub fn contains_point(&self, button_pos: Vec2, point: Vec2) -> bool {
        let half_w = self.width / 2.0;
        let half_h = self.height / 2.0;
        
        point.x >= button_pos.x - half_w
            && point.x <= button_pos.x + half_w
            && point.y >= button_pos.y - half_h
            && point.y <= button_pos.y + half_h
    }

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
