//! LVGL-aligned 2D widget taxonomy and rendering skeletons.

use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;
use fhre::{Color, RenderCommand, Vec2};
use fhre::math::Rect;

use crate::launcher::AppInfo;
use crate::theme::ThemePalette;

pub type WidgetId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetAction {
    ToggleLauncher,
    CycleTheme,
    ApplySelectedTheme,
    ApplyButtonMatrixSelection,
    CloseWindow,
    MinimizeWindow,
    MaximizeWindow,
    LaunchAppNamed(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetEventKind {
    HoverEnter,
    HoverLeave,
    Press,
    Release,
    Click,
    ToggleChanged,
    ValueChanged,
    SelectionChanged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextEditCommand {
    Insert(char),
    Backspace,
    Delete,
    Newline,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveLineStart,
    MoveLineEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WidgetEvent {
    pub widget_id: WidgetId,
    pub kind: WidgetEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct WidgetInteractionState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct WidgetResponse {
    pub consumed: bool,
    pub event: Option<WidgetEvent>,
    pub action: Option<WidgetAction>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetVisualStyle {
    pub bg: Color,
    pub bg_hover: Color,
    pub bg_pressed: Color,
    pub bg_disabled: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_on_accent: Color,
    pub border: Color,
    pub radius: f32,
}

impl WidgetVisualStyle {
    pub fn from_palette(palette: ThemePalette) -> Self {
        Self {
            bg: palette.surface,
            bg_hover: palette.surface_alt,
            bg_pressed: palette.accent_pressed,
            bg_disabled: palette.overlay,
            text: palette.text,
            text_muted: palette.text_muted,
            text_on_accent: Color::WHITE,
            border: palette.border,
            radius: 10.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum WidgetKind {
    Panel,
    Label { text: &'static str },
    Button { text: &'static str, pressed: bool },
    Image { title: &'static str },
    CheckBox { text: &'static str, checked: bool },
    Switch { on: bool },
    Slider { value: u8 },
    Bar { value: u8 },
    Arc { value: u8 },
    Spinner,
    TextArea { text: String, placeholder: &'static str, cursor: usize, scroll_row: usize },
    List { title: &'static str, items: &'static [&'static str], selected: usize },
    Menu { title: &'static str, items: &'static [&'static str] },
    TabView { tabs: &'static [&'static str], active: usize },
    Table { headers: &'static [&'static str], rows: &'static [&'static [&'static str]] },
    Chart { values: &'static [u8] },
    Canvas { title: &'static str },
    Keyboard { rows: &'static [&'static str] },
    Dropdown { options: &'static [&'static str], selected: usize, expanded: bool },
    Roller { options: &'static [&'static str], selected: usize },
    SpinBox { value: i32 },
    Calendar { month: &'static str },
    MsgBox { title: &'static str, body: &'static str },
    TileView { pages: &'static [&'static str], active: usize },
    ButtonMatrix { labels: &'static [&'static str], columns: usize, selected: Option<usize> },
    Led { on: bool },
    Line,
}

#[derive(Clone, Debug)]
pub struct WidgetNode {
    pub id: WidgetId,
    pub kind: WidgetKind,
    pub rect: Rect,
    pub accent_color: Color,
    pub background_color: Color,
    pub visible: bool,
    pub enabled: bool,
    pub interaction: WidgetInteractionState,
    pub action: Option<WidgetAction>,
    pub style: WidgetVisualStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WidgetLayout {
    Free,
    Vertical { spacing: f32, padding: f32 },
    Horizontal { spacing: f32, padding: f32 },
    Grid { columns: usize, spacing: f32, padding: f32 },
}

#[derive(Clone, Debug)]
pub struct WidgetTreeNode {
    pub widget: WidgetNode,
    pub children: Vec<WidgetTreeNode>,
    pub layout: WidgetLayout,
}

impl WidgetTreeNode {
    pub fn new(widget: WidgetNode) -> Self {
        Self {
            widget,
            children: Vec::new(),
            layout: WidgetLayout::Free,
        }
    }

    pub fn with_layout(mut self, layout: WidgetLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_children(mut self, children: Vec<WidgetTreeNode>) -> Self {
        self.children = children;
        self
    }

    pub fn render(&self, origin: Vec2) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        self.render_into(origin, &mut commands);
        commands
    }

    pub fn render_into(&self, origin: Vec2, commands: &mut Vec<RenderCommand>) {
        let absolute = Vec2::new(origin.x + self.widget.rect.x, origin.y + self.widget.rect.y);
        commands.extend(self.widget.render(origin));
        for child in &self.children {
            child.render_into(absolute, commands);
        }
    }

    pub fn relayout(&mut self) {
        match self.layout {
            WidgetLayout::Free => {}
            WidgetLayout::Vertical { spacing, padding } => {
                let mut y = padding;
                for child in &mut self.children {
                    child.widget.rect.x = padding;
                    child.widget.rect.y = y;
                    child.widget.rect.width = child.widget.rect.width.min((self.widget.rect.width - padding * 2.0).max(0.0));
                    y += child.widget.rect.height + spacing;
                    child.relayout();
                }
                return;
            }
            WidgetLayout::Horizontal { spacing, padding } => {
                let mut x = padding;
                for child in &mut self.children {
                    child.widget.rect.x = x;
                    child.widget.rect.y = padding;
                    child.widget.rect.height = child.widget.rect.height.min((self.widget.rect.height - padding * 2.0).max(0.0));
                    x += child.widget.rect.width + spacing;
                    child.relayout();
                }
                return;
            }
            WidgetLayout::Grid { columns, spacing, padding } => {
                let cols = columns.max(1);
                let cell_w = ((self.widget.rect.width - padding * 2.0) - spacing * (cols.saturating_sub(1)) as f32) / cols as f32;
                for (index, child) in self.children.iter_mut().enumerate() {
                    let col = index % cols;
                    let row = index / cols;
                    child.widget.rect.x = padding + col as f32 * (cell_w + spacing);
                    child.widget.rect.y = padding + row as f32 * (child.widget.rect.height + spacing);
                    child.widget.rect.width = cell_w.max(0.0);
                    child.relayout();
                }
                return;
            }
        }

        for child in &mut self.children {
            child.relayout();
        }
    }

    pub fn hit_test(&self, local_point: Vec2) -> Option<WidgetId> {
        if !self.widget.visible {
            return None;
        }

        let child_local = Vec2::new(local_point.x - self.widget.rect.x, local_point.y - self.widget.rect.y);
        for child in self.children.iter().rev() {
            if let Some(id) = child.hit_test(child_local) {
                return Some(id);
            }
        }

        if self.widget.contains_local(local_point) && self.widget.is_interactive() {
            Some(self.widget.id)
        } else {
            None
        }
    }

    pub fn find_widget_mut(&mut self, widget_id: WidgetId) -> Option<&mut WidgetNode> {
        if self.widget.id == widget_id {
            return Some(&mut self.widget);
        }
        for child in &mut self.children {
            if let Some(widget) = child.find_widget_mut(widget_id) {
                return Some(widget);
            }
        }
        None
    }

    pub fn find_widget(&self, widget_id: WidgetId) -> Option<&WidgetNode> {
        if self.widget.id == widget_id {
            return Some(&self.widget);
        }
        for child in &self.children {
            if let Some(widget) = child.find_widget(widget_id) {
                return Some(widget);
            }
        }
        None
    }

    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.widget.apply_theme(palette);
        for child in &mut self.children {
            child.apply_theme(palette);
        }
    }
}

impl WidgetNode {
    pub fn new(kind: WidgetKind, rect: Rect, accent_color: Color) -> Self {
        Self {
            id: 0,
            kind,
            rect,
            accent_color,
            background_color: Color::rgb(248, 250, 252),
            visible: true,
            enabled: true,
            interaction: WidgetInteractionState::default(),
            action: None,
            style: WidgetVisualStyle::from_palette(ThemePalette::aurora(accent_color)),
        }
    }

    pub fn with_id(mut self, id: WidgetId) -> Self {
        self.id = id;
        self
    }

    pub fn with_action(mut self, action: WidgetAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_style(mut self, style: WidgetVisualStyle) -> Self {
        self.style = style;
        self
    }

    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.style = WidgetVisualStyle::from_palette(palette.with_accent(self.accent_color));
        self.background_color = self.style.bg;
    }

    fn fill_color(&self) -> Color {
        if !self.enabled {
            return self.style.bg_disabled;
        }
        if self.interaction.pressed {
            return self.style.bg_pressed;
        }
        if self.interaction.hovered {
            return self.style.bg_hover;
        }
        self.style.bg
    }

    fn accent_fill_color(&self) -> Color {
        if !self.enabled {
            return self.style.bg_disabled;
        }
        if self.interaction.pressed {
            return ThemePalette::aurora(self.accent_color).accent_pressed;
        }
        if self.interaction.hovered {
            return ThemePalette::aurora(self.accent_color).accent_hover;
        }
        self.accent_color
    }

    pub fn render(&self, origin: Vec2) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        if !self.visible {
            return commands;
        }

        let rect = Rect::new(
            origin.x + self.rect.x,
            origin.y + self.rect.y,
            self.rect.width,
            self.rect.height,
        );

        match &self.kind {
            WidgetKind::Panel => {
                commands.push(RenderCommand::draw_rect_rounded(rect, self.fill_color(), self.style.radius + 2.0));
            }
            WidgetKind::Label { text } => {
                commands.push(RenderCommand::draw_text(
                    Vec2::new(rect.x, rect.y + 12.0),
                    text,
                    self.style.text,
                    13.0,
                ));
            }
            WidgetKind::Button { text, pressed } => {
                commands.push(RenderCommand::draw_rect_rounded(
                    rect,
                    if *pressed { ThemePalette::aurora(self.accent_color).accent_pressed } else { self.accent_fill_color() },
                    self.style.radius,
                ));
                commands.push(RenderCommand::draw_text(
                    Vec2::new(rect.x + 12.0, rect.y + rect.height * 0.58),
                    text,
                    self.style.text_on_accent,
                    13.0,
                ));
            }
            WidgetKind::Image { title } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, self.accent_color, 12.0));
                commands.push(RenderCommand::draw_line_thick(
                    Vec2::new(rect.x + 10.0, rect.bottom() - 18.0),
                    Vec2::new(rect.x + rect.width * 0.42, rect.y + rect.height * 0.45),
                    Color::WHITE,
                    3.0,
                ));
                commands.push(RenderCommand::draw_line_thick(
                    Vec2::new(rect.x + rect.width * 0.42, rect.y + rect.height * 0.45),
                    Vec2::new(rect.right() - 12.0, rect.bottom() - 26.0),
                    Color::WHITE,
                    3.0,
                ));
                commands.push(RenderCommand::draw_text(
                    Vec2::new(rect.x + 12.0, rect.y + 18.0),
                    title,
                    Color::WHITE,
                    12.0,
                ));
            }
            WidgetKind::CheckBox { text, checked } => {
                let box_rect = Rect::new(rect.x, rect.y, 18.0, 18.0);
                commands.push(RenderCommand::draw_rect_rounded(box_rect, Color::rgb(230, 235, 243), 4.0));
                if *checked {
                    commands.push(RenderCommand::draw_rect_rounded(
                        box_rect.inset(4.0, 4.0),
                        self.accent_color,
                        2.0,
                    ));
                }
                commands.push(RenderCommand::draw_text(
                    Vec2::new(rect.x + 26.0, rect.y + 13.0),
                    text,
                    self.style.text,
                    12.0,
                ));
            }
            WidgetKind::Switch { on } => {
                commands.push(RenderCommand::draw_rect_rounded(
                    rect,
                    if *on { self.accent_fill_color() } else { self.style.border },
                    rect.height * 0.5,
                ));
                let knob_x = if *on { rect.right() - rect.height + 2.0 } else { rect.x + 2.0 };
                commands.push(RenderCommand::draw_rect_rounded(
                    Rect::new(knob_x, rect.y + 2.0, rect.height - 4.0, rect.height - 4.0),
                    Color::WHITE,
                    rect.height * 0.5,
                ));
            }
            WidgetKind::Slider { value } | WidgetKind::Bar { value } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, self.style.border, 8.0));
                let width = rect.width * (*value as f32 / 100.0);
                commands.push(RenderCommand::draw_rect_rounded(
                    Rect::new(rect.x, rect.y, width, rect.height),
                    self.accent_fill_color(),
                    8.0,
                ));
            }
            WidgetKind::Arc { value } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(237, 241, 247), 999.0));
                let progress = rect.width * (*value as f32 / 100.0);
                commands.push(RenderCommand::draw_rect_rounded(
                    Rect::new(rect.x, rect.y, progress, rect.height),
                    self.accent_color,
                    999.0,
                ));
                commands.push(RenderCommand::draw_text(
                    Vec2::new(rect.center().x - 12.0, rect.center().y + 4.0),
                    "%",
                    Color::WHITE,
                    12.0,
                ));
            }
            WidgetKind::Spinner => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(236, 240, 246), 999.0));
                commands.push(RenderCommand::draw_rect_rounded(
                    Rect::new(rect.x + rect.width * 0.38, rect.y + 4.0, rect.width * 0.24, rect.height - 8.0),
                    self.accent_color,
                    999.0,
                ));
            }
            WidgetKind::TextArea { text, placeholder, cursor, scroll_row } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, self.fill_color(), self.style.radius));
                if self.interaction.focused {
                    commands.push(RenderCommand::draw_rect_rounded(
                        Rect::new(rect.x - 1.0, rect.y - 1.0, rect.width + 2.0, rect.height + 2.0),
                        self.accent_fill_color(),
                        self.style.radius + 1.0,
                    ));
                    commands.push(RenderCommand::draw_rect_rounded(rect, self.fill_color(), self.style.radius));
                }

                let display_text = if text.is_empty() {
                    (*placeholder).into()
                } else {
                    visible_text_rows(text.as_str(), rect.width - 20.0, rect.height - 16.0, 12.0, *scroll_row)
                };
                commands.push(RenderCommand::draw_text(
                    Vec2::new(rect.x + 10.0, rect.y + 16.0),
                    display_text.as_str(),
                    if text.is_empty() { self.style.text_muted } else { self.style.text },
                    12.0,
                ));

                if self.interaction.focused {
                    let caret = caret_position(text.as_str(), *cursor, rect.width - 20.0, 12.0);
                    let caret_x = rect.x + 10.0 + caret.x;
                    let line_height = text_line_height(12.0);
                    let caret_y = rect.y + 8.0 + caret.y - *scroll_row as f32 * line_height;
                    if caret_y >= rect.y + 4.0 && caret_y + line_height <= rect.bottom() - 4.0 {
                        commands.push(RenderCommand::draw_rect(
                            Rect::new(caret_x, caret_y, 1.0, 12.0),
                            self.style.text,
                        ));
                    }
                }
            }
            WidgetKind::List { title, items, selected } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, self.fill_color(), self.style.radius));
                commands.push(RenderCommand::draw_text(Vec2::new(rect.x + 10.0, rect.y + 14.0), title, self.style.text, 12.0));
                for (index, item) in items.iter().enumerate() {
                    let row = Rect::new(rect.x + 8.0, rect.y + 22.0 + index as f32 * 22.0, rect.width - 16.0, 18.0);
                    if *selected == index {
                        commands.push(RenderCommand::draw_rect_rounded(row, self.accent_fill_color(), 8.0));
                    }
                    commands.push(RenderCommand::draw_text(
                        Vec2::new(row.x + 8.0, row.y + 12.0),
                        item,
                        if *selected == index { self.style.text_on_accent } else { self.style.text },
                        11.0,
                    ));
                }
            }
            WidgetKind::Menu { title, items } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(245, 248, 252), 12.0));
                commands.push(RenderCommand::draw_text(Vec2::new(rect.x + 10.0, rect.y + 14.0), title, Color::rgb(52, 62, 82), 12.0));
                for (index, item) in items.iter().enumerate() {
                    commands.push(RenderCommand::draw_text(
                        Vec2::new(rect.x + 14.0, rect.y + 34.0 + index as f32 * 18.0),
                        item,
                        Color::rgb(78, 88, 108),
                        11.0,
                    ));
                }
            }
            WidgetKind::TabView { tabs, active } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, self.fill_color(), self.style.radius + 2.0));
                let tab_width = rect.width / tabs.len().max(1) as f32;
                for (index, tab) in tabs.iter().enumerate() {
                    let tab_rect = Rect::new(rect.x + index as f32 * tab_width, rect.y, tab_width, 24.0);
                    commands.push(RenderCommand::draw_rect_rounded(
                        tab_rect,
                        if *active == index { self.accent_fill_color() } else { self.style.border },
                        8.0,
                    ));
                    commands.push(RenderCommand::draw_text(
                        Vec2::new(tab_rect.x + 10.0, tab_rect.y + 15.0),
                        tab,
                        if *active == index { self.style.text_on_accent } else { self.style.text },
                        11.0,
                    ));
                }
            }
            WidgetKind::Table { headers, rows } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(248, 250, 253), 10.0));
                let cols = headers.len().max(1);
                let col_width = rect.width / cols as f32;
                for (index, header) in headers.iter().enumerate() {
                    commands.push(RenderCommand::draw_text(
                        Vec2::new(rect.x + 8.0 + index as f32 * col_width, rect.y + 14.0),
                        header,
                        Color::rgb(58, 68, 88),
                        11.0,
                    ));
                }
                for (row_index, row) in rows.iter().enumerate() {
                    for (col_index, cell) in row.iter().enumerate() {
                        commands.push(RenderCommand::draw_text(
                            Vec2::new(rect.x + 8.0 + col_index as f32 * col_width, rect.y + 32.0 + row_index as f32 * 18.0),
                            cell,
                            Color::rgb(90, 100, 118),
                            10.0,
                        ));
                    }
                }
            }
            WidgetKind::Chart { values } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(246, 249, 253), 10.0));
                if values.len() > 1 {
                    let step = rect.width / (values.len() - 1) as f32;
                    for index in 0..values.len() - 1 {
                        let y1 = rect.bottom() - 8.0 - (rect.height - 16.0) * (values[index] as f32 / 100.0);
                        let y2 = rect.bottom() - 8.0 - (rect.height - 16.0) * (values[index + 1] as f32 / 100.0);
                        commands.push(RenderCommand::draw_line_thick(
                            Vec2::new(rect.x + 8.0 + index as f32 * step, y1),
                            Vec2::new(rect.x + 8.0 + (index + 1) as f32 * step, y2),
                            self.accent_color,
                            2.0,
                        ));
                    }
                }
            }
            WidgetKind::Canvas { title } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(239, 244, 250), 10.0));
                commands.push(RenderCommand::draw_text(Vec2::new(rect.x + 10.0, rect.y + 14.0), title, Color::rgb(54, 64, 84), 11.0));
                commands.push(RenderCommand::draw_line_thick(
                    Vec2::new(rect.x + 14.0, rect.bottom() - 18.0),
                    Vec2::new(rect.center().x, rect.y + 24.0),
                    self.accent_color,
                    3.0,
                ));
                commands.push(RenderCommand::draw_line_thick(
                    Vec2::new(rect.center().x, rect.y + 24.0),
                    Vec2::new(rect.right() - 12.0, rect.bottom() - 22.0),
                    Color::rgb(255, 155, 88),
                    3.0,
                ));
            }
            WidgetKind::Keyboard { rows } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(245, 248, 252), 12.0));
                for (index, row) in rows.iter().enumerate() {
                    let row_rect = Rect::new(rect.x + 8.0, rect.y + 8.0 + index as f32 * 20.0, rect.width - 16.0, 16.0);
                    commands.push(RenderCommand::draw_rect_rounded(row_rect, Color::rgb(227, 233, 242), 6.0));
                    commands.push(RenderCommand::draw_text(Vec2::new(row_rect.x + 8.0, row_rect.y + 11.0), row, Color::rgb(82, 92, 112), 10.0));
                }
            }
            WidgetKind::Dropdown { options, selected, expanded } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, self.fill_color(), 8.0));
                let current = options.get(*selected).copied().unwrap_or("");
                commands.push(RenderCommand::draw_text(Vec2::new(rect.x + 10.0, rect.y + 13.0), current, self.style.text, 11.0));
                if *expanded {
                    let popup_height = options.len() as f32 * 18.0 + 8.0;
                    let popup = Rect::new(rect.x, rect.bottom() + 4.0, rect.width, popup_height);
                    commands.push(RenderCommand::draw_rect_rounded(popup, self.style.bg_hover, 8.0));
                    for (index, option) in options.iter().enumerate() {
                        let item_rect = Rect::new(popup.x + 4.0, popup.y + 4.0 + index as f32 * 18.0, popup.width - 8.0, 16.0);
                        if *selected == index {
                            commands.push(RenderCommand::draw_rect_rounded(item_rect, self.accent_fill_color(), 6.0));
                        }
                        commands.push(RenderCommand::draw_text(
                            Vec2::new(item_rect.x + 6.0, item_rect.y + 11.0),
                            *option,
                            if *selected == index { self.style.text_on_accent } else { self.style.text },
                            10.0,
                        ));
                    }
                }
            }
            WidgetKind::Roller { options, selected } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(242, 246, 251), 10.0));
                for (index, option) in options.iter().enumerate() {
                    commands.push(RenderCommand::draw_text(
                        Vec2::new(rect.x + 12.0, rect.y + 14.0 + index as f32 * 14.0),
                        option,
                        if *selected == index { self.accent_color } else { Color::rgb(104, 114, 132) },
                        10.0,
                    ));
                }
            }
            WidgetKind::SpinBox { value } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(250, 252, 255), 8.0));
                commands.push(RenderCommand::draw_text(
                    Vec2::new(rect.x + 10.0, rect.y + 13.0),
                    value.to_string(),
                    self.accent_color,
                    12.0,
                ));
            }
            WidgetKind::Calendar { month } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(245, 248, 253), 10.0));
                commands.push(RenderCommand::draw_rect_rounded(Rect::new(rect.x, rect.y, rect.width, 22.0), self.accent_color, 10.0));
                commands.push(RenderCommand::draw_text(Vec2::new(rect.x + 10.0, rect.y + 15.0), month, Color::WHITE, 11.0));
            }
            WidgetKind::MsgBox { title, body } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(251, 252, 255), 12.0));
                commands.push(RenderCommand::draw_text(Vec2::new(rect.x + 10.0, rect.y + 14.0), title, Color::rgb(52, 62, 82), 12.0));
                commands.push(RenderCommand::draw_text(Vec2::new(rect.x + 10.0, rect.y + 30.0), body, Color::rgb(98, 108, 126), 10.0));
            }
            WidgetKind::TileView { pages, active } => {
                commands.push(RenderCommand::draw_rect_rounded(rect, Color::rgb(245, 248, 252), 12.0));
                for (index, page) in pages.iter().enumerate() {
                    let tile = Rect::new(rect.x + 8.0 + index as f32 * 58.0, rect.y + 8.0, 50.0, rect.height - 16.0);
                    commands.push(RenderCommand::draw_rect_rounded(
                        tile,
                        if *active == index { self.accent_color } else { Color::rgb(220, 227, 236) },
                        10.0,
                    ));
                    commands.push(RenderCommand::draw_text(Vec2::new(tile.x + 8.0, tile.y + 16.0), page, if *active == index { Color::WHITE } else { Color::rgb(82, 92, 112) }, 10.0));
                }
            }
            WidgetKind::ButtonMatrix { labels, columns, selected } => {
                let cols = (*columns).max(1);
                let cell_w = (rect.width - 8.0) / cols as f32;
                for (index, label) in labels.iter().enumerate() {
                    let row = index / cols;
                    let col = index % cols;
                    let cell = Rect::new(rect.x + col as f32 * cell_w, rect.y + row as f32 * 26.0, cell_w - 4.0, 22.0);
                    commands.push(RenderCommand::draw_rect_rounded(
                        cell,
                        if *selected == Some(index) { self.accent_fill_color() } else { Color::rgb(231, 236, 244) },
                        6.0,
                    ));
                    commands.push(RenderCommand::draw_text(
                        Vec2::new(cell.x + 10.0, cell.y + 14.0),
                        label,
                        if *selected == Some(index) { self.style.text_on_accent } else { Color::rgb(60, 70, 90) },
                        10.0,
                    ));
                }
            }
            WidgetKind::Led { on } => {
                commands.push(RenderCommand::draw_rect_rounded(
                    rect,
                    if *on { Color::rgb(92, 214, 127) } else { Color::rgb(184, 192, 206) },
                    999.0,
                ));
            }
            WidgetKind::Line => {
                commands.push(RenderCommand::draw_line_thick(
                    Vec2::new(rect.x, rect.y),
                    Vec2::new(rect.right(), rect.bottom()),
                    self.accent_color,
                    2.0,
                ));
            }
        }

        if !self.enabled {
            commands.push(RenderCommand::draw_rect_rounded(rect, ThemePalette::aurora(self.accent_color).overlay, 10.0));
        }

        commands
    }

    pub fn contains_local(&self, point: Vec2) -> bool {
        if !self.visible {
            return false;
        }

        if self.rect.contains(point) {
            return true;
        }

        match &self.kind {
            WidgetKind::Dropdown { options, expanded, .. } if *expanded => {
                let popup = Rect::new(self.rect.x, self.rect.bottom() + 4.0, self.rect.width, options.len() as f32 * 18.0 + 8.0);
                popup.contains(point)
            }
            _ => false,
        }
    }

    pub fn is_interactive(&self) -> bool {
        matches!(
            self.kind,
            WidgetKind::Button { .. }
                | WidgetKind::CheckBox { .. }
                | WidgetKind::Switch { .. }
                | WidgetKind::Slider { .. }
                | WidgetKind::Bar { .. }
                | WidgetKind::Arc { .. }
                | WidgetKind::Dropdown { .. }
                | WidgetKind::TabView { .. }
                | WidgetKind::List { .. }
                | WidgetKind::Roller { .. }
                | WidgetKind::TileView { .. }
                | WidgetKind::SpinBox { .. }
                | WidgetKind::Led { .. }
                | WidgetKind::TextArea { .. }
                | WidgetKind::ButtonMatrix { .. }
        )
    }

    pub fn set_hovered(&mut self, hovered: bool) -> Option<WidgetEvent> {
        if self.interaction.hovered == hovered {
            return None;
        }
        self.interaction.hovered = hovered;
        Some(WidgetEvent {
            widget_id: self.id,
            kind: if hovered {
                WidgetEventKind::HoverEnter
            } else {
                WidgetEventKind::HoverLeave
            },
        })
    }

    pub fn press(&mut self, point: Vec2) -> WidgetResponse {
        if !self.enabled || !self.contains_local(point) || !self.is_interactive() {
            return WidgetResponse::default();
        }
        self.interaction.pressed = true;
        self.interaction.focused = true;
        WidgetResponse {
            consumed: true,
            event: Some(WidgetEvent {
                widget_id: self.id,
                kind: WidgetEventKind::Press,
            }),
            action: None,
        }
    }

    pub fn release(&mut self, point: Vec2) -> WidgetResponse {
        let was_pressed = self.interaction.pressed;
        self.interaction.pressed = false;
        if !was_pressed {
            return WidgetResponse::default();
        }

        if self.enabled && self.contains_local(point) {
            return self.activate(point, WidgetEventKind::Click);
        }

        WidgetResponse {
            consumed: true,
            event: Some(WidgetEvent {
                widget_id: self.id,
                kind: WidgetEventKind::Release,
            }),
            action: None,
        }
    }

    pub fn drag(&mut self, point: Vec2) -> WidgetResponse {
        if !self.interaction.pressed || !self.enabled {
            return WidgetResponse::default();
        }

        match &mut self.kind {
            WidgetKind::Slider { value } | WidgetKind::Bar { value } | WidgetKind::Arc { value } => {
                let new_value = percent_from_x(self.rect, point.x);
                if *value != new_value {
                    *value = new_value;
                    return WidgetResponse {
                        consumed: true,
                        event: Some(WidgetEvent {
                            widget_id: self.id,
                            kind: WidgetEventKind::ValueChanged,
                        }),
                        action: None,
                    };
                }
                WidgetResponse { consumed: true, event: None, action: None }
            }
            _ => WidgetResponse::default(),
        }
    }

    fn activate(&mut self, point: Vec2, terminal_event: WidgetEventKind) -> WidgetResponse {
        if !self.enabled || !self.contains_local(point) {
            return WidgetResponse::default();
        }

        match &mut self.kind {
            WidgetKind::Button { pressed, .. } => {
                *pressed = !*pressed;
                WidgetResponse {
                    consumed: true,
                    event: Some(WidgetEvent {
                        widget_id: self.id,
                        kind: terminal_event,
                    }),
                    action: self.action,
                }
            }
            WidgetKind::TextArea { cursor, text, scroll_row, .. } => {
                self.interaction.focused = true;
                let local_x = (point.x - self.rect.x - 10.0).max(0.0);
                let local_y = (point.y - self.rect.y - 8.0).max(0.0) + *scroll_row as f32 * text_line_height(12.0);
                *cursor = cursor_from_point(text.as_str(), local_x, local_y, self.rect.width - 20.0, 12.0);
                sync_textarea_scroll(text.as_str(), cursor, scroll_row, self.rect.width - 20.0, self.rect.height - 16.0, 12.0);
                WidgetResponse {
                    consumed: true,
                    event: Some(WidgetEvent {
                        widget_id: self.id,
                        kind: WidgetEventKind::Press,
                    }),
                    action: None,
                }
            }
            WidgetKind::CheckBox { checked, .. } => {
                *checked = !*checked;
                WidgetResponse {
                    consumed: true,
                    event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::ToggleChanged }),
                    action: self.action,
                }
            }
            WidgetKind::Switch { on } => {
                *on = !*on;
                WidgetResponse {
                    consumed: true,
                    event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::ToggleChanged }),
                    action: self.action,
                }
            }
            WidgetKind::Slider { value } | WidgetKind::Bar { value } | WidgetKind::Arc { value } => {
                *value = percent_from_x(self.rect, point.x);
                WidgetResponse {
                    consumed: true,
                    event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::ValueChanged }),
                    action: self.action,
                }
            }
            WidgetKind::Dropdown { options, selected, expanded } => {
                let popup_top = self.rect.bottom() + 4.0;
                if *expanded && point.y >= popup_top {
                    let item_index = ((point.y - popup_top - 4.0) / 18.0) as usize;
                    if item_index < options.len() {
                        *selected = item_index;
                        *expanded = false;
                        WidgetResponse {
                            consumed: true,
                            event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::SelectionChanged }),
                            action: self.action,
                        }
                    } else {
                        *expanded = false;
                        WidgetResponse::default()
                    }
                } else {
                    *expanded = !*expanded;
                    WidgetResponse {
                        consumed: true,
                        event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::ToggleChanged }),
                        action: self.action,
                    }
                }
            }
            WidgetKind::TabView { tabs, active } => {
                let width = self.rect.width / tabs.len().max(1) as f32;
                *active = ((point.x - self.rect.x) / width) as usize;
                if *active >= tabs.len() {
                    *active = tabs.len().saturating_sub(1);
                }
                WidgetResponse {
                    consumed: true,
                    event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::SelectionChanged }),
                    action: self.action,
                }
            }
            WidgetKind::List { items, selected, .. } => {
                let first_row_y = self.rect.y + 22.0;
                if point.y >= first_row_y {
                    let row = ((point.y - first_row_y) / 22.0) as usize;
                    if row < items.len() {
                        *selected = row;
                        return WidgetResponse {
                            consumed: true,
                            event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::SelectionChanged }),
                            action: self.action,
                        };
                    }
                }
                WidgetResponse::default()
            }
            WidgetKind::Roller { options, selected } => {
                let row = ((point.y - self.rect.y) / 14.0) as usize;
                if row < options.len() {
                    *selected = row;
                    WidgetResponse {
                        consumed: true,
                        event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::SelectionChanged }),
                        action: self.action,
                    }
                } else {
                    WidgetResponse::default()
                }
            }
            WidgetKind::TileView { pages, active } => {
                let tile_width = 58.0;
                let idx = ((point.x - self.rect.x - 8.0) / tile_width) as usize;
                if idx < pages.len() {
                    *active = idx;
                    WidgetResponse {
                        consumed: true,
                        event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::SelectionChanged }),
                        action: self.action,
                    }
                } else {
                    WidgetResponse::default()
                }
            }
            WidgetKind::SpinBox { value } => {
                if point.x < self.rect.center().x {
                    *value -= 1;
                } else {
                    *value += 1;
                }
                WidgetResponse {
                    consumed: true,
                    event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::ValueChanged }),
                    action: self.action,
                }
            }
            WidgetKind::Led { on } => {
                *on = !*on;
                WidgetResponse {
                    consumed: true,
                    event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::ToggleChanged }),
                    action: self.action,
                }
            }
            WidgetKind::ButtonMatrix { labels, columns, selected } => {
                let cols = (*columns).max(1);
                let cell_w = (self.rect.width - 8.0) / cols as f32;
                let col = (point.x / cell_w) as usize;
                let row = (point.y / 26.0) as usize;
                let idx = row * cols + col;
                if idx < labels.len() {
                    *selected = Some(idx);
                    WidgetResponse {
                        consumed: true,
                        event: Some(WidgetEvent { widget_id: self.id, kind: WidgetEventKind::SelectionChanged }),
                        action: self.action,
                    }
                } else {
                    WidgetResponse::default()
                }
            }
            _ => WidgetResponse::default(),
        }
    }

    pub fn handle_text_edit(&mut self, command: TextEditCommand) -> bool {
        match &mut self.kind {
            WidgetKind::TextArea { text, cursor, scroll_row, .. } if self.interaction.focused => {
                match command {
                    TextEditCommand::Insert(ch) => {
                        if *cursor <= text.len() {
                            text.insert(*cursor, ch);
                            *cursor += 1;
                        }
                    }
                    TextEditCommand::Backspace => {
                        if *cursor > 0 && *cursor <= text.len() {
                            let idx = *cursor - 1;
                            text.remove(idx);
                            *cursor = idx;
                        }
                    }
                    TextEditCommand::Delete => {
                        if *cursor < text.len() {
                            text.remove(*cursor);
                        }
                    }
                    TextEditCommand::Newline => {
                        if *cursor <= text.len() {
                            text.insert(*cursor, '\n');
                            *cursor += 1;
                        }
                    }
                    TextEditCommand::MoveLeft => {
                        if *cursor > 0 {
                            *cursor -= 1;
                        }
                    }
                    TextEditCommand::MoveRight => {
                        if *cursor < text.len() {
                            *cursor += 1;
                        }
                    }
                    TextEditCommand::MoveUp => {
                        let (row, col) = cursor_row_col(text.as_str(), *cursor, self.rect.width - 20.0, 12.0);
                        if row > 0 {
                            *cursor = cursor_from_row_col(text.as_str(), row - 1, col, self.rect.width - 20.0, 12.0);
                        }
                    }
                    TextEditCommand::MoveDown => {
                        let (row, col) = cursor_row_col(text.as_str(), *cursor, self.rect.width - 20.0, 12.0);
                        *cursor = cursor_from_row_col(text.as_str(), row + 1, col, self.rect.width - 20.0, 12.0);
                    }
                    TextEditCommand::MoveLineStart => {
                        let (row, _) = cursor_row_col(text.as_str(), *cursor, self.rect.width - 20.0, 12.0);
                        *cursor = cursor_from_row_col(text.as_str(), row, 0, self.rect.width - 20.0, 12.0);
                    }
                    TextEditCommand::MoveLineEnd => {
                        let (row, _) = cursor_row_col(text.as_str(), *cursor, self.rect.width - 20.0, 12.0);
                        *cursor = cursor_line_end(text.as_str(), row, self.rect.width - 20.0, 12.0);
                    }
                }
                sync_textarea_scroll(text.as_str(), cursor, scroll_row, self.rect.width - 20.0, self.rect.height - 16.0, 12.0);
                true
            }
            _ => false,
        }
    }
}

