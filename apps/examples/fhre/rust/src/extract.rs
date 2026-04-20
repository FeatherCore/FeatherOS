//! Custom Extractors for Demo
//!
//! These extractors follow Bevy's pattern:
//! 1. Extract Phase: Copy components from Main World to Render World ECS
//! 2. Queue Phase: Generate render commands from extracted components

use fhre::{
    MainWorld, RenderWorld, RenderCommand,
    Transform, Transform3D, Color, math::Rect,
    resources::{PrimaryScreen, Camera, ProjectionType},
    math::{Vec3, Mat4},
    render_world::{View, ViewBundle, ViewTarget, ClearConfig, ExtractedMesh, ExtractedUI},
};
use alloc::vec::Vec;
use alloc::vec;
use crate::components::{Button, Cube, SoccerBall};

// =============================================================================
// Extract Phase - Copy components from Main World to Render World
// =============================================================================

/// Extract view from Camera resource
pub fn extract_view(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let screen = main_world.resources().get::<PrimaryScreen>();
    let (width, height) = screen.map(|s| (s.width as f32, s.height as f32)).unwrap_or((800.0, 600.0));
    let canvas_pos = screen.map(|s| s.position()).unwrap_or(Vec3::new(width / 2.0, height / 2.0, 0.0));

    let view_bundle = if let Some(camera) = main_world.resources().get::<Camera>() {
        camera_to_view_bundle(&camera, canvas_pos, width, height)
    } else {
        create_perspective_view_with_canvas(canvas_pos, width, height)
    };
    let view_idx = render_world.add_view(view_bundle);
    render_world.set_current_view(Some(view_idx));
}

/// Extract 3D components (Cube, SoccerBall) to Render World ECS
pub fn extract_3d_components(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let view = match render_world.current_view() {
        Some(v) => v.view.clone(),
        None => return,
    };

    for (entity, transform) in main_world.query::<Transform>() {
        if let Some(cube) = main_world.get_component::<Cube>(entity) {
            let render_entity = render_world.get_or_spawn_synced(entity);
            
            let vertices = cube.get_vertices().to_vec();
            let faces: Vec<Vec<usize>> = cube.get_faces().iter().map(|f| f.to_vec()).collect();
            let face_colors = cube.face_colors.to_vec();
            let face_textures = cube.face_textures.to_vec();
            
            let mesh = ExtractedMesh {
                vertices,
                faces,
                face_colors,
                face_textures,
                position: transform.position,
                rotation: cube.rotation,
                wireframe: cube.wireframe,
                wireframe_color: cube.wireframe_color,
            };
            
            render_world.insert_component(render_entity, mesh);
            continue;
        }

        if let Some(soccer_ball) = main_world.get_component::<SoccerBall>(entity) {
            let render_entity = render_world.get_or_spawn_synced(entity);
            
            let mut faces: Vec<Vec<usize>> = Vec::new();
            for face in soccer_ball.get_hexagon_faces().iter() {
                faces.push(face.to_vec());
            }
            for face in soccer_ball.get_pentagon_faces().iter() {
                faces.push(face.to_vec());
            }
            
            let mut face_colors: Vec<Color> = Vec::new();
            for color in soccer_ball.hexagon_colors.iter() {
                face_colors.push(*color);
            }
            for color in soccer_ball.pentagon_colors.iter() {
                face_colors.push(*color);
            }
            
            let face_textures: Vec<Option<u32>> = vec![None; faces.len()];
            
            let mesh = ExtractedMesh {
                vertices: soccer_ball.get_vertices().to_vec(),
                faces,
                face_colors,
                face_textures,
                position: transform.position,
                rotation: soccer_ball.rotation,
                wireframe: soccer_ball.wireframe,
                wireframe_color: soccer_ball.wireframe_color,
            };
            
            render_world.insert_component(render_entity, mesh);
        }
    }
}

/// Extract 2D UI components (Button) to Render World ECS
pub fn extract_buttons(main_world: &MainWorld, render_world: &mut RenderWorld) {
    for (entity, transform) in main_world.query::<Transform>() {
        if let Some(button) = main_world.get_component::<Button>(entity) {
            let render_entity = render_world.get_or_spawn_synced(entity);
            
            let ui = ExtractedUI {
                position: fhre::math::Vec2::new(transform.position.x, transform.position.y),
                width: button.width,
                height: button.height,
                color: button.current_color(),
            };
            
            render_world.insert_component(render_entity, ui);
        }
    }
}

// =============================================================================
// Queue Phase - Generate render commands from extracted components
// =============================================================================

/// Queue render commands from extracted 3D meshes
pub fn queue_meshes(_main_world: &MainWorld, render_world: &mut RenderWorld) {
    let view = match render_world.current_view() {
        Some(v) => v.view.clone(),
        None => return,
    };
    
    let meshes: Vec<ExtractedMesh> = render_world.query::<ExtractedMesh>()
        .map(|(_, mesh)| mesh.clone())
        .collect();
    
    for mesh in meshes {
        let commands = generate_mesh_commands(&mesh, &view);
        for cmd in commands {
            render_world.add_command(cmd);
        }
    }
}

