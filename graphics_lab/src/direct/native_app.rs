//! Native GPU image presentation through CoreGraphics (softbuffer on macOS).
use super::{g17, probe, trace};
use anyhow::{Context, Result, anyhow, ensure};
use clap::Parser;
use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[derive(Parser)]
#[command(about = "Render a cube using native M5 G17 machine code and IOGPU, without Metal APIs")]
struct Options {
    /// Only discover/open the driver; no rendering in this diagnostic mode.
    #[arg(long, conflicts_with_all=["trace","capture","self_test"])]
    probe: bool,
    #[arg(long, requires = "probe")]
    report: Option<PathBuf>,
    /// Trace a separate Metal reference process for protocol research.
    #[arg(long, conflicts_with_all=["capture","self_test"])]
    trace: Option<PathBuf>,
    /// Save the GPU-rendered image as PNG, without opening a window.
    #[arg(long)]
    capture: Option<PathBuf>,
    /// Native image resolution (128 or 256); window presentation scales the image.
    #[arg(long, default_value_t = 256)]
    size: u32,
    #[arg(long, default_value_t = 0.65)]
    angle: f32,
    #[arg(long, default_value_t = 0.35)]
    tilt: f32,
    /// Render N successive angles in the window and then close (smoke test).
    #[arg(long, value_parser=clap::value_parser!(u32).range(1..=300))]
    frames: Option<u32>,
    /// Run the real GPU's 1024-lane arithmetic validation, then exit.
    #[arg(long)]
    self_test: bool,
    #[arg(long, hide = true)]
    worker: bool,
}

pub fn run() -> Result<()> {
    let options = Options::parse();
    if options.worker {
        return worker_loop();
    }
    if let Some(path) = &options.trace {
        return trace::record(path);
    }
    if options.probe {
        let (report, device) = probe()?;
        let json = serde_json::to_string_pretty(&report)?;
        println!("{json}");
        if let Some(path) = &options.report {
            make_parent(path)?;
            std::fs::write(path, format!("{json}\n"))?;
        }
        ensure!(
            device.is_some(),
            "AGX connection failed: {}",
            report.open_message
        );
        return Ok(());
    }
    ensure!(
        matches!(options.size, 128 | 256),
        "--size must be 128 or 256"
    );
    ensure!(
        options.angle.is_finite() && options.tilt.is_finite(),
        "Angles must be finite"
    );
    if options.self_test || options.capture.is_some() {
        let mut renderer = objc::rc::autoreleasepool(g17::Renderer::new)?;
        println!(
            "Native G17 GPU arithmetic: 1024/1024 values passed; AGXMetal rendering driver absent"
        );
        if options.self_test {
            return Ok(());
        }
        let (pixels, receipt) = renderer.render(options.size, options.angle, options.tilt)?;
        let path = options.capture.as_ref().unwrap();
        save_png(path, options.size, &pixels)?;
        let report_path = path.with_extension("json");
        std::fs::write(
            &report_path,
            format!("{}\n", serde_json::to_string_pretty(&receipt)?),
        )?;
        println!(
            "Native cube: {}\nExecution receipt: {}",
            path.display(),
            report_path.display()
        );
        return Ok(());
    }
    // AppKit/CoreGraphics may use Metal for macOS window presentation. Keep all
    // cube computation in a separate worker which rejects an AGXMetal image.
    let mut renderer = RenderClient::new()?;
    let (pixels, receipt) = renderer.render(options.size, options.angle, options.tilt)?;
    println!(
        "Native worker: 1024/1024 arithmetic values passed; GPU rendered {} pixels in {:.1} ms",
        pixels.len(),
        receipt.elapsed_ms
    );
    println!("Left/right rotate | Up/down tilt | R reset | Esc quit");
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        options,
        renderer,
        pixels,
        surface: None,
        window: None,
        context: None,
        drawn: 0,
        error: None,
    };
    event_loop.run_app(&mut app)?;
    if let Some(error) = app.error {
        return Err(error);
    }
    Ok(())
}

