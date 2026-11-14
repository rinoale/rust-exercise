//! Direct Metal rendering: one exposed-face world mesh and a small HUD mesh.
use crate::voxel::VoxelVertex;
use anyhow::{Context, Result, anyhow, ensure};
use bytemuck::{Pod, Zeroable};
use metal::*;
use std::{fs::File, io::BufWriter, path::Path};

pub use crate::presentation::RenderScene;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    view_projection: [[f32; 4]; 4],
    eye: [f32; 4],
    selected: [f32; 4],
}
pub struct VoxelRenderer {
    pub device: Device,
    queue: CommandQueue,
    world_pipeline: RenderPipelineState,
    hud_pipeline: RenderPipelineState,
    depth_on: DepthStencilState,
    depth_off: DepthStencilState,
    depth: Option<Texture>,
    mesh: Option<Buffer>,
    vertices: u64,
    revision: Option<u64>,
}
impl VoxelRenderer {
    pub fn new() -> Result<Self> {
        let device = Device::system_default().context("No Metal GPU available")?;
        let library = device
            .new_library_with_source(include_str!("voxel.metal"), &CompileOptions::new())
            .map_err(|e| anyhow!(e))?;
        let pipeline = |vertex, fragment| -> Result<RenderPipelineState> {
            let desc = RenderPipelineDescriptor::new();
            let vertex_function = library.get_function(vertex, None).map_err(|e| anyhow!(e))?;
            let fragment_function = library
                .get_function(fragment, None)
                .map_err(|e| anyhow!(e))?;
            desc.set_vertex_function(Some(&vertex_function));
            desc.set_fragment_function(Some(&fragment_function));
            desc.color_attachments()
                .object_at(0)
                .unwrap()
                .set_pixel_format(MTLPixelFormat::BGRA8Unorm_sRGB);
            desc.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);
            device
                .new_render_pipeline_state(&desc)
                .map_err(|e| anyhow!(e))
        };
        let world_pipeline = pipeline("world_vertex", "world_fragment")?;
        let hud_pipeline = pipeline("hud_vertex", "hud_fragment")?;
        let desc = DepthStencilDescriptor::new();
        desc.set_depth_compare_function(MTLCompareFunction::Less);
        desc.set_depth_write_enabled(true);
        let depth_on = device.new_depth_stencil_state(&desc);
        desc.set_depth_compare_function(MTLCompareFunction::Always);
        desc.set_depth_write_enabled(false);
        let depth_off = device.new_depth_stencil_state(&desc);
        Ok(Self {
            queue: device.new_command_queue(),
            device,
            world_pipeline,
            hud_pipeline,
            depth_on,
            depth_off,
            depth: None,
            mesh: None,
            vertices: 0,
            revision: None,
        })
    }
    fn upload(&self, vertices: &[VoxelVertex]) -> Buffer {
        let bytes: &[u8] = bytemuck::cast_slice(vertices);
        self.device.new_buffer_with_data(
            bytes.as_ptr().cast(),
            bytes.len() as u64,
            MTLResourceOptions::StorageModeShared,
        )
    }
    pub fn render(
        &mut self,
        game: &impl RenderScene,
        target: &TextureRef,
        drawable: Option<&MetalDrawableRef>,
        captured: bool,
    ) -> Result<()> {
        if self.revision != Some(game.revision()) {
            let vertices = game.mesh();
            self.vertices = vertices.len() as u64;
            self.mesh = if vertices.is_empty() {
                None
            } else {
                Some(self.upload(&vertices))
            };
            self.revision = Some(game.revision());
        }
        if !self
            .depth
            .as_ref()
            .is_some_and(|t| t.width() == target.width() && t.height() == target.height())
        {
            let desc = TextureDescriptor::new();
            desc.set_width(target.width());
            desc.set_height(target.height());
            desc.set_pixel_format(MTLPixelFormat::Depth32Float);
            desc.set_storage_mode(MTLStorageMode::Private);
            desc.set_usage(MTLTextureUsage::RenderTarget);
            self.depth = Some(self.device.new_texture(&desc));
        }
        let pass = RenderPassDescriptor::new();
        let color = pass.color_attachments().object_at(0).unwrap();
        color.set_texture(Some(target));
        color.set_load_action(MTLLoadAction::Clear);
        color.set_store_action(MTLStoreAction::Store);
        color.set_clear_color(MTLClearColor::new(0.46, 0.68, 0.86, 1.0));
        let depth = pass.depth_attachment().unwrap();
        depth.set_texture(self.depth.as_deref());
        depth.set_load_action(MTLLoadAction::Clear);
        depth.set_store_action(MTLStoreAction::DontCare);
        depth.set_clear_depth(1.0);
        let uniforms = Uniforms {
            view_projection: game
                .view_projection(target.width() as f32 / target.height() as f32)
                .to_cols_array_2d(),
            eye: game.eye().extend(1.0).to_array(),
            selected: game.selection(),
        };
        let command = self.queue.new_command_buffer();
        let encoder = command.new_render_command_encoder(pass);
        encoder.set_render_pipeline_state(&self.world_pipeline);
        encoder.set_depth_stencil_state(&self.depth_on);
        encoder.set_front_facing_winding(MTLWinding::CounterClockwise);
        encoder.set_cull_mode(MTLCullMode::Back);
        let bytes = bytemuck::bytes_of(&uniforms);
        encoder.set_vertex_bytes(1, bytes.len() as u64, bytes.as_ptr().cast());
        encoder.set_fragment_bytes(1, bytes.len() as u64, bytes.as_ptr().cast());
        if let Some(mesh) = &self.mesh {
            encoder.set_vertex_buffer(0, Some(mesh), 0);
            encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, self.vertices);
        }
        let preview = game.preview();
        // Retain the buffer through command completion, just like the HUD.
        let preview_buffer = (!preview.is_empty()).then(|| self.upload(&preview));
        if let Some(buffer) = &preview_buffer {
            encoder.set_vertex_buffer(0, Some(buffer), 0);
            encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, preview.len() as u64);
        }
        let hud = self.upload(&game.hud(target.width() as f32, target.height() as f32, captured));
        encoder.set_render_pipeline_state(&self.hud_pipeline);
        encoder.set_depth_stencil_state(&self.depth_off);
        encoder.set_cull_mode(MTLCullMode::None);
        encoder.set_vertex_buffer(0, Some(&hud), 0);
        encoder.draw_primitives(
            MTLPrimitiveType::Triangle,
            0,
            hud.length() / std::mem::size_of::<VoxelVertex>() as u64,
        );
        encoder.end_encoding();
        if let Some(drawable) = drawable {
            command.present_drawable(drawable);
        }
        command.commit();
        command.wait_until_completed();
        ensure!(
            command.status() == MTLCommandBufferStatus::Completed,
            "Voxel GPU frame failed: {:?}",
            command.status()
        );
        Ok(())
    }
    pub fn capture(
        &mut self,
        game: &impl RenderScene,
        width: u32,
        height: u32,
        path: &Path,
    ) -> Result<()> {
        let desc = TextureDescriptor::new();
        desc.set_width(width as u64);
        desc.set_height(height as u64);
        desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm_sRGB);
        desc.set_storage_mode(MTLStorageMode::Shared);
        desc.set_usage(MTLTextureUsage::RenderTarget);
        let target = self.device.new_texture(&desc);
        self.render(game, &target, None, true)?;
        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        target.get_bytes(
            pixels.as_mut_ptr().cast(),
            width as u64 * 4,
            MTLRegion::new_2d(0, 0, width as u64, height as u64),
            0,
        );
        for p in pixels.chunks_exact_mut(4) {
            p.swap(0, 2);
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let mut png = png::Encoder::new(BufWriter::new(File::create(path)?), width, height);
        png.set_color(png::ColorType::Rgba);
        png.set_depth(png::BitDepth::Eight);
        png.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        png.write_header()?.write_image_data(&pixels)?;
        println!("Saved {}", path.display());
        Ok(())
    }
}
