//! ECS systems for Wing shell.

mod animation;
mod gesture;
mod pointer;
mod picking;
mod shell;

pub use animation::wing_overlay_animation_system;
pub use gesture::wing_gesture_system;
pub use shell::{
    setup_wing_shell, 
    wing_shell_interaction_system, 
    wing_shell_layout_system, 
    wing_shell_overlay_layout_system,
    wing_shell_notification_panel_layout_system,
    wing_shell_quick_controls_layout_system,
    wing_shell_notification_cards_layout_system,
    wing_shell_stack_layout_system,
    wing_shell_overlay_card_layout_system,
};
pub use pointer::wing_minimal_button_interaction_system;
pub use picking::wing_picking_system;
