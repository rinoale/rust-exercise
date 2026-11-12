//! Native window and controls for the diagonal-world sandbox.
use crate::{
    diagonal::{Game, Shape},
    voxel::Input,
    voxel_renderer::VoxelRenderer,
};
use anyhow::{Result, anyhow};
use clap::Parser;
use core_graphics_types::geometry::CGSize;
use metal::MetalLayer;
use objc::{
    msg_send,
    rc::autoreleasepool,
    runtime::{Object, YES},
    sel, sel_impl,
};
use std::{collections::HashSet, path::PathBuf, time::Instant};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{CursorGrabMode, Window, WindowId},
};

#[derive(Parser)]
#[command(about = "A 5-12-13 world of cubes, boxes, and real diagonal ramps")]
struct Options {
    /// Render the starting world to a PNG without opening a window.
    #[arg(long)]
    capture: Option<PathBuf>,
    /// Start in an aerial view. Captures use this view unless --walk-view is set.
    #[arg(long)]
    overview: bool,
    #[arg(long)]
    walk_view: bool,
    #[arg(long,default_value_t=1100,value_parser=clap::value_parser!(u32).range(320..=4096))]
    width: u32,
    #[arg(long,default_value_t=760,value_parser=clap::value_parser!(u32).range(240..=4096))]
    height: u32,
    /// Exit after presenting this many frames (window smoke test).
    #[arg(long,value_parser=clap::value_parser!(u64).range(1..))]
    frames: Option<u64>,
}
pub fn run() -> Result<()> {
    let options = Options::parse();
    let mut game = Game::default();
    if options.overview || (options.capture.is_some() && !options.walk_view) {
        game.player.overview();
    }
    if let Some(path) = &options.capture {
        return autoreleasepool(|| {
            VoxelRenderer::new()?.capture(&game, options.width, options.height, path)
        });
    }
    println!(
        "Click to capture mouse | WASD walk | Mouse look | Space jump | Left dig | Right place | 1-6 / wheel select | R rotate | F fly | V overview | T respawn | Esc release | Q quit"
    );
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        options,
        game,
        surface: None,
        keys: HashSet::new(),
        captured: false,
        last_frame: Instant::now(),
        total_frames: 0,
        error: None,
    };
    event_loop.run_app(&mut app)?;
    if let Some(error) = app.error {
        return Err(error);
    }
    Ok(())
}
// Drop GPU resources before the NSView/window they are attached to.
struct Surface {
    renderer: VoxelRenderer,
    layer: MetalLayer,
    window: Window,
}
impl Surface {
    fn new(event_loop: &ActiveEventLoop, width: u32, height: u32) -> Result<Self> {
        let window = event_loop.create_window(
            Window::default_attributes()
                .with_title("Diagonal World | Click to play")
                .with_inner_size(PhysicalSize::new(width, height)),
        )?;
        let renderer = VoxelRenderer::new()?;
        let mut layer = MetalLayer::new();
        layer.set_device(&renderer.device);
        layer.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm_sRGB);
        layer.set_presents_with_transaction(false);
        layer.set_display_sync_enabled(true);
        layer.set_opaque(true);
        layer.set_contents_scale(window.scale_factor());
        let RawWindowHandle::AppKit(handle) = window.window_handle()?.as_raw() else {
            return Err(anyhow!("Expected AppKit window"));
        };
        // winit owns a live NSView on this main thread; AppKit retains the layer.
        unsafe {
            let view = handle.ns_view.as_ptr().cast::<Object>();
            let _: () = msg_send![view, setWantsLayer: YES];
            let pointer: *mut Object = (&mut *layer as *mut metal::MetalLayerRef).cast();
            let _: () = msg_send![view, setLayer: pointer];
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
    game: Game,
    surface: Option<Surface>,
    keys: HashSet<KeyCode>,
    captured: bool,
    last_frame: Instant,
    total_frames: u64,
    error: Option<anyhow::Error>,
}
impl App {
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        self.error = Some(error);
        event_loop.exit();
    }
    fn capture_mouse(&mut self, capture: bool) {
        let Some(surface) = &self.surface else {
            return;
        };
        if capture {
            if let Err(error) = surface
                .window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| surface.window.set_cursor_grab(CursorGrabMode::Confined))
            {
                eprintln!("Could not capture mouse: {error}");
                return;
            }
        } else {
            let _ = surface.window.set_cursor_grab(CursorGrabMode::None);
        }
        surface.window.set_cursor_visible(!capture);
        self.captured = capture;
        self.keys.clear();
        self.last_frame = Instant::now();
    }
    fn redraw(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().min(0.1);
        self.last_frame = now;
        if self.captured {
            let down = |key| f32::from(self.keys.contains(&key));
            let input = Input {
                forward: down(KeyCode::KeyW) - down(KeyCode::KeyS),
                right: down(KeyCode::KeyD) - down(KeyCode::KeyA),
            };
            self.game.player.tick(
                &self.game.world,
                input,
                down(KeyCode::Space) - down(KeyCode::ShiftLeft),
                self.keys.contains(&KeyCode::ControlLeft),
                dt,
            );
        }
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
        surface.window.pre_present_notify();
        surface.renderer.render(
            &self.game,
            drawable.texture(),
            Some(drawable),
            self.captured,
        )?;
        surface.window.set_title(&format!(
            "Diagonal World | {} | Rotation {} | Left remove / Right build | {}",
            Shape::ALL[self.game.selected].name(),
            self.game.rotation as u16 * 90,
            if self.captured {
                "Esc release"
            } else {
                "Click to play"
            }
        ));
        self.total_frames += 1;
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
                    surface.window.request_redraw();
                    self.surface = Some(surface);
                    self.last_frame = Instant::now();
                }
                Err(error) => self.fail(event_loop, error),
            }
        }
    }
    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if self.captured
            && let DeviceEvent::MouseMotion { delta } = event
        {
            self.game.player.look(delta.0, delta.1);
        }
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(false) => self.capture_mouse(false),
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
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    if event.state == ElementState::Released {
                        self.keys.remove(&key);
                    } else if !event.repeat {
                        match key {
                            KeyCode::Escape => self.capture_mouse(false),
                            KeyCode::KeyQ => event_loop.exit(),
                            _ if !self.captured => (),
                            KeyCode::Space => {
                                self.game.player.jump();
                                self.keys.insert(key);
                            }
                            KeyCode::KeyT => self.game.player.respawn(&self.game.world),
                            KeyCode::KeyR => self.game.rotation = (self.game.rotation + 1) % 4,
                            KeyCode::KeyV => self.game.player.overview(),
                            KeyCode::KeyF => {
                                self.game.player.flying = !self.game.player.flying;
                                self.game.player.velocity_y = 0.;
                                if !self.game.player.flying
                                    && self.game.world.collides(self.game.player.feet)
                                {
                                    self.game.player.respawn(&self.game.world);
                                }
                            }
                            KeyCode::Digit1 => self.game.selected = 0,
                            KeyCode::Digit2 => self.game.selected = 1,
                            KeyCode::Digit3 => self.game.selected = 2,
                            KeyCode::Digit4 => self.game.selected = 3,
                            KeyCode::Digit5 => self.game.selected = 4,
                            KeyCode::Digit6 => self.game.selected = 5,
                            _ => {
                                self.keys.insert(key);
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                if !self.captured {
                    if button == MouseButton::Left {
                        self.capture_mouse(true);
                    }
                } else {
                    match button {
                        MouseButton::Left => {
                            self.game.dig();
                        }
                        MouseButton::Right => {
                            self.game.place();
                        }
                        _ => (),
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } if self.captured => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                if y != 0.0 {
                    self.game.selected = (self.game.selected as i32 + if y > 0.0 { -1 } else { 1 })
                        .rem_euclid(6) as usize;
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
