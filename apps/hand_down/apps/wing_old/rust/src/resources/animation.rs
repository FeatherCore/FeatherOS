use super::shell::ShellOverlayMode;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShellOverlayAnimation {
    pub notification_panel_progress: f32,
    pub app_switcher_progress: f32,
    pub target_notification_panel: f32,
    pub target_app_switcher: f32,
    pub overlay_alpha: f32,
    pub card_alpha: f32,
    /// Previous overlay mode for transition detection
    pub previous_mode: crate::resources::ShellOverlayMode,
}

impl ShellOverlayAnimation {
    pub const ANIMATION_SPEED: f32 = 4.0;
    pub const OVERLAY_MAX_ALPHA: f32 = 0.6;
    pub const CARD_MAX_ALPHA: f32 = 1.0;

    pub fn set_notification_panel_open(&mut self, open: bool) {
        self.target_notification_panel = if open { 1.0 } else { 0.0 };
    }

    pub fn set_app_switcher_open(&mut self, open: bool) {
        self.target_app_switcher = if open { 1.0 } else { 0.0 };
    }

    pub fn update(&mut self, delta_seconds: f32) {
        let speed = Self::ANIMATION_SPEED * delta_seconds;

        if (self.notification_panel_progress - self.target_notification_panel).abs() > 0.001 {
            if self.notification_panel_progress < self.target_notification_panel {
                self.notification_panel_progress = (self.notification_panel_progress + speed).min(self.target_notification_panel);
            } else {
                self.notification_panel_progress = (self.notification_panel_progress - speed).max(self.target_notification_panel);
            }
        } else {
            self.notification_panel_progress = self.target_notification_panel;
        }

        if (self.app_switcher_progress - self.target_app_switcher).abs() > 0.001 {
            if self.app_switcher_progress < self.target_app_switcher {
                self.app_switcher_progress = (self.app_switcher_progress + speed).min(self.target_app_switcher);
            } else {
                self.app_switcher_progress = (self.app_switcher_progress - speed).max(self.target_app_switcher);
            }
        } else {
            self.app_switcher_progress = self.target_app_switcher;
        }

        self.overlay_alpha = Self::OVERLAY_MAX_ALPHA 
            * (self.notification_panel_progress.max(self.app_switcher_progress));
        self.card_alpha = Self::CARD_MAX_ALPHA 
            * Self::ease_out_cubic(self.notification_panel_progress.max(self.app_switcher_progress));
    }

    pub fn is_animating(&self) -> bool {
        (self.notification_panel_progress - self.target_notification_panel).abs() > 0.001
            || (self.app_switcher_progress - self.target_app_switcher).abs() > 0.001
    }

    pub fn ease_out_cubic(t: f32) -> f32 {
        let one_minus_t = 1.0 - t;
        1.0 - (one_minus_t * one_minus_t * one_minus_t)
    }

    pub fn notification_panel_eased(&self) -> f32 {
        Self::ease_out_cubic(self.notification_panel_progress)
    }

    pub fn app_switcher_eased(&self) -> f32 {
        Self::ease_out_cubic(self.app_switcher_progress)
    }

    pub fn overlay_alpha_eased(&self) -> f32 {
        self.overlay_alpha
    }

    pub fn card_alpha_eased(&self) -> f32 {
        self.card_alpha
    }
}

impl fhre::resources::Resource for ShellOverlayAnimation {}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ThemeAnimation {
    pub transition_progress: f32,
    pub target_progress: f32,
    pub is_transitioning: bool,
}

impl ThemeAnimation {
    pub const TRANSITION_SPEED: f32 = 3.0;

    pub fn start_transition(&mut self) {
        self.target_progress = 1.0;
        self.is_transitioning = true;
    }

    pub fn update(&mut self, delta_seconds: f32) {
        if !self.is_transitioning {
            return;
        }

        let speed = Self::TRANSITION_SPEED * delta_seconds;
        if self.transition_progress < self.target_progress {
            self.transition_progress = (self.transition_progress + speed).min(self.target_progress);
        }

        if self.transition_progress >= 1.0 {
            self.transition_progress = 0.0;
            self.target_progress = 0.0;
            self.is_transitioning = false;
        }
    }

    pub fn alpha_factor(&self) -> f32 {
        if self.transition_progress < 0.5 {
            self.transition_progress * 2.0
        } else {
            2.0 - self.transition_progress * 2.0
        }
    }
}

impl fhre::resources::Resource for ThemeAnimation {}
