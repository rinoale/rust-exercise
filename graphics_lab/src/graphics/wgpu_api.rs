use super::*;
use anyhow::{Context, bail};
use wgpu::util::DeviceExt;

pub(super) struct WgpuApi {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'static>>,
    window: Option<Arc<Window>>,
    config: Option<wgpu::SurfaceConfiguration>,
    layout: wgpu::BindGroupLayout,
    shader: wgpu::ShaderModule,
    checker: wgpu::TextureView,
    sampler: wgpu::Sampler,
    pipelines:
        std::collections::HashMap<(wgpu::TextureFormat, u8, bool, bool), wgpu::RenderPipeline>,
    name: String,
    meshes: Vec<CachedMesh>,
}
struct CachedMesh {
    vertices: std::sync::Weak<[VoxelVertex]>,
    wireframe: bool,
    buffer: wgpu::Buffer,
    count: u32,
}
impl WgpuApi {
    pub fn new(window: Option<Arc<Window>>, mut backends: wgpu::Backends) -> Result<Self> {
        if let Ok(value) = std::env::var("GRAPHICS_BACKEND") {
            let requested = match value.as_str() {
                "metal" => wgpu::Backends::METAL,
                "dx12" => wgpu::Backends::DX12,
                "vulkan" => wgpu::Backends::VULKAN,
                "gl" => wgpu::Backends::GL,
                _ => bail!("GRAPHICS_BACKEND must be metal, dx12, vulkan, or gl"),
            };
            ensure!(
                backends.contains(requested),
                "Backend {value} is unavailable on this platform"
            );
            backends = requested;
        }
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let surface = window
            .clone()
            .map(|w| instance.create_surface(w))
            .transpose()?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: surface.as_ref(),
            force_fallback_adapter: false,
        }))
        .context("No compatible GPU adapter (software Vulkan is also supported)")?;
        let info = adapter.get_info();
        let name = format!("{:?}: {} ({:?})", info.backend, info.name, info.device_type);
        println!("Graphics adapter: {name}");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("graphics lab"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                ..Default::default()
            }))?;
        let config = surface
            .as_ref()
            .map(|s| {
                let size = window.as_ref().unwrap().inner_size();
                let mut c = s
                    .get_default_config(&adapter, size.width.max(1), size.height.max(1))
                    .context("Surface has no configuration")?;
                let caps = s.get_capabilities(&adapter);
                c.format = caps
                    .formats
                    .iter()
                    .copied()
                    .find(|f| f.is_srgb())
                    .context("Surface requires an sRGB format")?;
                s.configure(&device, &c);
                Ok::<_, anyhow::Error>(c)
            })
            .transpose()?;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("checker mipmaps"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 7,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for level in 0..7 {
            let size = 64 >> level;
            let mut pixels = Vec::new();
            for y in 0..size {
                for x in 0..size {
                    // Average in linear space once the checker cells become subpixel.
                    let shade = if level >= 5 {
                        190
                    } else if ((x << level) / 16 + (y << level) / 16) % 2 == 0 {
                        245
                    } else {
                        100
                    };
                    pixels.extend([shade, shade, shade, 255]);
                }
            }
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size * 4),
                    rows_per_image: Some(size),
                },
                wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
            );
        }
        let checker = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shared 3D shaders"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        Ok(Self {
            device,
            queue,
            surface,
            window,
            config,
            layout,
            shader,
            checker,
            sampler,
            pipelines: Default::default(),
            name,
            meshes: Vec::new(),
        })
    }
    fn pipeline(&mut self, format: wgpu::TextureFormat, draw: &DrawCall) {
        let key = (format, draw.shading as u8, draw.depth, draw.wireframe);
        if self.pipelines.contains_key(&key) {
            return;
        }
        let layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&self.layout],
                push_constant_ranges: &[],
            });
        let attrs = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x4,3=>Float32x2,4=>Float32x2];
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vertex"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &attrs,
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some(match draw.shading {
                        Shading::Cube => "cube",
                        Shading::World => "world",
                        Shading::Hud => "hud",
                    }),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: if draw.wireframe {
                        wgpu::PrimitiveTopology::LineList
                    } else {
                        wgpu::PrimitiveTopology::TriangleList
                    },
                    cull_mode: if matches!(draw.shading, Shading::Hud) || draw.wireframe {
                        None
                    } else {
                        Some(wgpu::Face::Back)
                    },
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: draw.depth,
                    depth_compare: if draw.depth {
                        wgpu::CompareFunction::Less
                    } else {
                        wgpu::CompareFunction::Always
                    },
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            });
        self.pipelines.insert(key, pipeline);
    }
    fn texture(
        &self,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    }
    fn encode(
        &mut self,
        frame: &Frame,
        target: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> wgpu::CommandEncoder {
        let depth = self
            .texture(
                width,
                height,
                wgpu::TextureFormat::Depth32Float,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            )
            .create_view(&Default::default());
        self.meshes.retain(|mesh| mesh.vertices.strong_count() > 0);
        let mut resources = Vec::new();
        for draw in &frame.draws {
            self.pipeline(format, draw);
            let cached = self.meshes.iter().find(|mesh| {
                mesh.wireframe == draw.wireframe
                    && mesh
                        .vertices
                        .upgrade()
                        .is_some_and(|v| Arc::ptr_eq(&v, &draw.vertices))
            });
            let (buffer, count) = if let Some(mesh) = cached {
                (mesh.buffer.clone(), mesh.count)
            } else {
                let lines: Vec<_> = if draw.wireframe {
                    draw.vertices
                        .chunks_exact(3)
                        .flat_map(|t| [t[0], t[1], t[1], t[2], t[2], t[0]])
                        .collect()
                } else {
                    Vec::new()
                };
                let vertices: &[VoxelVertex] = if draw.wireframe {
                    &lines
                } else {
                    &draw.vertices
                };
                let buffer = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                let count = vertices.len() as u32;
                self.meshes.push(CachedMesh {
                    vertices: Arc::downgrade(&draw.vertices),
                    wireframe: draw.wireframe,
                    buffer: buffer.clone(),
                    count,
                });
                (buffer, count)
            };
            let uniform = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::bytes_of(&draw.uniforms),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
            let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&self.checker),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            resources.push((buffer, bind, count));
        }
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: frame.clear[0],
                            g: frame.clear[1],
                            b: frame.clear[2],
                            a: frame.clear[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            for (draw, (buffer, bind, count)) in frame.draws.iter().zip(&resources) {
                if *count == 0 {
                    continue;
                }
                pass.set_pipeline(
                    &self.pipelines[&(format, draw.shading as u8, draw.depth, draw.wireframe)],
                );
                pass.set_bind_group(0, bind, &[]);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..*count, 0..1);
            }
        }
        encoder
    }
}
impl GraphicsApi for WgpuApi {
    fn name(&self) -> &str {
        &self.name
    }
    fn size(&self) -> (u32, u32) {
        self.window.as_ref().map_or((1, 1), |w| {
            let s = w.inner_size();
            (s.width, s.height)
        })
    }
    fn render(&mut self, frame: &Frame) -> Result<()> {
        let (width, height) = self.size();
        if width == 0 || height == 0 {
            return Ok(());
        }
        let surface = self
            .surface
            .as_ref()
            .context("Offscreen adapter cannot present")?;
        let config = self.config.as_mut().unwrap();
        if config.width != width || config.height != height {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
        }
        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                surface.configure(&self.device, config);
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        let format = config.format;
        let encoder = self.encode(
            frame,
            &output.texture.create_view(&Default::default()),
            format,
            width,
            height,
        );
        self.queue.submit([encoder.finish()]);
        self.window.as_ref().unwrap().pre_present_notify();
        output.present();
        Ok(())
    }
    fn read_pixels(&mut self, frame: &Frame, width: u32, height: u32) -> Result<Vec<u8>> {
        validate_size(width, height)?;
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let target = self.texture(
            width,
            height,
            format,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let mut encoder = self.encode(
            frame,
            &target.create_view(&Default::default()),
            format,
            width,
            height,
        );
        let stride = (width * 4).div_ceil(256) * 256;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(stride) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::PollType::Wait)?;
        rx.recv()??;
        let mapped = buffer.slice(..).get_mapped_range();
        let pixels = mapped
            .chunks_exact(stride as usize)
            .flat_map(|row| row[..width as usize * 4].iter().copied())
            .collect();
        drop(mapped);
        buffer.unmap();
        Ok(pixels)
    }
}
