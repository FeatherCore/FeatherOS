use crate::{
    fixed_div, fixed_from_i32, fixed_mul, fixed_to_i32, Color, DepthSpan, DrawCommand, DrawList,
    Fixed16, ImageId, Point, Rect, Size, TexCoord, FIXED_ONE,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Vec3 {
    pub x: Fixed16,
    pub y: Fixed16,
    pub z: Fixed16,
}

impl Vec3 {
    pub const ZERO: Self = Self::from_i32(0, 0, 0);
    pub const ONE: Self = Self::from_i32(1, 1, 1);

    pub const fn new(x: Fixed16, y: Fixed16, z: Fixed16) -> Self {
        Self { x, y, z }
    }

    pub const fn from_i32(x: i32, y: i32, z: i32) -> Self {
        Self {
            x: fixed_from_i32(x),
            y: fixed_from_i32(y),
            z: fixed_from_i32(z),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rotation {
    None,
    Rotate2D(i16),
    Euler { x: i16, y: i16, z: i16 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Transform3D {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Rotation,
}

impl Transform3D {
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        scale: Vec3::ONE,
        rotation: Rotation::None,
    };

    pub const fn from_xyz(x: i32, y: i32, z: i32) -> Self {
        Self {
            position: Vec3::from_i32(x, y, z),
            scale: Vec3::ONE,
            rotation: Rotation::None,
        }
    }

    pub const fn screen(x: i32, y: i32, z: i32) -> Self {
        Self::from_xyz(x, y, z)
    }

    pub const fn with_scale(mut self, x: Fixed16, y: Fixed16, z: Fixed16) -> Self {
        self.scale = Vec3::new(x, y, z);
        self
    }

    pub const fn with_rotation(mut self, rotation: Rotation) -> Self {
        self.rotation = rotation;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Projection {
    Orthographic {
        scale: Fixed16,
    },
    Perspective {
        fov_y: i16,
        near: Fixed16,
        far: Fixed16,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Camera {
    pub transform: Transform3D,
    pub projection: Projection,
    pub viewport: Rect,
}

impl Camera {
    pub const fn screen_canvas(width: u16, height: u16) -> Self {
        Self {
            transform: Transform3D::IDENTITY,
            projection: Projection::Orthographic { scale: FIXED_ONE },
            viewport: Rect::new(0, 0, width, height),
        }
    }

    pub fn project_point(&self, world: Vec3) -> ProjectedPoint {
        match self.projection {
            Projection::Orthographic { scale } => {
                let x = fixed_to_i32(fixed_mul(world.x - self.transform.position.x, scale));
                let y = fixed_to_i32(fixed_mul(world.y - self.transform.position.y, scale));
                ProjectedPoint {
                    point: Point::new(self.viewport.x + x, self.viewport.y + y),
                    depth: world.z - self.transform.position.z,
                }
            }
            Projection::Perspective { near, .. } => {
                let rel_x = world.x - self.transform.position.x;
                let rel_y = world.y - self.transform.position.y;
                let rel_z = (world.z - self.transform.position.z).max(near.max(FIXED_ONE));
                let focal = fixed_from_i32((self.viewport.h.max(1) / 2) as i32);
                let x = fixed_to_i32(fixed_div(fixed_mul(rel_x, focal), rel_z));
                let y = fixed_to_i32(fixed_div(fixed_mul(rel_y, focal), rel_z));
                ProjectedPoint {
                    point: Point::new(
                        self.viewport.x + self.viewport.w as i32 / 2 + x,
                        self.viewport.y + self.viewport.h as i32 / 2 + y,
                    ),
                    depth: rel_z,
                }
            }
        }
    }

    pub fn project_rect(&self, transform: Transform3D, size: Size) -> ProjectedRect {
        let origin = self.project_point(transform.position);
        let w = scale_u16(size.w, transform.scale.x);
        let h = scale_u16(size.h, transform.scale.y);
        ProjectedRect {
            rect: Rect::new(origin.point.x, origin.point.y, w, h),
            depth: origin.depth,
            planar: is_planar_canvas(transform),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectedPoint {
    pub point: Point,
    pub depth: Fixed16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectedRect {
    pub rect: Rect,
    pub depth: Fixed16,
    pub planar: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderNode {
    pub transform: Transform3D,
    pub bounds: Size,
    pub clip: Option<Rect>,
    pub opacity: u8,
    pub layer: i16,
}

impl RenderNode {
    pub const fn new(transform: Transform3D, bounds: Size) -> Self {
        Self {
            transform,
            bounds,
            clip: None,
            opacity: 255,
            layer: 0,
        }
    }

    pub fn project(&self, camera: &Camera) -> ProjectedRect {
        camera.project_rect(self.transform, self.bounds)
    }
}

fn scale_u16(value: u16, scale: Fixed16) -> u16 {
    let scaled = fixed_mul(fixed_from_i32(value as i32), scale);
    fixed_to_i32(scaled).max(1).min(u16::MAX as i32) as u16
}

fn is_planar_canvas(transform: Transform3D) -> bool {
    transform.position.z == 0
        && matches!(transform.rotation, Rotation::None)
        && transform.scale.z == FIXED_ONE
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Vertex3D {
    pub position: Vec3,
    pub color: Color,
}

impl Vertex3D {
    pub const fn new(position: Vec3, color: Color) -> Self {
        Self { position, color }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeshRef {
    pub vertices: &'static [Vertex3D],
    pub indices: &'static [u16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TexturedVertex3D {
    pub position: Vec3,
    pub uv: TexCoord,
}

impl TexturedVertex3D {
    pub const fn new(position: Vec3, uv: TexCoord) -> Self {
        Self { position, uv }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TexturedMeshRef {
    pub vertices: &'static [TexturedVertex3D],
    pub indices: &'static [u16],
    pub image: ImageId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeshDrawOptions {
    pub wireframe: bool,
    pub fill: bool,
    pub painter_sort: bool,
    pub wire_color: Color,
}

impl MeshDrawOptions {
    pub const FLAT: Self = Self {
        wireframe: false,
        fill: true,
        painter_sort: true,
        wire_color: Color::WHITE,
    };

    pub const WIREFRAME: Self = Self {
        wireframe: true,
        fill: false,
        painter_sort: true,
        wire_color: Color::WHITE,
    };
}

impl MeshRef {
    pub const fn new(vertices: &'static [Vertex3D], indices: &'static [u16]) -> Self {
        Self { vertices, indices }
    }

    pub fn emit_orthographic<const N: usize>(
        self,
        camera: &Camera,
        transform: Transform3D,
        list: &mut DrawList<N>,
    ) -> usize {
        self.emit_with_options(camera, transform, MeshDrawOptions::FLAT, list)
    }

    pub fn emit_with_options<const N: usize>(
        self,
        camera: &Camera,
        transform: Transform3D,
        options: MeshDrawOptions,
        list: &mut DrawList<N>,
    ) -> usize {
        let mut emitted = 0;
        let mut index = 0;
        while index + 2 < self.indices.len() {
            let Some(a) = self.vertex(self.indices[index]) else {
                break;
            };
            let Some(b) = self.vertex(self.indices[index + 1]) else {
                break;
            };
            let Some(c) = self.vertex(self.indices[index + 2]) else {
                break;
            };
            let pa = camera.project_point(apply_transform(a.position, transform));
            let pb = camera.project_point(apply_transform(b.position, transform));
            let pc = camera.project_point(apply_transform(c.position, transform));
            if options.fill {
                if list.push(DrawCommand::DrawTriangle {
                    p0: pa.point,
                    p1: pb.point,
                    p2: pc.point,
                    depth: DepthSpan {
                        a: pa.depth,
                        b: pb.depth,
                        c: pc.depth,
                    },
                    color: average_color(a.color, b.color, c.color),
                }) {
                    emitted += 1;
                }
            }
            if options.wireframe {
                let depth = pa.depth.max(pb.depth).max(pc.depth);
                if list.push(DrawCommand::StrokeLine {
                    from: pa.point,
                    to: pb.point,
                    depth,
                    color: options.wire_color,
                    width: 1,
                }) {
                    emitted += 1;
                }
                if list.push(DrawCommand::StrokeLine {
                    from: pb.point,
                    to: pc.point,
                    depth,
                    color: options.wire_color,
                    width: 1,
                }) {
                    emitted += 1;
                }
                if list.push(DrawCommand::StrokeLine {
                    from: pc.point,
                    to: pa.point,
                    depth,
                    color: options.wire_color,
                    width: 1,
                }) {
                    emitted += 1;
                }
            }
            index += 3;
        }
        if options.painter_sort {
            list.sort_by_depth();
        }
        emitted
    }

    fn vertex(self, index: u16) -> Option<Vertex3D> {
        self.vertices.get(index as usize).copied()
    }
}

impl TexturedMeshRef {
    pub const fn new(
        vertices: &'static [TexturedVertex3D],
        indices: &'static [u16],
        image: ImageId,
    ) -> Self {
        Self {
            vertices,
            indices,
            image,
        }
    }

    pub fn emit<const N: usize>(
        self,
        camera: &Camera,
        transform: Transform3D,
        opacity: u8,
        painter_sort: bool,
        list: &mut DrawList<N>,
    ) -> usize {
        let mut emitted = 0;
        let mut index = 0;
        while index + 2 < self.indices.len() {
            let Some(a) = self.vertex(self.indices[index]) else {
                break;
            };
            let Some(b) = self.vertex(self.indices[index + 1]) else {
                break;
            };
            let Some(c) = self.vertex(self.indices[index + 2]) else {
                break;
            };
            let pa = camera.project_point(apply_transform(a.position, transform));
            let pb = camera.project_point(apply_transform(b.position, transform));
            let pc = camera.project_point(apply_transform(c.position, transform));
            if list.push(DrawCommand::DrawTexturedTriangle {
                p0: pa.point,
                p1: pb.point,
                p2: pc.point,
                uv0: a.uv,
                uv1: b.uv,
                uv2: c.uv,
                depth: DepthSpan {
                    a: pa.depth,
                    b: pb.depth,
                    c: pc.depth,
                },
                image: self.image,
                opacity,
            }) {
                emitted += 1;
            }
            index += 3;
        }
        if painter_sort {
            list.sort_by_depth();
        }
        emitted
    }

    fn vertex(self, index: u16) -> Option<TexturedVertex3D> {
        self.vertices.get(index as usize).copied()
    }
}

fn apply_transform(point: Vec3, transform: Transform3D) -> Vec3 {
    Vec3::new(
        transform
            .position
            .x
            .saturating_add(fixed_mul(point.x, transform.scale.x)),
        transform
            .position
            .y
            .saturating_add(fixed_mul(point.y, transform.scale.y)),
        transform
            .position
            .z
            .saturating_add(fixed_mul(point.z, transform.scale.z)),
    )
}

fn average_color(a: Color, b: Color, c: Color) -> Color {
    Color::rgba(
        ((a.r as u16 + b.r as u16 + c.r as u16) / 3) as u8,
        ((a.g as u16 + b.g as u16 + c.g as u16) / 3) as u8,
        ((a.b as u16 + b.b as u16 + c.b as u16) / 3) as u8,
        ((a.a as u16 + b.a as u16 + c.a as u16) / 3) as u8,
    )
}