/// Queue render commands from extracted UI
pub fn queue_ui(_main_world: &MainWorld, render_world: &mut RenderWorld) {
    let uis: Vec<ExtractedUI> = render_world.query::<ExtractedUI>()
        .map(|(_, ui)| ui.clone())
        .collect();
    
    for ui in uis {
        let rect = Rect::from_center_size(
            fhre::math::Vec2::new(ui.position.x, ui.position.y),
            fhre::math::Vec2::new(ui.width, ui.height)
        );
        render_world.add_command(RenderCommand::DrawRect { rect, color: ui.color });
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn create_perspective_view_with_canvas(canvas_pos: Vec3, width: f32, height: f32) -> ViewBundle {
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

fn camera_to_view_bundle(camera: &Camera, canvas_pos: Vec3, width: f32, height: f32) -> ViewBundle {
    let viewport = Rect::new(0.0, 0.0, width, height);
    let projection = camera.projection.build_projection_matrix(width, height);
    let view = Mat4::look_at_rh(camera.position, canvas_pos, camera.up);
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

fn generate_mesh_commands(mesh: &ExtractedMesh, view: &View) -> Vec<RenderCommand> {
    use fhre::math::Vec2;
    use alloc::vec;
    
    let mut commands = Vec::new();

    let rot_x = Mat4::from_rotation_x(mesh.rotation.x.to_radians());
    let rot_y = Mat4::from_rotation_y(mesh.rotation.y.to_radians());
    let rot_z = Mat4::from_rotation_z(mesh.rotation.z.to_radians());
    let rotation = rot_z.mul(&rot_y).mul(&rot_x);

    let mut world_vertices: Vec<Vec3> = Vec::with_capacity(mesh.vertices.len());
    for v in &mesh.vertices {
        let rotated = rotation.mul_vec3(*v);
        let world_pos = rotated + mesh.position;
        world_vertices.push(world_pos);
    }

    let mut screen_vertices: Vec<Vec2> = Vec::with_capacity(mesh.vertices.len());
    let mut view_z: Vec<f32> = Vec::with_capacity(mesh.vertices.len());
    
    for world_pos in &world_vertices {
        let view_pos = view.view.mul_vec3(*world_pos);
        view_z.push(view_pos.z);
        
        if let Some((x, y)) = view.world_to_screen(*world_pos) {
            screen_vertices.push(Vec2::new(x, y));
        } else {
            screen_vertices.clear();
            break;
        }
    }

    if screen_vertices.len() != mesh.vertices.len() {
        return commands;
    }

    let mut visible_faces: Vec<(usize, f32, Vec<Vec2>)> = Vec::new();
    
    for (face_idx, face) in mesh.faces.iter().enumerate() {
        if face.len() < 3 {
            continue;
        }
        
        let screen_face: Vec<Vec2> = face.iter()
            .filter_map(|&i| screen_vertices.get(i).copied())
            .collect();
        
        if screen_face.len() != face.len() {
            continue;
        }
        
        let avg_view_z = face.iter()
            .filter_map(|&i| view_z.get(i))
            .sum::<f32>() / face.len() as f32;
        
        visible_faces.push((face_idx, avg_view_z, screen_face));
    }

    visible_faces.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    for (face_idx, _, screen_face) in visible_faces {
        let color = mesh.face_colors.get(face_idx).copied().unwrap_or(Color::WHITE);
        let texture_id = mesh.face_textures.get(face_idx).copied().flatten();

        if let Some(tex_id) = texture_id {
            let uvs: Vec<Vec2> = match screen_face.len() {
                4 => vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(1.0, 1.0),
                    Vec2::new(0.0, 1.0),
                ],
                3 => vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(0.5, 1.0),
                ],
                _ => screen_face.iter().enumerate().map(|(i, _)| {
                    let angle = i as f32 / screen_face.len() as f32 * 6.28318;
                    Vec2::new(0.5 + 0.5 * libm::cosf(angle), 0.5 + 0.5 * libm::sinf(angle))
                }).collect(),
            };
            
            commands.push(RenderCommand::DrawPolygonTextured {
                vertices: screen_face.clone(),
                uvs,
                texture_id: tex_id,
                color,
            });
        } else {
            commands.push(RenderCommand::DrawPolygon {
                vertices: screen_face.clone(),
                color,
            });
        }

        if mesh.wireframe && screen_face.len() >= 2 {
            let wf_color = mesh.wireframe_color;
            for i in 0..screen_face.len() {
                let start = screen_face[i];
                let end = screen_face[(i + 1) % screen_face.len()];
                commands.push(RenderCommand::DrawLine {
                    start,
                    end,
                    color: wf_color,
                    thickness: 1.0,
                });
            }
        }
    }

    commands
}
