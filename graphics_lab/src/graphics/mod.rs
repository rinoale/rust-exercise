//! Platform-independent graphics commands. Apps never handle native API objects.
use crate::voxel::VoxelVertex;
use anyhow::{Result, ensure};
use bytemuck::{Pod, Zeroable};
use std::{path::Path, sync::Arc};
use winit::window::Window;
mod wgpu_api;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Uniforms {
    pub view_projection: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
    pub tint: [f32; 4],
    pub eye: [f32; 4],
    pub options: [f32; 4],
    pub selected: [f32; 4],
}
#[derive(Clone, Copy)]
pub enum Shading {
    Cube,
    World,
    Hud,
}
pub struct DrawCall {
    pub vertices: Arc<[VoxelVertex]>,
    pub uniforms: Uniforms,
    pub shading: Shading,
    pub depth: bool,
    pub wireframe: bool,
}
pub struct Frame {
    pub clear: [f64; 4],
    pub draws: Vec<DrawCall>,
}
/// Rust's equivalent of an abstract graphics class. Implementations own devices,
/// swapchains, pipelines, readback synchronization, and platform presentation.
pub trait GraphicsApi {
    fn render(&mut self, frame: &Frame) -> Result<()>;
    fn read_pixels(&mut self, frame: &Frame, width: u32, height: u32) -> Result<Vec<u8>>;
    fn size(&self) -> (u32, u32);
    fn name(&self) -> &str;
}
macro_rules! adapter {
    ($name:ident, $backends:expr) => {
        pub struct $name(wgpu_api::WgpuApi);
        impl $name {
            pub fn new(window: Option<Arc<Window>>) -> Result<Self> {
                Ok(Self(wgpu_api::WgpuApi::new(window, $backends)?))
            }
        }
        impl GraphicsApi for $name {
            fn render(&mut self, frame: &Frame) -> Result<()> {
                self.0.render(frame)
            }
            fn read_pixels(&mut self, frame: &Frame, width: u32, height: u32) -> Result<Vec<u8>> {
                self.0.read_pixels(frame, width, height)
            }
            fn size(&self) -> (u32, u32) {
                self.0.size()
            }
            fn name(&self) -> &str {
                self.0.name()
            }
        }
    };
}
adapter!(MacAdapter, wgpu::Backends::METAL);
adapter!(WindowsAdapter, wgpu::Backends::DX12);
adapter!(LinuxAdapter, wgpu::Backends::VULKAN | wgpu::Backends::GL);

pub fn create(window: Option<Arc<Window>>) -> Result<Box<dyn GraphicsApi>> {
    #[cfg(target_os = "macos")]
    let api = MacAdapter::new(window)?;
    #[cfg(target_os = "windows")]
    let api = WindowsAdapter::new(window)?;
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let api = LinuxAdapter::new(window)?;
    Ok(Box::new(api))
}
pub fn validate_size(width: u32, height: u32) -> Result<()> {
    ensure!(
        (1..=4096).contains(&width) && (1..=4096).contains(&height),
        "Image size must be 1..=4096"
    );
    Ok(())
}
pub fn save_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<()> {
    validate_size(width, height)?;
    ensure!(
        pixels.len() == width as usize * height as usize * 4,
        "Invalid RGBA image size"
    );
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(std::fs::File::create(path)?),
        width,
        height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    encoder.write_header()?.write_image_data(pixels)?;
    println!("Saved {}", path.display());
    Ok(())
}

/// WSLg's Wayland compositor can disconnect with small software-rendered
/// windows. Use its X11 server on WSL; other desktops keep winit's default.
pub fn event_loop() -> Result<winit::event_loop::EventLoop<()>> {
    let mut builder = winit::event_loop::EventLoop::builder();
    #[cfg(target_os = "linux")]
    {
        use winit::platform::{wayland::EventLoopBuilderExtWayland, x11::EventLoopBuilderExtX11};
        match std::env::var("GRAPHICS_WINDOW_SYSTEM").as_deref() {
            Ok("x11") => {
                builder.with_x11();
            }
            Ok("wayland") => {
                builder.with_wayland();
            }
            Ok(_) => anyhow::bail!("GRAPHICS_WINDOW_SYSTEM must be x11 or wayland"),
            Err(_)
                if std::env::var_os("WSL_DISTRO_NAME").is_some()
                    && std::env::var_os("DISPLAY").is_some() =>
            {
                builder.with_x11();
            }
            Err(_) => (),
        }
    }
    Ok(builder.build()?)
}
