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
use crate::ui::{Button, Cube};
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

impl Default for ExtractSchedule {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract sprites
pub fn extract_sprites(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let transforms: Vec<_> = main_world.query::<Transform>().collect();

    for (transform_entity, transform) in transforms {
        if let Some(sprite) = main_world.get_component::<Sprite>(transform_entity) {
            if sprite.visible {
                let render_object = RenderObject {
                    position: transform.position,
                    rotation: transform.rotation,
                    scale: transform.scale,
                    color: sprite.color,
                    size: Vec2::new(sprite.width, sprite.height),
                    visible: sprite.visible,
                    z_order: 0,
                };
                render_world.add_object(render_object);
            }
        }
    }
}

/// Extract buttons - simple 2D rendering
pub fn extract_buttons(main_world: &MainWorld, render_world: &mut RenderWorld) {
    // Query all entities with Transform2D and Button
    let transforms: Vec<_> = main_world.query::<Transform2D>().collect();

    for (entity, transform) in transforms {
        if let Some(button) = main_world.get_component::<Button>(entity) {
            let pos = Vec2::new(transform.position.x, transform.position.y);
            let (tl, tr, bl, br) = button.get_rect(pos);
            let color = button.current_color();

            // Draw button as two triangles (quad)
            // Triangle 1: tl, tr, br
            render_world.add_command(RenderCommand::DrawTriangle {
                p0: tl,
                p1: tr,
                p2: br,
                color,
            });
            // Triangle 2: tl, br, bl
            render_world.add_command(RenderCommand::DrawTriangle {
                p0: tl,
                p1: br,
                p2: bl,
                color,
            });

            // Draw border if needed
            if button.border_width > 0.0 {
                let border_color = button.border_color;
                let thickness = button.border_width;

                // Top border
                render_world.add_command(RenderCommand::DrawLine {
                    start: tl,
                    end: tr,
                    color: border_color,
                    thickness,
                });
                // Right border
                render_world.add_command(RenderCommand::DrawLine {
                    start: tr,
                    end: br,
                    color: border_color,
                    thickness,
                });
                // Bottom border
                render_world.add_command(RenderCommand::DrawLine {
                    start: br,
                    end: bl,
                    color: border_color,
                    thickness,
                });
                // Left border
                render_world.add_command(RenderCommand::DrawLine {
                    start: bl,
                    end: tl,
                    color: border_color,
                    thickness,
                });
            }
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

                // 如果 cross_z > 0，面朝向相机（逆时针 winding）
                if cross_z > 0.0 {
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
            // 在 view space 中，Z 越大表示离相机越远
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

                    // Draw face edges
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
    // Perspective projection for 3D
    let projection = Mat4::perspective_rh(
        60.0_f32.to_radians(),
        width / height,
        0.1,
        1000.0,
    );
    // Camera looking at origin from positive Z
    let view = Mat4::look_at_rh(
        Vec3::new(width / 2.0, height / 2.0, 400.0),
        Vec3::new(width / 2.0, height / 2.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
    );
    let vp_matrix = projection * view;

    let view = View {
        projection,
        view,
        view_projection: vp_matrix,
        camera_position: Vec3::new(width / 2.0, height / 2.0, 400.0),
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

/// Extract system - Runs all extraction
pub fn extract_system(main_world: &MainWorld, render_world: &mut RenderWorld) {
    render_world.reset();

    // Get screen dimensions
    let (width, height) = if let Some(screen) = main_world.resources().get::<PrimaryScreen>() {
        screen.dimensions()
    } else {
        (800, 600)
    };

    // Create perspective view for 3D cube rendering
    let default_view = create_perspective_view(width as f32, height as f32);
    let view_idx = render_world.add_view(default_view);
    render_world.set_current_view(Some(view_idx));

    // Extract sprites
    extract_sprites(main_world, render_world);

    // Extract buttons
    extract_buttons(main_world, render_world);

    // Extract cubes
    extract_cubes(main_world, render_world);
}

/// Plugin trait for extract systems
pub trait ExtractPlugin {
    fn register(&self, schedule: &mut ExtractSchedule);
}

/// Built-in sprite extractor plugin
pub struct SpriteExtractor;

impl ExtractPlugin for SpriteExtractor {
    fn register(&self, schedule: &mut ExtractSchedule) {
        schedule.add_extractor(extract_sprites);
    }
}

/// Button extractor plugin
pub struct ButtonExtractor;

impl ExtractPlugin for ButtonExtractor {
    fn register(&self, schedule: &mut ExtractSchedule) {
        schedule.add_extractor(extract_buttons);
    }
}
