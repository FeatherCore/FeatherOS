//! Window management.

use alloc::vec::Vec;
use fhre::{Color, RenderCommand, Vec2};
use fhre::math::Rect;

use crate::launcher::AppInfo;
use crate::theme::{ThemePalette, shell_palette};
use crate::widgets::{TextEditCommand, WidgetAction, WidgetEvent, WidgetId, WidgetKind, WidgetNode, WidgetResponse, WidgetTreeNode, default_widget_tree_for_app};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowWidgetEvent {
    pub window_id: WindowId,
    pub event: WidgetEvent,
    pub action: Option<WidgetAction>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CalculatorState {
    accumulator: i32,
    pending_op: Option<char>,
    reset_input: bool,
}

fn assign_tree_ids(tree: &mut WidgetTreeNode, next_id: WidgetId) -> WidgetId {
    tree.widget.id = next_id;
    let mut current = next_id + 1;
    for child in &mut tree.children {
        current = assign_tree_ids(child, current);
    }
    current
}

fn widget_local_point(tree: &WidgetTreeNode, widget_id: WidgetId, point: Vec2, parent_offset: Vec2) -> Option<Vec2> {
    let offset = Vec2::new(parent_offset.x + tree.widget.rect.x, parent_offset.y + tree.widget.rect.y);
    if tree.widget.id == widget_id {
        return Some(Vec2::new(point.x - offset.x, point.y - offset.y));
    }
    for child in &tree.children {
        if let Some(local) = widget_local_point(child, widget_id, point, offset) {
            return Some(local);
        }
    }
    None
}

fn local_to_widget_space(trees: &[WidgetTreeNode], widget_id: WidgetId, point: Vec2) -> Vec2 {
    for tree in trees {
        if let Some(local) = widget_local_point(tree, widget_id, point, Vec2::ZERO) {
            return local;
        }
    }
    point
}

/// Unique window identifier.
pub type WindowId = u32;

/// Window state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowHit {
    Close,
    Minimize,
    Maximize,
    TitleBar,
    Content,
    Outside,
}

/// Desktop window.
pub struct Window {
    pub id: WindowId,
    pub title: &'static str,
    pub position: Vec2,
    pub size: Vec2,
    pub state: WindowState,
    pub is_focused: bool,
    pub content_color: Color,
    pub border_color: Color,
    pub accent_color: Color,
    pub theme: ThemePalette,
    pub title_bar_height: f32,
    pub border_width: f32,
    pub widgets: Vec<WidgetTreeNode>,
    hovered_widget: Option<WidgetId>,
    pressed_widget: Option<WidgetId>,
    pending_widget_responses: Vec<WidgetResponse>,
    calculator: CalculatorState,
    restore_bounds: Option<(Vec2, Vec2)>,
}

impl Window {
    pub fn new(id: WindowId, title: &'static str, size: Vec2, position: Vec2) -> Self {
        let palette = shell_palette();
        Self {
            id,
            title,
            position,
            size,
            state: WindowState::Normal,
            is_focused: false,
            content_color: palette.surface,
            border_color: palette.border,
            accent_color: palette.accent,
            theme: palette,
            title_bar_height: 28.0,
            border_width: 2.0,
            widgets: Vec::new(),
            hovered_widget: None,
            pressed_widget: None,
            pending_widget_responses: Vec::new(),
            calculator: CalculatorState::default(),
            restore_bounds: None,
        }
    }

    pub fn with_accent_color(mut self, accent_color: Color) -> Self {
        self.accent_color = accent_color;
        self.border_color = accent_color;
        self.content_color = self.theme.surface;
        self
    }

    pub fn apply_theme(&mut self, palette: ThemePalette) {
        self.theme = palette;
        self.content_color = palette.surface;
        self.border_color = if self.is_focused { self.accent_color } else { palette.border };
        for widget in &mut self.widgets {
            widget.apply_theme(palette);
        }
    }

    pub fn with_widgets(mut self, widgets: Vec<WidgetTreeNode>) -> Self {
        self.widgets = widgets
            .into_iter()
            .enumerate()
            .map(|(index, mut tree)| {
                assign_tree_ids(&mut tree, ((index + 1) * 100) as u32);
                tree.relayout();
                tree
            })
            .collect();
        self
    }

