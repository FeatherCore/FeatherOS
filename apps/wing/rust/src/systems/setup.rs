use fhre::{Commands, Pickable, PickableBounds, PrimaryScreen, Res, ResMut, Transform};

use crate::components::{ButtonWidget, DesktopRoot, DesktopWallpaper, LauncherPanel, TaskbarRoot, WidgetLayoutNode, WindowChrome, WindowContentRoot, WindowContentText, WindowControlButton, WindowControlKind, WindowFocus, WindowFrame, WindowPlacement, WindowTitleBar, WindowTitleIcon, WindowTitleText};
use crate::resources::{FocusState, WindowManagerState};
use crate::types::WindowState;

const MINIMAL_WINDOW_ID: u32 = 0x5749_4E47;
const MINIMAL_WINDOW_TITLE: &str = "Wing Desktop";
const MINIMAL_WINDOW_CONTENT: &str = "Empty desktop shell";

pub fn setup_wing_desktop(
    mut commands: Commands,
    screen: Res<PrimaryScreen>,
    mut window_manager_state: ResMut<WindowManagerState>,
    mut focus_state: ResMut<FocusState>,
) {
    let width = screen.width as f32;
    let height = screen.height as f32;
    let center_x = width * 0.5;
    let center_y = height * 0.5;

    commands
        .spawn()
        .insert(DesktopRoot)
        .insert(DesktopWallpaper::visible())
        .insert(Transform::from_position(center_x, center_y, 0.0))
        .insert(WidgetLayoutNode { width, height });

    commands
        .spawn()
        .insert(TaskbarRoot)
        .insert(Transform::from_position(center_x, height - 24.0, 0.05))
        .insert(WidgetLayoutNode {
            width,
            height: 48.0,
        });

    commands
        .spawn()
        .insert(LauncherPanel)
        .insert(Transform::from_position(170.0, height - 132.0, 0.06))
        .insert(WidgetLayoutNode {
            width: 300.0,
            height: 160.0,
        });

    commands
        .spawn()
        .insert(WindowFrame::new(MINIMAL_WINDOW_ID, WindowState::Normal))
        .insert(WindowPlacement::new(center_x, center_y - 24.0, 320.0, 220.0))
        .insert(WindowFocus::focused())
        .insert(PickableBounds::from_size(320.0, 220.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, center_y - 24.0, 0.10))
        .insert(WidgetLayoutNode {
            width: 320.0,
            height: 220.0,
        });

    commands
        .spawn()
        .insert(WindowTitleBar::new(MINIMAL_WINDOW_ID))
        .insert(WindowChrome::draggable())
        .insert(PickableBounds::from_size(320.0, 28.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x, center_y - 120.0, 0.11))
        .insert(WidgetLayoutNode {
            width: 320.0,
            height: 28.0,
        });

    commands
        .spawn()
        .insert(WindowTitleText::new(MINIMAL_WINDOW_ID, MINIMAL_WINDOW_TITLE))
        .insert(Transform::from_position(center_x - 96.0, center_y - 120.0, 0.115))
        .insert(WidgetLayoutNode {
            width: 160.0,
            height: 18.0,
        });

    commands
        .spawn()
        .insert(WindowTitleIcon::new(MINIMAL_WINDOW_ID))
        .insert(Transform::from_position(center_x - 140.0, center_y - 120.0, 0.114))
        .insert(WidgetLayoutNode {
            width: 12.0,
            height: 12.0,
        });

    commands
        .spawn()
        .insert(WindowContentRoot::new(MINIMAL_WINDOW_ID))
        .insert(Transform::from_position(center_x, center_y - 10.0, 0.105))
        .insert(WidgetLayoutNode {
            width: 296.0,
            height: 180.0,
        });

    commands
        .spawn()
        .insert(WindowContentText::new(MINIMAL_WINDOW_ID, MINIMAL_WINDOW_CONTENT))
        .insert(Transform::from_position(center_x, center_y - 10.0, 0.106))
        .insert(WidgetLayoutNode {
            width: 200.0,
            height: 18.0,
        });

    commands
        .spawn()
        .insert(WindowControlButton::new(MINIMAL_WINDOW_ID, WindowControlKind::Minimize))
        .insert(ButtonWidget::idle())
        .insert(PickableBounds::from_size(14.0, 14.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x + 104.0, center_y - 120.0, 0.12))
        .insert(WidgetLayoutNode {
            width: 14.0,
            height: 14.0,
        });

    commands
        .spawn()
        .insert(WindowControlButton::new(MINIMAL_WINDOW_ID, WindowControlKind::Maximize))
        .insert(ButtonWidget::idle())
        .insert(PickableBounds::from_size(14.0, 14.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x + 124.0, center_y - 120.0, 0.12))
        .insert(WidgetLayoutNode {
            width: 14.0,
            height: 14.0,
        });

    commands
        .spawn()
        .insert(WindowControlButton::new(MINIMAL_WINDOW_ID, WindowControlKind::Close))
        .insert(ButtonWidget::idle())
        .insert(PickableBounds::from_size(14.0, 14.0))
        .insert(Pickable::DEFAULT)
        .insert(Transform::from_position(center_x + 144.0, center_y - 120.0, 0.12))
        .insert(WidgetLayoutNode {
            width: 14.0,
            height: 14.0,
        });

    window_manager_state.active_window = Some(MINIMAL_WINDOW_ID);
    window_manager_state.dragging_window = None;
    window_manager_state.track_window(MINIMAL_WINDOW_ID);

    focus_state.window_id = Some(MINIMAL_WINDOW_ID);
    focus_state.widget_id = None;
}
