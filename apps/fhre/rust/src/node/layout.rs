//! Layout 组件
//!
//! 提供 Flexbox 和 Grid 布局系统，支持 2D 和 3D 布局。

use crate::Component;
use crate::math::Vec2;
use alloc::vec::Vec;

/// 布局类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutType {
    /// 无布局（绝对定位）
    None,
    /// Flexbox 布局
    Flex,
    /// Grid 布局
    Grid,
    /// 堆叠布局（子节点重叠）
    Stack,
    /// 列表布局（垂直或水平）
    List(ListDirection),
}

impl Default for LayoutType {
    fn default() -> Self {
        LayoutType::None
    }
}

/// 列表方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListDirection {
    Vertical,
    Horizontal,
}

/// Flex 方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl Default for FlexDirection {
    fn default() -> Self {
        FlexDirection::Row
    }
}

/// 主轴对齐方式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        JustifyContent::Start
    }
}

/// 交叉轴对齐方式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

impl Default for AlignItems {
    fn default() -> Self {
        AlignItems::Stretch
    }
}

/// 换行方式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl Default for FlexWrap {
    fn default() -> Self {
        FlexWrap::NoWrap
    }
}

/// 布局组件
///
/// 控制子节点的排列方式
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// 布局类型
    pub layout_type: LayoutType,
    /// Flex 方向
    pub flex_direction: FlexDirection,
    /// 主轴对齐
    pub justify_content: JustifyContent,
    /// 交叉轴对齐
    pub align_items: AlignItems,
    /// 换行
    pub flex_wrap: FlexWrap,
    /// 间距
    pub gap: Vec2,
    /// 行间距（多行时）
    pub row_gap: f32,
    /// 列间距（多列时）
    pub column_gap: f32,
    /// 是否启用 3D 布局
    pub use_3d_layout: bool,
    /// 3D 深度间距
    pub depth_gap: f32,
}

impl Layout {
    /// 创建默认布局
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建 Flex 布局
    pub fn flex() -> Self {
        Self {
            layout_type: LayoutType::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
            gap: Vec2::ZERO,
            row_gap: 0.0,
            column_gap: 0.0,
            use_3d_layout: false,
            depth_gap: 0.0,
        }
    }

    /// 创建垂直列表布局
    pub fn column() -> Self {
        Self {
            layout_type: LayoutType::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
            gap: Vec2::ZERO,
            row_gap: 0.0,
            column_gap: 0.0,
            use_3d_layout: false,
            depth_gap: 0.0,
        }
    }

    /// 创建水平列表布局
    pub fn row() -> Self {
        Self {
            layout_type: LayoutType::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
            gap: Vec2::ZERO,
            row_gap: 0.0,
            column_gap: 0.0,
            use_3d_layout: false,
            depth_gap: 0.0,
        }
    }

    /// 创建网格布局
    pub fn grid() -> Self {
        Self {
            layout_type: LayoutType::Grid,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::Wrap,
            gap: Vec2::ZERO,
            row_gap: 0.0,
            column_gap: 0.0,
            use_3d_layout: false,
            depth_gap: 0.0,
        }
    }

    /// 创建堆叠布局
    pub fn stack() -> Self {
        Self {
            layout_type: LayoutType::Stack,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_wrap: FlexWrap::NoWrap,
            gap: Vec2::ZERO,
            row_gap: 0.0,
            column_gap: 0.0,
            use_3d_layout: false,
            depth_gap: 0.0,
        }
    }

    /// 设置 Flex 方向
    pub fn with_direction(mut self, direction: FlexDirection) -> Self {
        self.flex_direction = direction;
        self
    }

    /// 设置主轴对齐
    pub fn with_justify(mut self, justify: JustifyContent) -> Self {
        self.justify_content = justify;
        self
    }

    /// 设置交叉轴对齐
    pub fn with_align(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }

