//! Extract System Implementation
//!
//! The Extract phase syncs data from Main World to Render World.
//!
//! # Architecture
//!
//! FHRE uses a single 3D perspective camera. The rendering pipeline:
//! 1. Camera captures the 3D scene via MVP transformation
//! 2. 3D objects are projected onto the 2D screen canvas (幕布)
//! 3. 2D UI elements are drawn directly in screen canvas pixel coordinates
//! 4. Both share the same screen canvas coordinate system (origin top-left, Y-down)
//! 5. The screen canvas content is then presented to the user via a presentation window

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::render_world::RenderObject;
use crate::render_world::RenderCommand;
use crate::render_world::{ViewBundle, View, ClearConfig, ViewTarget};
use crate::main_world::{Transform, Sprite};
use crate::node::Transform2D;
use crate::node::Transform3D;
use crate::ui::{Button, Cube, Dodecahedron, SoccerBall};
use crate::resources::{Time, PrimaryScreen, Camera, ProjectionType};
use crate::math::{Vec2, Vec3, Mat4, Color, Rect};
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Extract trait - Defines extract operations
pub trait Extract {
    fn extract(&self, main_world: &MainWorld, render_world: &mut RenderWorld);
}

/// ExtractSchedule - Collection of extract operations
pub struct ExtractSchedule {
    extractors: Vec<Box<dyn Fn(&MainWorld, &mut RenderWorld)>>,
}

impl ExtractSchedule {
    pub fn new() -> Self {
        Self {
            extractors: Vec::new(),
        }
    }

    pub fn add_extractor<F>(&mut self, extractor: F)
    where
        F: Fn(&MainWorld, &mut RenderWorld) + 'static,
    {
        self.extractors.push(Box::new(extractor));
    }

    pub fn run(&self, main_world: &MainWorld, render_world: &mut RenderWorld) {
        render_world.clear_objects();
        for extractor in &self.extractors {
            extractor(main_world, render_world);
        }
    }
}

/// Extract all renderable components from Main World to Render World
/// 
/// Single 3D camera architecture:
/// 1. Create the 3D perspective View from the Camera resource
///    - Camera target automatically tracks the screen canvas position
/// 2. 3D objects (Cube, SoccerBall) are projected via Camera MVP → screen canvas coordinates
/// 3. 2D UI (Button) is drawn directly in screen canvas pixel coordinates
/// 4. Both share the same screen canvas coordinate system
pub fn extract_renderable_components(main_world: &MainWorld, render_world: &mut RenderWorld) {
    use crate::render_world::RenderComponent;
    use crate::node::Transform3D;
    
    unsafe {
        extern "C" {
            fn printf(format: *const u8, ...) -> i32;
        }
        printf(b"[EXTRACT] Starting extract_renderable_components (single camera)\n\0".as_ptr());
    }
    
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
    
    let mut has_3d_renderables = false;
    let mut transform3d_count = 0;
    for (entity, _transform) in main_world.query::<Transform3D>() {
        transform3d_count += 1;
        if main_world.get_component::<Cube>(entity).is_some() 
            || main_world.get_component::<SoccerBall>(entity).is_some() {
            has_3d_renderables = true;
        }
    }
    
    unsafe {
        extern "C" {
            fn printf(format: *const u8, ...) -> i32;
        }
        printf(b"[EXTRACT] Found %d Transform3D components, has_3d_renderables=%d\n\0".as_ptr(),
               transform3d_count, has_3d_renderables as i32);
    }
    
    if has_3d_renderables {
        let view_clone = render_world.current_view().map(|v| v.view.clone());
        let mut cube_count = 0;
        if let Some(ref view) = view_clone {
            for (entity, transform) in main_world.query::<Transform3D>() {
                if let Some(cube) = main_world.get_component::<Cube>(entity) {
                    let commands = cube.generate_render_commands(transform, view);
                    let cmd_count = commands.len();
                    for command in commands {
                        render_world.add_command(command);
                    }
                    cube_count += 1;
                    unsafe {
                        extern "C" {
                            fn printf(format: *const u8, ...) -> i32;
                        }
                        printf(b"[EXTRACT] Cube %d: generated %d commands at pos (%f, %f, %f)\n\0".as_ptr(),
                               cube_count, cmd_count as i32,
                               transform.position.x as f64,
                               transform.position.y as f64,
                               transform.position.z as f64);
                    }
                }
            }
        }
        
        unsafe {
            extern "C" {
                fn printf(format: *const u8, ...) -> i32;
            }
            printf(b"[EXTRACT] Total cubes extracted: %d\n\0".as_ptr(), cube_count);
        }
        
        let view_clone = render_world.current_view().map(|v| v.view.clone());
        if let Some(ref view) = view_clone {
            for (entity, transform) in main_world.query::<Transform3D>() {
                if let Some(soccer_ball) = main_world.get_component::<SoccerBall>(entity) {
                    let commands = soccer_ball.generate_render_commands(transform, view);
                    for command in commands {
                        render_world.add_command(command);
                    }
                }
            }
        }
    }
    
    extract_buttons(main_world, render_world);
}

