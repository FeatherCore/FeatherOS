use crate::math::Color;
use crate::shell::ThemeKind;
use crate::ui::{BorderStyle, DrawStyle, ShadowStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShellStyleTokens {
    pub(crate) accent: Color,
    pub(crate) accent_bar: Color,
    icon_muted: Color,
    text_dark: Color,
    text_muted: Color,
    pub(crate) panel: DrawStyle,
    pub(crate) back_button: DrawStyle,
    pub(crate) settings_row: DrawStyle,
    pub(crate) system_row: DrawStyle,
    quick_active: DrawStyle,
    quick_inactive: DrawStyle,
    notification_priority: DrawStyle,
    notification_plain: DrawStyle,
    pub(crate) app_card: DrawStyle,
}

impl ShellStyleTokens {
    pub(crate) fn for_theme(theme: ThemeKind) -> Self {
        let (accent, accent_bar, icon_muted) = match theme {
            ThemeKind::Aurora => (
                Color::rgba(58, 116, 255, 232),
                Color::rgba(58, 116, 255, 220),
                Color::rgba(63, 78, 102, 220),
            ),
            ThemeKind::Dusk => (
                Color::rgba(232, 92, 142, 232),
                Color::rgba(236, 104, 142, 220),
                Color::rgba(84, 58, 78, 220),
            ),
        };
        Self {
            accent,
            accent_bar,
            icon_muted,
            text_dark: Color::rgba(18, 32, 58, 238),
            text_muted: Color::rgba(50, 66, 92, 210),
            panel: DrawStyle::round_rect(Color::rgba(255, 255, 255, 24), 24)
                .with_vertical_gradient(
                    Color::rgba(255, 255, 255, 42),
                    Color::rgba(255, 255, 255, 18),
                )
                .with_border(BorderStyle::new(Color::rgba(255, 255, 255, 42), 1)),
            back_button: DrawStyle::round_rect(Color::rgba(255, 255, 255, 34), 10)
                .with_vertical_gradient(
                    Color::rgba(255, 255, 255, 56),
                    Color::rgba(255, 255, 255, 24),
                )
                .with_border(BorderStyle::new(Color::rgba(255, 255, 255, 62), 1)),
            settings_row: DrawStyle::round_rect(Color::rgba(255, 255, 255, 30), 14)
                .with_vertical_gradient(
                    Color::rgba(255, 255, 255, 52),
                    Color::rgba(255, 255, 255, 20),
                )
                .with_border(BorderStyle::new(Color::rgba(255, 255, 255, 44), 1)),
            system_row: DrawStyle::round_rect(Color::rgba(255, 255, 255, 24), 8)
                .with_vertical_gradient(
                    Color::rgba(255, 255, 255, 38),
                    Color::rgba(255, 255, 255, 16),
                )
                .with_border(BorderStyle::new(Color::rgba(255, 255, 255, 34), 1)),
            quick_active: DrawStyle::round_rect(accent.with_alpha(238), 16)
                .with_vertical_gradient(accent.with_alpha(248), accent_bar.with_alpha(224))
                .with_shadow(ShadowStyle::new(Color::rgba(0, 0, 0, 36), 0, 6, 8, 1)),
            quick_inactive: DrawStyle::round_rect(Color::rgba(242, 246, 252, 222), 16)
                .with_vertical_gradient(
                    Color::rgba(255, 255, 255, 238),
                    Color::rgba(232, 238, 248, 214),
                )
                .with_shadow(ShadowStyle::new(Color::rgba(0, 0, 0, 36), 0, 6, 8, 1)),
            notification_priority: DrawStyle::round_rect(accent, 18)
                .with_vertical_gradient(accent.with_alpha(248), accent_bar.with_alpha(232))
                .with_shadow(ShadowStyle::new(Color::rgba(0, 0, 0, 42), 0, 8, 10, 1)),
            notification_plain: DrawStyle::round_rect(Color::rgba(255, 255, 255, 232), 18)
                .with_vertical_gradient(
                    Color::rgba(255, 255, 255, 242),
                    Color::rgba(236, 241, 250, 224),
                )
                .with_shadow(ShadowStyle::new(Color::rgba(0, 0, 0, 42), 0, 8, 10, 1)),
            app_card: DrawStyle::round_rect(Color::rgba(255, 255, 255, 232), 18)
                .with_vertical_gradient(
                    Color::rgba(255, 255, 255, 242),
                    Color::rgba(234, 240, 250, 224),
                )
                .with_shadow(ShadowStyle::new(Color::rgba(0, 0, 0, 58), 2, 8, 12, 1)),
        }
    }

    pub(crate) fn quick_toggle(self, active: bool) -> DrawStyle {
        if active {
            self.quick_active
        } else {
            self.quick_inactive
        }
    }

    pub(crate) fn quick_icon_tint(self, active: bool) -> Color {
        if active {
            Color::rgba(255, 255, 255, 235)
        } else {
            self.icon_muted
        }
    }

    pub(crate) fn quick_label_color(self, active: bool) -> Color {
        if active {
            Color::rgba(255, 255, 255, 232)
        } else {
            self.text_dark
        }
    }

    pub(crate) fn notification_card(self, priority: bool) -> DrawStyle {
        if priority {
            self.notification_priority
        } else {
            self.notification_plain
        }
    }

    pub(crate) fn notification_title_color(self, priority: bool) -> Color {
        if priority {
            Color::WHITE
        } else {
            self.text_dark
        }
    }

    pub(crate) fn notification_body_color(self, priority: bool) -> Color {
        if priority {
            Color::rgba(255, 255, 255, 218)
        } else {
            self.text_muted
        }
    }

    pub(crate) fn notification_icon_color(self, priority: bool) -> Color {
        if priority {
            Color::rgba(255, 255, 255, 232)
        } else {
            self.accent.with_alpha(224)
        }
    }

    pub(crate) fn app_icon_color(self) -> Color {
        self.accent.with_alpha(232)
    }
}
