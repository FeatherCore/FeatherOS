//! Picking System
//!
//! Provides pointer-based interaction with entities.
//!
//! # Architecture (Event-Driven)
//!
//! ```text
//! PointerInput → Backend → PointerHits → HoverMap → Pointer<E> Events
//! ```
//!
//! Inspired by Bevy's picking system.

mod pickable;
mod bounds;
mod hover;
mod backend;
mod pointer;
mod events;
mod system;
mod plugin;

pub use pickable::Pickable;
pub use bounds::PickableBounds;
pub use hover::{HoverMap, PreviousHoverMap};
pub use backend::{PointerHits, HitData};
pub use pointer::{PointerId, PointerButton, PointerPress, PointerLocation, PointerAction, PointerInput};
pub use events::{Pointer, Over, Out, Enter, Leave, Press, Release, Click, Move, DragStart, Drag, DragEnd};
pub use system::{update_hover_map, ui_picking_backend, pointer_events};
pub use plugin::{PickingPlugin, PointerHitsBuffer};

extern crate alloc;
