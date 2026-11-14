//! Portable single-cube rendering for the platform-specific research demos.
//! This submits real GPU draw calls; it does not emulate Apple assembly or AGX.
use crate::{
    graphics::{self, DrawCall, Frame, GraphicsApi, Shading, Uniforms},
    scene,
    voxel::VoxelVertex,
};
use anyhow::{Result, ensure};
use clap::Parser;
use glam::{Mat4, Vec3};
use std::{path::PathBuf, sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[derive(Parser)]
#[command(
    about = "Portable GPU cube (native Apple assembly/AGX execution is unavailable on this platform)"
)]
struct Options {
    #[arg(long)]
    capture: Option<PathBuf>,
    #[arg(long,default_value_t=600,value_parser=clap::value_parser!(u32).range(64..=4096))]
    size: u32,
    #[arg(long, default_value_t = 0.65)]
    angle: f32,
    #[arg(long,value_parser=clap::value_parser!(u64).range(1..))]
    frames: Option<u64>,
    /// Verify GPU image rotation and readback without opening a window.
    #[arg(long)]
    self_test: bool,
}
pub fn frame(angle: f32, tilt: f32, aspect: f32) -> Frame {
    let mesh = scene::cube();
    let colors = [
        [0.9, 0.25, 0.15, 1.],
        [0.15, 0.65, 0.9, 1.],
        [0.2, 0.85, 0.35, 1.],
        [0.9, 0.65, 0.15, 1.],
        [0.3, 0.4, 1., 1.],
        [0.8, 0.2, 0.8, 1.],
    ];
    let vertices = mesh
        .indices
        .iter()
        .map(|&i| {
            let v = mesh.vertices[i as usize];
            VoxelVertex {
                position: v.position,
                normal: v.normal,
                color: colors[i as usize / 4],
                uv: v.uv,
                padding: [0.; 2],
            }
        })
        .collect();
    let eye = Vec3::new(0., 1.8, 5.5);
    let uniforms = Uniforms {
        view_projection: (Mat4::perspective_rh(50f32.to_radians(), aspect, 0.1, 100.)
            * Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y))
        .to_cols_array_2d(),
        model: (Mat4::from_rotation_y(angle) * Mat4::from_rotation_x(tilt)).to_cols_array_2d(),
        tint: [1.; 4],
        eye: eye.extend(1.).to_array(),
        options: [0.; 4],
        selected: [0.; 4],
    };
    Frame {
        clear: scene::CLEAR_COLOR,
        draws: vec![DrawCall {
            vertices,
            uniforms,
            shading: Shading::Cube,
            depth: true,
            wireframe: false,
        }],
    }
}
pub fn run() -> Result<()> {
    let options = Options::parse();
    ensure!(options.angle.is_finite(), "--angle must be finite");
    println!(
        "Portable GPU mode: shared graphics adapter; no Apple assembly or private AGX execution"
    );
    if options.capture.is_some() || options.self_test {
        let mut api = graphics::create(None)?;
        if options.self_test {
            let a = api.read_pixels(&frame(0.5, 0., 1.), 128, 128)?;
            let b = api.read_pixels(&frame(1.2, 0., 1.), 128, 128)?;
            ensure!(
                a != b && a.chunks_exact(4).any(|p| p != &a[..4]),
                "GPU cube self-test failed"
            );
            println!("Portable GPU image rotation/readback self-test passed");
        }
        if let Some(path) = options.capture {
            let pixels =
                api.read_pixels(&frame(options.angle, 0., 1.), options.size, options.size)?;
            graphics::save_png(&path, options.size, options.size, &pixels)?;
        }
        return Ok(());
    }
    let event_loop = crate::graphics::event_loop()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        angle: options.angle,
        tilt: 0.,
        options,
        window: None,
        api: None,
        last: Instant::now(),
        paused: false,
        frames: 0,
        error: None,
    };
    event_loop.run_app(&mut app)?;
    if let Some(e) = app.error {
        return Err(e);
    }
    Ok(())
}
struct App {
    options: Options,
    window: Option<Arc<Window>>,
    api: Option<Box<dyn GraphicsApi>>,
    last: Instant,
    angle: f32,
    tilt: f32,
    paused: bool,
    frames: u64,
    error: Option<anyhow::Error>,
}
impl App {
    fn fail(&mut self, e: anyhow::Error, el: &ActiveEventLoop) {
        self.error = Some(e);
        el.exit();
    }
    fn open(&mut self, el: &ActiveEventLoop) -> Result<()> {
        let w = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_title("Portable GPU cube | Space pause | arrows rotate | R reset")
                    .with_inner_size(PhysicalSize::new(self.options.size, self.options.size)),
            )?,
        );
        self.api = Some(graphics::create(Some(w.clone()))?);
        self.window = Some(w);
        Ok(())
    }
    fn draw(&mut self) -> Result<()> {
        let now = Instant::now();
        if !self.paused {
            self.angle += now.duration_since(self.last).as_secs_f32().min(0.1) * 0.7;
        }
        self.last = now;
        if let Some(api) = &mut self.api {
            let (w, h) = api.size();
            if w == 0 || h == 0 {
                return Ok(());
            }
            api.render(&frame(self.angle, self.tilt, w as f32 / h as f32))?;
            self.frames += 1;
        }
        Ok(())
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_none()
            && let Err(e) = self.open(el)
        {
            self.fail(e, el);
        }
    }
    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::Escape) => el.exit(),
                    PhysicalKey::Code(KeyCode::Space) if !event.repeat => {
                        self.paused = !self.paused
                    }
                    PhysicalKey::Code(KeyCode::ArrowLeft) => self.angle -= 0.1,
                    PhysicalKey::Code(KeyCode::ArrowRight) => self.angle += 0.1,
                    PhysicalKey::Code(KeyCode::ArrowUp) => self.tilt += 0.1,
                    PhysicalKey::Code(KeyCode::ArrowDown) => self.tilt -= 0.1,
                    PhysicalKey::Code(KeyCode::KeyR) => {
                        self.angle = self.options.angle;
                        self.tilt = 0.;
                    }
                    _ => (),
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.draw() {
                    self.fail(e, el);
                }
                if self.options.frames.is_some_and(|n| self.frames >= n) {
                    println!("Presented {} portable GPU frames", self.frames);
                    el.exit();
                }
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}
