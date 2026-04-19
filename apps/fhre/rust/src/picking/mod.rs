//! Picking System
//!
//! Provides pointer-based interaction with entities.

mod pickable;
mod bounds;
mod hover;
mod backend;
mod system;
mod plugin;

pub use pickable::Pickable;
pub use bounds::PickableBounds;
pub use hover::{HoverMap, PreviousHoverMap, PointerId};
pub use backend::{PointerHits, HitData};
pub use system::{update_hover_map, ui_picking_backend};
pub use plugin::{PickingPlugin, PointerHitsBuffer};

extern crate alloc;
