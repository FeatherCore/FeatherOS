//! ECS systems for Wing shell and legacy migration code.

mod pointer;
mod picking;
mod shell;

pub use shell::{setup_wing_shell, wing_notification_text_layout_system, wing_shell_interaction_system, wing_shell_layout_system, wing_shell_overlay_layout_system, wing_shell_stack_layout_system};
pub use pointer::wing_minimal_button_interaction_system;
pub use picking::wing_picking_system;