    pub fn get_rect(&self) -> Rect {
        Rect::new(
            self.position.x - self.border_width,
            self.position.y - self.title_bar_height - self.border_width,
            self.size.x + self.border_width * 2.0,
            self.size.y + self.title_bar_height + self.border_width * 2.0,
        )
    }

    pub fn get_content_rect(&self) -> Rect {
        Rect::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    pub fn get_title_bar_rect(&self) -> Rect {
        Rect::new(
            self.position.x - self.border_width,
            self.position.y - self.title_bar_height - self.border_width,
            self.size.x + self.border_width * 2.0,
            self.title_bar_height,
        )
    }

    pub fn close_button_rect(&self) -> Rect {
        let title_bar = self.get_title_bar_rect();
        Rect::new(title_bar.right() - 24.0, title_bar.y + 6.0, 14.0, 14.0)
    }

    pub fn maximize_button_rect(&self) -> Rect {
        let title_bar = self.get_title_bar_rect();
        Rect::new(title_bar.right() - 44.0, title_bar.y + 6.0, 14.0, 14.0)
    }

    pub fn minimize_button_rect(&self) -> Rect {
        let title_bar = self.get_title_bar_rect();
        Rect::new(title_bar.right() - 64.0, title_bar.y + 6.0, 14.0, 14.0)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        self.get_rect().contains(point)
    }

    fn content_local_point(&self, point: Vec2) -> Option<Vec2> {
        let content = self.get_content_rect();
        if !content.contains(point) {
            return None;
        }
        Some(Vec2::new(point.x - content.x, point.y - content.y))
    }

    fn handle_content_click(&mut self, point: Vec2) -> bool {
        self.handle_content_press(point).consumed
    }

    fn handle_content_move(&mut self, point: Vec2) -> WidgetResponse {
        let local = self.content_local_point(point);
        let hovered = local.and_then(|local_point| {
            self.widgets
                .iter()
                .rev()
                .find_map(|widget| widget.hit_test(local_point))
        });

        if self.hovered_widget != hovered {
            if let Some(previous_id) = self.hovered_widget {
                if let Some(widget) = self.widget_by_id_mut(previous_id) {
                    if let Some(event) = widget.set_hovered(false) {
                        self.pending_widget_responses.push(WidgetResponse {
                            consumed: false,
                            event: Some(event),
                            action: None,
                        });
                    }
                }
            }
            if let Some(current_id) = hovered {
                if let Some(widget) = self.widget_by_id_mut(current_id) {
                    if let Some(event) = widget.set_hovered(true) {
                        self.pending_widget_responses.push(WidgetResponse {
                            consumed: false,
                            event: Some(event),
                            action: None,
                        });
                    }
                }
            }
            self.hovered_widget = hovered;
        }

        if let (Some(local_point), Some(pressed_id)) = (local, self.pressed_widget) {
            if let Some(widget) = self.widget_by_id_mut(pressed_id) {
                let response = widget.drag(local_point);
                if response.event.is_some() {
                    self.pending_widget_responses.push(response);
                }
                return response;
            }
        }

        WidgetResponse::default()
    }

    fn handle_content_press(&mut self, point: Vec2) -> WidgetResponse {
        let local = match self.content_local_point(point) {
            Some(local) => local,
            None => return WidgetResponse::default(),
        };

        if let Some(widget_id) = self.widgets.iter().rev().find_map(|widget| widget.hit_test(local)) {
            let widget_local = local_to_widget_space(&self.widgets, widget_id, local);
            if let Some(widget) = self.widget_by_id_mut(widget_id) {
                let response = widget.press(widget_local);
                if response.consumed {
                    self.pressed_widget = Some(widget.id);
                    self.pending_widget_responses.push(response);
                    return response;
                }
            }
        }

        WidgetResponse::default()
    }

    fn handle_content_release(&mut self, point: Vec2) -> WidgetResponse {
        let local = self.content_local_point(point);
        let pressed_id = match self.pressed_widget.take() {
            Some(id) => id,
            None => return WidgetResponse::default(),
        };

        let widget_local = local
            .map(|p| local_to_widget_space(&self.widgets, pressed_id, p))
            .unwrap_or(Vec2::new(-1.0, -1.0));

        let response = if let Some(widget) = self.widget_by_id_mut(pressed_id) {
            let response = widget.release(widget_local);
            if response.event.is_some() {
                self.pending_widget_responses.push(response);
            }
            response
        } else {
            WidgetResponse::default()
        };

        response
    }

    fn clear_widget_hover(&mut self) {
        if let Some(widget_id) = self.hovered_widget.take() {
            if let Some(widget) = self.widget_by_id_mut(widget_id) {
                if let Some(event) = widget.set_hovered(false) {
                    self.pending_widget_responses.push(WidgetResponse {
                        consumed: false,
                        event: Some(event),
                        action: None,
                    });
                }
            }
        }
    }

    fn drain_widget_responses(&mut self) -> Vec<WidgetResponse> {
        core::mem::take(&mut self.pending_widget_responses)
    }

    fn widget_by_id_mut(&mut self, widget_id: WidgetId) -> Option<&mut WidgetNode> {
        for tree in &mut self.widgets {
            if let Some(widget) = tree.find_widget_mut(widget_id) {
                return Some(widget);
            }
        }
        None
    }

    pub fn widget_by_id(&self, widget_id: WidgetId) -> Option<&WidgetNode> {
        for tree in &self.widgets {
            if let Some(widget) = tree.find_widget(widget_id) {
                return Some(widget);
            }
        }
        None
    }

    pub fn apply_button_matrix_selection(&mut self, widget_id: WidgetId) {
        let selected_label = self.widget_by_id(widget_id).and_then(|widget| match &widget.kind {
            WidgetKind::ButtonMatrix { labels, selected: Some(index), .. } => labels.get(*index).copied(),
            _ => None,
        });

        let Some(label) = selected_label else { return; };

        for tree in &mut self.widgets {
            if let Some(widget) = tree.find_widget_mut(101) {
                if let WidgetKind::SpinBox { value } = &mut widget.kind {
                    match label {
                        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                            let digit = (label.as_bytes()[0] - b'0') as i32;
                            if self.calculator.reset_input {
                                *value = digit;
                                self.calculator.reset_input = false;
                            } else {
                                *value = value.saturating_mul(10).saturating_add(digit);
                            }
                        }
                        "+" | "-" | "*" | "/" => {
                            let current = *value;
                            if let Some(op) = self.calculator.pending_op {
                                self.calculator.accumulator = apply_calc_op(self.calculator.accumulator, current, op);
                                *value = self.calculator.accumulator;
                            } else {
                                self.calculator.accumulator = current;
                            }
                            self.calculator.pending_op = Some(label.as_bytes()[0] as char);
                            self.calculator.reset_input = true;
                        }
                        "=" => {
                            if let Some(op) = self.calculator.pending_op.take() {
                                let result = apply_calc_op(self.calculator.accumulator, *value, op);
                                *value = result;
                                self.calculator.accumulator = result;
                                self.calculator.reset_input = true;
                            }
                        }
                        "C" => {
                            *value = 0;
                            self.calculator = CalculatorState::default();
                        }
                        _ => {}
                    }
                    break;
                }
            }
        }
    }