    /// 设置间距
    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = Vec2::splat(gap);
        self
    }

    /// 设置行列间距
    pub fn with_gap_xy(mut self, x: f32, y: f32) -> Self {
        self.gap = Vec2::new(x, y);
        self
    }

    /// 启用换行
    pub fn with_wrap(mut self) -> Self {
        self.flex_wrap = FlexWrap::Wrap;
        self
    }

    /// 启用 3D 布局
    pub fn with_3d(mut self, depth_gap: f32) -> Self {
        self.use_3d_layout = true;
        self.depth_gap = depth_gap;
        self
    }

    /// 计算子节点位置（简化版）
    pub fn calculate_positions(
        &self,
        container_size: Vec2,
        child_sizes: &[Vec2],
    ) -> Vec<Vec2> {
        match self.layout_type {
            LayoutType::Flex => self.calculate_flex_positions(container_size, child_sizes),
            LayoutType::Stack => self.calculate_stack_positions(container_size, child_sizes),
            LayoutType::List(direction) => {
                self.calculate_list_positions(container_size, child_sizes, direction)
            }
            _ => child_sizes.iter().map(|_| Vec2::ZERO).collect(),
        }
    }

    fn calculate_flex_positions(&self, container_size: Vec2, child_sizes: &[Vec2]) -> Vec<Vec2> {
        let mut positions = Vec::new();
        let mut current_pos = Vec2::ZERO;

        let is_row = matches!(
            self.flex_direction,
            FlexDirection::Row | FlexDirection::RowReverse
        );

        let gap = if is_row { self.gap.x } else { self.gap.y };

        for (i, size) in child_sizes.iter().enumerate() {
            if i > 0 {
                if is_row {
                    current_pos.x += gap;
                } else {
                    current_pos.y += gap;
                }
            }

            positions.push(current_pos);

            if is_row {
                current_pos.x += size.x;
            } else {
                current_pos.y += size.y;
            }
        }

        // 应用对齐
        self.apply_alignment(&mut positions, container_size, child_sizes, is_row);

        positions
    }

    fn calculate_stack_positions(&self, container_size: Vec2, child_sizes: &[Vec2]) -> Vec<Vec2> {
        let center = Vec2::new(container_size.x * 0.5, container_size.y * 0.5);

        child_sizes
            .iter()
            .map(|size| Vec2::new(center.x - size.x * 0.5, center.y - size.y * 0.5))
            .collect()
    }

    fn calculate_list_positions(
        &self,
        _container_size: Vec2,
        child_sizes: &[Vec2],
        direction: ListDirection,
    ) -> Vec<Vec2> {
        let mut positions = Vec::new();
        let mut current_pos = Vec2::ZERO;

        for size in child_sizes.iter() {
            positions.push(current_pos);

            match direction {
                ListDirection::Vertical => {
                    current_pos.y += size.y + self.gap.y;
                }
                ListDirection::Horizontal => {
                    current_pos.x += size.x + self.gap.x;
                }
            }
        }

        positions
    }

    fn apply_alignment(
        &self,
        positions: &mut [Vec2],
        container_size: Vec2,
        child_sizes: &[Vec2],
        is_row: bool,
    ) {
        if positions.is_empty() || child_sizes.is_empty() {
            return;
        }

        let last_pos = positions.last().unwrap_or(&Vec2::ZERO);
        let last_size = child_sizes.last().unwrap_or(&Vec2::ZERO);

        // 计算内容总尺寸
        let total_content_size = if is_row {
            last_pos.x + last_size.x
        } else {
            last_pos.y + last_size.y
        };

        // 计算偏移量
        let offset = match self.justify_content {
            JustifyContent::Start => 0.0,
            JustifyContent::Center => {
                if is_row {
                    (container_size.x - total_content_size) * 0.5
                } else {
                    (container_size.y - total_content_size) * 0.5
                }
            }
            JustifyContent::End => {
                if is_row {
                    container_size.x - total_content_size
                } else {
                    container_size.y - total_content_size
                }
            }
            _ => 0.0, // 其他对齐方式简化处理
        };

        // 应用偏移
        for pos in positions.iter_mut() {
            if is_row {
                pos.x += offset;
            } else {
                pos.y += offset;
            }
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            layout_type: LayoutType::None,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
            gap: Vec2::ZERO,
            row_gap: 0.0,
            column_gap: 0.0,
            use_3d_layout: false,
            depth_gap: 0.0,
        }
    }
}

impl Component for Layout {
    fn type_name() -> &'static str {
        "Layout"
    }
}

/// 布局结果（由布局系统计算）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutResult {
    /// 计算后的位置
    pub position: Vec2,
    /// 计算后的尺寸
    pub size: Vec2,
}

impl Default for LayoutResult {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::ZERO,
        }
    }
}

impl Component for LayoutResult {
    fn type_name() -> &'static str {
        "LayoutResult"
    }
}
