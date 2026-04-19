//! Pickable Component

use crate::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Pickable {
    pub should_block_lower: bool,
    pub is_hoverable: bool,
}

impl Pickable {
    pub const DEFAULT: Self = Self { should_block_lower: true, is_hoverable: true };
    pub const IGNORE: Self = Self { should_block_lower: false, is_hoverable: false };
    pub const BLOCK: Self = Self { should_block_lower: true, is_hoverable: false };
}

impl Component for Pickable {
    fn type_name() -> &'static str { "Pickable" }
}
