use std::{error::Error, fs::File, io::BufWriter, path::Path, time::Instant};

use metal_asm::cube::Cube;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowId},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn capture(path: &Path) -> Result<()> {
    let mut cube = Cube::offscreen()?;
    let mut pixels = cube.capture(800, 600, 0.65)?;
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2); // Metal BGRA -> PNG RGBA; no rendering happens here.
    }
    let mut encoder = png::Encoder::new(BufWriter::new(File::create(path)?), 800, 600);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&pixels)?;
    println!("Captured assembly-rendered Metal cube: {}", path.display());
    Ok(())
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut frames = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--capture" => {
                let path = args.next().ok_or("--capture requires a PNG path")?;
                if args.next().is_some() {
                    return Err("--capture must be used by itself".into());
                }
                return capture(Path::new(&path));
            }
            "--frames" => {
                let count = args
                    .next()
                    .ok_or("--frames requires a positive number")?
                    .parse::<u64>()?;
                if count == 0 {
                    return Err("--frames must be positive".into());
                }
                frames = Some(count);
            }
            "--help" | "-h" => {
                println!(
                    "cargo run -- [--frames N | --capture cube.png]\nSpace: pause/resume | Esc: quit"
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown argument: {arg}").into()),
        }
    }
    println!("Rust -> ARM64 assembly -> Metal render pipeline -> rotating 3D cube");
    println!("Space: pause/resume | Esc: quit");
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        surface: None,
        previous: Instant::now(),
        angle: 0.65,
        paused: false,
        frames: 0,
        limit: frames,
        error: None,
    };
    event_loop.run_app(&mut app)?;
    if let Some(error) = app.error {
        return Err(error);
    }
    Ok(())
}

// Drop in declaration order: assembly detaches its layer before NSView disappears.
struct Surface {
    cube: Cube,
    window: Window,
}

impl Surface {
    fn new(event_loop: &ActiveEventLoop) -> Result<Self> {
        let window = event_loop.create_window(
            Window::default_attributes()
                .with_title("Rust -> ARM64 assembly -> Metal | 3D cube")
                .with_inner_size(PhysicalSize::new(800, 600)),
        )?;
        let RawWindowHandle::AppKit(handle) = window.window_handle()?.as_raw() else {
            return Err("Expected an AppKit window".into());
        };
        // SAFETY: winit gives a live NSView on the main thread. This surface drops
        // Cube before Window; neither object is moved to another thread.
        let cube = unsafe { Cube::for_ns_view(handle.ns_view.as_ptr()) }?;
        Ok(Self { cube, window })
    }
}

struct App {
    surface: Option<Surface>,
    previous: Instant,
    angle: f32,
    paused: bool,
    frames: u64,
    limit: Option<u64>,
    error: Option<Box<dyn Error>>,
}

impl App {
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: Box<dyn Error>) {
        self.error = Some(error);
        event_loop.exit();
    }

    fn redraw(&mut self) -> Result<()> {
        let now = Instant::now();
        if !self.paused {
            self.angle += now.duration_since(self.previous).as_secs_f32().min(0.1) * 0.7;
        }
        self.previous = now;
        if let Some(surface) = &mut self.surface {
            let size = surface.window.inner_size();
            if size.width == 0 || size.height == 0 || surface.window.is_minimized() == Some(true) {
                return Ok(());
            }
            surface.window.pre_present_notify();
            surface.cube.render(size.width, size.height, self.angle)?;
            self.frames += 1;
        }
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.surface.is_none() {
            match Surface::new(event_loop) {
                Ok(surface) => {
                    self.surface = Some(surface);
                    self.previous = Instant::now();
                }
                Err(error) => self.fail(event_loop, error),
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                    PhysicalKey::Code(KeyCode::Space) if !event.repeat => {
                        self.paused = !self.paused
                    }
                    _ => (),
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.redraw() {
                    self.fail(event_loop, error);
                }
                if self.limit.is_some_and(|limit| self.frames >= limit) {
                    println!("Presented {} frames through assembly/Metal", self.frames);
                    event_loop.exit();
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

    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.surface = None;
    }
}