    pub fn handle_text_edit(&mut self, command: TextEditCommand) -> bool {
        if let Some(widget_id) = self.pressed_widget.or(self.hovered_widget) {
            if let Some(widget) = self.widget_by_id_mut(widget_id) {
                return widget.handle_text_edit(command);
            }
        }

        for tree in &mut self.widgets {
            if let Some(widget) = tree.find_widget_mut(100) {
                if widget.handle_text_edit(command) {
                    return true;
                }
            }
        }

        false
    }

    fn hit_test(&self, point: Vec2) -> WindowHit {
        if self.close_button_rect().contains(point) {
            WindowHit::Close
        } else if self.maximize_button_rect().contains(point) {
            WindowHit::Maximize
        } else if self.minimize_button_rect().contains(point) {
            WindowHit::Minimize
        } else if self.get_title_bar_rect().contains(point) {
            WindowHit::TitleBar
        } else if self.get_content_rect().contains(point) {
            WindowHit::Content
        } else {
            WindowHit::Outside
        }
    }

    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        if self.state == WindowState::Minimized || self.state == WindowState::Closed {
            return commands;
        }

        let rect = self.get_rect();
        let title_bar = self.get_title_bar_rect();
        let content = self.get_content_rect();
        let palette = self.theme;
        let frame_color = if self.is_focused {
            self.accent_color
        } else {
            self.border_color
        };

