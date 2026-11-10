//! Native Metal implementation, including offscreen rendering for validation.

use std::{fs::File, io::BufWriter, path::Path};

use anyhow::{Context, Result, anyhow, ensure};
use metal::*;

use crate::scene::{self, MeshKind, Scene};

struct GpuMesh {
    vertices: Buffer,
    indices: Buffer,
    index_count: u64,
}

impl GpuMesh {
    fn new(device: &DeviceRef, mesh: scene::Mesh) -> Self {
        let vertices: &[u8] = bytemuck::cast_slice(&mesh.vertices);
        let indices: &[u8] = bytemuck::cast_slice(&mesh.indices);
        Self {
            vertices: device.new_buffer_with_data(
                vertices.as_ptr().cast(),
                vertices.len() as u64,
                MTLResourceOptions::StorageModeShared,
            ),
            indices: device.new_buffer_with_data(
                indices.as_ptr().cast(),
                indices.len() as u64,
                MTLResourceOptions::StorageModeShared,
            ),
            index_count: mesh.indices.len() as u64,
        }
    }
}

pub struct MetalRenderer {
    pub device: Device,
    queue: CommandQueue,
    pipeline: RenderPipelineState,
    depth_enabled: DepthStencilState,
    depth_disabled: DepthStencilState,
    depth_texture: Option<Texture>,
    cube: GpuMesh,
    floor: GpuMesh,
    checker: Texture,
    sampler: SamplerState,
}

impl MetalRenderer {
    pub fn new() -> Result<Self> {
        let device = Device::system_default().context("No Metal GPU is available")?;
        println!("Metal GPU: {}", device.name());
        let library = device
            .new_library_with_source(include_str!("shaders.metal"), &CompileOptions::new())
            .map_err(|e| anyhow!("Runtime shader compilation failed: {e}"))?;
        let pipeline_desc = RenderPipelineDescriptor::new();
        pipeline_desc.set_label("Checkerboard cubes + floor");
        let vertex_function = library
            .get_function("cube_vertex", None)
            .map_err(|e| anyhow!(e))?;
        let fragment_function = library
            .get_function("cube_fragment", None)
            .map_err(|e| anyhow!(e))?;
        pipeline_desc.set_vertex_function(Some(&vertex_function));
        pipeline_desc.set_fragment_function(Some(&fragment_function));
        pipeline_desc
            .color_attachments()
            .object_at(0)
            .unwrap()
            .set_pixel_format(MTLPixelFormat::BGRA8Unorm_sRGB);
        pipeline_desc.set_depth_attachment_pixel_format(MTLPixelFormat::Depth32Float);
        let pipeline = device
            .new_render_pipeline_state(&pipeline_desc)
            .map_err(|e| anyhow!(e))?;

        let depth_desc = DepthStencilDescriptor::new();
        depth_desc.set_depth_compare_function(MTLCompareFunction::Less);
        depth_desc.set_depth_write_enabled(true);
        let depth_enabled = device.new_depth_stencil_state(&depth_desc);
        depth_desc.set_depth_compare_function(MTLCompareFunction::Always);
        depth_desc.set_depth_write_enabled(false);
        let depth_disabled = device.new_depth_stencil_state(&depth_desc);

        let tex_desc = TextureDescriptor::new();
        tex_desc.set_texture_type(MTLTextureType::D2);
        tex_desc.set_width(scene::CHECKER_SIZE as u64);
        tex_desc.set_height(scene::CHECKER_SIZE as u64);
        tex_desc.set_mipmap_level_count(7);
        tex_desc.set_pixel_format(MTLPixelFormat::RGBA8Unorm_sRGB);
        tex_desc.set_storage_mode(MTLStorageMode::Shared);
        tex_desc.set_usage(MTLTextureUsage::ShaderRead);
        let checker = device.new_texture(&tex_desc);
        checker.set_label("64x64 checkerboard");
        checker.replace_region(
            MTLRegion::new_2d(0, 0, scene::CHECKER_SIZE as u64, scene::CHECKER_SIZE as u64),
            0,
            scene::checkerboard().as_ptr().cast(),
            (scene::CHECKER_SIZE * 4) as u64,
        );
        let queue = device.new_command_queue();
        let upload = queue.new_command_buffer();
        let blit = upload.new_blit_command_encoder();
        blit.generate_mipmaps(&checker);
        blit.end_encoding();
        upload.commit();
        upload.wait_until_completed();
        ensure!(
            upload.status() == MTLCommandBufferStatus::Completed,
            "Checkerboard mipmap generation failed"
        );
        let sampler_desc = SamplerDescriptor::new();
        sampler_desc.set_min_filter(MTLSamplerMinMagFilter::Linear);
        sampler_desc.set_mag_filter(MTLSamplerMinMagFilter::Nearest);
        sampler_desc.set_mip_filter(MTLSamplerMipFilter::Linear);
        sampler_desc.set_max_anisotropy(8);
        sampler_desc.set_address_mode_s(MTLSamplerAddressMode::Repeat);
        sampler_desc.set_address_mode_t(MTLSamplerAddressMode::Repeat);
        let sampler = device.new_sampler(&sampler_desc);

        Ok(Self {
            queue,
            cube: GpuMesh::new(&device, scene::cube()),
            floor: GpuMesh::new(&device, scene::floor()),
            device,
            pipeline,
            depth_enabled,
            depth_disabled,
            depth_texture: None,
            checker,
            sampler,
        })
    }

