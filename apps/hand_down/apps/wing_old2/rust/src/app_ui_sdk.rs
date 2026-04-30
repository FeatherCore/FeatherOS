pub mod app {
    pub use crate::app_ui::{AppUiApp, AppUiContext, AppUiScheduledApp};
}

pub mod command {
    pub use crate::app_ui::{
        AppUiCommand, AppUiCommandQueue, AppUiEventQueue, AppUiFlow, AppUiSignal, AppUiSignalId,
        AppUiUpdateResult,
    };
}

pub mod geometry {
    pub use crate::math::{Color, Point, Rect, Size};
}

pub mod input {
    pub use crate::app_ui::{AppUiGesture, AppUiKey, AppUiSwipe};
}

pub mod layout {
    pub use crate::app_ui::{AppUiAxis, AppUiGrid, AppUiPadding, AppUiStack};
}

pub mod render {
    pub use crate::render::VectorIcon;
    pub use crate::app_ui::{
        app_ui_budget_label, app_ui_budget_pressure, app_ui_budget_summary,
        app_ui_frame_flags_label, app_ui_frame_result_label, AppUiDiagnosticsSummary, AppUiFrame,
        AppUiFrameBudget, AppUiFrameFlags, AppUiFrameHint, AppUiFramePacing, AppUiFrameResult,
        AppUiFrameStats, AppUiRenderResult, AppUiRuntime, APP_UI_DEFAULT_CAPACITY,
        APP_UI_DIRTY_RECT_CAPACITY,
    };
}

pub mod runner {
    pub use crate::app_ui::{
        run_app_ui_rgb565, run_app_ui_rgb565_from_argv,
        run_app_ui_rgb565_from_argv_with_svg_store, run_app_ui_rgb565_with_svg_store, AppUiLoop,
    };
}

pub mod schedule {
    pub use crate::app_ui::{
        AppUiApplyCommand, AppUiSchedule, AppUiStagedSchedule, AppUiSystem,
    };
}

pub mod settings {
    pub use crate::app_sdk::{
        parse_settings_snapshot, WingSettingsClient, WingSettingsError, WingSettingsKey,
    };
}

pub mod surface {
    pub use crate::app_sdk::{
        WingDirtyRect, WingPointerEvent, WingPointerKind, WingSurface, WingSurfaceDescriptor,
        WingSurfaceError, WingSurfaceFormat, WingSurfaceInput, WingSurfaceTransport,
    };
}

pub mod widget {
    pub use crate::app_ui::{
        scroll_list_first_after_swipe, scroll_list_first_from_point, scroll_list_handle_key,
        scroll_list_metrics, scroll_list_visible_rows, slider_value_from_point,
        stepper_decrement_key, stepper_increment_key, AppUiBuilder, AppUiBundle, AppUiButtonCircle,
        AppUiButtonRect, AppUiButtonRoundRect, AppUiCircle, AppUiDialog, AppUiDialogButton,
        AppUiDialogStyle, AppUiDragHandle, AppUiHitArea, AppUiLabel, AppUiListItem,
        AppUiIcon, AppUiListStyle, AppUiRadioList, AppUiRadioStyle, AppUiRect, AppUiRoundRect,
        AppUiRowStyle, AppUiScrollList, AppUiScrollMetrics, AppUiSegment, AppUiSegmentStyle,
        AppUiSegmented, AppUiSlider, AppUiStatusRow, AppUiStepper, AppUiSwitchRow, AppUiToast,
        AppUiToastStyle, AppUiToggle, AppUiTopBar, AppUiTopBarStyle, AppUiView,
    };
}

pub mod prelude {
    pub use super::app::*;
    pub use super::command::*;
    pub use super::geometry::*;
    pub use super::input::*;
    pub use super::layout::*;
    pub use super::render::*;
    pub use super::runner::*;
    pub use super::schedule::*;
    pub use super::surface::WingSurface;
    pub use super::widget::*;
}
