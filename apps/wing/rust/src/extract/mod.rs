//! FHRE-style extractors for the Wing shell.

mod primitives;
mod shell;
mod view;

pub use shell::{extract_wing_shell, ExtractedShellText};
pub use primitives::queue_wing_primitives;
pub use view::extract_view;