fn percent_from_x(rect: Rect, x: f32) -> u8 {
    let width = rect.width.max(1.0);
    let relative = ((x - rect.x) / width).clamp(0.0, 1.0);
    (relative * 100.0) as u8
}

fn glyph_advance(size: f32) -> f32 {
    let scale = ((size / 8.0).max(1.0)) as u32;
    let scale = scale.max(1) as f32;
    6.0 * scale
}

fn text_line_height(size: f32) -> f32 {
    8.0 * (((size / 8.0).max(1.0)) as u32).max(1) as f32
}

fn wrap_text_to_width(text: &str, width: f32, size: f32) -> String {
    let advance = glyph_advance(size).max(1.0);
    let cols = ((width / advance) as usize).max(1);
    let mut out = String::new();
    let mut col = 0usize;

    for ch in text.chars() {
        if ch == '\n' {
            out.push(ch);
            col = 0;
            continue;
        }
        if col >= cols {
            out.push('\n');
            col = 0;
        }
        out.push(ch);
        col += 1;
    }

    out
}

fn visible_text_rows(text: &str, width: f32, height: f32, size: f32, scroll_row: usize) -> String {
    let wrapped = wrap_text_to_width(text, width, size);
    let max_rows = ((height / text_line_height(size)) as usize).max(1);
    let mut out = String::new();

    for (row, line) in wrapped.split('\n').enumerate() {
        if row < scroll_row {
            continue;
        }
        if row >= scroll_row + max_rows {
            break;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line);
    }

    out
}

