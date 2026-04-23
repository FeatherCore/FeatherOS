use fhre::{
    resources::{Camera, PrimaryScreen, ProjectionType},
    render_world::{ClearConfig, View, ViewBundle, ViewTarget},
    Color, MainWorld, Mat4, RenderWorld, Vec3,
};

pub fn extract_view(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let screen = main_world.resources().get::<PrimaryScreen>();
    let (width, height) = screen
        .map(|s| (s.width as f32, s.height as f32))
        .unwrap_or((800.0, 600.0));
    let canvas_pos = screen
        .map(|s| s.position())
        .unwrap_or(Vec3::new(width * 0.5, height * 0.5, 0.0));

    let view_bundle = if let Some(camera) = main_world.resources().get::<Camera>() {
        camera_to_view_bundle(camera, canvas_pos, width, height)
    } else {
        create_perspective_view_with_canvas(canvas_pos, width, height)
    };

    let view_idx = render_world.add_view(view_bundle);
    render_world.set_current_view(Some(view_idx));
}

fn create_perspective_view_with_canvas(canvas_pos: Vec3, width: f32, height: f32) -> ViewBundle {
    let viewport = fhre::math::Rect::new(0.0, 0.0, width, height);
    let projection = Mat4::perspective_rh(45.0_f32.to_radians(), width / height, 0.1, 1000.0);
    let camera_pos = Vec3::new(canvas_pos.x, canvas_pos.y, canvas_pos.z + 600.0);
    let view = Mat4::look_at_rh(camera_pos, canvas_pos, Vec3::new(0.0, 1.0, 0.0));

    ViewBundle {
        view: View {
            projection,
            view,
            view_projection: projection * view,
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

fn camera_to_view_bundle(camera: &Camera, canvas_pos: Vec3, width: f32, height: f32) -> ViewBundle {
    let viewport = fhre::math::Rect::new(0.0, 0.0, width, height);
    let projection = camera.projection.build_projection_matrix(width, height);
    let view = Mat4::look_at_rh(camera.position, canvas_pos, camera.up);
    let (near, far) = match camera.projection {
        ProjectionType::Perspective { near, far, .. } => (near, far),
    };

    ViewBundle {
        view: View {
            projection,
            view,
            view_projection: projection * view,
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
