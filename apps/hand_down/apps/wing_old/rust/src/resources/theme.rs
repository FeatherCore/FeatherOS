use crate::WingTheme;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeVariant {
    Aurora,
    Dusk,
}

impl ThemeVariant {
    pub const fn next(&self) -> Self {
        match self {
            ThemeVariant::Aurora => ThemeVariant::Dusk,
            ThemeVariant::Dusk => ThemeVariant::Aurora,
        }
    }

    pub const fn to_theme(&self) -> WingTheme {
        match self {
            ThemeVariant::Aurora => WingTheme::aurora(),
            ThemeVariant::Dusk => WingTheme::dusk(),
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            ThemeVariant::Aurora => "Aurora",
            ThemeVariant::Dusk => "Dusk",
        }
    }
}

pub struct ThemeState {
    pub current: WingTheme,
    pub previous: WingTheme,
    pub variant: ThemeVariant,
    pub pending_variant: Option<ThemeVariant>,
    pub is_transitioning: bool,
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            current: WingTheme::default(),
            previous: WingTheme::default(),
            variant: ThemeVariant::Aurora,
            pending_variant: None,
            is_transitioning: false,
        }
    }
}

impl ThemeState {
    pub fn switch_next_theme(&mut self) {
        if self.is_transitioning {
            return;
        }
        self.pending_variant = Some(self.variant.next());
        self.previous = self.current;
        self.is_transitioning = true;
    }

    pub fn apply_pending_theme(&mut self) {
        if let Some(new_variant) = self.pending_variant.take() {
            self.variant = new_variant;
            self.current = self.variant.to_theme();
        }
        self.is_transitioning = false;
    }

    pub fn transition_progress(&self) -> f32 {
        if !self.is_transitioning {
            1.0
        } else {
            0.0
        }
    }

    pub fn current_theme_name(&self) -> &'static str {
        self.variant.name()
    }
}

impl fhre::resources::Resource for ThemeState {}