fn caret_position(text: &str, cursor: usize, width: f32, size: f32) -> Vec2 {
    let advance = glyph_advance(size).max(1.0);
    let cols = ((width / advance) as usize).max(1);
    let line_height = text_line_height(size);
    let mut x = 0usize;
    let mut y = 0usize;

    for (idx, ch) in text.chars().enumerate() {
        if idx >= cursor {
            break;
        }
        if ch == '\n' || x >= cols {
            y += 1;
            x = 0;
            if ch == '\n' {
                continue;
            }
        }
        x += 1;
    }

    Vec2::new(x as f32 * advance, y as f32 * line_height)
}

fn sync_textarea_scroll(text: &str, cursor: &usize, scroll_row: &mut usize, width: f32, height: f32, size: f32) {
    let (row, _) = cursor_row_col(text, *cursor, width, size);
    let visible_rows = ((height / text_line_height(size)) as usize).max(1);

    if row < *scroll_row {
        *scroll_row = row;
    } else if row >= *scroll_row + visible_rows {
        *scroll_row = row + 1 - visible_rows;
    }
}

fn cursor_row_col(text: &str, cursor: usize, width: f32, size: f32) -> (usize, usize) {
    let advance = glyph_advance(size).max(1.0);
    let cols = ((width / advance) as usize).max(1);
    let mut row = 0usize;
    let mut col = 0usize;

    for (idx, ch) in text.chars().enumerate() {
        if idx >= cursor {
            break;
        }
        if ch == '\n' {
            row += 1;
            col = 0;
            continue;
        }
        if col >= cols {
            row += 1;
            col = 0;
        }
        col += 1;
    }

    (row, col)
}

