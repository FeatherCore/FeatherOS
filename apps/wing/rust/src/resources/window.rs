use crate::types::WindowId;

/// Active drag transaction extracted from the transitional window manager flow.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DragTransaction {
    pub active: bool,
    pub window_id: Option<WindowId>,
    pub origin_x: f32,
    pub origin_y: f32,
}

/// Shared window-manager state targeted by Phase 2 migration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowManagerState {
    pub active_window: Option<WindowId>,
    pub dragging_window: Option<WindowId>,
    pub window_order: alloc::vec::Vec<WindowId>,
}

impl WindowManagerState {
    pub fn track_window(&mut self, window_id: WindowId) {
        if !self.window_order.contains(&window_id) {
            self.window_order.push(window_id);
        }
    }

    pub fn sync_from_order(&mut self, window_order: &[WindowId]) {
        self.window_order.clear();
        self.window_order.extend_from_slice(window_order);
    }

    pub fn bring_to_front(&mut self, window_id: WindowId) {
        if let Some(index) = self.window_order.iter().position(|id| *id == window_id) {
            self.window_order.remove(index);
        }
        self.window_order.push(window_id);
    }
}

impl fhre::resources::Resource for DragTransaction {}
impl fhre::resources::Resource for WindowManagerState {}
