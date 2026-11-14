use crate::{
    graphics::{self, DrawCall, Frame, GraphicsApi, Shading, Uniforms},
    scene::{self, Scene},
    voxel::VoxelVertex,
};
use anyhow::Result;
use std::{path::Path, sync::Arc};
use winit::window::Window;

pub struct CubeRenderer {
    api: Box<dyn GraphicsApi>,
}
impl CubeRenderer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            api: graphics::create(None)?,
        })
    }
    pub fn for_window(window: Arc<Window>) -> Result<Self> {
        Ok(Self {
            api: graphics::create(Some(window))?,
        })
    }
    pub fn render(&mut self, scene: &Scene) -> Result<()> {
        let (w, h) = self.api.size();
        self.api.render(&frame(scene, w as f32 / h.max(1) as f32))
    }
    pub fn capture(&mut self, scene: &Scene, width: u32, height: u32, path: &Path) -> Result<()> {
        let pixels = self.api.read_pixels(
            &frame(scene, width as f32 / height.max(1) as f32),
            width,
            height,
        )?;
        graphics::save_png(path, width, height, &pixels)
    }
}
pub fn frame(scene: &Scene, aspect: f32) -> Frame {
    let draws = scene
        .draws()
        .iter()
        .map(|draw| {
            let mesh = match draw.mesh {
                scene::MeshKind::Cube => scene::cube(),
                scene::MeshKind::Floor => scene::floor(),
            };
            let vertices = mesh
                .indices
                .iter()
                .map(|&i| {
                    let v = mesh.vertices[i as usize];
                    VoxelVertex {
                        position: v.position,
                        normal: v.normal,
                        color: [1.; 4],
                        uv: v.uv,
                        padding: [0.; 2],
                    }
                })
                .collect();
            let u = scene.uniforms(draw, aspect);
            DrawCall {
                vertices,
                uniforms: Uniforms {
                    view_projection: u.view_projection,
                    model: u.model,
                    tint: u.tint,
                    eye: u.eye,
                    options: u.options,
                    selected: [0.; 4],
                },
                shading: Shading::Cube,
                depth: scene.depth_test,
                wireframe: scene.wireframe,
            }
        })
        .collect();
    Frame {
        clear: scene::CLEAR_COLOR,
        draws,
    }
}