fn cursor_from_row_col(text: &str, target_row: usize, target_col: usize, width: f32, size: f32) -> usize {
    let advance = glyph_advance(size).max(1.0);
    let cols = ((width / advance) as usize).max(1);
    let mut row = 0usize;
    let mut col = 0usize;
    let mut cursor = 0usize;

    for (idx, ch) in text.chars().enumerate() {
        if row == target_row && col >= target_col {
            return idx;
        }
        if ch == '\n' {
            if row == target_row {
                return idx;
            }
            row += 1;
            col = 0;
            cursor = idx + 1;
            continue;
        }
        if col >= cols {
            if row == target_row {
                return idx;
            }
            row += 1;
            col = 0;
        }
        col += 1;
        cursor = idx + 1;
    }

    cursor
}

fn cursor_line_end(text: &str, target_row: usize, width: f32, size: f32) -> usize {
    let advance = glyph_advance(size).max(1.0);
    let cols = ((width / advance) as usize).max(1);
    let mut row = 0usize;
    let mut col = 0usize;
    let mut last_in_row = 0usize;

    for (idx, ch) in text.chars().enumerate() {
        if row > target_row {
            break;
        }
        if ch == '\n' {
            if row == target_row {
                return idx;
            }
            row += 1;
            col = 0;
            last_in_row = idx + 1;
            continue;
        }
        if col >= cols {
            if row == target_row {
                return idx;
            }
            row += 1;
            col = 0;
        }
        col += 1;
        if row == target_row {
            last_in_row = idx + 1;
        }
    }

    last_in_row
}