/// Extract sprites from Main World to Render World
///
/// Sprites are drawn directly in screen canvas pixel coordinates,
/// sharing the same coordinate system as the 3D camera projection output.
pub fn extract_sprites(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let transforms: Vec<_> = main_world.query::<Transform>().collect();

    for (entity, transform) in transforms {
        if let Some(sprite) = main_world.get_component::<Sprite>(entity) {
            let render_obj = RenderObject::new()
                .with_position(transform.position.x, transform.position.y, transform.position.z)
                .with_size(sprite.width, sprite.height)
                .with_color(sprite.color);

            render_world.add_object(render_obj);
        }
    }
}

/// Extract buttons from Main World to Render World
///
/// Buttons are drawn directly in screen canvas pixel coordinates.
/// No separate orthographic view is needed — they share the same
/// screen canvas coordinate system as the 3D camera projection output.
pub fn extract_buttons(main_world: &MainWorld, render_world: &mut RenderWorld) {
    unsafe {
        extern "C" {
            fn printf(format: *const u8, ...) -> i32;
        }
        printf(b"[EXTRACT] Starting extract_buttons\n\0".as_ptr());
    }

    let transforms: Vec<_> = main_world.query::<Transform2D>().collect();
    
    unsafe {
        extern "C" {
            fn printf(format: *const u8, ...) -> i32;
        }
        printf(b"[EXTRACT] Found %d Transform2D components\n\0".as_ptr(), transforms.len() as i32);
    }

    let mut button_count = 0;
    for (entity, transform) in transforms {
        if let Some(button) = main_world.get_component::<Button>(entity) {
            button_count += 1;
            unsafe {
                extern "C" {
                    fn printf(format: *const u8, ...) -> i32;
                }
                printf(b"[EXTRACT] Button %d at pos (%f, %f), size %fx%f\n\0".as_ptr(),
                       button_count,
                       transform.position.x as f64,
                       transform.position.y as f64,
                       button.width as f64,
                       button.height as f64);
            }
            let color = button.current_color();

            let rect = Rect::new(
                transform.position.x,
                transform.position.y,
                button.width,
                button.height,
            );

            render_world.add_command(RenderCommand::DrawRect { rect, color });

            let text_rect = Rect::new(
                transform.position.x + button.width * 0.2,
                transform.position.y + button.height * 0.3,
                button.width * 0.6,
                button.height * 0.4,
            );
            render_world.add_command(RenderCommand::DrawRect {
                rect: text_rect,
                color: Color::WHITE,
            });
        }
    }
    
    unsafe {
        extern "C" {
            fn printf(format: *const u8, ...) -> i32;
        }
        printf(b"[EXTRACT] Total buttons extracted: %d\n\0".as_ptr(), button_count);
    }
}

