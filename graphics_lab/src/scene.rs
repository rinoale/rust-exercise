//! Backend-independent geometry and transforms. No GPU API calls live here.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};

pub const CLEAR_COLOR: [f64; 4] = [0.035, 0.048, 0.075, 1.0];
pub const CHECKER_SIZE: u32 = 64;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub uv: [f32; 2],
    pub padding: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Uniforms {
    pub view_projection: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    pub tint: [f32; 4],
    pub eye: [f32; 4],
    pub options: [f32; 4],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

#[derive(Clone, Copy)]
pub enum MeshKind {
    Cube,
    Floor,
}

pub struct Draw {
    pub mesh: MeshKind,
    pub model: Mat4,
    pub tint: [f32; 4],
}

pub struct Scene {
    pub time: f32,
    pub paused: bool,
    pub wireframe: bool,
    pub depth_test: bool,
    pub textured: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            time: 0.0,
            paused: false,
            wireframe: false,
            depth_test: true,
            textured: true,
            yaw: 0.6,
            pitch: 0.38,
            distance: 9.5,
        }
    }
}

impl Scene {
    pub fn tick(&mut self, seconds: f32) {
        if !self.paused {
            self.time += seconds;
        }
    }

    pub fn eye(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        ) * self.distance
            + Vec3::new(0.0, -0.2, 0.0)
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        // Metal clip depth is 0..1; perspective_rh uses that range.
        Mat4::perspective_rh(50.0_f32.to_radians(), aspect, 0.1, 100.0)
            * Mat4::look_at_rh(self.eye(), Vec3::new(0.0, -0.2, 0.0), Vec3::Y)
    }

    pub fn draws(&self) -> [Draw; 3] {
        // Floor last makes disabling depth testing visibly incorrect.
        [
            Draw {
                mesh: MeshKind::Cube,
                model: Mat4::from_scale_rotation_translation(
                    Vec3::ONE,
                    Quat::from_rotation_y(self.time * 0.55) * Quat::from_rotation_x(0.22),
                    Vec3::new(-0.8, 0.0, 0.0),
                ),
                tint: [0.30, 0.74, 1.0, 1.0],
            },
            Draw {
                mesh: MeshKind::Cube,
                model: Mat4::from_scale_rotation_translation(
                    Vec3::splat(0.7),
                    Quat::from_rotation_y(-0.35 - self.time * 0.25),
                    Vec3::new(1.6, -0.6, -0.9),
                ),
                tint: [1.0, 0.52, 0.24, 1.0],
            },
            Draw {
                mesh: MeshKind::Floor,
                model: Mat4::IDENTITY,
                tint: [0.38, 0.45, 0.55, 1.0],
            },
        ]
    }

    pub fn uniforms(&self, draw: &Draw, aspect: f32) -> Uniforms {
        Uniforms {
            view_projection: self.view_projection(aspect).to_cols_array_2d(),
            model: draw.model.to_cols_array_2d(),
            tint: draw.tint,
            eye: self.eye().extend(1.0).to_array(),
            options: [f32::from(self.textured), 0.0, 0.0, 0.0],
        }
    }
}

fn vertex(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Vertex {
    Vertex {
        position: [position[0], position[1], position[2], 1.0],
        normal: [normal[0], normal[1], normal[2], 0.0],
        uv,
        padding: [0.0; 2],
    }
}

pub fn cube() -> Mesh {
    let faces = [
        (
            [0., 0., 1.],
            [[-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.]],
        ),
        (
            [0., 0., -1.],
            [
                [1., -1., -1.],
                [-1., -1., -1.],
                [-1., 1., -1.],
                [1., 1., -1.],
            ],
        ),
        (
            [1., 0., 0.],
            [[1., -1., 1.], [1., -1., -1.], [1., 1., -1.], [1., 1., 1.]],
        ),
        (
            [-1., 0., 0.],
            [
                [-1., -1., -1.],
                [-1., -1., 1.],
                [-1., 1., 1.],
                [-1., 1., -1.],
            ],
        ),
        (
            [0., 1., 0.],
            [[-1., 1., 1.], [1., 1., 1.], [1., 1., -1.], [-1., 1., -1.]],
        ),
        (
            [0., -1., 0.],
            [
                [-1., -1., -1.],
                [1., -1., -1.],
                [1., -1., 1.],
                [-1., -1., 1.],
            ],
        ),
    ];
    let mut mesh = Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    for (normal, positions) in faces {
        let base = mesh.vertices.len() as u16;
        for (position, uv) in positions
            .into_iter()
            .zip([[0., 0.], [2., 0.], [2., 2.], [0., 2.]])
        {
            mesh.vertices.push(vertex(position, normal, uv));
        }
        mesh.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    mesh
}

pub fn floor() -> Mesh {
    Mesh {
        vertices: [
            ([-6., -1.3, 6.], [0., 0.]),
            ([6., -1.3, 6.], [10., 0.]),
            ([6., -1.3, -6.], [10., 10.]),
            ([-6., -1.3, -6.], [0., 10.]),
        ]
        .into_iter()
        .map(|(p, uv)| vertex(p, [0., 1., 0.], uv))
        .collect(),
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

pub fn checkerboard() -> Vec<u8> {
    let mut pixels = Vec::with_capacity((CHECKER_SIZE * CHECKER_SIZE * 4) as usize);
    for y in 0..CHECKER_SIZE {
        for x in 0..CHECKER_SIZE {
            let shade = if (x / 16 + y / 16) % 2 == 0 { 245 } else { 100 };
            pixels.extend([shade, shade, shade, 255]);
        }
    }
    pixels
}