fn cursor_from_point(text: &str, x: f32, y: f32, width: f32, size: f32) -> usize {
    let advance = glyph_advance(size).max(1.0);
    let cols = ((width / advance) as usize).max(1);
    let line_height = 8.0 * (((size / 8.0).max(1.0)) as u32).max(1) as f32;
    let target_row = (y / line_height) as usize;
    let target_col = (x / advance) as usize;

    let mut row = 0usize;
    let mut col = 0usize;
    let mut cursor = 0usize;

    for (idx, ch) in text.chars().enumerate() {
        if row == target_row && col >= target_col {
            return idx;
        }
        if ch == '\n' {
            if row == target_row {
                return idx;
            }
            row += 1;
            col = 0;
            cursor = idx + 1;
            continue;
        }
        if col >= cols {
            row += 1;
            col = 0;
            if row > target_row {
                return idx;
            }
        }
        col += 1;
        cursor = idx + 1;
    }

    cursor
}

const FILE_ROWS: &[&[&str]] = &[
    &["src", "dir", "today"],
    &["assets", "dir", "today"],
    &["readme", "txt", "1 KB"],
];

fn make_row_panel(app: &AppInfo, rect: Rect, children: Vec<WidgetTreeNode>) -> WidgetTreeNode {
    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Panel, rect, app.icon_color))
        .with_layout(WidgetLayout::Horizontal { spacing: 12.0, padding: 12.0 })
        .with_children(children)
}