/// Extract cubes - 3D rendering with perspective projection
/// 
/// 使用画家算法（Painter's Algorithm）：按深度排序，先画远的面，后画近的面
/// 同时做背面剔除（Backface Culling）：只绘制朝向相机的面
pub fn extract_cubes(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // Query all entities with Transform3D and Cube
    let transforms: Vec<_> = main_world.query::<Transform3D>().collect();

    for (entity, transform) in transforms {
        if let Some(cube) = main_world.get_component::<Cube>(entity) {
            // Get current view for 3D projection
            let view = match render_world.current_view() {
                Some(v) => v,
                None => continue,
            };

            // Get cube vertices in local space
            let vertices = cube.get_vertices();
            let faces = cube.get_faces();

            // Build rotation matrix from euler angles (in degrees)
            let rot_x = Mat4::from_rotation_x(cube.rotation.x.to_radians());
            let rot_y = Mat4::from_rotation_y(cube.rotation.y.to_radians());
            let rot_z = Mat4::from_rotation_z(cube.rotation.z.to_radians());
            let rotation = rot_z.mul(&rot_y).mul(&rot_x);

            // Transform vertices to world space
            let mut world_vertices: Vec<Vec3> = Vec::with_capacity(8);
            for v in &vertices {
                let rotated = rotation.mul_vec3(*v);
                let world_pos = rotated + transform.position;
                world_vertices.push(world_pos);
            }

            // Project vertices to screen space and calculate view-space Z
            let mut screen_vertices: Vec<Vec2> = Vec::with_capacity(8);
            let mut view_z: Vec<f32> = Vec::with_capacity(8);
            
            for world_pos in &world_vertices {
                // Transform to view space (camera-relative) using view matrix
                let view_pos = view.view.view.mul_vec3(*world_pos);
                view_z.push(view_pos.z);
                
                if let Some((x, y)) = view.view.world_to_screen(*world_pos) {
                    screen_vertices.push(Vec2::new(x, y));
                } else {
                    // Vertex behind camera, skip this cube
                    screen_vertices.clear();
                    break;
                }
            }

            if screen_vertices.len() != 8 {
                continue;
            }

            // 收集所有可见的面及其深度
            let mut visible_faces: Vec<(usize, f32, [Vec2; 4])> = Vec::new();

            for (face_idx, face) in faces.iter().enumerate() {
                // 获取面的四个顶点（屏幕坐标）
                let v0 = screen_vertices[face[0]];
                let v1 = screen_vertices[face[1]];
                let v2 = screen_vertices[face[2]];
                let v3 = screen_vertices[face[3]];

                // 背面剔除：计算面的法向量（在屏幕空间）
                // 使用叉积判断面的朝向
                let edge1 = v1 - v0;
                let edge2 = v3 - v0;
                let _cross_z = edge1.x * edge2.y - edge1.y * edge2.x;

                // 背面剔除：不启用
                // 原因：前面的面可能带透明度，需要绘制背面才能正确显示
                // 画家算法（按深度排序）已足够处理遮挡关系
                if true {
                    // 计算面的平均 view-space Z（用于排序）
                    // view-space Z 越小表示越近（相机在 +Z 看向 -Z）
                    let avg_view_z = (view_z[face[0]] 
                                    + view_z[face[1]] 
                                    + view_z[face[2]] 
                                    + view_z[face[3]]) / 4.0;
                    
                    visible_faces.push((face_idx, avg_view_z, [v0, v1, v2, v3]));
                }
            }

            // 按 view-space Z 排序（远的先画，Z 值大的先画）
            // 在右手坐标系 view space 中，相机看向 -Z
            // Z 值越小（越负）表示越近，Z 值越大（越接近 0）表示越远
            // 画家算法：先画远的（Z 值大的），后画近的（Z 值小的）
            visible_faces.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            // 绘制可见的面（从远到近）
            for (face_idx, _, [v0, v1, v2, v3]) in visible_faces {
                let color = cube.face_colors[face_idx];

                // Triangle 1: v0, v1, v2
                render_world.add_command(RenderCommand::DrawTriangle {
                    p0: v0,
                    p1: v1,
                    p2: v2,
                    color,
                });
                // Triangle 2: v0, v2, v3
                render_world.add_command(RenderCommand::DrawTriangle {
                    p0: v0,
                    p1: v2,
                    p2: v3,
                    color,
                });

                // Draw wireframe if enabled
                if cube.wireframe {
                    let wf_color = cube.wireframe_color;
                    let thickness = 1.0;

                    render_world.add_command(RenderCommand::DrawLine {
                        start: v0,
                        end: v1,
                        color: wf_color,
                        thickness,
                    });
                    render_world.add_command(RenderCommand::DrawLine {
                        start: v1,
                        end: v2,
                        color: wf_color,
                        thickness,
                    });
                    render_world.add_command(RenderCommand::DrawLine {
                        start: v2,
                        end: v3,
                        color: wf_color,
                        thickness,
                    });
                    render_world.add_command(RenderCommand::DrawLine {
                        start: v3,
                        end: v0,
                        color: wf_color,
                        thickness,
                    });
                }
            }
        }
    }
}

