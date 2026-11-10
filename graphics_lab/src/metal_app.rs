use std::{path::PathBuf, time::Instant};

use anyhow::{Result, anyhow, ensure};
use clap::Parser;
use core_graphics_types::geometry::CGSize;
use metal::MetalLayer;
use objc::{
    msg_send,
    rc::autoreleasepool,
    runtime::{Object, YES},
    sel, sel_impl,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowId},
};

use crate::{metal_renderer::MetalRenderer, scene::Scene};

#[derive(Parser)]
#[command(
    about = "Native Metal: two checkerboard cubes, perspective, lighting, and a depth-tested floor"
)]
struct Options {
    /// Render a deterministic PNG offscreen, without opening a window.
    #[arg(long)]
    capture: Option<PathBuf>,
    #[arg(long, default_value_t = 1100, value_parser = clap::value_parser!(u32).range(64..=4096))]
    width: u32,
    #[arg(long, default_value_t = 760, value_parser = clap::value_parser!(u32).range(64..=4096))]
    height: u32,
    /// Initial animation time in seconds (also used by offscreen capture).
    #[arg(long, default_value_t = 1.5)]
    time: f32,
    /// Close after this many presented frames, for smoke tests.
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
    frames: Option<u64>,
    #[arg(long)]
    wireframe: bool,
    #[arg(long)]
    no_depth: bool,
    #[arg(long)]
    no_texture: bool,
}

pub fn run() -> Result<()> {
    let options = Options::parse();
    ensure!(options.time.is_finite(), "--time must be finite");
    let scene = Scene {
        time: options.time,
        wireframe: options.wireframe,
        depth_test: !options.no_depth,
        textured: !options.no_texture,
        ..Scene::default()
    };
    if let Some(path) = &options.capture {
        return autoreleasepool(|| {
            MetalRenderer::new()?.capture(&scene, options.width, options.height, path)
        });
    }
    println!(
        "Space pause | W wireframe | D depth | T texture | arrows orbit | +/- zoom | R reset | Esc quit"
    );
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        options,
        scene,
        surface: None,
        last_frame: Instant::now(),
        stats_start: Instant::now(),
        total_frames: 0,
        stats_frames: 0,
        frame_ms: 0.0,
        error: None,
    };
    event_loop.run_app(&mut app)?;
    if let Some(error) = app.error {
        return Err(error);
    }
    Ok(())
}

// Fields drop in declaration order: detach the renderer/layer before the window.
struct Surface {
    renderer: MetalRenderer,
    layer: MetalLayer,
    window: Window,
}

impl Surface {
    fn new(event_loop: &ActiveEventLoop, width: u32, height: u32) -> Result<Self> {
        let window = event_loop.create_window(
            Window::default_attributes()
                .with_title("Metal / 3D graphics lab")
                .with_inner_size(PhysicalSize::new(width, height)),
        )?;
        let renderer = MetalRenderer::new()?;
        let mut layer = MetalLayer::new();
        layer.set_device(&renderer.device);
        layer.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm_sRGB);
        layer.set_presents_with_transaction(false);
        layer.set_display_sync_enabled(true);
        layer.set_opaque(true);
        layer.set_contents_scale(window.scale_factor());
        let RawWindowHandle::AppKit(handle) = window.window_handle()?.as_raw() else {
            return Err(anyhow!("Expected an AppKit window"));
        };
        // winit supplies a live NSView on the main thread. AppKit retains its layer.
        unsafe {
            let view = handle.ns_view.as_ptr().cast::<Object>();
            let _: () = msg_send![view, setWantsLayer: YES];
            let layer_pointer: *mut Object = (&mut *layer as *mut metal::MetalLayerRef).cast();
            let _: () = msg_send![view, setLayer: layer_pointer];
        }
        let size = window.inner_size();
        layer.set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
        Ok(Self {
            renderer,
            layer,
            window,
        })
    }
}