fn make_column_panel(app: &AppInfo, rect: Rect, children: Vec<WidgetTreeNode>) -> WidgetTreeNode {
    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Panel, rect, app.icon_color))
        .with_layout(WidgetLayout::Vertical { spacing: 10.0, padding: 12.0 })
        .with_children(children)
}

pub fn default_widget_tree_for_app(app: &AppInfo) -> Vec<WidgetTreeNode> {
    match app.name {
        "Calculator" => vec![
            make_column_panel(
                app,
                Rect::new(16.0, 96.0, 210.0, 170.0),
                vec![
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Label { text: "SpinBox" }, Rect::new(0.0, 0.0, 80.0, 18.0), app.icon_color)),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::SpinBox { value: 42 }, Rect::new(0.0, 0.0, 100.0, 24.0), app.icon_color)),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::ButtonMatrix { labels: &["7", "8", "9", "/", "4", "5", "6", "*", "1", "2", "3", "-", "C", "0", "=", "+"], columns: 4, selected: None }, Rect::new(0.0, 0.0, 220.0, 110.0), app.icon_color)
                        .with_action(WidgetAction::ApplyButtonMatrixSelection)),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Bar { value: 68 }, Rect::new(0.0, 0.0, 180.0, 14.0), app.icon_color)),
                ],
            ),
        ],
        "Files" => vec![
            make_row_panel(
                app,
                Rect::new(16.0, 96.0, 388.0, 148.0),
                vec![
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Menu { title: "Menu", items: &["Home", "Projects", "Media", "Trash"] }, Rect::new(0.0, 0.0, 110.0, 108.0), app.icon_color)),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Table { headers: &["Name", "Type", "Size"], rows: FILE_ROWS }, Rect::new(0.0, 0.0, 190.0, 110.0), app.icon_color)),
                ],
            ),
            make_row_panel(
                app,
                Rect::new(16.0, 252.0, 388.0, 108.0),
                vec![
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::List { title: "Recent", items: &["desktop", "notes", "capture"], selected: 1 }, Rect::new(0.0, 0.0, 180.0, 96.0), app.icon_color)
                        .with_action(WidgetAction::LaunchAppNamed("Terminal"))),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::TileView { pages: &["A", "B", "C"], active: 1 }, Rect::new(0.0, 0.0, 160.0, 72.0), app.icon_color)),
                ],
            ),
        ],
        "Terminal" => vec![
            make_column_panel(
                app,
                Rect::new(16.0, 96.0, 360.0, 170.0),
                vec![
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::TextArea { text: String::from("$ wing --status"), placeholder: "command", cursor: 14, scroll_row: 0 }, Rect::new(0.0, 0.0, 320.0, 88.0), app.icon_color)),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Keyboard { rows: &["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"] }, Rect::new(0.0, 0.0, 320.0, 74.0), app.icon_color)),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Line, Rect::new(0.0, 0.0, 320.0, 24.0), app.icon_color)),
                ],
            ),
            WidgetTreeNode::new(WidgetNode::new(WidgetKind::Led { on: true }, Rect::new(386.0, 106.0, 16.0, 16.0), app.icon_color)
                .with_action(WidgetAction::CloseWindow)),
        ],
        "Settings" => vec![
            make_row_panel(
                app,
                Rect::new(16.0, 96.0, 360.0, 120.0),
                vec![
                    make_column_panel(
                        app,
                        Rect::new(0.0, 0.0, 180.0, 96.0),
                        vec![
                            WidgetTreeNode::new(WidgetNode::new(WidgetKind::CheckBox { text: "Enable blur", checked: true }, Rect::new(0.0, 0.0, 140.0, 18.0), app.icon_color)),
                            WidgetTreeNode::new(WidgetNode::new(WidgetKind::Switch { on: true }, Rect::new(0.0, 0.0, 42.0, 20.0), app.icon_color)),
                            WidgetTreeNode::new(WidgetNode::new(WidgetKind::Slider { value: 72 }, Rect::new(0.0, 0.0, 156.0, 14.0), app.icon_color)),
                            WidgetTreeNode::new(WidgetNode::new(WidgetKind::Dropdown { options: &["Aurora", "Dusk", "System"], selected: 0, expanded: false }, Rect::new(0.0, 0.0, 160.0, 24.0), app.icon_color)
                                .with_action(WidgetAction::ApplySelectedTheme)),
                        ],
                    ),
                    make_column_panel(
                        app,
                        Rect::new(0.0, 0.0, 120.0, 96.0),
                        vec![
                            WidgetTreeNode::new(WidgetNode::new(WidgetKind::Roller { options: &["Low", "Medium", "High"], selected: 1 }, Rect::new(0.0, 0.0, 90.0, 60.0), app.icon_color)),
                            WidgetTreeNode::new(WidgetNode::new(WidgetKind::TabView { tabs: &["Shell", "Display", "Input"], active: 1 }, Rect::new(0.0, 0.0, 90.0, 30.0), app.icon_color)),
                        ],
                    ),
                ],
            ),
        ],
        "Gallery" => vec![
            make_row_panel(
                app,
                Rect::new(16.0, 96.0, 388.0, 132.0),
                vec![
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Image { title: "Preview" }, Rect::new(0.0, 0.0, 160.0, 110.0), app.icon_color)
                        .with_action(WidgetAction::LaunchAppNamed("Files"))),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Chart { values: &[12, 28, 46, 39, 72, 54, 86] }, Rect::new(0.0, 0.0, 160.0, 110.0), app.icon_color)),
                ],
            ),
            make_row_panel(
                app,
                Rect::new(16.0, 236.0, 260.0, 100.0),
                vec![
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Canvas { title: "Canvas" }, Rect::new(0.0, 0.0, 140.0, 90.0), app.icon_color)),
                    make_column_panel(
                        app,
                        Rect::new(0.0, 0.0, 80.0, 90.0),
                        vec![
                            WidgetTreeNode::new(WidgetNode::new(WidgetKind::Spinner, Rect::new(0.0, 0.0, 36.0, 36.0), app.icon_color)
                                .with_action(WidgetAction::MaximizeWindow)),
                            WidgetTreeNode::new(WidgetNode::new(WidgetKind::Arc { value: 65 }, Rect::new(0.0, 0.0, 70.0, 24.0), app.icon_color)),
                        ],
                    ),
                ],
            ),
        ],
        _ => vec![
            make_column_panel(
                app,
                Rect::new(16.0, 96.0, 200.0, 120.0),
                vec![
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Label { text: "Label" }, Rect::new(0.0, 0.0, 80.0, 18.0), app.icon_color)),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::Button { text: "Theme", pressed: false }, Rect::new(0.0, 0.0, 92.0, 28.0), app.icon_color)
                        .with_action(WidgetAction::CycleTheme)),
                    WidgetTreeNode::new(WidgetNode::new(WidgetKind::MsgBox { title: "MsgBox", body: "Wing widget sample" }, Rect::new(0.0, 0.0, 180.0, 56.0), app.icon_color)),
                ],
            ),
            WidgetTreeNode::new(WidgetNode::new(WidgetKind::Calendar { month: "Apr 2026" }, Rect::new(230.0, 96.0, 120.0, 80.0), app.icon_color)),
        ],
    }
}
