//! Style 组件
//!
//! 提供类似 LVGL 的样式系统，用于控制节点的外观。

use crate::Component;
use crate::math::{Color, Vec2};

/// 尺寸类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    /// 自动（根据内容）
    Auto,
    /// 固定像素
    Pixel(f32),
    /// 百分比（相对于父节点）
    Percent(f32),
    /// 填充剩余空间
    Fill,
    /// 根据内容调整
    FitContent,
}

impl Default for Dimension {
    fn default() -> Self {
        Dimension::Auto
    }
}

/// 矩形结构（用于边距、内边距等）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T: Copy> Rect<T> {
    /// 创建统一值的矩形
    pub fn uniform(value: T) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    /// 创建水平/垂直对称的矩形
    pub fn symmetric(horizontal: T, vertical: T) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }
}

impl Rect<f32> {
    /// 零矩形
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    /// 获取水平总和
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// 获取垂直总和
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// 边框样式
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Border {
    /// 边框宽度
    pub width: f32,
    /// 边框颜色
    pub color: Color,
    /// 圆角半径
    pub radius: f32,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0,
        }
    }
}

/// 阴影样式
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    /// 阴影颜色
    pub color: Color,
    /// 阴影偏移
    pub offset: Vec2,
    /// 阴影模糊半径
    pub blur: f32,
    /// 阴影扩散半径
    pub spread: f32,
}

impl Default for Shadow {
    fn default() -> Self {
        Self {
            color: Color::TRANSPARENT,
            offset: Vec2::ZERO,
            blur: 0.0,
            spread: 0.0,
        }
    }
}

impl Shadow {
    /// 创建默认阴影
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置阴影颜色
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// 设置阴影偏移
    pub fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Vec2::new(x, y);
        self
    }

    /// 设置阴影模糊
    pub fn with_blur(mut self, blur: f32) -> Self {
        self.blur = blur;
        self
    }
}

/// 文本样式
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// 字体大小
    pub font_size: f32,
    /// 字体颜色
    pub color: Color,
    /// 字体家族
    pub font_family: Option<alloc::string::String>,
    /// 行高
    pub line_height: f32,
    /// 字间距
    pub letter_spacing: f32,
    /// 对齐方式
    pub alignment: TextAlignment,
    /// 是否粗体
    pub bold: bool,
    /// 是否斜体
    pub italic: bool,
    /// 下划线
    pub underline: bool,
    /// 删除线
    pub strikethrough: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            color: Color::BLACK,
            font_family: None,
            line_height: 1.2,
            letter_spacing: 0.0,
            alignment: TextAlignment::Left,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }
}

/// 背景样式
#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    /// 纯色背景
    Solid(Color),
    /// 渐变背景
    Gradient(Gradient),
    /// 图像背景
    Image(ImageBackground),
    /// 无背景
    None,
}

impl Default for Background {
    fn default() -> Self {
        Background::None
    }
}