// Note: SoccerBall rendering is now handled by extract_renderable_components
// through the RenderComponent trait. The extract_soccer_balls function has been
// removed as it's no longer needed.

/// Extract time resource
pub fn extract_time(main_world: &MainWorld, _render_world: &mut RenderWorld) {
    if let Some(time) = main_world.resources().get::<Time>() {
        let _elapsed = time.elapsed();
    }
}

/// Create default extract schedule
pub fn default_extract_schedule() -> ExtractSchedule {
    let mut schedule = ExtractSchedule::new();
    schedule.add_extractor(extract_sprites);
    schedule.add_extractor(extract_buttons);
    schedule.add_extractor(extract_time);
    schedule
}

/// Extract parameters
pub struct ExtractParams<'a> {
    pub main_world: &'a MainWorld,
    pub render_world: &'a mut RenderWorld,
}

impl<'a> ExtractParams<'a> {
    pub fn new(main_world: &'a MainWorld, render_world: &'a mut RenderWorld) -> Self {
        Self { main_world, render_world }
    }
}

/// Create perspective view for 3D rendering (fallback when no Camera resource)
///
/// Uses the screen canvas position as the camera's look-at target.
fn create_perspective_view_with_canvas(canvas_pos: Vec3, width: f32, height: f32) -> ViewBundle {
    let viewport = Rect::new(0.0, 0.0, width, height);
    let projection = Mat4::perspective_rh(
        45.0_f32.to_radians(),
        width / height,
        0.1,
        1000.0,
    );

    let camera_pos = Vec3::new(canvas_pos.x, canvas_pos.y, canvas_pos.z + 600.0);
    let view = Mat4::look_at_rh(
        camera_pos,
        canvas_pos,
        Vec3::new(0.0, 1.0, 0.0),
    );
    let vp_matrix = projection * view;

    let view_struct = View {
        projection,
        view,
        view_projection: vp_matrix,
        camera_position: camera_pos,
        near: 0.1,
        far: 1000.0,
        orthographic: false,
        viewport,
    };

    ViewBundle {
        view: view_struct,
        target: ViewTarget::Screen,
        clear: ClearConfig::color(Color::BLACK),
    }
}

/// Build ViewBundle from Camera Resource (declarative)
///
/// Reads position/fov/projection from user-configured Camera Resource.
/// The camera's look-at target is overridden to track the screen canvas position,
/// ensuring the camera always points at the canvas regardless of canvas movement.
fn camera_to_view_bundle(camera: &Camera, canvas_pos: Vec3, width: f32, height: f32) -> ViewBundle {
    let viewport = Rect::new(0.0, 0.0, width, height);
    let projection = camera.projection.build_projection_matrix(width, height);
    let view = Mat4::look_at_rh(camera.position, canvas_pos, camera.up);
    let vp_matrix = projection * view;

    let (near, far) = match camera.projection {
        ProjectionType::Perspective { near, far, .. } => (near, far),
    };

    let view_struct = View {
        projection,
        view,
        view_projection: vp_matrix,
        camera_position: camera.position,
        near,
        far,
        orthographic: false,
        viewport,
    };

    ViewBundle {
        view: view_struct,
        target: ViewTarget::Screen,
        clear: ClearConfig::color(Color::BLACK),
    }
}


