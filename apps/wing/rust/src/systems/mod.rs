//! Transitional ECS systems registered by the Wing desktop plugin.

mod input;
mod sync;
mod update;

pub use input::{
    wing_pointer_input_system,
    wing_shell_shortcut_system,
    wing_text_input_system,
};
pub use sync::{wing_shell_state_sync_system, wing_window_state_sync_system};
pub use update::wing_update_system;
