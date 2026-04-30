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

use crate::{App, Plugin, MainWorld, RenderWorld};
use crate::resources::{Camera, PrimaryScreen, ProjectionType};
use crate::math::{Vec3, Mat4, Rect};
use crate::render_world::{View, ViewBundle, ViewTarget, ClearConfig};
use crate::math::Color;

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

        app.add_extractor(extract_view);
    }
}

/// Extract view from Camera and PrimaryScreen resources
///
/// This is automatically registered by CameraPlugin.
/// Creates a ViewBundle from the Camera and PrimaryScreen resources.
pub fn extract_view(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let screen = main_world.resources().get::<PrimaryScreen>();
    let (width, height) = screen
        .map(|s| (s.width as f32, s.height as f32))
        .unwrap_or((800.0, 600.0));
    let canvas_pos = screen
        .map(|s| s.position())
        .unwrap_or(Vec3::new(width / 2.0, height / 2.0, 0.0));

    let view_bundle = if let Some(camera) = main_world.resources().get::<Camera>() {
        camera_to_view_bundle(&camera, canvas_pos, width, height)
    } else {
        create_default_view(canvas_pos, width, height)
    };

    let view_idx = render_world.add_view(view_bundle);
    render_world.set_current_view(Some(view_idx));
}

/// Convert Camera resource to ViewBundle
fn camera_to_view_bundle(camera: &Camera, canvas_pos: Vec3, width: f32, height: f32) -> ViewBundle {
    let viewport = Rect::new(0.0, 0.0, width, height);
    let projection = camera.projection.build_projection_matrix(width, height);
    let view = Mat4::look_at_rh(camera.position, camera.target, camera.up);
    let vp_matrix = projection * view;

    let (near, far) = match camera.projection {
        ProjectionType::Perspective { near, far, .. } => (near, far),
    };

    ViewBundle {
        view: View {
            projection,
            view,
            view_projection: vp_matrix,
            camera_position: camera.position,
            near,
            far,
            orthographic: false,
            viewport,
        },
        target: ViewTarget::Screen,
        clear: ClearConfig::color(Color::BLACK),
    }
}

/// Create default view when no Camera resource exists
fn create_default_view(canvas_pos: Vec3, width: f32, height: f32) -> ViewBundle {
    let viewport = Rect::new(0.0, 0.0, width, height);
    let projection = Mat4::perspective_rh(45.0_f32.to_radians(), width / height, 0.1, 1000.0);
    let camera_pos = Vec3::new(canvas_pos.x, canvas_pos.y, canvas_pos.z + 600.0);
    let view = Mat4::look_at_rh(camera_pos, canvas_pos, Vec3::new(0.0, 1.0, 0.0));
    let vp_matrix = projection * view;

    ViewBundle {
        view: View {
            projection,
            view,
            view_projection: vp_matrix,
            camera_position: camera_pos,
            near: 0.1,
            far: 1000.0,
            orthographic: false,
            viewport,
        },
        target: ViewTarget::Screen,
        clear: ClearConfig::color(Color::BLACK),
    }
}
