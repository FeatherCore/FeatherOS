//! Simple Button Component
//!
//! 简单的按钮控件，用于 UI 交互。

use crate::Component;
use crate::math::{Color, Vec2};

/// 按钮状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    /// 正常状态
    Normal,
    /// 悬停状态
    Hover,
    /// 按下状态
    Pressed,
    /// 禁用状态
    Disabled,
}

/// 简单按钮组件
#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    /// 按钮宽度
    pub width: f32,
    /// 按钮高度
    pub height: f32,
    /// 当前状态
    pub state: ButtonState,
    /// 正常状态颜色
    pub normal_color: Color,
    /// 悬停状态颜色
    pub hover_color: Color,
    /// 按下状态颜色
    pub pressed_color: Color,
    /// 禁用状态颜色
    pub disabled_color: Color,
    /// 边框颜色
    pub border_color: Color,
    /// 边框宽度
    pub border_width: f32,
    /// 圆角半径
    pub corner_radius: f32,
    /// 按钮文本
    pub text: &'static str,
    /// 文本颜色
    pub text_color: Color,
}

impl Button {
    /// 创建新按钮
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            state: ButtonState::Normal,
            normal_color: Color::rgb(70, 130, 180),    // 钢蓝色
            hover_color: Color::rgb(100, 160, 210),    // 亮蓝色
            pressed_color: Color::rgb(50, 100, 150),   // 深蓝色
            disabled_color: Color::rgb(150, 150, 150), // 灰色
            border_color: Color::rgb(200, 200, 200),   // 浅灰边框
            border_width: 2.0,
            corner_radius: 8.0,
            text: "Button",
            text_color: Color::WHITE,
        }
    }

    /// 设置文本
    pub fn with_text(mut self, text: &'static str) -> Self {
        self.text = text;
        self
    }

    /// 设置颜色
    pub fn with_colors(mut self, normal: Color, hover: Color, pressed: Color) -> Self {
        self.normal_color = normal;
        self.hover_color = hover;
        self.pressed_color = pressed;
        self
    }

    /// 获取当前状态的颜色
    pub fn current_color(&self) -> Color {
        match self.state {
            ButtonState::Normal => self.normal_color,
            ButtonState::Hover => self.hover_color,
            ButtonState::Pressed => self.pressed_color,
            ButtonState::Disabled => self.disabled_color,
        }
    }

    /// 设置状态
    pub fn set_state(&mut self, state: ButtonState) {
        self.state = state;
    }

    /// 检查点是否在按钮内
    pub fn contains_point(&self, button_pos: Vec2, point: Vec2) -> bool {
        let half_w = self.width / 2.0;
        let half_h = self.height / 2.0;
        
        point.x >= button_pos.x - half_w
            && point.x <= button_pos.x + half_w
            && point.y >= button_pos.y - half_h
            && point.y <= button_pos.y + half_h
    }

    /// 获取按钮矩形（用于渲染）
    pub fn get_rect(&self, center_pos: Vec2) -> (Vec2, Vec2, Vec2, Vec2) {
        let half_w = self.width / 2.0;
        let half_h = self.height / 2.0;

        let tl = Vec2::new(center_pos.x - half_w, center_pos.y - half_h); // 左上
        let tr = Vec2::new(center_pos.x + half_w, center_pos.y - half_h); // 右上
        let bl = Vec2::new(center_pos.x - half_w, center_pos.y + half_h); // 左下
        let br = Vec2::new(center_pos.x + half_w, center_pos.y + half_h); // 右下

        (tl, tr, bl, br)
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new(120.0, 40.0)
    }
}

impl Component for Button {
    fn type_name() -> &'static str {
        "Button"
    }
}
