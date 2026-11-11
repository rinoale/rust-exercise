//! Direct Metal rendering: one exposed-face world mesh and a small HUD mesh.
use crate::voxel::{Block, Game, VoxelVertex};
use anyhow::{Context, Result, anyhow, ensure};
use bytemuck::{Pod, Zeroable};
use metal::*;
use std::{fs::File, io::BufWriter, path::Path};

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
        game: &Game,
        target: &TextureRef,
        drawable: Option<&MetalDrawableRef>,
        captured: bool,
    ) -> Result<()> {
        if self.revision != Some(game.world.revision) {
            let vertices = game.world.mesh();
            self.vertices = vertices.len() as u64;
            self.mesh = if vertices.is_empty() {
                None
            } else {
                Some(self.upload(&vertices))
            };
            self.revision = Some(game.world.revision);
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
                .player
                .view_projection(target.width() as f32 / target.height() as f32)
                .to_cols_array_2d(),
            eye: game.player.eye().extend(1.0).to_array(),
            selected: game
                .target()
                .map_or([0.0; 4], |hit| hit.cell.as_vec3().extend(1.0).to_array()),
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
        let hud = self.upload(&hud(
            game,
            target.width() as f32,
            target.height() as f32,
            captured,
        ));
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
    pub fn capture(&mut self, game: &Game, width: u32, height: u32, path: &Path) -> Result<()> {
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

struct Hud {
    vertices: Vec<VoxelVertex>,
    width: f32,
    height: f32,
    scale: f32,
}
impl Hud {
    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        for [u, v] in [[0., 0.], [1., 0.], [1., 1.], [0., 0.], [1., 1.], [0., 1.]] {
            self.vertices.push(VoxelVertex {
                position: [
                    (x + u * w) * self.scale / self.width * 2.0 - 1.0,
                    1.0 - (y + v * h) * self.scale / self.height * 2.0,
                    0.0,
                    1.0,
                ],
                normal: [0.0; 4],
                color,
                uv: [0.0; 2],
                padding: [0.0; 2],
            });
        }
    }
    fn text(&mut self, text: &str, x: f32, y: f32, color: [f32; 4]) {
        for (i, ch) in text.to_ascii_uppercase().chars().enumerate() {
            let rows = glyph(ch);
            for (row, bits) in rows.into_iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        self.rect(
                            x + i as f32 * 6.0 + col as f32,
                            y + row as f32,
                            1.0,
                            1.0,
                            color,
                        );
                    }
                }
            }
        }
    }
}
fn hud(game: &Game, width: f32, height: f32, captured: bool) -> Vec<VoxelVertex> {
    let scale = (width / 640.0).min(height / 400.0).max(0.5);
    let (w, h) = (width / scale, height / scale);
    let mut hud = Hud {
        vertices: Vec::new(),
        width,
        height,
        scale,
    };
    let white = [0.95, 0.97, 1.0, 1.0];
    let dark = [0.035, 0.05, 0.065, 1.0];
    let gold = [1.0, 0.7, 0.2, 1.0];
    hud.rect(10., 10., 332., 33., dark);
    hud.text("BLOCK WORLD", 18., 17., gold);
    hud.text("WASD MOVE   SPACE JUMP   MOUSE LOOK", 18., 30., white);
    hud.rect(10., h - 23., 370., 15., dark);
    hud.text(
        if captured {
            "LEFT DIG   RIGHT PLACE   1-4 SELECT   ESC RELEASE"
        } else {
            "CLICK TO PLAY   WASD MOVE   ESC RELEASE   Q QUIT"
        },
        18.,
        h - 19.,
        white,
    );
    for (slot, block) in Block::ALL.into_iter().enumerate() {
        let x = w * 0.5 - 140.0 + slot as f32 * 72.0;
        let selected = slot == game.selected;
        hud.rect(
            x,
            h - 88.,
            68.,
            55.,
            if selected {
                gold
            } else {
                [0.1, 0.13, 0.16, 1.0]
            },
        );
        hud.rect(x + 2., h - 86., 64., 51., dark);
        let c = block.color();
        hud.rect(x + 8., h - 78., 16., 16., [c[0], c[1], c[2], 1.0]);
        hud.text(&format!("{}", slot + 1), x + 51., h - 78., white);
        hud.text(
            block.name(),
            x + 8.,
            h - 57.,
            if selected { gold } else { white },
        );
        hud.text(&game.inventory[slot].to_string(), x + 8., h - 46., white);
    }
    hud.rect(w * 0.5 - 7., h * 0.5 - 2., 14., 4., dark);
    hud.rect(w * 0.5 - 2., h * 0.5 - 7., 4., 14., dark);
    hud.rect(w * 0.5 - 6., h * 0.5 - 0.5, 12., 1., white);
    hud.rect(w * 0.5 - 0.5, h * 0.5 - 6., 1., 12., white);
    if let Some(hit) = game.target() {
        let label = format!("{}  {:.1}M", hit.block.name(), hit.distance);
        hud.text(
            &label,
            w * 0.5 - label.len() as f32 * 3.0,
            h * 0.5 + 16.,
            white,
        );
    }
    hud.vertices
}
fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        _ => [0; 7],
    }
}
