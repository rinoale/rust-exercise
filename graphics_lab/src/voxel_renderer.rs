pub use crate::presentation::RenderScene;
use crate::{
    graphics::{self, DrawCall, Frame, GraphicsApi, Shading, Uniforms},
    voxel::VoxelVertex,
};
use anyhow::Result;
use std::{path::Path, sync::Arc};
use winit::window::Window;

pub struct VoxelRenderer {
    api: Box<dyn GraphicsApi>,
    revision: Option<u64>,
    mesh: Arc<[VoxelVertex]>,
}
impl VoxelRenderer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            api: graphics::create(None)?,
            revision: None,
            mesh: Arc::from([]),
        })
    }
    pub fn for_window(window: Arc<Window>) -> Result<Self> {
        Ok(Self {
            api: graphics::create(Some(window))?,
            revision: None,
            mesh: Arc::from([]),
        })
    }
    fn frame(
        &mut self,
        scene: &impl RenderScene,
        width: u32,
        height: u32,
        captured: bool,
    ) -> Frame {
        if self.revision != Some(scene.revision()) {
            self.mesh = scene.mesh().into();
            self.revision = Some(scene.revision());
        }
        let u = Uniforms {
            view_projection: scene
                .view_projection(width as f32 / height.max(1) as f32)
                .to_cols_array_2d(),
            model: glam::Mat4::IDENTITY.to_cols_array_2d(),
            tint: [1.; 4],
            eye: scene.eye().extend(1.).to_array(),
            options: [0.; 4],
            selected: scene.selection(),
        };
        let mut draws = vec![DrawCall {
            vertices: self.mesh.clone(),
            uniforms: u,
            shading: Shading::World,
            depth: true,
            wireframe: false,
        }];
        let preview = scene.preview();
        if !preview.is_empty() {
            draws.push(DrawCall {
                vertices: preview.into(),
                uniforms: u,
                shading: Shading::World,
                depth: true,
                wireframe: false,
            });
        }
        draws.push(DrawCall {
            vertices: scene.hud(width as f32, height as f32, captured).into(),
            uniforms: Uniforms {
                view_projection: glam::Mat4::IDENTITY.to_cols_array_2d(),
                ..u
            },
            shading: Shading::Hud,
            depth: false,
            wireframe: false,
        });
        Frame {
            clear: [0.46, 0.68, 0.86, 1.],
            draws,
        }
    }
    pub fn render(&mut self, scene: &impl RenderScene, captured: bool) -> Result<()> {
        let (w, h) = self.api.size();
        let frame = self.frame(scene, w, h, captured);
        self.api.render(&frame)
    }
    pub fn capture(
        &mut self,
        scene: &impl RenderScene,
        width: u32,
        height: u32,
        path: &Path,
    ) -> Result<()> {
        graphics::validate_size(width, height)?;
        let frame = self.frame(scene, width, height, true);
        let pixels = self.api.read_pixels(&frame, width, height)?;
        graphics::save_png(path, width, height, &pixels)
    }
}
