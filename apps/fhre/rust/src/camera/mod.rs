//! Camera Plugin for FHRE
//!
//! Registers the default 3D perspective camera as a Resource.
//!
//! # Architecture
//!
//! FHRE uses a single 3D perspective camera. The camera captures the 3D scene
//! and projects it onto a 2D screen canvas (幕布). Both 3D objects and 2D UI
//! elements are rendered onto this same screen canvas.
//!
//! The camera's look-at target defaults to the screen canvas position,
//! ensuring the canvas fills the presentation window 1:1 by default.
//!
//! # Usage
//! ```rust
//! App::new(640, 480)
//!     .add_plugins(CameraPlugin)  // registers default Camera aimed at screen canvas
//!     // Override default if needed:
//!     .insert_resource(Camera::perspective_3d(
//!         Vec3::new(320.0, 160.0, 500.0),
//!         Vec3::new(320.0, 160.0, 0.0),
//!         60.0,
//!     ))
//!     .run();
//! ```

use crate::{App, Plugin};
use crate::resources::{Camera, PrimaryScreen};
use crate::math::Vec3;

/// Plugin that registers the default 3D camera
///
/// Automatically inserts:
/// - **`Camera`** — 3D perspective camera aimed at the screen canvas center
///
/// The camera position defaults to (canvas_center.x, canvas_center.y, 600),
/// looking at the canvas center position. This ensures the screen canvas
/// fills the presentation window 1:1 by default.
///
/// Users can override by calling `insert_resource()` after this plugin.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        let screen = app.main_world.resources().get::<PrimaryScreen>();
        let (width, height) = screen
            .map(|s| s.dimensions())
            .unwrap_or((800, 600));

        if app.main_world.resources().get::<Camera>().is_none() {
            let canvas_pos = screen
                .map(|s| s.position())
                .unwrap_or(Vec3::new(width as f32 / 2.0, height as f32 / 2.0, 0.0));

            app.insert_resource(Camera::perspective_3d(
                Vec3::new(canvas_pos.x, canvas_pos.y, canvas_pos.z + 600.0),
                canvas_pos,
                45.0,
            ));
        }
    }
}
