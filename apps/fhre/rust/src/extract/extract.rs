//! Extract System Implementation
//!
//! The Extract phase syncs data from Main World to Render World.

use crate::main_world::MainWorld;
use crate::render_world::RenderWorld;
use crate::render_world::RenderObject;
use crate::render_world::RenderCommand;
use crate::render_world::{ViewBundle, View, ClearConfig, ViewTarget};
use crate::main_world::{Transform, Sprite};
use crate::node::Transform2D;
use crate::node::Transform3D;
use crate::ui::{Button, Cube, Dodecahedron, SoccerBall};
use crate::resources::{Time, PrimaryScreen};
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

/// Extract sprites from Main World to Render World
pub fn extract_sprites(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // Get PrimaryScreen for default view
    let screen = main_world.resources().get::<PrimaryScreen>();
    let (width, height) = screen.map(|s| (s.width as f32, s.height as f32)).unwrap_or((800.0, 600.0));

    // Create default orthographic view
    let view_bundle = create_default_view(width, height);
    let view_idx = render_world.add_view(view_bundle);
    render_world.set_current_view(Some(view_idx));

    // Query all entities with Transform and Sprite
    let transforms: Vec<_> = main_world.query::<Transform>().collect();

    for (entity, transform) in transforms {
        if let Some(sprite) = main_world.get_component::<Sprite>(entity) {
            // Create render object
            let render_obj = RenderObject::new()
                .with_position(transform.position.x, transform.position.y, transform.position.z)
                .with_size(sprite.width, sprite.height)
                .with_color(sprite.color);

            render_world.add_object(render_obj);
        }
    }
}

