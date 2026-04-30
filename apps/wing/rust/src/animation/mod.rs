use fhre::Tween;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Animation {
    pub opacity: Option<Tween>,
}

impl Animation {
    pub const fn none() -> Self {
        Self { opacity: None }
    }

    pub const fn opacity(opacity: Tween) -> Self {
        Self {
            opacity: Some(opacity),
        }
    }
}