struct App {
    options: Options,
    scene: Scene,
    surface: Option<Surface>,
    last_frame: Instant,
    stats_start: Instant,
    total_frames: u64,
    stats_frames: u32,
    frame_ms: f64,
    error: Option<anyhow::Error>,
}

impl App {
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        self.error = Some(error);
        event_loop.exit();
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let now = Instant::now();
        self.scene
            .tick(now.duration_since(self.last_frame).as_secs_f32().min(0.1));
        self.last_frame = now;
        let Some(surface) = &mut self.surface else {
            return Ok(());
        };
        let size = surface.window.inner_size();
        if size.width == 0 || size.height == 0 || surface.window.is_minimized() == Some(true) {
            return Ok(());
        }
        let Some(drawable) = surface.layer.next_drawable() else {
            return Ok(());
        };
        let start = Instant::now();
        surface.window.pre_present_notify();
        surface
            .renderer
            .render(&self.scene, drawable.texture(), Some(drawable))?;
        self.frame_ms += start.elapsed().as_secs_f64() * 1000.0;
        self.total_frames += 1;
        self.stats_frames += 1;
        if self.stats_start.elapsed().as_secs_f64() >= 0.5 {
            let fps = self.stats_frames as f64 / self.stats_start.elapsed().as_secs_f64();
            surface.window.set_title(&format!(
                "Metal | {fps:.0} fps | {:.2} ms submit+wait | depth {} | texture {} | W wireframe · Space pause",
                self.frame_ms / self.stats_frames as f64,
                if self.scene.depth_test { "on" } else { "off" },
                if self.scene.textured { "on" } else { "off" },
            ));
            self.stats_start = Instant::now();
            self.stats_frames = 0;
            self.frame_ms = 0.0;
        }
        if self.options.frames.is_some_and(|n| self.total_frames >= n) {
            println!("Presented {} frames successfully", self.total_frames);
            event_loop.exit();
        }
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.surface.is_none() {
            match autoreleasepool(|| {
                Surface::new(event_loop, self.options.width, self.options.height)
            }) {
                Ok(surface) => {
                    self.surface = Some(surface);
                    self.last_frame = Instant::now();
                }
                Err(error) => self.fail(event_loop, error),
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(surface) = &self.surface {
                    surface
                        .layer
                        .set_drawable_size(CGSize::new(size.width as f64, size.height as f64));
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(surface) = &self.surface {
                    surface.layer.set_contents_scale(scale_factor);
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match key {
                        KeyCode::Escape => event_loop.exit(),
                        KeyCode::Space if !event.repeat => self.scene.paused = !self.scene.paused,
                        KeyCode::KeyW if !event.repeat => {
                            self.scene.wireframe = !self.scene.wireframe
                        }
                        KeyCode::KeyD if !event.repeat => {
                            self.scene.depth_test = !self.scene.depth_test
                        }
                        KeyCode::KeyT if !event.repeat => {
                            self.scene.textured = !self.scene.textured
                        }
                        KeyCode::KeyR => self.scene = Scene::default(),
                        KeyCode::ArrowLeft => self.scene.yaw -= 0.08,
                        KeyCode::ArrowRight => self.scene.yaw += 0.08,
                        KeyCode::ArrowUp => self.scene.pitch = (self.scene.pitch + 0.05).min(1.4),
                        KeyCode::ArrowDown => {
                            self.scene.pitch = (self.scene.pitch - 0.05).max(0.05)
                        }
                        KeyCode::Equal | KeyCode::NumpadAdd => {
                            self.scene.distance = (self.scene.distance - 0.3).max(4.0)
                        }
                        KeyCode::Minus | KeyCode::NumpadSubtract => {
                            self.scene.distance = (self.scene.distance + 0.3).min(25.0)
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = autoreleasepool(|| self.redraw(event_loop)) {
                    self.fail(event_loop, error);
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(surface) = &self.surface {
            surface.window.request_redraw();
        }
    }
}