fn make_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn save_png(path: &Path, size: u32, pixels: &[u32]) -> Result<()> {
    make_parent(path)?;
    let mut rgba = Vec::with_capacity(pixels.len() * 4);
    for p in pixels {
        rgba.extend([(p >> 16) as u8, (p >> 8) as u8, *p as u8, 255]);
    }
    let mut encoder = png::Encoder::new(BufWriter::new(File::create(path)?), size, size);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    encoder.write_header()?.write_image_data(&rgba)?;
    Ok(())
}

struct App {
    options: Options,
    renderer: RenderClient,
    pixels: Vec<u32>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    context: Option<softbuffer::Context<Arc<Window>>>,
    window: Option<Arc<Window>>,
    drawn: u32,
    error: Option<anyhow::Error>,
}

const WORKER_MAGIC: u32 = 0x47524331;

struct RenderClient {
    child: Child,
    input: ChildStdin,
    output: ChildStdout,
}

impl RenderClient {
    fn new() -> Result<Self> {
        let mut child = Command::new(std::env::current_exe()?)
            .arg("--worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let input = child.stdin.take().context("Native worker stdin absent")?;
        let output = child.stdout.take().context("Native worker stdout absent")?;
        let mut client = Self {
            child,
            input,
            output,
        };
        let mut hello = [0_u8; 4];
        client
            .output
            .read_exact(&mut hello)
            .context("Native worker failed during GPU initialization")?;
        ensure!(
            u32::from_le_bytes(hello) == WORKER_MAGIC,
            "Native worker initialization protocol mismatch"
        );
        Ok(client)
    }

    fn render(&mut self, size: u32, angle: f32, tilt: f32) -> Result<(Vec<u32>, g17::Receipt)> {
        for word in [WORKER_MAGIC, size, angle.to_bits(), tilt.to_bits()] {
            self.input.write_all(&word.to_le_bytes())?;
        }
        self.input.flush()?;
        let mut lengths = [0_u8; 8];
        self.output
            .read_exact(&mut lengths)
            .context("Native GPU worker did not return an image")?;
        let json_size = u32::from_le_bytes(lengths[..4].try_into().unwrap()) as usize;
        let pixel_size = u32::from_le_bytes(lengths[4..].try_into().unwrap()) as usize;
        ensure!(
            json_size <= 16384 && pixel_size == size as usize * size as usize * 4,
            "Invalid native worker reply size"
        );
        let mut json = vec![0; json_size];
        self.output.read_exact(&mut json)?;
        let receipt: g17::Receipt = serde_json::from_slice(&json)?;
        ensure!(
            receipt.arithmetic_self_test_passed
                && !receipt
                    .loaded_metal_images
                    .iter()
                    .any(|p| p.contains("AGXMetal")),
            "Native worker did not preserve independent execution"
        );
        let mut bytes = vec![0_u8; pixel_size];
        self.output.read_exact(&mut bytes)?;
        let pixels = bytes
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        Ok((pixels, receipt))
    }
}

impl Drop for RenderClient {
    fn drop(&mut self) {
        let _ = self.input.write_all(&[0; 16]);
        let _ = self.input.flush();
        let deadline = Instant::now() + Duration::from_secs(1);
        while Instant::now() < deadline {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn worker_loop() -> Result<()> {
    let mut renderer = objc::rc::autoreleasepool(g17::Renderer::new)?;
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    output.write_all(&WORKER_MAGIC.to_le_bytes())?;
    output.flush()?;
    loop {
        let mut request = [0_u8; 16];
        if input.read_exact(&mut request).is_err() {
            break;
        }
        let words: Vec<_> = request
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        if words[0] == 0 {
            break;
        }
        ensure!(words[0] == WORKER_MAGIC, "Invalid native worker request");
        let angle = f32::from_bits(words[2]);
        let tilt = f32::from_bits(words[3]);
        ensure!(
            angle.is_finite() && tilt.is_finite(),
            "Invalid native camera"
        );
        let (pixels, receipt) =
            objc::rc::autoreleasepool(|| renderer.render(words[1], angle, tilt))?;
        let json = serde_json::to_vec(&receipt)?;
        output.write_all(&(json.len() as u32).to_le_bytes())?;
        output.write_all(&((pixels.len() * 4) as u32).to_le_bytes())?;
        output.write_all(&json)?;
        output.write_all(bytemuck::cast_slice(&pixels))?;
        output.flush()?;
    }
    Ok(())
}

impl App {
    fn render(&mut self) -> Result<()> {
        let (pixels, receipt) =
            self.renderer
                .render(self.options.size, self.options.angle, self.options.tilt)?;
        self.pixels = pixels;
        if let Some(window) = &self.window {
            window.set_title(&format!(
                "Native G17 GPU cube | {:.1} ms | Arrows rotate · Esc quit",
                receipt.elapsed_ms
            ));
            window.request_redraw();
        }
        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        let window = self.window.as_ref().context("Native window is absent")?;
        let size = window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return Ok(());
        };
        let surface = self
            .surface
            .as_mut()
            .context("Presentation surface is absent")?;
        surface
            .resize(w, h)
            .map_err(|e| anyhow!("CoreGraphics resize: {e}"))?;
        let mut buffer = surface
            .buffer_mut()
            .map_err(|e| anyhow!("CoreGraphics buffer: {e}"))?;
        // This only copies/scales finished GPU pixels; it does no 3D rendering.
        let extent = size.width.min(size.height) as usize;
        let left = (size.width as usize - extent) / 2;
        let top = (size.height as usize - extent) / 2;
        for y in 0..size.height as usize {
            for x in 0..size.width as usize {
                buffer[y * size.width as usize + x] =
                    if (left..left + extent).contains(&x) && (top..top + extent).contains(&y) {
                        let sx = (x - left) * self.options.size as usize / extent;
                        let sy = (y - top) * self.options.size as usize / extent;
                        self.pixels[sy * self.options.size as usize + sx] & 0xffffff
                    } else {
                        0x142032
                    };
            }
        }
        window.pre_present_notify();
        buffer
            .present()
            .map_err(|e| anyhow!("CoreGraphics present: {e}"))?;
        self.drawn += 1;
        Ok(())
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        self.error = Some(error);
        event_loop.exit();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let result = (|| -> Result<()> {
            let window = Arc::new(
                event_loop.create_window(
                    Window::default_attributes()
                        .with_title("Native G17 GPU cube | Arrows rotate · Esc quit")
                        .with_inner_size(PhysicalSize::new(800, 800)),
                )?,
            );
            let context = softbuffer::Context::new(window.clone())
                .map_err(|e| anyhow!("CoreGraphics context: {e}"))?;
            let surface = softbuffer::Surface::new(&context, window.clone())
                .map_err(|e| anyhow!("CoreGraphics surface: {e}"))?;
            window.request_redraw();
            self.window = Some(window);
            self.context = Some(context);
            self.surface = Some(surface);
            Ok(())
        })();
        if let Err(error) = result {
            self.fail(event_loop, error);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match key {
                        KeyCode::Escape => {
                            event_loop.exit();
                            return;
                        }
                        KeyCode::ArrowLeft => self.options.angle -= 0.12,
                        KeyCode::ArrowRight => self.options.angle += 0.12,
                        KeyCode::ArrowUp => self.options.tilt = (self.options.tilt + 0.08).min(1.2),
                        KeyCode::ArrowDown => {
                            self.options.tilt = (self.options.tilt - 0.08).max(-1.2)
                        }
                        KeyCode::KeyR => {
                            self.options.angle = 0.65;
                            self.options.tilt = 0.35;
                        }
                        _ => return,
                    }
                    if let Err(error) = objc::rc::autoreleasepool(|| self.render()) {
                        self.fail(event_loop, error);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.present() {
                    self.fail(event_loop, error);
                    return;
                }
                if let Some(frames) = self.options.frames {
                    if self.drawn >= frames {
                        println!("Presented {} native GPU frames", self.drawn);
                        event_loop.exit();
                    } else {
                        self.options.angle += 0.12;
                        if let Err(error) = objc::rc::autoreleasepool(|| self.render()) {
                            self.fail(event_loop, error);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