/// 渐变
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gradient {
    /// 起始颜色
    pub start_color: Color,
    /// 结束颜色
    pub end_color: Color,
    /// 渐变角度（度）
    pub angle: f32,
    /// 渐变类型
    pub gradient_type: GradientType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientType {
    Linear,
    Radial,
}

/// 图像背景
#[derive(Debug, Clone, PartialEq)]
pub struct ImageBackground {
    /// 图像路径或句柄
    pub source: alloc::string::String,
    /// 缩放模式
    pub scaling: ImageScaling,
    /// 重复模式
    pub repeat: ImageRepeat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageScaling {
    Fill,
    Fit,
    Stretch,
    Tile,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageRepeat {
    NoRepeat,
    Repeat,
    RepeatX,
    RepeatY,
}

/// 样式组件
///
/// 控制节点的外观，类似 CSS/LVGL 的样式系统
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    // ==================== 尺寸 ====================
    /// 宽度
    pub width: Dimension,
    /// 高度
    pub height: Dimension,
    /// 最小宽度
    pub min_width: Dimension,
    /// 最小高度
    pub min_height: Dimension,
    /// 最大宽度
    pub max_width: Dimension,
    /// 最大高度
    pub max_height: Dimension,
    /// 宽高比（保持比例）
    pub aspect_ratio: Option<f32>,

    // ==================== 间距 ====================
    /// 外边距（与父节点的间距）
    pub margin: Rect<f32>,
    /// 内边距（与内容的间距）
    pub padding: Rect<f32>,

    // ==================== 背景 ====================
    /// 背景
    pub background: Background,

    // ==================== 边框 ====================
    /// 边框
    pub border: Border,

    // ==================== 阴影 ====================
    /// 阴影
    pub shadow: Shadow,

    // ==================== 文本 ====================
    /// 文本样式
    pub text: TextStyle,

    // ==================== 变换 ====================
    /// 不透明度 (0.0 - 1.0)
    pub opacity: f32,
    /// 可见性
    pub visible: bool,
    /// 裁剪溢出内容
    pub overflow_clip: bool,
}

impl Style {
    /// 创建默认样式
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建 UI 控件默认样式
    pub fn ui_default() -> Self {
        Self {
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_width: Dimension::Auto,
            max_height: Dimension::Auto,
            aspect_ratio: None,
            margin: Rect::ZERO,
            padding: Rect::uniform(8.0),
            background: Background::Solid(Color::WHITE),
            border: Border::default(),
            shadow: Shadow::default(),
            text: TextStyle::default(),
            opacity: 1.0,
            visible: true,
            overflow_clip: false,
        }
    }

    /// 设置宽度
    pub fn with_width(mut self, width: Dimension) -> Self {
        self.width = width;
        self
    }

    /// 设置高度
    pub fn with_height(mut self, height: Dimension) -> Self {
        self.height = height;
        self
    }

    /// 设置固定尺寸
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = Dimension::Pixel(width);
        self.height = Dimension::Pixel(height);
        self
    }

    /// 设置外边距
    pub fn with_margin(mut self, margin: Rect<f32>) -> Self {
        self.margin = margin;
        self
    }

    /// 设置内边距
    pub fn with_padding(mut self, padding: Rect<f32>) -> Self {
        self.padding = padding;
        self
    }

    /// 设置背景颜色
    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background = Background::Solid(color);
        self
    }

    /// 设置边框
    pub fn with_border(mut self, width: f32, color: Color) -> Self {
        self.border = Border {
            width,
            color,
            radius: 0.0,
        };
        self
    }

    /// 设置圆角
    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border.radius = radius;
        self
    }

    /// 设置阴影
    pub fn with_shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = shadow;
        self
    }

    /// 设置文本颜色
    pub fn with_text_color(mut self, color: Color) -> Self {
        self.text.color = color;
        self
    }

    /// 设置字体大小
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.text.font_size = size;
        self
    }

    /// 设置不透明度
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// 设置可见性
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// 计算实际宽度
    pub fn compute_width(&self, parent_width: f32, content_width: f32) -> f32 {
        let width = match self.width {
            Dimension::Auto => content_width,
            Dimension::Pixel(w) => w,
            Dimension::Percent(p) => parent_width * p / 100.0,
            Dimension::Fill => parent_width, // 简化处理
            Dimension::FitContent => content_width,
        };

        // 应用最小/最大宽度限制
        let min_width = match self.min_width {
            Dimension::Auto => 0.0,
            Dimension::Pixel(w) => w,
            Dimension::Percent(p) => parent_width * p / 100.0,
            _ => 0.0,
        };

        let max_width = match self.max_width {
            Dimension::Auto => f32::MAX,
            Dimension::Pixel(w) => w,
            Dimension::Percent(p) => parent_width * p / 100.0,
            _ => f32::MAX,
        };

        width.max(min_width).min(max_width)
    }

    /// 计算实际高度
    pub fn compute_height(&self, parent_height: f32, content_height: f32) -> f32 {
        let height = match self.height {
            Dimension::Auto => content_height,
            Dimension::Pixel(h) => h,
            Dimension::Percent(p) => parent_height * p / 100.0,
            Dimension::Fill => parent_height,
            Dimension::FitContent => content_height,
        };

        // 应用最小/最大高度限制
        let min_height = match self.min_height {
            Dimension::Auto => 0.0,
            Dimension::Pixel(h) => h,
            Dimension::Percent(p) => parent_height * p / 100.0,
            _ => 0.0,
        };

        let max_height = match self.max_height {
            Dimension::Auto => f32::MAX,
            Dimension::Pixel(h) => h,
            Dimension::Percent(p) => parent_height * p / 100.0,
            _ => f32::MAX,
        };

        height.max(min_height).min(max_height)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_width: Dimension::Auto,
            max_height: Dimension::Auto,
            aspect_ratio: None,
            margin: Rect::ZERO,
            padding: Rect::ZERO,
            background: Background::None,
            border: Border::default(),
            shadow: Shadow::default(),
            text: TextStyle::default(),
            opacity: 1.0,
            visible: true,
            overflow_clip: false,
        }
    }
}

impl Component for Style {
    fn type_name() -> &'static str {
        "Style"
    }
}