/// Extract buttons from Main World to Render World
pub fn extract_buttons(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // Get PrimaryScreen for default view
    let screen = main_world.resources().get::<PrimaryScreen>();
    let (width, height) = screen.map(|s| (s.width as f32, s.height as f32)).unwrap_or((800.0, 600.0));

    // Create default orthographic view
    let view_bundle = create_default_view(width, height);
    let view_idx = render_world.add_view(view_bundle);
    render_world.set_current_view(Some(view_idx));

    // Query all entities with Transform2D and Button
    let transforms: Vec<_> = main_world.query::<Transform2D>().collect();

    for (entity, transform) in transforms {
        if let Some(button) = main_world.get_component::<Button>(entity) {
            // Get button color based on state
            let color = button.current_color();

            // Create render command for button background
            let rect = Rect::new(
                transform.position.x,
                transform.position.y,
                button.width,
                button.height,
            );

            render_world.add_command(RenderCommand::DrawRect { rect, color });

            // Draw button text (simplified as a smaller rect for now)
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
                let cross_z = edge1.x * edge2.y - edge1.y * edge2.x;

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

            // 按 view-space Z 排序（远的先画，Z 值小的先画）
            // 在右手坐标系 view space 中，相机看向 -Z
            // Z 值越小（越负）表示越近，Z 值越大（越接近 0）表示越远
            // 画家算法：先画远的（Z 值大的），后画近的（Z 值小的）
            visible_faces.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

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

/// Extract soccer ball components (truncated icosahedron)
/// 
/// 足球（截角二十面体）有32个面：
/// - 12个五边形面，每个分解为3个三角形
/// - 20个六边形面，每个分解为4个三角形
pub fn extract_soccer_balls(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // Query all entities with Transform3D and SoccerBall
    let transforms: Vec<_> = main_world.query::<Transform3D>().collect();

    for (entity, transform) in transforms {
        if let Some(soccer_ball) = main_world.get_component::<SoccerBall>(entity) {
            // Get current view for 3D projection
            let view = match render_world.current_view() {
                Some(v) => v,
                None => continue,
            };

            // Get soccer ball vertices in local space (60 vertices)
            let vertices = soccer_ball.get_vertices();
            let pentagons = soccer_ball.get_pentagon_faces();
            let hexagons = soccer_ball.get_hexagon_faces();

            // DEBUG: Print first few vertices
            unsafe {
                extern "C" {
                    fn printf(format: *const u8, ...);
                }
                printf(b"[DEBUG] SoccerBall vertices[0]: (%f, %f, %f)\n\0".as_ptr(),
                    vertices[0].x as f64, vertices[0].y as f64, vertices[0].z as f64);
                printf(b"[DEBUG] SoccerBall vertices[1]: (%f, %f, %f)\n\0".as_ptr(),
                    vertices[1].x as f64, vertices[1].y as f64, vertices[1].z as f64);
                printf(b"[DEBUG] SoccerBall pentagon[0]: [%d, %d, %d, %d, %d]\n\0".as_ptr(),
                    pentagons[0][0], pentagons[0][1], pentagons[0][2], pentagons[0][3], pentagons[0][4]);
                printf(b"[DEBUG] SoccerBall hexagon[0]: [%d, %d, %d, %d, %d, %d]\n\0".as_ptr(),
                    hexagons[0][0], hexagons[0][1], hexagons[0][2], hexagons[0][3], hexagons[0][4], hexagons[0][5]);
            }

            // Build rotation matrix from euler angles (in degrees)
            let rot_x = Mat4::from_rotation_x(soccer_ball.rotation.x.to_radians());
            let rot_y = Mat4::from_rotation_y(soccer_ball.rotation.y.to_radians());
            let rot_z = Mat4::from_rotation_z(soccer_ball.rotation.z.to_radians());
            let rotation = rot_z.mul(&rot_y).mul(&rot_x);

            // Transform vertices to world space
            let mut world_vertices: Vec<Vec3> = Vec::with_capacity(60);
            for v in &vertices {
                let rotated = rotation.mul_vec3(*v);
                let world_pos = rotated + transform.position;
                world_vertices.push(world_pos);
            }

            // Project vertices to screen space and calculate view-space Z
            let mut screen_vertices: Vec<Vec2> = Vec::with_capacity(60);
            let mut view_z: Vec<f32> = Vec::with_capacity(60);
            
            for world_pos in &world_vertices {
                // Transform to view space (camera-relative) using view matrix
                let view_pos = view.view.view.mul_vec3(*world_pos);
                view_z.push(view_pos.z);
                
                if let Some((x, y)) = view.view.world_to_screen(*world_pos) {
                    screen_vertices.push(Vec2::new(x, y));
                } else {
                    // Vertex behind camera, skip this soccer ball
                    screen_vertices.clear();
                    break;
                }
            }

            if screen_vertices.len() != 60 {
                continue;
            }

            // Collect all faces with depth for sorting
            // 五边形: (face_index, is_pentagon, avg_z, vertices)
            let mut all_faces: Vec<(usize, bool, f32, Vec<Vec2>)> = Vec::new();

            // Process 12 pentagon faces
            for (face_idx, face) in pentagons.iter().enumerate() {
                let verts: Vec<Vec2> = face.iter()
                    .map(|&idx| screen_vertices[idx])
                    .collect();
                let avg_z: f32 = face.iter()
                    .map(|&idx| view_z[idx])
                    .sum::<f32>() / 5.0;
                all_faces.push((face_idx, true, avg_z, verts));
            }

            // Process 20 hexagon faces
            for (face_idx, face) in hexagons.iter().enumerate() {
                let verts: Vec<Vec2> = face.iter()
                    .map(|&idx| screen_vertices[idx])
                    .collect();
                let avg_z: f32 = face.iter()
                    .map(|&idx| view_z[idx])
                    .sum::<f32>() / 6.0;
                all_faces.push((face_idx, false, avg_z, verts));
            }

            // Sort by view-space Z (far to near) - Painter's algorithm
            // 在右手坐标系 view space 中，相机看向 -Z
            // Z 值越小（越负）表示越近，Z 值越大（越接近 0）表示越远
            // 画家算法：先画远的（Z 值大的），后画近的（Z 值小的）
            all_faces.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

            // Draw all faces
            for (face_idx, is_pentagon, _, verts) in all_faces {
                if is_pentagon {
                    // Draw pentagon (5 vertices -> 3 triangles)
                    let color = soccer_ball.pentagon_colors[face_idx];
                    let v = &verts;
                    
                    // Pentagon triangulation (fan from v0)
                    render_world.add_command(RenderCommand::DrawTriangle {
                        p0: v[0], p1: v[1], p2: v[2], color,
                    });
                    render_world.add_command(RenderCommand::DrawTriangle {
                        p0: v[0], p1: v[2], p2: v[3], color,
                    });
                    render_world.add_command(RenderCommand::DrawTriangle {
                        p0: v[0], p1: v[3], p2: v[4], color,
                    });

                    // Draw wireframe for pentagon (5 edges)
                    if soccer_ball.wireframe {
                        let wf_color = soccer_ball.wireframe_color;
                        for i in 0..5 {
                            render_world.add_command(RenderCommand::DrawLine {
                                start: v[i],
                                end: v[(i + 1) % 5],
                                color: wf_color,
                                thickness: 1.0,
                            });
                        }
                    }
                } else {
                    // Draw hexagon (6 vertices -> 4 triangles)
                    let color = soccer_ball.hexagon_colors[face_idx];
                    let v = &verts;
                    
                    // Hexagon triangulation (fan from v0)
                    render_world.add_command(RenderCommand::DrawTriangle {
                        p0: v[0], p1: v[1], p2: v[2], color,
                    });
                    render_world.add_command(RenderCommand::DrawTriangle {
                        p0: v[0], p1: v[2], p2: v[3], color,
                    });
                    render_world.add_command(RenderCommand::DrawTriangle {
                        p0: v[0], p1: v[3], p2: v[4], color,
                    });
                    render_world.add_command(RenderCommand::DrawTriangle {
                        p0: v[0], p1: v[4], p2: v[5], color,
                    });

                    // Draw wireframe for hexagon (6 edges)
                    if soccer_ball.wireframe {
                        let wf_color = soccer_ball.wireframe_color;
                        for i in 0..6 {
                            render_world.add_command(RenderCommand::DrawLine {
                                start: v[i],
                                end: v[(i + 1) % 6],
                                color: wf_color,
                                thickness: 1.0,
                            });
                        }
                    }
                }
            }
        }
    }
}

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
    schedule.add_extractor(extract_cubes);
    schedule.add_extractor(extract_soccer_balls);
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

/// Create default orthographic view
fn create_default_view(width: f32, height: f32) -> ViewBundle {
    let viewport = Rect::new(0.0, 0.0, width, height);
    let projection = Mat4::orthographic_rh(0.0, width, height, 0.0, -1000.0, 1000.0);
    let view = Mat4::IDENTITY;
    let vp_matrix = projection * view;

    let view = View {
        projection,
        view,
        view_projection: vp_matrix,
        camera_position: Vec3::new(width / 2.0, height / 2.0, 100.0),
        near: -1000.0,
        far: 1000.0,
        orthographic: true,
        viewport,
    };

    ViewBundle {
        view,
        target: ViewTarget::Screen,
        clear: ClearConfig::color(Color::BLACK),
    }
}

/// Create perspective view for 3D rendering
fn create_perspective_view(width: f32, height: f32) -> ViewBundle {
    let viewport = Rect::new(0.0, 0.0, width, height);
    // Perspective projection for 3D - use wider FOV for better 3D effect
    let projection = Mat4::perspective_rh(
        45.0_f32.to_radians(),
        width / height,
        0.1,
        1000.0,
    );

    // Camera positioned to look at the object
    let camera_pos = Vec3::new(width / 2.0, height / 2.0, 600.0);
    let target_pos = Vec3::new(width / 2.0, height / 3.0, 0.0);
    let view = Mat4::look_at_rh(
        camera_pos,
        target_pos,
        Vec3::new(0.0, 1.0, 0.0), // Y is up
    );
    let vp_matrix = projection * view;

    let view = View {
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
        view,
        target: ViewTarget::Screen,
        clear: ClearConfig::color(Color::BLACK),
    }
}