    fn ensure_depth(&mut self, width: u64, height: u64) {
        if self
            .depth_texture
            .as_ref()
            .is_some_and(|t| t.width() == width && t.height() == height)
        {
            return;
        }
        let desc = TextureDescriptor::new();
        desc.set_texture_type(MTLTextureType::D2);
        desc.set_width(width);
        desc.set_height(height);
        desc.set_pixel_format(MTLPixelFormat::Depth32Float);
        desc.set_storage_mode(MTLStorageMode::Private);
        desc.set_usage(MTLTextureUsage::RenderTarget);
        self.depth_texture = Some(self.device.new_texture(&desc));
    }

    /// One in-flight frame keeps this learning sample's resource ownership simple.
    /// Completion is awaited, so reported CPU frame time includes the GPU wait.
    pub fn render(
        &mut self,
        scene: &Scene,
        target: &TextureRef,
        drawable: Option<&MetalDrawableRef>,
    ) -> Result<()> {
        self.ensure_depth(target.width(), target.height());
        let pass = RenderPassDescriptor::new();
        let color = pass.color_attachments().object_at(0).unwrap();
        color.set_texture(Some(target));
        color.set_load_action(MTLLoadAction::Clear);
        color.set_store_action(MTLStoreAction::Store);
        color.set_clear_color(MTLClearColor::new(
            scene::CLEAR_COLOR[0],
            scene::CLEAR_COLOR[1],
            scene::CLEAR_COLOR[2],
            1.0,
        ));
        let depth = pass.depth_attachment().unwrap();
        depth.set_texture(self.depth_texture.as_deref());
        depth.set_load_action(MTLLoadAction::Clear);
        depth.set_store_action(MTLStoreAction::DontCare);
        depth.set_clear_depth(1.0);

        let command = self.queue.new_command_buffer();
        command.set_label("3D sample frame");
        let encoder = command.new_render_command_encoder(pass);
        encoder.set_render_pipeline_state(&self.pipeline);
        encoder.set_depth_stencil_state(if scene.depth_test {
            &self.depth_enabled
        } else {
            &self.depth_disabled
        });
        encoder.set_front_facing_winding(MTLWinding::CounterClockwise);
        encoder.set_cull_mode(MTLCullMode::Back);
        encoder.set_triangle_fill_mode(if scene.wireframe {
            MTLTriangleFillMode::Lines
        } else {
            MTLTriangleFillMode::Fill
        });
        encoder.set_fragment_texture(0, Some(&self.checker));
        encoder.set_fragment_sampler_state(0, Some(&self.sampler));

        for draw in scene.draws() {
            let mesh = match draw.mesh {
                MeshKind::Cube => &self.cube,
                MeshKind::Floor => &self.floor,
            };
            let uniforms = scene.uniforms(&draw, target.width() as f32 / target.height() as f32);
            let bytes = bytemuck::bytes_of(&uniforms);
            encoder.set_vertex_buffer(0, Some(&mesh.vertices), 0);
            encoder.set_vertex_bytes(1, bytes.len() as u64, bytes.as_ptr().cast());
            encoder.set_fragment_bytes(1, bytes.len() as u64, bytes.as_ptr().cast());
            encoder.draw_indexed_primitives(
                MTLPrimitiveType::Triangle,
                mesh.index_count,
                MTLIndexType::UInt16,
                &mesh.indices,
                0,
            );
        }
        encoder.end_encoding();
        if let Some(drawable) = drawable {
            command.present_drawable(drawable);
        }
        command.commit();
        command.wait_until_completed();
        ensure!(
            command.status() == MTLCommandBufferStatus::Completed,
            "Metal command failed with status {:?}",
            command.status()
        );
        Ok(())
    }

    pub fn capture(&mut self, scene: &Scene, width: u32, height: u32, path: &Path) -> Result<()> {
        let desc = TextureDescriptor::new();
        desc.set_texture_type(MTLTextureType::D2);
        desc.set_width(width as u64);
        desc.set_height(height as u64);
        desc.set_pixel_format(MTLPixelFormat::BGRA8Unorm_sRGB);
        desc.set_storage_mode(MTLStorageMode::Shared);
        desc.set_usage(MTLTextureUsage::RenderTarget);
        let target = self.device.new_texture(&desc);
        self.render(scene, &target, None)?;
        let mut pixels = vec![0_u8; width as usize * height as usize * 4];
        target.get_bytes(
            pixels.as_mut_ptr().cast(),
            width as u64 * 4,
            MTLRegion::new_2d(0, 0, width as u64, height as u64),
            0,
        );
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let mut png = png::Encoder::new(BufWriter::new(File::create(path)?), width, height);
        png.set_color(png::ColorType::Rgba);
        png.set_depth(png::BitDepth::Eight);
        png.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        png.write_header()?.write_image_data(&pixels)?;
        println!(
            "Saved {}x{} frame at t={:.2}s to {}",
            width,
            height,
            scene.time,
            path.display()
        );
        Ok(())
    }
}
