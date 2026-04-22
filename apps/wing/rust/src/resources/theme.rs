use crate::WingTheme;

/// Theme resource decoupled from the transitional runtime object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeState {
    pub current: WingTheme,
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            current: WingTheme::default(),
        }
    }
}

impl fhre::resources::Resource for ThemeState {}