        commands.push(RenderCommand::draw_rect_rounded(
            Rect::new(rect.x + 4.0, rect.y + 6.0, rect.width, rect.height),
            palette.background,
            14.0,
        ));
        commands.push(RenderCommand::draw_rect_rounded(rect, frame_color, 14.0));
        commands.push(RenderCommand::draw_rect_rounded(
            Rect::new(title_bar.x + 2.0, title_bar.y + 2.0, title_bar.width - 4.0, title_bar.height),
            if self.is_focused {
                palette.background
            } else {
                palette.surface_alt
            },
            12.0,
        ));
        commands.push(RenderCommand::draw_rect(
            content,
            self.content_color,
        ));
        commands.push(RenderCommand::draw_text(
            Vec2::new(title_bar.x + 12.0, title_bar.y + 18.0),
            self.title,
            Color::WHITE,
            14.0,
        ));

        commands.push(RenderCommand::draw_rect_rounded(self.minimize_button_rect(), Color::rgb(255, 191, 71), 7.0));
        commands.push(RenderCommand::draw_rect_rounded(self.maximize_button_rect(), Color::rgb(88, 201, 126), 7.0));
        commands.push(RenderCommand::draw_rect_rounded(self.close_button_rect(), Color::rgb(255, 107, 107), 7.0));

        let card = Rect::new(content.x + 16.0, content.y + 18.0, content.width - 32.0, 62.0);
        commands.push(RenderCommand::draw_rect_rounded(card, self.accent_color, 12.0));
        commands.push(RenderCommand::draw_text(
            Vec2::new(card.x + 14.0, card.y + 18.0),
            self.title,
            Color::WHITE,
            16.0,
        ));
        commands.push(RenderCommand::draw_text(
            Vec2::new(card.x + 14.0, card.y + 38.0),
            "FHRE surface attached",
            palette.surface,
            12.0,
        ));

        for index in 0..3 {
            let bar = Rect::new(
                content.x + 16.0,
                content.y + 96.0 + index as f32 * 28.0,
                content.width - 32.0 - index as f32 * 30.0,
                14.0,
            );
            commands.push(RenderCommand::draw_rect_rounded(bar, palette.border, 7.0));
        }

        commands.push(RenderCommand::set_scissor(content));
        for widget in &self.widgets {
            commands.extend(widget.render(content.min()));
        }
        commands.push(RenderCommand::disable_scissor());

        commands
    }
}

fn apply_calc_op(lhs: i32, rhs: i32, op: char) -> i32 {
    match op {
        '+' => lhs.saturating_add(rhs),
        '-' => lhs.saturating_sub(rhs),
        '*' => lhs.saturating_mul(rhs),
        '/' => {
            if rhs == 0 { lhs } else { lhs / rhs }
        }
        _ => rhs,
    }
}

/// Window manager.
pub struct WindowManager {
    windows: Vec<Window>,
    next_id: WindowId,
    focused_window: Option<WindowId>,
    dragging_window: Option<WindowId>,
    screen_size: Vec2,
    window_order: Vec<WindowId>,
    widget_events: Vec<WindowWidgetEvent>,
}

impl WindowManager {
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            windows: Vec::new(),
            next_id: 1,
            focused_window: None,
            dragging_window: None,
            screen_size,
            window_order: Vec::new(),
            widget_events: Vec::new(),
        }
    }

    pub fn windows(&self) -> &[Window] {
        &self.windows
    }

    pub fn focused_window(&self) -> Option<WindowId> {
        self.focused_window
    }

    pub fn dragging_window(&self) -> Option<WindowId> {
        self.dragging_window
    }

    pub fn resize_screen(&mut self, screen_size: Vec2) {
        self.screen_size = screen_size;
    }

    pub fn apply_theme(&mut self, palette: ThemePalette) {
        for window in &mut self.windows {
            window.apply_theme(palette);
        }
    }

    pub fn create_window(&mut self, title: &'static str, size: Vec2, position: Vec2) -> WindowId {
        let id = self.next_id;
        self.next_id += 1;

        let window = Window::new(id, title, size, self.clamp_position(position, size));
        self.windows.push(window);
        self.window_order.insert(0, id);
        self.focus_window(id);
        id
    }

    pub fn create_app_window(&mut self, app_info: &AppInfo) -> WindowId {
        let id = self.next_id;
        self.next_id += 1;

        let window = Window::new(
            id,
            app_info.name,
            app_info.default_size,
            self.clamp_position(app_info.initial_position, app_info.default_size),
        )
        .with_accent_color(app_info.icon_color)
        .with_widgets(default_widget_tree_for_app(app_info));

        self.windows.push(window);
        self.window_order.insert(0, id);
        self.focus_window(id);
        id
    }

    pub fn close_window(&mut self, window_id: WindowId) {
        if let Some(index) = self.windows.iter().position(|window| window.id == window_id) {
            self.windows.remove(index);
        }
        if let Some(index) = self.window_order.iter().position(|id| *id == window_id) {
            self.window_order.remove(index);
        }
        if self.focused_window == Some(window_id) {
            self.focused_window = None;
            if let Some(next) = self.window_order.first().copied() {
                self.focus_window(next);
            }
        }
        if self.dragging_window == Some(window_id) {
            self.dragging_window = None;
        }
    }

    pub fn get_window(&self, window_id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|window| window.id == window_id)
    }

    pub fn get_widget(&self, window_id: WindowId, widget_id: WidgetId) -> Option<&WidgetNode> {
        self.get_window(window_id)?.widget_by_id(widget_id)
    }

    pub fn apply_button_matrix_selection(&mut self, window_id: WindowId, widget_id: WidgetId) {
        if let Some(window) = self.get_window_mut(window_id) {
            window.apply_button_matrix_selection(widget_id);
        }
    }

    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|window| window.id == window_id)
    }

    pub fn drain_widget_events(&mut self) -> Vec<WindowWidgetEvent> {
        core::mem::take(&mut self.widget_events)
    }

    pub fn handle_mouse_move(&mut self, position: Vec2) {
        let active_window = self.get_window_at(position);
        let mut queued = Vec::new();
        for window in &mut self.windows {
            if Some(window.id) != active_window {
                window.clear_widget_hover();
            }
            let window_id = window.id;
            queued.extend(window.drain_widget_responses().into_iter().filter_map(|response| {
                response.event.map(|event| WindowWidgetEvent {
                    window_id,
                    event,
                    action: response.action,
                })
            }));
        }
        if let Some(window_id) = active_window {
            if let Some(window) = self.get_window_mut(window_id) {
                let _ = window.handle_content_move(position);
                queued.extend(window.drain_widget_responses().into_iter().filter_map(|response| {
                    response.event.map(|event| WindowWidgetEvent {
                        window_id,
                        event,
                        action: response.action,
                    })
                }));
            }
        }
        self.widget_events.extend(queued);
    }

    pub fn clear_focus(&mut self) {
        self.focused_window = None;
        for window in &mut self.windows {
            window.is_focused = false;
        }
    }

    pub fn focus_window(&mut self, window_id: WindowId) {
        for window in &mut self.windows {
            window.is_focused = window.id == window_id;
        }

        self.focused_window = Some(window_id);
        if let Some(index) = self.window_order.iter().position(|id| *id == window_id) {
            self.window_order.remove(index);
            self.window_order.insert(0, window_id);
        }

        if let Some(window) = self.get_window_mut(window_id) {
            if window.state == WindowState::Minimized {
                window.state = WindowState::Normal;
            }
        }
    }

    pub fn move_window(&mut self, window_id: WindowId, delta: Vec2) {
        let screen_size = self.screen_size;
        if let Some(window) = self.get_window_mut(window_id) {
            if window.state != WindowState::Normal {
                return;
            }
            window.position.x += delta.x;
            window.position.y += delta.y;
            window.position.x = window.position.x.max(0.0).min((screen_size.x - window.size.x).max(0.0));
            window.position.y = window.position.y.max(window.title_bar_height + 4.0).min((screen_size.y - 8.0).max(window.title_bar_height + 4.0));
        }
    }

    pub fn toggle_window_visibility(&mut self, window_id: WindowId) {
        let is_focused = self.focused_window == Some(window_id);
        if let Some(window) = self.get_window_mut(window_id) {
            if window.state == WindowState::Minimized {
                window.state = WindowState::Normal;
                self.focus_window(window_id);
            } else if is_focused {
                window.state = WindowState::Minimized;
                window.is_focused = false;
                self.focused_window = None;
            } else {
                self.focus_window(window_id);
            }
        }
    }

    pub fn toggle_maximized(&mut self, window_id: WindowId) {
        let screen_size = self.screen_size;
        if let Some(window) = self.get_window_mut(window_id) {
            match window.state {
                WindowState::Maximized => {
                    if let Some((position, size)) = window.restore_bounds.take() {
                        window.position = position;
                        window.size = size;
                    }
                    window.state = WindowState::Normal;
                }
                WindowState::Normal => {
                    window.restore_bounds = Some((window.position, window.size));
                    window.position = Vec2::new(8.0, window.title_bar_height + 12.0);
                    window.size = Vec2::new(screen_size.x - 16.0, screen_size.y - window.title_bar_height - 76.0);
                    window.state = WindowState::Maximized;
                }
                _ => {}
            }
        }
    }

    pub fn get_window_at(&self, point: Vec2) -> Option<WindowId> {
        for window_id in &self.window_order {
            if let Some(window) = self.get_window(*window_id) {
                if window.state != WindowState::Minimized && window.contains(point) {
                    return Some(*window_id);
                }
            }
        }
        None
    }

    pub fn handle_click(&mut self, window_id: WindowId, position: Vec2) -> bool {
        let hit = match self.get_window(window_id) {
            Some(window) => window.hit_test(position),
            None => return false,
        };

        match hit {
            WindowHit::Close => {
                self.close_window(window_id);
                true
            }
            WindowHit::Minimize => {
                self.toggle_window_visibility(window_id);
                false
            }
            WindowHit::Maximize => {
                self.toggle_maximized(window_id);
                false
            }
            WindowHit::TitleBar => {
                self.dragging_window = Some(window_id);
                false
            }
            WindowHit::Content => {
                if let Some(window) = self.get_window_mut(window_id) {
                    let consumed = window.handle_content_click(position);
                    let queued: Vec<WindowWidgetEvent> = window.drain_widget_responses().into_iter().filter_map(|response| {
                        response.event.map(|event| WindowWidgetEvent {
                            window_id,
                            event,
                            action: response.action,
                        })
                    }).collect();
                    self.widget_events.extend(queued);
                    consumed
                } else {
                    false
                }
            }
            WindowHit::Outside => false,
        }
    }

    pub fn handle_mouse_release(&mut self, position: Vec2) {
        if let Some(window_id) = self.focused_window {
            if let Some(window) = self.get_window_mut(window_id) {
                let _ = window.handle_content_release(position);
                let queued: Vec<WindowWidgetEvent> = window.drain_widget_responses().into_iter().filter_map(|response| {
                    response.event.map(|event| WindowWidgetEvent {
                        window_id,
                        event,
                        action: response.action,
                    })
                }).collect();
                self.widget_events.extend(queued);
            }
        }
    }

    pub fn handle_text_edit(&mut self, command: TextEditCommand) {
        if let Some(window_id) = self.focused_window {
            if let Some(window) = self.get_window_mut(window_id) {
                let _ = window.handle_text_edit(command);
            }
        }
    }

    pub fn stop_dragging(&mut self) {
        self.dragging_window = None;
    }

    pub fn update(&mut self, _delta_time: f32) {}

    pub fn generate_render_commands(&self) -> Vec<RenderCommand> {
        let mut commands = Vec::new();
        for window_id in self.window_order.iter().rev() {
            if let Some(window) = self.get_window(*window_id) {
                commands.extend(window.generate_render_commands());
            }
        }
        commands
    }

    fn clamp_position(&self, position: Vec2, size: Vec2) -> Vec2 {
        Vec2::new(
            position.x.max(8.0).min((self.screen_size.x - size.x - 8.0).max(8.0)),
            position.y.max(48.0).min((self.screen_size.y - size.y - 56.0).max(48.0)),
        )
    }
}
